// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, head_seed_search, template_probes};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-all-shapes/seed-controls.json"
    ))
    .unwrap()
}
fn image(c: &Value) -> GrayImage {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(c["image"].as_str().unwrap());
    let image = GrayImage::decode(&std::fs::read(p).unwrap()).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&image.pixels)),
        c["binary_sha256"]
    );
    image
}
fn equal(a: &Value, e: &Value) {
    match (a, e) {
        (Value::Number(a), Value::Number(e)) if a.is_f64() || e.is_f64() => assert!(
            (a.as_f64().unwrap() - e.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {e}"
        ),
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(a.len(), e.len());
            for (a, e) in a.iter().zip(e) {
                equal(a, e);
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            assert_eq!(a.len(), e.len());
            for (k, e) in e {
                equal(&a[k], e);
            }
        }
        _ => assert_eq!(a, e),
    }
}
fn overlaps(b: [i32; 4], r: &Value) -> bool {
    if r.is_null() {
        return false;
    }
    let r: [i32; 4] = serde_json::from_value(r.clone()).unwrap();
    b[0] < r[0] + r[2] && r[0] < b[0] + b[2] && b[1] < r[1] + r[3] && r[1] < b[1] + b[3]
}
#[test]
fn original_1440_new_hollow_seed_windows_both_anchors_and_bar_vetoes() {
    let g = controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["images"].as_array().unwrap().len(), 240);
    let shapes: Vec<&str> = g["source_shapes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert_eq!(shapes.len(), 5);
    let mut windows = 0;
    let mut noncentral = 0;
    for c in g["images"].as_array().unwrap() {
        let bank = Bank::load_all_head_coverage(c["family"].as_str().unwrap()).unwrap();
        let table = template_probes::distances(&image(c), None).unwrap();
        for w in c["windows"].as_array().unwrap() {
            let xo: Vec<i32> = serde_json::from_value(w["x_offsets"].clone()).unwrap();
            let yo: Vec<i32> = serde_json::from_value(w["y_offsets"].clone()).unwrap();
            let (mut calls, mut vetoes) = (0, 0);
            let actual = head_seed_search::search_all_heads_with_veto(
                &table,
                (&bank, c["point_size"].as_u64().unwrap() as u16),
                serde_json::from_value(w["pivot"].clone()).unwrap(),
                (&xo, &yo),
                w["anchor"].as_str().unwrap(),
                &shapes,
                &mut |b| {
                    calls += 1;
                    let veto = overlaps(b, &w["veto_rectangle"]);
                    vetoes += usize::from(veto);
                    Ok(veto)
                },
            )
            .unwrap();
            equal(&json!(actual), &w["matches"]);
            assert_eq!(calls, w["original_eval_count"].as_u64().unwrap());
            assert_eq!(
                vetoes as u64,
                w["original_bar_veto_count"].as_u64().unwrap()
            );
            if w["veto_mode"] == "central" {
                assert!(actual.is_empty());
                assert_eq!(calls, shapes.len() as u64);
            }
            if c["variant"] == "empty" {
                assert!(actual.is_empty());
            }
            noncentral += w["original_noncentral_veto_count"].as_u64().unwrap();
            windows += 1;
        }
    }
    assert_eq!(windows, 1440);
    assert!(noncentral > 0);
}
