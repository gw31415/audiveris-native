// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::stem_parts_cli::{self, Request};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn request(f: &Value) -> Value {
    let sections = |name: &str| {
        f[name].as_array().unwrap().iter().map(|s|json!({"id":s["id"],"vertical":s["vertical"],"first_pos":s["first_pos"],"runs":s["runs"]})).collect::<Vec<_>>()
    };
    let searches=f["cases"].as_array().unwrap().iter().map(|c|json!({"polygon":c["area_path"].as_array().unwrap().iter().filter(|s|s["type"]==0||s["type"]==1).map(|s|s["coordinates"].clone()).collect::<Vec<_>>(),"stump":c["stump"],"parameters":{"direction":c["direction"],"slope":c["slope"],"max_stem_thickness":c["max_stem_thickness"],"max_line_section_dx":c["max_line_section_dx"],"last_head_y":c["last_head_y"],"theoretical_line":c["theoretical_line"]}})).collect::<Vec<_>>();
    json!({"schema":stem_parts_cli::REQUEST_SCHEMA,"vertical_sections":sections("vertical_sections"),"horizontal_sections":sections("horizontal_sections"),"searches":searches})
}
#[test]
fn cli_2343_original_controls_preserve_raw_collection_and_explicit_unfinished_flags() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-lookup/controls.json"
    ))
    .unwrap();
    let root = std::env::temp_dir().join(format!(
        "audiveris-stem-lookup-original-{}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    let input = root.join("request.json");
    let output = root.join("report.json");
    fs::write(&input, serde_json::to_vec(&request(&f)).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("stem-parts-lookup")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(report["schema"], stem_parts_cli::REPORT_SCHEMA);
    assert_eq!(report["lookup_geometry"], "caller_supplied_linear_polygons");
    assert_eq!(report["sections"], 3035);
    for flag in ["registered_to_sig", "heads_complete", "stems_complete"] {
        assert_eq!(report[flag], false);
    }
    assert_eq!(report["cases"].as_array().unwrap().len(), 2343);
    for (a, b) in f["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(report["cases"].as_array().unwrap())
    {
        assert_eq!(a["selected_vertical_ids"], b["vertical_ids"]);
        assert_eq!(a["selected_horizontal_ids"], b["horizontal_ids"]);
        assert_eq!(a["raw_sticks_min_core0"], b["raw_glyphs"]);
    }
    let before = fs::read(&output).unwrap();
    let repeat = Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("stem-parts-lookup")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!repeat.status.success());
    assert_eq!(before, fs::read(&output).unwrap());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn empty_inputs_and_malformed_late_search_refuse_before_report_creation() {
    let empty = json!({"schema":stem_parts_cli::REQUEST_SCHEMA,"vertical_sections":[],"horizontal_sections":[],"searches":[]});
    let parsed: Request = serde_json::from_value(empty.clone()).unwrap();
    let report = stem_parts_cli::evaluate(&parsed).unwrap();
    assert!(report.cases.is_empty());
    assert!(!report.stems_complete);
    let mut invalid = empty.clone();
    invalid["schema"] = json!("wrong");
    assert!(stem_parts_cli::evaluate(&serde_json::from_value(invalid).unwrap()).is_err());
    invalid = empty.clone();
    invalid["extra"] = json!(true);
    assert!(serde_json::from_value::<Request>(invalid).is_err());
    invalid = empty.clone();
    invalid["vertical_sections"] = json!([{"id":1,"vertical":true,"first_pos":20,"runs":[[0,-1]]}]);
    assert!(stem_parts_cli::evaluate(&serde_json::from_value(invalid).unwrap()).is_err());
    let search = json!({"polygon":[],"stump":null,"parameters":{"direction":1,"slope":0.,"max_stem_thickness":5,"max_line_section_dx":3.3,"last_head_y":null,"theoretical_line":[20.,0.,20.,100.]}});
    let mut excessive = empty.clone();
    excessive["searches"] = json!(vec![search.clone(); 4097]);
    assert_eq!(
        stem_parts_cli::evaluate(&serde_json::from_value(excessive).unwrap())
            .err()
            .unwrap(),
        "stem_lookup_request_budget"
    );
    let sections = (1..=500)
        .map(|id| json!({"id":id,"vertical":true,"first_pos":20,"runs":[[100,1000]]}))
        .collect::<Vec<_>>();
    excessive = empty.clone();
    excessive["vertical_sections"] = json!(sections);
    excessive["searches"] = json!(vec![search.clone(); 401]);
    assert_eq!(
        stem_parts_cli::evaluate(&serde_json::from_value(excessive).unwrap())
            .err()
            .unwrap(),
        "stem_lookup_request_work_budget"
    );
    let mut late = search.clone();
    late["parameters"]["slope"] = json!(0.01);
    invalid = empty.clone();
    invalid["searches"] = json!([search, late]);
    let root = std::env::temp_dir().join(format!(
        "audiveris-stem-lookup-refusal-{}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    let input = root.join("request.json");
    let output = root.join("report.json");
    fs::write(&input, serde_json::to_vec(&invalid).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("stem-parts-lookup")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
    let mut file = fs::File::create(&input).unwrap();
    use std::io::Write;
    file.write_all(b" ").unwrap();
    file.set_len((stem_parts_cli::MAX_REQUEST_BYTES + 1) as u64)
        .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("stem-parts-lookup")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
    fs::remove_dir_all(root).unwrap();
}
