// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_item_events_controls.rs"]
mod controls;
use audiveris_native::{
    stem_item_events::{self, Kind, Source},
    stem_items_cli,
};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-item-events/controls.json"
    ))
    .unwrap()
}
#[test]
fn original_constructors_sort_gaps_and_composed_lengths() {
    let v = controls::observe(&fixture());
    assert_eq!(v["report"]["cases"].as_array().unwrap().len(), 4432);
    assert_eq!(v["declared_line_boundaries"].as_array().unwrap().len(), 18);
}
#[test]
fn actual_cli_original_controls_and_atomic_invalid_late_input() {
    let o = fixture();
    let req = controls::request(&o);
    let dir = std::env::temp_dir().join(format!("stem-items-cli-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let input = dir.join("input.json");
    let output = dir.join("output.json");
    // Case/Request intentionally Deserialize-only; construct equivalent fixture JSON.
    let gap_map = &o["gap_map"];
    let cases=o["groups"].as_array().unwrap().iter().flat_map(|g|g["cases"].as_array().unwrap().iter().map(move|c|json!({"direction":g["direction"],"start_y":g["start_y"],"y_range":g["y_range"],"sources":c["sources"],"gap_map":gap_map,"stem_profile":c["stem_profile"],"max_profile":4,"initial_lengths":{},"skew":0.}))).collect::<Vec<_>>();
    let mut request = json!({"schema":stem_items_cli::REQUEST_SCHEMA,"cases":cases});
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();
    let invoke = |out: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
            .arg("stem-items-plan")
            .arg(&input)
            .arg(out)
            .output()
            .unwrap()
    };
    assert!(invoke(&output).status.success());
    let bytes = fs::read(&output).unwrap();
    let native = serde_json::from_slice(&bytes).unwrap();
    controls::compare_report(&o, &native);
    assert!(!invoke(&output).status.success());
    assert_eq!(fs::read(&output).unwrap(), bytes);
    request["cases"].as_array_mut().unwrap().last_mut().unwrap()["skew"] = json!(0.001);
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();
    let late = dir.join("late.json");
    assert!(!invoke(&late).status.success());
    assert!(!late.exists());
    request["unexpected"] = json!(true);
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();
    assert!(!invoke(&late).status.success());
    assert!(!late.exists());
    fs::write(&input, vec![b' '; stem_items_cli::MAX_REQUEST_BYTES + 1]).unwrap();
    assert!(!invoke(&late).status.success());
    assert!(!late.exists());
    assert_eq!(stem_items_cli::evaluate(&req).unwrap().cases.len(), 4432);
    fs::remove_dir_all(&dir).unwrap();
}
#[test]
fn aliases_limits_empty_and_strict_event_refusals() {
    let s = Source {
        token: 1,
        kind: Kind::HalfLinkerItem,
        glyph: None,
        reference: Some([0., 0.]),
        beam_height: None,
        head_bounds: None,
    };
    let mut duplicate = s.clone();
    assert!(stem_item_events::construct(&[s.clone(), duplicate.clone()], [0, 0, 0, 20]).is_ok());
    duplicate.reference = Some([0., 1.]);
    assert!(stem_item_events::construct(&[s.clone(), duplicate], [0, 0, 0, 20]).is_err());
    assert!(stem_item_events::construct(&vec![s.clone(); 33], [0, 0, 0, 20]).is_err());
    assert!(stem_item_events::construct(&[s], [0, 0, 0, -1]).is_err());
    let empty = stem_item_events::events(&[], 1, 0).unwrap();
    assert!(empty.items.is_empty() && !empty.truncated);
    assert!(stem_item_events::events(&[], 0, 0).is_err());
    assert!(stem_item_events::events(&[], 1, -1).is_err());
    let mut req = controls::request(&fixture());
    req.cases.last_mut().unwrap().max_profile = 31;
    assert!(stem_items_cli::evaluate(&req).is_err());
    req.schema = "wrong".into();
    assert!(stem_items_cli::evaluate(&req).is_err());
}
