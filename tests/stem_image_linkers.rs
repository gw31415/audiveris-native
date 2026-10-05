// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_linkers_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_linkers};
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
        "../fixtures/official-stem-image-linkers/controls.json"
    ))
    .unwrap()
}
#[test]
fn actual_image_other_head_linkers_match_before_original_linking() {
    let p = page();
    let before = p.clone();
    let owned = stem_image_linkers::prepare(&image(), &p).unwrap();
    controls::compare(&oracle(), &owned.report);
    assert_eq!(
        owned.snapshot_registry(),
        owned.report["source_plan"]["stump_plan"]["physical_registry"]
    );
    for sys in owned.report["systems"].as_array().unwrap() {
        for c in sys["cases"].as_array().unwrap() {
            for target in c["selected_other_head_linkers"].as_array().unwrap() {
                assert_eq!(target["head_origin"]["phase"], "HEAD");
                assert_eq!(target["head_origin"]["system"], sys["system"]);
                assert_ne!(target["head_id"], c["head"]["head_id"]);
            }
        }
    }
    assert_eq!(p, before);
}
#[test]
fn empty_heads_and_late_unsupported_image_staff_or_skew_preserve_input() {
    let original = page();
    let im = image();
    let mut p = original.clone();
    p["grid_ir"]["staffs"][5]["drum"] = json!(true);
    assert!(
        stem_image_linkers::analyze(&im, &p)
            .unwrap_err()
            .contains("staff")
    );
    p = original.clone();
    p["line_completion"]["global_slope"] = json!(0.01);
    assert!(
        stem_image_linkers::analyze(&im, &p)
            .unwrap_err()
            .contains("skew")
    );
    p = original.clone();
    for s in p["head_registration"]["systems"].as_array_mut().unwrap() {
        s["heads"] = json!([]);
    }
    let owned = stem_image_linkers::prepare(&im, &p).unwrap();
    assert_eq!(owned.report["selected_other_head_linker_occurrences"], 0);
    assert_eq!(owned.report["stems_complete"], false);
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_derives_other_head_targets_without_corner_requests() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-sources-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--head-linkers")
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-grid/laura-gray.png"),
        )
        .arg(d.join("residual.pgm"))
        .arg(d.join("heads.json"))
        .arg(d.join("stems.json"))
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let heads = fs::read(d.join("heads.json")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&heads)),
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    let n: Value = serde_json::from_slice(&fs::read(d.join("stems.json")).unwrap()).unwrap();
    controls::compare(&oracle(), &n);
    fs::remove_dir_all(d).unwrap();
}

#[test]
fn live_head_origin_validation_refuses_stale_snapshot_before_targets() {
    let mut p = page();
    let before = p.clone();
    let v = p["head_registration"]["shared_sig"]["inters"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|v| v["kind"] == "HeadInter")
        .unwrap();
    v["origin"]["phase"] = json!("STEM");
    assert!(
        stem_image_linkers::prepare(&image(), &p)
            .err()
            .unwrap()
            .contains("origin")
    );
    assert_ne!(p, before);
    assert_eq!(page(), before);
}
