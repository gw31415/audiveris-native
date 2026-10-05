// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    ledger_beam_purge::{LedgerBeam, candidate_prefix, filter_sections, good_full_beams},
    ledger_candidates::retrieve_candidates,
    vertical_sticks::SeedSection,
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

fn bank() -> Value {
    serde_json::from_str(include_str!("../fixtures/official-ledger-beam-purge.json")).unwrap()
}
fn compare_sections(actual: &[SeedSection], expected: &Value) {
    let rows = expected.as_array().unwrap();
    assert_eq!(actual.len(), rows.len());
    for (a, e) in actual.iter().zip(rows) {
        compare(&a.snapshot(), e, "section");
    }
}
#[test]
fn original_all_beam_area_purge_and_intrinsic_full_beam_selection_match() {
    let b = bank();
    assert_eq!(b["native_outputs_used"], false);
    assert_eq!(b["good_grade"].as_f64().unwrap(), 0.35);
    let mut discarded = 0;
    let mut retained = 0;
    let mut good = 0;
    for c in b["cases"].as_array().unwrap() {
        let sections: Vec<SeedSection> = serde_json::from_value(c["sections"].clone()).unwrap();
        let beams: Vec<LedgerBeam> = serde_json::from_value(c["beams"].clone()).unwrap();
        let actual = filter_sections(&sections, &beams).unwrap();
        compare_sections(&actual, &c["filtered"]);
        if actual.is_empty() {
            discarded += 1
        } else {
            retained += 1
        };
        let selected = good_full_beams(&beams).unwrap();
        good += selected.len();
        let mut expected_good = c["good_beams"].clone();
        for beam in expected_good.as_array_mut().unwrap() {
            beam["segments"]
                .as_array_mut()
                .unwrap()
                .retain(|s| s[0] != s[1]);
        }
        compare(
            &serde_json::to_value(&selected).unwrap(),
            &expected_good,
            "good-beams",
        );
        let candidates = retrieve_candidates(
            &actual,
            b["interline"].as_u64().unwrap() as usize,
            b["fore"].as_u64().unwrap() as usize,
            &selected,
        )
        .unwrap();
        compare(
            &serde_json::json!(candidates),
            &c["candidates"],
            "candidates",
        );
    }
    assert!(discarded > 0 && retained > 0 && good > 0);
    println!(
        "beam ledger controls: {discarded} discarded, {retained} retained, {good} good full beams"
    );
}
#[test]
fn real_laura_native_ledger_candidate_prerequisites_match_original_beams() {
    let b = bank();
    let grid: Value = serde_json::from_str(include_str!(
        "../fixtures/official-ledger-reference-grid.json"
    ))
    .unwrap();
    let m = &b["no_staff"];
    let image = GrayImage::new(
        m["width"].as_u64().unwrap() as usize,
        m["height"].as_u64().unwrap() as usize,
        include_bytes!("../fixtures/official-ledger-beam-purge.gray").to_vec(),
    )
    .unwrap();
    let systems = b["real_systems"].as_array().unwrap();
    let beams: Vec<(u64, Vec<LedgerBeam>)> = systems
        .iter()
        .map(|s| {
            (
                s["system_id"].as_u64().unwrap(),
                serde_json::from_value(s["observation"]["beams"].clone()).unwrap(),
            )
        })
        .collect();
    let result = candidate_prefix(
        &image,
        &grid["grid"],
        b["interline"].as_u64().unwrap() as usize,
        b["fore"].as_u64().unwrap() as usize,
        &beams,
    )
    .unwrap();
    assert_eq!(result["ledgers_complete"], false);
    let mut count = 0;
    for (a, e) in result["systems"].as_array().unwrap().iter().zip(systems) {
        assert_eq!(a["system_id"], e["system_id"]);
        let sections: Vec<SeedSection> = serde_json::from_value(a["sections"].clone()).unwrap();
        compare_sections(&sections, &e["observation"]["filtered"]);
        compare(
            &a["candidates"],
            &e["observation"]["candidates"],
            "real-candidates",
        );
        count += a["candidates"].as_array().unwrap().len();
    }
    assert!(count > 0);
    println!("real Laura ledger candidates: {count}");
}
#[test]
fn invalid_beam_geometry_and_removed_beams_stop_or_are_excluded() {
    let b = bank();
    let c = &b["cases"][0];
    let sections: Vec<SeedSection> = serde_json::from_value(c["sections"].clone()).unwrap();
    let mut beams: Vec<LedgerBeam> = serde_json::from_value(c["beams"].clone()).unwrap();
    beams[0].height = f64::NAN;
    assert!(filter_sections(&sections, &beams).is_err());
    beams[0].removed = true;
    assert_eq!(
        filter_sections(&sections, &beams).unwrap().len(),
        sections.len()
    );
    assert!(good_full_beams(&beams).unwrap().is_empty());
}
