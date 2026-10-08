// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control};
use super::tuplet_shrink_existing_tests::{clocks, set_time};
use super::*;
use crate::{music_font_symbols::TupletFontSymbols, music_xml::RhythmCandidates, shared_sig::Key};
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-shrink-fresh/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    for f in d["clock_sets"].as_array_mut().unwrap() {
        f["relations"] = rs[f["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    let cs = d["clock_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
        for cb in c["callbacks"].as_array_mut().unwrap() {
            cb["clocks"] = cs[cb["clocks"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
fn prepare(case: &Value) -> (Prepared, RhythmCandidates) {
    let mut p = control(case);
    let candidates = p.c.owner.bind_rhythm_candidates(1).unwrap();
    for row in case["geometry"].as_array().unwrap() {
        let key = chord_key(&p.c, row["chord"].as_str().unwrap());
        p.c.owner
            .owner
            .graph
            .declare_chord_geometry(
                key,
                serde_json::from_value(row["bounds"].clone()).unwrap(),
                serde_json::from_value(row["head"].clone()).unwrap(),
                serde_json::from_value(row["tail"].clone()).unwrap(),
            )
            .unwrap();
    }
    let seed = chord_key(&p.c, case["seed"].as_str().unwrap());
    set_time(&mut p, seed, &case["seed_time"]);
    (p, candidates)
}
fn normalized(mut value: Value) -> Value {
    value["owned"]["relations"] = json!([]);
    value
}
fn callback_clocks(value: &Value) -> Value {
    json!(value.as_array().unwrap().iter().flat_map(|rows|rows.as_array().unwrap()).map(|r|json!({"label":format!("narrow{}",r["id"].as_u64().unwrap()-1),"id":r["id"],"time":r["time_offset"]})).collect::<Vec<_>>())
}
#[test]
fn original_fresh_shrink_callbacks_hold_actual_old_clocks_before_exact_chord_and_narrow_updates() {
    for case in original()["cases"].as_array().unwrap() {
        let (mut p, mut candidates) = prepare(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        assert_eq!(
            frame(&mut p.c),
            normalized(case["phases"][0]["state"].clone())
        );
        assert_eq!(clocks(&candidates), case["phases"][0]["narrow_clocks"]);
        let before = p.c.owner.owner.graph.snapshot();
        let identity = p.c.refs[0].identity.clone();
        let stop = case["stop"].as_str().map(|s| chord_key(&p.c, s));
        let font =
            TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
                .unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        let result = p.c.owner.shrink_rhythm_voice(
            &mut candidates,
            &identity,
            stop,
            case["last_sync"].as_str(),
            &font,
        );
        let keys = if case["error"].is_null() {
            result.unwrap()
        } else {
            assert_eq!(case["error"], "NullPointerException");
            assert_eq!(result.unwrap_err(), "music_tuplet_null_shrink_chord_time");
            assert_eq!(p.c.owner.owner.graph.snapshot(), before);
            Vec::new()
        };
        let callbacks = p.c.owner.owner.graph.take_tuplet_additions();
        let mut generated = Vec::<Key>::new();
        let observed=callbacks.iter().map(|cb|{
   if !generated.contains(&cb.tuplet){generated.push(cb.tuplet);}
   let held=std::mem::replace(&mut p.c.owner.owner.graph,cb.graph.clone());let owned=frame(&mut p.c)["owned"].clone();p.c.owner.owner.graph=held;
   let graph=cb.graph.snapshot();let id=cb.graph.id(cb.tuplet).unwrap();let n=graph["inters"].as_array().unwrap().iter().find(|n|n["id"]==id).unwrap();
   json!({"phase":cb.phase,"tuplet":format!("generated{}",generated.iter().position(|k|*k==cb.tuplet).unwrap()),"sig_attached":cb.sig_attached,"abnormal":n["abnormal"]==true,"measure_contains":cb.measure_tuplets.contains(&cb.tuplet),"stack_contains":cb.stack_tuplets.contains(&cb.tuplet),"clocks":owned,"narrow_clocks":callback_clocks(cb.candidate_clocks.as_ref().unwrap())})
  }).collect::<Vec<_>>();
        let mut expected_callbacks = case["callbacks"].clone();
        for cb in expected_callbacks.as_array_mut().unwrap() {
            cb["clocks"]["relations"] = json!([]);
        }
        assert_eq!(
            json!(observed),
            expected_callbacks,
            "{} callback clocks",
            case["name"]
        );
        assert_eq!(
            json!(
                keys.iter()
                    .map(|k| format!(
                        "generated{}",
                        generated.iter().position(|g| g == k).unwrap()
                    ))
                    .collect::<Vec<_>>()
            ),
            case["generated"]
        );
        assert_eq!(
            frame(&mut p.c),
            normalized(case["phases"][1]["state"].clone()),
            "{} final",
            case["name"]
        );
        assert_eq!(clocks(&candidates), case["phases"][1]["narrow_clocks"]);
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}
#[test]
fn fresh_shrink_relation_callback_failure_rolls_back_actual_candidates_chords_and_container_objects()
 {
    let d = original();
    let case = &d["cases"][0];
    let (mut p, mut candidates) = prepare(case);
    let before = frame(&mut p.c);
    let clocks_before = candidates.observe().unwrap();
    let chord_storage = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
    let expected = p.c.owner.measures[0].timing.expected;
    let identity = p.c.refs[0].identity.clone();
    let font = TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
        .unwrap();
    p.c.owner.owner.graph.trace_tuplet_additions();
    p.c.owner.owner.graph.fail_tuplet_addition_after(Some(2));
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner
            .shrink_rhythm_voice(&mut candidates, &identity, None, Some("0"), &font)
            .unwrap_err(),
        "music_tuplet_addition_observer_failure"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    assert_eq!(frame(&mut p.c), before);
    assert_eq!(candidates.observe().unwrap(), clocks_before);
    assert_eq!(p.c.owner.chords.as_ptr(), chord_storage);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
    assert_eq!(p.c.owner.measures[0].timing.expected, expected);
    p.c.owner.owner.graph.fail_tuplet_addition_after(None);
    let keys =
        p.c.owner
            .shrink_rhythm_voice(&mut candidates, &identity, None, Some("0"), &font)
            .unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(p.c.owner.owner.graph.take_tuplet_additions().len(), 7);
    assert_eq!(
        frame(&mut p.c),
        normalized(case["phases"][1]["state"].clone())
    );
    assert_eq!(clocks(&candidates), case["phases"][1]["narrow_clocks"]);
}
