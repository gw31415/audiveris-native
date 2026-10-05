// SPDX-License-Identifier: AGPL-3.0-or-later
#[allow(dead_code)]
#[path = "../examples/support/stem_image_head_expansion_controls.rs"]
mod controls;
use audiveris_native::{
    GrayImage, stem_foundation_purge, stem_image_head_linking, stem_image_owned,
};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn compare(g: &Value) {
    let original: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-foundation-purge/first-purge-subset.json"
    ))
    .unwrap();
    let nodes = g["inters"].as_array().unwrap();
    let heads = nodes
        .iter()
        .filter(|v| v["kind"] == "HeadInter")
        .collect::<Vec<_>>();
    let stems = nodes
        .iter()
        .filter(|v| v["kind"] == "StemInter")
        .collect::<Vec<_>>();
    let edges = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "HeadStemRelation")
        .collect::<Vec<_>>();
    assert_eq!((heads.len(), stems.len(), edges.len()), (30, 10, 22));
    for h in original["heads"].as_array().unwrap() {
        let n = heads
            .iter()
            .find(|n| n["bounds"] == h["bounds"] && n["shape"] == h["shape"])
            .unwrap();
        assert_eq!(n["abnormal"], h["abnormal"]);
        assert_eq!(
            format!("{:.3}", n["grade"].as_f64().unwrap())
                .parse::<f64>()
                .unwrap(),
            h["grade"].as_f64().unwrap()
        );
    }
    for s in original["stems"].as_array().unwrap() {
        let n = stems.iter().find(|n| n["glyph"] == s["glyph"]).unwrap();
        let mut n = (*n).clone();
        n["in_sig"] = json!(true);
        controls::same(s, &n);
    }
    for e in original["head_stem_relations"].as_array().unwrap() {
        let h = heads
            .iter()
            .find(|h| h["bounds"] == e["head_bounds"] && h["shape"] == e["shape"])
            .unwrap();
        let s = stems
            .iter()
            .find(|s| s["glyph"] == e["stem"]["glyph"])
            .unwrap();
        let n = edges
            .iter()
            .find(|n| n["source"] == h["id"] && n["target"] == s["id"])
            .unwrap();
        controls::same(&e["relation"], n);
    }
}
#[test]
fn automatic_image_purge_updates_actual_graph_retaining_cache_registry_and_prior_ids() {
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-stem-image-44/quarter-melody.png"
    ))
    .unwrap();
    let owned =
        stem_image_head_linking::continue_linking(stem_image_owned::prepare_items(&image).unwrap())
            .unwrap();
    let before = owned.graph.snapshot();
    let systems = owned.prepared.report["systems"].clone();
    let pointers = (1..=owned.prepared.registry.len())
        .map(|i| owned.prepared.registry.get(i).unwrap().glyph.runs.as_ptr())
        .collect::<Vec<_>>();
    let mut owned = stem_foundation_purge::continue_purge(owned).unwrap();
    let after = owned.graph.snapshot();
    compare(&after);
    assert_eq!(owned.prepared.report["shared_sig"], after);
    assert_eq!(
        owned.prepared.report["removed_native_vertex_ids"]
            .as_array()
            .unwrap()
            .len(),
        14
    );
    for n in after["inters"].as_array().unwrap() {
        let old = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == n["id"])
            .unwrap();
        for key in [
            "bounds",
            "shape",
            "grade",
            "glyph",
            "origin",
            "staff_id",
            "contextual_grade",
        ] {
            assert_eq!(n[key], old[key]);
        }
    }
    let white = GrayImage::new(image.width, image.height, vec![255; image.pixels.len()]).unwrap();
    let parameters = audiveris_native::stem_checks::Parameters::new(11, 5, 0., 3).unwrap();
    for sys in systems.as_array().unwrap() {
        let cache = owned
            .caches
            .get_mut(&sys["system"].as_u64().unwrap())
            .unwrap();
        for s in sys["glyphs"].as_array().unwrap() {
            let glyph = audiveris_native::beam_dispatch::glyph_from_json(&s["glyph"]).unwrap();
            let a = cache
                .create_selected(
                    &mut owned.prepared.registry,
                    std::slice::from_ref(&glyph),
                    &white,
                    &parameters,
                )
                .unwrap();
            let b = cache
                .create_selected(&mut owned.prepared.registry, &[glyph], &white, &parameters)
                .unwrap();
            assert!(a.cache_hit && b.cache_hit);
            let a = a.stem.unwrap();
            assert!(std::sync::Arc::ptr_eq(&a, b.stem.as_ref().unwrap()));
            assert_eq!(a.snapshot().unwrap(), *s);
            cache
                .register(&owned.prepared.registry, &mut owned.graph, &a)
                .unwrap();
        }
    }
    assert_eq!(owned.graph.snapshot(), after);
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
    }
    for key in [
        "heads_complete",
        "stems_complete",
        "musicxml_complete",
        "epilog_complete",
        "automatic_raster_pipeline_complete",
    ] {
        assert_eq!(owned.prepared.report[key], false);
    }
    owned.prepared.report["schema"] = json!("stale-owner");
    assert!(stem_foundation_purge::continue_purge(owned).is_err());
}
#[test]
fn image_cli_purge_uses_actual_owner_and_refuses_existing_outputs() {
    let root = std::env::temp_dir().join(format!("native-weak-foundations-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-stem-image-44/quarter-melody.png");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
            .arg("--weak-foundations")
            .arg(&input)
            .arg(root.join("residual.pgm"))
            .arg(root.join("heads.json"))
            .arg(root.join("stems.json"))
            .output()
            .unwrap()
    };
    let r = run();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let bytes = fs::read(root.join("stems.json")).unwrap();
    let n: Value = serde_json::from_slice(&bytes).unwrap();
    compare(&n["shared_sig"]);
    assert_eq!(n["actual_live_Graph_updated"], true);
    assert_eq!(n["musicxml_complete"], false);
    assert!(!run().status.success());
    assert_eq!(bytes, fs::read(root.join("stems.json")).unwrap());
    fs::remove_dir_all(root).unwrap();
}
