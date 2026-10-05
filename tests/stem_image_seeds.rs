// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_seeds_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_seeds};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};
fn image() -> GrayImage {
    GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap()
}
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-image-seeds/controls.json"
    ))
    .unwrap()
}
fn page() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-stem-head-reference-context/native-laura-report.json.gz");
    let o=Command::new("python3").args(["-c","import gzip,sys;sys.stdout.buffer.write(gzip.decompress(open(sys.argv[1],'rb').read()))"]).arg(p).output().unwrap();
    assert!(o.status.success());
    serde_json::from_slice(&o.stdout).unwrap()
}
#[test]
fn image_seed_branch_preserves_page_registry_and_matches_all_original_choices() {
    let p = page();
    let before = p.clone();
    let n = stem_image_seeds::analyze(&image(), &p).unwrap();
    controls::compare(&oracle(), &n);
    assert_eq!(
        n["physical_registry"],
        p["head_registration"]["live_glyph_retrieval"]["registry"]
    );
    assert_eq!(p, before);
}
#[test]
fn empty_duplicate_bad_late_glyph_and_angled_bar_do_not_mutate_caller_state() {
    let original = page();
    let im = image();
    let mut p = original.clone();
    for s in p["head_registration"]["systems"].as_array_mut().unwrap() {
        s["heads"] = json!([]);
    }
    let n = stem_image_seeds::analyze(&im, &p).unwrap();
    assert_eq!(n["corner_count"], 0);
    assert_eq!(n["stems_complete"], false);
    p = original.clone();
    let g = p["stem_seeds"]["systems"][1]["accepted_seeds"][0].clone();
    p["stem_seeds"]["systems"][1]["accepted_seeds"]
        .as_array_mut()
        .unwrap()
        .push(g);
    assert!(
        stem_image_seeds::analyze(&im, &p)
            .unwrap_err()
            .contains("duplicate")
    );
    p = original.clone();
    p["stem_seeds"]["systems"][1]["accepted_seeds"][201]["runs"] = json!([[0, 0, 0]]);
    assert!(stem_image_seeds::analyze(&im, &p).is_err());
    p = original.clone();
    p["head_registration"]["shared_sig"]["inters"][0]["median"][2] = json!(234.1);
    assert!(
        stem_image_seeds::analyze(&im, &p)
            .unwrap_err()
            .contains("angled")
    );
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_automatically_selects_existing_physical_seeds() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-seeds-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--seeds")
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-grid/laura-gray.png"),
        )
        .arg(d.join("residual.pgm"))
        .arg(d.join("heads.json"))
        .arg(d.join("stems.json"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(&fs::read(d.join("stems.json")).unwrap()).unwrap();
    controls::compare(&oracle(), &report);
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(fs::read(d.join("heads.json")).unwrap())
        ),
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    fs::remove_dir_all(d).unwrap();
}
