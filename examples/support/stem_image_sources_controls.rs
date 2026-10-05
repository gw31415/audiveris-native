// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::Value;
use std::collections::BTreeMap;
fn key(v: &Value) -> String {
    serde_json::to_string(&serde_json::json!([
        v["head_bounds"],
        v["shape"],
        v["side"],
        v["vertical"]
    ]))
    .unwrap()
}
pub fn compare(o: &Value, n: &Value) {
    assert_eq!(o["native_outputs_used"], false);
    assert_eq!(
        o["original_phase"],
        "inspectStems_before_linkStems_and_finalizeStems"
    );
    assert_eq!(o["corners"], n["corner_count"]);
    assert_eq!(n["corner_count"], 1408);
    assert_eq!(o["selected_seed_occurrences"], 248);
    assert_eq!(o["aligned_seed_occurrences"], 246);
    assert_eq!(
        o["selected_seed_occurrences"],
        n["selected_builder_seed_occurrences"]
    );
    assert_eq!(
        o["aligned_seed_occurrences"],
        n["aligned_builder_seed_occurrences"]
    );
    assert_eq!(
        o["systems"].as_array().unwrap().len(),
        n["systems"].as_array().unwrap().len()
    );
    let mut count = 0;
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        for f in [
            "system",
            "min_seed_contribution",
            "max_line_seed_dx",
            "max_alignment_dx",
            "max_alignment_dy",
        ] {
            assert_eq!(a[f], b[f]);
        }
        let e = a["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| (key(c), c))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(e.len(), a["cases"].as_array().unwrap().len());
        assert_eq!(e.len(), b["cases"].as_array().unwrap().len());
        for c in b["cases"].as_array().unwrap() {
            let x = e[&key(&c["head"])];
            for f in [
                "theoretical_line",
                "y_range",
                "stump",
                "selected_seeds",
                "aligned_seeds",
            ] {
                assert_eq!(x[f], c[f], "{} {f}", key(&c["head"]));
            }
            count += 1;
        }
    }
    assert_eq!(count, 1408);
    assert_eq!(n["image_builder_seed_selection_complete"], true);
    assert_eq!(n["image_seed_alignment_complete"], true);
    for f in [
        "other_head_linker_selection_complete",
        "beam_sibling_linker_selection_complete",
        "final_stem_glyph_selection_complete",
        "automatic_raster_pipeline_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(n[f], false);
    }
}
