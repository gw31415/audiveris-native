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
pub fn compare(o: &Value, n: &Value, page: &Value) {
    assert_eq!(o["native_outputs_used"], false);
    assert_eq!(o["corners"], 1408);
    let mut count = 0;
    let mut selected = 0;
    let mut sections = 0;
    let reg = n["physical_registry"].as_array().unwrap();
    let upstream = page["head_registration"]["live_glyph_retrieval"]["registry"]
        .as_array()
        .unwrap();
    assert_eq!(&reg[..upstream.len()], upstream);
    assert_eq!(upstream.len(), 1220);
    let vlag = page["grid_ir"]["vlag"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["id"].as_u64().unwrap(), s))
        .collect::<BTreeMap<_, _>>();
    for ((a, b), g) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
        .zip(page["grid_ir"]["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        assert_eq!(a["main_stem_thickness"], 4);
        assert_eq!(a["stump_area_dy"], 2);
        assert_eq!(a["stump_area_dx_in"], 1.1);
        assert_eq!(a["stump_area_dx_out"], 1.1);
        let native = g["vertical_section_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| vlag[&id.as_u64().unwrap()])
            .collect::<Vec<_>>();
        let orig = a["sections"].as_array().unwrap();
        assert_eq!(orig.len(), native.len());
        for (x, y) in orig.iter().zip(native) {
            for field in ["id", "first_pos", "runs", "bounds"] {
                assert_eq!(x[field], y[field]);
            }
            sections += 1;
        }
        let expected = a["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| (key(c), c))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(expected.len(), b["cases"].as_array().unwrap().len());
        for c in b["cases"].as_array().unwrap() {
            let e = expected[&key(&c["head"])];
            assert_eq!(e["stump"], c["selected_stump"], "{}", key(&c["head"]));
            if let Some(id) = c["selected_stump_id"].as_u64() {
                assert_eq!(reg[id as usize - 1]["glyph"], e["stump"]);
                selected += 1;
            }
            if let Some(id) = c["registered_fallback_id"].as_u64() {
                assert!(id > 0 && id as usize <= reg.len());
            }
            count += 1;
        }
    }
    assert_eq!((count, selected), (1408, 724));
    assert!(sections > 0);
    assert_eq!(n["selected_section_stump_corners"], 147);
    assert_eq!(n["registered_rejected_fallback_corners"], 554);
    assert_eq!(n["new_registered_fallback_glyphs"], 346);
    assert_eq!(n["image_corner_stump_selection_complete"], true);
    for k in [
        "final_stem_glyph_selection_complete",
        "automatic_raster_pipeline_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(n[k], false);
    }
}
