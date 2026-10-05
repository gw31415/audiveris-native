// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    head_stem_relation::VerticalSide,
    shared_sig::{Graph, Key, Phase},
    stem_attachment::HeadRequest,
    stem_cache_attachment::{self, Contact},
    stem_cache_sig, stem_checks, stem_glyph,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use std::sync::Arc;
pub fn subset(a: &Value, b: &Value) {
    match b {
        Value::Number(n) if n.is_f64() => assert!(
            (a.as_f64().unwrap() - n.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {b}"
        ),
        Value::Object(m) => {
            for (k, v) in m {
                subset(&a[k], v)
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                subset(x, y)
            }
        }
        _ => assert_eq!(a, b),
    }
}
pub fn observe(original: &Value) -> Value {
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let mut groups = Vec::new();
    for (g, group) in original["groups"].as_array().unwrap().iter().enumerate() {
        let mut registry = Registry::default();
        let mut cache = stem_cache_sig::Retriever::new(1).unwrap();
        let mut graph = Graph::default();
        graph.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]},{"id":2,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}})).unwrap();
        let prior = graph.snapshot();
        let rows = group["cases"].as_array().unwrap();
        let heads=rows.iter().enumerate().map(|(i,c)|json!({"id":i+1,"kind":"HeadInter","shape":c["shape"],"bounds":c["head_bounds"],"grade":0.8,"staff_id":1,"abnormal":true,"removed":false,"manual":false})).collect::<Vec<_>>();
        graph.sync_heads(1, &heads, &[], &[]).unwrap();
        let parts = group["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| beam_dispatch::glyph_from_json(p).unwrap())
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
        let params = stem_checks::Parameters::new(
            original["interline"].as_u64().unwrap() as usize,
            5,
            0.,
            0,
        )
        .unwrap();
        let stem = cache
            .create_selected(&mut registry, &parts, &image, &params)
            .unwrap()
            .stem
            .unwrap();
        let id = cache.register(&registry, &mut graph, &stem).unwrap();
        let mut created = stem.snapshot().unwrap();
        created["id"] = json!(1);
        subset(&created, &group["created"]);
        let mut cases = Vec::new();
        for (i, c) in rows.iter().enumerate() {
            let key = Key {
                system: 1,
                phase: Phase::Head,
                local: i + 1,
            };
            let head_id = graph.id(key).unwrap();
            let state = |s: Value| {
                let inters = s["inters"].as_array().unwrap();
                json!({"head_abnormal":inters.iter().find(|v|v["id"]==head_id).unwrap()["abnormal"],"stem_abnormal":inters.iter().find(|v|v["id"]==id).unwrap()["abnormal"]})
            };
            let request = HeadRequest {
                local_relation: i + 1,
                head: key,
                stem_local: stem.glyph_id(),
                point_size: 51,
                vertical: if c["vertical"] == "TOP" {
                    VerticalSide::Top
                } else {
                    VerticalSide::Bottom
                },
                profile: c["profile"].as_u64().unwrap() as u8,
                learned_dx: [None, None],
            };
            let stump = if c["stump"].is_null() {
                None
            } else {
                Some(beam_dispatch::glyph_from_json(&c["stump"]).unwrap())
            };
            let before = state(graph.snapshot());
            let registry_before = format!("{registry:?}");
            let cached_before = stem.snapshot().unwrap();
            let edges = graph.snapshot()["relations"].as_array().unwrap().len();
            let relation = stem_cache_attachment::connect(
                &mut cache,
                &registry,
                &mut graph,
                &stem,
                Contact {
                    request: &request,
                    stump: stump.as_ref(),
                    interline: params.interline,
                    bank: &bank,
                },
            )
            .unwrap_or_else(|e| panic!("group{g}case{i} {e}"));
            subset(&json!(relation), &c["relation"]);
            let after = state(graph.snapshot());
            let delta = graph.snapshot()["relations"].as_array().unwrap().len() - edges;
            image.pixels.fill(255);
            let mut other = params.clone();
            other.profile = 1;
            let cached = cache
                .create_selected(&mut registry, &parts, &image, &other)
                .unwrap();
            assert!(cached.cache_hit);
            let cached = cached.stem.unwrap();
            let same = Arc::ptr_eq(&cached, &stem);
            let stable = cached.snapshot().unwrap()["id"] == id;
            if relation.is_some() {
                stem_cache_attachment::disconnect(
                    &mut cache,
                    &registry,
                    &mut graph,
                    &stem,
                    request.local_relation,
                )
                .unwrap();
            }
            let removed = state(graph.snapshot());
            assert_eq!(registry_before, format!("{registry:?}"));
            assert_eq!(cached_before, stem.snapshot().unwrap());
            let value = json!({"before":before,"relation":relation,"after":after,"relation_delta":delta,"cached_same_object_with_white_image":same,"cached_positive_SIG_id_stable":stable,"after_edge_removal":removed});
            for k in [
                "before",
                "after",
                "relation_delta",
                "cached_same_object_with_white_image",
                "cached_positive_SIG_id_stable",
                "after_edge_removal",
            ] {
                assert_eq!(value[k], c[k], "group{g}case{i}{k}");
            }
            cases.push(value);
        }
        for v in prior["inters"].as_array().unwrap() {
            assert!(graph.snapshot()["inters"].as_array().unwrap().contains(v));
        }
        assert_eq!(graph.snapshot()["relations"], prior["relations"]);
        groups.push(json!({"created":created,"cases":cases}));
    }
    json!({"groups":groups,"numeric_JVM_SIG_ids_normalized":true,"registry_and_cache_identity_preserved":true,"automatic_parts_or_page_STEMS_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false})
}
