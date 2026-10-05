// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Registry},
    stem_chunks,
    stem_parts_alignment::Parameters,
};
use serde_json::{Value, json};
pub fn observe(original: &Value) -> Value {
    let mut groups = Vec::new();
    for (g, group) in original["groups"].as_array().unwrap().iter().enumerate() {
        let mut cases = Vec::new();
        for (i, c) in group["cases"].as_array().unwrap().iter().enumerate() {
            let mut registry = Registry::default();
            for e in c["initial_catalog"].as_array().unwrap() {
                assert_eq!(
                    json!(
                        registry
                            .register(&beam_dispatch::glyph_from_json(&e["glyph"]).unwrap())
                            .unwrap()
                    ),
                    e["id"]
                );
            }
            let raw = c["raw_glyphs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
                .collect::<Vec<_>>();
            let existing = serde_json::from_value::<Vec<usize>>(c["existing_ids"].clone()).unwrap();
            let stump = c["stump_id"].as_u64().map(|n| n as usize);
            let head = if c["head_glyph"].is_null() {
                None
            } else {
                Some(beam_dispatch::glyph_from_json(&c["head_glyph"]).unwrap())
            };
            let p = Parameters::new(
                c["direction"].as_i64().unwrap() as i8,
                c["max_dx"].as_f64().unwrap(),
                c["max_dy"].as_f64().unwrap(),
                c["slope"].as_f64().unwrap(),
            )
            .unwrap();
            let result = stem_chunks::select(
                &mut registry,
                &raw,
                &existing,
                stump,
                head.as_ref().map(|h| (h, c["vip"].as_bool().unwrap())),
                &p,
            )
            .unwrap_or_else(|e| panic!("group{g} case{i}: {e}"));
            let mut value = json!(result);
            for k in [
                "registered_ids",
                "retained_ids",
                "new_count",
                "registry_count",
            ] {
                assert_eq!(value[k], c[k], "group{g} case{i} {k}");
            }
            let catalog = registry
                .snapshot()
                .as_array()
                .unwrap()
                .iter()
                .map(|e| json!({"id":e["id"],"glyph":e["glyph"]}))
                .collect::<Vec<_>>();
            value["final_catalog"] = json!(catalog);
            assert_eq!(
                value["final_catalog"], c["final_catalog"],
                "group{g} case{i} catalog"
            );
            cases.push(value);
        }
        groups.push(json!({"cases":cases}));
    }
    json!({"groups":groups,"JVM_ids_normalized_to_canonical_identity_tokens":true,"supplied_raw_chunks_composition":true,"automatic_page_STEMS_complete":false,"registered_to_SIG":false,"heads_complete":false,"stems_complete":false})
}
