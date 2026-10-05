// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::clef_competition::{ClefNode, ExclusionEdge, competing_clef_order};
use serde_json::Value;

#[test]
fn original_get_competing_clefs_controls_match_order_exactly() {
    let fixture: Value = serde_json::from_slice(include_bytes!(
        "../fixtures/clef-competition/official-controlled.json"
    ))
    .unwrap();
    assert_eq!(fixture["controlled_input"], true);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 76);
    for case in cases {
        let nodes = case["nodes"]
            .as_array()
            .unwrap()
            .iter()
            // SIG.inters(staff,ClefInter.class) excludes other inter types.
            .filter(|n| n["is_clef"].as_bool().unwrap())
            .map(|n| serde_json::from_value::<ClefNode>(n.clone()).unwrap())
            .collect::<Vec<_>>();
        // Only edges the original directed graph accepted enter its lookup.
        let edges: Vec<ExclusionEdge> =
            serde_json::from_value(case["effective_edges"].clone()).unwrap();
        let expected: Vec<u64> = serde_json::from_value(case["expected_ids"].clone()).unwrap();
        let actual = competing_clef_order(
            case["target_staff_id"].as_u64().unwrap(),
            case["x"].as_i64().unwrap(),
            &nodes,
            &edges,
        )
        .unwrap();
        assert_eq!(actual, expected, "{}", case["name"]);
    }
}
