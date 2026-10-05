// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Registry},
    stem_chunk_registration,
};
use serde_json::{Value, json};
pub fn observe(original: &Value) -> Value {
    let mut groups = Vec::new();
    for (g, group) in original["groups"].as_array().unwrap().iter().enumerate() {
        let mut registry = Registry::default();
        let mut cases = Vec::new();
        for (i, c) in group["cases"].as_array().unwrap().iter().enumerate() {
            let raw = c["raw_glyphs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| beam_dispatch::glyph_from_json(p).unwrap())
                .collect::<Vec<_>>();
            let existing = serde_json::from_value::<Vec<usize>>(c["existing_ids"].clone()).unwrap();
            let result =
                stem_chunk_registration::register_and_exclude(&mut registry, &raw, &existing)
                    .unwrap_or_else(|e| panic!("group{g} case{i}: {e}"));
            let value = json!(result);
            for k in [
                "registered_ids",
                "retained_ids",
                "new_count",
                "registry_count",
            ] {
                assert_eq!(value[k], c[k], "group{g} case{i} {k}");
            }
            // Every registered token still names its original physical glyph,
            // including all tokens excluded from the returned candidate list.
            for (glyph, id) in raw.iter().zip(&result.registered_ids) {
                assert_eq!(
                    beam_dispatch::glyph_json(glyph),
                    beam_dispatch::glyph_json(&registry.get(*id).unwrap().glyph)
                );
            }
            cases.push(value);
        }
        groups.push(json!({"cases":cases}));
    }
    json!({"groups":groups,"JVM_ids_normalized_to_canonical_identity_tokens":true,"chunk_filters_complete":false,"registered_to_SIG":false,"heads_complete":false,"stems_complete":false})
}
