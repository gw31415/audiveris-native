// SPDX-License-Identifier: AGPL-3.0-or-later
use super::reset_rhythm_tests::{append_rebuilt, prepare};
use super::retired_beam_tests::{compare_owned, measure_state, named};
use super::retired_tie_tests::keep_rows;
use super::*;
const NAMES: [&str; 11] = [
    "old_voice",
    "support_voice",
    "target_voice",
    "target_lower_voice",
    "new_voice",
    "rebuilt_voice0",
    "rebuilt_voice1",
    "rebuilt_target0",
    "rebuilt_target1",
    "rebuilt_target2",
    "rebuilt_target3",
];
fn original() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-selected-rhythm/observed.json"
    ))
    .unwrap()
}
fn append_target(owner: &OwnedMusic, refs: &mut Vec<Voice>) {
    let rebuilt = owner
        .voices
        .iter()
        .filter(|v| v.measure == 1)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(rebuilt.len(), 4);
    for v in &rebuilt {
        assert!(!refs.iter().any(|r| Arc::ptr_eq(&r.identity, &v.identity)));
    }
    refs.extend(rebuilt);
}
fn baseline(owner: &OwnedMusic) -> BTreeMap<usize, Arc<()>> {
    owner
        .chords
        .iter()
        .map(|c| {
            (
                c.id,
                owner
                    .owner
                    .graph
                    .chord_slot_identity(c.key)
                    .unwrap()
                    .clone(),
            )
        })
        .collect()
}
fn compare(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    slots: &BTreeMap<usize, Arc<()>>,
    group: usize,
    case: &Value,
    step: &Value,
) {
    let relations = owner.owner.graph.snapshot()["relations"].clone();
    let source = slots
        .iter()
        .filter(|(id, _)| owner.chords.iter().any(|c| c.id == **id && c.measure == 0))
        .map(|(id, s)| (*id, s.clone()))
        .collect::<BTreeMap<_, _>>();
    compare_owned(
        owner,
        labels,
        (refs, &NAMES, Some(&source)),
        group,
        case,
        step,
        &relations,
    );
    if !step["state"]["target_state"].is_null() {
        let mut target = measure_state(owner, labels, refs, &NAMES, 1);
        for row in target["chords"].as_array_mut().unwrap() {
            let chord = owner
                .chords
                .iter()
                .find(|c| labels[&c.id] == row["label"])
                .unwrap();
            row["same_baseline_slot"] = json!(
                owner
                    .owner
                    .graph
                    .chord_slot_identity(chord.key)
                    .is_some_and(|s| Arc::ptr_eq(s, &slots[&chord.id]))
            );
        }
        for (field, wanted) in step["state"]["target_state"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.as_str() != "relations")
        {
            assert_eq!(&target[field], wanted, "{} target: {field}", step["phase"]);
        }
    }
}
#[test]
fn original_selected_impact_preserves_first_encounters_duplicates_and_source_first_real_owners() {
    for case in original()["cases"].as_array().unwrap() {
        let (mut owner, labels, mut refs, group) = prepare(case);
        let slots = baseline(&owner);
        let address = owner.chords.as_ptr();
        let relations = owner.owner.graph.snapshot()["relations"].clone();
        let mut original_rows = BTreeMap::new();
        for step in case["states"].as_array().unwrap() {
            match step["phase"].as_str().unwrap() {
                "before_reset" => {}
                "measure_reset" => reset_rhythm(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &mut owner.measures[0],
                    &owner.chords,
                    0,
                )
                .unwrap(),
                "page_stack" => {
                    owner.reprocess_page_stack_rhythm(1).unwrap();
                    append_rebuilt(&owner, &mut refs);
                }
                "page_target" => {
                    owner.reprocess_page_stack_rhythm(2).unwrap();
                    append_target(&owner, &mut refs);
                }
                "impact" => {
                    owner.reprocess_selected_stack_rhythms(&[1, 2, 1]).unwrap();
                    append_rebuilt(&owner, &mut refs);
                    append_target(&owner, &mut refs);
                }
                other => panic!("unobserved selected phase {other}"),
            }
            assert_eq!(owner.chords.as_ptr(), address);
            assert_eq!(owner.owner.graph.snapshot()["relations"], relations);
            assert_eq!(step["expected_object_same"], true);
            keep_rows(
                &mut original_rows,
                &step["state"]["source"]["relations"],
                "label",
            );
            compare(&owner, &labels, &refs, &slots, group, case, step);
        }
        assert_eq!(
            owner
                .owner
                .graph
                .retained_voice_observations()
                .as_array()
                .unwrap()
                .len(),
            5,
            "duplicate source stack must not construct and retire extra actual Voices"
        );
    }
}
#[test]
fn later_selected_actual_member_failure_restores_first_stack_objects_and_retries() {
    let oracle = original();
    let case = &oracle["cases"][3];
    let (mut owner, labels, mut refs, group) = prepare(case);
    let slots = baseline(&owner);
    reset_rhythm(
        &mut owner.owner.graph,
        &mut owner.voices,
        &mut owner.measures[0],
        &owner.chords,
        0,
    )
    .unwrap();
    let target = named(&owner, &labels, case["target"].as_str().unwrap());
    let i = owner.chords.iter().position(|c| c.id == target.id).unwrap();
    let duplicate = owner.chords[i].members[0].clone();
    owner.chords[i].members.push(duplicate);
    let before = owner.owner.graph.snapshot();
    let voices = projection(&owner.voices, &owner.measures);
    let pool = owner.owner.graph.retained_voice_observations();
    let layout = owner.layout();
    let report = owner.owner.prepared.report.clone();
    let chords = owner.chords.clone();
    let address = owner.chords.as_ptr();
    let storage = owner
        .measures
        .iter()
        .map(|m| m.timing.slots.as_ptr())
        .collect::<Vec<_>>();
    let identities = owner
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        owner
            .reprocess_selected_stack_rhythms(&[1, 2, 1])
            .unwrap_err(),
        "musicxml_actual_chord_members_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), before);
    assert_eq!(projection(&owner.voices, &owner.measures), voices);
    assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
    assert_eq!(owner.layout(), layout);
    assert_eq!(owner.owner.prepared.report, report);
    assert_eq!(owner.chords.as_ptr(), address);
    assert_eq!(
        owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>(),
        storage
    );
    for (v, id) in owner.voices.iter().zip(identities) {
        assert!(Arc::ptr_eq(&v.identity, &id));
    }
    for (ch, old) in owner.chords.iter().zip(chords) {
        assert_eq!(ch.members, old.members);
        assert_eq!(ch.stem, old.stem);
        assert_eq!(ch.bounds, old.bounds);
        assert_eq!(ch.key, old.key);
        assert_eq!(ch.measure, old.measure);
        if ch.measure == 0 {
            assert!(owner.owner.graph.chord_slot_identity(ch.key).is_none());
        } else {
            assert!(Arc::ptr_eq(
                owner.owner.graph.chord_slot_identity(ch.key).unwrap(),
                &slots[&ch.id]
            ));
        }
    }
    owner.chords[i].members.pop();
    owner.reprocess_selected_stack_rhythms(&[1, 2, 1]).unwrap();
    append_rebuilt(&owner, &mut refs);
    append_target(&owner, &mut refs);
    compare(
        &owner,
        &labels,
        &refs,
        &slots,
        group,
        case,
        &case["states"][2],
    );
}
