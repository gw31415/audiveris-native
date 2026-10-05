// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    Run, filament_merge,
    scale::{Beam, Range, Scale},
    sections::{Orientation, Section, SectionId},
};
use serde_json::Value;
fn range(v: &Value) -> Range {
    Range {
        min: v[0].as_u64().unwrap() as usize,
        main: v[1].as_u64().unwrap() as usize,
        max: v[2].as_u64().unwrap() as usize,
    }
}
#[test]
fn pinned_java_merge_curve_filter_and_comb_goldens() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../fixtures/curved-filament-golden.json")).unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let sc = Scale {
            line: range(&case["line"]),
            interline: range(&case["interline"]),
            small_interline: None,
            beam: Beam {
                thickness: 1,
                extrapolated: true,
            },
            small_beam: None,
        };
        let sections: Vec<_> = case["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Section {
                id: SectionId(v["id"].as_u64().unwrap() as u32),
                orientation: Orientation::Horizontal,
                first_pos: v["first_pos"].as_u64().unwrap() as usize,
                weight: v["weight"].as_u64().unwrap() as usize,
                runs: v["runs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| Run {
                        start: r[0].as_u64().unwrap() as usize,
                        length: r[1].as_u64().unwrap() as usize,
                    })
                    .collect(),
            })
            .collect();
        let got = filament_merge::analyze(case["width"].as_u64().unwrap() as usize, &sc, &sections);
        let java = &case["java"];
        compare(&got, java);
        assert_eq!(got["staff_detection"], "not_complete");
    }
}

fn compare(got: &Value, expected: &Value) {
    match expected {
        Value::Object(map) => {
            for (k, v) in map {
                compare(&got[k], v)
            }
        }
        Value::Array(v) => {
            let a = got.as_array().unwrap();
            assert_eq!(a.len(), v.len());
            for (x, y) in a.iter().zip(v) {
                compare(x, y);
            }
        }
        Value::Number(n) => {
            let y = n.as_f64().unwrap();
            let x = got.as_f64().unwrap();
            assert!((x - y).abs() <= 1e-9 + 1e-12 * y.abs(), "{x} != {y}");
        }
        _ => assert_eq!(got, expected),
    }
}

#[test]
fn section_budget_rejects_before_returning_partial_factory_output() {
    let sc = Scale {
        line: Range {
            min: 2,
            main: 2,
            max: 2,
        },
        interline: Range {
            min: 16,
            main: 16,
            max: 16,
        },
        small_interline: None,
        beam: Beam {
            thickness: 9,
            extrapolated: true,
        },
        small_beam: None,
    };
    let sections: Vec<_> = (0..5001)
        .map(|i| Section {
            id: SectionId(i),
            orientation: Orientation::Horizontal,
            first_pos: i as usize,
            runs: vec![Run {
                start: 0,
                length: 10,
            }],
            weight: 10,
        })
        .collect();
    let result = filament_merge::analyze(320, &sc, &sections);
    assert_eq!(result["error"], "filament_section_limit");
    assert_eq!(result["status"], "failed");
    assert!(result.get("filaments").is_none());
}
