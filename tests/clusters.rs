// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    Run,
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
fn pinned_java_cluster_and_initial_staff_goldens() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../fixtures/cluster-golden.json")).unwrap();
    assert_eq!(cases.len(), 9);
    for case in cases {
        let sc = Scale {
            line: range(&case["scale"]["line"]),
            interline: range(&case["scale"]["interline"]),
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
        let got = audiveris_native::clusters::analyze(&sc, &sections, &case["factory"]);
        let java = &case["java"];
        compare(&got, java);
        assert_eq!(got["status"], "initial_staffs");
        let mut dual = sc;
        dual.small_interline = Some(dual.interline);
        assert_eq!(
            audiveris_native::clusters::analyze(&dual, &sections, &case["factory"])["status"],
            "unsupported"
        );
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
