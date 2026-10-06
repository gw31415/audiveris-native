// SPDX-License-Identifier: AGPL-3.0-or-later
// Compare independently observed original identities by actual chord bounds.
// No reconstruction of the implementation's relationship or partition rules.
use serde_json::{Value, json};
pub fn assert_original(report: &Value, page_index: usize) {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-measure-slot-retrieval/observed.json"
    ))
    .unwrap();
    let expected = oracle["pages"][page_index]["measures"].as_array().unwrap();
    let actual = report["preliminary_measure_slots"].as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual["measure_number"], expected["number"]);
        let ordinal = |id: &Value| {
            let live = report["shared_sig"]["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == *id)
                .unwrap();
            let matches = expected["chords"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|f| f["bounds"] == live["bounds"])
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            matches[0]["id"].as_u64().unwrap()
        };
        for mode in ["narrow", "wide"] {
            let slots = actual[mode]["slots"].as_array().unwrap().iter().map(|s|json!({"id":s["id"],"chord_ids":s["chord_ids"].as_array().unwrap().iter().map(&ordinal).collect::<Vec<_>>()})).collect::<Vec<_>>();
            assert_eq!(json!(slots), expected[mode]["slots"]);
            let mut relations = actual[mode]["relations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    (
                        ordinal(&e["a"]),
                        ordinal(&e["b"]),
                        e["rel"].as_str().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            let mut original = expected[mode]["relations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    (
                        e["a"].as_u64().unwrap(),
                        e["b"].as_u64().unwrap(),
                        e["rel"].as_str().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            relations.sort();
            original.sort();
            assert_eq!(relations, original);
        }
    }
}
