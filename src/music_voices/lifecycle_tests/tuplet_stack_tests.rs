// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::{music_font_symbols::TupletFontSymbols, shared_sig::Key};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-stack/observed.json"
    ))
    .unwrap();
    let relations = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] =
            relations[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    for f in d["clock_sets"].as_array_mut().unwrap() {
        f["relations"] = relations[f["relations"].as_u64().unwrap() as usize].clone();
    }
    let frames = d["frame_sets"].as_array().unwrap().clone();
    let clocks = d["clock_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for s in c["scenarios"].as_array_mut().unwrap() {
            for k in ["before", "generated", "removed"] {
                s[k] = frames[s[k].as_u64().unwrap() as usize].clone();
            }
            for cb in s["callbacks"].as_array_mut().unwrap() {
                cb["clocks"] = clocks[cb["clocks"].as_u64().unwrap() as usize].clone();
            }
        }
    }
    d
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
fn declare(p: &mut Prepared, s: &Value) -> Key {
    let row = &s["geometry"][0];
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
    key
}
fn containers(p: &Prepared, key: Key) -> Value {
    json!(
        p.c.owner
            .measures
            .iter()
            .filter(|m| m.system == key.system)
            .map(|m| {
                let measure = m.tuplets.contains(&key);
                let unassigned = m.stack_tuplets.contains(&key);
                json!({"measure":measure,"stack":measure||unassigned,"unassigned":unassigned})
            })
            .collect::<Vec<_>>()
    )
}
#[test]
fn original_stack_abscissa_null_and_other_stack_refs_survive_generator_measure_removal() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots =
            p.c.owner
                .measures
                .iter()
                .map(|m| m.timing.slots.as_ptr())
                .collect::<Vec<_>>();
        let report = p.c.owner.owner.prepared.report.clone();
        assert_eq!(
            json!(
                p.c.owner
                    .measures
                    .iter()
                    .map(|m| json!({"left":m.left,"right":m.right}))
                    .collect::<Vec<_>>()
            ),
            case["ranges"]
        );
        for s in case["scenarios"].as_array().unwrap() {
            let chord = declare(&mut p, s);
            let before = frame(&mut p.c);
            assert_eq!(before, normalized(s["before"].clone()));
            let font = TupletFontSymbols::load("Bravura", p.c.owner.owner.interline).unwrap();
            p.c.owner.owner.graph.trace_tuplet_additions();
            let key =
                p.c.owner
                    .generate_rhythm_tuplet(1, &[chord], &font)
                    .unwrap();
            let got = p.c.owner.owner.graph.take_tuplet_additions().iter().map(|cb| {
                let held=std::mem::replace(&mut p.c.owner.owner.graph,cb.graph.clone());
                let clocks=frame(&mut p.c)["owned"].clone();
                p.c.owner.owner.graph=held;
                let snapshot=cb.graph.snapshot();let id=cb.graph.id(key).unwrap();
                let n=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==id).unwrap();
                json!({"phase":cb.phase,"sig_attached":cb.sig_attached,"abnormal":n["abnormal"]==true,"containers":cb.containers.iter().map(|v|json!({"measure":v[0],"stack":v[1],"unassigned":v[2]})).collect::<Vec<_>>(),"clocks":clocks})
            }).collect::<Vec<_>>();
            let mut expected = s["callbacks"].clone();
            for cb in expected.as_array_mut().unwrap() {
                cb["clocks"]["relations"] = json!([]);
            }
            assert_eq!(
                json!(got),
                expected,
                "{} x={} callbacks",
                case["name"],
                s["x"]
            );
            let snapshot = p.c.owner.owner.graph.snapshot();
            let id = p.c.owner.owner.graph.id(key).unwrap();
            let n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .unwrap();
            assert_eq!(n["bounds"], s["bounds"]);
            assert_eq!(containers(&p, key), s["generated_containers"]);
            assert_eq!(frame(&mut p.c), normalized(s["generated"].clone()));
            if s["staff_warmed_before_remove"] == true {
                let staff =
                    p.c.owner
                        .owner
                        .graph
                        .tuplet_staff(key, &p.c.owner.owner.staffs)
                        .unwrap()
                        .unwrap();
                assert!(Arc::ptr_eq(&staff, &p.c.owner.owner.staffs[0]));
            }
            p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
            assert_eq!(containers(&p, key), s["removed_containers"]);
            assert_eq!(
                p.c.owner.owner.graph.tuplet_registration(key)["removed"],
                s["removed_flag"]
            );
            assert_eq!(s["registered_after_remove"], false);
            assert_eq!(frame(&mut p.c), normalized(s["removed"].clone()));
            // The observer separately cleans other Stack refs between controls.
            // Native deliberately retains them here: removal of this Measure
            // must not normalize unrelated actual Stack ownership.
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            for (m, ptr) in p.c.owner.measures.iter().zip(&slots) {
                assert_eq!(m.timing.slots.as_ptr(), *ptr);
            }
        }
    }
}
#[test]
fn late_other_stack_addition_failure_restores_both_actual_container_storages() {
    let d = original();
    let case = &d["cases"][0];
    let mut p = control(case);
    let s = case["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["located"] == 1)
        .unwrap();
    let ch = declare(&mut p, s);
    let font = TupletFontSymbols::load("Bravura", p.c.owner.owner.interline).unwrap();
    let retired = p.c.owner.generate_rhythm_tuplet(1, &[ch], &font).unwrap();
    p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
    assert!(p.c.owner.measures[1].stack_tuplets.contains(&retired));
    let before = frame(&mut p.c);
    let physical = p.c.owner.chords.as_ptr();
    let storage =
        p.c.owner
            .measures
            .iter()
            .map(|m| {
                (
                    m.tuplets.as_ptr(),
                    m.stack_tuplets.as_ptr(),
                    m.timing.slots.as_ptr(),
                )
            })
            .collect::<Vec<_>>();
    p.c.owner.owner.graph.trace_tuplet_additions();
    p.c.owner.owner.graph.fail_tuplet_addition_after(Some(1));
    let graph = p.c.owner.owner.graph.snapshot();
    let font = TupletFontSymbols::load("Bravura", p.c.owner.owner.interline).unwrap();
    assert_eq!(
        p.c.owner
            .generate_rhythm_tuplet(1, &[ch], &font)
            .unwrap_err(),
        "music_tuplet_addition_observer_failure"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    assert_eq!(frame(&mut p.c), before);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    for (m, (tuplets, stack, slots)) in p.c.owner.measures.iter().zip(storage) {
        assert_eq!(m.tuplets.as_ptr(), tuplets);
        assert_eq!(m.stack_tuplets.as_ptr(), stack);
        assert_eq!(m.timing.slots.as_ptr(), slots);
    }
    p.c.owner.owner.graph.fail_tuplet_addition_after(None);
    let key = p.c.owner.generate_rhythm_tuplet(1, &[ch], &font).unwrap();
    assert_eq!(containers(&p, key), s["generated_containers"]);
    p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
    assert_eq!(containers(&p, key), s["removed_containers"]);
}
