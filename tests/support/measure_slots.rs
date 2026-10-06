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
        let compound_oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-compound-slots/observed.json"
        ))
        .unwrap();
        let expected_compounds = &compound_oracle["pages"][page_index]["measures"]
            [actual["measure_number"].as_u64().unwrap() as usize - 1]["cases"][0]["compounds"];
        let compounds=actual["compounds"].as_array().unwrap().iter().map(|c|json!({"id":c["id"],"narrow_slot_ids":c["narrow_slot_ids"],"chord_ids":c["chord_ids"].as_array().unwrap().iter().map(&ordinal).collect::<Vec<_>>(),"lookup":c["lookup"].as_array().unwrap().iter().map(|l|json!({"chord_id":ordinal(&l["chord_id"]),"narrow_slot_id":l["narrow_slot_id"]})).collect::<Vec<_>>()})).collect::<Vec<_>>();
        assert_eq!(json!(compounds), *expected_compounds);
        let starting: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-starting-voices/observed.json"
        ))
        .unwrap();
        let expected_start = &starting["pages"][page_index]["measures"]
            [actual["measure_number"].as_u64().unwrap() as usize - 1];
        let start_ordinal = |id: &Value| {
            let live = report["shared_sig"]["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == *id)
                .unwrap();
            let matches = expected_start["chords"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|f| f["bounds"] == live["bounds"])
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            matches[0]["id"].as_u64().unwrap()
        };
        let mut initial = actual["starting_state"].clone();
        for voice in initial["voices"].as_array_mut().unwrap() {
            voice["chord_ids"] = json!(
                voice["chord_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(&start_ordinal)
                    .collect::<Vec<_>>()
            );
            if !voice["measure_rest_chord_id"].is_null() {
                voice["measure_rest_chord_id"] =
                    json!(start_ordinal(&voice["measure_rest_chord_id"]));
            }
        }
        for chord in initial["chords"].as_array_mut().unwrap() {
            chord["id"] = json!(start_ordinal(&chord["id"]));
        }
        initial["chords"]
            .as_array_mut()
            .unwrap()
            .sort_by_key(|c| c["id"].as_u64());
        assert_eq!(initial["voices"], expected_start["voices"]);
        assert_eq!(initial["chords"], expected_start["chords"]);
        let mapper: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-rookie-mapping/observed.json"
        ))
        .unwrap();
        let expected_steps = &mapper["pages"][page_index]["measures"]
            [actual["measure_number"].as_u64().unwrap() as usize - 1]["steps"];
        let mut steps = actual["mapping_steps"].clone();
        for step in steps.as_array_mut().unwrap() {
            for key in ["rookies", "actives"] {
                step[key] = json!(
                    step[key]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(&start_ordinal)
                        .collect::<Vec<_>>()
                );
            }
            for t in step["known_times"].as_array_mut().unwrap() {
                t["chord_ids"] = json!(
                    t["chord_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(&start_ordinal)
                        .collect::<Vec<_>>()
                );
            }
            for cost in step["costs"].as_array_mut().unwrap() {
                for key in ["rookie", "active"] {
                    cost[key] = json!(start_ordinal(&cost[key]));
                }
            }
            if !step["mapping"].is_null() {
                for pair in step["mapping"]["pairs"].as_array_mut().unwrap() {
                    for key in ["rookie", "active"] {
                        pair[key] = json!(start_ordinal(&pair[key]));
                    }
                }
            }
            for chord in step["after"].as_array_mut().unwrap() {
                chord["chord_id"] = json!(start_ordinal(&chord["chord_id"]));
            }
        }
        assert_eq!(steps, *expected_steps);
    }
}
