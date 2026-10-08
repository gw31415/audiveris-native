// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame, prepare_control};
use super::*;
use crate::shared_sig::{Key, Phase};
use crate::sheet_skew::SheetSkew;

fn original() -> Value {
    let mut observed: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-added/observed.json"
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
fn node(c: &Control, id: usize) -> Value {
    c.owner.owner.graph.snapshot()["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
        .clone()
}
fn prepare(case: &Value) -> (Control, Vec<Key>) {
    let mut c = prepare_control(case);
    let gid = c.owner.actual_measure_beam_groups(1).unwrap()[0];
    let group: Key = serde_json::from_value(node(&c, gid)["origin"].clone()).unwrap();
    let next = c.owner.owner.graph.snapshot()["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| serde_json::from_value::<Key>(n["origin"].clone()).unwrap())
        .filter(|k| k.system == 1 && k.phase == Phase::Beam)
        .map(|k| k.local)
        .max()
        .unwrap()
        + 1;
    let mut beams = Vec::new();
    for (i, row) in case["declared_beams"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let beam = Key {
            system: 1,
            phase: Phase::Beam,
            local: next + i,
        };
        let id = c.owner.owner.graph.vertex(beam,json!({"kind":"BeamInter","median":row["median"],"height":row["height"],"abnormal":true}),false).unwrap();
        c.labels.insert(id, row["label"].as_str().unwrap().into());
        c.owner
            .owner
            .graph
            .relations(
                1,
                Phase::Group,
                vec![(group, beam, json!({"kind":"Containment"}))],
            )
            .unwrap();
        beams.push(beam);
    }
    (c, beams)
}
fn added_frame(c: &mut Control, beams: &[Key], added: &[usize]) -> Value {
    let mut actual = frame(c);
    for beam in beams {
        let id = c.owner.owner.graph.id(*beam).unwrap();
        actual["beams"]
            .as_array_mut()
            .unwrap()
            .push(json!({"label":c.labels[&id],"abnormal":node(c,id)["abnormal"]==true}));
    }
    actual["added_relations"] = json!(
        added
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let mut row = c.owner.owner.graph.beam_rest_registration(*id);
                row["label"] = json!(format!("added{i}"));
                row
            })
            .collect::<Vec<_>>()
    );
    actual["live_added_order"] = json!(
        c.owner.owner.graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| added
                .iter()
                .position(|id| e["id"] == *id)
                .map(|i| format!("added{i}")))
            .collect::<Vec<_>>()
    );
    actual
}
fn assert_frame(c: &mut Control, beams: &[Key], added: &[usize], case: &Value, step: &Value) {
    let mut expected = step["state"].clone();
    // Compare the held original relation objects independently. Their labels
    // are observations and cannot become native relation ownership.
    assert_eq!(
        expected["owned"]["relations"],
        case["states"][0]["state"]["owned"]["relations"]
    );
    expected["owned"]["relations"] = json!([]);
    assert_eq!(
        added_frame(c, beams, added),
        expected,
        "{} {}",
        case["name"],
        step["phase"]
    );
}
fn callback_frame(
    c: &Control,
    beam: Key,
    observation: &crate::shared_sig::CallbackObservation,
    added: &[usize],
    baseline_slots: &BTreeMap<Key, Option<Arc<()>>>,
) -> Value {
    let snapshot = &observation.snapshot;
    let relation = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == observation.edge)
        .unwrap();
    let bid = c.owner.owner.graph.id(beam).unwrap();
    let clocks = c
        .owner
        .chords
        .iter()
        .filter(|ch| ch.measure == 0)
        .map(|ch| {
            let n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == ch.id)
                .unwrap();
            let (_, voice, slot) = observation
                .owners
                .iter()
                .find(|(key, _, _)| *key == ch.key)
                .unwrap();
            let voice_label = voice.as_ref().map(|identity| {
                format!(
                    "voice{}",
                    c.refs
                        .iter()
                        .position(|v| Arc::ptr_eq(identity, &v.identity))
                        .unwrap()
                )
            });
            let same_slot = match (slot, &baseline_slots[&ch.key]) {
                (Some(now), Some(old)) => Arc::ptr_eq(now, old),
                (None, None) => true,
                _ => false,
            };
            json!({"label":c.labels[&ch.id],"manual":n["manual"]==true,"voice":voice_label,
            "slot_id":n["slot_id"],"same_baseline_slot":same_slot,"time_offset":n["time_offset"]})
        })
        .collect::<Vec<_>>();
    json!({"phase":observation.phase,"edge":format!("added{}",added.iter().position(|id|*id==observation.edge).unwrap()),
        "registered":true,"manual":relation["manual"]==true,"dy":relation["dy"],"portion":relation["beam_portion"],
        "beam_abnormal":snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==bid).unwrap()["abnormal"]==true,"clocks":clocks})
}

#[test]
fn original_beam_rest_addition_preserves_parallel_manual_identity_callback_clocks_and_cache_lifetimes()
 {
    for case in original()["cases"].as_array().unwrap() {
        let (mut c, beams) = prepare(case);
        let chord_storage = c.owner.chords.as_ptr();
        let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
        let slot_refs = c.owner.measures[0]
            .timing
            .slots
            .iter()
            .map(|s| s.identity.clone())
            .collect::<Vec<_>>();
        let baseline_slots = c
            .owner
            .chords
            .iter()
            .map(|ch| {
                (
                    ch.key,
                    c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let before_edges = c.owner.owner.graph.snapshot()["relations"].clone();
        let mut added = Vec::new();
        c.owner.owner.graph.trace_beam_rest_callbacks();
        for step in case["states"].as_array().unwrap() {
            let phase = step["phase"].as_str().unwrap();
            if let Some(index) = phase.strip_prefix("added") {
                let i: usize = index.parse().unwrap();
                let action = &case["actions"][i];
                assert_eq!(action["accepted"], true);
                let beam = beams[action["beam"].as_u64().unwrap() as usize];
                let rest = c.rests[action["rest"].as_u64().unwrap() as usize];
                c.owner.owner.sheet_skew =
                    SheetSkew::new(action["slope"].as_f64().unwrap()).unwrap();
                let id = c
                    .owner
                    .add_interleaved_rests(1, &[(beam, rest, action["manual"].as_bool().unwrap())])
                    .unwrap()[0];
                assert!(!added.contains(&id));
                added.push(id);
                let callbacks = c.owner.owner.graph.take_beam_rest_callbacks();
                assert_eq!(callbacks.len(), 2);
                for (offset, observation) in callbacks.iter().enumerate() {
                    assert_eq!(
                        callback_frame(&c, beam, observation, &added, &baseline_slots),
                        case["callbacks"][2 * i + offset]
                    );
                }
            } else {
                match phase {
                    "before_add" => {}
                    "clear" | "clear_again" => {
                        c.owner.owner.sheet_skew = SheetSkew::new(0.).unwrap();
                        c.owner.clear_interleaved_rests(1).unwrap();
                    }
                    "measure_reset" => reset_rhythm(
                        &mut c.owner.owner.graph,
                        &mut c.owner.voices,
                        &mut c.owner.measures[0],
                        &c.owner.chords,
                        0,
                    )
                    .unwrap(),
                    "rest_invalidate" => {
                        for key in &c.rests {
                            c.owner
                                .owner
                                .graph
                                .invalidate_owned_rest_cache(*key)
                                .unwrap();
                        }
                    }
                    _ => unreachable!(),
                }
            }
            assert_frame(&mut c, &beams, &added, case, step);
            let mut actual_edges = c.owner.owner.graph.snapshot()["relations"].clone();
            actual_edges
                .as_array_mut()
                .unwrap()
                .retain(|e| !added.iter().any(|id| e["id"] == *id));
            assert_eq!(actual_edges, before_edges);
            assert_eq!(c.owner.chords.as_ptr(), chord_storage);
            assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
            for (slot, old) in c.owner.measures[0].timing.slots.iter().zip(&slot_refs) {
                assert!(Arc::ptr_eq(&slot.identity, old));
            }
            for chord in &c.owner.chords {
                let current = c.owner.owner.graph.chord_slot_identity(chord.key);
                if chord.measure == 0 && matches!(phase, "measure_reset" | "rest_invalidate") {
                    assert!(current.is_none());
                } else {
                    match (current, &baseline_slots[&chord.key]) {
                        (Some(now), Some(old)) => assert!(Arc::ptr_eq(now, old)),
                        (None, None) => {}
                        _ => panic!("Slot object changed outside genuine reset"),
                    }
                }
            }
        }
    }
}

#[test]
fn beam_rest_addition_late_member_failure_rolls_back_relation_identity_manual_flags_actual_owners_and_clocks()
 {
    let observed = original();
    let case = &observed["cases"][0];
    let (mut c, beams) = prepare(case);
    let failed_chord = c
        .owner
        .chords
        .iter()
        .position(|ch| ch.key == c.rests[1])
        .unwrap();
    let members = std::mem::take(&mut c.owner.chords[failed_chord].members);
    let snapshot = c.owner.owner.graph.snapshot();
    let state = added_frame(&mut c, &beams, &[]);
    let chord_storage = c.owner.chords.as_ptr();
    let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
    let report = c.owner.owner.prepared.report.clone();
    let upstream = c.owner.owner.upstream_report.clone();
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
    c.owner.owner.graph.trace_beam_rest_callbacks();
    assert_eq!(
        c.owner.add_interleaved_rests(
            1,
            &[(beams[0], c.rests[0], true), (beams[0], c.rests[1], false)]
        ),
        Err("music_rest_actual_note_required".into())
    );
    assert!(c.owner.owner.graph.take_beam_rest_callbacks().is_empty());
    assert_eq!(c.owner.owner.graph.snapshot(), snapshot);
    assert_eq!(added_frame(&mut c, &beams, &[]), state);
    assert!(c.owner.chords[failed_chord].members.is_empty());
    assert_eq!(c.owner.chords.as_ptr(), chord_storage);
    assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
    assert_eq!(c.owner.owner.prepared.report, report);
    assert_eq!(c.owner.owner.upstream_report, upstream);
    for (key, old) in slots {
        match (old, c.owner.owner.graph.chord_slot_identity(key)) {
            (Some(old), Some(now)) => assert!(Arc::ptr_eq(&old, now)),
            (None, None) => {}
            _ => panic!("rollback lost actual Slot"),
        }
    }
    c.owner.chords[failed_chord].members = members;
    let expected_id = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_u64().unwrap() as usize)
        .max()
        .unwrap()
        + 1;
    let id = c
        .owner
        .add_interleaved_rests(1, &[(beams[0], c.rests[0], true)])
        .unwrap()[0];
    assert_eq!(id, expected_id);
    assert_frame(&mut c, &beams, &[id], case, &case["states"][1]);
}
