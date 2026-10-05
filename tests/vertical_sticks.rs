// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::vertical_sticks::{SeedSection, retrieve};
use serde_json::Value;
fn compare(a: &Value, b: &Value, path: &str) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                compare(&a[k], v, &format!("{path}.{k}"));
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len(), "{path}");
            for (i, (x, y)) in a.as_array().unwrap().iter().zip(v).enumerate() {
                compare(x, y, &format!("{path}[{i}]"));
            }
        }
        Value::Number(_) => {
            if b.as_i64().is_some() {
                assert_eq!(a, b, "{path}");
            } else {
                assert!(
                    (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-8,
                    "{path}: {a} != {b}"
                );
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}
#[test]
fn original_section_inputs_produce_identical_stick_members_and_geometry() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-stem-seeds"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 12);
    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap();
        println!("sticks {name}");
        let original: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let il = original["interline"].as_f64().unwrap();
        let max = original["stem_scale"]["max"].as_i64().unwrap();
        for system in original["systems"].as_array().unwrap() {
            let left = system["left"].as_i64().unwrap();
            let right = system["right"].as_i64().unwrap();
            let parse = |key: &str, wanted: bool| -> Vec<SeedSection> {
                serde_json::from_value::<Vec<SeedSection>>(system[key].clone())
                    .unwrap()
                    .into_iter()
                    .filter(|s| {
                        let b = s.bounds();
                        let center = b[0] + b[2] / 2;
                        center > left && center < right && (wanted || b[2] == 1)
                    })
                    .collect()
            };
            let v = parse("vertical_sections", true);
            let h = parse("horizontal_sections", false);
            let actual = retrieve(&v, &h, max, (1.5 * il).round_ties_even() as i64).unwrap();
            let expected = system["candidates"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{name}");
            for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
                for key in [
                    "bounds",
                    "members",
                    "center",
                    "start",
                    "stop",
                    "line",
                    "mean_distance",
                    "glyph",
                ] {
                    compare(&a[key], &b[key], &format!("{name}.{i}.{key}"));
                }
            }
        }
    }
}

#[test]
fn direct_original_factory_boundaries_preserve_order_and_membership() {
    let bank: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/stick-factory-controls.json"
    ))
    .unwrap();
    let cases = bank["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 26);
    for case in cases {
        let v: Vec<SeedSection> = serde_json::from_value(case["vertical"].clone()).unwrap();
        let h: Vec<SeedSection> = serde_json::from_value(case["horizontal"].clone()).unwrap();
        let actual = retrieve(
            &v,
            &h,
            case["max_thickness"].as_i64().unwrap(),
            case["min_core"].as_i64().unwrap(),
        )
        .unwrap();
        compare(
            &serde_json::json!(actual),
            &case["candidates"],
            case["name"].as_str().unwrap(),
        );
    }
}
