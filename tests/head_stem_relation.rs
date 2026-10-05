// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::head_stem_relation as stem;
use serde_json::Value;
fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-stem-kernel/controls.json"
    ))
    .unwrap()
}
#[test]
fn original_head_stem_portion_consistency_invading_and_profile_boundaries() {
    let gold = original();
    assert_eq!(gold["native_outputs_used"], false);
    assert_eq!(gold["portions"].as_array().unwrap().len(), 264);
    for c in gold["portions"].as_array().unwrap() {
        let bounds = serde_json::from_value(c["bounds"].clone()).unwrap();
        let line = serde_json::from_value(c["stem"].clone()).unwrap();
        let got = stem::stem_portion(bounds, line, c["anchor_y"].as_f64().unwrap()).unwrap();
        assert_eq!(serde_json::to_value(got).unwrap(), c["portion"], "{c}");
    }
    assert_eq!(gold["consistency"].as_array().unwrap().len(), 20);
    for c in gold["consistency"].as_array().unwrap() {
        let got = stem::consistency(
            c["small"].as_bool().unwrap(),
            Some(c["length_interlines"].as_f64().unwrap()),
        )
        .unwrap();
        if c["consistency"] == "Infinity" {
            assert_eq!(got, f64::INFINITY);
        } else {
            assert_eq!(got, c["consistency"].as_f64().unwrap(), "{c}");
        }
    }
    assert_eq!(gold["invading"].as_array().unwrap().len(), 70);
    for c in gold["invading"].as_array().unwrap() {
        assert_eq!(
            stem::is_invading(c["dx"].as_f64().unwrap(), c["dy"].as_f64().unwrap()).unwrap(),
            c["invading"].as_bool().unwrap(),
            "{c}"
        );
    }
    assert_eq!(gold["gap_limits"].as_array().unwrap().len(), 3);
    for c in gold["gap_limits"].as_array().unwrap() {
        let got = stem::gap_limits(c["profile"].as_u64().unwrap() as u8).unwrap();
        assert_eq!(got.x_in_max, c["x_in_max"].as_f64().unwrap());
        assert_eq!(got.x_out_max, c["x_out_max"].as_f64().unwrap());
        assert_eq!(got.y_max, c["y_max"].as_f64().unwrap());
    }
    assert_eq!(
        stem::consistency(false, None).unwrap(),
        gold["default_consistency"].as_f64().unwrap()
    );
}
#[test]
fn invalid_geometry_fraction_units_and_zero_length_ieee_contract_are_explicit() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 4_000_001.] {
        assert!(stem::stem_portion([0, 0, 15, 10], [0., 0., 0., bad], 5.).is_err());
        assert!(stem::stem_portion([0, 0, 15, 10], [0., 0., 0., 10.], bad).is_err());
        assert!(stem::is_invading(bad, 0.).is_err());
        assert!(stem::is_invading(0., bad).is_err());
        assert!(stem::consistency(false, Some(bad)).is_err());
    }
    for bounds in [
        [0, 0, 0, 10],
        [0, 0, 15, 0],
        [0, 0, 15, -1],
        [i32::MIN, 0, 15, 10],
    ] {
        assert!(stem::stem_portion(bounds, [0., 0., 0., 10.], 5.).is_err());
    }
    assert!(stem::consistency(false, Some(-1.)).is_err());
    assert_eq!(stem::consistency(false, Some(0.)).unwrap(), 0.);
    assert_eq!(stem::consistency(true, Some(0.)).unwrap(), f64::INFINITY);
    assert_eq!(stem::consistency(true, None).unwrap(), 1.);
    assert!(stem::gap_limits(3).is_err());
    assert!(stem::gap_limits(u8::MAX).is_err());
    // Signed overlap and inclusive contact gates, in interline fractions.
    assert!(stem::is_invading(-0.2, 0.).unwrap());
    assert!(stem::is_invading(0.05, 0.).unwrap());
    assert!(!stem::is_invading(0.05 + 1e-9, 0.).unwrap());
    assert!(!stem::is_invading(0., 1e-9).unwrap());
}
