// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_chunks_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_chunks};
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
        "../fixtures/official-stem-image-head-chunks/controls.json"
    ))
    .unwrap()
}

#[test]
fn image_chunk_physical_sections_glyphs_and_same_owned_registry_match_original() {
    use audiveris_native::beam_dispatch;
    let p = page();
    let before = p.clone();
    let owned = stem_image_head_chunks::prepare(&image(), &p).unwrap();
    controls::compare(&oracle(), &owned.report);
    let old = &owned.report["head_parts_plan"]["beam_generation_plan"]["beam_target_plan"]["head_linker_plan"]
        ["source_plan"]["stump_plan"]["physical_registry"];
    let snapshot = owned.snapshot_registry();
    assert_eq!(
        &snapshot.as_array().unwrap()[..1566],
        old.as_array().unwrap().as_slice()
    );
    assert_eq!(owned.registry.len(), 2335);
    let mut rejected_registered = 0;
    for s in owned.report["systems"].as_array().unwrap() {
        for c in s["cases"].as_array().unwrap() {
            let l = &c["chunk_lookup"];
            for (id, g) in l["registered_native_ids"]
                .as_array()
                .unwrap()
                .iter()
                .zip(l["registered_glyphs"].as_array().unwrap())
            {
                assert_eq!(
                    beam_dispatch::glyph_json(
                        &owned
                            .registry
                            .get(id.as_u64().unwrap() as usize)
                            .unwrap()
                            .glyph
                    ),
                    *g
                );
                if !l["retained_native_ids"].as_array().unwrap().contains(id) {
                    rejected_registered += 1;
                }
            }
        }
    }
    assert!(rejected_registered > 0);
    assert_eq!(p, before);
}
#[test]
fn malformed_section_membership_and_bounds_refuse_without_changing_input() {
    let im = image();
    let mut p = page();
    p["grid_ir"]["hlag"][0]["bounds"][2] = json!(900);
    let before = p.clone();
    assert!(
        stem_image_head_chunks::prepare(&im, &p)
            .err()
            .unwrap()
            .contains("bounds")
    );
    assert_eq!(p, before);
    p = page();
    let ids = p["grid_ir"]["systems"][0]["horizontal_section_ids"]
        .as_array_mut()
        .unwrap();
    ids.push(ids[0].clone());
    assert!(
        stem_image_head_chunks::prepare(&im, &p)
            .err()
            .unwrap()
            .contains("duplicate")
    );
}
#[test]
fn empty_heads_and_unsupported_skew_staff_remain_explicit() {
    let im = image();
    let original = page();
    let mut p = original.clone();
    p["line_completion"]["global_slope"] = json!(0.01);
    assert!(
        stem_image_head_chunks::prepare(&im, &p)
            .err()
            .unwrap()
            .contains("skew")
    );
    p = original.clone();
    p["grid_ir"]["staffs"][5]["drum"] = json!(true);
    assert!(
        stem_image_head_chunks::prepare(&im, &p)
            .err()
            .unwrap()
            .contains("staff")
    );
    p = original.clone();
    for s in p["head_registration"]["systems"].as_array_mut().unwrap() {
        s["heads"] = json!([]);
    }
    let r = stem_image_head_chunks::prepare(&im, &p).unwrap();
    assert_eq!(r.report["corner_count"], 0);
    assert_eq!(r.report["raw_chunk_occurrences"], 0);
    assert_eq!(r.report["new_chunk_registrations"], 0);
    assert_eq!(r.report["stems_complete"], false);
    assert_eq!(page(), original);
}
#[test]
fn actual_image_cli_retrieves_and_registers_chunks_without_condition_requests() {
    use sha2::{Digest, Sha256};
    let d = std::env::temp_dir().join(format!("stem-image-head-chunks-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
        .arg("--head-chunks")
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
