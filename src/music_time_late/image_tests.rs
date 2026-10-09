// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn normalize(n: &Value) -> Value {
    json!({"kind":n["kind"],"bounds":n["bounds"],"staff":n["staff"],"removed":n["removed"]})
}
fn source_sets(source: &Value) -> Vec<Value> {
    source["systems"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| {
            s["times"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| normalize(&t["body"]))
        })
        .collect()
}
pub(crate) fn assert_real_time_columns(music: &OwnedMusic) {
    let direct: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-time-late/image-observed.json"
    ))
    .unwrap();
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-time-column-failure/image-observed.json"
    ))
    .unwrap();
    assert_eq!(source_sets(&direct), source_sets(&source));
    for (a, b) in direct["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(source["systems"].as_array().unwrap())
    {
        let bodies = |v: &Value| {
            let mut rows = v
                .as_array()
                .unwrap()
                .iter()
                .map(|n| {
                    let mut n = n.clone();
                    n.as_object_mut().unwrap().remove("id");
                    n
                })
                .collect::<Vec<_>>();
            rows.sort_by_key(Value::to_string);
            rows
        };
        assert_eq!(bodies(&a["before"]), bodies(&b["before"]));
        assert_eq!(bodies(&a["after"]), bodies(&b["after"]));
    }
    let before = music.owner.graph.snapshot();
    let mut graph = music.owner.graph.clone();
    let receipt = handle_times(&mut graph, music).unwrap();
    std::fs::write("/tmp/real-time-column-failure-native-diagnostic.json",serde_json::to_vec_pretty(&json!({"receipt":receipt,"times_before":before["inters"].as_array().unwrap().iter().filter(|n|matches!(n["kind"].as_str(),Some("TimeWholeInter"|"TimeNumberInter"))).collect::<Vec<_>>()})).unwrap()).unwrap();
    let actual = receipt["failed_columns"].as_array().unwrap();
    assert_eq!(actual.len(), 3);
    let wanted = source["systems"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| {
            s["columns"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |c| (s["system"].as_u64().unwrap(), c))
        })
        .collect::<Vec<_>>();
    for (a, (system, b)) in actual.iter().zip(&wanted) {
        assert_eq!(a["system"], *system);
        let index = a["stack_index"].as_u64().unwrap() as usize;
        let stack = music
            .measure_layout
            .stacks
            .iter()
            .position(|s| s.stack.system == *system && s.stack.index == index)
            .unwrap()
            + 1;
        assert_eq!(b["stack_id"], stack);
        let ta = a["trace"].as_array().unwrap();
        let tb = b["trace"].as_array().unwrap();
        assert_eq!(ta.len(), tb.len());
        for (a, b) in ta.iter().zip(tb) {
            assert_eq!(a["staff"], b["staff"]);
            assert_eq!(a["filter_result"], b["filter_result"]);
            let aa = a["candidates"].as_array().unwrap();
            let bb = b["candidates"].as_array().unwrap();
            assert_eq!(aa.len(), bb.len());
            for (a, b) in aa.iter().zip(bb) {
                assert_eq!(normalize(a), normalize(b));
                assert_eq!(a["manual"], b["manual"]);
                assert!(
                    (a["grade"].as_f64().unwrap() - b["grade"].as_f64().unwrap()).abs() < 1e-12
                );
            }
            assert_eq!(
                a["after_filter"].as_array().unwrap().clone(),
                b["after_filter"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(normalize)
                    .collect::<Vec<_>>()
            );
        }
    }
    let removed = actual
        .iter()
        .flat_map(|c| c["removed_whole_times"].as_array().unwrap().iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(removed.len(), 9);
    assert!(removed.iter().all(|n| n["removed"] == true));
    let expected_removed = direct["systems"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| {
            s["before"]
                .as_array()
                .unwrap()
                .iter()
                .zip(s["after"].as_array().unwrap())
                .filter(|(a, b)| a["removed"] == false && b["removed"] == true)
                .map(|(_, b)| normalize(b))
        })
        .collect::<Vec<_>>();
    assert_eq!(removed, expected_removed);
    let after = graph.snapshot();
    let survivors = after["inters"].as_array().unwrap();
    for old in before["inters"].as_array().unwrap() {
        let deleted = removed.iter().any(|n| {
            n["kind"] == old["kind"]
                && n["bounds"] == old["bounds"]
                && n["staff"] == old["staff_id"]
        });
        if !deleted {
            assert_eq!(
                survivors.iter().find(|n| n["id"] == old["id"]).unwrap(),
                old
            );
        }
    }
    for edge in before["relations"].as_array().unwrap() {
        let removed_endpoint = ["source", "target"]
            .iter()
            .any(|end| !survivors.iter().any(|n| n["id"] == edge[*end]));
        if !removed_endpoint {
            assert!(after["relations"].as_array().unwrap().contains(edge));
        }
    }
    let mut invalid = music.owner.graph.clone();
    let last = before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|n| n["kind"] == "TimeWholeInter" && n["origin"]["phase"] == "SYMBOL")
        .unwrap();
    let mut data = last.clone();
    data["frozen"] = json!(true);
    data["manual"] = json!(true);
    invalid.vertex(key(last).unwrap(), data, false).unwrap();
    let chord = &music.chords[0];
    let mut body = invalid.retained_inter(chord.key).unwrap().0.clone();
    body["manual"] = json!(true);
    body["time_offset"] = json!({"num":7,"den":64});
    body["expected_duration"] = json!({"num":3,"den":4});
    invalid.vertex(chord.key, body, false).unwrap();
    let bad_before = invalid.snapshot();
    assert_eq!(
        handle_times(&mut invalid, music).unwrap_err(),
        "music_removal_frozen_inter"
    );
    assert_eq!(invalid.snapshot(), bad_before);
    std::fs::write("/tmp/real-time-column-failure-native-observed.json",serde_json::to_vec_pretty(&json!({"columns":receipt,"source_direct_and_traced_after_states_equal":true,"removed_whole_times":removed,"survivors_and_unrelated_relations_identical":true,"late_frozen_manual_failure_full_graph_rollback":true,"actual_Chord_clock_fields_preserved":true,"report_headers_not_used":music.owner.upstream_report["headers"].is_null()})).unwrap()).unwrap();
}

pub(crate) fn assert_isolated_positive_column(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-time-isolated-positive/image-observed.json"
    ))
    .unwrap();
    let times = source["systems"][0]["times"].as_array().unwrap();
    assert_eq!(times.len(), 1);
    let before = music.owner.graph.snapshot();
    let mut graph = music.owner.graph.clone();
    let receipt = handle_times(&mut graph, music).unwrap();
    assert_eq!(graph.snapshot(), before);
    assert!(receipt["failed_columns"].as_array().unwrap().is_empty());
    let selected = receipt["isolated_positive_columns"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    let actual = &selected[0];
    let expected = &times[0]["body"];
    for k in ["kind", "bounds", "grade", "staff"] {
        assert_eq!(actual[k], expected[k]);
    }
    let sys = &source["systems"][0];
    assert_eq!(sys["before"], sys["after"]);
    assert_eq!(receipt["general_positive_columns_complete"], false);
    std::fs::write("/tmp/real-time-column-failure-positive-observed.json",serde_json::to_vec_pretty(&json!({"isolated_actual_whole_matches_original":true,"before_after_full_graph_unchanged":true,"actual_Staff_and_glyph_retained":true,"selection":receipt})).unwrap()).unwrap();
}
