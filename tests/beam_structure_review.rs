// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent review: seeded direct original-JAR geometry and minimized
// strict-branch regressions. These calls are not production builder decisions.
use audiveris_native::beam_structure::{BeamStructure, ItemParameters, Limits, VerticalSide};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};

fn params() -> ItemParameters {
    ItemParameters::new(16, 8.0, false).unwrap()
}
fn from_case(case: &Value) -> GlyphPixels {
    let b = &case["glyph"]["bounds"];
    GlyphPixels {
        left: b[0].as_i64().unwrap() as i32,
        top: b[1].as_i64().unwrap() as i32,
        width: b[2].as_u64().unwrap() as usize,
        height: b[3].as_u64().unwrap() as usize,
        orientation: Orientation::Vertical,
        runs: case["glyph"]["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| Run {
                position: r[0].as_u64().unwrap() as usize,
                start: r[1].as_u64().unwrap() as usize,
                length: r[2].as_u64().unwrap() as usize,
            })
            .collect(),
    }
}
fn check(a: &Value, b: &Value, path: &str, max_error: &mut f64) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                assert_eq!(a, b, "{path}");
            } else {
                let a = a.as_f64().unwrap();
                let b = b.as_f64().unwrap();
                assert!(a.is_finite() && b.is_finite(), "{path}");
                let delta = (a - b).abs();
                *max_error = max_error.max(delta);
                assert!(delta <= 1e-8, "{path}: {a:.17} != {b:.17}, delta={delta}");
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: array length/branch mismatch");
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                check(a, b, &format!("{path}[{i}]"), max_error);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: field count mismatch");
            for (key, a) in a {
                check(a, &b[key], &format!("{path}.{key}"), max_error);
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}
#[test]
fn seeded_direct_original_geometry_and_minimized_split_branches() {
    let original: Value = serde_json::from_str(include_str!(
        "../fixtures/beam-structure-review-controls.json"
    ))
    .unwrap();
    let cases = original["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1002);
    let mut max_error: f64 = 0.0;
    for expected in cases {
        assert!(expected.get("exception").is_none());
        let label = expected["case"]["name"].as_str().unwrap();
        let mut structure = BeamStructure::new(&from_case(&expected["case"]), params()).unwrap();
        let distance = structure.compute_lines().unwrap();
        let mut actual = json!({
            "case": expected["case"],
            "global_distance": distance,
            "sections": structure.sections(),
            "after_compute_lines": structure.lines(),
        });
        if distance.is_some() {
            actual["width"] = json!(structure.width());
            actual["slope_difference"] = json!(structure.compare_slopes());
            structure.adjust_sides();
            actual["after_adjust_sides"] = json!(structure.lines());
            structure.extend_middle_lines();
            actual["after_extend_middle"] = json!(structure.lines());
            structure.split_lines();
            actual["after_split_lines"] = json!(structure.lines());
            let jitters: Vec<Value> = structure
                .lines()
                .iter()
                .map(|line| {
                    match (
                        structure.compute_jitter(line, VerticalSide::Top),
                        structure.compute_jitter(line, VerticalSide::Bottom),
                    ) {
                        (Ok(top), Ok(bottom)) => json!({"top": top, "bottom": bottom}),
                        (Err(e), _) | (_, Err(e)) => {
                            assert_eq!(e, "undefined_beam_structure_line", "{label}");
                            json!({"exception": "UndefinedLineException"})
                        }
                    }
                })
                .collect();
            actual["jitters"] = json!(jitters);
        }
        check(&actual, expected, label, &mut max_error);
    }
    println!(
        "independent_beam_geometry: cases=1002 max_abs_error={max_error:.12e} tolerance=1e-8 all_branches_exact"
    );
}
fn rectangle(width: usize, height: usize) -> GlyphPixels {
    GlyphPixels {
        left: 0,
        top: 0,
        width,
        height,
        orientation: Orientation::Vertical,
        runs: (0..width)
            .map(|position| Run {
                position,
                start: 0,
                length: height,
            })
            .collect(),
    }
}
#[test]
fn exact_page_and_pixel_boundaries() {
    let mut glyph = rectangle(32, 8);
    glyph.left = 16352;
    glyph.top = 16376;
    let mut accepted = BeamStructure::new(&glyph, params()).unwrap();
    assert!(accepted.compute_lines().unwrap().is_some());
    glyph.left += 1;
    assert_eq!(
        BeamStructure::new(&glyph, params()).unwrap_err(),
        "unsupported_beam_structure_coordinates"
    );
    let exact = rectangle(2000, 2000);
    assert!(BeamStructure::new(&exact, params()).is_ok());
    let beyond = rectangle(2001, 2000);
    assert!(BeamStructure::new(&beyond, params()).is_err());
}
#[test]
fn exact_work_budget_and_transactional_rejection() {
    let glyph = rectangle(32, 8);
    let limits = Limits {
        max_work: 62,
        ..Limits::default()
    };
    let mut exact = BeamStructure::with_limits(&glyph, params(), limits).unwrap();
    assert!(exact.compute_lines().unwrap().is_some());
    assert_eq!(
        BeamStructure::with_limits(
            &glyph,
            params(),
            Limits {
                max_work: 61,
                ..limits
            }
        )
        .unwrap_err(),
        "unsupported_beam_structure_work_budget"
    );
    // One core section plus isolated one-run speckles: construction requires
    // 24 adjacency comparisons, but per-border section scans require 42.
    let mut sparse = rectangle(53, 8);
    sparse.runs = (0..13)
        .chain((14..53).step_by(2))
        .map(|position| Run {
            position,
            start: 0,
            length: 8,
        })
        .collect();
    let mut limited = BeamStructure::with_limits(
        &sparse,
        params(),
        Limits {
            max_work: 24,
            ..Limits::default()
        },
    )
    .unwrap();
    let sections = json!(limited.sections());
    for _ in 0..2 {
        assert_eq!(
            limited.compute_lines().unwrap_err(),
            "unsupported_beam_structure_work_budget"
        );
        assert!(limited.lines().is_empty());
        assert_eq!(json!(limited.sections()), sections);
    }
}
