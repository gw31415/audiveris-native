// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn key(v: &Value) -> String {
    serde_json::to_string(&json!([
        v["head_bounds"],
        v["shape"],
        v["side"],
        v["vertical"]
    ]))
    .unwrap()
}
fn ordered(v: impl Iterator<Item = Value>) -> Vec<String> {
    let mut s = v
        .map(|x| serde_json::to_string(&x).unwrap())
        .collect::<Vec<_>>();
    s.sort();
    s
}
pub fn compare(o: &Value, n: &Value) {
    let registry = n["physical_registry"].as_array().unwrap();
    let glyph = |id: &Value| registry[id.as_u64().unwrap() as usize - 1]["glyph"].clone();
    let mut count = 0;
    let mut selected = 0;
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        for p in [
            "max_head_in_dx",
            "max_head_out_dx",
            "max_head_seed_dy",
            "min_head_stump_dy",
        ] {
            assert_eq!(a[p], b[p]);
        }
        assert_eq!(
            ordered(a["no_stem_areas"].as_array().unwrap().iter().cloned()),
            ordered(b["no_stem_areas"].as_array().unwrap().iter().cloned())
        );
        assert_eq!(
            ordered(a["surviving_seeds"].as_array().unwrap().iter().cloned()),
            ordered(
                b["surviving_seed_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(glyph)
            )
        );
        let expected = a["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (key(v), v))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(expected.len(), b["cases"].as_array().unwrap().len());
        for c in b["cases"].as_array().unwrap() {
            let e = expected[&key(&c["head"])];
            assert_eq!(e["reference"], c["head"]["reference"]);
            assert_eq!(
                ordered(e["neighbors"].as_array().unwrap().iter().cloned()),
                ordered(c["neighbor_seed_ids"].as_array().unwrap().iter().map(glyph))
            );
            let candidates=c["candidates"].as_array().unwrap().iter().map(|v|json!({"glyph":v["glyph"],"gap":v["gap"],"eligible":v["eligible"],"stands_out":v["stands_out"]})).collect::<Vec<_>>();
            assert_eq!(e["candidates"], json!(candidates));
            assert_eq!(e["selected_seed"], c["selected_seed"]);
            if !c["selected_seed"].is_null() {
                assert_eq!(e["actual_stump"], c["selected_seed"]);
                assert_eq!(glyph(&c["selected_seed_id"]), c["selected_seed"]);
                selected += 1;
            }
            count += 1;
        }
    }
    assert_eq!((count, selected), (1408, 577));
    assert_eq!(n["section_fallback_corners"], 831);
    assert_eq!(n["image_seed_branch_complete"], true);
    for k in [
        "automatic_stump_selection_complete",
        "automatic_raster_pipeline_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(n[k], false);
    }
}
