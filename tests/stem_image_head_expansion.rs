// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_expansion_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_expansion, stem_image_owned};
use serde_json::Value;
use std::{fs, process::Command};
fn image() -> GrayImage {
    GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap()
}
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-image-head-expansion/controls.json"
    ))
    .unwrap()
}
#[test]
fn whole_image_head_expansion_parts_relations_grades_and_actual_cache_match_original() {
    let owned = stem_image_owned::prepare_items(&image()).unwrap();
    let before = format!("{:?}", owned.graph);
    let registry = owned.prepared.registry.snapshot();
    let pointers = (1..=owned.prepared.registry.len())
        .map(|id| owned.prepared.registry.get(id).unwrap().glyph.runs.as_ptr())
        .collect::<Vec<_>>();
    let mut owned = stem_image_head_expansion::continue_expansion(owned).unwrap();
    controls::compare(&oracle(), &owned.prepared.report);
    assert_eq!(format!("{:?}", owned.graph), before);
    let after_registry = owned.prepared.registry.snapshot();
    for (i, p) in pointers.iter().enumerate() {
        assert_eq!(
            *p,
            owned
                .prepared
                .registry
                .get(i + 1)
                .unwrap()
                .glyph
                .runs
                .as_ptr()
        );
        assert_eq!(registry[i], after_registry[i])
    }
    // The reported creations are entries of the actual retained system cache.
    // Recreating from each canonical final glyph must be the same cached object,
    // even when the valid grade profile and image are changed after creation.
    let white = GrayImage::new(
        image().width,
        image().height,
        vec![255; image().pixels.len()],
    )
    .unwrap();
    let p = audiveris_native::stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
    let reports = owned.prepared.report["systems"].as_array().unwrap().clone();
    for sys in reports {
        let cache = owned
            .caches
            .get_mut(&sys["system"].as_u64().unwrap())
            .unwrap();
        for c in sys["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| !c["created"].is_null())
        {
            let id = c["registered_glyph_id"].as_u64().unwrap() as usize;
            let glyph = owned.prepared.registry.get(id).unwrap().glyph.clone();
            let first = cache
                .create_selected(
                    &mut owned.prepared.registry,
                    std::slice::from_ref(&glyph),
                    &white,
                    &p,
                )
                .unwrap();
            let second = cache
                .create_selected(&mut owned.prepared.registry, &[glyph], &white, &p)
                .unwrap();
            assert!(first.cache_hit && second.cache_hit);
            assert!(std::sync::Arc::ptr_eq(
                first.stem.as_ref().unwrap(),
                second.stem.as_ref().unwrap()
            ));
            assert_eq!(first.stem.unwrap().snapshot().unwrap(), c["created"]);
        }
    }
    assert_eq!(format!("{:?}", owned.graph), before);
    if let Ok(path) = std::env::var("AUDIVERIS_HEAD_EXPANSION_PROOF") {
        let proof = serde_json::json!({"status":"passed","full_private_Graph_unchanged":true,"all2335_preexisting_registry_run_allocations_retained":true,"same_actual_cache_entries_reused_with_white_image_and_changed_profile":true,"registered_physical_glyphs":owned.prepared.registry.len(),"accepted_creations":owned.prepared.report["accepted_creations"],"cache_sizes":owned.prepared.report["systems"].as_array().unwrap().iter().map(|s|s["cache_size"].clone()).collect::<Vec<_>>(),"original_scheduler_SIG_head_beam_attachment_cleaner_epilog_verified":false});
        fs::write(path, serde_json::to_vec_pretty(&proof).unwrap()).unwrap();
    }
}
#[test]
fn actual_standalone_image_cli_executes_expansion_without_an_upstream_binary() {
    let root = std::env::temp_dir().join(format!("native-head-expansion-{}", std::process::id()));
    assert!(!root.exists());
    fs::create_dir(&root).unwrap();
    let exe = root.join("audiveris-stems");
    fs::copy(env!("CARGO_BIN_EXE_audiveris-stems"), &exe).unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-grid/laura-gray.png");
    let output = Command::new(&exe)
        .arg("--head-stems")
        .arg(input)
        .arg(root.join("residual.pgm"))
        .arg(root.join("heads.json"))
        .arg(root.join("stems.json"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value =
        serde_json::from_slice(&fs::read(root.join("stems.json")).unwrap()).unwrap();
    controls::compare(&oracle(), &report);
    assert_eq!(
        report["native_HEADS_report_sha256"],
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    assert_eq!(report["head_items_plan"]["final_head_item_count"], 3076);
    fs::remove_dir_all(root).unwrap();
}
