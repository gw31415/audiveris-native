// SPDX-License-Identifier: AGPL-3.0-or-later
#[allow(dead_code)] // Reuse the existing recursive oracle comparator only.
#[path = "../examples/support/stem_image_head_expansion_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_linking, stem_image_owned};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn image() -> GrayImage {
    GrayImage::decode(include_bytes!(
        "../fixtures/official-stem-image-44/quarter-melody.png"
    ))
    .unwrap()
}
fn compare(n: &Value) {
    let o: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-retry-profiles/linking-controls.json"
    ))
    .unwrap();
    let o = &o["systems"][0]["linked"];
    let g = &n["shared_sig"];
    let vertices = g["inters"].as_array().unwrap();
    let heads = vertices
        .iter()
        .filter(|v| v["kind"] == "HeadInter")
        .collect::<Vec<_>>();
    let stems = vertices
        .iter()
        .filter(|v| v["kind"] == "StemInter")
        .collect::<Vec<_>>();
    let edges = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "HeadStemRelation")
        .collect::<Vec<_>>();
    assert_eq!((heads.len(), stems.len(), edges.len()), (43, 10, 25));
    for h in o["heads"].as_array().unwrap() {
        let target = heads
            .iter()
            .find(|n| n["bounds"] == h["bounds"] && n["shape"] == h["shape"])
            .unwrap();
        assert_eq!(target["abnormal"], h["abnormal"]);
        let raw = target["grade"].as_f64().unwrap();
        assert_eq!(
            format!("{raw:.3}").parse::<f64>().unwrap(),
            h["grade"].as_f64().unwrap()
        );
    }
    for s in o["stems"].as_array().unwrap() {
        let target = stems.iter().find(|n| n["glyph"] == s["glyph"]).unwrap();
        let mut native = (*target).clone();
        native["in_sig"] = json!(true);
        controls::same(s, &native);
    }
    for e in o["head_stem_relations"].as_array().unwrap() {
        let head = heads
            .iter()
            .find(|n| n["bounds"] == e["head_bounds"] && n["shape"] == e["shape"])
            .unwrap();
        let stem = stems
            .iter()
            .find(|n| n["glyph"] == e["stem"]["glyph"])
            .unwrap();
        let rel = edges
            .iter()
            .find(|r| r["source"] == head["id"] && r["target"] == stem["id"])
            .unwrap();
        controls::same(&e["relation"], rel);
    }
    for k in [
        "heads_complete",
        "stems_complete",
        "musicxml_complete",
        "epilog_complete",
        "automatic_raster_pipeline_complete",
    ] {
        assert_eq!(n[k], false)
    }
}
#[test]
fn automatic_image_head_linking_retains_actual_registry_graph_and_cache_identity() {
    let owned = stem_image_owned::prepare_items(&image()).unwrap();
    let before = owned.graph.snapshot();
    let pointers = (1..=owned.prepared.registry.len())
        .map(|i| owned.prepared.registry.get(i).unwrap().glyph.runs.as_ptr())
        .collect::<Vec<_>>();
    let mut owned = stem_image_head_linking::continue_linking(owned).unwrap();
    compare(&owned.prepared.report);
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
        )
    }
    let after = owned.graph.snapshot();
    for old in before["inters"].as_array().unwrap() {
        let current = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == old["id"])
            .unwrap();
        if old["kind"] != "HeadInter" {
            assert_eq!(old, current)
        } else {
            for key in ["bounds", "shape", "grade", "glyph", "origin", "staff_id"] {
                assert_eq!(old[key], current[key])
            }
        }
    }
    let contextual = after["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| matches!(v["kind"].as_str(), Some("HeadInter" | "StemInter")))
        .collect::<Vec<_>>();
    assert_eq!(contextual.len(), 53);
    assert!(contextual.iter().all(|v| {
        v["contextual_grade"]
            .as_f64()
            .is_some_and(|c| c.is_finite() && c >= v["grade"].as_f64().unwrap())
    }));
    let systems = owned.prepared.report["systems"].as_array().unwrap().clone();
    let white = GrayImage::new(
        image().width,
        image().height,
        vec![255; image().pixels.len()],
    )
    .unwrap();
    let params = audiveris_native::stem_checks::Parameters::new(11, 5, 0., 3).unwrap();
    for sys in systems {
        let cache = owned
            .caches
            .get_mut(&sys["system"].as_u64().unwrap())
            .unwrap();
        for stem in sys["glyphs"].as_array().unwrap() {
            let glyph = audiveris_native::beam_dispatch::glyph_from_json(&stem["glyph"]).unwrap();
            let a = cache
                .create_selected(
                    &mut owned.prepared.registry,
                    std::slice::from_ref(&glyph),
                    &white,
                    &params,
                )
                .unwrap();
            let b = cache
                .create_selected(&mut owned.prepared.registry, &[glyph], &white, &params)
                .unwrap();
            assert!(a.cache_hit && b.cache_hit);
            let a = a.stem.unwrap();
            assert!(std::sync::Arc::ptr_eq(&a, b.stem.as_ref().unwrap()));
            assert_eq!(a.snapshot().unwrap(), *stem);
            cache
                .register(&owned.prepared.registry, &mut owned.graph, &a)
                .unwrap();
        }
    }
    assert_eq!(owned.graph.snapshot(), after);
}
#[test]
fn standalone_image_cli_links_actual_owner_and_refuses_existing_outputs() {
    let root = std::env::temp_dir().join(format!("native-image-head-links-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-stem-image-44/quarter-melody.png");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
            .arg("--linked-head-stems")
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
    compare(&serde_json::from_slice(&bytes).unwrap());
    let heads = fs::read(root.join("heads.json")).unwrap();
    assert!(!run().status.success());
    assert_eq!(bytes, fs::read(root.join("stems.json")).unwrap());
    assert_eq!(heads, fs::read(root.join("heads.json")).unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn supplied_whole_breve_head_boundary_retains_stemless_nodes_without_scheduling_corners() {
    // This injects a boundary into an actual image owner. It does not certify
    // automatic whole/breve page recognition or the full 24-shape pipeline.
    let mut owned = stem_image_owned::prepare_items(&image()).unwrap();
    let original = owned.graph.snapshot();
    let inters = original["inters"].as_array().unwrap();
    let heads_original = inters
        .iter()
        .filter(|v| v["kind"] == "HeadInter")
        .collect::<Vec<_>>();
    let globals = heads_original
        .iter()
        .map(|h| {
            (
                h["id"].as_u64().unwrap(),
                h["origin"]["local"].as_u64().unwrap(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut heads = heads_original
        .iter()
        .map(|h| {
            let mut v = (*h).clone();
            v["id"] = v["origin"]["local"].clone();
            v
        })
        .collect::<Vec<_>>();
    let max = heads
        .iter()
        .map(|h| h["id"].as_u64().unwrap())
        .max()
        .unwrap();
    for (i, shape) in ["WHOLE_NOTE", "BREVE"].into_iter().enumerate() {
        let mut h = heads[0].clone();
        h["id"] = serde_json::json!(max + 1 + i as u64);
        h["shape"] = serde_json::json!(shape);
        h["bounds"] = serde_json::json!([1200 + i as i32 * 30, 220, 18, 10]);
        h["glyph"] = Value::Null;
        h["grade"] = serde_json::json!(0.99);
        h["abnormal"] = serde_json::json!(false);
        heads.push(h);
    }
    let exclusions = original["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "Exclusion")
        .map(|r| {
            let mut r = r.clone();
            r["source"] = serde_json::json!(globals[&r["source"].as_u64().unwrap()]);
            r["target"] = serde_json::json!(globals[&r["target"].as_u64().unwrap()]);
            r
        })
        .collect::<Vec<_>>();
    owned.graph.sync_heads(1, &heads, &exclusions, &[]).unwrap();
    let owned = audiveris_native::stem_image_head_linking::continue_linking(owned).unwrap();
    let snapshot = owned.graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    assert_eq!(
        nodes.iter().filter(|v| v["kind"] == "HeadInter").count(),
        45
    );
    assert_eq!(
        nodes.iter().filter(|v| v["kind"] == "StemInter").count(),
        10
    );
    assert_eq!(
        snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "HeadStemRelation")
            .count(),
        25
    );
    for shape in ["WHOLE_NOTE", "BREVE"] {
        let h = nodes.iter().find(|v| v["shape"] == shape).unwrap();
        assert_eq!(h["grade"], serde_json::json!(0.99));
        assert_eq!(h["contextual_grade"], h["grade"]);
        assert_eq!(h["abnormal"], false);
        assert!(
            !snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["kind"] == "HeadStemRelation" && r["source"] == h["id"])
        );
    }
    assert_eq!(owned.prepared.report["musicxml_complete"], false);
}
