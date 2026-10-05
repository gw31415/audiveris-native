// SPDX-License-Identifier: AGPL-3.0-or-later
// Frozen outputs of unmodified official 5.11.0 Java methods, development only.
use audiveris_native::{GrayImage, beam_spots::*};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn fixture(name: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_slice(&fs::read(root.join("fixtures").join(name)).unwrap()).unwrap()
}
fn pixels(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|n| u8::try_from(n.as_u64().unwrap()).unwrap())
        .collect()
}
fn input(case: &Value) -> GrayImage {
    GrayImage::new(
        case["width"].as_u64().unwrap() as usize,
        case["height"].as_u64().unwrap() as usize,
        pixels(&case["input"]),
    )
    .unwrap()
}
fn radius(value: &Value) -> f32 {
    f32::from_bits(value.as_i64().unwrap() as u32)
}

#[test]
fn original_32_filter_inputs_and_224_close_results_match_exact_bytes() {
    let controls = fixture("beam-spots-controls.json");
    assert_eq!(controls["native_outputs_used"], false);
    let cases = controls["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 32);
    let mut closed = 0;
    for (i, case) in cases.iter().enumerate() {
        let image = input(case);
        assert_eq!(
            median_filter(&image).unwrap().pixels,
            pixels(&case["median"]),
            "median case {i}"
        );
        assert_eq!(
            gaussian_filter(&image).unwrap().pixels,
            pixels(&case["gaussian"]),
            "gaussian case {i}"
        );
        for control in case["closes"].as_array().unwrap() {
            let beam = control["beam"].as_f64().unwrap();
            let result = close(&image, beam).unwrap();
            assert_eq!(
                result.pixels,
                pixels(&control["closed"]),
                "close case {i}, beam {beam}"
            );
            assert_eq!(
                threshold(&result, 140).unwrap().pixels,
                pixels(&control["threshold_140"]),
                "threshold case {i}, beam {beam}"
            );
            closed += 1;
        }
    }
    assert_eq!(closed, 224);
}

#[test]
fn original_24_seeded_gray_controls_and_240_direct_radius_closes_match() {
    let controls = fixture("beam-spots-controls-additional.json");
    assert_eq!(controls["native_outputs_used"], false);
    let cases = controls["filters"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    let mut closed = 0;
    for (i, case) in cases.iter().enumerate() {
        let image = input(case);
        assert_eq!(
            median_filter(&image).unwrap().pixels,
            pixels(&case["median"]),
            "median case {i}"
        );
        assert_eq!(
            gaussian_filter(&image).unwrap().pixels,
            pixels(&case["gaussian"]),
            "gaussian case {i}"
        );
        for control in case["closes"].as_array().unwrap() {
            let radius = radius(&control["radius_bits"]);
            assert_eq!(
                close_with_radius(&image, radius).unwrap().pixels,
                pixels(&control["closed"]),
                "close case {i}, radius {radius}"
            );
            closed += 1;
        }
    }
    assert_eq!(closed, 240);
}

#[test]
fn original_gaussian_kernel_bits_and_ten_structure_vectors_match() {
    let controls = fixture("beam-spots-controls-additional.json");
    let expected: Vec<_> = controls["kernel_bits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_u64().unwrap() as u32)
        .collect();
    assert_eq!(gaussian_kernel().map(f32::to_bits).as_slice(), expected);
    let supports = controls["supports"].as_array().unwrap();
    assert_eq!(supports.len(), 10);
    for support in supports {
        let radius = radius(&support["radius_bits"]);
        // Original vector coordinates are row, column, height; native x, y.
        let mut expected: Vec<_> = support["vector"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                assert_eq!(v[2], 255);
                [v[1].as_i64().unwrap() as i32, v[0].as_i64().unwrap() as i32]
            })
            .collect();
        let mut actual = circular_offsets(radius).unwrap();
        expected.sort_unstable();
        actual.sort_unstable();
        assert_eq!(actual, expected, "radius {radius}");
    }
}

#[test]
fn original_120_length_filters_and_24_glyph_sets_match_in_original_order() {
    let controls = fixture("beam-spots-controls-additional.json");
    let cases = controls["binary_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    let mut lengths = 0;
    for (i, case) in cases.iter().enumerate() {
        let image = input(case);
        for length in case["lengths"].as_array().unwrap() {
            let max_stem = length["max_stem"].as_u64().unwrap() as usize;
            assert_eq!(
                filter_horizontal(&image, max_stem).unwrap().pixels,
                pixels(&length["filtered"]),
                "horizontal case {i}, max_stem {max_stem}"
            );
            lengths += 1;
        }
        let glyphs = extract_spots(&image).unwrap();
        let actual: Vec<_> = glyphs
            .iter()
            .map(|g| {
                json!({
                    "left":g.left,"top":g.top,"width":g.width,"height":g.height,
                    "runs":g.runs.iter().map(|r| [r.position,r.start,r.length]).collect::<Vec<_>>()
                })
            })
            .collect();
        assert_eq!(Value::Array(actual), case["glyphs"], "glyphs case {i}");
    }
    assert_eq!(lengths, 120);
}

#[test]
fn original_roi_clipping_matches_and_overflowing_java_case_is_rejected() {
    let controls = fixture("beam-spots-controls-additional.json");
    let image = GrayImage::new(4, 3, vec![0; 12]).unwrap();
    let cases = controls["rois"].as_array().unwrap();
    assert_eq!(cases.len(), 14);
    for (i, case) in cases.iter().enumerate() {
        let r = case["rect"].as_array().unwrap();
        let rect = HeaderRect {
            left: r[0].as_i64().unwrap() as i32,
            top: r[1].as_i64().unwrap() as i32,
            right: r[2].as_i64().unwrap() as i32,
            bottom: r[3].as_i64().unwrap() as i32,
        };
        let actual = erase_headers(&image, &[rect]);
        if rect.right == i32::MAX || rect.bottom == i32::MAX {
            // Native explicitly rejects signed endpoint overflow. The original
            // throws for overflowing x but silently no-ops for overflowing y.
            assert_eq!(
                actual.unwrap_err(),
                "invalid_beam_header_rectangle",
                "ROI {i}"
            );
        } else if case["original_exception"].is_null() {
            assert_eq!(actual.unwrap().pixels, pixels(&case["output"]), "ROI {i}");
        } else {
            assert_eq!(
                case["original_exception"],
                "java.lang.ArrayIndexOutOfBoundsException"
            );
            assert!(actual.is_err(), "reject unsafe original ROI {i}");
        }
    }
}

#[test]
fn original_42_staff_line_samples_match_header_rectangles_with_half_even_rounding() {
    let controls = fixture("beam-spots-controls-additional.json");
    let image = GrayImage::new(160, 256, vec![255; 160 * 256]).unwrap();
    let cases = controls["staff_lines"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    let mut count = 0;
    for (case_id, case) in cases.iter().enumerate() {
        let points = case["points"].as_array().unwrap();
        let lines: Vec<_> = (0..5).map(|index| json!({"points":points.iter().map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap() + 16. * index as f64]).collect::<Vec<_>>()})).collect();
        let grid = json!({"staffs":[{"id":1,"system_id":1,"lines":lines}],"systems":[{"id":1,"staff_ids":[1],"area":{"x_range":[0,160]}}]});
        for sample in case["samples"].as_array().unwrap() {
            let stop = sample["stop"].as_i64().unwrap();
            let headers = json!({"headers_complete":true,"staffs":[{"staff_id":1,"system_id":1,"stop":stop}]});
            let (rects, _) =
                audiveris_native::beam_preparation::header_rectangles(&image, &grid, &headers, 16)
                    .unwrap();
            assert_eq!(
                rects,
                vec![HeaderRect {
                    left: 0,
                    right: stop as i32,
                    top: sample["top"].as_i64().unwrap() as i32 - 32,
                    bottom: sample["bottom"].as_i64().unwrap() as i32 + 32
                }],
                "curve case {case_id}, stop {stop}"
            );
            count += 1;
        }
    }
    assert_eq!(count, 42);
}
