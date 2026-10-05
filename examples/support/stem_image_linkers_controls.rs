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
fn normalize(v: &Value) -> Value {
    serde_json::json!({"head_bounds":v["head_bounds"],"shape":v["shape"],"side":v["side"],"vertical":v["vertical"],"reference":v["reference"],"stump":v["stump"]})
}
pub fn compare(o: &Value, n: &Value) {
    assert_eq!(o["native_outputs_used"], false);
    assert_eq!(
        o["original_phase"],
        "inspectStems_before_linkStems_and_finalizeStems"
    );
    assert_eq!(o["corners"], n["corner_count"]);
    assert_eq!(n["corner_count"], 1408);
    assert_eq!(
        o["selected_other_head_linkers"],
        n["selected_other_head_linker_occurrences"]
    );
    assert_eq!(n["selected_other_head_linker_occurrences"], 1401);
    let mut count = 0;
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        assert_eq!(b["min_head_head_dy"], 3);
        let e = a["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| (key(&c["head"]), c))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(e.len(), b["cases"].as_array().unwrap().len());
        for c in b["cases"].as_array().unwrap() {
            let x = e[&key(&c["head"])];
            let actual = c["selected_other_head_linkers"]
                .as_array()
                .unwrap()
                .iter()
                .map(normalize)
                .collect::<Vec<_>>();
            assert_eq!(
                actual,
                x["selected"].as_array().unwrap().clone(),
                "{}",
                key(&c["head"])
            );
            count += 1;
        }
    }
    assert_eq!(count, 1408);
    assert_eq!(o["controls"].as_array().unwrap().len(), 14);
    assert_eq!(n["other_head_linker_selection_complete"], true);
    for f in [
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
