// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_items_controls.rs"]
mod controls;
use audiveris_native::{
    GrayImage, page_runtime,
    shared_sig::{Graph, Key},
    stem_image_owned,
};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

fn image() -> GrayImage {
    GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap()
}
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-image-head-items/controls.json"
    ))
    .unwrap()
}
#[test]
fn actual_upstream_registry_run_storage_graph_history_and_heads_survive_image_items() {
    use sha2::{Digest, Sha256};
    let image = image();
    let upstream = page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    let mut bytes = serde_json::to_vec(&upstream.report).unwrap();
    bytes.push(b'\n');
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9"
    );
    let registry = upstream.registry.as_ref().unwrap();
    assert_eq!(registry.len(), 1220);
    // Vec<Entry> can grow as stumps/chunks register; each glyph's original run
    // allocation must survive. Replay/clone would allocate different buffers.
    let pointers: Vec<_> = (1..=registry.len())
        .map(|id| registry.get(id).unwrap().glyph.runs.as_ptr())
        .collect();
    let snapshot = registry.snapshot();
    let full_private_graph_before = format!("{:?}", upstream.graph);
    let graph_snapshot = upstream.graph.snapshot();
    let owned = stem_image_owned::continue_items(&image, upstream).unwrap();
    controls::compare(&oracle(), &owned.prepared.report);
    assert_eq!(owned.prepared.registry.len(), 2335);
    let after_registry = owned.prepared.registry.snapshot();
    for (i, ptr) in pointers.iter().enumerate() {
        assert_eq!(
            *ptr,
            owned
                .prepared
                .registry
                .get(i + 1)
                .unwrap()
                .glyph
                .runs
                .as_ptr()
        );
        assert_eq!(snapshot[i], after_registry[i]);
    }
    assert_eq!(format!("{:?}", owned.graph), full_private_graph_before);
    assert_eq!(owned.graph.snapshot(), graph_snapshot);
    assert_eq!(owned.caches.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
    let heads: Vec<_> = graph_snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["kind"] == "HeadInter")
        .collect();
    assert_eq!(heads.len(), 352);
    for head in heads {
        let key: Key = serde_json::from_value(head["origin"].clone()).unwrap();
        assert_eq!(
            owned.graph.id(key),
            Some(head["id"].as_u64().unwrap() as usize)
        );
    }
    assert_eq!(owned.prepared.report["page_cache_sig_complete"], false);
    assert_eq!(owned.prepared.report["stems_complete"], false);
}
#[test]
fn mismatched_live_graph_or_registry_refuses_before_stems() {
    let image = image();
    let mut upstream = page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    upstream.graph = Graph::default();
    assert!(
        stem_image_owned::continue_items(&image, upstream)
            .err()
            .unwrap()
            .to_string()
            .contains("Graph does not match")
    );
    let mut upstream = page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    upstream.registry = Some(Default::default());
    assert!(
        stem_image_owned::continue_items(&image, upstream)
            .err()
            .unwrap()
            .to_string()
            .contains("physical registry does not match")
    );
    assert!(page_runtime::analyze(&image, "unsupported", None, None).is_err());
    let invalid = GrayImage {
        width: 1,
        height: 1,
        pixels: vec![],
    };
    assert!(page_runtime::analyze(&invalid, "page-glyphs", None, None).is_err());
}
#[test]
fn actual_head_items_cli_runs_in_process_without_sibling_upstream_binary() {
    let d = std::env::temp_dir().join(format!("stem-image-owned-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    fs::copy(
        env!("CARGO_BIN_EXE_audiveris-stems"),
        d.join("audiveris-stems"),
    )
    .unwrap();
    assert!(!d.join("audiveris-native").exists());
    let o = Command::new(d.join("audiveris-stems"))
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
    let report = serde_json::from_slice(&fs::read(d.join("stems.json")).unwrap()).unwrap();
    controls::compare(&oracle(), &report);
    fs::remove_dir_all(d).unwrap();
}
