// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_beam_generation_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_beam_generation};
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
        "../fixtures/official-stem-image-beam-generation/controls.json"
    ))
    .unwrap()
}

#[test]
fn original_ordered_physical_beam_stumps_and_all_ninety_linkers() {
    let p = page();
    let before = p.clone();
    let owned = stem_image_beam_generation::prepare(&image(), &p).unwrap();
    controls::compare(&oracle(), &owned.report);
    assert_eq!(
        owned.snapshot_registry(),
        owned.report["beam_target_plan"]["head_linker_plan"]["source_plan"]["stump_plan"]["physical_registry"]
    );
    assert_eq!(p, before);
}
#[test]
fn empty_heads_and_late_unsupported_inputs_are_explicit() {
    let original = page();
    let im = image();
    let mut p = original.clone();
    p["line_completion"]["global_slope"] = json!(0.01);
    assert!(
        stem_image_beam_generation::analyze(&im, &p)
            .unwrap_err()
            .contains("skew")
    );
    p = original.clone();
    p["grid_ir"]["staffs"][5]["drum"] = json!(true);
    assert!(
        stem_image_beam_generation::analyze(&im, &p)
            .unwrap_err()
            .contains("staff")
    );
    p = original.clone();
    p["stem_seeds"]["stem_scaling"]["stem_scale"]["max"] = json!(0);
    assert!(
        stem_image_beam_generation::analyze(&im, &p)
            .unwrap_err()
            .contains("thickness")
    );
    p = original.clone();
    for h in p["head_registration"]["systems"].as_array_mut().unwrap() {
        h["heads"] = json!([]);
    }
    let result = stem_image_beam_generation::prepare(&im, &p).unwrap();
    assert_eq!(result.report["corner_count"], 0);
    assert_eq!(result.report["selected_blinker_occurrences"], 0);
    assert_eq!(result.report["stems_complete"], false);
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_generates_stumps_linkers_without_oracle_at_runtime() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-beam-generation-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--beam-generation")
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
