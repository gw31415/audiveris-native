// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::ledger_candidates::{GoodBeamArea, retrieve_candidates};
use audiveris_native::vertical_sticks::SeedSection;
use serde_json::Value;

fn compare(actual: &Value, expected: &Value, path: &str) {
    match expected {
        Value::Object(m) => {
            for (key, value) in m {
                compare(&actual[key], value, &format!("{path}.{key}"));
            }
        }
        Value::Array(values) => {
            assert_eq!(actual.as_array().unwrap().len(), values.len(), "{path}");
            for (i, value) in values.iter().enumerate() {
                compare(&actual[i], value, &format!("{path}[{i}]"));
            }
        }
        Value::Number(_) if expected.as_i64().is_none() => {
            assert!(
                (actual.as_f64().unwrap() - expected.as_f64().unwrap()).abs() < 1e-8,
                "{path}: {actual} != {expected}"
            );
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}

#[test]
fn original_ledger_candidate_prefix_matches_members_geometry_and_purges() {
    let bank: Value =
        serde_json::from_str(include_str!("../fixtures/official-ledger-candidates.json")).unwrap();
    assert_eq!(bank["native_outputs_used"], false);
    let cases = bank["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 93);
    let mut positive = 0;
    let mut absent = 0;
    for case in cases {
        let sections: Vec<SeedSection> = serde_json::from_value(case["sections"].clone()).unwrap();
        let beams: Vec<GoodBeamArea> = serde_json::from_value(case["good_beams"].clone()).unwrap();
        let actual = retrieve_candidates(
            &sections,
            case["interline"].as_u64().unwrap() as usize,
            case["fore"].as_u64().unwrap() as usize,
            &beams,
        )
        .unwrap();
        if actual.is_empty() {
            absent += 1;
        } else {
            positive += 1;
        }
        compare(
            &serde_json::json!(actual),
            &case["candidates"],
            case["name"].as_str().unwrap(),
        );
    }
    assert!(positive > 0 && absent > 0);
    println!("original ledger controls: {positive} positive, {absent} absent");
}

#[test]
fn malformed_scale_orientation_and_good_beam_order_stop_explicitly() {
    assert!(retrieve_candidates(&[], 0, 2, &[]).is_err());
    let vertical = SeedSection {
        id: 1,
        vertical: true,
        first_pos: 10,
        runs: vec![[10, 50]],
    };
    assert!(retrieve_candidates(&[vertical], 16, 2, &[]).is_err());
    let beam = |x| GoodBeamArea {
        bounds: [x, 0, 10, 10],
        segments: vec![
            vec![[x as f64, 0.], [x as f64 + 10., 0.]],
            vec![[x as f64 + 10., 0.], [x as f64 + 10., 10.]],
            vec![[x as f64 + 10., 10.], [x as f64, 10.]],
            vec![[x as f64, 10.], [x as f64, 0.]],
        ],
    };
    assert!(retrieve_candidates(&[], 16, 2, &[beam(20), beam(10)]).is_err());
}
