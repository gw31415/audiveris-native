// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_merge::{MergeInput, merge_of},
    beam_structure::BeamItem,
};

#[test]
fn direct_original_merge_candidates_match_and_inputs_are_unchanged() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-merge.json")).unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 215);
    let mut accepted = 0;
    let mut small = 0;
    let mut max_grade_error: f64 = 0.;
    for (i, case) in cases.iter().enumerate() {
        let c = &case["input"];
        let w = c["width"].as_u64().unwrap() as usize;
        let h = c["height"].as_u64().unwrap() as usize;
        let mut pixels = vec![255; w * h];
        for p in c["black_pixels"].as_array().unwrap() {
            pixels[p[1].as_u64().unwrap() as usize * w + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(w, h, pixels).unwrap();
        let input = |key: &str| MergeInput {
            item: BeamItem {
                median: std::array::from_fn(|j| c[key]["median"][j].as_f64().unwrap()),
                height: c[key]["height"].as_f64().unwrap(),
            },
            distance_impact: c[key]["distance_impact"].as_f64().unwrap(),
        };
        let a = input("first");
        let b = input("second");
        let before = (a.clone(), b.clone(), image.pixels.clone());
        let result = merge_of(
            &image,
            &a,
            &b,
            c["interline"].as_u64().unwrap() as usize,
            c["standard_height"].as_f64().unwrap(),
            c["small_height"].as_f64(),
        )
        .unwrap();
        assert_eq!((a, b, image.pixels), before, "case {i} input mutation");
        let expected = &case["result"];
        assert_eq!(result.is_some(), !expected.is_null(), "case {i} acceptance");
        if let Some(got) = result {
            accepted += 1;
            small += usize::from(got.is_small);
            let median: [f64; 4] = std::array::from_fn(|j| expected["median"][j].as_f64().unwrap());
            assert_eq!(got.item.median, median, "case {i} median");
            assert_eq!(
                got.item.height,
                expected["height"].as_f64().unwrap(),
                "case {i} height"
            );
            assert_eq!(
                got.is_small,
                expected["is_small"].as_bool().unwrap(),
                "case {i} type"
            );
            let impacts: [f64; 6] =
                std::array::from_fn(|j| expected["impacts"][j].as_f64().unwrap());
            assert_eq!(got.impacts.values, impacts, "case {i} impact values");
            let error = (got.impacts.grade - expected["grade"].as_f64().unwrap()).abs();
            max_grade_error = max_grade_error.max(error);
            assert!(error <= 1e-12, "case {i} grade error {error}");
        }
    }
    assert_eq!(accepted, fixture["accepted"].as_u64().unwrap() as usize);
    assert_eq!(small, fixture["small_accepted"].as_u64().unwrap() as usize);
    assert!(accepted > 0 && accepted < cases.len() && small > 0);
    println!(
        "merge cases={} accepted={accepted} rejected={} small={small} max_grade_abs_error={max_grade_error:e}",
        cases.len(),
        cases.len() - accepted
    );
}

#[test]
fn invalid_merge_inputs_fail_without_mutating_the_raster() {
    let image = GrayImage::new(64, 64, vec![255; 64 * 64]).unwrap();
    let a = MergeInput {
        item: BeamItem {
            median: [10., 20., 30., 20.],
            height: 8.,
        },
        distance_impact: 1.,
    };
    let original = image.pixels.clone();
    for bad in [f64::NAN, f64::INFINITY, -1.] {
        let mut b = a.clone();
        b.item.height = bad;
        assert!(merge_of(&image, &a, &b, 16, 8., Some(4.)).is_err());
    }
    let mut b = a.clone();
    b.item.median[2] = b.item.median[0];
    assert!(merge_of(&image, &a, &b, 16, 8., None).is_err());
    b = a.clone();
    b.distance_impact = f64::NAN;
    assert!(merge_of(&image, &b, &a, 16, 8., None).is_err());
    assert_eq!(image.pixels, original);
}
