// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, beam_dispatch, head_glyph_retrieval, head_range_search};
use audiveris_template_bank::Bank;
use serde_json::Value;
use sha2::{Digest, Sha256};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-font-coverage/controls.json"
    ))
    .unwrap()
}
fn image(c: &Value) -> GrayImage {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-head-font-coverage")
        .join(c["image"].as_str().unwrap());
    let image = GrayImage::decode(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&image.pixels)),
        c["binary_sha256"]
    );
    image
}
fn equal(a: &Value, e: &Value) {
    match (a, e) {
        (Value::Number(a), Value::Number(e)) if a.is_f64() || e.is_f64() => {
            assert!(
                (a.as_f64().unwrap() - e.as_f64().unwrap()).abs() < 1e-12,
                "{a} != {e}"
            );
        }
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
#[test]
fn original_three_fonts_four_sizes_four_shapes_range_and_stemless_gates() {
    let g = controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["images"].as_array().unwrap().len(), 192);
    for c in g["images"].as_array().unwrap() {
        let bank = Bank::load_coverage(c["family"].as_str().unwrap()).unwrap();
        let image = image(c);
        let table = audiveris_native::template_probes::distances(&image, None).unwrap();
        let pivot = serde_json::from_value(c["pivot"].clone()).unwrap();
        for mode in c["modes"].as_array().unwrap() {
            let actual = head_range_search::candidates_at_size(
                &table,
                (&bank, c["point_size"].as_u64().unwrap() as u16),
                pivot,
                &[0, -1, 1],
                mode["black_relevant"].as_bool().unwrap(),
                &mut |_| Ok(false),
            )
            .unwrap();
            equal(&actual["matches"], &mode["matches"]);
            equal(&actual["rejected"], &mode["rejected"]);
        }
    }
}
#[test]
fn original_three_fonts_four_sizes_physical_glyphs_clean_erased_clipped_and_empty() {
    let g = controls();
    for c in g["images"].as_array().unwrap() {
        let bank = Bank::load_coverage(c["family"].as_str().unwrap()).unwrap();
        let result = head_glyph_retrieval::retrieve_at_size(
            &image(c),
            &bank,
            c["point_size"].as_u64().unwrap() as u16,
            &c["input"],
        )
        .unwrap();
        assert_eq!(
            result
                .as_ref()
                .map(beam_dispatch::glyph_json)
                .unwrap_or(Value::Null),
            c["glyph"],
            "{}",
            c["image"]
        );
    }
}
#[test]
fn missing_size_or_shape_is_explicit_and_matching_veto_still_abandons_center() {
    let c = &controls()["images"][0];
    let bank = Bank::load_coverage("Bravura").unwrap();
    let image = image(c);
    let table = audiveris_native::template_probes::distances(&image, None).unwrap();
    let pivot = serde_json::from_value(c["pivot"].clone()).unwrap();
    for size in [0, 31, 33, 50, 52, 65, u16::MAX] {
        assert!(
            head_range_search::candidates_at_size(
                &table,
                (&bank, size),
                pivot,
                &[0, -1, 1],
                true,
                &mut |_| Ok(false)
            )
            .is_err()
        );
        assert!(head_glyph_retrieval::retrieve_at_size(&image, &bank, size, &c["input"]).is_err());
    }
    let mut veto_calls = 0;
    let actual = head_range_search::candidates_at_size(
        &table,
        (&bank, 32),
        pivot,
        &[0, -1, 1],
        true,
        &mut |_| {
            veto_calls += 1;
            Ok(true)
        },
    )
    .unwrap();
    assert_eq!(veto_calls, 4);
    assert!(actual["matches"].as_array().unwrap().is_empty());
    assert!(actual["rejected"].as_array().unwrap().is_empty());
}
