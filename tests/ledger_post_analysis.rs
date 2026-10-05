// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::Registry,
    ledger_post_analysis::{Info, filter, process},
    ledger_registration::{Staff, build},
};
use serde_json::{Value, json};
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

fn bank() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-ledger-post-analysis.json"
    ))
    .unwrap()
}
#[test]
fn original_sample_variance_and_rounded_statistical_cutoffs_match() {
    let b = bank();
    assert_eq!(b["native_outputs_used"], false);
    let mut retained = 0;
    let mut discarded = 0;
    for c in b["controls"].as_array().unwrap() {
        let infos: Vec<Info> = serde_json::from_value(c["infos"].clone()).unwrap();
        let result = filter(&infos).unwrap();
        compare(&result, &c["statistics"], "statistics");
        assert_eq!(result["discarded"], c["discarded"]);
        discarded += result["discarded"].as_array().unwrap().len();
        retained += infos.len() - result["discarded"].as_array().unwrap().len();
    }
    assert!(retained > 0 && discarded > 0);
    println!("post-analysis controls: {retained} retained, {discarded} discarded");
}
#[test]
fn original_real_laura_post_filter_rebuild_and_translated_paths_match() {
    let b = bank();
    let before: Value = serde_json::from_str(include_str!(
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
    let il = raster["interline"].as_u64().unwrap() as usize;
    let fore = raster["fore"].as_u64().unwrap() as usize;
    let mut registry = Registry::default();
    let mut specs = Vec::new();
    let mut candidates = Vec::new();
    let mut systems = Vec::new();
    for sys in before["systems"].as_array().unwrap() {
        let staffs: Vec<Staff> = serde_json::from_value(sys["staffs"].clone()).unwrap();
        let report = build(
            &image,
            sys["candidates"].as_array().unwrap(),
            &staffs,
            il,
            fore,
            &mut registry,
        )
        .unwrap();
        systems.push(json!({"system_id":sys["system_id"],"registration":report}));
        specs.push((sys["system_id"].as_u64().unwrap(), staffs));
        candidates.push(json!({"system_id":sys["system_id"],"candidates":sys["candidates"]}));
    }
    let result = process(
        &image,
        &systems,
        &specs,
        &candidates,
        il,
        fore,
        &mut registry,
    )
    .unwrap();
    compare(&result["statistics"], &b["statistics"], "real-statistics");
    let mut actual_infos = result["infos"].as_array().unwrap().clone();
    let mut original_infos = b["infos"].as_array().unwrap().clone();
    let key = |i: &Value| {
        (
            i["system_id"].as_u64().unwrap(),
            i["ledger_id"].as_u64().unwrap(),
        )
    };
    actual_infos.sort_by_key(key);
    original_infos.sort_by_key(key);
    compare(&json!(actual_infos), &json!(original_infos), "infos");
    assert_eq!(result["discarded"], b["discarded"]);
    let mut count = 0;
    for (a, e) in result["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(b["systems"].as_array().unwrap())
    {
        let r = &a["registration"];
        let mut nodes = r["inters"].as_array().unwrap().clone();
        nodes.sort_by_key(|n| n["candidate_index"].as_u64().unwrap());
        compare(&json!(nodes), &e["inters"], "nodes");
        let mut maps = r["staff_maps"].as_array().unwrap().clone();
        for m in &mut maps {
            m["lines"]
                .as_array_mut()
                .unwrap()
                .sort_by_key(|l| l["index"].as_i64().unwrap());
        }
        compare(&json!(maps), &e["staff_maps"], "staff-maps");
        compare(&r["ledger_paths"], &e["ledger_paths"], "ledger-paths");
        count += r["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["removed"] == false)
            .count();
    }
    assert!(count > 0);
    println!("real Laura final ledgers: {count}");
}
