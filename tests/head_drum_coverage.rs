// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, beam_dispatch, head_glyph_retrieval, head_range_search, head_registration,
    shared_sig::{Graph, Key, Phase},
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const QUARTERS: &[&str] = &[
    "NOTEHEAD_BLACK_SMALL",
    "NOTEHEAD_CROSS",
    "NOTEHEAD_DIAMOND_FILLED",
    "NOTEHEAD_TRIANGLE_DOWN_FILLED",
    "NOTEHEAD_CIRCLE_X",
];
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-drum-coverage/controls.json"
    ))
    .unwrap()
}
fn image(c: &Value) -> GrayImage {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-head-drum-coverage")
        .join(c["image"].as_str().unwrap());
    let image = GrayImage::decode(&std::fs::read(p).unwrap()).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&image.pixels)),
        c["binary_sha256"]
    );
    image
}
fn equal(a: &Value, e: &Value) {
    match (a, e) {
        (Value::Number(a), Value::Number(e)) if a.is_f64() || e.is_f64() => assert!(
            (a.as_f64().unwrap() - e.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {e}"
        ),
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(a.len(), e.len());
            for (a, e) in a.iter().zip(e) {
                equal(a, e);
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            assert_eq!(a.len(), e.len());
            for (k, e) in e {
                equal(&a[k], e);
            }
        }
        _ => assert_eq!(a, e),
    }
}
#[test]
fn original_480_selected_quarter_range_controls_three_fonts_four_sizes() {
    let g = controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["images"].as_array().unwrap().len(), 240);
    for c in g["images"].as_array().unwrap() {
        let bank = Bank::load_drum_coverage(c["family"].as_str().unwrap()).unwrap();
        let image = image(c);
        let table = audiveris_native::template_probes::distances(&image, None).unwrap();
        let pivot = serde_json::from_value(c["pivot"].clone()).unwrap();
        for mode in c["modes"].as_array().unwrap() {
            let shapes = if mode["black_relevant"] == true {
                QUARTERS
            } else {
                &[]
            };
            let actual = head_range_search::candidates_for_shapes(
                &table,
                (&bank, c["point_size"].as_u64().unwrap() as u16),
                pivot,
                &[0, -1, 1],
                shapes,
                &mut |_| Ok(false),
            )
            .unwrap();
            equal(&actual["matches"], &mode["matches"]);
            equal(&actual["rejected"], &mode["rejected"]);
        }
    }
}
#[test]
fn original_240_quarter_physical_foreground_glyphs_with_small_and_edge_bounds() {
    for c in controls()["images"].as_array().unwrap() {
        let bank = Bank::load_drum_coverage(c["family"].as_str().unwrap()).unwrap();
        let actual = head_glyph_retrieval::retrieve_at_size(
            &image(c),
            &bank,
            c["point_size"].as_u64().unwrap() as u16,
            &c["input"],
        )
        .unwrap();
        assert_eq!(
            actual
                .as_ref()
                .map(beam_dispatch::glyph_json)
                .unwrap_or(Value::Null),
            c["glyph"],
            "{}",
            c["image"]
        );
        if !c["glyph"].is_null() {
            assert_eq!(c["physical_bounds"], c["glyph"]["bounds"]);
        }
    }
}
#[test]
fn original_quarter_same_overlap_and_shared_sig_add_remove_order_contracts() {
    let grid = json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"},{"id":2,"kind":"BarlineInter"}],"relations":[{"kind":"BarGroupRelation","source":1,"target":2}]}]}});
    let mut base = Graph::default();
    base.import_grid(&grid).unwrap();
    let before = base.snapshot();
    for c in controls()["images"].as_array().unwrap() {
        let a = json!({"id":1,"kind":"HeadInter","shape":c["input"]["shape"],"bounds":c["physical_bounds"],"grade":0.8,"pitch":0.,"staff_id":1,"glyph":c["glyph"]});
        let mut b = a.clone();
        b["id"] = json!(2);
        b["glyph"] = Value::Null;
        assert_eq!(head_registration::same(&a, &b).unwrap(), c["same_twin"]);
        assert_eq!(
            head_registration::overlaps(&a, &b).unwrap(),
            c["overlaps_twin"]
        );
        assert_eq!(c["SIG_registered_in_staff_order"], true);
        assert_eq!(c["SIG_retired"], true);
        let mut graph = base.clone();
        let mut nodes = [a, b];
        graph.sync_heads(1, &nodes, &[], &[]).unwrap();
        let ids = [1, 2].map(|local| {
            graph
                .id(Key {
                    system: 1,
                    phase: Phase::Head,
                    local,
                })
                .unwrap()
        });
        assert!(ids[0] < ids[1]);
        let snapshot = graph.snapshot();
        let prior = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["origin"]["phase"] != "HEAD")
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(json!(prior), before["inters"]);
        for h in &mut nodes {
            h["removed"] = json!(true);
        }
        graph.sync_heads(1, &nodes, &[], &[]).unwrap();
        assert_eq!(graph.snapshot()["inters"], before["inters"]);
        assert_eq!(
            graph.snapshot()["tombstones"].as_array().unwrap().len(),
            before["tombstones"].as_array().unwrap().len() + 2
        );
    }
}
#[test]
fn explicit_catalog_rejects_unported_shape_duplicate_and_size_without_veto_mutation() {
    let c = &controls()["images"][0];
    let bank = Bank::load_drum_coverage("Bravura").unwrap();
    let image = image(c);
    let table = audiveris_native::template_probes::distances(&image, None).unwrap();
    let pivot = serde_json::from_value(c["pivot"].clone()).unwrap();
    let mut calls = 0;
    for shapes in [
        &["WHOLE_NOTE_SMALL"][..],
        &["NOTEHEAD_CROSS", "NOTEHEAD_CROSS"][..],
    ] {
        assert!(
            head_range_search::candidates_for_shapes(
                &table,
                (&bank, 32),
                pivot,
                &[0, -1, 1],
                shapes,
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    assert_eq!(calls, 0);
    assert!(
        head_range_search::candidates_for_shapes(
            &table,
            (&bank, 33),
            pivot,
            &[0, -1, 1],
            QUARTERS,
            &mut |_| Ok(false)
        )
        .is_err()
    );
    let empty = head_range_search::candidates_for_shapes(
        &table,
        (&bank, 32),
        pivot,
        &[0, -1, 1],
        &[],
        &mut |_| Ok(false),
    )
    .unwrap();
    assert!(empty["matches"].as_array().unwrap().is_empty());
    assert!(empty["rejected"].as_array().unwrap().is_empty());
}
