// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measures::OwnedMusic;
use std::sync::Arc;

fn label_map(owner: &OwnedMusic) -> BTreeMap<usize, String> {
    let mut chords = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
        .collect::<Vec<_>>();
    chords.sort_by_key(|c| c.bounds[0]);
    let mut labels = BTreeMap::new();
    for (index, chord) in chords.iter().enumerate() {
        labels.insert(chord.id, format!("chord{index}"));
        for (member, head) in chord.members.iter().enumerate() {
            labels.insert(
                head["id"].as_u64().unwrap() as usize,
                format!("head{index}_{member}"),
            );
        }
    }
    labels
}
fn prepare(case: &Value) -> (OwnedMusic, BTreeMap<usize, String>, Vec<Voice>) {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
    ))
    .unwrap();
    let mut owner = crate::music_xml::recognize(&image).unwrap().owner;
    let labels = label_map(&owner);
    let snapshot = owner.owner.graph.snapshot();
    for name in ["chord0", "chord3"] {
        let id = *labels
            .iter()
            .find(|(_, label)| label.as_str() == name)
            .unwrap()
            .0;
        let chord = owner.chords.iter().find(|c| c.id == id).unwrap();
        let mut node = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap()
            .clone();
        if name == "chord0" {
            node["manual"] = json!(true);
        } else {
            node["time_offset"] = case["declared_last_offset"].clone();
        }
        owner.owner.graph.vertex(chord.key, node, false).unwrap();
    }
    let index = owner.voices.iter().position(|v| v.measure == 0).unwrap();
    check_duration(
        &owner.owner.graph,
        &mut owner.voices[index],
        &mut owner.measures[0],
    )
    .unwrap();
    let refs = vec![owner.voices[index].clone()];
    (owner, labels, refs)
}
fn actual_voice<'a>(owner: &'a OwnedMusic, reference: &Voice) -> &'a Voice {
    owner
        .voices
        .iter()
        .find(|voice| Arc::ptr_eq(&voice.identity, &reference.identity))
        .or_else(|| owner.owner.graph.retained_voice_owner(&reference.identity))
        .expect("actual Voice lost after lifecycle removal")
}
fn state(owner: &OwnedMusic, labels: &BTreeMap<usize, String>, refs: &[Voice]) -> Value {
    let voice_label = |voice: &Voice| {
        ["old_voice", "new_voice", "rebuilt_voice"][refs
            .iter()
            .position(|reference| Arc::ptr_eq(&voice.identity, &reference.identity))
            .unwrap()]
    };
    let graph = &owner.owner.graph;
    let snapshot = graph.snapshot();
    let chords = owner.chords.iter().filter(|c| c.measure == 0).map(|chord| {
        let node = snapshot["inters"].as_array().unwrap().iter().find(|n| n["id"] == chord.id).unwrap();
        let voice = refs.iter().find(|v| graph.chord_has_voice(chord.key, v)).map(voice_label);
        json!({"label":labels[&chord.id],"registered":true,"same_measure":chord.measure == 0,
            "manual":node["manual"] == true,"members":chord.members.iter().map(|n| labels[&(n["id"].as_u64().unwrap() as usize)].clone()).collect::<Vec<_>>(),
            "voice":voice,"same_baseline_slot":node["slot_id"] == refs[0].slots.iter().find(|(_,s)| s.chord_id == chord.id && !s.continuing).map(|(id,_)|json!(id)).unwrap(),
            "slot_id":node["slot_id"],"time_offset":node["time_offset"],"duration":graph.chord_duration(chord.key).unwrap(),"end_time":graph.chord_end_time(chord.key).unwrap()})
    }).collect::<Vec<_>>();
    let voices = refs.iter().map(|reference| {
        let voice = actual_voice(owner, reference);
        json!({"label":voice_label(voice),"id":voice.id,"same_measure":voice.measure == 0,
            "measure_registered":owner.voices.iter().any(|v| Arc::ptr_eq(&v.identity, &voice.identity)),
            "chords":voice.chord_ids.iter().map(|id| labels[id].clone()).collect::<Vec<_>>(),
            "slots":voice.slots.iter().map(|(id,s)|json!({"id":id,"chord":labels[&s.chord_id],"status":if s.continuing {"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>(),
            "termination":voice.termination.map(crate::music_time::format_delta),"excess":voice.excess.map(crate::music_time::format_fraction)})
    }).collect::<Vec<_>>();
    let timing = &owner.measures[0].timing;
    json!({"chords":chords,"voices":voices,"measure_voices":owner.voices.iter().filter(|v| v.measure == 0).map(voice_label).collect::<Vec<_>>(),
        "measure_chords":owner.chords.iter().filter(|c| c.measure == 0).map(|c| labels[&c.id].clone()).collect::<Vec<_>>(),
        "measure_abnormal":timing.abnormal,"expected":timing.expected.map(crate::music_time::format_fraction),"actual":timing.actual.map(crate::music_time::format_fraction),
        "excess":timing.excess.map(crate::music_time::format_fraction),"stack_slot_ids":timing.slots.iter().map(|s| s.id).collect::<Vec<_>>()})
}
fn compare(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    expected: &Value,
    native_relations: &Value,
    original_relations: &Value,
) {
    let mut wanted = expected.clone();
    assert_eq!(
        &wanted.as_object_mut().unwrap().remove("relations").unwrap(),
        original_relations
    );
    assert_eq!(
        &owner.owner.graph.snapshot()["relations"],
        native_relations,
        "existing native relation identities/manual status changed"
    );
    assert_eq!(state(owner, labels, refs), wanted);
}
fn original() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/official-voice-lifecycle/observed.json"
    ))
    .unwrap()
}
#[test]
fn original_refine_purge_keeps_short_voice_slots_and_reused_number_identity() {
    compare_purge(0);
}
#[test]
fn original_refine_purge_keeps_positive_excess_and_reused_number_identity() {
    compare_purge(1);
}
fn compare_purge(index: usize) {
    compare_lifecycle(index, 8);
}
#[test]
fn original_measure_stack_reset_keeps_short_old_owners_and_legitimate_chord_reset() {
    compare_lifecycle(0, 12);
}
#[test]
fn original_measure_stack_reset_keeps_excess_old_owners_and_legitimate_chord_reset() {
    compare_lifecycle(1, 12);
}
fn compare_lifecycle(index: usize, count: usize) {
    let oracle = original();
    let case = &oracle["cases"][index];
    let (mut owner, labels, mut refs) = prepare(case);
    let relations = owner.owner.graph.snapshot()["relations"].clone();
    let source_relations = &case["states"][0]["state"]["relations"];
    for step in case["states"].as_array().unwrap().iter().take(count) {
        match step["phase"].as_str().unwrap() {
            "timed" => {}
            "emptied" => owner
                .voices
                .iter_mut()
                .find(|v| v.measure == 0)
                .unwrap()
                .chord_ids
                .clear(),
            "purged" | "purged_again" | "repurged" | "refined_reused" => refine_ordinary_system(
                &mut owner.owner.graph,
                &mut owner.voices,
                &owner.measures,
                1,
            )
            .unwrap(),
            "restored_old" => {
                owner.voices.push(actual_voice(&owner, &refs[0]).clone());
            }
            "measure_reset" | "measure_reset_again" => {
                let chords = owner.chords.as_ptr();
                let slots = owner.measures[0].timing.slots.as_ptr();
                reset_rhythm(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &mut owner.measures[0],
                    &owner.chords,
                    0,
                )
                .unwrap();
                assert_eq!(owner.chords.as_ptr(), chords);
                assert_eq!(owner.measures[0].timing.slots.as_ptr(), slots);
            }
            "stack_reset" => {
                let expected = owner.measures[0].timing.expected;
                reset_stack_rhythm(&mut owner.measures[0], expected);
            }
            "reused" | "rebuilt" => {
                let id = *labels
                    .iter()
                    .find(|(_, label)| label.as_str() == "chord0")
                    .unwrap()
                    .0;
                let seed = owner.chords.iter().find(|c| c.id == id).unwrap().clone();
                let index = construct_owned_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    &seed,
                )
                .unwrap();
                refs.push(owner.voices[index].clone());
            }
            _ => unreachable!(),
        }
        compare(
            &owner,
            &labels,
            &refs,
            &step["state"],
            &relations,
            source_relations,
        );
        if matches!(
            step["phase"].as_str(),
            Some("purged" | "purged_again" | "repurged" | "refined_reused")
        ) {
            assert_eq!(
                owner
                    .owner
                    .graph
                    .retained_voice_observations()
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
        }
    }
}

#[test]
fn failed_refine_after_purge_keeps_actual_owners_clocks_and_retry_identity() {
    let oracle = original();
    let case = &oracle["cases"][1];
    let (mut owner, labels, refs) = prepare(case);
    owner
        .voices
        .iter_mut()
        .find(|v| v.measure == 0)
        .unwrap()
        .chord_ids
        .clear();
    let other = owner.voices.iter().position(|v| v.measure == 1).unwrap();
    let staff = owner.voices[other].starting_staff;
    owner.voices[other].starting_staff = staff + 1;
    let graph = owner.owner.graph.snapshot();
    let layout = owner.layout();
    let before = state(&owner, &labels, &refs);
    let voice_objects = owner
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
    let chord_address = owner.chords.as_ptr();
    assert_eq!(
        refine_ordinary_system(
            &mut owner.owner.graph,
            &mut owner.voices,
            &owner.measures,
            1
        )
        .unwrap_err(),
        "music_refine_owned_HIGH_voice_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(owner.layout(), layout);
    assert_eq!(state(&owner, &labels, &refs), before);
    assert!(
        owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(owner.chords.as_ptr(), chord_address);
    for (voice, identity) in owner.voices.iter().zip(&voice_objects) {
        assert!(Arc::ptr_eq(&voice.identity, identity));
    }
    owner.voices[other].starting_staff = staff;
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    compare(
        &owner,
        &labels,
        &refs,
        &case["states"][2]["state"],
        &graph["relations"],
        &case["states"][0]["state"]["relations"],
    );
    assert_eq!(
        owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn failed_measure_reset_keeps_actual_owners_chords_clocks_and_retry_identity() {
    let oracle = original();
    let case = &oracle["cases"][1];
    let (mut owner, labels, mut refs) = prepare(case);
    owner
        .voices
        .iter_mut()
        .find(|v| v.measure == 0)
        .unwrap()
        .chord_ids
        .clear();
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    let seed = owner
        .chords
        .iter()
        .find(|c| labels.get(&c.id).is_some_and(|label| label == "chord0"))
        .unwrap()
        .clone();
    let index = construct_owned_voice(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        &seed,
    )
    .unwrap();
    refs.push(owner.voices[index].clone());
    let graph = owner.owner.graph.snapshot();
    let layout = owner.layout();
    let before = state(&owner, &labels, &refs);
    let retained = owner.owner.graph.retained_voice_observations();
    let voice_objects = owner
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
    let chord_address = owner.chords.as_ptr();
    let slot_address = owner.measures[0].timing.slots.as_ptr();
    let mut invalid = owner.chords.clone();
    invalid
        .iter_mut()
        .filter(|c| c.measure == 0)
        .last()
        .unwrap()
        .id += 1000;
    assert_eq!(
        reset_rhythm(
            &mut owner.owner.graph,
            &mut owner.voices,
            &mut owner.measures[0],
            &invalid,
            0
        )
        .unwrap_err(),
        "music_rhythm_reset_owned_chord_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(owner.layout(), layout);
    assert_eq!(state(&owner, &labels, &refs), before);
    assert_eq!(owner.owner.graph.retained_voice_observations(), retained);
    assert_eq!(owner.chords.as_ptr(), chord_address);
    assert_eq!(owner.measures[0].timing.slots.as_ptr(), slot_address);
    for (voice, identity) in owner.voices.iter().zip(&voice_objects) {
        assert!(Arc::ptr_eq(&voice.identity, identity));
    }
    reset_rhythm(
        &mut owner.owner.graph,
        &mut owner.voices,
        &mut owner.measures[0],
        &owner.chords,
        0,
    )
    .unwrap();
    compare(
        &owner,
        &labels,
        &refs,
        &case["states"][8]["state"],
        &graph["relations"],
        &case["states"][0]["state"]["relations"],
    );
    assert_eq!(owner.chords.as_ptr(), chord_address);
    assert_eq!(owner.measures[0].timing.slots.as_ptr(), slot_address);
    assert_eq!(
        owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

fn removal_state(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    reference: &Voice,
    chords: &[Chord],
) -> Value {
    let voice = actual_voice(owner, reference);
    let timing = &owner.measures[0].timing;
    json!({"manual_seed":owner.owner.graph.cached_chord_manual_status(chords[0].key),"voices":[{"label":"old_voice","id":voice.id,"same_measure":voice.measure == 0,
        "measure_registered":owner.voices.iter().any(|v| Arc::ptr_eq(&v.identity, &voice.identity)),
        "chords":voice.chord_ids.iter().map(|id|labels[id].clone()).collect::<Vec<_>>(),
        "slots":voice.slots.iter().map(|(id,s)|json!({"id":id,"chord":labels[&s.chord_id],"status":if s.continuing {"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>(),
        "termination":voice.termination.map(crate::music_time::format_delta),"excess":voice.excess.map(crate::music_time::format_fraction)}],
        "measure_voices":owner.voices.iter().filter(|v|v.measure == 0).map(|_|"old_voice").collect::<Vec<_>>(),
        "measure_chords":owner.chords.iter().filter(|c|c.measure == 0).map(|c|labels[&c.id].clone()).collect::<Vec<_>>(),
        "cached_chord_voices":chords.iter().map(|c|if owner.owner.graph.chord_has_voice(c.key, reference) {Some("old_voice")} else {None}).collect::<Vec<_>>(),
        "measure_abnormal":timing.abnormal,"expected":timing.expected.map(crate::music_time::format_fraction),
        "actual":timing.actual.map(crate::music_time::format_fraction),"excess":timing.excess.map(crate::music_time::format_fraction),
        "stack_slot_ids":timing.slots.iter().map(|s|s.id).collect::<Vec<_>>()})
}
#[test]
fn original_normal_chord_removal_retains_short_and_excess_actual_voice_owners() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-normal-removal/observed.json"
    ))
    .unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let (mut owner, labels, refs) = prepare(case);
        let chords = owner
            .chords
            .iter()
            .filter(|c| c.measure == 0)
            .cloned()
            .collect::<Vec<_>>();
        for (index, step) in case["states"].as_array().unwrap().iter().enumerate() {
            if index > 0 {
                let chord = &chords[(index - 1).min(3)];
                owner.owner.graph.remove_music_inter(chord.key).unwrap();
                crate::music_containers::refresh(
                    &mut owner.owner.graph,
                    &owner.measures,
                    &mut owner.chords,
                    &mut owner.voices,
                )
                .unwrap();
            }
            assert_eq!(
                removal_state(&owner, &labels, &refs[0], &chords),
                step["state"],
                "{} {}",
                case["name"],
                step["phase"]
            );
        }
    }
}

#[test]
fn failed_normal_refresh_keeps_actual_chords_voice_manual_clocks_and_retry_owner() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-normal-removal/observed.json"
    ))
    .unwrap();
    let case = &oracle["cases"][1];
    let (mut owner, labels, refs) = prepare(case);
    let chords = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
        .cloned()
        .collect::<Vec<_>>();
    for chord in &chords[..3] {
        owner.owner.graph.remove_music_inter(chord.key).unwrap();
        crate::music_containers::refresh(
            &mut owner.owner.graph,
            &owner.measures,
            &mut owner.chords,
            &mut owner.voices,
        )
        .unwrap();
    }
    owner.owner.graph.remove_music_inter(chords[3].key).unwrap();
    let graph = owner.owner.graph.snapshot();
    let layout = owner.layout();
    let before = removal_state(&owner, &labels, &refs[0], &chords);
    let chord_address = owner.chords.as_ptr();
    let slots = owner.measures[0].timing.slots.as_ptr();
    let identities = owner
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
    let saved = owner.chords.last().unwrap().id;
    owner.chords.last_mut().unwrap().id += 1000;
    let invalid_chords = format!("{:?}", owner.chords);
    assert_eq!(
        crate::music_containers::refresh(
            &mut owner.owner.graph,
            &owner.measures,
            &mut owner.chords,
            &mut owner.voices
        )
        .err()
        .unwrap(),
        "music_container_chord_identity_required"
    );
    assert_eq!(format!("{:?}", owner.chords), invalid_chords);
    owner.chords.last_mut().unwrap().id = saved;
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(owner.layout(), layout);
    assert_eq!(removal_state(&owner, &labels, &refs[0], &chords), before);
    assert_eq!(owner.chords.as_ptr(), chord_address);
    assert_eq!(owner.measures[0].timing.slots.as_ptr(), slots);
    assert!(
        owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .is_empty()
    );
    for (voice, identity) in owner.voices.iter().zip(identities) {
        assert!(Arc::ptr_eq(&voice.identity, &identity));
    }
    crate::music_containers::refresh(
        &mut owner.owner.graph,
        &owner.measures,
        &mut owner.chords,
        &mut owner.voices,
    )
    .unwrap();
    assert_eq!(
        removal_state(&owner, &labels, &refs[0], &chords),
        case["states"][4]["state"]
    );
    assert_eq!(
        owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

fn prepare_retired_owners(
    case: &Value,
) -> (OwnedMusic, BTreeMap<usize, String>, Vec<Voice>, Vec<Chord>) {
    let (mut owner, mut labels, mut refs) = prepare(case);
    let mut target_chords = owner
        .chords
        .iter()
        .filter(|c| c.measure == 1)
        .cloned()
        .collect::<Vec<_>>();
    target_chords.sort_by_key(|c| c.bounds[0]);
    for (index, chord) in target_chords.iter().enumerate() {
        labels.insert(chord.id, format!("chord{}", index + 4));
        for (member, head) in chord.members.iter().enumerate() {
            labels.insert(
                head["id"].as_u64().unwrap() as usize,
                format!("head{}_{}", index + 4, member),
            );
        }
    }
    let old = owner.voices.iter().position(|v| v.measure == 0).unwrap();
    set_owned_voice_id(&mut owner.owner.graph, &mut owner.voices[old], 7).unwrap();
    owner.voices[old].chord_ids.clear();
    refs.push(
        owner
            .voices
            .iter()
            .find(|v| v.measure == 1)
            .unwrap()
            .clone(),
    );
    (owner, labels, refs, target_chords)
}
fn prepare_retired_next(
    case: &Value,
) -> (OwnedMusic, BTreeMap<usize, String>, Vec<Voice>, Vec<Chord>) {
    let (mut owner, labels, refs, target_chords) = prepare_retired_owners(case);
    let source = owner
        .chords
        .iter()
        .find(|c| labels.get(&c.id).is_some_and(|label| label == "chord3"))
        .unwrap();
    owner
        .owner
        .graph
        .relations(
            1,
            crate::shared_sig::Phase::Symbol,
            vec![(
                source.key,
                target_chords[0].key,
                json!({"kind":"NextInVoiceRelation","manual":true}),
            )],
        )
        .unwrap();
    (owner, labels, refs, target_chords)
}
fn compare_retired_next(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    chords: &[Chord],
    expected: &Value,
    native_relations: &Value,
    original_relations: &Value,
) {
    compare(
        owner,
        labels,
        &refs[..1],
        &expected["source"],
        native_relations,
        original_relations,
    );
    let voice = actual_voice(owner, &refs[1]);
    let target = json!({"label":"target_voice","id":voice.id,"same_measure":voice.measure == 1,
        "measure_registered":owner.voices.iter().any(|v|Arc::ptr_eq(&v.identity, &voice.identity)),
        "chords":voice.chord_ids.iter().map(|id|labels[id].clone()).collect::<Vec<_>>(),
        "slots":voice.slots.iter().map(|(id,s)|json!({"id":id,"chord":labels[&s.chord_id],"status":if s.continuing {"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>(),
        "termination":voice.termination.map(crate::music_time::format_delta),"excess":voice.excess.map(crate::music_time::format_fraction)});
    assert_eq!(target, expected["target"]);
    let source = owner
        .chords
        .iter()
        .find(|c| labels.get(&c.id).is_some_and(|label| label == "chord3"))
        .unwrap();
    assert_eq!(
        json!(owner.owner.graph.chord_has_voice(source.key, &refs[0])),
        expected["source_ref_equal"]
    );
    assert_eq!(
        json!(owner.owner.graph.chord_has_voice(chords[0].key, &refs[1])),
        expected["target_ref_equal"]
    );
    let next = native_relations
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "NextInVoiceRelation")
        .unwrap();
    assert_eq!(expected["relation_registered"], true);
    assert_eq!(next["manual"], expected["relation_manual"]);
}
fn set_next_preferred(owner: &mut OwnedMusic, chord: &Chord, value: Value) {
    let snapshot = owner.owner.graph.snapshot();
    let mut node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == chord.id)
        .unwrap()
        .clone();
    node["preferred_voice_id"] = value;
    owner.owner.graph.vertex(chord.key, node, false).unwrap();
}
#[test]
fn original_next_refinement_reads_actual_retired_empty_voice_in_both_clock_cases() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-retired-next/observed.json"
    ))
    .unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let (mut owner, labels, refs, chords) = prepare_retired_next(case);
        let relations = owner.owner.graph.snapshot()["relations"].clone();
        let original_relations = &case["states"][0]["state"]["source"]["relations"];
        for step in case["states"].as_array().unwrap() {
            let phase = step["phase"].as_str().unwrap();
            if phase == "preferred" {
                set_next_preferred(&mut owner, &chords[0], json!(2));
            }
            if phase != "before" {
                refine_ordinary_system(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    1,
                )
                .unwrap();
            }
            compare_retired_next(
                &owner,
                &labels,
                &refs,
                &chords,
                &step["state"],
                &relations,
                original_relations,
            );
        }
    }
}

#[test]
fn failed_retired_next_refinement_keeps_actual_refs_clocks_and_retry_order() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-retired-next/observed.json"
    ))
    .unwrap();
    let case = &oracle["cases"][1];
    let (mut owner, labels, refs, chords) = prepare_retired_next(case);
    set_next_preferred(&mut owner, &chords[0], json!("invalid"));
    let graph = owner.owner.graph.snapshot();
    let layout = owner.layout();
    let source = state(&owner, &labels, &refs[..1]);
    let identities = owner
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
    let target = crate::shared_sig::Graph::retained_voice_observations(&owner.owner.graph);
    let voices = projection(&owner.voices, &owner.measures);
    let chord_address = owner.chords.as_ptr();
    let slots = owner
        .measures
        .iter()
        .map(|m| m.timing.slots.as_ptr())
        .collect::<Vec<_>>();
    assert_eq!(
        refine_ordinary_system(
            &mut owner.owner.graph,
            &mut owner.voices,
            &owner.measures,
            1
        )
        .unwrap_err(),
        "music_refine_preferred_ID_outside_native_range_pending"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(owner.layout(), layout);
    assert_eq!(state(&owner, &labels, &refs[..1]), source);
    assert_eq!(owner.owner.graph.retained_voice_observations(), target);
    assert_eq!(projection(&owner.voices, &owner.measures), voices);
    assert_eq!(owner.chords.as_ptr(), chord_address);
    assert_eq!(
        owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>(),
        slots
    );
    for (voice, identity) in owner.voices.iter().zip(identities) {
        assert!(Arc::ptr_eq(&voice.identity, &identity));
    }
    set_next_preferred(&mut owner, &chords[0], json!(2));
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    compare_retired_next(
        &owner,
        &labels,
        &refs,
        &chords,
        &case["states"][3]["state"],
        &graph["relations"],
        &case["states"][0]["state"]["source"]["relations"],
    );
    assert_eq!(
        owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

mod retired_tie_tests;

mod retired_beam_tests;

mod reset_rhythm_tests;

mod beam_rest_added_tests;
mod beam_rest_clear_tests;
mod beam_rest_linked_tests;
mod beam_rest_nearest_tests;
mod beam_rest_plain_tests;
mod selected_rhythm_tests;
mod staff_midline_tests;

mod beam_rest_polygon_tests;

mod beam_rest_competition_tests;

mod beam_rest_detect_tests;

mod tuplet_preparation_tests;

mod selected_preparation_tests;
mod tuplet_placement_tests;
mod tuplet_reuse_tests;

mod tuplet_getters_tests;

mod tuplet_raw_base_tests;

mod tuplet_generation_tests;

mod tuplet_stack_tests;

mod tuplet_sans_tests;

mod tuplet_beat_tests;
