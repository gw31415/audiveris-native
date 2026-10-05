// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::Registry,
    ledger_registration::{Ledger, Staff, build, reduce},
};
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
fn original_real_laura_acceptance_reduction_and_staff_maps_match() {
    let b: Value = serde_json::from_str(include_str!(
        "../fixtures/official-ledger-registration.json"
    ))
    .unwrap();
    let raster: Value =
        serde_json::from_str(include_str!("../fixtures/official-ledger-beam-purge.json")).unwrap();
    let m = &raster["no_staff"];
    let image = GrayImage::new(
        m["width"].as_u64().unwrap() as usize,
        m["height"].as_u64().unwrap() as usize,
        include_bytes!("../fixtures/official-ledger-beam-purge.gray").to_vec(),
    )
    .unwrap();
    let mut registry = Registry::default();
    let mut total = 0;
    for s in b["systems"].as_array().unwrap() {
        let staffs: Vec<Staff> = serde_json::from_value(s["staffs"].clone()).unwrap();
        let result = build(
            &image,
            s["candidates"].as_array().unwrap(),
            &staffs,
            raster["interline"].as_u64().unwrap() as usize,
            raster["fore"].as_u64().unwrap() as usize,
            &mut registry,
        )
        .unwrap();
        let mut actual = result["inters"].as_array().unwrap().clone();
        actual.sort_by_key(|l| l["candidate_index"].as_u64().unwrap());
        let expected = s["inters"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            for k in [
                "candidate_index",
                "index",
                "staff_id",
                "bounds",
                "grade",
                "removed",
            ] {
                compare(&a[k], &e[k], k);
            }
            for k in ["bounds", "orientation", "runs"] {
                compare(&a["glyph"][k], &e["glyph"][k], k);
            }
            if a["removed"] == false {
                total += 1;
            }
        }
        for (a, e) in result["staff_maps"]
            .as_array()
            .unwrap()
            .iter()
            .zip(s["staff_maps"].as_array().unwrap())
        {
            assert_eq!(a["staff_id"], e["staff_id"]);
            let mut lines = a["lines"].as_array().unwrap().clone();
            lines.sort_by_key(|l| l["index"].as_i64().unwrap());
            assert_eq!(lines.len(), e["lines"].as_array().unwrap().len());
            for (l, o) in lines.iter().zip(e["lines"].as_array().unwrap()) {
                assert_eq!(l["index"], o["index"]);
                let ids = l["ledger_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| {
                        result["inters"][id.as_u64().unwrap() as usize - 1]["candidate_index"]
                            .clone()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(serde_json::json!(ids), o["candidate_indices"]);
            }
        }
    }
    assert!(total > 0);
    println!(
        "original real Laura accepted ledgers: {total}, registered glyphs: {}",
        registry.len()
    );
}
#[test]
fn original_exclusion_strongest_edge_and_registration_tie_order_match() {
    let b: Value = serde_json::from_str(include_str!(
        "../fixtures/official-ledger-registration.json"
    ))
    .unwrap();
    for c in b["reduction_cases"].as_array().unwrap() {
        let mut nodes = c["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| Ledger {
                id: n["id"].as_u64().unwrap() as usize,
                candidate_index: 1,
                glyph_id: 1,
                index: 1,
                staff_id: None,
                bounds: serde_json::from_value(n["bounds"].clone()).unwrap(),
                grade: n["grade"].as_f64().unwrap(),
                removed: false,
                glyph: Value::Null,
            })
            .collect::<Vec<_>>();
        let mut ids = (0..nodes.len()).collect();
        reduce(&mut nodes, &mut ids);
        assert_eq!(
            serde_json::json!(ids.iter().map(|&n| nodes[n].id).collect::<Vec<_>>()),
            c["kept_ids"]
        );
    }
    println!(
        "original reduction controls: {}",
        b["reduction_cases"].as_array().unwrap().len()
    );
}
