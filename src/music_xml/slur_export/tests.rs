// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-slur-export/observed.json"
    ))
    .unwrap()
}
fn attributes(mark: &Value) -> BTreeMap<String, String> {
    [
        "type",
        "number",
        "placement",
        "default-x",
        "default-y",
        "bezier-x",
        "bezier-y",
        "bezier-x2",
        "bezier-y2",
    ]
    .iter()
    .filter_map(|k| {
        let v = &mark[*k];
        if v.is_null() {
            None
        } else {
            Some((
                k.to_string(),
                v.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string()),
            ))
        }
    })
    .collect()
}
fn assert_serialized_slur_notes(xml: &str, observation: &Value, source: &Value) -> Value {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new("python3")
        .args(["-c", include_str!("read_serialized.py")])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python3 stdlib parser for actual serialized MusicXML");
    {
        let mut input = child.stdin.take().unwrap();
        input.write_all(xml.as_bytes()).unwrap();
    }
    let parsed = child.wait_with_output().unwrap();
    assert!(
        parsed.status.success(),
        "{}",
        String::from_utf8_lossy(&parsed.stderr)
    );
    let actual: Vec<Value> = serde_json::from_slice(&parsed.stdout).unwrap();
    let mut expected = Vec::new();
    let mut indexes = BTreeMap::<(u64, u64), usize>::new();
    for event in observation["events"].as_array().unwrap() {
        let part = event["part_id"].as_u64().unwrap();
        let measure = event["measure_number"].as_u64().unwrap();
        let index = indexes.entry((part, measure)).or_default();
        let pitch = if event["rest"] == true {
            Value::Null
        } else {
            let mut pitch = json!({"step":event["step"].as_str().unwrap(),
                "octave":event["octave"].as_i64().unwrap().to_string()});
            let alter = event["alter"].as_i64().unwrap();
            if alter != 0 {
                pitch["alter"] = json!(alter.to_string());
            }
            pitch
        };
        expected.push(json!({"part":part,"physical_measure":measure,
            "measure_score_id":event["measure_score_id"],"note_index":*index,
            "voice":event["voice"],"staff_number":event["staff_number"].as_u64().unwrap_or(1),
            "pitch":pitch,"slurs":event["slurs"].as_array().into_iter().flatten().map(attributes).collect::<Vec<_>>()}));
        *index += 1;
    }
    assert_eq!(
        actual, expected,
        "serialized note/Slur order, pitch and every emitted attribute"
    );
    assert_eq!(
        actual.len() as u64,
        source["counts"]["xml_notes"].as_u64().unwrap()
    );
    let mut endpoints = Vec::new();
    for note in &actual {
        for mark in note["slurs"].as_array().unwrap() {
            let mut row = note.clone();
            row.as_object_mut().unwrap().remove("slurs");
            row["attributes"] = mark.clone();
            endpoints.push(row);
        }
    }
    let original = source["endpoints_in_export_order"].as_array().unwrap();
    assert_eq!(endpoints.len(), 12);
    assert_eq!(endpoints.len(), original.len());
    for (actual, original) in endpoints.iter().zip(original) {
        for field in [
            "part",
            "physical_measure",
            "measure_score_id",
            "note_index",
            "voice",
            "staff_number",
            "pitch",
        ] {
            assert_eq!(
                actual[field], original[field],
                "serialized endpoint {field}"
            );
        }
        for field in ["type", "number", "placement"] {
            assert_eq!(
                actual["attributes"][field], original["attributes"][field],
                "serialized Slur {field}"
            );
        }
    }
    json!({"note_count":actual.len(),"endpoints":endpoints})
}

#[test]
fn original_saved_owned_endpoints_preserve_export_geometry() {
    let source = source();
    assert_eq!(source["rounded_Book_geometry_differences"], json!([]));
    for endpoint in source["endpoints_in_export_order"].as_array().unwrap() {
        let control = &endpoint["saved_geometry_control"];
        let c: Vec<f64> = serde_json::from_value(control["curve"].clone()).unwrap();
        let curve = Curve {
            bezier: [[c[0], c[1]], [c[2], c[3]], [c[4], c[5]], [c[6], c[7]]],
            above: control["above"].as_bool().unwrap(),
        };
        let side = if endpoint["side"] == "LEFT" {
            Side::Left
        } else {
            Side::Right
        };
        let value = mark(
            curve,
            side,
            false,
            Some(1),
            control["head_left"].as_i64().unwrap(),
            control["staff_top_at_end"].as_f64().unwrap(),
            control["interline"].as_u64().unwrap() as usize,
        )
        .unwrap();
        let expected: BTreeMap<String, String> =
            serde_json::from_value(endpoint["attributes"].clone()).unwrap();
        assert_eq!(attributes(&value), expected);
    }
}
fn key(local: usize) -> Key {
    Key {
        system: 1,
        phase: crate::shared_sig::Phase::Curve,
        local,
    }
}
#[test]
fn overlapping_number_lifetimes_reuse_the_lowest_free_value_and_omit_exhaustion() {
    // Declared source-code contracts; these overlap/exhaustion cases were not present in Der Winter.
    let mut numbers = Numbers::default();
    let visits = [1, 2, 1, 3, 2, 3, 1, 1].map(|id| numbers.visit(key(id)));
    assert_eq!(
        visits,
        [
            Some(1),
            Some(2),
            Some(1),
            Some(1),
            Some(2),
            Some(1),
            Some(1),
            Some(1)
        ]
    );
    let mut full = Numbers::default();
    for id in 1..=16 {
        assert_eq!(full.visit(key(id)), Some(id as u8));
    }
    assert_eq!(full.visit(key(17)), None);
    assert_eq!(full.visit(key(1)), Some(1));
    assert_eq!(full.visit(key(17)), Some(1));
    assert_eq!(Numbers::default().visit(key(17)), Some(1));
}
#[test]
fn continued_left_uses_second_bezier_and_tenths_use_even_rounding() {
    // Source addSlur scalar branch only; automatic orphan/extension admission stays guarded.
    let curve = Curve {
        bezier: [[10., 20.], [14., 25.], [26., 25.], [30., 20.]],
        above: true,
    };
    let left = mark(curve, Side::Left, true, Some(2), 8, 10., 10).unwrap();
    assert_eq!(
        left,
        json!({"type":"continue","number":2,"placement":"above",
        "default-x":2,"default-y":-10,"bezier-x2":4,"bezier-y2":-5})
    );
    let right = mark(curve, Side::Right, true, None, 8, 10., 10).unwrap();
    assert_eq!(
        right,
        json!({"type":"continue","number":null,"placement":null,
        "default-x":22,"default-y":-10,"bezier-x":-4,"bezier-y":-5})
    );
    assert_eq!(tenths(2.75, 11).unwrap(), 2);
    assert_eq!(tenths(8.25, 11).unwrap(), 8);
    assert_eq!(tenths(-2.75, 11).unwrap(), -2);
    assert_eq!(tenths(-8.25, 11).unwrap(), -8);
}
/// Invoke on the already recognized Der Winter owner after its existing MusicXML write.
/// This reads source expectations only after native recognition and emission have completed.
pub(crate) fn assert_same_real_page(m: &OwnedMusic, observation: &Value, xml: &str) {
    let source = source();
    let serialized = assert_serialized_slur_notes(xml, observation, &source);
    let baseline = m.owner.graph.snapshot();
    let state = m.slurs.as_ref().expect("actual native Slur state");
    let live = state
        .slurs
        .iter()
        .filter(|s| !m.owner.graph.retained_inter(s.key).unwrap().1)
        .collect::<Vec<_>>();
    assert_eq!(live.len(), 6);
    let mut matched = BTreeSet::new();
    for expected in source["slurs"].as_array().unwrap() {
        let matches = live
            .iter()
            .filter(|slur| {
                slur.system.id == expected["system"].as_u64().unwrap()
                    && slur.part.id == expected["part"].as_u64().unwrap()
                    && slur
                        .endpoints
                        .iter()
                        .zip(expected["endpoints"].as_array().unwrap())
                        .all(|(endpoint, want)| {
                            m.owner.graph.retained_inter(endpoint.head).unwrap().0["bounds"]
                                == want["head_bounds"]
                        })
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "actual full pair: {expected}");
        let slur = *matches[0];
        assert!(matched.insert(slur.key));
        validate(m, &m.owner.graph, slur).unwrap();
        let mut stale = m.owner.graph.clone();
        for (id, relation) in slur.relation_ids.iter().zip(&slur.relations) {
            assert!(Arc::ptr_eq(
                m.owner.graph.fresh_slur_relation_owner(*id).unwrap(),
                relation,
            ));
            assert!(Arc::ptr_eq(
                stale.fresh_slur_relation_owner(*id).unwrap(),
                relation,
            ));
        }
        stale.declare_stale_slur_export_relation(slur.relation_ids[0]);
        let declared = stale.snapshot();
        assert!(validate(m, &stale, slur).is_err());
        assert_eq!(stale.snapshot(), declared);
    }
    let mut actual = Vec::new();
    let mut note_indexes = BTreeMap::<(u64, u64), usize>::new();
    for event in observation["events"].as_array().unwrap() {
        let part = event["part_id"].as_u64().unwrap();
        let measure = event["measure_number"].as_u64().unwrap();
        let index = note_indexes.entry((part, measure)).or_default();
        for mark in event["slurs"].as_array().into_iter().flatten() {
            actual.push(
                json!({"part":part,"physical_measure":measure,"note_index":*index,
                "measure_score_id":event["measure_score_id"],"voice":event["voice"],
                "staff_number":event["staff_number"].as_u64().unwrap_or(1),
                "head_bounds":mark["head_bounds"],"attributes":attributes(mark),
                "native_slur_key":mark["slur_key"],"actual_relation_id":mark["relation_id"]}),
            );
        }
        *index += 1;
    }
    let expected = source["endpoints_in_export_order"].as_array().unwrap();
    assert_eq!(actual.len(), 12);
    let mut geometry_differences = Vec::new();
    for (row, expected) in actual.iter().zip(expected) {
        for field in [
            "part",
            "physical_measure",
            "note_index",
            "measure_score_id",
            "voice",
            "staff_number",
            "head_bounds",
        ] {
            assert_eq!(row[field], expected[field], "{field}");
        }
        for field in ["type", "number", "placement"] {
            assert_eq!(
                row["attributes"][field], expected["attributes"][field],
                "{field}"
            );
        }
        for field in ["default-x", "default-y", "bezier-x", "bezier-y"] {
            if row["attributes"][field] != expected["attributes"][field] {
                geometry_differences.push(json!({"part":row["part"],"measure":row["physical_measure"],
                    "note_index":row["note_index"],"attribute":field,"native":row["attributes"][field],
                    "original":expected["attributes"][field]}));
            }
        }
    }
    assert_eq!(m.owner.graph.snapshot(), baseline);
    std::fs::write("/tmp/real-rest-native-slur-export.json",serde_json::to_vec_pretty(&json!({
        "endpoint_relationships_numbering_and_placement_match":true,"six_actual_Slur_pairs":true,
        "serialized_MusicXML_notes_checked":serialized["note_count"],
        "serialized_MusicXML_Slur_endpoints":serialized["endpoints"],
        "serialized_MusicXML_matches_native_event_attributes":true,
        "serialized_MusicXML_endpoint_pitches_match_original":true,
        "twelve_actual_relationship_endpoints":actual,"actual_graph_manual_clocks_and_relations_unchanged":true,
        "stale_actual_relation_refused_without_mutation":true,"same_selected_relation_Arcs_across_Graph_clone":true,"geometry_differences":geometry_differences,
        "twelve_Slur_endpoint_geometry_equal":geometry_differences.is_empty(),"quality_accepted":false})).unwrap()).unwrap();
}
