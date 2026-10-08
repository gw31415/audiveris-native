// SPDX-License-Identifier: AGPL-3.0-or-later
use super::tuplet_preparation_tests::{Prepared, callback, control, state};
use super::*;
use crate::shared_sig::Key;

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-selected-preparation/observed.json"
    ))
    .unwrap();
    let frames = d["frame_sets"].as_array().unwrap().clone();
    let clocks = d["clock_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for ph in c["phases"].as_array_mut().unwrap() {
            ph["state"] = frames[ph["state"].as_u64().unwrap() as usize].clone();
        }
        for cb in c["callbacks"].as_array_mut().unwrap() {
            cb["clocks"] = clocks[cb["clocks"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
fn observed(p: &mut Prepared, slots: &BTreeMap<Key, Option<Arc<()>>>) -> Value {
    let mut v = state(p);
    for row in v["state"]["owned"]["chords"].as_array_mut().unwrap() {
        let key =
            p.c.owner
                .chords
                .iter()
                .find(|ch| p.c.labels[&ch.id] == row["label"].as_str().unwrap())
                .unwrap()
                .key;
        row["same_baseline_slot"] = json!(match (
            p.c.owner.owner.graph.chord_slot_identity(key),
            &slots[&key]
        ) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        });
    }
    v
}
fn normalized(mut v: Value) -> Value {
    v.as_object_mut().unwrap().remove("phase");
    v.as_object_mut().unwrap().remove("same_expected_object");
    v["state"]["owned"]["relations"] = json!([]);
    v
}
fn append(p: &mut Prepared) {
    let voices =
        p.c.owner
            .voices
            .iter()
            .filter(|v| v.measure == 0)
            .cloned()
            .collect::<Vec<_>>();
    assert_eq!(voices.len(), 2);
    for voice in voices {
        assert!(
            !p.c.refs
                .iter()
                .any(|old| Arc::ptr_eq(&old.identity, &voice.identity))
        );
        p.c.refs.push(voice);
    }
}
fn slots(p: &Prepared) -> BTreeMap<Key, Option<Arc<()>>> {
    p.c.owner
        .chords
        .iter()
        .map(|ch| {
            (
                ch.key,
                p.c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
            )
        })
        .collect()
}
#[test]
fn original_complete_preparation_keeps_callback_clocks_then_rebuilds_before_system_refinement() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let slots = slots(&p);
        let physical = p.c.owner.chords.as_ptr();
        let target_slots = p.c.owner.measures[1].timing.slots.as_ptr();
        let staffs = p.c.owner.owner.staffs.clone();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        p.c.owner.owner.graph.trace_tuplet_removals();
        for ph in case["phases"].as_array().unwrap() {
            match ph["phase"].as_str().unwrap() {
                "before_preparation" => {}
                "stack_rhythm" => {
                    p.c.owner.reprocess_stack_rhythm(1).unwrap();
                    append(&mut p);
                }
                "refined" => refine_ordinary_system(
                    &mut p.c.owner.owner.graph,
                    &mut p.c.owner.voices,
                    &p.c.owner.measures,
                    1,
                )
                .unwrap(),
                _ => unreachable!(),
            }
            assert_eq!(
                observed(&mut p, &slots),
                normalized(ph.clone()),
                "{} {}",
                case["name"],
                ph["phase"]
            );
            assert_eq!(ph["same_expected_object"], true);
            assert_eq!(p.c.owner.measures[0].timing.expected, expected);
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[1].timing.slots.as_ptr(), target_slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            for (a, b) in staffs.iter().zip(&p.c.owner.owner.staffs) {
                assert!(Arc::ptr_eq(a, b));
                for (a, b) in a.lines.iter().zip(&b.lines) {
                    assert!(Arc::ptr_eq(a, b));
                }
            }
        }
        let trace = p.c.owner.owner.graph.take_tuplet_removals();
        let observations = trace
            .iter()
            .filter(|cb| cb.phase == "finished")
            .map(|cb| callback(&p, cb, &slots))
            .collect::<Vec<_>>();
        assert_eq!(json!(observations), case["callbacks"]);
        for cb in &trace {
            assert_eq!(
                callback(&p, cb, &slots)["clocks"],
                case["callbacks"][0]["clocks"]
            );
        }
    }
}
#[test]
fn selected_late_target_failure_restores_actual_implicit_tuplets_old_slots_and_chords() {
    let d = original();
    let case = &d["cases"][0];
    let mut p = control(case);
    let slots = slots(&p);
    let expected = p.c.owner.measures[1].timing.expected.take();
    let before = observed(&mut p, &slots);
    let graph = p.c.owner.owner.graph.snapshot();
    let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
    let physical = p.c.owner.chords.as_ptr();
    let storage =
        p.c.owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>();
    let voices = projection(&p.c.owner.voices, &p.c.owner.measures);
    p.c.owner.owner.graph.trace_tuplet_removals();
    assert_eq!(
        p.c.owner
            .reprocess_selected_stack_rhythms(&[1, 2, 1])
            .unwrap_err(),
        "music_reprocess_unknown_expected_export_pending"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(observed(&mut p, &slots), before);
    assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(
        p.c.owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>(),
        storage
    );
    assert_eq!(projection(&p.c.owner.voices, &p.c.owner.measures), voices);
    assert!(p.c.owner.owner.graph.take_tuplet_removals().is_empty());
    p.c.owner.measures[1].timing.expected = expected;
    p.c.owner.reprocess_selected_stack_rhythms(&[1, 1]).unwrap();
    append(&mut p);
    assert_eq!(
        observed(&mut p, &slots),
        normalized(case["phases"][2].clone())
    );
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
}
