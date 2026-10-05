// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_head_neighborhood_controls.rs"]
mod controls;
use audiveris_native::stem_head_neighborhood::{self, Bounds, Neighborhood, Request};
use serde_json::{Value, json};
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-head-neighborhood/controls.json"
    ))
    .unwrap()
}
#[test]
fn original_full_laura_state_neighborhood_area_targets_and_rectangle_edges() {
    let f = fixture();
    assert_eq!(f["corners"], 1408);
    let o = controls::observe(&f);
    assert_eq!(o["rectangle_boundaries"].as_array().unwrap().len(), 14);
}
#[test]
fn actual_cli_page_state_and_atomic_late_refusal() {
    use std::{fs, process::Command};
    let f = fixture();
    let mut r = controls::request(&f);
    let dir = std::env::temp_dir().join(format!("stem-head-plan-test-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let request = dir.join("request.json");
    let report = dir.join("report.json");
    fs::write(&request, serde_json::to_vec(&r).unwrap()).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
            .arg("stem-head-plan")
            .arg(&request)
            .arg(&report)
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    let bytes = fs::read(&report).unwrap();
    let native = serde_json::from_slice(&bytes).unwrap();
    controls::compare(&f, &native);
    assert!(!run().status.success());
    assert_eq!(fs::read(&report).unwrap(), bytes);
    fs::remove_file(&report).unwrap();
    let last = r["systems"].as_array().unwrap().len() - 1;
    r["systems"][last]["cases"][0]["head"]["slope"] = json!(0.01);
    fs::write(&request, serde_json::to_vec(&r).unwrap()).unwrap();
    assert!(!run().status.success() && !report.exists());
    r["systems"][last]["cases"][0]["head"]["slope"] = json!(0.);
    r["unexpected"] = json!(true);
    fs::write(&request, serde_json::to_vec(&r).unwrap()).unwrap();
    assert!(!run().status.success() && !report.exists());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn late_unknown_ids_empty_page_and_catalog_budgets_are_explicit() {
    let bounds = vec![Bounds {
        id: 1,
        bounds: [10, 20, 3, 4],
        removed: false,
    }];
    let n = Neighborhood::new(&bounds).unwrap();
    assert!(
        n.neighbors(&[1, 2], [0, 0, 1, 1], [0, 0, 100, 100], 0)
            .is_err()
    );
    assert!(n.initial_candidates(&[1, 2], &[]).is_err());
    assert_eq!(
        n.neighbors(&[1, 1], [10, 0, 3, 1], [0, 0, 100, 100], 0)
            .unwrap(),
        vec![1, 1]
    );
    let empty: Request = serde_json::from_value(
        json!({"schema":stem_head_neighborhood::REQUEST_SCHEMA,"systems":[]}),
    )
    .unwrap();
    let e = serde_json::to_value(stem_head_neighborhood::evaluate(&empty).unwrap()).unwrap();
    assert_eq!(e["systems"], json!([]));
    assert_eq!(e["stems_complete"], false);
    let mut r = controls::request(&fixture());
    r["systems"][1]["beam_bounds"][0]["id"] = json!(9999);
    assert!(stem_head_neighborhood::evaluate(&serde_json::from_value(r).unwrap()).is_err());
    let mut r = controls::request(&fixture());
    r["systems"][0]["cases"] = json!([]);
    r["systems"][0]["vicinity_margin"] = json!(-1);
    assert!(stem_head_neighborhood::evaluate(&serde_json::from_value(r).unwrap()).is_err());
    assert!(Neighborhood::new(&vec![bounds[0].clone(); 4097]).is_err());
    let mut r = controls::request(&fixture());
    r["systems"][0]["beams"] = json!([r["systems"][0]["beams"][0].clone()]);
    r["systems"][0]["beam_ids"] = json!(vec![1; 4096]);
    r["systems"][0]["cases"] = json!(vec![r["systems"][0]["cases"][0].clone(); 8192]);
    let request: Request = serde_json::from_value(r).unwrap();
    assert!(
        stem_head_neighborhood::evaluate(&request)
            .err()
            .unwrap()
            .contains("work budget")
    );
}
