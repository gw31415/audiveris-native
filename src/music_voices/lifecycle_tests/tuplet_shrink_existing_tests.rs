// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::music_font_symbols::TupletFontSymbols;

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-shrink-existing/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
fn set_time(p: &mut Prepared, key: crate::shared_sig::Key, value: &Value) {
    let snap = p.c.owner.owner.graph.snapshot();
    let mut n = snap["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == p.c.owner.owner.graph.id(key).unwrap())
        .unwrap()
        .clone();
    n["time_offset"] = value.clone();
    p.c.owner.owner.graph.vertex(key, n, false).unwrap();
}
fn clocks(candidates: &crate::music_xml::RhythmCandidates) -> Value {
    let mut rows = Vec::new();
    for slot in &candidates.compounds {
        for narrow in &slot.members {
            rows.push(json!({"label":format!("narrow{}",narrow.id-1),"id":narrow.id,"time":candidates.clocks.get_owner(narrow).unwrap().map(crate::music_time::format_fraction)}));
        }
    }
    json!(rows)
}
#[test]
fn original_existing_shrink_binds_real_candidates_then_pushes_exact_chords_before_narrow_clocks() {
    for case in original()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"] != "short_late_stack")
    {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let stack_slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        let mut candidates = p.c.owner.bind_rhythm_candidates(1).unwrap();
        let ns = candidates.compounds.iter().flat_map(|co|&co.members).map(|n|json!({"label":format!("narrow{}",n.id-1),"id":n.id,"chords":n.chord_ids.iter().map(|id|p.c.labels[id].clone()).collect::<Vec<_>>()})).collect::<Vec<_>>();
        let cs = candidates.compounds.iter().enumerate().map(|(i,co)|json!({"label":format!("compound{i}"),"id":co.id,"members":co.members.iter().map(|n|format!("narrow{}",n.id-1)).collect::<Vec<_>>(),"chords":co.chord_ids.iter().map(|id|p.c.labels[id].clone()).collect::<Vec<_>>()})).collect::<Vec<_>>();
        assert_eq!(json!(ns), case["narrow"]);
        assert_eq!(json!(cs), case["compounds"]);
        let seed = chord_key(&p.c, case["seed"].as_str().unwrap());
        set_time(&mut p, seed, &case["seed_time"]);
        let mut source = case["phases"][0]["state"].clone();
        source["owned"]["relations"] = json!([]);
        assert_eq!(frame(&mut p.c), source);
        assert_eq!(clocks(&candidates), case["phases"][0]["narrow_clocks"]);
        let graph = p.c.owner.owner.graph.snapshot();
        let stop = case["stop"].as_str().map(|name| chord_key(&p.c, name));
        let identity = p.c.refs[0].identity.clone();
        let wrong_font = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        let result = p.c.owner.shrink_rhythm_voice(
            &mut candidates,
            &identity,
            stop,
            case["last_sync"].as_str(),
            &wrong_font,
        );
        if case["error"].is_null() {
            let keys = result.unwrap();
            assert!(!keys.is_empty());
            for key in keys {
                assert!(p.tuplets.contains(&key));
            }
        } else {
            assert_eq!(case["error"], "NullPointerException");
            assert_eq!(result.unwrap_err(), "music_tuplet_null_shrink_chord_time");
            assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
        }
        assert_eq!(case["graph_changes"], 0);
        assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
        let mut source = case["phases"][1]["state"].clone();
        source["owned"]["relations"] = json!([]);
        assert_eq!(frame(&mut p.c), source);
        assert_eq!(clocks(&candidates), case["phases"][1]["narrow_clocks"]);
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), stack_slots);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}
#[test]
fn late_real_stack_slot_failure_rolls_back_actual_chords_candidates_and_manual_tuplets_then_retries()
 {
    let d = original();
    let case = d["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "short_late_stack")
        .unwrap();
    assert_eq!(case["error"], "ClassCastException");
    assert_eq!(case["phases"][1]["narrow_clocks"][1]["time"], "1/12");
    assert_eq!(case["phases"][1]["narrow_clocks"][2]["time"], "1/6");
    assert_ne!(case["phases"][0]["state"], case["phases"][1]["state"]);
    let mut p = control(case);
    let mut candidates = p.c.owner.bind_rhythm_candidates(1).unwrap();
    let late = chord_key(&p.c, "chord5");
    let old = p.c.chord_slots[&late].as_ref().unwrap().clone();
    let slot = p.c.owner.measures[0]
        .timing
        .slots
        .iter()
        .find(|s| Arc::ptr_eq(&s.identity, &old))
        .unwrap();
    let old_id = slot.id;
    p.c.owner
        .owner
        .graph
        .set_chord_slot(late, old_id, &old)
        .unwrap();
    let mut source = case["phases"][0]["state"].clone();
    source["owned"]["relations"] = json!([]);
    assert_eq!(frame(&mut p.c), source);
    let graph = p.c.owner.owner.graph.snapshot();
    let before = candidates.observe().unwrap();
    let chord_storage = p.c.owner.chords.as_ptr();
    let stack_storage = p.c.owner.measures[0].timing.slots.as_ptr();
    let tuplet_storage = p.c.owner.measures[0].tuplets.as_ptr();
    let identity = p.c.refs[0].identity.clone();
    let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
    assert_eq!(
        p.c.owner
            .shrink_rhythm_voice(&mut candidates, &identity, None, Some("0"), &wrong)
            .unwrap_err(),
        "music_tuplet_actual_compound_slot_required"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(candidates.observe().unwrap(), before);
    assert_eq!(frame(&mut p.c), source);
    assert_eq!(p.c.owner.chords.as_ptr(), chord_storage);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), stack_storage);
    assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplet_storage);
    let id = p.c.owner.owner.graph.id(late).unwrap();
    let compound = candidates
        .compounds
        .iter()
        .find(|s| s.chord_ids.contains(&id))
        .unwrap();
    p.c.owner
        .owner
        .graph
        .set_chord_slot(late, compound.id, &compound.identity)
        .unwrap();
    let keys =
        p.c.owner
            .shrink_rhythm_voice(&mut candidates, &identity, None, Some("0"), &wrong)
            .unwrap();
    assert!(keys.iter().all(|key| p.tuplets.contains(key)));
    let whole = &d["cases"][0]["phases"][1];
    let mut source = whole["state"].clone();
    source["owned"]["relations"] = json!([]);
    assert_eq!(frame(&mut p.c), source);
    assert_eq!(clocks(&candidates), whole["narrow_clocks"]);
}
