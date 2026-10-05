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
    for k in [
        "initial_seed_occurrences",
        "initial_linker_occurrences",
        "retained_linker_occurrences",
    ] {
        assert_eq!(o[k], n[k]);
    }
    let mut count = 0;
    let mut items = 0;
    assert_eq!(
        o["systems"].as_array().unwrap().len(),
        n["systems"].as_array().unwrap().len()
    );
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
                "theoretical_line",
                "builder_y_range",
                "start_stump",
                "initial_seeds",
                "selected_linkers",
                "retained_seeds",
                "retained_linker_indices",
                "last_head_y",
                "pre_chunk_items",
            ] {
                same(&e[k], &c[k]);
            }
            count += 1;
            items += c["pre_chunk_items"].as_array().unwrap().len();
        }
    }
    assert_eq!((count, items), (1408, 1884));
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
