// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, head_seed_search, template_probes};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-seed-coverage/controls.json"
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
fn original_2016_seed_windows_seven_shapes_three_fonts_four_sizes_both_anchors_and_vetoes() {
    let g = controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["images"].as_array().unwrap().len(), 336);
    assert_eq!(json!(head_seed_search::STEM_SHAPES), g["source_shapes"]);
    let mut windows = 0;
    let mut noncentral = 0;
    for c in g["images"].as_array().unwrap() {
        let bank = Bank::load_drum_coverage(c["family"].as_str().unwrap()).unwrap();
        let table = template_probes::distances(&image(c), None).unwrap();
        for w in c["windows"].as_array().unwrap() {
            let xo: Vec<i32> = serde_json::from_value(w["x_offsets"].clone()).unwrap();
            let yo: Vec<i32> = serde_json::from_value(w["y_offsets"].clone()).unwrap();
            let (mut calls, mut vetoes) = (0, 0);
            let actual = head_seed_search::search_selected_with_veto(
                &table,
                (&bank, c["point_size"].as_u64().unwrap() as u16),
                serde_json::from_value(w["pivot"].clone()).unwrap(),
                (&xo, &yo),
                w["anchor"].as_str().unwrap(),
                &head_seed_search::STEM_SHAPES,
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
                assert_eq!(calls, 7);
            }
            if c["variant"] == "empty" {
                assert!(actual.is_empty());
            }
            noncentral += w["original_noncentral_veto_count"].as_u64().unwrap();
            windows += 1;
        }
    }
    assert_eq!(windows, 2016);
    assert!(noncentral > 0);
}
#[test]
fn absent_assets_duplicates_and_nonstem_shapes_fail_before_veto_side_effects() {
    let white = GrayImage {
        width: 128,
        height: 96,
        pixels: vec![255; 128 * 96],
    };
    let table = template_probes::distances(&white, None).unwrap();
    let bank = Bank::load_drum_coverage("Bravura").unwrap();
    let mut calls = 0;
    for shapes in [
        vec!["BREVE"],
        vec!["WHOLE_NOTE"],
        vec!["NOTEHEAD_VOID_SMALL"],
        vec!["NOTEHEAD_BLACK", "NOTEHEAD_BLACK"],
        vec!["NOTEHEAD_VOID", "UNKNOWN"],
    ] {
        assert_eq!(
            head_seed_search::search_selected_with_veto(
                &table,
                (&bank, 51),
                [40, 45],
                (&[0, 1], &[0, -1]),
                "LEFT_STEM",
                &shapes,
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .unwrap_err(),
            "unsupported_head_seed_shapes"
        );
    }
    for size in [0, 31, 33, 50, 52, 65, u16::MAX] {
        assert!(
            head_seed_search::search_selected_with_veto(
                &table,
                (&bank, size),
                [40, 45],
                (&[0, 1], &[0, -1]),
                "RIGHT_STEM",
                &head_seed_search::STEM_SHAPES,
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    let builtin = Bank::load_builtin().unwrap();
    assert!(
        head_seed_search::search_selected_with_veto(
            &table,
            (&builtin, 51),
            [40, 45],
            (&[0, 1], &[0, -1]),
            "LEFT_STEM",
            &["NOTEHEAD_VOID", "NOTEHEAD_CROSS"],
            &mut |_| {
                calls += 1;
                Ok(false)
            }
        )
        .is_err()
    );
    assert_eq!(calls, 0);
}
#[test]
fn empty_catalog_window_limits_and_veto_failure_keep_seed_contract() {
    let white = GrayImage {
        width: 128,
        height: 96,
        pixels: vec![255; 128 * 96],
    };
    let table = template_probes::distances(&white, None).unwrap();
    let bank = Bank::load_drum_coverage("Leland").unwrap();
    let mut calls = 0;
    assert!(
        head_seed_search::search_selected_with_veto(
            &table,
            (&bank, 32),
            [40, 45],
            (&[0], &[0]),
            "LEFT_STEM",
            &[],
            &mut |_| {
                calls += 1;
                Ok(false)
            }
        )
        .unwrap()
        .is_empty()
    );
    for anchor in ["MIDDLE_LEFT", "NONE", "unknown"] {
        assert!(
            head_seed_search::search_selected_with_veto(
                &table,
                (&bank, 32),
                [40, 45],
                (&[0], &[0]),
                anchor,
                &[],
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    for (pivot, xo, yo) in [
        ([4_000_001, 0], vec![0], vec![0]),
        ([0, 0], vec![], vec![0]),
        ([0, 0], vec![1], vec![0]),
        ([0, 0], vec![0, 129], vec![0]),
        ([0, 0], vec![0; 130], vec![0]),
    ] {
        assert!(
            head_seed_search::search_selected_with_veto(
                &table,
                (&bank, 32),
                pivot,
                (&xo, &yo),
                "LEFT_STEM",
                &[],
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    assert_eq!(calls, 0);
    assert_eq!(
        head_seed_search::search_selected_with_veto(
            &table,
            (&bank, 32),
            [40, 45],
            (&[0, 1], &[0]),
            "LEFT_STEM",
            &head_seed_search::STEM_SHAPES,
            &mut |_| {
                calls += 1;
                Err("fixture_veto_failure".into())
            }
        )
        .unwrap_err(),
        "fixture_veto_failure"
    );
    assert_eq!(calls, 1);
}
#[test]
fn explicit_source_order_and_legacy_builtin_51_search_are_preserved() {
    let g = controls();
    let c = g["images"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["family"] == "Bravura"
                && c["point_size"] == 51
                && c["rendered_shape"] == "NOTEHEAD_VOID"
                && c["variant"] == "clean"
        })
        .unwrap();
    let table = template_probes::distances(&image(c), None).unwrap();
    let bank = Bank::load_drum_coverage("Bravura").unwrap();
    let builtin = Bank::load_builtin().unwrap();
    let mut reversed = head_seed_search::STEM_SHAPES;
    reversed.reverse();
    for w in c["windows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| w["veto_mode"] == "none")
    {
        let pivot = serde_json::from_value(w["pivot"].clone()).unwrap();
        let anchor = w["anchor"].as_str().unwrap();
        let result = head_seed_search::search_selected_with_veto(
            &table,
            (&bank, 51),
            pivot,
            (&[0, 1, -1, 2, -2], &[0, -1, 1]),
            anchor,
            &reversed,
            &mut |_| Ok(false),
        )
        .unwrap();
        let mut expected = w["matches"].as_array().unwrap().clone();
        expected.reverse();
        equal(&json!(result), &json!(expected));
        let legacy = head_seed_search::search_with_veto(
            &table,
            &builtin,
            pivot,
            (&[0, 1, -1, 2, -2], &[0, -1, 1]),
            anchor,
            &mut |_| Ok(false),
        )
        .unwrap();
        let selected = head_seed_search::search_selected_with_veto(
            &table,
            (&builtin, 51),
            pivot,
            (&[0, 1, -1, 2, -2], &[0, -1, 1]),
            anchor,
            &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"],
            &mut |_| Ok(false),
        )
        .unwrap();
        equal(&json!(selected), &json!(legacy));
    }
}
