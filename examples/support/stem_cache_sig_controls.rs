// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    shared_sig::Graph,
    stem_cache_sig, stem_checks, stem_glyph,
};
use serde_json::{Value, json};
use std::sync::Arc;
pub fn observe(f: &Value) -> Value {
    let mut registry = Registry::default();
    let mut cache = stem_cache_sig::Retriever::new(1).unwrap();
    let mut graph = Graph::default();
    graph.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"},{"id":2,"kind":"BarlineInter"}],"relations":[{"kind":"BarGroupRelation","source":1,"target":2}]},{"id":2,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}})).unwrap();
    let prior = graph.snapshot();
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
            width: 896,
            height: 160,
            pixels: vec![255; 896 * 160],
        };
        for r in &painted.runs {
            for y in r.start..r.start + r.length {
                image.pixels[(painted.top as usize + y) * image.width
                    + painted.left as usize
                    + r.position] = 0;
            }
        }
        let params = stem_checks::Parameters::new(11, 5, 0.0, 0).unwrap();
        let first = cache
            .create_selected(&mut registry, &parts, &image, &params)
            .unwrap()
            .stem
            .unwrap();
        let before = first.snapshot().unwrap();
        let vertices = graph.snapshot()["inters"].as_array().unwrap().len();
        let id = cache.register(&registry, &mut graph, &first).unwrap();
        let after = first.snapshot().unwrap();
        assert_eq!(after["id"], id);
        image.pixels.fill(255);
        let mut other = params.clone();
        other.profile = 1;
        let cached = cache
            .create_selected(&mut registry, &parts, &image, &other)
            .unwrap();
        assert!(cached.cache_hit);
        let cached = cached.stem.unwrap();
        let after_cached = cached.snapshot().unwrap();
        let added = graph.snapshot()["inters"].as_array().unwrap().len() - vertices;
        cache.remove(&mut graph, &first).unwrap();
        let removed = cache
            .create_selected(&mut registry, &parts, &image, &params)
            .unwrap();
        assert!(removed.cache_hit);
        let removed = removed.stem.unwrap();
        let after_removed = removed.snapshot().unwrap();
        let fields = |v: &Value| json!({"grade":v["grade"],"bounds":v["bounds"],"median":v["median"],"width":v["width"],"glyph":v["glyph"],"impacts":v["impacts"],"in_sig":v["in_sig"],"id":v["id"]});
        cases.push(json!({"parts":c["parts"],"before_SIG":fields(&before),"after_SIG":fields(&after),"after_cached_white":fields(&after_cached),"cached_same_object_after_SIG":Arc::ptr_eq(&first,&cached),"vertices_added":added,"removed":after_removed["removed"],"cached_same_object_after_removal":Arc::ptr_eq(&first,&removed),"after_removed":fields(&after_removed),"cached_removed_flag":after_removed["removed"],"vertices_after_removal_delta":graph.snapshot()["inters"].as_array().unwrap().len()-vertices}));
        assert_eq!(graph.snapshot()["inters"], prior["inters"]);
        assert_eq!(graph.snapshot()["relations"], prior["relations"]);
    }
    json!({"cases":cases})
}
