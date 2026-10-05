// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    curves::Curve,
    ledger_inputs::staff_distance_sections,
    ledger_reference::{PreviousLedger, select},
};
use serde_json::Value;
fn bank() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-ledger-reference-grid.json"
    ))
    .unwrap()
}
#[test]
fn original_signed_reference_order_strict_overlap_and_orphans_match() {
    let b = bank();
    let staff = b["grid"]["staffs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == b["reference_staff_id"])
        .unwrap();
    let curve = |i: usize| {
        Curve::interpolate(serde_json::from_value(staff["lines"][i]["points"].clone()).unwrap())
            .unwrap()
    };
    let first = curve(0);
    let last = curve(4);
    let mut positives = 0;
    let mut absent = 0;
    for c in b["reference_cases"].as_array().unwrap() {
        let ledgers: Vec<PreviousLedger> = serde_json::from_value(c["ledgers"].clone()).unwrap();
        let actual = select(
            &first,
            &last,
            c["index"].as_i64().unwrap() as i32,
            serde_json::from_value(c["bounds"].clone()).unwrap(),
            b["interline"].as_u64().unwrap() as usize,
            &ledgers,
        )
        .unwrap();
        if let Some(actual) = actual {
            positives += 1;
            assert!((actual.y - c["expected"]["y"].as_f64().unwrap()).abs() < 1e-9);
            assert_eq!(
                serde_json::json!(actual.previous_id),
                c["expected"]["previous_id"]
            );
        } else {
            absent += 1;
            assert!(c["expected"].is_null());
        }
    }
    assert!(positives > 0 && absent > 0);
    println!("reference controls: {positives} positive, {absent} orphan");
}
#[test]
fn real_laura_no_staff_raster_matches_original_ledger_filter_lag_and_dispatch() {
    let b = bank();
    assert_eq!(b["native_outputs_used"], false);
    let mask = &b["no_staff"];
    let pixels = include_bytes!("../fixtures/official-ledger-reference-grid.gray").to_vec();
    let image = GrayImage::new(
        mask["width"].as_u64().unwrap() as usize,
        mask["height"].as_u64().unwrap() as usize,
        pixels,
    )
    .unwrap();
    let actual = staff_distance_sections(
        &image,
        &b["grid"],
        b["interline"].as_u64().unwrap() as usize,
    )
    .unwrap();
    let expected = b["grid"]["systems"].as_array().unwrap();
    let rows = actual["systems"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    let mut total = 0;
    for (a, e) in rows.iter().zip(expected) {
        assert_eq!(a["system_id"], e["id"]);
        let original = e["sections"].as_array().unwrap();
        let native = a["sections"].as_array().unwrap();
        assert_eq!(native.len(), original.len());
        for (a, e) in native.iter().zip(original) {
            for key in ["id", "vertical", "first_pos", "runs"] {
                assert_eq!(a[key], e[key], "{} {key}", e["id"]);
            }
        }
        total += native.len();
    }
    assert!(total > 0);
    assert_eq!(actual["ledgers_complete"], false);
    assert_eq!(actual["beam_section_filter_complete"], false);
    println!("real Laura ledger input sections: {total}");
}
