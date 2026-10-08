// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame, prepare_control};
use super::beam_rest_linked_tests::{chord_key, declare_links, queries};
use super::*;
use crate::shared_sig::Key;

fn original() -> Value {
    let mut observed: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-plain/observed.json"
    ))
    .unwrap();
    let relations = observed["relation_sets"].as_array().unwrap().clone();
    let mut owned = observed["owned_sets"].as_array().unwrap().clone();
    for state in &mut owned {
        state["relations"] = relations[state["relations"].as_u64().unwrap() as usize].clone();
    }
    for case in observed["cases"].as_array_mut().unwrap() {
        for step in case["states"].as_array_mut().unwrap() {
            step["state"]["owned"] =
                owned[step["state"]["owned"].as_u64().unwrap() as usize].clone();
        }
    }
    observed
}
fn add_manual(c: &mut Control, rest: usize) -> usize {
    let snapshot = c.owner.owner.graph.snapshot();
    let beam: Key = serde_json::from_value(
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == c.beam)
            .unwrap()["origin"]
            .clone(),
    )
    .unwrap();
    let chord = c
        .owner
        .chords
        .iter()
        .find(|ch| ch.key == c.rests[rest])
        .unwrap();
    let note: Key = serde_json::from_value(chord.members[0]["origin"].clone()).unwrap();
    c.owner
        .owner
        .graph
        .add_owned_beam_rest(
            beam,
            note,
            true,
            c.owner.owner.sheet_skew,
            c.owner.owner.interline,
        )
        .unwrap()
}
fn plain(c: &mut Control) -> Value {
    let groups = c.owner.interleaved_rest_candidates(1).unwrap();
    assert_eq!(groups.as_array().unwrap().len(), 1);
    let row = &groups[0];
    assert_eq!(row["rest_detection_complete"], false);
    json!({"measure_numbers":row["measure_numbers"],"plain":row["plain_rest_ids"].as_array().unwrap().iter()
        .map(|id|c.labels[&(id.as_u64().unwrap() as usize)].clone()).collect::<Vec<_>>()})
}

#[test]
fn original_plain_candidates_follow_real_group_measures_and_rest_insertion_order_without_assigning_voices()
 {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        let mut links = declare_links(&mut c, case);
        let chord_storage = c.owner.chords.as_ptr();
        let slot_storage = c
            .owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>();
        let report = c.owner.owner.prepared.report.clone();
        let target_voices = c
            .owner
            .voices
            .iter()
            .filter(|v| v.measure == 1)
            .map(|v| v.identity.clone())
            .collect::<Vec<_>>();
        for step in case["states"].as_array().unwrap() {
            match step["phase"].as_str().unwrap() {
                "linked" => {}
                "manual_beam_rest" => {
                    let id = add_manual(&mut c, 0);
                    links.push((id, "manual_beam_rest".into()));
                }
                "manual_foreign" => {
                    let id = add_manual(&mut c, 5);
                    links.push((id, "foreign_manual_beam_rest".into()));
                }
                "cleared" => c.owner.clear_interleaved_rests(1).unwrap(),
                "measure_reset" => reset_rhythm(
                    &mut c.owner.owner.graph,
                    &mut c.owner.voices,
                    &mut c.owner.measures[0],
                    &c.owner.chords,
                    0,
                )
                .unwrap(),
                _ => unreachable!(),
            }
            let before = c.owner.owner.graph.snapshot();
            let before_state = frame(&mut c);
            let slots = c
                .owner
                .chords
                .iter()
                .map(|ch| {
                    (
                        ch.key,
                        c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
                    )
                })
                .collect::<Vec<_>>();
            let mut actual_queries = queries(&mut c, case["seed"].as_str().unwrap(), &links);
            let plain = plain(&mut c);
            actual_queries["measure_numbers"] = plain["measure_numbers"].clone();
            actual_queries["plain"] = plain["plain"].clone();
            assert_eq!(
                actual_queries, step["queries"],
                "{} {}",
                case["name"], step["phase"]
            );
            assert_eq!(c.owner.owner.graph.snapshot(), before);
            assert_eq!(frame(&mut c), before_state);
            let mut actual = frame(&mut c);
            let mut expected = step["state"].clone();
            expected["owned"]["relations"]
                .as_array_mut()
                .unwrap()
                .retain(|row| {
                    if row["label"].as_str().unwrap().starts_with("relation") {
                        assert_eq!(
                            row,
                            case["states"][0]["state"]["owned"]["relations"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .find(|old| old["label"] == row["label"])
                                .unwrap()
                        );
                        false
                    } else {
                        true
                    }
                });
            actual["owned"]["relations"] = json!(
                links
                    .iter()
                    .map(|(id, label)| json!({"label":label,"registered":true,
                "manual":c.owner.owner.graph.edge_manual_status(*id).unwrap()}))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                actual, expected,
                "{} {} actual owners",
                case["name"], step["phase"]
            );
            assert_eq!(c.owner.chords.as_ptr(), chord_storage);
            assert_eq!(c.owner.owner.prepared.report, report);
            for (measure, old) in c.owner.measures.iter().zip(&slot_storage) {
                assert_eq!(measure.timing.slots.as_ptr(), *old);
            }
            for identity in &target_voices {
                assert!(
                    c.owner
                        .voices
                        .iter()
                        .any(|v| Arc::ptr_eq(&v.identity, identity))
                );
            }
            for (key, old) in slots {
                match (old, c.owner.owner.graph.chord_slot_identity(key)) {
                    (Some(old), Some(now)) => assert!(Arc::ptr_eq(&old, now)),
                    (None, None) => {}
                    _ => panic!("candidate query changed actual Slot"),
                }
            }
        }
    }
}

#[test]
fn plain_candidate_late_missing_foreign_owner_refuses_instead_of_silently_omitting_actual_rest() {
    let observed = original();
    let case = &observed["cases"][0];
    let mut c = prepare_control(case);
    declare_links(&mut c, case);
    add_manual(&mut c, 5);
    let index = c
        .owner
        .chords
        .iter()
        .position(|ch| ch.key == chord_key(&c, "rest_chord8"))
        .unwrap();
    let held = c.owner.chords.remove(index);
    let snapshot = c.owner.owner.graph.snapshot();
    let layout = c.owner.layout();
    let voices = projection(&c.owner.voices, &c.owner.measures);
    let slots = c
        .owner
        .chords
        .iter()
        .map(|ch| {
            (
                ch.key,
                c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
            )
        })
        .collect::<Vec<_>>();
    let storage = c.owner.chords.as_ptr();
    let report = c.owner.owner.prepared.report.clone();
    assert_eq!(
        c.owner.interleaved_rest_candidates(1).unwrap_err(),
        "music_rest_actual_chord_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), snapshot);
    assert_eq!(c.owner.layout(), layout);
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), voices);
    assert_eq!(c.owner.chords.as_ptr(), storage);
    assert_eq!(c.owner.owner.prepared.report, report);
    for (key, old) in slots {
        match (old, c.owner.owner.graph.chord_slot_identity(key)) {
            (Some(old), Some(now)) => assert!(Arc::ptr_eq(&old, now)),
            (None, None) => {}
            _ => panic!("failed query changed Slot"),
        }
    }
    for (key, old) in c.rests.iter().zip(&c.baseline) {
        assert!(Arc::ptr_eq(
            &c.owner.owner.graph.rest_chord_beams(*key).unwrap(),
            old
        ));
    }
    c.owner.chords.insert(index, held);
    let result = plain(&mut c);
    assert_eq!(result["measure_numbers"], json!([1, 2]));
    assert_eq!(
        result["plain"],
        json!(["rest_chord6", "rest_chord7", "rest_chord8"])
    );
}
