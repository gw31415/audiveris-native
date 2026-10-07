// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::shared_sig::{Key, Phase};

fn original_tie() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-retired-tie/observed.json"
    ))
    .unwrap()
}
fn chord_named(owner: &OwnedMusic, labels: &BTreeMap<usize, String>, name: &str) -> Chord {
    owner
        .chords
        .iter()
        .find(|c| labels.get(&c.id).is_some_and(|label| label == name))
        .unwrap()
        .clone()
}
pub(super) fn add_tie(owner: &mut OwnedMusic, source: &Chord, target: &Chord) {
    let snapshot = owner.owner.graph.snapshot();
    let slur = Key {
        system: 1,
        phase: Phase::Symbol,
        local: snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["origin"]["phase"] == "SYMBOL")
            .filter_map(|n| n["origin"]["local"].as_u64())
            .max()
            .unwrap_or(0) as usize
            + 1,
    };
    let left = serde_json::from_value(source.members[0]["origin"].clone()).unwrap();
    let right = serde_json::from_value(target.members[0]["origin"].clone()).unwrap();
    owner
        .owner
        .graph
        .vertex(
            slur,
            json!({"kind":"SlurInter","is_tie":true,"manual":true,"grade":0.8}),
            false,
        )
        .unwrap();
    owner
        .owner
        .graph
        .relations(
            1,
            Phase::Symbol,
            vec![
                (
                    slur,
                    left,
                    json!({"kind":"SlurHeadRelation","side":"LEFT","manual":true}),
                ),
                (
                    slur,
                    right,
                    json!({"kind":"SlurHeadRelation","side":"RIGHT","manual":true}),
                ),
            ],
        )
        .unwrap();
}
pub(super) fn assign_cached_reference(
    owner: &mut OwnedMusic,
    chord: &Chord,
    reference: Option<&Voice>,
) {
    let snapshot = owner.owner.graph.snapshot();
    let mut node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == chord.id)
        .unwrap()
        .clone();
    node["voice"] = reference.map(|v| json!(v.id)).unwrap_or(Value::Null);
    owner.owner.graph.vertex(chord.key, node, false).unwrap();
    if let Some(voice) = reference {
        owner
            .owner
            .graph
            .bind_chord_to_live_voice(chord.key, voice)
            .unwrap();
    }
}
fn prepare_tie(case: &Value) -> (OwnedMusic, BTreeMap<usize, String>, Vec<Voice>, Vec<Chord>) {
    let (mut owner, labels, refs, chords) = prepare_retired_owners(case);
    let source = chord_named(&owner, &labels, "chord3");
    add_tie(&mut owner, &source, &chords[0]);
    (owner, labels, refs, chords)
}
fn source_refs(refs: &[Voice]) -> Vec<Voice> {
    let mut source = vec![refs[0].clone()];
    if refs.len() > 2 {
        source.push(refs[2].clone());
    }
    source
}
fn tied_id(owner: &OwnedMusic, target: &Chord) -> Option<usize> {
    owner
        .owner
        .graph
        .incoming_tied_voice_id(target.id, |id| {
            chord_voice_id(&owner.owner.graph, &owner.voices, &owner.measures, id)
        })
        .unwrap()
}
pub(super) fn keep_rows(known: &mut BTreeMap<String, Value>, rows: &Value, field: &str) {
    let current = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row[field].to_string(), row))
        .collect::<BTreeMap<_, _>>();
    for (key, old) in known.iter() {
        assert_eq!(*current.get(key).unwrap(), old);
    }
    known.extend(current.into_iter().map(|(key, row)| (key, row.clone())));
}
fn compare_tie(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    target: &Chord,
    expected: &Value,
    relations: &Value,
) {
    let source = source_refs(refs);
    compare(
        owner,
        labels,
        &source,
        &expected["source"],
        relations,
        &expected["source"]["relations"],
    );
    let voice = actual_voice(owner, &refs[1]);
    assert_eq!(
        json!({"label":"target_voice","id":voice.id,"same_measure":voice.measure == 1,
        "measure_registered":owner.voices.iter().any(|v|Arc::ptr_eq(&v.identity,&voice.identity)),
        "chords":voice.chord_ids.iter().map(|id|labels[id].clone()).collect::<Vec<_>>(),
        "slots":voice.slots.iter().map(|(id,s)|json!({"id":id,"chord":labels[&s.chord_id],"status":if s.continuing {"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>(),
        "termination":voice.termination.map(crate::music_time::format_delta),"excess":voice.excess.map(crate::music_time::format_fraction)}),
        expected["target"]
    );
    let chord = chord_named(owner, labels, "chord3");
    let fallback = chord_named(owner, labels, "chord0");
    assert_eq!(json!(tied_id(owner, target)), expected["tied_id"]);
    assert_eq!(
        json!(owner.owner.graph.chord_has_voice(chord.key, &refs[0])),
        expected["source_ref_equal"]
    );
    assert_eq!(
        json!(refs.len() > 2 && owner.owner.graph.chord_has_voice(fallback.key, &refs[2])),
        expected["fallback_ref_equal"]
    );
    assert_eq!(
        json!(owner.owner.graph.chord_has_voice(target.key, &refs[1])),
        expected["target_ref_equal"]
    );
    let graph = &owner.owner.graph;
    let snapshot = graph.snapshot();
    let head_id = chord.members[0]["id"].as_u64().unwrap();
    let parent_id = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "Containment" && e["target"] == head_id)
        .unwrap()["source"]
        .as_u64()
        .unwrap() as usize;
    assert_eq!(parent_id, chord.id);
    let parent_key = serde_json::from_value(
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == parent_id)
            .unwrap()["origin"]
            .clone(),
    )
    .unwrap();
    let head_parent = graph.chord_voice_owner(parent_key, &owner.voices).unwrap();
    let chord_voice = graph.chord_voice_owner(chord.key, &owner.voices).unwrap();
    let same = match (head_parent, chord_voice) {
        (Some(a), Some(b)) => Arc::ptr_eq(&a.identity, &b.identity),
        (None, None) => true,
        _ => false,
    };
    assert_eq!(json!(same), expected["head_ref_equal"]);
}
#[test]
fn original_retired_tie_keeps_identity_null_fallback_reused_numbers_and_reset() {
    for case in original_tie()["cases"].as_array().unwrap() {
        let (mut owner, labels, mut refs, chords) = prepare_tie(case);
        let source = chord_named(&owner, &labels, "chord3");
        let seed = chord_named(&owner, &labels, "chord0");
        let mut original_rows = BTreeMap::new();
        let mut native_rows = BTreeMap::new();
        for step in case["states"].as_array().unwrap() {
            let phase = step["phase"].as_str().unwrap();
            match phase {
                "reused" => {
                    let index = construct_owned_voice(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &owner.measures,
                        &seed,
                    )
                    .unwrap();
                    set_owned_voice_id(&mut owner.owner.graph, &mut owner.voices[index], 7)
                        .unwrap();
                    refs.push(owner.voices[index].clone());
                    add_tie(&mut owner, &seed, &chords[0]);
                }
                "null" => assign_cached_reference(&mut owner, &source, None),
                "restored" => {
                    let old = actual_voice(&owner, &refs[0]).clone();
                    assign_cached_reference(&mut owner, &source, Some(&old));
                }
                "next" => owner
                    .owner
                    .graph
                    .relations(
                        1,
                        Phase::Symbol,
                        vec![(
                            seed.key,
                            chords[0].key,
                            json!({"kind":"NextInVoiceRelation","manual":true}),
                        )],
                    )
                    .unwrap(),
                "preferred" => set_next_preferred(&mut owner, &chords[0], json!(2)),
                "measure_reset" => {
                    let address = owner.chords.as_ptr();
                    let slots = owner.measures[0].timing.slots.as_ptr();
                    reset_rhythm(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &mut owner.measures[0],
                        &owner.chords,
                        0,
                    )
                    .unwrap();
                    assert_eq!(owner.chords.as_ptr(), address);
                    assert_eq!(owner.measures[0].timing.slots.as_ptr(), slots);
                }
                _ => {}
            }
            let relations = owner.owner.graph.snapshot()["relations"].clone();
            if matches!(
                phase,
                "refined" | "null_refined" | "restored_refined" | "next" | "preferred"
            ) {
                refine_ordinary_system(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    1,
                )
                .unwrap();
            }
            keep_rows(
                &mut original_rows,
                &step["state"]["source"]["relations"],
                "label",
            );
            keep_rows(&mut native_rows, &relations, "id");
            compare_tie(
                &owner,
                &labels,
                &refs,
                &chords[0],
                &step["state"],
                &relations,
            );
        }
    }
}
#[test]
fn late_retired_tie_failure_keeps_real_owners_and_retries_cached_old_voice() {
    let oracle = original_tie();
    let case = &oracle["cases"][1];
    let (mut owner, labels, refs, chords) = prepare_tie(case);
    set_next_preferred(&mut owner, &chords[0], json!("invalid"));
    let graph = owner.owner.graph.snapshot();
    let layout = owner.layout();
    let source = state(&owner, &labels, &refs[..1]);
    let voices = projection(&owner.voices, &owner.measures);
    let pool = owner.owner.graph.retained_voice_observations();
    let identities = owner
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
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
    assert_eq!(projection(&owner.voices, &owner.measures), voices);
    assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
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
    set_next_preferred(&mut owner, &chords[0], Value::Null);
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    compare_tie(
        &owner,
        &labels,
        &refs,
        &chords[0],
        &case["states"][1]["state"],
        &graph["relations"],
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
