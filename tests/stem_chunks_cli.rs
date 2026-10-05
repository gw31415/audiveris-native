// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::stem_chunks_cli::{self, Request};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn invoke(input: &std::path::Path, output: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("stem-chunks-lookup")
        .arg(input)
        .arg(output)
        .output()
        .unwrap()
}
#[test]
fn cli_original_3048_lookup_and_chunk_controls() {
    let parts: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-lookup/controls.json"
    ))
    .unwrap();
    let chunks: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-chunks/controls.json"
    ))
    .unwrap();
    let sections = |k: &str| {
        parts[k].as_array().unwrap().iter().map(|s|json!({"id":s["id"],"vertical":s["vertical"],"first_pos":s["first_pos"],"runs":s["runs"]})).collect::<Vec<_>>()
    };
    let mut searches = Vec::new();
    let actual = parts["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["mode"] == "actual");
    for (upstream, group) in actual.zip(chunks["groups"].as_array().unwrap()) {
        for c in group["cases"].as_array().unwrap() {
            searches.push(json!({"polygon":upstream["area_path"].as_array().unwrap().iter().filter(|s|s["type"]==0||s["type"]==1).map(|s|s["coordinates"].clone()).collect::<Vec<_>>(),"lookup_parameters":{"direction":upstream["direction"],"slope":upstream["slope"],"max_stem_thickness":upstream["max_stem_thickness"],"max_line_section_dx":upstream["max_line_section_dx"],"last_head_y":upstream["last_head_y"],"theoretical_line":upstream["theoretical_line"]},"initial_catalog":c["initial_catalog"].as_array().unwrap().iter().map(|e|e["glyph"].clone()).collect::<Vec<_>>(),"existing_ids":c["existing_ids"],"stump_id":c["stump_id"],"head_glyph":c["head_glyph"],"vip":c["vip"],"max_alignment_dx":c["max_dx"],"max_alignment_dy":c["max_dy"]}));
        }
    }
    let bytes=serde_json::to_vec(&json!({"schema":stem_chunks_cli::REQUEST_SCHEMA,"vertical_sections":sections("vertical_sections"),"horizontal_sections":sections("horizontal_sections"),"searches":searches})).unwrap();

    let root =
        std::env::temp_dir().join(format!("audiveris-chunks-original-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let input = root.join("request.json");
    let output = root.join("report.json");
    fs::write(&input, bytes).unwrap();
    let result = invoke(&input, &output);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    let expected = chunks["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["cases"].as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(report["schema"], stem_chunks_cli::REPORT_SCHEMA);
    assert_eq!(report["sections"], 3035);
    assert_eq!(report["cases"].as_array().unwrap().len(), 3048);
    for (a, b) in expected
        .into_iter()
        .zip(report["cases"].as_array().unwrap())
    {
        assert_eq!(a["raw_glyphs"], b["lookup"]["raw_glyphs"]);
        for k in [
            "registered_ids",
            "retained_ids",
            "new_count",
            "registry_count",
        ] {
            assert_eq!(a[k], b["chunks"][k]);
        }
        let catalog = b["registry"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| json!({"id":e["id"],"glyph":e["glyph"]}))
            .collect::<Vec<_>>();
        assert_eq!(a["final_catalog"], json!(catalog));
    }
    for k in ["registered_to_sig", "heads_complete", "stems_complete"] {
        assert_eq!(report[k], false);
    }
    let before = fs::read(&output).unwrap();
    assert!(!invoke(&input, &output).status.success());
    assert_eq!(fs::read(&output).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn empty_catalog_late_identity_skew_and_budgets_refuse_before_output() {
    let empty = json!({"schema":stem_chunks_cli::REQUEST_SCHEMA,"vertical_sections":[],"horizontal_sections":[],"searches":[]});
    let evaluate =
        |v: Value| stem_chunks_cli::evaluate(&serde_json::from_value::<Request>(v).unwrap());
    assert!(evaluate(empty.clone()).unwrap().cases.is_empty());
    let mut bad = empty.clone();
    bad["extra"] = json!(true);
    assert!(serde_json::from_value::<Request>(bad).is_err());
    bad = empty.clone();
    bad["vertical_sections"] = json!([{"id":1,"vertical":true,"first_pos":0,"runs":[]}]);
    assert!(evaluate(bad).is_err());
    let search = json!({"polygon":[],"lookup_parameters":{"direction":1,"slope":0.,"max_stem_thickness":5,"max_line_section_dx":3.3,"last_head_y":null,"theoretical_line":[20.,0.,20.,100.]},"initial_catalog":[],"existing_ids":[],"stump_id":null,"head_glyph":null,"vip":false,"max_alignment_dx":1.65,"max_alignment_dy":44.});
    let mut late = search.clone();
    late["existing_ids"] = json!([999]);
    bad = empty.clone();
    bad["searches"] = json!([search.clone(), late]);
    let root = std::env::temp_dir().join(format!("audiveris-chunks-refuse-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let input = root.join("request.json");
    let output = root.join("report.json");
    fs::write(&input, serde_json::to_vec(&bad).unwrap()).unwrap();
    assert!(!invoke(&input, &output).status.success());
    assert!(!output.exists());
    let mut skew = search.clone();
    skew["lookup_parameters"]["slope"] = json!(0.01);
    bad = empty.clone();
    bad["searches"] = json!([skew]);
    assert!(evaluate(bad).is_err());
    bad = empty.clone();
    bad["searches"] = json!(vec![search.clone(); 4097]);
    assert!(evaluate(bad).is_err());
    let g = json!({"bounds":[20,10,1,4],"orientation":"VERTICAL","runs":[[0,0,4]]});
    let mut alias = search.clone();
    alias["initial_catalog"] = json!([g.clone(), g]);
    bad = empty.clone();
    bad["searches"] = json!([alias]);
    assert!(evaluate(bad).is_err());
    bad = empty.clone();
    bad["vertical_sections"] = json!(
        (1..=500)
            .map(|id| json!({"id":id,"vertical":true,"first_pos":20,"runs":[[100,1000]]}))
            .collect::<Vec<_>>()
    );
    bad["searches"] = json!(vec![search; 401]);
    assert_eq!(
        evaluate(bad).err().unwrap(),
        "stem_chunks_request_work_budget"
    );
    let file = fs::File::create(&input).unwrap();
    file.set_len((stem_chunks_cli::MAX_REQUEST_BYTES + 1) as u64)
        .unwrap();
    assert!(!invoke(&input, &output).status.success());
    assert!(!output.exists());
    fs::remove_dir_all(root).unwrap();
}
