// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_items_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_items};
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
        "../fixtures/official-stem-image-head-items/controls.json"
    ))
    .unwrap()
}

#[test]
fn actual_image_head_builder_items_and_lengths_match_original_with_owned_glyph_ids() {
    use audiveris_native::beam_dispatch;
    let p = page();
    let before = p.clone();
    let owned = stem_image_head_items::prepare(&image(), &p).unwrap();
    controls::compare(&oracle(), &owned.report);
    assert_eq!(
        owned.snapshot_registry(),
        owned.report["head_chunks_plan"]["physical_registry"]
    );
    assert_eq!(owned.registry.len(), 2335);
    let mut gaps = 0;
    let mut truncations = 0;
    for s in owned.report["systems"].as_array().unwrap() {
        for c in s["cases"].as_array().unwrap() {
            truncations += usize::from(c["builder_items"]["truncated"] == true);
            for it in c["builder_items"]["items"].as_array().unwrap() {
                if it["gap"] == true {
                    gaps += 1;
                    assert!(it["glyph_id"].is_null());
                    assert!(it["source_token"].is_null());
                } else if !it["glyph"].is_null() {
                    let id = it["glyph_id"].as_u64().unwrap() as usize;
                    assert_eq!(
                        beam_dispatch::glyph_json(&owned.registry.get(id).unwrap().glyph),
                        it["glyph"]
                    );
                }
            }
        }
    }
    assert_eq!((gaps, truncations), (92, 1090));
    assert_eq!(p, before);
}
#[test]
fn empty_heads_and_late_unsupported_skew_staff_preserve_input() {
    let im = image();
    let original = page();
    let mut p = original.clone();
    p["line_completion"]["global_slope"] = json!(0.01);
    assert!(
        stem_image_head_items::prepare(&im, &p)
            .err()
            .unwrap()
            .contains("skew")
    );
    p = original.clone();
    p["grid_ir"]["staffs"][5]["drum"] = json!(true);
    assert!(
        stem_image_head_items::prepare(&im, &p)
            .err()
            .unwrap()
            .contains("staff")
    );
    p = original.clone();
    for s in p["head_registration"]["systems"].as_array_mut().unwrap() {
        s["heads"] = json!([]);
    }
    let r = stem_image_head_items::prepare(&im, &p).unwrap();
    assert_eq!(r.report["corner_count"], 0);
    assert_eq!(r.report["final_head_item_count"], 0);
    assert_eq!(r.report["gap_item_count"], 0);
    assert_eq!(r.report["stems_complete"], false);
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_constructs_final_items_gaps_and_lengths_without_condition_requests() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-head-items-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--head-items")
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
