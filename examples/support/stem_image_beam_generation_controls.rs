// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::Value;
fn links(a: &Value, b: &Value) {
    let a = a.as_array().unwrap();
    let b = b.as_array().unwrap();
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b) {
        for k in ["side", "stump", "anchor", "directions"] {
            assert_eq!(x[k], y[k]);
        }
        for (u, v) in x["reference"]
            .as_array()
            .unwrap()
            .iter()
            .zip(y["reference"].as_array().unwrap())
        {
            assert!((u.as_f64().unwrap() - v.as_f64().unwrap()).abs() <= 1e-12);
        }
    }
}
pub fn compare(o: &Value, n: &Value) {
    let mut stumps = 0;
    let mut base = 0;
    let mut final_count = 0;
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        for k in [
            "main_stem_thickness",
            "max_stem_thickness",
            "min_beam_stems_dx",
            "max_beam_seed_dx",
            "max_beam_linker_dx",
            "min_beam_stump_dy",
        ] {
            assert_eq!(a[k], b[k]);
        }
        for (x, y) in a["beam_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .zip(b["beam_catalog"].as_array().unwrap())
        {
            assert_eq!(x["beam"]["median"], y["beam"]["median"]);
            assert_eq!(x["beam"]["height"], y["beam"]["height"]);
            for k in ["admitted_seeds", "purged_seeds", "stumps"] {
                assert_eq!(x[k], y[k]);
            }
            links(&x["base_blinkers"], &y["base_blinkers"]);
            links(&x["final_blinkers"], &y["final_blinkers"]);
            stumps += y["stumps"].as_array().unwrap().len();
            base += y["base_blinkers"].as_array().unwrap().len();
            final_count += y["final_blinkers"].as_array().unwrap().len();
        }
    }
    assert_eq!((stumps, base, final_count), (40, 43, 90));
    assert_eq!(n["selected_blinker_occurrences"], 339);
    for k in [
        "automatic_raster_pipeline_complete",
        "final_stem_glyph_selection_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(n[k], false);
    }
}
