// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "stem_parts_cache_head_controls.rs"]
mod cache_head;
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    shared_sig::Graph,
    stem_cache_sig, stem_checks, stem_glyph,
    stem_parts_cache::{self, Item},
};
use serde_json::{Value, json};
use std::sync::Arc;
fn load_items(registry: &mut Registry, values: &Value) -> Vec<Item> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            if !v["glyph"].is_null() {
                assert_eq!(v["glyph_class"], "org.audiveris.omr.glyph.Glyph");
            }
            Item {
                gap: v["gap"].as_bool().unwrap(),
                glyph_id: if v["glyph"].is_null() {
                    None
                } else {
                    Some(
                        registry
                            .register(&beam_dispatch::glyph_from_json(&v["glyph"]).unwrap())
                            .unwrap(),
                    )
                },
            }
        })
        .collect()
}
fn parts(registry: &Registry, ids: &[usize]) -> Value {
    json!(
        ids.iter()
            .map(|&id| beam_dispatch::glyph_json(&registry.get(id).unwrap().glyph))
            .collect::<Vec<_>>()
    )
}
pub fn observe(original: &Value) -> Value {
    let mut groups = Vec::new();
    let mut total = 0;
    for g in original["groups"]
        .as_array()
        .unwrap()
        .iter()
        .chain(original["orientation_groups"].as_array().unwrap())
    {
        let mut cases = Vec::new();
        for c in g["cases"].as_array().unwrap() {
            let mut registry = Registry::default();
            let items = load_items(&mut registry, &c["items"]);
            let mut prefixes = Vec::new();
            let mut gaps = Vec::new();
            for last in -1..items.len() as isize {
                let selected = stem_parts_cache::select_until(&registry, &items, last).unwrap();
                let p = parts(&registry, &selected);
                assert_eq!(p, c["prefix_parts"][(last + 1) as usize]);
                prefixes.push(p);
                total += 1;
            }
            for before in 0..=items.len() {
                let v = stem_parts_cache::last_gap_before(&items, before).unwrap();
                assert_eq!(json!(v), c["last_gap_before"][before]);
                gaps.push(v);
            }
            cases.push(json!({"prefix_parts":prefixes,"last_gap_before":gaps}));
        }
        groups.push(json!({"cases":cases}));
    }
    let old: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-stem-cache-head/controls.json"
    ))
    .unwrap();
    let mut cached = Vec::new();
    for (i, c) in original["selected_cache_controls"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(c["selected"], old["groups"][i]["parts"]);
        cache_head::subset(&c["created"], &old["groups"][i]["created"]);
        let mut registry = Registry::default();
        let items = load_items(&mut registry, &c["items"]);
        let ids =
            stem_parts_cache::select_until(&registry, &items, items.len() as isize - 1).unwrap();
        assert_eq!(parts(&registry, &ids), c["selected"]);
        let physical = ids
            .iter()
            .map(|&id| registry.get(id).unwrap().glyph.clone())
            .collect::<Vec<_>>();
        let painted = stem_glyph::assemble(&physical).unwrap();
        let mut image = GrayImage {
            width: 896,
            height: 160,
            pixels: vec![255; 896 * 160],
        };
        for r in &painted.runs {
            for y in r.start..r.start + r.length {
                image.pixels
                    [(painted.top as usize + y) * 896 + painted.left as usize + r.position] = 0;
            }
        }
        let parameters = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
        let mut retriever = stem_cache_sig::Retriever::new(1).unwrap();
        let creation = stem_parts_cache::create_until(
            &mut retriever,
            &mut registry,
            &items,
            items.len() as isize - 1,
            &image,
            &parameters,
        )
        .unwrap();
        assert!(!creation.cache_hit);
        let stem = creation.stem.unwrap();
        let mut graph = Graph::default();
        let sig_id = retriever.register(&registry, &mut graph, &stem).unwrap();
        let mut created = stem.snapshot().unwrap();
        created["id"] = json!(1);
        cache_head::subset(&created, &c["created"]);
        image.pixels.fill(255);
        let mut p = parameters.clone();
        p.profile = 1;
        let hit = stem_parts_cache::create_until(
            &mut retriever,
            &mut registry,
            &items,
            items.len() as isize - 1,
            &image,
            &p,
        )
        .unwrap();
        assert!(hit.cache_hit);
        let hit = hit.stem.unwrap();
        let same = Arc::ptr_eq(&hit, &stem);
        let stable = sig_id > 0 && hit.snapshot().unwrap()["id"] == sig_id;
        assert_eq!(json!(same), c["cached_same_object_with_white_image"]);
        assert_eq!(json!(stable), c["positive_SIG_id_stable"]);
        cached.push(json!({"selected":parts(&registry,&ids),"created":created,"cached_same_object_with_white_image":same,"positive_SIG_id_stable":stable}));
    }
    let head = cache_head::observe(&old);
    json!({"groups":groups,"prefix_controls":total,"selected_cache_controls":cached,"prior_head_controls_with_native_prefix_cache_composition":head,"caller_endpoints_and_selected_painted_cache_controls":true,"automatic_image_endpoint_selection_complete":false,"page_cache_SIG_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false})
}
