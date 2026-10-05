// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Registry},
    stem_chunk_head_filter,
};
use serde_json::{Value, json};
pub fn observe(original: &Value) -> Value {
    let mut groups = Vec::new();
    for (g, group) in original["groups"].as_array().unwrap().iter().enumerate() {
        let head = beam_dispatch::glyph_from_json(&group["head_glyph"]).unwrap();
        let mut registry = Registry::default();
        for entry in group["catalog"].as_array().unwrap() {
            let glyph = beam_dispatch::glyph_from_json(&entry["glyph"]).unwrap();
            assert_eq!(json!(registry.register(&glyph).unwrap()), entry["id"]);
        }
        let before = format!("{registry:?}");
        let mut cases = Vec::new();
        for (i, c) in group["cases"].as_array().unwrap().iter().enumerate() {
            let ids = serde_json::from_value::<Vec<usize>>(c["part_ids"].clone()).unwrap();
            let result =
                stem_chunk_head_filter::filter(&registry, &ids, &head, c["vip"].as_bool().unwrap())
                    .unwrap_or_else(|e| panic!("group{g} case{i}: {e}"));
            let value = json!(result);
            for k in ["remaining_weights", "retained_ids"] {
                assert_eq!(value[k], c[k], "group{g} case{i} {k}");
            }
            cases.push(value);
        }
        assert_eq!(format!("{registry:?}"), before);
        groups.push(json!({"cases":cases}));
    }
    json!({"groups":groups,"original_VIP_condition_preserved":true,"JVM_ids_normalized_to_canonical_identity_tokens":true,"chunk_filter_sort_composition_complete":false,"registered_to_SIG":false,"heads_complete":false,"stems_complete":false})
}
