// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_parts_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_parts};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};
fn image() -> GrayImage {
    GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap()
}
fn page() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-stem-head-reference-context/native-laura-report.json.gz");
    let o=Command::new("python3").args(["-c","import gzip,sys;sys.stdout.buffer.write(gzip.decompress(open(sys.argv[1],'rb').read()))"]).arg(p).output().unwrap();
    assert!(o.status.success());
    serde_json::from_slice(&o.stdout).unwrap()
}
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-image-head-parts/controls.json"
    ))
    .unwrap()
}

#[test]
fn image_sources_and_all_head_pre_chunk_filter_items_match_original() {
    let p = page();
    let before = p.clone();
    let owned = stem_image_head_parts::prepare(&image(), &p).unwrap();
    controls::compare(&oracle(), &owned.report);
    assert_eq!(
        owned.snapshot_registry(),
        owned.report["beam_generation_plan"]["beam_target_plan"]["head_linker_plan"]["source_plan"]
            ["stump_plan"]["physical_registry"]
    );
    for sys in owned.report["systems"].as_array().unwrap() {
        for c in sys["cases"].as_array().unwrap() {
            for l in c["selected_linkers"].as_array().unwrap() {
                if l["kind"] == "HeadCorner" {
                    assert_eq!(l["native_head_origin"]["phase"], "HEAD");
                    assert!(l["native_head_origin"]["local"].as_u64().unwrap() > 0);
                } else {
                    assert!(l["native_beam_id"].as_u64().unwrap() > 0);
                    assert!(l["native_linker_id"].as_u64().unwrap() > 0);
                }
            }
        }
    }
    assert_eq!(p, before);
}
#[test]
fn empty_head_input_and_unsupported_skew_staff_remain_explicit() {
    let original = page();
    let im = image();
    let mut p = original.clone();
    p["line_completion"]["global_slope"] = json!(0.01);
    assert!(
        stem_image_head_parts::analyze(&im, &p)
            .unwrap_err()
            .contains("skew")
    );
    p = original.clone();
    p["grid_ir"]["staffs"][5]["drum"] = json!(true);
    assert!(
        stem_image_head_parts::analyze(&im, &p)
            .unwrap_err()
            .contains("staff")
    );
    p = original.clone();
    for h in p["head_registration"]["systems"].as_array_mut().unwrap() {
        h["heads"] = json!([]);
    }
    let r = stem_image_head_parts::prepare(&im, &p).unwrap();
    assert_eq!(r.report["corner_count"], 0);
    assert_eq!(r.report["pre_chunk_item_count"], 0);
    assert_eq!(r.report["stems_complete"], false);
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_connects_head_builder_inputs_without_condition_requests() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-head-parts-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--head-parts")
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-grid/laura-gray.png"),
        )
        .arg(d.join("residual.pgm"))
        .arg(d.join("heads.json"))
        .arg(d.join("stems.json"))
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(fs::read(d.join("heads.json")).unwrap())
        ),
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    let n = serde_json::from_slice(&fs::read(d.join("stems.json")).unwrap()).unwrap();
    controls::compare(&oracle(), &n);
    fs::remove_dir_all(d).unwrap();
}
