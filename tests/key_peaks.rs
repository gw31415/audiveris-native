// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::key_peaks::{Parameters, find};
use serde_json::Value;

#[test]
fn ten_original_keybuilder_peak_observations_match_exactly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in [
        "laura-gray",
        "synthetic-system-bars",
        "cclef-pair",
        "bracket-pair",
        "line-root",
        "rendered-clef-g_clef",
        "rendered-clef-f_clef",
        "rendered-clef-c_clef",
        "rendered-clef-empty",
        "rendered-clef-noise",
    ] {
        let oracle: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-key-peaks.json")))
                .unwrap(),
        )
        .unwrap();
        for s in oracle["staffs"].as_array().unwrap() {
            let p = &s["params"];
            let params = Parameters {
                min_value: p["min_value"].as_i64().unwrap() as i32,
                min_derivative: p["min_peak_derivative"].as_i64().unwrap() as i32,
                min_gain_ratio: p["min_gain_ratio"].as_f64().unwrap(),
                quorum: p["min_peak_cumul"].as_i64().unwrap() as i32,
            };
            let values = s["projection"]["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_i64().unwrap() as i32)
                .collect::<Vec<_>>();
            let mut peaks = find(
                &values,
                s["projection"]["x_min"].as_u64().unwrap() as usize,
                [
                    s["browse_start"].as_u64().unwrap() as usize,
                    s["browse_stop"].as_u64().unwrap() as usize,
                ],
                params,
            )
            .unwrap();
            peaks.sort_by_key(|p| p.main);
            assert_eq!(
                serde_json::to_value(peaks).unwrap(),
                s["peaks"],
                "{name} staff {}",
                s["staff_id"]
            );
        }
    }
}

#[test]
fn original_hilo_113_controlled_cases_preserve_order_and_threshold_edges() {
    let oracle: Value = serde_json::from_slice(include_bytes!(
        "../fixtures/key-peaks/official-controlled.json"
    ))
    .unwrap();
    assert_eq!(oracle["controlled_input"], true);
    let cases = oracle["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 113);
    for c in cases {
        let params = Parameters {
            min_value: c["min_value"].as_i64().unwrap() as i32,
            min_derivative: c["min_derivative"].as_i64().unwrap() as i32,
            min_gain_ratio: c["min_gain_ratio"].as_f64().unwrap(),
            quorum: c["quorum"].as_i64().unwrap() as i32,
        };
        let values = c["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as i32)
            .collect::<Vec<_>>();
        let peaks = find(
            &values,
            c["function_x_min"].as_u64().unwrap() as usize,
            [
                c["domain_start"].as_u64().unwrap() as usize,
                c["domain_stop"].as_u64().unwrap() as usize,
            ],
            params,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(peaks).unwrap(),
            c["returned_order"],
            "{}",
            c["name"]
        );
    }
}
