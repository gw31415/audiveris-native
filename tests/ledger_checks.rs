// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    ledger_checks::{LedgerStick, Parameters, check},
};
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
            for (i, x) in v.iter().enumerate() {
                compare(&a[i], x, &format!("{path}[{i}]"));
            }
        }
        Value::Number(_) => {
            assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10,
                "{path}: {a} != {b}"
            );
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}
#[test]
fn original_ledger_suite_matches_every_raw_check_impact_grade_and_decision() {
    let bank: Value =
        serde_json::from_str(include_str!("../fixtures/official-ledger-suite.json")).unwrap();
    assert_eq!(bank["native_outputs_used"], false);
    let cases = bank["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1728);
    let width = bank["width"].as_u64().unwrap() as usize;
    let height = bank["height"].as_u64().unwrap() as usize;
    let mut positives = 0;
    let mut vetoes = 0;
    let mut pitch_zero = 0;
    for c in cases {
        let mut image = GrayImage {
            width,
            height,
            pixels: vec![255; width * height],
        };
        for p in c["black_pixels"].as_array().unwrap() {
            let x = p[0].as_u64().unwrap() as usize;
            let y = p[1].as_u64().unwrap() as usize;
            image.pixels[y * width + x] = 0;
        }
        let stick: LedgerStick = serde_json::from_value(c["filament"].clone()).unwrap();
        let il = c["interline"].as_u64().unwrap() as usize;
        let params = Parameters {
            main_interline: il,
            specific_interline: il,
            fore: c["fore"].as_u64().unwrap() as usize,
            target_y: c["target_y"].as_f64().unwrap(),
            wide_previous: c["wide_previous"].as_bool().unwrap(),
        };
        let result = check(&image, &stick, &params).unwrap();
        compare(
            &serde_json::to_value(&result).unwrap(),
            &c["result"],
            c["name"].as_str().unwrap(),
        );
        positives += usize::from(result.accepted);
        if result.checks[1].impact == 0. {
            assert_eq!(result.grade, 0.);
            vetoes += 1;
        }
        pitch_zero += usize::from(result.checks[5].impact == 0.);
    }
    assert!(positives > 0 && positives < cases.len() && vetoes > 0 && pitch_zero > 0);
    println!(
        "original ledger suite: {positives} accepted, {} rejected, {vetoes} zero-weight veto controls",
        cases.len() - positives
    );
}
#[test]
fn malformed_raster_and_nonfinite_geometry_stop_explicitly() {
    let image = GrayImage {
        width: 10,
        height: 10,
        pixels: vec![255; 99],
    };
    let stick = LedgerStick {
        bounds: [1, 1, 4, 1],
        start: [1., 1.],
        stop: [4., 1.],
        weight: 4,
        mean_distance: 0.,
    };
    let p = Parameters {
        main_interline: 16,
        specific_interline: 16,
        fore: 2,
        target_y: 1.,
        wide_previous: false,
    };
    assert!(check(&image, &stick, &p).is_err());
    let image = GrayImage {
        pixels: vec![255; 100],
        ..image
    };
    let mut invalid = stick;
    invalid.mean_distance = f64::NAN;
    assert!(check(&image, &invalid, &p).is_err());
}
