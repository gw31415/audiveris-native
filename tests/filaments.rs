// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    Run, filaments,
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
fn pinned_java_seed_and_restricted_comb_goldens() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../fixtures/filament-golden.json")).unwrap();
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
        let got = filaments::analyze(case["width"].as_u64().unwrap() as usize, &sc, &sections);
        let java = &case["java"];
        assert_eq!(got["decisions"], java["decisions"]);
        assert_eq!(got["combs"], java["combs"]);
        let seeds = got["seeds"].as_array().unwrap();
        let expected = java["seeds"].as_array().unwrap();
        assert_eq!(seeds.len(), expected.len());
        for (a, b) in seeds.iter().zip(expected) {
            for key in ["section_id", "bounds", "comb_eligible"] {
                assert_eq!(a[key], b[key]);
            }
            for i in 0..3 {
                let x = a["line_abc"][i].as_f64().unwrap();
                let y = b["line_abc"][i].as_f64().unwrap();
                assert!((x - y).abs() <= 1e-10 + 1e-12 * y.abs());
            }
        }
        assert_eq!(got["staff_detection"], "not_complete");
    }
}
