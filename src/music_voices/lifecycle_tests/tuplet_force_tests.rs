// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::{music_font_symbols::TupletFontSymbols, shared_sig::Key};
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-force/observed.json"
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
        for s in c["scenarios"].as_array_mut().unwrap() {
            s["removed"] = fs[s["removed"].as_u64().unwrap() as usize].clone();
            for p in s["passes"].as_array_mut().unwrap() {
                for k in ["before", "after"] {
                    p[k] = fs[p[k].as_u64().unwrap() as usize].clone();
                }
                for cb in p["callbacks"].as_array_mut().unwrap() {
                    cb["clocks"] = cs[cb["clocks"].as_u64().unwrap() as usize].clone();
                }
            }
        }
    }
    d
}
pub(super) fn prepare(case: &Value) -> Prepared {
    let mut p = control(case);
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
    p
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_force_actual_voice_extends_generates_and_reuses_same_tuplet_results_with_old_clocks() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = prepare(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let font =
            TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
                .unwrap();
        for s in case["scenarios"].as_array().unwrap() {
            let vi = s["voice"]
                .as_str()
                .unwrap()
                .strip_prefix("voice")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let identity = p.c.refs[vi].identity.clone();
            let mut generated = Vec::<Key>::new();
            for pass in s["passes"].as_array().unwrap() {
                assert_eq!(frame(&mut p.c), normalized(pass["before"].clone()));
                p.c.owner.owner.graph.trace_tuplet_additions();
                let result =
                    p.c.owner
                        .force_rhythm_voice_tuplets(
                            1,
                            &identity,
                            case["actual_beat"].as_str().unwrap(),
                            &font,
                        )
                        .unwrap();
                let callbacks = p.c.owner.owner.graph.take_tuplet_additions();
                let observed=callbacks.iter().map(|cb|{
                    if !generated.contains(&cb.tuplet){generated.push(cb.tuplet);}
                    let held=std::mem::replace(&mut p.c.owner.owner.graph,cb.graph.clone());let clocks=frame(&mut p.c)["owned"].clone();p.c.owner.owner.graph=held;
                    let graph=cb.graph.snapshot();let id=cb.graph.id(cb.tuplet).unwrap();let n=graph["inters"].as_array().unwrap().iter().find(|n|n["id"]==id).unwrap();
                    json!({"phase":cb.phase,"tuplet":format!("generated{}",generated.iter().position(|k|*k==cb.tuplet).unwrap()),"sig_attached":cb.sig_attached,"abnormal":n["abnormal"]==true,"measure_contains":cb.measure_tuplets.contains(&cb.tuplet),"stack_contains":cb.stack_tuplets.contains(&cb.tuplet),"clocks":clocks})
                }).collect::<Vec<_>>();
                let mut expected = pass["callbacks"].clone();
                for cb in expected.as_array_mut().unwrap() {
                    cb["clocks"]["relations"] = json!([]);
                }
                assert_eq!(
                    json!(observed),
                    expected,
                    "{} {} pass{} callbacks",
                    case["name"],
                    s["voice"],
                    pass["pass"]
                );
                assert_eq!(
                    json!(
                        result
                            .iter()
                            .map(|key| format!(
                                "generated{}",
                                generated.iter().position(|k| k == key).unwrap()
                            ))
                            .collect::<Vec<_>>()
                    ),
                    pass["returned"]
                );
                let graph = p.c.owner.owner.graph.snapshot();
                let mut rows = Vec::new();
                for (i, key) in generated.iter().enumerate() {
                    let id = p.c.owner.owner.graph.id(*key).unwrap();
                    let n = graph["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == id)
                        .unwrap();
                    let mut chords =
                        p.c.owner
                            .owner
                            .graph
                            .tuplet_chords(*key)
                            .unwrap()
                            .into_iter()
                            .map(|key| {
                                let bounds =
                                    p.c.owner
                                        .owner
                                        .graph
                                        .retained_or_live_chord_bounds(key)
                                        .unwrap();
                                (key, bounds[0])
                            })
                            .collect::<Vec<_>>();
                    chords.sort_by_key(|(_, x)| *x);
                    rows.push(json!({"label":format!("generated{i}"),"shape":n["shape"],"bounds":n["bounds"],"chords":chords.iter().map(|(k,_)|p.c.labels[&p.c.owner.owner.graph.id(*k).unwrap()].clone()).collect::<Vec<_>>(),"manual":n["manual"]==true,"implicit":n["implicit"]==true,"abnormal":n["abnormal"]==true}));
                }
                assert_eq!(json!(rows), pass["tuplets"]);
                assert_eq!(frame(&mut p.c), normalized(pass["after"].clone()));
                assert_eq!(p.c.owner.chords.as_ptr(), physical);
                assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
                assert_eq!(p.c.owner.owner.prepared.report, report);
            }
            p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
            assert_eq!(frame(&mut p.c), normalized(s["removed"].clone()));
        }
    }
}
#[test]
fn later_force_allocation_failure_restores_earlier_completed_tuplet_and_real_owners() {
    let d = original();
    let case = &d["cases"][0];
    let mut p = prepare(case);
    let identity = p.c.refs[1].identity.clone();
    let before = frame(&mut p.c);
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let storage =
        p.c.owner
            .measures
            .iter()
            .map(|m| (m.tuplets.as_ptr(), m.stack_tuplets.as_ptr()))
            .collect::<Vec<_>>();
    let font = TupletFontSymbols::load("Bravura", p.c.owner.owner.interline).unwrap();
    p.c.owner.owner.graph.trace_tuplet_additions();
    p.c.owner
        .owner
        .graph
        .fail_tuplet_registration_after(Some(2));
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner
            .force_rhythm_voice_tuplets(1, &identity, "1/4", &font)
            .unwrap_err(),
        "music_tuplet_addition_observer_failure"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    assert_eq!(frame(&mut p.c), before);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    for (m, (tuplets, stack)) in p.c.owner.measures.iter().zip(storage) {
        assert_eq!(m.tuplets.as_ptr(), tuplets);
        assert_eq!(m.stack_tuplets.as_ptr(), stack);
    }
    p.c.owner.owner.graph.fail_tuplet_registration_after(None);
    let keys =
        p.c.owner
            .force_rhythm_voice_tuplets(1, &identity, "1/4", &font)
            .unwrap();
    assert_eq!(keys.len(), 3);
    p.c.owner.owner.graph.take_tuplet_additions();
    p.c.owner.owner.graph.trace_tuplet_additions();
    let repeated_storage =
        p.c.owner
            .measures
            .iter()
            .map(|m| (m.tuplets.as_ptr(), m.stack_tuplets.as_ptr()))
            .collect::<Vec<_>>();
    let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
    assert_eq!(
        p.c.owner
            .force_rhythm_voice_tuplets(1, &identity, "1/4", &wrong)
            .unwrap(),
        keys
    );
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    for (m, (tuplets, stack)) in p.c.owner.measures.iter().zip(repeated_storage) {
        assert_eq!(m.tuplets.as_ptr(), tuplets);
        assert_eq!(m.stack_tuplets.as_ptr(), stack);
    }
    p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
    assert_eq!(frame(&mut p.c), before);
}
