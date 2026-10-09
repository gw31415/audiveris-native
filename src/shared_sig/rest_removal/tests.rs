// SPDX-License-Identifier: AGPL-3.0-or-later
//! Declared scalar owner controls, not fresh image or original observer evidence.
use super::*;
use crate::{
    curves::Curve,
    music_time::Fraction,
    music_voices::{self, Slot, SlotVoice},
};
use serde_json::{Value, json};

struct OwnedRest {
    graph: Graph,
    staff: Arc<StaffGeometry>,
    measures: Vec<Measure>,
    chords: Vec<Chord>,
    voices: Vec<Voice>,
    rest: Key,
    chord: Key,
    edge: usize,
    slot: Slot,
}
fn fixture(excess: bool) -> OwnedRest {
    let staff = Arc::new(StaffGeometry {
        id: 1,
        system: 1,
        specific_interline: 10,
        abscissae: [0, 200],
        lines: (0..5)
            .map(|i| {
                Arc::new(
                    Curve::interpolate(vec![[0., i as f64 * 10.], [200., i as f64 * 10.]]).unwrap(),
                )
            })
            .collect(),
    });
    let mut graph = Graph::default();
    let rest = Key {
        system: 1,
        phase: Phase::Symbol,
        local: 1,
    };
    let rid = graph
        .vertex(
            rest,
            json!({"kind":"RestInter","shape":"QUARTER_REST",
        "bounds":[20,10,6,20],"staff_id":1,"grade":0.2,"manual":true,
        "frozen":false,"abnormal":false}),
            false,
        )
        .unwrap();
    graph.register_rest_membership(rest, staff.clone()).unwrap();
    let chord = Key {
        system: 1,
        phase: Phase::Chord,
        local: 1,
    };
    let cid = graph
        .vertex(
            chord,
            json!({"kind":"RestChordInter","bounds":null,
        "staff_id":1,"part_id":1,"measure_index":1,"member_ids":[],"grade":null,
        "voice":null,"time_offset":null,"manual":true,"frozen":false}),
            false,
        )
        .unwrap();
    graph
        .relations(
            1,
            Phase::Chord,
            vec![(chord, rest, json!({"kind":"Containment","manual":true}))],
        )
        .unwrap();
    let edge = graph.incident_edges(cid)[0];
    assert_eq!(graph.edges[edge - 1].key.target, rid);
    let chords = vec![Chord {
        kind: ChordKind::Rest,
        key: chord,
        id: cid,
        stem: None,
        members: vec![graph.retained_inter(rest).unwrap().0.clone()],
        measure: 0,
        bounds: [20, 10, 6, 20],
    }];
    let mut measures = vec![Measure {
        system: 1,
        part: 1,
        staff: 1,
        index: 1,
        left: 0,
        right: 200,
        geometry: None,
        timing: Default::default(),
        tuplets: vec![],
        stack_tuplets: vec![],
    }];
    let mut voices = Vec::new();
    let vi = music_voices::create_voice(&mut graph, &mut voices, &measures, &chords[0]).unwrap();
    let slot = Slot {
        identity: Arc::new(()),
        id: 1,
        time_offset: Some(Fraction::new(7, 64).unwrap()),
        x_offset: 23,
        incoming_ids: vec![cid],
    };
    graph
        .set_chord_slot(chord, slot.id, &slot.identity)
        .unwrap();
    let mut body = graph.retained_inter(chord).unwrap().0.clone();
    body["time_offset"] = json!("7/64");
    graph.vertex(chord, body, false).unwrap();
    measures[0].timing.expected = Some(Fraction::new(if excess { 1 } else { 3 }, 4).unwrap());
    measures[0].timing.slots.push(slot.clone());
    voices[vi].slots.insert(
        slot.id,
        SlotVoice {
            chord_id: cid,
            continuing: false,
        },
    );
    music_voices::check_duration(&graph, &mut voices[vi], &mut measures[0]).unwrap();
    assert_eq!(graph.chord_end_time(chord).unwrap(), Some("23/64".into()));
    assert_eq!(
        voices[vi].termination.map(crate::music_time::format_delta),
        Some(if excess { "7/64" } else { "-25/64" }.into())
    );
    assert_eq!(
        voices[vi].excess,
        excess.then_some(Fraction::new(7, 64).unwrap())
    );
    OwnedRest {
        graph,
        staff,
        measures,
        chords,
        voices,
        rest,
        chord,
        edge,
        slot,
    }
}
fn state(g: &Graph, chords: &[Chord], voices: &[Voice], measures: &[Measure]) -> Value {
    json!({"graph":format!("{g:?}"), "chords":format!("{chords:?}"),
        "voices":music_voices::projection(voices, measures),
        "expected":measures[0].timing.expected.map(crate::music_time::format_fraction),
        "measure_excess":measures[0].timing.excess.map(crate::music_time::format_fraction)})
}
fn assert_retained_clock(g: &Graph, key: Key, voices: &[Voice], old: &Voice, slot: &Slot) {
    let id = g.id(key).unwrap();
    let vertex = &g.vertices[id - 1];
    assert_eq!(vertex.data["time_offset"], "7/64");
    assert_eq!(vertex.data["manual"], true);
    assert!(Arc::ptr_eq(
        vertex.voice_owner.as_ref().unwrap(),
        &old.identity
    ));
    assert!(Arc::ptr_eq(
        vertex.slot_owner.as_ref().unwrap(),
        &slot.identity
    ));
    let actual = voices
        .iter()
        .chain(&g.retained_voices)
        .find(|v| Arc::ptr_eq(&v.identity, &old.identity))
        .unwrap();
    assert_eq!(actual.id, old.id);
    assert_eq!(actual.termination, old.termination);
    assert_eq!(actual.excess, old.excess);
    assert_eq!(actual.measure_rest_chord, old.measure_rest_chord);
    assert_eq!(actual.slots.len(), old.slots.len());
    assert_eq!(
        actual.slots[&slot.id].chord_id,
        old.slots[&slot.id].chord_id
    );
    assert_eq!(
        actual.slots[&slot.id].continuing,
        old.slots[&slot.id].continuing
    );
}

struct ScalarObservationOwner {
    rest: Key,
    chord: Chord,
    staff: Arc<StaffGeometry>,
    voice: Voice,
    slot: Slot,
    relation_id: usize,
    relation_body: Value,
    chord_body: Value,
}
fn scalar_observation(
    graph: &Graph,
    chords: &[Chord],
    voices: &[Voice],
    measures: &[Measure],
    held: &ScalarObservationOwner,
) -> Value {
    let rest_id = graph.id(held.rest).unwrap();
    let rest = &graph.vertices[rest_id - 1];
    let chord = &graph.vertices[held.chord.id - 1];
    let actual = chords
        .iter()
        .find(|c| c.key == held.chord.key)
        .or_else(|| graph.retained_removed_rest_chord(held.chord.key))
        .unwrap();
    let voice = voices
        .iter()
        .chain(&graph.retained_voices)
        .find(|v| Arc::ptr_eq(&v.identity, &held.voice.identity))
        .unwrap();
    let measure = &measures[held.chord.measure];
    let slot = measure
        .timing
        .slots
        .iter()
        .find(|s| Arc::ptr_eq(&s.identity, &held.slot.identity))
        .unwrap();
    let relation = &graph.edges[held.relation_id - 1];
    let member_count = graph
        .edges
        .iter()
        .filter(|e| !e.removed && e.key.kind == "Containment" && e.key.source == held.chord.id)
        .count();
    let slot_entries = voice.slots.iter().map(|(id, entry)| json!({"slot_id":id,
        "chord_is_held":entry.chord_id == held.chord.id,"status":if entry.continuing {"CONTINUE"} else {"BEGIN"}})).collect::<Vec<_>>();
    let slots_unchanged = voice.slots.len() == held.voice.slots.len()
        && voice.slots.iter().all(|(id, actual)| {
            held.voice.slots.get(id).is_some_and(|old| {
                actual.chord_id == old.chord_id && actual.continuing == old.continuing
            })
        });
    json!({
        "rest":{"removed":rest.removed || rest.removing,"registered":!rest.removed,"manual":rest.data["manual"] == true,
            "native_id":rest_id,"same_actual_Staff":Arc::ptr_eq(graph.retained_rest_staff(held.rest).unwrap(), &held.staff)},
        "chord":{"removed":chord.removed || chord.removing,"registered":!chord.removed,"manual":chord.data["manual"] == true,
            "time_offset":chord.data["time_offset"],"member_count":member_count,
            "same_identity":actual.key == held.chord.key && actual.id == held.chord.id && graph.id(actual.key) == Some(actual.id),
            "same_bounds":actual.bounds == held.chord.bounds,
            "raw_bounds":chord.data["bounds"],"raw_bounds_unchanged":chord.data["bounds"] == held.chord_body["bounds"],
            "same_raw_staff_id":chord.data["staff_id"] == held.chord_body["staff_id"],
            "same_raw_part_id":chord.data["part_id"] == held.chord_body["part_id"],
            "same_measure_index":actual.measure == held.chord.measure,
            "same_voice":chord.voice_owner.as_ref().is_some_and(|v|Arc::ptr_eq(v,&held.voice.identity)),
            "same_slot":chord.slot_owner.as_ref().is_some_and(|s|Arc::ptr_eq(s,&held.slot.identity)),
            "retired_actual_Chord":graph.retained_removed_rest_chord(held.chord.key).map(|c|json!({"key":c.key,"id":c.id,
                "measure_index":c.measure,"bounds":c.bounds,"stem":c.stem,"member_ids":c.members.iter().map(|n|n["id"].clone()).collect::<Vec<_>>()}))},
        "staff_contains_rest":graph.staff_rest_members(&held.staff).unwrap().contains(&held.rest),
        "measure_contains_chord":chords.iter().any(|c|c.key == held.chord.key),
        "measure_contains_voice":voices.iter().any(|v|Arc::ptr_eq(&v.identity,&held.voice.identity)),
        "measure_geometry_is_none":measure.geometry.is_none(),
        "expected_duration":measure.timing.expected.map(crate::music_time::format_fraction),
        "scalar_measure_excess":measure.timing.excess.map(crate::music_time::format_fraction),
        "voice":{"same_identity":Arc::ptr_eq(&voice.identity,&held.voice.identity),"id":voice.id,
            "same_measure_index":voice.measure == held.voice.measure,"chord_count":voice.chord_ids.len(),
            "chord_ids":voice.chord_ids,"slot_count":voice.slots.len(),"slot_entries_unchanged":slots_unchanged,"slot_entries":slot_entries,
            "termination":voice.termination.map(crate::music_time::format_delta),"excess":voice.excess.map(crate::music_time::format_fraction)},
        "slot":{"same_identity":Arc::ptr_eq(&slot.identity,&held.slot.identity),"id":slot.id,
            "time_offset":slot.time_offset.map(crate::music_time::format_fraction),
            "incoming_same_chord":slot.incoming_ids == [held.chord.id],"retained_in_measure_timing":true},
        "containment":{"registered":!relation.removed,"manual":relation.data["manual"] == true,
            "same_identity":relation.data["id"] == held.relation_id && relation.key.source == held.chord.id && relation.key.target == rest_id,
            "same_body":relation.data == held.relation_body,"native_relation_id":held.relation_id,"body":relation.data}
    })
}

#[test]
fn weak_rest_cascade_preserves_real_clock_owners_and_observes_containers_before_callbacks() {
    let mut observations = Vec::new();
    for excess in [false, true] {
        let mut p = fixture(excess);
        let old_voice = p.voices[0].clone();
        let (rest, chord, edge) = (p.rest, p.chord, p.edge);
        let staff = p.staff.clone();
        let held = ScalarObservationOwner {
            rest,
            chord: p.chords[0].clone(),
            staff: staff.clone(),
            voice: old_voice.clone(),
            slot: p.slot.clone(),
            relation_id: edge,
            relation_body: p.graph.edges[edge - 1].data.clone(),
            chord_body: p.graph.retained_inter(chord).unwrap().0.clone(),
        };
        let prepared = scalar_observation(&p.graph, &p.chords, &p.voices, &p.measures, &held);
        let mut observed_events = Vec::new();
        let mut previous_observed = prepared.clone();
        let mut events = Vec::new();
        let removed = p
            .graph
            .remove_first_weak_rest(
                rest,
                &p.measures,
                &mut p.chords,
                &mut p.voices,
                |g, chords, voices, event| {
                    assert_retained_clock(g, chord, voices, &old_voice, &p.slot);
                    assert!(Arc::ptr_eq(g.retained_rest_staff(rest)?, &staff));
                    assert!(g.staff_rest_members(&staff)?.is_empty());
                    match event.stage {
                        RestRemovalStage::Staff => {
                            assert_eq!(g.weak_rest_registration(rest)?, (false, true));
                            assert_eq!(chords.len(), 1);
                            assert_eq!(voices.len(), 1);
                        }
                        RestRemovalStage::Measure => {
                            assert_eq!(g.weak_rest_registration(rest)?, (true, true));
                            assert_eq!(g.weak_rest_registration(chord)?, (false, true));
                            assert!(chords.is_empty() && voices.is_empty());
                            assert_eq!(
                                g.retained_removed_rest_chord(chord).unwrap().members.len(),
                                1
                            );
                        }
                        RestRemovalStage::Edge => {
                            assert_eq!(event.relation_id, Some(edge));
                            assert_eq!(g.weak_rest_registration(rest)?, (true, true));
                            assert_eq!(g.weak_rest_registration(chord)?, (true, true));
                            assert!(g.edges[edge - 1].removed);
                            assert_eq!(g.edges[edge - 1].data["manual"], true);
                            assert!(
                                g.retained_removed_rest_chord(chord)
                                    .unwrap()
                                    .members
                                    .is_empty()
                            );
                        }
                        RestRemovalStage::Vertex => {
                            assert_eq!(g.weak_rest_registration(event.inter)?, (true, false));
                        }
                    }
                    let current = scalar_observation(g, chords, voices, &p.measures, &held);
                    observed_events.push(json!({"phase":event.stage.as_str(),
                        "subject_kind":g.retained_inter(event.inter)?.0["kind"],
                        "native_relation_id":event.relation_id,"previous_observed_state":previous_observed,
                        "state":current}));
                    previous_observed = current;
                    events.push((event.stage, event.inter));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(removed, vec![chord, rest]);
        assert_eq!(
            events,
            vec![
                (RestRemovalStage::Staff, rest),
                (RestRemovalStage::Measure, chord),
                (RestRemovalStage::Edge, chord),
                (RestRemovalStage::Vertex, chord),
                (RestRemovalStage::Vertex, rest)
            ]
        );
        assert_eq!(
            p.measures[0].timing.expected,
            Some(Fraction::new(if excess { 1 } else { 3 }, 4).unwrap())
        );
        assert!(Arc::ptr_eq(
            &p.measures[0].timing.slots[0].identity,
            &p.slot.identity
        ));
        let before = state(&p.graph, &p.chords, &p.voices, &p.measures);
        assert!(
            p.graph
                .remove_first_weak_rest(
                    chord,
                    &p.measures,
                    &mut p.chords,
                    &mut p.voices,
                    |_, _, _, _| panic!("already-cascaded queue item emitted a callback")
                )
                .unwrap()
                .is_empty()
        );
        assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
        observations.push(json!({"name":if excess {"excess"} else {"underfill"},
            "declared_expected":p.measures[0].timing.expected.map(crate::music_time::format_fraction),
            "prepared":prepared,"events":observed_events,
            "after":scalar_observation(&p.graph,&p.chords,&p.voices,&p.measures,&held),
            "later_queued_Chord_removal_idempotent":true}));
    }
    std::fs::write("/tmp/rest-owned-native-scalar-observed.json",serde_json::to_vec_pretty(&json!({
        "scope":"Existing two scalar singleton quarter-Rest controls, real Voice/Slot Arcs and typed Chord/Staff owners; no source fixture inputs",
        "measure_geometry":"None, declared scalar Measure index; does not establish actual Part/Measure/Stack Arc parity",
        "identity_scope":"Voice/Slot/Staff use retained Arc identities; Chord identity is retained key/id and typed Chord record; SlotVoice and Rational are values, not Java object-identity claims",
        "boundary_scope":"Prepared state plus five actual callback aftermath states and after. previous_observed_state is the prior recorded boundary, not an injected method-entry hook",
        "controls":observations})).unwrap()).unwrap();
}

#[test]
fn late_rest_callback_failure_restores_actual_memberships_manual_identity_and_clocks() {
    let mut p = fixture(true);
    let before = state(&p.graph, &p.chords, &p.voices, &p.measures);
    let chord_storage = p.chords.as_ptr();
    let old_voice = p.voices[0].clone();
    let rest = p.rest;
    let mut reached = false;
    let error = p
        .graph
        .remove_first_weak_rest(
            rest,
            &p.measures,
            &mut p.chords,
            &mut p.voices,
            |_, chords, voices, event| {
                if event.stage == RestRemovalStage::Vertex && event.inter == rest {
                    assert!(chords.is_empty() && voices.is_empty());
                    reached = true;
                    return Err("declared_late_rest_callback_failure".into());
                }
                Ok(())
            },
        )
        .unwrap_err();
    assert!(reached);
    assert_eq!(error, "declared_late_rest_callback_failure");
    assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
    assert_eq!(p.chords.as_ptr(), chord_storage);
    assert_retained_clock(&p.graph, p.chord, &p.voices, &old_voice, &p.slot);
    assert_eq!(p.graph.staff_rest_members(&p.staff).unwrap(), [p.rest]);
}

#[test]
fn direct_singleton_rest_chord_removal_removes_member_before_ensemble_vertex() {
    let mut p = fixture(false);
    let mut order = Vec::new();
    let removed = p
        .graph
        .remove_first_weak_rest(
            p.chord,
            &p.measures,
            &mut p.chords,
            &mut p.voices,
            |_, _, _, event| {
                order.push((event.stage, event.inter));
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(removed, vec![p.rest, p.chord]);
    assert_eq!(
        order,
        vec![
            (RestRemovalStage::Measure, p.chord),
            (RestRemovalStage::Staff, p.rest),
            (RestRemovalStage::Edge, p.rest),
            (RestRemovalStage::Vertex, p.rest),
            (RestRemovalStage::Vertex, p.chord)
        ]
    );
}

#[test]
fn a_different_staff_object_with_the_same_number_cannot_replace_actual_rest_owner() {
    let mut p = fixture(false);
    let other = Arc::new(StaffGeometry {
        id: p.staff.id,
        system: p.staff.system,
        specific_interline: p.staff.specific_interline,
        abscissae: p.staff.abscissae,
        lines: p.staff.lines.clone(),
    });
    let before = state(&p.graph, &p.chords, &p.voices, &p.measures);
    assert_eq!(
        p.graph.register_rest_membership(p.rest, other).unwrap_err(),
        "music_weak_rest_Staff_identity_changed"
    );
    assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
}

#[test]
fn captured_weak_queue_retries_after_late_failure_and_calls_cascaded_chord_idempotently() {
    for excess in [false, true] {
        let mut p = fixture(excess);
        // Declared two-object source deleteWeakInters queue. It is captured
        // before removal; no scheduler/context implementation is claimed here.
        let queue = vec![p.rest, p.chord];
        for key in &queue {
            assert!(
                p.graph.retained_inter(*key).unwrap().0["grade"]
                    .as_f64()
                    .unwrap()
                    < 0.5
            );
        }
        let expected = p.measures[0].timing.expected;
        let old_voice = p.voices[0].clone();
        let before = state(&p.graph, &p.chords, &p.voices, &p.measures);
        let (rest, chord) = (p.rest, p.chord);
        let mut failed_calls = Vec::new();
        let failure = (|| -> Result<(), String> {
            for key in &queue {
                failed_calls.push(*key);
                p.graph.remove_first_weak_rest(
                    *key,
                    &p.measures,
                    &mut p.chords,
                    &mut p.voices,
                    |g, _, voices, event| {
                        assert_retained_clock(g, chord, voices, &old_voice, &p.slot);
                        if event.inter == rest && event.stage == RestRemovalStage::Vertex {
                            assert!(g.retained_removed_rest_chord(chord).is_some());
                            return Err("declared_captured_queue_callback_failure".into());
                        }
                        Ok(())
                    },
                )?;
            }
            Ok(())
        })()
        .unwrap_err();
        assert_eq!(failure, "declared_captured_queue_callback_failure");
        assert_eq!(failed_calls, vec![rest]);
        assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
        assert_eq!(p.measures[0].timing.expected, expected);
        assert_eq!(queue, vec![rest, chord]);

        let mut calls = Vec::new();
        let mut vertices = Vec::new();
        let mut callback_counts = Vec::new();
        for key in &queue {
            calls.push((*key, p.graph.weak_rest_registration(*key).unwrap()));
            let mut callbacks = 0;
            let removed = p
                .graph
                .remove_first_weak_rest(
                    *key,
                    &p.measures,
                    &mut p.chords,
                    &mut p.voices,
                    |g, _, voices, event| {
                        callbacks += 1;
                        assert_retained_clock(g, chord, voices, &old_voice, &p.slot);
                        if event.stage == RestRemovalStage::Vertex {
                            vertices.push(event.inter);
                        }
                        Ok(())
                    },
                )
                .unwrap();
            if *key == chord {
                assert!(removed.is_empty());
            }
            callback_counts.push(callbacks);
            assert_eq!(p.measures[0].timing.expected, expected);
            assert!(Arc::ptr_eq(
                &p.measures[0].timing.slots[0].identity,
                &p.slot.identity
            ));
        }
        assert_eq!(calls, vec![(rest, (false, true)), (chord, (true, false))]);
        assert_eq!(vertices, vec![chord, rest]);
        assert!(callback_counts[0] > 0);
        assert_eq!(callback_counts[1], 0);
        assert!(p.chords.is_empty() && p.voices.is_empty());
        assert_retained_clock(&p.graph, chord, &p.voices, &old_voice, &p.slot);
    }
}

#[test]
fn owned_symbol_reducer_rolls_back_earlier_rest_when_later_owner_is_missing() {
    for excess in [false, true] {
        let mut p = fixture(excess);
        let old_voice = p.voices[0].clone();
        let invalid = Key { local: 2, ..p.rest };
        p.graph
            .vertex(
                invalid,
                json!({"kind":"RestInter","shape":"QUARTER_REST",
            "bounds":[400,10,6,20],"staff_id":1,"grade":0.2}),
                false,
            )
            .unwrap();
        let before = state(&p.graph, &p.chords, &p.voices, &p.measures);
        let chord_storage = p.chords.as_ptr();
        let voice_storage = p.voices.as_ptr();
        assert_eq!(
            crate::music_symbol_reduction::reduce_owned(
                &mut p.graph,
                &p.measures,
                &mut p.chords,
                &mut p.voices
            )
            .unwrap_err(),
            "music_weak_rest_added_owner_required"
        );
        assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
        assert_eq!(p.chords.as_ptr(), chord_storage);
        assert_eq!(p.voices.as_ptr(), voice_storage);
        assert_retained_clock(&p.graph, p.chord, &p.voices, &old_voice, &p.slot);
        // The deliberately unowned scalar vertex has no factory membership.
        p.graph.remove_music_inter(invalid).unwrap();
        let result = crate::music_symbol_reduction::reduce_owned(
            &mut p.graph,
            &p.measures,
            &mut p.chords,
            &mut p.voices,
        )
        .unwrap();
        assert_eq!(
            result["removed_weak_symbol_ids"],
            json!([p.graph.id(p.rest).unwrap()])
        );
        assert_eq!(
            result["removed_owned_chord_ids"],
            json!([p.graph.id(p.chord).unwrap()])
        );
        assert!(p.chords.is_empty() && p.voices.is_empty());
        assert_retained_clock(&p.graph, p.chord, &p.voices, &old_voice, &p.slot);
    }
}

#[test]
fn registered_rest_and_its_chord_cannot_bypass_actual_owner_removal() {
    let mut p = fixture(false);
    let before = state(&p.graph, &p.chords, &p.voices, &p.measures);
    for key in [p.rest, p.chord] {
        assert_eq!(
            p.graph.remove_music_inter(key).unwrap_err(),
            "music_removal_actual_Rest_owners_required"
        );
        assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
    }
    assert_eq!(
        crate::music_symbol_reduction::reduce(&mut p.graph).unwrap_err(),
        "music_removal_actual_Rest_owners_required"
    );
    assert_eq!(state(&p.graph, &p.chords, &p.voices, &p.measures), before);
}
