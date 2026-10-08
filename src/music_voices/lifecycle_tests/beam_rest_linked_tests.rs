// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame, prepare_control};
use super::*;
use crate::music_xml::beam_rest::linked_rest_ids;
use crate::shared_sig::{Key, Phase};

fn original() -> Value {
    let mut observed: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-linked/observed.json"
    ))
    .unwrap();
    let relations = observed["relation_sets"].as_array().unwrap().clone();
    for case in observed["cases"].as_array_mut().unwrap() {
        for step in case["states"].as_array_mut().unwrap() {
            let index = step["state"]["owned"]["relations"].as_u64().unwrap() as usize;
            step["state"]["owned"]["relations"] = relations[index].clone();
        }
    }
    observed
}
pub(super) fn chord_key(c: &Control, label: &str) -> Key {
    c.owner
        .chords
        .iter()
        .find(|ch| c.labels.get(&ch.id).is_some_and(|name| name == label))
        .unwrap()
        .key
}
pub(super) fn declare_links(c: &mut Control, case: &Value) -> Vec<(usize, String)> {
    let endpoint = |s: &str| {
        match s {
            "seed" => case["seed"].as_str().unwrap(),
            "outside" => case["outside"].as_str().unwrap(),
            _ => s,
        }
        .to_owned()
    };
    let mut registered = Vec::new();
    for (i, spec) in case["declared_links"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let source = chord_key(c, &endpoint(spec["source"].as_str().unwrap()));
        let target = chord_key(c, &endpoint(spec["target"].as_str().unwrap()));
        c.owner
            .owner
            .graph
            .relations(
                1,
                Phase::Chord,
                vec![(
                    source,
                    target,
                    json!({"kind":spec["kind"],"manual":spec["manual"]}),
                )],
            )
            .unwrap();
        let id = c.owner.owner.graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["kind"] == spec["kind"]
                    && e["source"] == c.owner.owner.graph.id(source).unwrap()
                    && e["target"] == c.owner.owner.graph.id(target).unwrap()
            })
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        registered.push((id, format!("link{i}")));
    }
    registered
}
fn add_manual(c: &mut Control) -> usize {
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
        .find(|ch| ch.key == c.rests[0])
        .unwrap();
    let rest = chord.members[0]["id"].as_u64().unwrap() as usize;
    let note: Key = serde_json::from_value(
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == rest)
            .unwrap()["origin"]
            .clone(),
    )
    .unwrap();
    c.owner
        .owner
        .graph
        .relations(
            1,
            Phase::Symbol,
            vec![(
                beam,
                note,
                json!({"kind":"BeamRestRelation","manual":true,"beam_portion":"CENTER"}),
            )],
        )
        .unwrap();
    c.owner.owner.graph.snapshot()["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "BeamRestRelation" && e["target"] == rest)
        .unwrap()["id"]
        .as_u64()
        .unwrap() as usize
}
pub(super) fn queries(c: &mut Control, seed: &str, links: &[(usize, String)]) -> Value {
    let groups = c.owner.interleaved_rest_candidates(1).unwrap();
    assert_eq!(groups.as_array().unwrap().len(), 1);
    let row = &groups[0];
    assert_eq!(row["rest_detection_complete"], false);
    let seeds = row["seed_chord_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect::<Vec<_>>();
    let names = |ids: Vec<usize>| {
        json!(
            ids.iter()
                .map(|id| c.labels[id].clone())
                .collect::<Vec<_>>()
        )
    };
    let linked = |transitive, kinds: &[&str]| {
        names(
            linked_rest_ids(
                &c.owner.owner.graph,
                &c.owner.chords,
                &seeds,
                transitive,
                kinds,
            )
            .unwrap(),
        )
    };
    let mut result = json!({"seeds":names(seeds.clone()),"next_direct":linked(false,&["NextInVoiceRelation"]),
        "next_transitive":linked(true,&["NextInVoiceRelation"]),"same_transitive":linked(true,&["SameVoiceRelation"]),
        "mixed_transitive":linked(true,&["NextInVoiceRelation","SameVoiceRelation"]),"separate_direct":linked(false,&["SeparateVoiceRelation"]),
        "separate_transitive":linked(true,&["SeparateVoiceRelation"]),"white":names(row["white_rest_ids"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as usize).collect())});
    let id = c.owner.owner.graph.id(chord_key(c, seed)).unwrap();
    let snapshot = c.owner.owner.graph.snapshot();
    let edges = snapshot["relations"].as_array().unwrap();
    result["seed_next_relations"] = json!(
        edges
            .iter()
            .filter(|e| e["target"] == id)
            .chain(
                edges
                    .iter()
                    .filter(|e| e["source"] == id && e["target"] != id)
            )
            .filter(|e| e["kind"] == "NextInVoiceRelation")
            .map(|e| links.iter().find(|(i, _)| e["id"] == *i).unwrap().1.clone())
            .collect::<Vec<_>>()
    );
    result
}

#[test]
fn original_linked_rests_keep_incoming_order_rest_only_closure_and_actual_foreign_measure_refs() {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        let mut links = declare_links(&mut c, case);
        let base = c.owner.owner.graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| !links.iter().any(|(id, _)| e["id"] == *id))
            .cloned()
            .collect::<Vec<_>>();
        let chord_storage = c.owner.chords.as_ptr();
        let report = c.owner.owner.prepared.report.clone();
        for step in case["states"].as_array().unwrap() {
            match step["phase"].as_str().unwrap() {
                "linked" => {}
                "manual_beam_rest" => {
                    let id = add_manual(&mut c);
                    links.push((id, "manual_beam_rest".into()));
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
            let before = frame(&mut c);
            let before_graph = c.owner.owner.graph.snapshot();
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
            assert_eq!(
                queries(&mut c, case["seed"].as_str().unwrap(), &links),
                step["queries"],
                "{} {}",
                case["name"],
                step["phase"]
            );
            assert_eq!(frame(&mut c), before);
            assert_eq!(c.owner.owner.graph.snapshot(), before_graph);
            assert_eq!(c.owner.chords.as_ptr(), chord_storage);
            assert_eq!(c.owner.owner.prepared.report, report);
            for (key, slot) in slots {
                match (slot, c.owner.owner.graph.chord_slot_identity(key)) {
                    (Some(old), Some(actual)) => assert!(Arc::ptr_eq(&old, actual)),
                    (None, None) => {}
                    _ => panic!("query changed actual Slot reference"),
                }
            }
            let mut actual = frame(&mut c);
            let mut expected = step["state"].clone();
            let old_rows = &case["states"][0]["state"]["owned"]["relations"];
            expected["owned"]["relations"]
                .as_array_mut()
                .unwrap()
                .retain(|row| {
                    let name = row["label"].as_str().unwrap();
                    if name.starts_with("relation") {
                        assert_eq!(
                            row,
                            old_rows
                                .as_array()
                                .unwrap()
                                .iter()
                                .find(|r| r["label"] == row["label"])
                                .unwrap()
                        );
                        false
                    } else {
                        true
                    }
                });
            actual["owned"]["relations"]=json!(links.iter().map(|(id,label)|json!({"label":label,"registered":true,"manual":c.owner.owner.graph.edge_manual_status(*id).unwrap()})).collect::<Vec<_>>());
            assert_eq!(
                actual, expected,
                "{} {} owned intermediate state",
                case["name"], step["phase"]
            );
            let snapshot = c.owner.owner.graph.snapshot();
            let untouched = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| !links.iter().any(|(id, _)| e["id"] == *id))
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(untouched, base);
        }
    }
}

#[test]
fn linked_rest_candidate_missing_actual_owner_refuses_without_mutating_real_clocks() {
    let observed = original();
    let case = &observed["cases"][0];
    let mut c = prepare_control(case);
    declare_links(&mut c, case);
    let index = c
        .owner
        .chords
        .iter()
        .position(|ch| {
            c.labels
                .get(&ch.id)
                .is_some_and(|name| name == "rest_chord5")
        })
        .unwrap();
    let mut held = std::mem::take(&mut c.owner.chords);
    let foreign = held.remove(index);
    c.owner.chords = held;
    let graph = c.owner.owner.graph.snapshot();
    let voices = projection(&c.owner.voices, &c.owner.measures);
    let layout = c.owner.layout();
    let report = c.owner.owner.prepared.report.clone();
    let caches = c.baseline.clone();
    let slots = c
        .owner
        .measures
        .iter()
        .map(|m| m.timing.slots.as_ptr())
        .collect::<Vec<_>>();
    assert_eq!(
        c.owner.interleaved_rest_candidates(1).unwrap_err(),
        "music_rest_actual_chord_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), graph);
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), voices);
    assert_eq!(c.owner.layout(), layout);
    assert_eq!(c.owner.owner.prepared.report, report);
    for (key, old) in c.rests.iter().zip(caches) {
        assert!(Arc::ptr_eq(
            &c.owner.owner.graph.rest_chord_beams(*key).unwrap(),
            &old
        ));
    }
    for (m, old) in c.owner.measures.iter().zip(slots) {
        assert_eq!(m.timing.slots.as_ptr(), old);
    }
    c.owner.chords.insert(index, foreign);
    let result = c.owner.interleaved_rest_candidates(1).unwrap();
    let id = c
        .owner
        .chords
        .iter()
        .find(|ch| {
            c.labels
                .get(&ch.id)
                .is_some_and(|name| name == "rest_chord5")
        })
        .unwrap()
        .id;
    assert!(
        result[0]["white_rest_ids"]
            .as_array()
            .unwrap()
            .contains(&json!(id))
    );
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), voices);
}
