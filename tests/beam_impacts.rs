// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_impacts::{
        Evaluator, Impacts, ItemAreas, Limits, MIN_BEAM_GRADE, MIN_HOOK_GRADE, Parameters,
    },
    beam_structure::{BeamItem, ItemParameters},
};
use serde_json::Value;
fn raster(c: &Value) -> GrayImage {
    let width = c["image_width"].as_u64().unwrap_or(160) as usize;
    let height = c["image_height"].as_u64().unwrap_or(96) as usize;
    let mut pixels = vec![255; width * height];
    for r in c["rectangles"].as_array().unwrap() {
        let x = r[0].as_i64().unwrap();
        let y = r[1].as_i64().unwrap();
        let w = r[2].as_i64().unwrap();
        let h = r[3].as_i64().unwrap();
        for yy in y..y + h {
            for xx in x..x + w {
                if xx >= 0 && yy >= 0 && xx < (width as i64) && yy < (height as i64) {
                    pixels[yy as usize * width + xx as usize] = r[4].as_u64().unwrap_or(0) as u8;
                }
            }
        }
    }
    if let Some(pp) = c["pixels"].as_array() {
        for p in pp {
            pixels[p[1].as_u64().unwrap() as usize * width + p[0].as_u64().unwrap() as usize] =
                p[2].as_u64().unwrap() as u8;
        }
    }
    GrayImage::new(width, height, pixels).unwrap()
}
#[test]
fn original_item_masks_and_impacts() {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/beam-impacts-controls.json")).unwrap();
    assert_eq!(MIN_BEAM_GRADE, fixture["min_beam_grade"].as_f64().unwrap());
    assert_eq!(MIN_HOOK_GRADE, fixture["min_hook_grade"].as_f64().unwrap());
    let mut count = 0;
    let mut points = 0;
    let mut grades = 0;
    let mut max_error: f64 = 0.0;
    for original in fixture["cases"].as_array().unwrap() {
        let c = &original["case"];
        let name = c["name"].as_str().unwrap();
        let il = c["interline"].as_u64().unwrap() as usize;
        let item = BeamItem {
            median: std::array::from_fn(|i| c["median"][i].as_f64().unwrap()),
            height: c["height"].as_f64().unwrap(),
        };
        let p = ItemParameters::new(
            il,
            c["typical_height"].as_f64().unwrap(),
            c["small"].as_bool().unwrap_or(false),
        )
        .unwrap();
        let mut params = Parameters::new(il).unwrap();
        if let Some(r) = c["parameter_overrides"]["maxBeltBlackRatio"].as_f64() {
            params.max_belt_black_ratio = r;
        }
        if let Some(r) = c["parameter_overrides"]["minCoreBlackRatio"].as_f64() {
            params.min_core_black_ratio = r;
        }
        assert_eq!(
            [params.belt_margin_dx, params.belt_margin_dy],
            [
                original["belt_margins"][0].as_u64().unwrap() as usize,
                original["belt_margins"][1].as_u64().unwrap() as usize
            ],
            "{name}"
        );
        let hook = c["hook"].as_bool().unwrap_or(false);
        let top = hook || c["top"].as_bool().unwrap_or(true);
        let bottom = hook || c["bottom"].as_bool().unwrap_or(true);
        let areas = ItemAreas::new(
            &item,
            params.belt_margin_dx,
            if top { params.belt_margin_dy } else { 0 },
            if bottom { params.belt_margin_dy } else { 0 },
        )
        .unwrap();
        for key in ["core", "belt"] {
            let b = if key == "core" {
                areas.core_bounds()
            } else {
                areas.belt_bounds()
            };
            let mut actual = vec![];
            for y in b[1]..b[1] + b[3] {
                for x in b[0]..b[0] + b[2] {
                    let inside = if key == "core" {
                        areas.core_contains([x as f64, y as f64])
                    } else {
                        areas.belt_contains([x as f64, y as f64])
                    }
                    .unwrap();
                    if inside {
                        actual.push([x, y]);
                    }
                }
            }
            let expected: Vec<[i32; 2]> =
                serde_json::from_value(original[key]["points"].clone()).unwrap();
            assert_eq!(actual, expected, "{name} {key} point membership");
            points += actual.len();
        }
        let image = raster(c);
        let evaluator = Evaluator::with_parameters(&image, params, Limits::default()).unwrap();
        let d = c["distance_impact"].as_f64().unwrap_or(1.0);
        let got = if hook {
            evaluator.compute_hook_impacts(&item, d, &p)
        } else {
            evaluator.compute_beam_impacts(&item, top, bottom, d, &p)
        }
        .unwrap();
        assert_eq!(
            got.width as i64,
            original["width"].as_i64().unwrap(),
            "{name}"
        );
        for (key, counts) in [("core", got.core), ("belt", got.belt)] {
            assert_eq!(
                counts.total as u64,
                original[key]["total"].as_u64().unwrap(),
                "{name} {key} total"
            );
            assert_eq!(
                counts.black as u64,
                original[key]["black"].as_u64().unwrap(),
                "{name} {key} black"
            );
        }
        assert_eq!(
            got.impacts.is_some(),
            !original["impacts"].is_null(),
            "{name} impacts presence"
        );
        if let Some(impacts) = got.impacts {
            for (i, actual) in impacts.values.into_iter().enumerate() {
                let expected = original["impacts"]["values"][i]
                    .as_f64()
                    .unwrap_or(f64::NAN);
                if expected.is_nan() {
                    assert!(actual.is_nan(), "{name} NaN impact {i}");
                    continue;
                }
                assert_eq!(actual, expected, "{name} impact {i}");
            }
            let original_grade = original["impacts"]["grade"].as_f64().unwrap_or(f64::NAN);
            let error = if original_grade.is_nan() {
                assert!(impacts.grade.is_nan(), "{name} NaN grade");
                0.0
            } else {
                (impacts.grade - original_grade).abs()
            };
            max_error = max_error.max(error);
            assert!(error <= 1e-15, "{name} grade error {error}");
            assert_eq!(
                impacts.accepted_as_beam(),
                original["accepted"].as_bool().unwrap(),
                "{name} acceptance"
            );
            assert_eq!(
                impacts.non_finite,
                !impacts.grade.is_finite(),
                "{name} nonfinite flag"
            );
            if impacts.non_finite {
                let serialized = serde_json::to_value(&impacts).unwrap();
                assert_eq!(serialized["non_finite"], true);
                assert!(serialized["grade"].is_null());
            }
            grades += 1;
        }
        count += 1;
    }
    println!(
        "beam_impacts_parity: {count} cases, {points} mask points, {grades} graded; max_grade_error={max_error:.17e}"
    );
}
#[test]
fn validates_bounds_and_limits_without_mutating_image() {
    let image = GrayImage::new(100, 100, vec![0; 10_000]).unwrap();
    let item = BeamItem {
        median: [10., 20., 80., 20.],
        height: 8.,
    };
    let p = ItemParameters::new(16, 8., false).unwrap();
    let e = Evaluator::with_parameters(
        &image,
        Parameters::new(16).unwrap(),
        Limits { max_mask_cells: 1 },
    )
    .unwrap();
    assert_eq!(
        e.compute_beam_impacts(&item, true, true, 1., &p)
            .unwrap_err(),
        "beam_mask_budget_exceeded"
    );
    assert!(Parameters::new(0).is_err());
    assert!(
        ItemAreas::new(
            &BeamItem {
                median: [1., 0., 1., 4.],
                height: 8.
            },
            4,
            2,
            2
        )
        .is_err()
    );
    assert!(
        ItemAreas::new(
            &BeamItem {
                median: [0., f64::NAN, 1., 4.],
                height: 8.
            },
            4,
            2,
            2
        )
        .is_err()
    );
    assert!(Impacts::new([f64::NAN; 6]).is_err());
    assert!(
        Evaluator::new(
            &GrayImage {
                width: 2,
                height: 2,
                pixels: vec![0]
            },
            16
        )
        .is_err()
    );
    let mut invalid = p;
    invalid.min_beam_width_high = invalid.min_beam_width_low;
    assert!(
        Evaluator::new(&image, 16)
            .unwrap()
            .compute_beam_impacts(&item, true, true, 1., &invalid)
            .is_err()
    );
    assert!(image.pixels.iter().all(|v| *v == 0));
}
#[test]
fn grade_gate_equality_and_clamping_are_separate_from_method_acceptance() {
    let full = Impacts::new([2.; 6]).unwrap();
    assert_eq!(full.values, [1.; 6]);
    assert_eq!(full.grade, 0.8);
    let zero = Impacts::new([0., 1., 1., 1., 1., 1.]).unwrap();
    assert_eq!(zero.grade, 0.);
    assert!(!zero.accepted_as_beam());
    let boundary = Impacts {
        values: [0.; 6],
        grade: MIN_BEAM_GRADE,
        non_finite: false,
    };
    assert!(boundary.accepted_as_beam());
    assert!(boundary.accepted_as_hook());
    let below = Impacts {
        grade: MIN_BEAM_GRADE.next_down(),
        ..boundary
    };
    assert!(!below.accepted_as_beam());
}
