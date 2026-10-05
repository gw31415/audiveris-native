// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::stem_length_map::{self, Item};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn items(group: &Value) -> Vec<Option<Item>> {
    group["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            if i.is_null() {
                None
            } else {
                Some(Item {
                    gap: i["gap"].as_bool().unwrap(),
                    contrib: i["contrib"].as_i64().unwrap() as i32,
                    line: serde_json::from_value(i["line"].clone()).unwrap(),
                    head_bounds: serde_json::from_value(i["head_bounds"].clone()).unwrap(),
                })
            }
        })
        .collect()
}
pub fn observe(original: &Value) -> Value {
    let gaps: BTreeMap<i32, i32> = serde_json::from_value(original["gap_map"].clone()).unwrap();
    let mut groups = Vec::new();
    let mut total = 0;
    for g in original["groups"].as_array().unwrap() {
        let base = items(g);
        let dir = g["direction"].as_i64().unwrap() as i8;
        let y = g["start_y"].as_f64().unwrap();
        for (i, j) in base.iter().zip(g["items"].as_array().unwrap()) {
            if let Some(i) = i {
                assert_eq!(
                    json!(i.line.map(|l| stem_length_map::line_bounds(l).unwrap())),
                    j["line_bounds"]
                );
            }
        }
        let prefixes = stem_length_map::prefix_lengths(&base, dir, y).unwrap();
        assert_eq!(json!(prefixes), g["prefix_lengths"]);
        let mut starts = Vec::new();
        for s in g["start_y_controls"].as_array().unwrap() {
            let p = stem_length_map::prefix_lengths(&base, dir, s["start_y"].as_f64().unwrap())
                .unwrap();
            assert_eq!(json!(p), s["prefix_lengths"]);
            starts.push(json!(p));
        }
        let mut cases = Vec::new();
        for c in g["cases"].as_array().unwrap() {
            let mut selected = if c["mode"] == "empty" {
                Vec::new()
            } else {
                base.clone()
            };
            if let Some(index) = c["gap_index"].as_u64() {
                selected[index as usize].as_mut().unwrap().contrib =
                    c["gap_contrib"].as_i64().unwrap() as i32;
            }
            let initial = serde_json::from_value(c["initial_lengths"].clone()).unwrap();
            let lengths = stem_length_map::retrieve_lengths(
                &selected,
                dir,
                y,
                &gaps,
                c["max_profile"].as_i64().unwrap() as i32,
                &initial,
            )
            .unwrap();
            assert_eq!(json!(lengths), c["lengths"]);
            cases.push(json!(lengths));
            total += 1;
        }
        groups.push(
            json!({"prefix_lengths":prefixes,"start_y_prefix_lengths":starts,"case_lengths":cases}),
        );
    }
    json!({"groups":groups,"controls":total,"automatic_items_parts_page_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false})
}
