// SPDX-License-Identifier: AGPL-3.0-or-later
#[allow(dead_code)]
#[path = "../examples/support/stem_image_head_expansion_controls.rs"]
mod controls;
use audiveris_native::{
    GrayImage, stem_foundation_purge, stem_head_foundations, stem_image_head_linking,
    stem_image_owned,
};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn compare(g: &Value) {
    let o: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-head-foundations/reduced-subset.json"
    ))
    .unwrap();
    let nodes = g["inters"].as_array().unwrap();
    let heads = nodes
        .iter()
        .filter(|n| n["kind"] == "HeadInter")
        .collect::<Vec<_>>();
    let stems = nodes
        .iter()
        .filter(|n| n["kind"] == "StemInter")
        .collect::<Vec<_>>();
    let edges = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "HeadStemRelation")
        .collect::<Vec<_>>();
    assert_eq!((heads.len(), stems.len(), edges.len()), (8, 8, 8));
    for h in o["heads"].as_array().unwrap() {
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
    for s in o["stems"].as_array().unwrap() {
        let n = stems.iter().find(|n| n["glyph"] == s["glyph"]).unwrap();
        let mut n = (*n).clone();
        n["in_sig"] = json!(true);
        controls::same(s, &n);
    }
    for e in o["head_stem_relations"].as_array().unwrap() {
        let h = heads
            .iter()
            .find(|n| n["bounds"] == e["head_bounds"] && n["shape"] == e["shape"])
            .unwrap();
        let s = stems
            .iter()
            .find(|n| n["glyph"] == e["stem"]["glyph"])
            .unwrap();
        let r = edges
            .iter()
            .find(|r| r["source"] == h["id"] && r["target"] == s["id"])
            .unwrap();
        controls::same(&e["relation"], r);
    }
}
#[test]
fn image_reduction_updates_actual_owner_and_removed_canonical_cache_states() {
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-stem-image-44/quarter-melody.png"
    ))
    .unwrap();
    let owned = stem_foundation_purge::continue_purge(
        stem_image_head_linking::continue_linking(stem_image_owned::prepare_items(&image).unwrap())
            .unwrap(),
    )
    .unwrap();
    let before = owned.graph.snapshot();
    let systems = owned.prepared.report["linked_head_stems"]["systems"].clone();
    let pointers = (1..=owned.prepared.registry.len())
        .map(|i| owned.prepared.registry.get(i).unwrap().glyph.runs.as_ptr())
        .collect::<Vec<_>>();
    let mut owned = stem_head_foundations::continue_reduction(owned).unwrap();
    let after = owned.graph.snapshot();
    compare(&after);
    let notes = owned.prepared.report["header_diatonic_pitch"]["candidates"]
        .as_array()
        .unwrap();
    let expected = [
        ("B", 4),
        ("G", 4),
        ("E", 4),
        ("G", 4),
        ("B", 4),
        ("D", 5),
        ("F", 5),
        ("D", 5),
    ];
    assert_eq!(notes.len(), expected.len());
    for (n, (s, o)) in notes.iter().zip(expected) {
        assert_eq!(n["step"], s);
        assert_eq!(n["octave"], o);
        assert_eq!(n["status"], "header_diatonic_candidate");
        assert!(
            after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["kind"] == "HeadInter"
                    && v["id"] == n["head_id"]
                    && v["bounds"] == n["bounds"])
        );
        for k in ["alter", "duration", "measure", "voice"] {
            assert_eq!(n[k], Value::Null);
        }
        assert_eq!(n["final_note"], false);
    }
    assert_eq!(
        owned.prepared.report["removed_cached_stems"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(after["tombstones"].as_array().unwrap().len(), 38);
    for n in after["inters"].as_array().unwrap() {
        let old = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"] == n["id"])
            .unwrap();
        for k in ["bounds", "shape", "grade", "glyph", "origin", "staff_id"] {
            assert_eq!(n[k], old[k]);
        }
    }
    let white = GrayImage::new(image.width, image.height, vec![255; image.pixels.len()]).unwrap();
    let params = audiveris_native::stem_checks::Parameters::new(11, 5, 0., 3).unwrap();
    let mut removed = 0;
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
                    &params,
                )
                .unwrap();
            let b = cache
                .create_selected(&mut owned.prepared.registry, &[glyph], &white, &params)
                .unwrap();
            assert!(a.cache_hit && b.cache_hit);
            let a = a.stem.unwrap();
            assert!(std::sync::Arc::ptr_eq(&a, b.stem.as_ref().unwrap()));
            let state = a.snapshot().unwrap();
            for k in [
                "id", "glyph", "grade", "median", "bounds", "impacts", "in_sig",
            ] {
                assert_eq!(state[k], s[k]);
            }
            if state["removed"] == true {
                removed += 1;
                assert!(
                    cache
                        .register(&owned.prepared.registry, &mut owned.graph, &a)
                        .is_err()
                );
            } else {
                cache
                    .register(&owned.prepared.registry, &mut owned.graph, &a)
                    .unwrap();
            }
        }
    }
    assert_eq!(removed, 2);
    assert_eq!(owned.graph.snapshot(), after);
    assert_eq!(pointers.len(), owned.prepared.registry.len());
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
    for k in [
        "heads_complete",
        "stems_complete",
        "musicxml_complete",
        "epilog_complete",
        "automatic_raster_pipeline_complete",
    ] {
        assert_eq!(owned.prepared.report[k], false);
    }
    let music = audiveris_native::music_measures::continue_measures(owned).unwrap();
    let layout = music.layout();
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-music-measures/quarter-observed.json"
    ))
    .unwrap();
    assert_eq!(
        layout["measures"].as_array().unwrap().len(),
        oracle["systems"][0]["measure_stacks"].as_u64().unwrap() as usize
    );
    let graph = music.owner.graph.snapshot();
    for old in after["inters"].as_array().unwrap() {
        assert_eq!(
            graph["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == old["id"])
                .unwrap(),
            old
        );
    }
    assert_eq!(graph["tombstones"], after["tombstones"]);
    assert_eq!(layout["head_chords"].as_array().unwrap().len(), 8);
    for note in oracle["systems"][0]["notes"].as_array().unwrap() {
        let head = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadInter" && n["bounds"] == note["bounds"])
            .unwrap();
        let chord = layout["head_chords"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["head_ids"].as_array().unwrap().contains(&head["id"]))
            .unwrap();
        assert_eq!(chord["measure_index"], note["measure_index"]);
        let measure = layout["measures"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["index"] == note["measure_index"])
            .unwrap();
        assert_eq!(measure["left"], note["measure_left"]);
        assert_eq!(measure["right"], note["measure_right"]);
        assert_eq!(chord["duration"], Value::Null);
        assert!(
            graph["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "Containment"
                    && e["source"] == chord["id"]
                    && e["target"] == head["id"])
        );
    }
    assert_eq!(music.owner.prepared.report["musicxml_complete"], false);
    let music = audiveris_native::music_symbols::continue_symbols(music).unwrap();
    let symbols = &music.owner.prepared.report;
    let filter: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-music-measures/quarter-symbols-filter.json"
    ))
    .unwrap();
    assert_eq!(
        symbols["symbol_filter_PGM_SHA256"],
        filter["prototype_filter_PGM_SHA256"]
    );
    let mut native = symbols["symbol_glyphs"].as_array().unwrap().clone();
    let mut expected = filter["glyphs"].as_array().unwrap().clone();
    native.sort_by_key(|n| serde_json::to_string(n).unwrap());
    expected.sort_by_key(|n| serde_json::to_string(n).unwrap());
    assert_eq!(native, expected);
    let accepted = symbols["symbols"].as_array().unwrap();
    assert_eq!(accepted.len(), 1);
    for field in ["kind", "shape", "bounds"] {
        assert_eq!(
            accepted[0][field],
            oracle["systems"][0]["symbols"][0][field]
        );
    }
    assert!(
        (accepted[0]["grade"].as_f64().unwrap()
            - oracle["systems"][0]["symbols"][0]["grade"]
                .as_f64()
                .unwrap())
        .abs()
            < 1e-14
    );
    assert_eq!(accepted[0]["numerator"], 4);
    assert_eq!(accepted[0]["denominator"], 4);
    for old in graph["inters"].as_array().unwrap() {
        assert_eq!(
            music.owner.graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == old["id"])
                .unwrap(),
            old
        );
    }
    assert_eq!(symbols["musicxml_complete"], false);
    let export = audiveris_native::music_xml::continue_export(music).unwrap();
    let rhythm: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-music-measures/quarter-rhythm-observed.json"
    ))
    .unwrap();
    let events = export.owner.owner.prepared.report["events"]
        .as_array()
        .unwrap();
    assert_eq!(events.len(), 8);
    for n in rhythm["systems"][0]["notes"].as_array().unwrap() {
        let e = events.iter().find(|e| e["bounds"] == n["bounds"]).unwrap();
        for field in [
            "step",
            "octave",
            "duration",
            "time_offset",
            "voice",
            "dots",
            "measure_index",
        ] {
            assert_eq!(e[field], n[field]);
        }
        assert_eq!(e["rest"], false);
        assert_eq!(e["alter"], 0);
    }
    assert_eq!(export.xml.matches("<note>").count(), 8);
    assert_eq!(export.xml.matches("<measure number=").count(), 2);
    assert!(
        export.xml.contains(
            "<divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time>"
        )
    );
    assert_eq!(
        export.owner.owner.prepared.report["original_XML_output_reused"],
        false
    );
    assert_eq!(
        export.owner.owner.prepared.report["limited_scope_practical_validation_complete"],
        false
    );
    assert_eq!(
        export.owner.owner.prepared.report["musicxml_complete"],
        false
    );
}
#[test]
fn image_cli_reduces_candidates_and_refuses_existing_outputs() {
    let root = std::env::temp_dir().join(format!("native-head-foundations-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-stem-image-44/quarter-melody.png");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audiveris-stems"))
            .arg("--head-foundations")
            .arg(&input)
            .arg(root.join("residual.pgm"))
            .arg(root.join("heads.json"))
            .arg(root.join("stems.json"))
            .output()
            .unwrap()
    };
    let r = run();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let data = fs::read(root.join("stems.json")).unwrap();
    let report: Value = serde_json::from_slice(&data).unwrap();
    compare(&report["shared_sig"]);
    assert_eq!(report["actual_live_Graph_and_cache_updated"], true);
    assert_eq!(
        report["header_diatonic_pitch"]["candidates"][0]["step"],
        "B"
    );
    assert_eq!(
        report["header_diatonic_pitch"]["key_and_individual_accidentals_checked"],
        false
    );
    assert_eq!(report["musicxml_complete"], false);
    assert!(!run().status.success());
    assert_eq!(fs::read(root.join("stems.json")).unwrap(), data);
    fs::remove_dir_all(root).unwrap();
}
