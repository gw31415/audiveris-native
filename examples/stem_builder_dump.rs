// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    stem_builder, stem_checks, stem_glyph,
};
use serde_json::{Value, json};
use std::sync::Arc;
fn observe(f: &Value) -> Value {
    let mut registry = Registry::default();
    let mut cache = stem_builder::Cache::new(1).unwrap();
    let mut cases = Vec::new();
    for c in f["cases"].as_array().unwrap() {
        let parts = c["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
            .collect::<Vec<_>>();
        let painted = stem_glyph::assemble(&parts).unwrap();
        let mut image = GrayImage {
            width: f["image_width"].as_u64().unwrap() as usize,
            height: f["image_height"].as_u64().unwrap() as usize,
            pixels: vec![
                255;
                (f["image_width"].as_u64().unwrap() * f["image_height"].as_u64().unwrap())
                    as usize
            ],
        };
        for r in &painted.runs {
            for y in r.start..r.start + r.length {
                image.pixels[(painted.top as usize + y) * image.width
                    + painted.left as usize
                    + r.position] = 0;
            }
        }
        let profile = c["profile"].as_u64().unwrap() as u8;
        let params = stem_checks::Parameters::new(
            f["interline"].as_u64().unwrap() as usize,
            f["max_stem"].as_u64().unwrap() as usize,
            f["global_slope"].as_f64().unwrap(),
            profile,
        )
        .unwrap();
        let before = registry.len();
        let cache_before = cache.len();
        let first = cache
            .create_selected(&mut registry, &parts, &image, &params)
            .unwrap();
        let glyph = registry.get(first.glyph_id).unwrap().glyph.clone();
        let first_json = first.stem.as_ref().map(|s| s.record.clone());
        let delta = registry.len() - before;
        let cache_delta = cache.len() - cache_before;
        image.pixels.fill(255);
        let mut other = params.clone();
        other.profile = 1 - profile;
        let second = cache
            .create_selected(&mut registry, &parts, &image, &other)
            .unwrap();
        assert_eq!(first.glyph_id, second.glyph_id);
        let same = match (&first.stem, &second.stem) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        let regrade = stem_checks::check_glyph(&image, &glyph, &other)
            .unwrap()
            .grade;
        cases.push(json!({"profile":profile,"parts":c["parts"],"registered":beam_dispatch::glyph_json(&glyph),"registered_delta":delta,"cache_delta":cache_delta,"first":first_json,"second":second.stem.as_ref().map(|s|&s.record),"cache_reused_same_stem":same,"second_white_image_if_regraded":regrade}));
    }
    json!({"builder_min_grade":stem_builder::MIN_GRADE,"cases":cases})
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    assert!(!std::path::Path::new(&args[2]).exists());
    let f: Value = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    std::fs::write(&args[2], serde_json::to_vec_pretty(&observe(&f)).unwrap()).unwrap();
}
