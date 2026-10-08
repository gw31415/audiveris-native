// SPDX-License-Identifier: AGPL-3.0-or-later
use super::retired_tie_tests::{add_tie, assign_cached_reference, keep_rows};
use super::*;
use crate::shared_sig::Phase;

fn original_beam() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-retired-beam/observed.json"
    ))
    .unwrap()
}
pub(super) fn named(owner: &OwnedMusic, labels: &BTreeMap<usize, String>, name: &str) -> Chord {
    owner
        .chords
        .iter()
        .find(|ch| labels[&ch.id] == name)
        .unwrap()
        .clone()
}
fn voice_name<'a>(voice: &Voice, refs: &[Voice], names: &'a [&str]) -> &'a str {
    names[refs
        .iter()
        .position(|r| Arc::ptr_eq(&r.identity, &voice.identity))
        .unwrap()]
}
pub(super) fn voice_row(
    owner: &OwnedMusic,
    reference: &Voice,
    refs: &[Voice],
    names: &[&str],
    labels: &BTreeMap<usize, String>,
    measure: usize,
) -> Value {
    let v = actual_voice(owner, reference);
    json!({"label":voice_name(v,refs,names),"id":v.id,"same_measure":v.measure==measure,
        "measure_registered":owner.voices.iter().any(|w|Arc::ptr_eq(&w.identity,&v.identity)),
        "chords":v.chord_ids.iter().map(|id|labels[id].clone()).collect::<Vec<_>>(),
        "slots":v.slots.iter().map(|(id,s)|json!({"id":id,"chord":labels[&s.chord_id],"status":if s.continuing{"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>(),
        "termination":v.termination.map(crate::music_time::format_delta),"excess":v.excess.map(crate::music_time::format_fraction)})
}
const NAMES: [&str; 5] = [
    "old_voice",
    "support_voice",
    "target_voice",
    "target_lower_voice",
    "new_voice",
];
pub(super) fn source_state(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
) -> Value {
    source_state_with_names(owner, labels, refs, &NAMES)
}
pub(super) fn source_state_with_names(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    names: &[&str],
) -> Value {
    measure_state(owner, labels, refs, names, 0)
}
pub(super) fn measure_state(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    names: &[&str],
    mi: usize,
) -> Value {
    let graph = &owner.owner.graph;
    let snap = graph.snapshot();
    let mut chords = owner
        .chords
        .iter()
        .filter(|c| c.measure == mi)
        .collect::<Vec<_>>();
    chords.sort_by_key(|c| (c.bounds[0], c.bounds[1]));
    let rows=chords.iter().map(|ch|{
        let node=snap["inters"].as_array().unwrap().iter().find(|n|n["id"]==ch.id).unwrap();
        let baseline=refs.iter().filter(|v|v.measure==mi).take(2).flat_map(|v|v.slots.iter()).find(|(_,s)|s.chord_id==ch.id&&!s.continuing).map(|(id,_)|json!(id)).unwrap_or(Value::Null);
        let v=graph.chord_voice_owner(ch.key,&owner.voices).unwrap();
        json!({"label":labels[&ch.id],"registered":true,"same_measure":ch.measure==mi,"manual":node["manual"]==true,
            "members":ch.members.iter().map(|n|labels[&(n["id"].as_u64().unwrap() as usize)].clone()).collect::<Vec<_>>(),
            "voice":v.map(|v|voice_name(v,refs,names)),"same_baseline_slot":node["slot_id"]==baseline,
            "slot_id":node["slot_id"],"time_offset":node["time_offset"],"duration":graph.chord_duration(ch.key).unwrap(),"end_time":graph.chord_end_time(ch.key).unwrap()})
    }).collect::<Vec<_>>();
    let source_refs = refs
        .iter()
        .filter(|r| actual_voice(owner, r).measure == mi)
        .map(|r| voice_row(owner, r, refs, names, labels, mi))
        .collect::<Vec<_>>();
    let timing = &owner.measures[mi].timing;
    json!({"chords":rows,"voices":source_refs,
        "measure_voices":owner.voices.iter().filter(|v|v.measure==mi).map(|v|voice_name(v,refs,names)).collect::<Vec<_>>(),
        "measure_chords":owner.chords.iter().filter(|c|c.measure==mi).map(|c|labels[&c.id].clone()).collect::<Vec<_>>(),
        "measure_abnormal":timing.abnormal,"expected":timing.expected.map(crate::music_time::format_fraction),"actual":timing.actual.map(crate::music_time::format_fraction),
        "excess":timing.excess.map(crate::music_time::format_fraction),"stack_slot_ids":timing.slots.iter().map(|s|s.id).collect::<Vec<_>>()})
}
pub(super) fn prepare_beam(
    case: &Value,
) -> (OwnedMusic, BTreeMap<usize, String>, Vec<Voice>, usize) {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-parallel-voices/parallel-beam.png"
    ))
    .unwrap();
    let mut owner = crate::music_xml::recognize(&image).unwrap().owner;
    let mut labels = BTreeMap::new();
    for row in case["geometry"].as_array().unwrap() {
        let ch = owner
            .chords
            .iter()
            .find(|ch| json!(ch.bounds) == row["bounds"])
            .unwrap();
        let label = row["label"].as_str().unwrap();
        labels.insert(ch.id, label.into());
        for (i, head) in ch.members.iter().enumerate() {
            labels.insert(
                head["id"].as_u64().unwrap() as usize,
                format!("head{}_{}", label.strip_prefix("chord").unwrap(), i),
            );
        }
    }
    let seed = named(&owner, &labels, case["seed"].as_str().unwrap());
    let last = named(&owner, &labels, case["last"].as_str().unwrap());
    let support = named(&owner, &labels, case["support_last"].as_str().unwrap());
    let target = named(&owner, &labels, case["target"].as_str().unwrap());
    let old = owner
        .voices
        .iter()
        .position(|v| v.measure == 0 && v.chord_ids.contains(&seed.id))
        .unwrap();
    let support_index = owner
        .voices
        .iter()
        .position(|v| v.measure == 0 && v.chord_ids.contains(&support.id))
        .unwrap();
    let target_index = owner
        .voices
        .iter()
        .position(|v| v.measure == 1 && v.chord_ids.contains(&target.id))
        .unwrap();
    let lower = owner
        .voices
        .iter()
        .position(|v| v.measure == 1 && v.chord_ids.first() != Some(&target.id))
        .unwrap();
    let refs = vec![
        owner.voices[old].clone(),
        owner.voices[support_index].clone(),
        owner.voices[target_index].clone(),
        owner.voices[lower].clone(),
    ];
    for (ch, field, value) in [
        (&seed, "manual", json!(true)),
        (&last, "time_offset", case["declared_last_offset"].clone()),
    ] {
        let snap = owner.owner.graph.snapshot();
        let mut node = snap["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == ch.id)
            .unwrap()
            .clone();
        node[field] = value;
        owner.owner.graph.vertex(ch.key, node, false).unwrap();
    }
    check_duration(
        &owner.owner.graph,
        &mut owner.voices[old],
        &mut owner.measures[0],
    )
    .unwrap();
    set_owned_voice_id(&mut owner.owner.graph, &mut owner.voices[old], 7).unwrap();
    owner.voices[old].reset_chords();
    add_tie(&mut owner, &support, &target);
    let snap = owner.owner.graph.snapshot();
    let beam = snap["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["kind"] == "BeamInter" && n["bounds"] == case["beam_bounds"])
        .unwrap();
    let stem_id = snap["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "ChordStemRelation" && e["source"] == target.id)
        .unwrap()["target"]
        .as_u64()
        .unwrap();
    let stem = snap["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == stem_id)
        .unwrap();
    owner
        .owner
        .graph
        .relations(
            1,
            Phase::Symbol,
            vec![(
                serde_json::from_value(beam["origin"].clone()).unwrap(),
                serde_json::from_value(stem["origin"].clone()).unwrap(),
                json!({"kind":"BeamStemRelation","beam_portion":"LEFT","manual":true}),
            )],
        )
        .unwrap();
    let group = chord_group(&owner.owner.graph.snapshot(), target.id)
        .unwrap()
        .unwrap();
    (owner, labels, refs, group)
}
fn compare_beam(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    refs: &[Voice],
    group_id: usize,
    case: &Value,
    step: &Value,
    relations: &Value,
) {
    compare_owned(
        owner,
        labels,
        (refs, &NAMES, None),
        group_id,
        case,
        step,
        relations,
    );
}
type SlotOwners = BTreeMap<usize, Arc<()>>;
pub(super) fn compare_owned(
    owner: &OwnedMusic,
    labels: &BTreeMap<usize, String>,
    references: (&[Voice], &[&str], Option<&SlotOwners>),
    group_id: usize,
    case: &Value,
    step: &Value,
    relations: &Value,
) {
    let (refs, names, slots) = references;
    let phase = step["phase"].as_str().unwrap();
    let expected = &step["state"];
    let mut actual = source_state_with_names(owner, labels, refs, names);
    if let Some(slots) = slots {
        for row in actual["chords"].as_array_mut().unwrap() {
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
    }
    for (key, wanted) in expected["source"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, _)| k.as_str() != "relations")
    {
        assert_eq!(&actual[key], wanted, "{phase}: {key}");
    }
    assert_eq!(owner.owner.graph.snapshot()["relations"], *relations);
    assert_eq!(
        json!([
            voice_row(owner, &refs[2], refs, names, labels, 1),
            voice_row(owner, &refs[3], refs, names, labels, 1)
        ]),
        expected["target"],
        "{phase}: target"
    );
    let projected = beam_groups(&owner.owner.graph, &owner.voices, &owner.measures).unwrap();
    let projected_group = projected
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["id"] == group_id)
        .unwrap();
    assert_eq!(
        projected_group["voice"], expected["group_voice_id"],
        "{phase}: actual group getter"
    );
    let mut source_chords = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
        .collect::<Vec<_>>();
    source_chords.sort_by_key(|c| (c.bounds[0], c.bounds[1]));
    assert_eq!(
        json!(
            source_chords
                .iter()
                .map(
                    |ch| chord_voice_id(&owner.owner.graph, &owner.voices, &owner.measures, ch.id)
                        .unwrap()
                )
                .collect::<Vec<_>>()
        ),
        expected["cached_chord_voice_ids"],
        "{phase}: current cached owner numbers"
    );
    let groups = beam_group_chords(&owner.owner.graph).unwrap();
    let group = groups
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["id"] == group_id)
        .unwrap();
    let rows=group["chords"].as_array().unwrap().iter().map(|c|{
        let chord=owner.chords.iter().find(|ch|ch.id==c["id"].as_u64().unwrap() as usize).unwrap();
        let voice=owner.owner.graph.chord_voice_owner(chord.key,&owner.voices).unwrap();
        json!({"chord":labels[&chord.id],"previous_measure":chord.measure==0,"voice":voice.map(|v|voice_name(v,refs,names))})
    }).collect::<Vec<_>>();
    assert_eq!(
        rows.first()
            .map(|r| r["voice"].clone())
            .unwrap_or(Value::Null),
        expected["group_voice"],
        "{phase}: actual first-Chord Voice identity"
    );
    assert_eq!(
        json!(rows),
        expected["group_chords"],
        "{phase}: group order and refs"
    );
    assert_eq!(
        json!(rows.iter().any(|r| r["previous_measure"] == true)),
        expected["group_contains_previous"]
    );
    let target = named(owner, labels, case["target"].as_str().unwrap());
    assert_eq!(
        json!(chord_group(&owner.owner.graph.snapshot(), target.id).unwrap() == Some(group_id)),
        expected["same_target_group"]
    );
    let group_rows = group["chords"].as_array().unwrap();
    let at = group_rows
        .iter()
        .position(|c| c["id"] == target.id)
        .unwrap();
    let predecessor = owner
        .chords
        .iter()
        .find(|ch| ch.id == group_rows[at - 1]["id"].as_u64().unwrap() as usize)
        .unwrap();
    assert_eq!(labels[&predecessor.id], expected["predecessor"]);
    assert_eq!(
        json!(owner.owner.graph.chord_has_voice(predecessor.key, &refs[0])),
        expected["predecessor_ref_old"]
    );
    assert_eq!(
        json!(
            owner
                .owner
                .graph
                .incoming_tied_voice_id(target.id, |id| chord_voice_id(
                    &owner.owner.graph,
                    &owner.voices,
                    &owner.measures,
                    id
                ))
                .unwrap()
        ),
        expected["tied_id"]
    );
}
#[test]
fn original_retired_beam_predecessor_keeps_real_group_owners_reuse_order_and_reset() {
    for case in original_beam()["cases"].as_array().unwrap() {
        let (mut owner, labels, mut refs, group) = prepare_beam(case);
        let seed = named(&owner, &labels, case["seed"].as_str().unwrap());
        let support = named(&owner, &labels, case["support_last"].as_str().unwrap());
        let target = named(&owner, &labels, case["target"].as_str().unwrap());
        let predecessor = named(
            &owner,
            &labels,
            case["states"][0]["state"]["predecessor"].as_str().unwrap(),
        );
        let mut original_rows = BTreeMap::new();
        let mut native_rows = BTreeMap::new();
        for step in case["states"].as_array().unwrap() {
            let phase = step["phase"].as_str().unwrap();
            match phase {
                "reused" => {
                    let i = construct_owned_voice(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &owner.measures,
                        &seed,
                    )
                    .unwrap();
                    set_owned_voice_id(&mut owner.owner.graph, &mut owner.voices[i], 7).unwrap();
                    refs.push(owner.voices[i].clone());
                }
                "restored_live" => {
                    let mut old = actual_voice(&owner, &refs[0]).clone();
                    old.chords_initialized = true;
                    old.chord_ids.push(predecessor.id);
                    owner.voices.push(old);
                }
                "emptied_live" => owner
                    .voices
                    .iter_mut()
                    .find(|v| Arc::ptr_eq(&v.identity, &refs[0].identity))
                    .unwrap()
                    .reset_chords(),
                "next" => owner
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
                    .unwrap(),
                "preferred" => set_next_preferred(&mut owner, &target, json!(1)),
                "null" => assign_cached_reference(&mut owner, &predecessor, None),
                "restored" => {
                    let old = actual_voice(&owner, &refs[0]).clone();
                    assign_cached_reference(&mut owner, &predecessor, Some(&old));
                }
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
                "refined" | "reuse_refined" | "live_refined" | "repurged" | "next" | "preferred"
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
            compare_beam(&owner, &labels, &refs, group, case, step, &relations);
            if matches!(phase, "null" | "measure_reset") {
                let graph = owner.owner.graph.snapshot();
                let voices = projection(&owner.voices, &owner.measures);
                let clocks = source_state(&owner, &labels, &refs);
                let pool = owner.owner.graph.retained_voice_observations();
                let address = owner.chords.as_ptr();
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
                assert_eq!(owner.owner.graph.snapshot(), graph);
                assert_eq!(projection(&owner.voices, &owner.measures), voices);
                assert_eq!(source_state(&owner, &labels, &refs), clocks);
                assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
                assert_eq!(owner.chords.as_ptr(), address);
                if phase == "measure_reset" {
                    assert_eq!(case["reset_refine_failure"], "NullPointerException");
                }
            }
        }
    }
}
#[test]
fn late_retired_beam_failure_preserves_actual_owner_clocks_and_retries() {
    let original = original_beam();
    let case = &original["cases"][1];
    let (mut owner, labels, refs, group) = prepare_beam(case);
    let target = named(&owner, &labels, case["target"].as_str().unwrap());
    set_next_preferred(&mut owner, &target, json!("invalid"));
    let graph = owner.owner.graph.snapshot();
    let voices = projection(&owner.voices, &owner.measures);
    let clocks = source_state(&owner, &labels, &refs);
    let pool = owner.owner.graph.retained_voice_observations();
    let layout = owner.layout();
    let address = owner.chords.as_ptr();
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
    assert_eq!(projection(&owner.voices, &owner.measures), voices);
    assert_eq!(source_state(&owner, &labels, &refs), clocks);
    assert_eq!(owner.owner.graph.retained_voice_observations(), pool);
    assert_eq!(owner.layout(), layout);
    assert_eq!(owner.chords.as_ptr(), address);
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
    set_next_preferred(&mut owner, &target, Value::Null);
    refine_ordinary_system(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        1,
    )
    .unwrap();
    compare_beam(
        &owner,
        &labels,
        &refs,
        group,
        case,
        &case["states"][1],
        &graph["relations"],
    );
}
