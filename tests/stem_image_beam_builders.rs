// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    stem_builder, stem_checks, stem_image_head_linking, stem_image_owned,
};
use serde_json::Value;
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beams/builder-observed.json"
    ))
    .unwrap()
}
#[test]
fn original_beam_side_artificial_grade_only_after_normal_rejection_and_cache_identity() {
    let o = oracle();
    let im = GrayImage::new(1280, 420, vec![255; 1280 * 420]).unwrap();
    let mut registry = Registry::default();
    let mut cache = stem_builder::Cache::new(1).unwrap();
    for c in o["white_raster_create_controls"].as_array().unwrap() {
        let g = beam_dispatch::glyph_from_json(&c["glyph"]).unwrap();
        let p =
            stem_checks::Parameters::new(11, 3, 0., c["profile"].as_u64().unwrap() as u8).unwrap();
        let first = cache
            .create_selected(&mut registry, std::slice::from_ref(&g), &im, &p)
            .unwrap();
        if c["stem"].is_null() {
            assert!(first.stem.is_none())
        } else {
            let a = first.stem.unwrap();
            assert_eq!(a.record["grade"], c["stem"]["grade"]);
            assert!(a.record["impacts"].is_null());
            let p = stem_checks::Parameters::new(11, 3, 0., 1).unwrap();
            let b = cache
                .create_selected(&mut registry, &[g], &im, &p)
                .unwrap()
                .stem
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&a, &b));
        }
    }
}
#[test]
fn original_image_beam_origin_items_expand_relations_and_selected_cache_glyphs() {
    let o = oracle();
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-image-musicxml-beams/beam-eighth-quarter.png"
    ))
    .unwrap();
    let owner =
        stem_image_head_linking::continue_linking(stem_image_owned::prepare_items(&image).unwrap())
            .unwrap();
    let rows = owner.prepared.report["systems"][0]["beam_origin_builders"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 2);
    for (n, c) in rows.iter().zip(o["cases"].as_array().unwrap()) {
        assert_eq!(n["builder"]["side"], c["horizontal"]);
        assert_eq!(n["builder"]["reference"], c["reference"]);
        assert_eq!(n["builder"]["theoretical_line"], c["theoretical_line"]);
        assert_eq!(n["builder"]["length_map"], c["length_map"]);
        assert_eq!(n["last_index"], c["last_index"]);
        assert_eq!(n["expanded_line"], c["expanded_line"]);
        let items = n["builder"]["items"].as_array().unwrap();
        let expected = c["items"].as_array().unwrap();
        assert_eq!(items.len(), expected.len());
        for (a, b) in items.iter().zip(expected) {
            for k in ["kind", "contrib", "line", "glyph"] {
                assert_eq!(a[k], b[k], "{k}");
            }
        }
        for k in ["bounds", "median", "glyph"] {
            assert_eq!(n["stem"][k], c["stem"][k], "{k}");
        }
        assert!(
            (n["stem"]["grade"].as_f64().unwrap() - c["stem"]["grade"].as_f64().unwrap()).abs()
                < 1e-12
        );
        let rels = n["head_relations"].as_array().unwrap();
        assert_eq!(rels.len(), c["head_relations"].as_array().unwrap().len());
        for (a, b) in rels.iter().zip(c["head_relations"].as_array().unwrap()) {
            for k in ["head_bounds", "shape", "side", "vertical", "stump"] {
                assert_eq!(a["linker"][k], b["linker"][k]);
            }
            for k in ["head_side", "extension_point"] {
                assert_eq!(a["relation"][k], b[k]);
            }
            assert!(
                (a["relation"]["grade"].as_f64().unwrap() - b["grade"].as_f64().unwrap()).abs()
                    < 1e-12
            );
        }
        let id = n["stem"]["glyph"].clone();
        assert!(
            owner.graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["kind"] == "StemInter" && v["glyph"] == id)
        );
    }
    assert_eq!(
        owner.graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "BeamStemRelation")
            .count(),
        2
    );
    assert_eq!(owner.prepared.report["stems_complete"], false);
}
