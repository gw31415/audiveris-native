// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::collections::BTreeMap;

fn actual_voice_clocks(owner: &crate::music_measures::OwnedMusic) -> Value {
    json!(owner.voices.iter().map(|v| json!({"id":v.id,"measure":v.measure,
        "starting_staff":v.starting_staff,"members":v.chord_ids,
        "slots":v.slots.iter().map(|(id,s)|json!([id,s.chord_id,s.continuing])).collect::<Vec<_>>(),
        "termination":v.termination.map(crate::music_time::format_delta),
        "excess":v.excess.map(crate::music_time::format_fraction),"measure_rest":v.measure_rest_chord
    })).collect::<Vec<_>>())
}

fn labels(owner: &crate::music_measures::OwnedMusic) -> BTreeMap<usize, String> {
    let mut chords = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
        .collect::<Vec<_>>();
    chords.sort_by_key(|c| c.bounds[0]);
    let mut labels = BTreeMap::new();
    for (i, chord) in chords.into_iter().enumerate() {
        labels.insert(chord.id, format!("chord{i}"));
        labels.insert(
            owner.owner.graph.id(chord.stem.unwrap()).unwrap(),
            format!("stem{i}"),
        );
        for (j, head) in chord.members.iter().enumerate() {
            labels.insert(
                head["id"].as_u64().unwrap() as usize,
                format!("head{i}_{j}"),
            );
        }
    }
    labels
}
fn compare_graph(
    graph: &Graph,
    snapshot: &Value,
    bounds: &BTreeMap<usize, Value>,
    labels: &BTreeMap<usize, String>,
    identities: &mut BTreeMap<String, usize>,
    facts: &Value,
    expected: &Value,
) {
    let nodes = snapshot["inters"].as_array().unwrap();
    let edges = snapshot["relations"].as_array().unwrap();
    let named = |id: &Value| {
        labels
            .get(&(id.as_u64().unwrap() as usize))
            .map(String::as_str)
    };
    let label_id = |label: &Value| {
        labels
            .iter()
            .find(|(_, name)| Some(name.as_str()) == label.as_str())
            .map(|(id, _)| *id)
    };
    for row in expected["chords"].as_array().unwrap() {
        let id = label_id(&row["label"]).unwrap();
        let Some(node) = nodes.iter().find(|n| n["id"] == id) else {
            assert_eq!(row["registered"], false);
            assert_eq!(row["removed"], false);
            assert!(row["bounds"].is_null());
            continue;
        };
        assert_eq!(
            node["registered"], row["registered"],
            "registered {}",
            row["label"]
        );
        assert_eq!(
            node["lifecycle_removed"], row["removed"],
            "removed {}",
            row["label"]
        );
        let mut member_nodes = edges
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == id)
            .map(|e| nodes.iter().find(|n| n["id"] == e["target"]).unwrap())
            .collect::<Vec<_>>();
        member_nodes.sort_by_key(|n| {
            std::cmp::Reverse(
                n["bounds"][1].as_i64().unwrap() + n["bounds"][3].as_i64().unwrap() / 2,
            )
        });
        assert_eq!(
            json!(
                member_nodes
                    .iter()
                    .map(|n| named(&n["id"]))
                    .collect::<Vec<_>>()
            ),
            row["members"]
        );
        let stem = edges
            .iter()
            .find(|e| e["kind"] == "ChordStemRelation" && e["source"] == id)
            .and_then(|e| named(&e["target"]));
        assert_eq!(json!(stem), row["stem"]);
        assert_eq!(bounds[&id], row["bounds"], "bounds {}", row["label"]);
        if let Some(grade) = row["grade"].as_f64() {
            assert!(
                (node["grade"].as_f64().unwrap() - grade).abs() < 1e-12,
                "grade {}: native {}, original {}",
                row["label"],
                node["grade"],
                grade
            );
        } else {
            assert!(node["grade"].is_null());
        }
        assert_eq!(json!(node["manual"] == true), row["manual"]);
        assert_eq!(node["part_id"], row["part_id"]);
        assert_eq!(node["staff_id"], row["staff_id"]);
        assert_eq!(node["voice"], row["voice_id"]);
        assert_eq!(node["slot_id"], row["slot_id"]);
        assert_eq!(node["time_offset"], row["time_offset"]);
        if row["label"] == "new_chord" {
            let owner = snapshot["editing_chord_owners"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == id)
                .unwrap();
            assert_eq!(
                json!(
                    owner["member_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(named)
                        .collect::<Vec<_>>()
                ),
                row["members"]
            );
            assert_eq!(
                json!(
                    owner["stem_id"]
                        .as_u64()
                        .and_then(|id| labels.get(&(id as usize)))
                ),
                row["stem"]
            );
            assert_eq!(owner["bounds"], row["bounds"]);
            assert_eq!(json!(owner["measure"] == 0), row["same_measure"]);
            assert_eq!(
                json!(owner["in_measure"] == true),
                json!(
                    expected["measure_chords"]
                        .as_array()
                        .unwrap()
                        .contains(&row["label"])
                )
            );
        }
    }
    for row in expected["relations"].as_array().unwrap() {
        let fact = facts
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["label"] == row["label"])
            .unwrap();
        let (Some(source), Some(target)) = (label_id(&fact["source"]), label_id(&fact["target"]))
        else {
            continue;
        };
        let name = row["label"].as_str().unwrap();
        let id = *identities.entry(name.into()).or_insert_with(|| {
            graph
                .edges
                .iter()
                .position(|e| {
                    e.key.source == source
                        && e.key.target == target
                        && json!(e.key.kind) == row["kind"]
                })
                .map_or(0, |i| i + 1)
        });
        if id == 0 {
            assert_eq!(row["registered"], false);
            identities.remove(name);
            continue;
        }
        assert_eq!(
            edges.iter().any(|e| e["id"] == id),
            row["registered"] == true,
            "relation {name}"
        );
        assert_eq!(
            json!(graph.edges[id - 1].data["manual"] == true),
            row["manual"],
            "manual {name}"
        );
    }
    if expected["cached_solo_voice"]["measure_registered"] == false {
        let id = label_id(&json!("chord0")).unwrap();
        let node = nodes.iter().find(|n| n["id"] == id).unwrap();
        let expected_chord = expected["chords"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["label"] == "chord0")
            .unwrap();
        let key = if expected_chord["same_voice"] == false {
            assert!(
                node["retained_voice_key"].is_null(),
                "setVoice must replace the detached owner even when its number is reused"
            );
            identities["$retained_solo_voice"]
        } else {
            node["retained_voice_key"].as_u64().unwrap() as usize
        };
        assert_eq!(
            *identities
                .entry("$retained_solo_voice".into())
                .or_insert(key),
            key
        );
        let actual = snapshot["retained_voice_owners"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["key"] == key)
            .unwrap();
        compare_solo_voice(actual, &expected["cached_solo_voice"], labels);
    }
}
fn compare_solo_voice(actual: &Value, expected: &Value, labels: &BTreeMap<usize, String>) {
    assert_eq!(actual["id"], expected["id"]);
    assert_eq!(actual["termination"], expected["termination"]);
    assert_eq!(
        json!(
            actual["chord_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| labels[&(id.as_u64().unwrap() as usize)].clone())
                .collect::<Vec<_>>()
        ),
        expected["chords"]
    );
    assert_eq!(json!(actual["slots"].as_array().unwrap().iter().map(|slot|json!({"id":slot["id"],"chord":labels[&(slot["chord_id"].as_u64().unwrap() as usize)],"status":slot["status"]})).collect::<Vec<_>>()),expected["slots"]);
}
fn graph_state(
    graph: &mut Graph,
    labels: &BTreeMap<usize, String>,
) -> (Value, BTreeMap<usize, Value>) {
    let mut state = graph.snapshot();
    state["editing_chord_owners"] = graph.editing_chord_observations();
    state["retained_voice_owners"] = graph.retained_voice_observations();
    state["inters"] = json!(
        graph
            .vertices
            .iter()
            .map(|v| {
                let mut node = v.data.clone();
                node["registered"] = json!(!v.removed);
                node["lifecycle_removed"] = json!(v.removed || v.removing);
                node["retained_voice_key"] = json!(v.retained_voice);
                node
            })
            .collect::<Vec<_>>()
    );
    let count = graph.vertices.len();
    let bounds = labels
        .iter()
        .filter(|(id, name)| {
            (name.starts_with("chord") || name.as_str() == "new_chord") && **id <= count
        })
        .map(|(id, _)| {
            (
                *id,
                json!(
                    graph
                        .lifecycle_chord_bounds(graph.vertices[*id - 1].key)
                        .unwrap()
                ),
            )
        })
        .collect();
    (state, bounds)
}
fn compare_owner(
    owner: &crate::music_measures::OwnedMusic,
    labels: &BTreeMap<usize, String>,
    expected: &Value,
) {
    let name = |id: &usize| labels.get(id).unwrap();
    assert_eq!(
        json!(
            owner
                .chords
                .iter()
                .filter(|c| c.measure == 0)
                .map(|c| name(&c.id))
                .chain(
                    owner
                        .owner
                        .graph
                        .editing_chords
                        .iter()
                        .filter(|c| c.in_measure && c.measure == Some(0))
                        .map(|c| name(&c.id))
                )
                .collect::<Vec<_>>()
        ),
        expected["measure_chords"]
    );
    assert_eq!(
        json!(
            owner
                .voices
                .iter()
                .filter(|v| v.measure == 0)
                .map(
                    |v| json!({"id":v.id,"chords":v.chord_ids.iter().map(name).collect::<Vec<_>>()})
                )
                .collect::<Vec<_>>()
        ),
        expected["voices"]
    );
    let timing = &owner.measures[0].timing;
    assert_eq!(
        json!(timing.expected.map(crate::music_time::format_fraction)),
        expected["expected"]
    );
    assert_eq!(
        json!(timing.actual.map(crate::music_time::format_fraction)),
        expected["actual"]
    );
    for chord in owner.chords.iter().filter(|c| c.measure == 0) {
        let row = expected["chords"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["label"] == *name(&chord.id))
            .unwrap();
        assert_eq!(
            json!(
                chord
                    .members
                    .iter()
                    .map(|h| name(&(h["id"].as_u64().unwrap() as usize)))
                    .collect::<Vec<_>>()
            ),
            row["members"]
        );
        assert_eq!(
            json!(chord.stem.map(|k| name(&owner.owner.graph.id(k).unwrap()))),
            row["stem"]
        );
        assert_eq!(json!(chord.bounds), row["bounds"]);
    }
    for chord in &owner.owner.graph.editing_chords {
        let row = expected["chords"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["label"] == *name(&chord.id))
            .unwrap();
        assert_eq!(
            json!(
                chord
                    .members
                    .iter()
                    .map(|h| name(&(h["id"].as_u64().unwrap() as usize)))
                    .collect::<Vec<_>>()
            ),
            row["members"]
        );
        assert_eq!(
            json!(chord.stem.map(|k| name(&owner.owner.graph.id(k).unwrap()))),
            row["stem"]
        );
        assert_eq!(json!(chord.bounds), row["bounds"]);
        assert_eq!(json!(chord.measure == Some(0)), row["same_measure"]);
        assert_eq!(
            chord.in_measure,
            expected["measure_chords"]
                .as_array()
                .unwrap()
                .contains(&json!(name(&chord.id)))
        );
    }
    if expected["cached_solo_voice"]["measure_registered"] == true {
        let voice = owner
            .voices
            .iter()
            .find(|v| json!(v.id) == expected["cached_solo_voice"]["id"] && v.measure == 0)
            .unwrap();
        compare_solo_voice(
            &json!({"id":voice.id,"chord_ids":voice.chord_ids,"termination":voice.termination.map(crate::music_time::format_delta),"slots":voice.slots.iter().map(|(id,s)|json!({"id":id,"chord_id":s.chord_id,"status":if s.continuing{"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>()}),
            &expected["cached_solo_voice"],
            labels,
        );
    }
    if expected["cached_reused_voice"]["measure_registered"] == true {
        let voice = owner
            .voices
            .iter()
            .find(|v| v.measure == 0 && json!(v.id) == expected["cached_reused_voice"]["id"])
            .unwrap();
        compare_solo_voice(
            &json!({"id":voice.id,"chord_ids":voice.chord_ids,"termination":voice.termination.map(crate::music_time::format_delta),"slots":voice.slots.iter().map(|(id,s)|json!({"id":id,"chord_id":s.chord_id,"status":if s.continuing{"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>()}),
            &expected["cached_reused_voice"],
            labels,
        );
    }
}
fn declare_solo_voice(
    owner: &mut crate::music_measures::OwnedMusic,
    labels: &BTreeMap<usize, String>,
) {
    declare_voice_for(owner, labels, "chord0");
}
fn declare_voice_for(
    owner: &mut crate::music_measures::OwnedMusic,
    labels: &BTreeMap<usize, String>,
    name: &str,
) -> usize {
    let id = *labels
        .iter()
        .find(|(_, label)| label.as_str() == name)
        .unwrap()
        .0;
    let chord = owner.chords.iter().find(|c| c.id == id).unwrap().clone();
    let slot_id = owner.owner.graph.vertices[id - 1].data["slot_id"]
        .as_u64()
        .unwrap() as usize;
    let old = owner
        .voices
        .iter_mut()
        .find(|v| v.measure == 0 && v.chord_ids.contains(&id))
        .unwrap();
    let slot_info = old.slots[&slot_id];
    old.chord_ids.retain(|c| *c != id);
    // This declared setup mirrors justAssignVoice(null), not a callback-time
    // rhythm reset. Existing Slot/time/expected clocks remain untouched.
    owner.owner.graph.vertices[id - 1].data["voice"] = Value::Null;
    let voice = crate::music_voices::create_voice(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        &chord,
    )
    .unwrap();
    owner.voices[voice].slots.insert(slot_id, slot_info);
    voice
}
fn declare_reused_voice(
    owner: &mut crate::music_measures::OwnedMusic,
    labels: &BTreeMap<usize, String>,
) {
    let voice = declare_voice_for(owner, labels, "chord1");
    let chord = *labels
        .iter()
        .find(|(_, label)| label.as_str() == "chord0")
        .unwrap()
        .0;
    crate::music_voices::assign_chord_voice(
        &mut owner.owner.graph,
        &mut owner.voices,
        &owner.measures,
        chord,
        voice,
    )
    .unwrap();
}
fn prepare(
    owner: &crate::music_measures::OwnedMusic,
    labels: &BTreeMap<usize, String>,
) -> EditingSequence {
    let key = |name: &str| {
        owner.owner.graph.vertices[*labels
            .iter()
            .find(|(_, label)| label.as_str() == name)
            .unwrap()
            .0
            - 1]
        .key
    };
    let head = key("head0_0");
    let stem = key("stem1");
    let old = owner
        .owner
        .graph
        .incident_edges(owner.owner.graph.id(head).unwrap())
        .into_iter()
        .map(|id| &owner.owner.graph.edges[id - 1])
        .find(|e| e.key.kind == "HeadStemRelation")
        .unwrap();
    EditingSequence::head_stem_pre_link(owner,head,stem,json!({"kind":"HeadStemRelation","manual":true,"grade":0.7,"head_side":old.data["head_side"],"extension_point":old.data["extension_point"]})).unwrap()
}
fn saved_numeric_control(
    owner: &mut crate::music_measures::OwnedMusic,
    labels: &BTreeMap<usize, String>,
    case: &Value,
) {
    // The original saved .omr reloads grades at three decimal places. This
    // declared numeric control leaves real image-owned membership intact.
    for (id, label) in labels {
        let data = &mut owner.owner.graph.vertices[*id - 1].data;
        let property = if label.starts_with("head") {
            "contextual_grade"
        } else if label.starts_with("chord") {
            "grade"
        } else {
            continue;
        };
        data[property] =
            json!((data[property].as_f64().unwrap() * 1000.0).round_ties_even() / 1000.0);
    }
    // This is a saved-owner lifecycle control, not new full-image stem
    // parity: the native image stem extends into the head by five pixels.
    // Declare original setBounds on the same canonical Arc, keeping its
    // glyph, median, SIG identity and all real Chord/Voice owners intact.
    for fact in case["stem_facts"].as_array().unwrap() {
        let id = *labels
            .iter()
            .find(|(_, label)| json!(label) == fact["label"])
            .unwrap()
            .0;
        let key = owner.owner.graph.vertices[id - 1].key;
        let canonical = owner.owner.caches[&key.system]
            .registered_stem(&owner.owner.graph, key.local)
            .unwrap();
        canonical.removal_guard().unwrap()["bounds"] = fact["bounds"].clone();
        owner.owner.graph.vertices[id - 1].data["bounds"] = fact["bounds"].clone();
    }
}
#[test]
fn existing_target_prelink_matches_original_empty_chord_tasks_and_callbacks() {
    compare_prelink_case(0);
}
#[test]
fn new_target_prelink_matches_original_unplaced_chord_and_retained_redo() {
    compare_prelink_case(1);
}
#[test]
fn last_chord_removal_retains_original_voice_and_slots_without_resurrecting_it() {
    compare_prelink_case(2);
}
#[test]
fn same_number_reassigned_voice_changes_actual_owner_without_old_slot_transfer() {
    compare_prelink_case(3);
}
fn compare_prelink_case(case_index: usize) {
    let original: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-headstem-prelink/observed.json"
    ))
    .unwrap();
    let case = &original["cases"][case_index];
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
    ))
    .unwrap();
    let mut export = crate::music_xml::recognize(&image).unwrap();
    let mut labels = labels(&export.owner);
    saved_numeric_control(&mut export.owner, &labels, case);
    if case_index >= 2 {
        declare_solo_voice(&mut export.owner, &labels);
    }
    if case_index == 1 {
        let id = *labels
            .iter()
            .find(|(_, label)| label.as_str() == "chord1")
            .unwrap()
            .0;
        let key = export.owner.owner.graph.vertices[id - 1].key;
        let mut removal = super::super::ChordRemovalTask::new(&export.owner, key).unwrap();
        removal
            .perform_owned(&mut export.owner, EditingOperation::Do)
            .unwrap();
    }
    let mut sequence = prepare(&export.owner, &labels);
    if case_index == 1 {
        labels.insert(
            export.owner.owner.graph.vertices.len() + 1,
            "new_chord".into(),
        );
    }
    let mut identities = BTreeMap::new();
    let graph = &mut export.owner.owner.graph;
    graph.observed_chords = labels
        .iter()
        .filter(|(_, name)| name.starts_with("chord"))
        .map(|(id, _)| graph.vertices[*id - 1].key)
        .collect();
    if let Some(EditingTask::ChordAddition(addition)) = sequence
        .tasks
        .iter()
        .find(|t| matches!(t, EditingTask::ChordAddition(_)))
    {
        graph.observed_chords.push(addition.key());
    }
    let (state, bounds) = graph_state(graph, &labels);
    compare_graph(
        graph,
        &state,
        &bounds,
        &labels,
        &mut identities,
        &case["relation_facts"],
        &case["before"],
    );
    compare_owner(&export.owner, &labels, &case["before"]);
    for (op_index, operation) in [
        EditingOperation::Do,
        EditingOperation::Undo,
        EditingOperation::Redo,
    ]
    .into_iter()
    .enumerate()
    {
        if case_index == 3 && op_index == 2 {
            let (state, bounds) = graph_state(&mut export.owner.owner.graph, &labels);
            compare_graph(
                &export.owner.owner.graph,
                &state,
                &bounds,
                &labels,
                &mut identities,
                &case["relation_facts"],
                &case["voice_rebind"]["before"],
            );
            compare_owner(&export.owner, &labels, &case["voice_rebind"]["before"]);
            declare_reused_voice(&mut export.owner, &labels);
            let (state, bounds) = graph_state(&mut export.owner.owner.graph, &labels);
            compare_graph(
                &export.owner.owner.graph,
                &state,
                &bounds,
                &labels,
                &mut identities,
                &case["relation_facts"],
                &case["voice_rebind"]["after"],
            );
            compare_owner(&export.owner, &labels, &case["voice_rebind"]["after"]);
        }
        let order = if matches!(operation, EditingOperation::Undo) {
            (0..sequence.tasks.len()).rev().collect::<Vec<_>>()
        } else {
            (0..sequence.tasks.len()).collect()
        };
        for (index, task_index) in order.into_iter().enumerate() {
            let expected = &case["steps"][op_index * sequence.tasks.len() + index];
            export.owner.owner.graph.enable_dot_callback_trace();
            EditingSequence::perform_task_owned(
                &mut sequence.tasks[task_index],
                &mut export.owner,
                operation,
            )
            .unwrap();
            let graph = &mut export.owner.owner.graph;
            let trace = graph
                .dot_callback_trace()
                .iter()
                .filter(|e| e["event"] != "publish")
                .collect::<Vec<_>>();
            assert_eq!(
                trace.len(),
                expected["callbacks"].as_array().unwrap().len(),
                "callbacks {op_index}/{index}"
            );
            for (actual, want) in trace.iter().zip(expected["callbacks"].as_array().unwrap()) {
                assert_eq!(actual["event"], want["event"]);
                assert_eq!(actual["stage"], want["stage"]);
                let bounds = actual["chord_observations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| (r["id"].as_u64().unwrap() as usize, r["bounds"].clone()))
                    .collect();
                compare_graph(
                    graph,
                    &actual["state"],
                    &bounds,
                    &labels,
                    &mut identities,
                    &case["relation_facts"],
                    &want["state"],
                );
            }
            let (state, bounds) = graph_state(graph, &labels);
            compare_graph(
                graph,
                &state,
                &bounds,
                &labels,
                &mut identities,
                &case["relation_facts"],
                &expected["state"],
            );
            compare_owner(&export.owner, &labels, &expected["state"]);
        }
        sequence
            .notify_impact(&mut export.owner.owner.graph, operation, "RHYTHMS", 18)
            .unwrap();
    }
    assert_eq!(
        sequence
            .require_ordinary_reprocess(&export.owner.owner.graph)
            .unwrap_err(),
        "music_reprocess_chord_lifecycle_pending"
    );
}
#[test]
fn failed_prelink_sequence_restores_actual_chords_voice_and_expected_clocks() {
    compare_failed_prelink(false, false);
}
#[test]
fn failed_new_target_prelink_restores_actual_owner_and_reuses_same_sequence() {
    compare_failed_prelink(true, false);
}
#[test]
fn failed_last_voice_removal_restores_actual_voice_owner_and_clocks() {
    compare_failed_prelink(false, true);
}
#[test]
fn failed_reused_number_voice_edit_restores_current_and_retired_owners() {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
    ))
    .unwrap();
    let mut export = crate::music_xml::recognize(&image).unwrap();
    let labels = labels(&export.owner);
    declare_solo_voice(&mut export.owner, &labels);
    let mut sequence = prepare(&export.owner, &labels);
    sequence
        .perform_owned(&mut export.owner, EditingOperation::Do, "RHYTHMS")
        .unwrap();
    sequence
        .perform_owned(&mut export.owner, EditingOperation::Undo, "RHYTHMS")
        .unwrap();
    declare_reused_voice(&mut export.owner, &labels);
    let snapshot = export.owner.owner.graph.snapshot();
    let layout = export.owner.layout();
    let retired = export.owner.owner.graph.retained_voice_observations();
    let voices = actual_voice_clocks(&export.owner);
    let expected = export.owner.measures[0].timing.expected;
    let wrong = labels
        .iter()
        .find(|(_, label)| label.as_str() == "stem0")
        .unwrap()
        .0;
    let key = export.owner.owner.graph.vertices[*wrong - 1].key;
    let canonical = export.owner.owner.caches[&key.system]
        .registered_stem(&export.owner.owner.graph, key.local)
        .unwrap();
    let EditingTask::Relation(last) = sequence.tasks.last_mut().unwrap() else {
        unreachable!()
    };
    let stem = last.owned_chord.as_ref().unwrap().stem.clone();
    last.owned_chord.as_mut().unwrap().stem = canonical;
    assert_eq!(
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Redo, "RHYTHMS")
            .unwrap_err(),
        "manual_head_stem_retained_owner_changed"
    );
    assert_eq!(export.owner.owner.graph.snapshot(), snapshot);
    assert_eq!(export.owner.layout(), layout);
    assert_eq!(
        export.owner.owner.graph.retained_voice_observations(),
        retired
    );
    assert_eq!(actual_voice_clocks(&export.owner), voices);
    assert_eq!(export.owner.measures[0].timing.expected, expected);
    let EditingTask::Relation(last) = sequence.tasks.last_mut().unwrap() else {
        unreachable!()
    };
    last.owned_chord.as_mut().unwrap().stem = stem;
    sequence
        .perform_owned(&mut export.owner, EditingOperation::Redo, "RHYTHMS")
        .unwrap();
    assert_eq!(export.owner.owner.graph.retained_voices.len(), 1);
    assert!(
        export
            .owner
            .voices
            .iter()
            .any(|v| v.measure == 0 && v.id == 2)
    );
}
fn compare_failed_prelink(new_target: bool, solo_voice: bool) {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
    ))
    .unwrap();
    let mut export = crate::music_xml::recognize(&image).unwrap();
    let labels = labels(&export.owner);
    if solo_voice {
        declare_solo_voice(&mut export.owner, &labels);
    }
    if new_target {
        let id = *labels
            .iter()
            .find(|(_, label)| label.as_str() == "chord1")
            .unwrap()
            .0;
        let key = export.owner.owner.graph.vertices[id - 1].key;
        let mut removal = super::super::ChordRemovalTask::new(&export.owner, key).unwrap();
        removal
            .perform_owned(&mut export.owner, EditingOperation::Do)
            .unwrap();
    }
    let mut sequence = prepare(&export.owner, &labels);
    let EditingTask::Relation(last) = sequence.tasks.last_mut().unwrap() else {
        unreachable!()
    };
    last.link.relation["head_side"] = json!("invalid");
    let graph = export.owner.owner.graph.snapshot();
    let layout = export.owner.layout();
    let chords_address = export.owner.chords.as_ptr();
    let voices = actual_voice_clocks(&export.owner);
    let expected = export.owner.measures[0].timing.expected;
    assert_eq!(
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Do, "RHYTHMS")
            .unwrap_err(),
        "manual_head_stem_declared_relation_properties_required"
    );
    assert_eq!(export.owner.owner.graph.snapshot(), graph);
    assert_eq!(export.owner.layout(), layout);
    assert!(export.owner.owner.graph.editing_chords.is_empty());
    assert!(export.owner.owner.graph.retained_voices.is_empty());
    assert_eq!(export.owner.chords.as_ptr(), chords_address);
    assert_eq!(export.owner.measures[0].timing.expected, expected);
    assert_eq!(actual_voice_clocks(&export.owner), voices);
    let EditingTask::Relation(last) = sequence.tasks.last_mut().unwrap() else {
        unreachable!()
    };
    last.link.relation["head_side"] = json!("RIGHT");
    sequence
        .perform_owned(&mut export.owner, EditingOperation::Do, "RHYTHMS")
        .unwrap();
    if new_target {
        assert_eq!(export.owner.owner.graph.editing_chords.len(), 1);
        let key = export.owner.owner.graph.editing_chords[0].key;
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Undo, "RHYTHMS")
            .unwrap();
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Redo, "RHYTHMS")
            .unwrap();
        assert_eq!(export.owner.owner.graph.editing_chords.len(), 1);
        assert_eq!(export.owner.owner.graph.editing_chords[0].key, key);
        assert_eq!(export.owner.measures[0].timing.expected, expected);
    }
    if solo_voice {
        assert_eq!(export.owner.owner.graph.retained_voices.len(), 1);
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Undo, "RHYTHMS")
            .unwrap();
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Redo, "RHYTHMS")
            .unwrap();
        assert_eq!(export.owner.owner.graph.retained_voices.len(), 1);
        assert!(
            !export
                .owner
                .voices
                .iter()
                .any(|v| v.measure == 0 && v.id == 2)
        );
    }
}
