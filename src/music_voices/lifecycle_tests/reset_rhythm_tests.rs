// SPDX-License-Identifier: AGPL-3.0-or-later
use super::retired_beam_tests::{compare_owned, named, prepare_beam, source_state_with_names};
use super::retired_tie_tests::keep_rows;
use super::*;
use crate::shared_sig::Phase;

const NAMES: [&str; 7] = [
    "old_voice",
    "support_voice",
    "target_voice",
    "target_lower_voice",
    "new_voice",
    "rebuilt_voice0",
    "rebuilt_voice1",
];
fn original() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-reset-rhythm/observed.json"
    ))
    .unwrap()
}
pub(super) fn prepare(case: &Value) -> (OwnedMusic, BTreeMap<usize, String>, Vec<Voice>, usize) {
    let (mut owner, labels, mut refs, group) = prepare_beam(case);
    let seed = named(&owner, &labels, case["seed"].as_str().unwrap());
    let support = named(&owner, &labels, case["support_last"].as_str().unwrap());
    let target = named(&owner, &labels, case["target"].as_str().unwrap());
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    let i = construct_owned_voice(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        &seed,
    )
    .unwrap();
    set_owned_voice_id(&mut owner.owner.graph, &mut owner.voices[i], 7).unwrap();
    refs.push(owner.voices[i].clone());
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    owner
        .owner
        .graph
        .relations(
            1,
            Phase::Symbol,
            vec![(
                support.key,
                target.key,
                json!({"kind":"NextInVoiceRelation","manual":true}),
            )],
        )
        .unwrap();
    set_next_preferred(&mut owner, &target, json!(1));
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    (owner, labels, refs, group)
}
pub(super) fn append_rebuilt(owner: &OwnedMusic, refs: &mut Vec<Voice>) {
    let rebuilt = owner
        .voices
        .iter()
        .filter(|v| v.measure == 0)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(rebuilt.len(), 2);
    for v in &rebuilt {
        assert!(!refs.iter().any(|r| Arc::ptr_eq(&r.identity, &v.identity)));
    }
    refs.extend(rebuilt);
}
#[test]
fn original_measure_reset_rebuilds_actual_voices_before_stack_slots_and_system_refinement() {
    for case in original()["cases"].as_array().unwrap() {
        let (mut owner, labels, mut refs, group) = prepare(case);
        let baseline_slots = owner
            .chords
            .iter()
            .filter(|c| c.measure == 0)
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
            .collect::<BTreeMap<_, _>>();
        let address = owner.chords.as_ptr();
        let target_slots = owner.measures[1].timing.slots.as_ptr();
        let original_slots = owner.measures[0].timing.slots.as_ptr();
        let expected = owner.measures[0].timing.expected;
        let relations = owner.owner.graph.snapshot()["relations"].clone();
        let mut original_rows = BTreeMap::new();
        for step in case["states"].as_array().unwrap() {
            let phase = step["phase"].as_str().unwrap();
            match phase {
                "before_reset" => {}
                "measure_reset" => {
                    reset_rhythm(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &mut owner.measures[0],
                        &owner.chords,
                        0,
                    )
                    .unwrap();
                    assert_eq!(owner.measures[0].timing.slots.as_ptr(), original_slots);
                    let before = source_state_with_names(&owner, &labels, &refs, &NAMES);
                    let graph = owner.owner.graph.snapshot();
                    assert_eq!(
                        refine_ordinary_system(
                            &mut owner.owner.graph,
                            &mut owner.voices,
                            &owner.measures,
                            1
                        )
                        .unwrap_err(),
                        "music_refine_beam_predecessor_owned_voice_required"
                    );
                    assert_eq!(
                        source_state_with_names(&owner, &labels, &refs, &NAMES),
                        before
                    );
                    assert_eq!(owner.owner.graph.snapshot(), graph);
                }
                "stack_reset" => reset_stack_rhythm(&mut owner.measures[0], expected),
                "measure_processed" => {
                    crate::music_xml::stack_rhythm::Context {
                        graph: &mut owner.owner.graph,
                        voices: &mut owner.voices,
                        measures: &mut owner.measures,
                        chords: &owner.chords,
                        interline: owner.owner.interline,
                    }
                    .process_measure(0)
                    .unwrap();
                    append_rebuilt(&owner, &mut refs);
                }
                "stack_slots" => {
                    generate_slots(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &mut owner.measures[0],
                        0,
                    )
                    .unwrap();
                    read_stack_actual_duration(
                        &owner.owner.graph,
                        &owner.voices,
                        &mut owner.measures[0],
                        0,
                    )
                    .unwrap();
                }
                "full_stack" => {
                    owner.reprocess_stack_rhythm(1).unwrap();
                    append_rebuilt(&owner, &mut refs);
                }
                "refined" => refine_ordinary_system(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    1,
                )
                .unwrap(),
                other => panic!("unobserved phase {other}"),
            }
            assert_eq!(owner.chords.as_ptr(), address);
            assert_eq!(owner.measures[1].timing.slots.as_ptr(), target_slots);
            assert_eq!(owner.measures[0].timing.expected, expected);
            assert_eq!(step["expected_object_same"], true);
            keep_rows(
                &mut original_rows,
                &step["state"]["source"]["relations"],
                "label",
            );
            compare_owned(
                &owner,
                &labels,
                (&refs, &NAMES, Some(&baseline_slots)),
                group,
                case,
                step,
                &relations,
            );
        }
    }
}
#[test]
fn selected_rhythm_failure_keeps_actual_chords_retired_owners_and_clock_state_for_retry() {
    let oracle = original();
    let case = &oracle["cases"][3];
    let (mut owner, labels, mut refs, group) = prepare(case);
    let baseline_slots = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
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
        .collect::<BTreeMap<_, _>>();
    let seed = named(&owner, &labels, case["seed"].as_str().unwrap());
    let i = owner.chords.iter().position(|c| c.id == seed.id).unwrap();
    let duplicate = owner.chords[i].members[0].clone();
    owner.chords[i].members.push(duplicate);
    let actual_members = owner.chords[i].members.clone();
    let address = owner.chords.as_ptr();
    let graph = owner.owner.graph.snapshot();
    let voices = projection(&owner.voices, &owner.measures);
    let clocks = source_state_with_names(&owner, &labels, &refs, &NAMES);
    let pool = owner.owner.graph.retained_voice_observations();
    let layout = owner.layout();
    let slots = owner
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
        owner.reprocess_stack_rhythm(1).unwrap_err(),
        "musicxml_actual_chord_members_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(projection(&owner.voices, &owner.measures), voices);
    assert_eq!(
        source_state_with_names(&owner, &labels, &refs, &NAMES),
        clocks
    );
    assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
    assert_eq!(owner.layout(), layout);
    assert_eq!(owner.chords.as_ptr(), address);
    assert_eq!(owner.chords[i].members, actual_members);
    assert_eq!(
        owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>(),
        slots
    );
    for (v, id) in owner.voices.iter().zip(identities) {
        assert!(Arc::ptr_eq(&v.identity, &id));
    }
    owner.chords[i].members.pop();
    owner.reprocess_stack_rhythm(1).unwrap();
    append_rebuilt(&owner, &mut refs);
    compare_owned(
        &owner,
        &labels,
        (&refs, &NAMES, Some(&baseline_slots)),
        group,
        case,
        &case["states"][2],
        &graph["relations"],
    );
}

fn original_page() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-page-rhythm/observed.json"
    ))
    .unwrap()
}
#[test]
fn original_page_stack_rebuild_precedes_system_refinement_and_preserves_actual_relationships() {
    for case in original_page()["cases"].as_array().unwrap() {
        let (mut owner, labels, mut refs, group) = prepare(case);
        let slots = owner
            .chords
            .iter()
            .filter(|c| c.measure == 0)
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
            .collect::<BTreeMap<_, _>>();
        let address = owner.chords.as_ptr();
        let target_slots = owner.measures[1].timing.slots.as_ptr();
        let relations = owner.owner.graph.snapshot()["relations"].clone();
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
                other => panic!("unobserved Page phase {other}"),
            }
            assert_eq!(owner.chords.as_ptr(), address);
            assert_eq!(owner.measures[1].timing.slots.as_ptr(), target_slots);
            assert_eq!(step["expected_object_same"], true);
            compare_owned(
                &owner,
                &labels,
                (&refs, &NAMES, Some(&slots)),
                group,
                case,
                step,
                &relations,
            );
        }
    }
}
#[test]
fn late_page_preferred_failure_rolls_back_rebuilt_actual_owners_chords_and_slots() {
    let original = original_page();
    let case = &original["cases"][1];
    let (mut owner, labels, mut refs, group) = prepare(case);
    let target = named(&owner, &labels, case["target"].as_str().unwrap());
    let slots = owner
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
        .collect::<BTreeMap<_, _>>();
    set_next_preferred(&mut owner, &target, json!("invalid"));
    let graph = owner.owner.graph.snapshot();
    let voices = projection(&owner.voices, &owner.measures);
    let clocks = source_state_with_names(&owner, &labels, &refs, &NAMES);
    let pool = owner.owner.graph.retained_voice_observations();
    let report = owner.owner.prepared.report.clone();
    let layout = owner.layout();
    let actual_chords = owner.chords.clone();
    let address = owner.chords.as_ptr();
    let slot_storage = owner
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
        owner.reprocess_page_stack_rhythm(1).unwrap_err(),
        "music_refine_preferred_ID_outside_native_range_pending"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(projection(&owner.voices, &owner.measures), voices);
    assert_eq!(
        source_state_with_names(&owner, &labels, &refs, &NAMES),
        clocks
    );
    assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
    assert_eq!(owner.owner.prepared.report, report);
    assert_eq!(owner.layout(), layout);
    assert_eq!(owner.chords.as_ptr(), address);
    assert_eq!(
        owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>(),
        slot_storage
    );
    for (ch, old) in owner.chords.iter().zip(actual_chords) {
        assert_eq!(ch.members, old.members);
        assert_eq!(ch.stem, old.stem);
        assert_eq!(ch.bounds, old.bounds);
        assert_eq!(ch.measure, old.measure);
        assert_eq!(ch.key, old.key);
        assert!(Arc::ptr_eq(
            owner.owner.graph.chord_slot_identity(ch.key).unwrap(),
            &slots[&ch.id]
        ));
    }
    for (v, id) in owner.voices.iter().zip(identities) {
        assert!(Arc::ptr_eq(&v.identity, &id));
    }
    set_next_preferred(&mut owner, &target, json!(1));
    owner.reprocess_page_stack_rhythm(1).unwrap();
    append_rebuilt(&owner, &mut refs);
    let source_slots = slots
        .into_iter()
        .filter(|(id, _)| owner.chords.iter().any(|c| c.id == *id && c.measure == 0))
        .collect::<BTreeMap<_, _>>();
    compare_owned(
        &owner,
        &labels,
        (&refs, &NAMES, Some(&source_slots)),
        group,
        case,
        &case["states"][2],
        &graph["relations"],
    );
}

#[test]
fn original_null_unselected_source_allows_next_stack_rebuild_then_refuses_page_refinement() {
    use super::retired_beam_tests::{measure_state, voice_row};
    let oracle: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-page-null-rhythm/observed.json"
    ))
    .unwrap();
    let case = &oracle["cases"][0];
    let (mut owner, labels, mut refs, group) = prepare(case);
    let names = [
        "old_voice",
        "support_voice",
        "target_voice",
        "target_lower_voice",
        "new_voice",
        "rebuilt_target0",
        "rebuilt_target1",
        "rebuilt_target2",
        "rebuilt_target3",
    ];
    let slots = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
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
        .collect::<BTreeMap<_, _>>();
    let relations = owner.owner.graph.snapshot()["relations"].clone();
    compare_owned(
        &owner,
        &labels,
        (&refs, &names, Some(&slots)),
        group,
        case,
        &case["states"][0],
        &relations,
    );
    reset_rhythm(
        &mut owner.owner.graph,
        &mut owner.voices,
        &mut owner.measures[0],
        &owner.chords,
        0,
    )
    .unwrap();
    compare_owned(
        &owner,
        &labels,
        (&refs, &names, Some(&slots)),
        group,
        case,
        &case["states"][1],
        &relations,
    );
    let graph = owner.owner.graph.snapshot();
    let projection_before = projection(&owner.voices, &owner.measures);
    let pool = owner.owner.graph.retained_voice_observations();
    let clock = source_state_with_names(&owner, &labels, &refs, &names);
    let report = owner.owner.prepared.report.clone();
    let layout = owner.layout();
    let address = owner.chords.as_ptr();
    let storage = owner
        .measures
        .iter()
        .map(|m| m.timing.slots.as_ptr())
        .collect::<Vec<_>>();
    let target_slots = owner
        .chords
        .iter()
        .filter(|c| c.measure == 1)
        .map(|c| {
            (
                c.key,
                owner
                    .owner
                    .graph
                    .chord_slot_identity(c.key)
                    .unwrap()
                    .clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        case["states"][2]["original_failure"],
        "NullPointerException"
    );
    assert_eq!(
        owner.reprocess_page_stack_rhythm(2).unwrap_err(),
        "music_refine_beam_predecessor_owned_voice_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(
        projection(&owner.voices, &owner.measures),
        projection_before
    );
    assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
    assert_eq!(
        source_state_with_names(&owner, &labels, &refs, &names),
        clock
    );
    assert_eq!(owner.owner.prepared.report, report);
    assert_eq!(owner.layout(), layout);
    assert_eq!(owner.chords.as_ptr(), address);
    assert_eq!(
        owner
            .measures
            .iter()
            .map(|m| m.timing.slots.as_ptr())
            .collect::<Vec<_>>(),
        storage
    );
    for (key, identity) in &target_slots {
        assert!(Arc::ptr_eq(
            owner.owner.graph.chord_slot_identity(*key).unwrap(),
            identity
        ));
    }
    assert_eq!(
        owner.reprocess_selected_stack_rhythms(&[2, 1]).unwrap_err(),
        "music_refine_beam_predecessor_owned_voice_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), graph);
    assert_eq!(
        projection(&owner.voices, &owner.measures),
        projection_before
    );
    owner.reprocess_stack_rhythm(2).unwrap();
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
    compare_owned(
        &owner,
        &labels,
        (&refs, &names, Some(&slots)),
        group,
        case,
        &case["states"][2],
        &relations,
    );
    assert_eq!(
        json!(
            refs[5..]
                .iter()
                .map(|r| voice_row(&owner, r, &refs, &names, &labels, 1))
                .collect::<Vec<_>>()
        ),
        case["states"][2]["state"]["target_rebuilt"]
    );
    let mut target_state = measure_state(&owner, &labels, &refs, &names, 1);
    for row in target_state["chords"].as_array_mut().unwrap() {
        let chord = owner
            .chords
            .iter()
            .find(|c| labels[&c.id] == row["label"])
            .unwrap();
        let old = &target_slots
            .iter()
            .find(|(key, _)| *key == chord.key)
            .unwrap()
            .1;
        row["same_baseline_slot"] = json!(Arc::ptr_eq(
            owner.owner.graph.chord_slot_identity(chord.key).unwrap(),
            old
        ));
    }
    for (field, wanted) in case["states"][2]["state"]["target_state"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, _)| k.as_str() != "relations")
    {
        assert_eq!(
            &target_state[field], wanted,
            "target rebuilt before null-source failure: {field}"
        );
    }
    let staged = owner.owner.graph.snapshot();
    assert_eq!(
        refine_ordinary_system(
            &mut owner.owner.graph,
            &mut owner.voices,
            &owner.measures,
            1
        )
        .unwrap_err(),
        "music_refine_beam_predecessor_owned_voice_required"
    );
    assert_eq!(owner.owner.graph.snapshot(), staged);
}
