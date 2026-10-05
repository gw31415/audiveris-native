use serde_json::{Value, json};
pub fn same(a: &Value, b: &Value) {
    match a {
        Value::Object(m) => {
            for (k, v) in m {
                same(v, &b[k])
            }
        }
        Value::Array(a) => {
            let b = b.as_array().unwrap();
            assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b) {
                same(x, y)
            }
        }
        Value::Number(n) if n.is_f64() => assert!(
            (n.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1e-12,
            "{a} != {b}"
        ),
        _ => assert_eq!(a, b),
    }
}
pub fn key(c: &Value) -> Value {
    json!([c["head_bounds"], c["shape"], c["side"], c["vertical"]])
}
pub fn compare(o: &Value, n: &Value) {
    assert_eq!(o["head_corners"], n["corner_count"]);
    assert_eq!(o["accepted_creations"], n["accepted_creations"]);
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        assert_eq!(a["cache_size"], b["cache_size"]);
        assert_eq!(
            a["cases"].as_array().unwrap().len(),
            b["cases"].as_array().unwrap().len()
        );
        for e in a["cases"].as_array().unwrap() {
            let c = b["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| key(e) == key(c))
                .unwrap();
            same(e, c)
        }
    }
    for k in [
        "final_stem_glyph_selection_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
        "automatic_raster_pipeline_complete",
    ] {
        assert_eq!(n[k], false)
    }
}
