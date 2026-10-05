// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_stumps_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_stumps};
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
        "../fixtures/official-stem-image-stumps/controls.json"
    ))
    .unwrap()
}
#[test]
fn image_stump_selection_and_both_system_section_catalogs_match_original() {
    let p = page();
    let before = p.clone();
    let prepared = stem_image_stumps::prepare(&image(), &p).unwrap();
    controls::compare(&oracle(), &prepared.report, &p);
    assert_eq!(
        prepared.snapshot_registry(),
        prepared.report["physical_registry"]
    );
    assert_eq!(p, before);
}
#[test]
fn late_corrupt_sections_unknown_duplicate_members_and_empty_heads_preserve_input() {
    let original = page();
    let im = image();
    let mut p = original.clone();
    let last = p["grid_ir"]["vlag"].as_array().unwrap().len() - 1;
    p["grid_ir"]["vlag"][last]["runs"] = json!([]);
    assert!(
        stem_image_stumps::analyze(&im, &p)
            .unwrap_err()
            .contains("empty")
    );
    p = original.clone();
    p["grid_ir"]["systems"][1]["vertical_section_ids"]
        .as_array_mut()
        .unwrap()
        .push(json!(999999));
    assert!(
        stem_image_stumps::analyze(&im, &p)
            .unwrap_err()
            .contains("unknown")
    );
    p = original.clone();
    let id = p["grid_ir"]["systems"][1]["vertical_section_ids"][0].clone();
    p["grid_ir"]["systems"][1]["vertical_section_ids"]
        .as_array_mut()
        .unwrap()
        .push(id);
    assert!(
        stem_image_stumps::analyze(&im, &p)
            .unwrap_err()
            .contains("duplicate")
    );
    p = original.clone();
    for s in p["head_registration"]["systems"].as_array_mut().unwrap() {
        s["heads"] = json!([]);
    }
    let r = stem_image_stumps::analyze(&im, &p).unwrap();
    assert_eq!(r["selected_stump_corners"], 0);
    assert_eq!(r["new_registered_fallback_glyphs"], 0);
    assert_eq!(r["stems_complete"], false);
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_selects_seed_and_section_stumps_preserving_old_heads() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-stumps-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--stumps")
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-grid/laura-gray.png"),
        )
        .arg(d.join("residual.pgm"))
        .arg(d.join("heads.json"))
        .arg(d.join("stems.json"))
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let p: Value = serde_json::from_slice(&fs::read(d.join("heads.json")).unwrap()).unwrap();
    let n: Value = serde_json::from_slice(&fs::read(d.join("stems.json")).unwrap()).unwrap();
    controls::compare(&oracle(), &n, &p);
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(fs::read(d.join("heads.json")).unwrap())
        ),
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    fs::remove_dir_all(d).unwrap();
}
