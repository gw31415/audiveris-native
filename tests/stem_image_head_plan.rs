// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_plan_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_plan};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};
fn image() -> GrayImage {
    GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap()
}
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-image-head-plan/controls.json"
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
fn image_heads_parts_and_all_corners_derive_original_targets_without_oracle_inputs() {
    let page = page();
    let before = serde_json::to_vec(&page).unwrap();
    let n = stem_image_head_plan::analyze(&image(), &page).unwrap();
    controls::compare(&oracle(), &n);
    assert_eq!(serde_json::to_vec(&page).unwrap(), before);
}
#[test]
fn late_bad_staff_skew_font_membership_and_empty_heads_preserve_source_state() {
    let mut p = page();
    let original = p.clone();
    p["line_completion"]["global_slope"] = json!(0.001);
    assert!(
        stem_image_head_plan::analyze(&image(), &p)
            .unwrap_err()
            .contains("skew")
    );
    p = original.clone();
    p["grid_ir"]["staffs"][5]["specific_interline"] = json!(7);
    assert!(
        stem_image_head_plan::analyze(&image(), &p)
            .unwrap_err()
            .contains("staff")
    );
    p = original.clone();
    p["beam_prolog"]["head_scale"]["music_font"]["name"] = json!("Leland");
    assert!(
        stem_image_head_plan::analyze(&image(), &p)
            .unwrap_err()
            .contains("font/size")
    );
    p = original.clone();
    p["grid_ir"]["systems"][1]["parts"][1]["staff_ids"] = json!([5]);
    assert!(
        stem_image_head_plan::analyze(&image(), &p)
            .unwrap_err()
            .contains("part coverage")
    );
    p = original.clone();
    for s in p["head_registration"]["systems"].as_array_mut().unwrap() {
        s["heads"] = json!([]);
    }
    let empty = stem_image_head_plan::analyze(&image(), &p).unwrap();
    assert_eq!(empty["corner_count"], 0);
    assert_eq!(empty["stems_complete"], false);
    assert_eq!(original, page());
}
#[test]
fn actual_image_cli_preserves_entire_old_report_and_refuses_existing_outputs() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-plan-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let image =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-grid/laura-gray.png");
    let residual = d.join("residual.pgm");
    let heads = d.join("heads.json");
    let stems = d.join("stems.json");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
            .arg(&image)
            .arg(&residual)
            .arg(&heads)
            .arg(&stems)
            .output()
            .unwrap()
    };
    let result = run();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = fs::read(&heads).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    let s = fs::read(&stems).unwrap();
    let n: Value = serde_json::from_slice(&s).unwrap();
    controls::compare(&oracle(), &n);
    assert_eq!(
        n["native_HEADS_report_sha256"],
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    assert!(!run().status.success());
    assert_eq!(fs::read(&stems).unwrap(), s);
    assert_eq!(fs::read(&heads).unwrap(), bytes);
    fs::remove_dir_all(d).unwrap();
}
