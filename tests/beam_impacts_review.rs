// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent reviewer controls; original JVM is development-only.
use audiveris_native::{
    GrayImage,
    beam_impacts::{Evaluator, Impacts, ItemAreas, Limits, Parameters},
    beam_structure::{BeamItem, ItemParameters},
};

#[test]
fn empty_masks_preserve_original_nonfinite_impacts_and_zero_grade() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../evidence/beam-impacts-review/nonfinite-original.json"
    ))
    .unwrap();
    let mut finite_zero = 0;
    let mut nan_grade = 0;
    for row in rows["cases"].as_array().unwrap() {
        let c = &row["case"];
        let name = c["name"].as_str().unwrap();
        let mut image = GrayImage::new(160, 96, vec![255; 160 * 96]).unwrap();
        for rect in c["rectangles"].as_array().unwrap() {
            let x = rect[0].as_u64().unwrap() as usize;
            let y = rect[1].as_u64().unwrap() as usize;
            let w = rect[2].as_u64().unwrap() as usize;
            let h = rect[3].as_u64().unwrap() as usize;
            for yy in y..y + h {
                for xx in x..x + w {
                    image.pixels[yy * 160 + xx] = 0;
                }
            }
        }
        let item = BeamItem {
            median: serde_json::from_value(c["median"].clone()).unwrap(),
            height: c["height"].as_f64().unwrap(),
        };
        let p = ItemParameters::new(1, c["typical_height"].as_f64().unwrap(), false).unwrap();
        let result = Evaluator::new(&image, 1)
            .unwrap()
            .compute_beam_impacts(
                &item,
                true,
                true,
                c["distance_impact"].as_f64().unwrap(),
                &p,
            )
            .unwrap();
        assert!(result.rejection.is_none(), "{name}");
        let impacts = result.impacts.unwrap();
        for (i, expected) in row["impacts"]["values"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            if let Some(value) = expected.as_f64() {
                assert_eq!(
                    impacts.values[i].to_bits(),
                    value.to_bits(),
                    "{name} impact {i}"
                );
            } else {
                assert!(impacts.values[i].is_nan(), "{name} impact {i}");
            }
        }
        if let Some(grade) = row["impacts"]["grade"].as_f64() {
            assert_eq!(impacts.grade, grade, "{name}");
            assert_eq!(grade, 0.);
            finite_zero += 1;
        } else {
            assert!(impacts.grade.is_nan(), "{name}");
            nan_grade += 1;
        }
        assert!(impacts.non_finite, "{name}");
        assert!(!impacts.accepted_as_beam(), "{name}");
        let json = serde_json::to_value(&impacts).unwrap();
        assert_eq!(
            json["grade"].is_null(),
            !impacts.grade.is_finite(),
            "{name}"
        );
        assert!(json["values"][4].is_null(), "{name}");
        assert_eq!(json["non_finite"], true);
    }
    assert_eq!((finite_zero, nan_grade), (6, 2));
}

#[test]
fn review_resource_and_parameter_boundaries() {
    let image = GrayImage::new(100, 100, vec![0; 10_000]).unwrap();
    let item = BeamItem {
        median: [20., 40., 84., 40.],
        height: 8.,
    };
    let p = ItemParameters::new(16, 8., false).unwrap();
    let params = Parameters::new(16).unwrap();
    let areas = ItemAreas::new(
        &item,
        params.belt_margin_dx,
        params.belt_margin_dy,
        params.belt_margin_dy,
    )
    .unwrap();
    let core = areas.core_bounds();
    let belt = areas.belt_bounds();
    let exact = core[2] as usize * core[3] as usize + belt[2] as usize * belt[3] as usize;
    for (budget, success) in [(exact - 1, false), (exact, true)] {
        let e = Evaluator::with_parameters(
            &image,
            params,
            Limits {
                max_mask_cells: budget,
            },
        )
        .unwrap();
        assert_eq!(e.sample(&areas).is_ok(), success);
    }
    for ratio in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        let mut q = params;
        q.min_core_black_ratio = ratio;
        assert!(Evaluator::with_parameters(&image, q, Limits::default()).is_err());
        let mut q = params;
        q.max_belt_black_ratio = ratio;
        assert!(Evaluator::with_parameters(&image, q, Limits::default()).is_err());
    }
    let mut q = params;
    q.min_core_black_ratio = 1.;
    assert!(Evaluator::with_parameters(&image, q, Limits::default()).is_err());
    let mut q = params;
    q.max_belt_black_ratio = 0.;
    assert!(Evaluator::with_parameters(&image, q, Limits::default()).is_err());
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            Evaluator::new(&image, 16)
                .unwrap()
                .compute_beam_impacts(&item, true, true, value, &p)
                .is_err()
        );
        let mut bad = item.clone();
        bad.height = value;
        assert!(ItemAreas::new(&bad, 4, 2, 2).is_err());
    }
    // GradeUtil's relational clamp preserves signed zero.
    let mut values = [1.; 6];
    values[5] = -0.;
    assert_eq!(
        Impacts::new(values).unwrap().values[5].to_bits(),
        (-0.0f64).to_bits()
    );
}

#[test]
fn independent_original_beam_item_integer_masks() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../evidence/beam-impacts-review/beam-geometry-original.json"
    ))
    .unwrap();
    let mut compared = 0;
    let mut points = 0;
    for row in rows.as_array().unwrap() {
        let c = &row["case"];
        let name = c["name"].as_str().unwrap();
        let item = BeamItem {
            median: serde_json::from_value(c["median"].clone()).unwrap(),
            height: c["height"].as_f64().unwrap(),
        };
        let area = ItemAreas::new(
            &item,
            c["dx"].as_u64().unwrap() as usize,
            c["top"].as_u64().unwrap() as usize,
            c["bottom"].as_u64().unwrap() as usize,
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        for (key, bounds) in [("core", area.core_bounds()), ("belt", area.belt_bounds())] {
            let expected_bounds: [i32; 4] =
                serde_json::from_value(row[key]["bounds"].clone()).unwrap();
            assert_eq!(bounds, expected_bounds, "{name} {key} bounds");
            let mask = row[key]["contains"].as_str().unwrap().as_bytes();
            let [x, y, w, h] = expected_bounds;
            assert_eq!(mask.len(), w as usize * h as usize);
            for yy in y..y + h {
                for xx in x..x + w {
                    let actual = if key == "core" {
                        area.core_contains([xx as f64, yy as f64])
                    } else {
                        area.belt_contains([xx as f64, yy as f64])
                    }
                    .unwrap();
                    let expected = mask[(yy - y) as usize * w as usize + (xx - x) as usize] == b'1';
                    assert_eq!(actual, expected, "{name} {key} ({xx},{yy})");
                    points += 1;
                }
            }
        }
        compared += 1;
    }
    println!(
        "beam_item_geometry_review: {compared} independent cases, {points} exact integer point comparisons"
    );
}

#[test]
fn original_cached_bounds_counts_change_core_threshold_and_grade() {
    let controls: serde_json::Value = serde_json::from_str(include_str!(
        "../evidence/beam-impacts-review/branch-original.json"
    ))
    .unwrap();
    for row in controls["cases"].as_array().unwrap() {
        let c = &row["case"];
        let name = c["name"].as_str().unwrap();
        let mut image = GrayImage::new(160, 96, vec![255; 160 * 96]).unwrap();
        for y in 0..8 {
            for x in 20..84 {
                image.pixels[y * 160 + x] = 0;
            }
        }
        let item = BeamItem {
            median: serde_json::from_value(c["median"].clone()).unwrap(),
            height: c["height"].as_f64().unwrap(),
        };
        let p = ItemParameters::new(16, 32., false).unwrap();
        let mut params = Parameters::new(16).unwrap();
        params.min_core_black_ratio = c["parameter_overrides"]["minCoreBlackRatio"]
            .as_f64()
            .unwrap();
        let got = Evaluator::with_parameters(&image, params, Limits::default())
            .unwrap()
            .compute_beam_impacts(&item, true, true, 1., &p)
            .unwrap();
        for (key, counts) in [("core", got.core), ("belt", got.belt)] {
            assert_eq!(
                counts.total as u64,
                row[key]["total"].as_u64().unwrap(),
                "{name} {key} total"
            );
            assert_eq!(
                counts.black as u64,
                row[key]["black"].as_u64().unwrap(),
                "{name} {key} black"
            );
        }
        assert!(got.rejection.is_none(), "{name}");
        let impacts = got.impacts.unwrap();
        let expected: [f64; 6] = serde_json::from_value(row["impacts"]["values"].clone()).unwrap();
        assert_eq!(impacts.values, expected, "{name}");
        assert!(
            (impacts.grade - row["impacts"]["grade"].as_f64().unwrap()).abs() <= 1e-15,
            "{name}"
        );
        assert_eq!(
            impacts.accepted_as_beam(),
            row["accepted"].as_bool().unwrap(),
            "{name}"
        );
    }
}
