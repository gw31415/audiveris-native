// SPDX-License-Identifier: AGPL-3.0-or-later
// Direct original BeamStructure calls are compared here. Replayed builder
// prechecks in the full-page observers are deliberately not treated as original
// production dispatch decisions or tested as if they were.
use audiveris_native::beam_structure::{BeamStructure, ItemParameters, VerticalSide};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};

#[derive(Default, Debug)]
struct Comparison {
    numbers: usize,
    maximum_absolute_error: f64,
}
impl Comparison {
    fn check(&mut self, actual: &Value, expected: &Value, path: &str) {
        match (actual, expected) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                    assert_eq!(a, b, "{path}");
                } else {
                    let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
                    let error = (a - b).abs();
                    assert!(
                        a.is_finite() && b.is_finite(),
                        "{path}: nonfinite {a} / {b}"
                    );
                    self.numbers += 1;
                    if error > self.maximum_absolute_error {
                        self.maximum_absolute_error = error;
                        if error > 1e-10 {
                            println!(
                                "beam_structure_delta: {path} actual={a:.17} original={b:.17} absolute_error={error:.12e}"
                            );
                        }
                    }
                    assert!(
                        error <= 1e-8,
                        "{path}: actual={a:.16} original={b:.16} absolute_error={error:.6e}"
                    );
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    self.check(a, b, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                for (key, a) in a {
                    self.check(
                        a,
                        b.get(key).unwrap_or_else(|| panic!("{path}.{key} missing")),
                        &format!("{path}.{key}"),
                    );
                }
            }
            _ => assert_eq!(actual, expected, "{path}"),
        }
    }
}
fn glyph(value: &Value) -> GlyphPixels {
    let b = &value["bounds"];
    assert_eq!(value["orientation"], "VERTICAL");
    GlyphPixels {
        left: b[0].as_i64().unwrap() as i32,
        top: b[1].as_i64().unwrap() as i32,
        width: b[2].as_u64().unwrap() as usize,
        height: b[3].as_u64().unwrap() as usize,
        orientation: Orientation::Vertical,
        runs: value["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Run {
                position: v[0].as_u64().unwrap() as usize,
                start: v[1].as_u64().unwrap() as usize,
                length: v[2].as_u64().unwrap() as usize,
            })
            .collect(),
    }
}
fn check_case(original: &Value, p: ItemParameters, label: &str, stats: &mut Comparison) {
    let g = glyph(&original["glyph"]);
    let mut s = BeamStructure::new(&g, p).unwrap();
    let distance = s.compute_lines().unwrap_or_else(|e| panic!("{label}: {e}"));
    stats.check(
        &json!(distance),
        &original["global_distance"],
        &format!("{label}.global_distance"),
    );
    stats.check(
        &json!(s.sections()),
        &original["sections"],
        &format!("{label}.sections"),
    );
    stats.check(
        &json!(s.lines()),
        &original["after_compute_lines"],
        &format!("{label}.after_compute_lines"),
    );
    if original.get("width").is_some() {
        stats.check(
            &json!(s.width()),
            &original["width"],
            &format!("{label}.width"),
        );
        stats.check(
            &json!(s.compare_slopes()),
            &original["slope_difference"],
            &format!("{label}.slope_difference"),
        );
    }
    if original.get("after_adjust_sides").is_none() {
        return;
    }
    s.adjust_sides();
    stats.check(
        &json!(s.lines()),
        &original["after_adjust_sides"],
        &format!("{label}.after_adjust_sides"),
    );
    s.extend_middle_lines();
    stats.check(
        &json!(s.lines()),
        &original["after_extend_middle"],
        &format!("{label}.after_extend_middle"),
    );
    s.split_lines();
    stats.check(
        &json!(s.lines()),
        &original["after_split_lines"],
        &format!("{label}.after_split_lines"),
    );
    let jitters: Vec<Value> = s
        .lines()
        .iter()
        .map(|l| {
            match (
                s.compute_jitter(l, VerticalSide::Top),
                s.compute_jitter(l, VerticalSide::Bottom),
            ) {
                (Ok(top), Ok(bottom)) => json!({"top":top,"bottom":bottom}),
                (Err(e), _) | (_, Err(e)) => {
                    assert_eq!(e, "undefined_beam_structure_line", "{label}");
                    json!({"exception":"UndefinedLineException"})
                }
            }
        })
        .collect();
    stats.check(
        &json!(jitters),
        &original["jitters"],
        &format!("{label}.jitters"),
    );
}
#[test]
fn original_geometry_invocations_in_thirteen_full_image_observers() {
    let mut paths = std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/official-beam-prolog"
    ))
    .unwrap()
    .map(|p| p.unwrap().path())
    .filter(|p| p.extension().is_some_and(|e| e == "json"))
    .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 13);
    let mut stats = Comparison::default();
    let mut invoked = 0;
    let mut precheck_only = 0;
    for path in paths {
        let original: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let params = ItemParameters::new(
            original["scale"]["interline"].as_u64().unwrap() as usize,
            original["scale"]["beam_thickness"].as_f64().unwrap(),
            false,
        )
        .unwrap();
        stats.check(
            &json!(params),
            &original["beam_parameters"]["standard"],
            "standard_parameters",
        );
        for (index, diag) in original["replayed_beam_structure_diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            if diag["original_structure_called"] != true {
                precheck_only += 1;
                continue;
            }
            invoked += 1;
            check_case(
                diag,
                params,
                &format!("{}:{index}", path.display()),
                &mut stats,
            );
        }
    }
    assert_eq!(invoked, 144);
    assert_eq!(precheck_only, 339);
    println!(
        "beam_structure_full_image_geometry: images=13 original_geometry_calls={invoked} replayed_prechecks_skipped={precheck_only} floating_values={} max_abs_error={:.12e} tolerance=1e-8",
        stats.numbers, stats.maximum_absolute_error
    );
}
#[test]
fn direct_original_synthetic_geometry_controls() {
    let controls: Value =
        serde_json::from_str(include_str!("../fixtures/beam-structure-controls.json")).unwrap();
    assert_eq!(
        controls["mode"],
        "direct_original_BeamStructure_methods_no_builder_decision_replay"
    );
    let cases = controls["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 279);
    let mut stats = Comparison::default();
    let (mut nulls, mut splits, mut multi, mut items) = (0, 0, 0, 0);
    for original in cases {
        assert!(original.get("exception").is_none());
        let case = &original["case"];
        let p = ItemParameters::new(
            case["interline"].as_u64().unwrap_or(16) as usize,
            case["typical_height"].as_f64().unwrap_or(8.0),
            false,
        )
        .unwrap();
        let mut p = json!(p);
        if let Some(overrides) = case["parameter_overrides"].as_object() {
            for (k, v) in overrides {
                p[k] = v.clone();
            }
        }
        let mut expected = original.clone();
        expected["glyph"] = case["glyph"].clone();
        check_case(
            &expected,
            serde_json::from_value(p).unwrap(),
            case["name"].as_str().unwrap(),
            &mut stats,
        );
        nulls += usize::from(original["global_distance"].is_null());
        let before = original["after_compute_lines"].as_array().unwrap();
        let after = original["after_split_lines"].as_array();
        splits += usize::from(after.is_some_and(|v| v.len() > before.len()));
        multi += usize::from(before.len() > 1);
        items += usize::from(
            before
                .iter()
                .any(|v| v["items"].as_array().unwrap().len() > 1),
        );
    }
    assert_eq!((nulls, splits, multi, items), (111, 35, 43, 13));
    println!(
        "beam_structure_direct_controls: cases=279 null_borders={nulls} stuck_splits={splits} multiple_lines={multi} multiple_items={items} floating_values={} max_abs_error={:.12e} tolerance=1e-8",
        stats.numbers, stats.maximum_absolute_error
    );
}
