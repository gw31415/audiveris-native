// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::{Value, json};
fn key(c: &Value) -> Value {
    let h = if c.get("head").is_some() {
        &c["head"]
    } else {
        c
    };
    json!([h["head_bounds"], h["shape"], h["side"], h["vertical"]])
}
fn same(a: &Value, b: &Value) {
    match a {
        Value::Object(m) => {
            for (k, v) in m {
                same(v, &b[k]);
            }
        }
        Value::Array(v) => {
            let w = b.as_array().unwrap();
            assert_eq!(v.len(), w.len());
            for (a, b) in v.iter().zip(w) {
                same(a, b);
            }
        }
        Value::Number(n) if n.is_f64() => {
            assert!((n.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1e-12)
        }
        _ => assert_eq!(a, b),
    }
}
pub fn compare(o: &Value, n: &Value) {
    assert_eq!(o["head_corners"], n["corner_count"]);
    assert_eq!(n["automatic_head_chunk_lookup_complete"], true);
    let mut count = 0;
    let mut raw = 0;
    let mut kept = 0;
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        assert_eq!(
            a["cases"].as_array().unwrap().len(),
            b["cases"].as_array().unwrap().len()
        );
        for c in b["cases"].as_array().unwrap() {
            let e = a["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| key(e) == key(c))
                .unwrap();
            for k in [
                "selected_vertical_ids",
                "selected_horizontal_ids",
                "selected_vertical_sections",
                "selected_horizontal_sections",
                "raw_glyphs",
                "retained_chunks",
                "head_glyph",
                "head_vip",
                "max_stem_thickness",
                "max_line_section_dx",
            ] {
                same(&e["chunk_lookup"][k], &c["chunk_lookup"][k]);
            }
            assert_eq!(
                c["chunk_lookup"]["raw_glyphs"],
                c["chunk_lookup"]["registered_glyphs"]
            );
            count += 1;
            raw += c["chunk_lookup"]["raw_glyphs"].as_array().unwrap().len();
            kept += c["chunk_lookup"]["retained_chunks"]
                .as_array()
                .unwrap()
                .len();
        }
    }
    assert_eq!((count, raw, kept), (1408, 3413, 2414));
    assert_eq!(n["new_chunk_registrations"], 769);
    assert_eq!(n["selected_vertical_section_occurrences"], 5186);
    assert_eq!(n["selected_horizontal_section_occurrences"], 6255);
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
