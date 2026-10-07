// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::collections::BTreeMap;

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
        let node = nodes.iter().find(|n| n["id"] == id).unwrap();
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
}
fn graph_state(
    graph: &mut Graph,
    labels: &BTreeMap<usize, String>,
) -> (Value, BTreeMap<usize, Value>) {
    let mut state = graph.snapshot();
    state["inters"] = json!(
        graph
            .vertices
            .iter()
            .map(|v| {
                let mut node = v.data.clone();
                node["registered"] = json!(!v.removed);
                node["lifecycle_removed"] = json!(v.removed || v.removing);
                node
            })
            .collect::<Vec<_>>()
    );
    let bounds = labels
        .iter()
        .filter(|(_, name)| name.starts_with("chord"))
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
    let original: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-headstem-prelink/observed.json"
    ))
    .unwrap();
    let case = &original["cases"][0];
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
    ))
    .unwrap();
    let mut export = crate::music_xml::recognize(&image).unwrap();
    let labels = labels(&export.owner);
    saved_numeric_control(&mut export.owner, &labels, case);
    let mut sequence = prepare(&export.owner, &labels);
    let mut identities = BTreeMap::new();
    let graph = &mut export.owner.owner.graph;
    graph.observed_chords = labels
        .iter()
        .filter(|(_, name)| name.starts_with("chord"))
        .map(|(id, _)| graph.vertices[*id - 1].key)
        .collect();
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
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
    ))
    .unwrap();
    let mut export = crate::music_xml::recognize(&image).unwrap();
    let labels = labels(&export.owner);
    let mut sequence = prepare(&export.owner, &labels);
    let EditingTask::Relation(last) = sequence.tasks.last_mut().unwrap() else {
        unreachable!()
    };
    last.link.relation["head_side"] = json!("invalid");
    let graph = export.owner.owner.graph.snapshot();
    let layout = export.owner.layout();
    let chords_address = export.owner.chords.as_ptr();
    let voices=export.owner.voices.iter().map(|v|json!({"id":v.id,"members":v.chord_ids,"slots":v.slots.iter().map(|(id,s)|json!([id,s.chord_id,s.continuing])).collect::<Vec<_>>(),"termination":v.termination.map(|d|d.to_string())})).collect::<Vec<_>>();
    let expected = export.owner.measures[0].timing.expected;
    assert_eq!(
        sequence
            .perform_owned(&mut export.owner, EditingOperation::Do, "RHYTHMS")
            .unwrap_err(),
        "manual_head_stem_declared_relation_properties_required"
    );
    assert_eq!(export.owner.owner.graph.snapshot(), graph);
    assert_eq!(export.owner.layout(), layout);
    assert_eq!(export.owner.chords.as_ptr(), chords_address);
    assert_eq!(export.owner.measures[0].timing.expected, expected);
    assert_eq!(export.owner.voices.iter().map(|v|json!({"id":v.id,"members":v.chord_ids,"slots":v.slots.iter().map(|(id,s)|json!([id,s.chord_id,s.continuing])).collect::<Vec<_>>(),"termination":v.termination.map(|d|d.to_string())})).collect::<Vec<_>>(),voices);
    let EditingTask::Relation(last) = sequence.tasks.last_mut().unwrap() else {
        unreachable!()
    };
    last.link.relation["head_side"] = json!("RIGHT");
    sequence
        .perform_owned(&mut export.owner, EditingOperation::Do, "RHYTHMS")
        .unwrap();
}
