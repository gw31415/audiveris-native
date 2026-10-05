// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, curves::Curve, head_seed_search, template_probes, vertical};
use audiveris_template_bank::Bank;
use serde_json::Value;
use sha2::{Digest, Sha256};
fn golden() -> Value {
    serde_json::from_str(include_str!("../fixtures/official-head-seed-search.json")).unwrap()
}
fn compare(actual: &[Value], expected: &Value) {
    let e = expected.as_array().unwrap();
    assert_eq!(actual.len(), e.len());
    for (a, b) in actual.iter().zip(e) {
        for key in ["shape", "source_shape", "anchor", "x", "y", "bounds"] {
            assert_eq!(a[key], b[key], "{key}");
        }
        for key in ["score", "grade"] {
            assert!(
                (a[key].as_f64().unwrap() - b[key].as_f64().unwrap()).abs() < 1e-12,
                "{key}: {a} != {b}"
            );
        }
    }
}
#[test]
fn independent_original_laura_seed_search_and_decision_controls() {
    let g = golden();
    assert_eq!(g["native_outputs_used"], false);
    let image =
        GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap();
    let binary = vertical::binarize(&image, |_, _, _| {});
    let mask = GrayImage::decode(include_bytes!("../fixtures/official-heads-mask.png")).unwrap();
    let table = template_probes::distances(&binary, Some(&mask)).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&mask.pixels)),
        g["mask_sha256"].as_str().unwrap()
    );
    let mut digest = Sha256::new();
    for v in table.values() {
        digest.update((v as i16).to_be_bytes());
    }
    assert_eq!(
        format!("{:x}", digest.finalize()),
        g["distance_sha256_be_i16"].as_str().unwrap()
    );
    let bank = Bank::load_builtin().unwrap();
    let mut positives = 0;
    let mut rejected = 0;
    for c in g["controls"].as_array().unwrap() {
        let xo: Vec<i32> = serde_json::from_value(c["x_offsets"].clone()).unwrap();
        let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone()).unwrap();
        let a = head_seed_search::search(
            &table,
            &bank,
            [
                c["x0"].as_i64().unwrap() as i32,
                c["y0"].as_i64().unwrap() as i32,
            ],
            (&xo, &yo),
            c["anchor"].as_str().unwrap(),
        )
        .unwrap();
        compare(&a, &c["matches"]);
        if a.is_empty() {
            rejected += 1
        } else {
            positives += 1
        }
    }
    let mut pairs = 0;
    let mut matches = 0;
    for s in g["systems"].as_array().unwrap() {
        for f in s["staffs"].as_array().unwrap() {
            for c in f["scanners"].as_array().unwrap() {
                let xo: Vec<i32> = serde_json::from_value(c["x_offsets"].clone()).unwrap();
                let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone()).unwrap();
                assert_eq!(head_seed_search::x_offsets(5).unwrap(), xo);
                let dir = c["dir"].as_i64().unwrap() as i32;
                let pitch = c["pitch"].as_i64().unwrap() as i32;
                assert_eq!(
                    head_seed_search::y_offsets(11, dir, pitch.abs() == 5).unwrap(),
                    yo
                );
                for z in c["seeds"].as_array().unwrap() {
                    let mut a = Vec::new();
                    for anchor in ["LEFT_STEM", "RIGHT_STEM"] {
                        a.extend(
                            head_seed_search::search(
                                &table,
                                &bank,
                                [
                                    z["x0"].as_i64().unwrap() as i32,
                                    z["y0"].as_i64().unwrap() as i32,
                                ],
                                (&xo, &yo),
                                anchor,
                            )
                            .unwrap(),
                        );
                    }
                    compare(&a, &z["matches"]);
                    pairs += 1;
                    matches += a.len();
                }
            }
        }
    }
    assert_eq!(pairs, 1107);
    assert_eq!(matches, 388);
    assert!(positives > 0 && rejected > 0);
    println!(
        "48 controls: {positives} accepted, {rejected} rejected; {pairs} original seed/pitch pairs, {matches} raw proposals"
    );
}
#[test]
fn original_offset_order_and_strict_curved_seed_slice() {
    assert_eq!(
        head_seed_search::x_offsets(5).unwrap(),
        vec![0, 1, -1, 2, -2]
    );
    assert_eq!(
        head_seed_search::x_offsets(4).unwrap(),
        vec![0, 1, -1, 2, -2]
    );
    for dir in [-1, 1] {
        assert_eq!(
            head_seed_search::y_offsets(11, dir, true).unwrap(),
            vec![0, dir, -dir]
        );
        assert_eq!(
            head_seed_search::y_offsets(11, dir, false).unwrap(),
            vec![0, -1]
        );
    }
    let line = Curve::interpolate(vec![[0., 10.], [20., 10.]]).unwrap();
    assert!(!head_seed_search::intersects_seed(&line, [5., 13., 2., 2.], -2., 2.).unwrap());
    assert!(head_seed_search::intersects_seed(&line, [5., 12.999, 2., 2.], -2., 2.).unwrap());
    assert!(!head_seed_search::intersects_seed(&line, [20., 9., 2., 2.], -2., 2.).unwrap());
    assert!(head_seed_search::intersects_seed(&line, [19.99, 9., 2., 2.], -2., 2.).unwrap());
    assert!(head_seed_search::x_offsets(0).is_err());
    assert!(head_seed_search::x_offsets(129).is_err());
    assert!(head_seed_search::y_offsets(11, 0, true).is_err());
    let page = GrayImage::new(10, 10, vec![255; 100]).unwrap();
    let table = template_probes::distances(&page, None).unwrap();
    let bank = Bank::load_builtin().unwrap();
    assert!(head_seed_search::search(&table, &bank, [0, 0], (&[1], &[0]), "LEFT_STEM").is_err());
    assert!(head_seed_search::search(&table, &bank, [0, 0], (&[0], &[0]), "UNKNOWN").is_err());
    assert!(
        head_seed_search::prepare(
            &page,
            &serde_json::json!({}),
            &serde_json::json!({}),
            &serde_json::json!({})
        )
        .is_err()
    );
}
