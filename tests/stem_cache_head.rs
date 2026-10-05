// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_cache_head_controls.rs"]
mod controls;
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
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-cache-head/controls.json"
    ))
    .unwrap()
}
fn contact<'a>(r: &'a HeadRequest, bank: &'a Bank) -> Contact<'a> {
    Contact {
        request: r,
        stump: None,
        interline: 11,
        bank,
    }
}
#[test]
fn original_cache_head_connections_and_callbacks_1152() {
    let _ = controls::observe(&fixture());
}
#[test]
fn preserve_other_edges_owned_cache_and_atomic_refusals() {
    let f = fixture();
    let c = f["groups"][0]["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| !c["relation"].is_null() && c["stump"].is_null())
        .unwrap();
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let mut registry = Registry::default();
    let mut retriever = stem_cache_sig::Retriever::new(1).unwrap();
    let mut graph = Graph::default();
    graph.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]},{"id":2,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}})).unwrap();
    let heads=(1..=2).map(|id|json!({"id":id,"kind":"HeadInter","shape":c["shape"],"bounds":c["head_bounds"],"grade":0.8,"staff_id":1,"abnormal":true,"removed":false,"manual":false})).collect::<Vec<_>>();
    graph.sync_heads(1, &heads, &[], &[]).unwrap();
    let parts = f["groups"][0]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
        .collect::<Vec<_>>();
    let glyph = stem_glyph::assemble(&parts).unwrap();
    let mut image = GrayImage {
        width: 896,
        height: 160,
        pixels: vec![255; 896 * 160],
    };
    for r in &glyph.runs {
        for y in r.start..r.start + r.length {
            image.pixels[(glyph.top as usize + y) * 896 + glyph.left as usize + r.position] = 0;
        }
    }
    let p = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
    let stem = retriever
        .create_selected(&mut registry, &parts, &image, &p)
        .unwrap()
        .stem
        .unwrap();
    let request = HeadRequest {
        local_relation: 1,
        head: Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        },
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
    let before = graph.snapshot();
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&request, &bank)
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    retriever.register(&registry, &mut graph, &stem).unwrap();
    let snapshot = stem.snapshot().unwrap();
    let registry_before = format!("{registry:?}");
    let before = graph.snapshot();
    let mut foreign = stem_cache_sig::Retriever::new(1).unwrap();
    assert!(
        stem_cache_attachment::connect(
            &mut foreign,
            &registry,
            &mut graph,
            &stem,
            contact(&request, &bank)
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    let mut invalid = request.clone();
    invalid.stem_local += 1;
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&invalid, &bank)
        )
        .is_err()
    );
    invalid = request.clone();
    invalid.head.system = 2;
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&invalid, &bank)
        )
        .is_err()
    );
    invalid = request.clone();
    invalid.head.local = 999;
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&invalid, &bank)
        )
        .is_err()
    );
    invalid = request.clone();
    invalid.profile = 4;
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&invalid, &bank)
        )
        .is_err()
    );
    let mut bad = glyph.clone();
    bad.runs.clear();
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            Contact {
                request: &request,
                stump: Some(&bad),
                interline: 11,
                bank: &bank
            }
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(stem.snapshot().unwrap(), snapshot);
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&request, &bank)
        )
        .unwrap()
        .is_some()
    );
    let first = graph.snapshot()["relations"].clone();
    let mut second = request.clone();
    second.head.local = 2;
    second.local_relation = 2;
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&second, &bank)
        )
        .unwrap()
        .is_some()
    );
    let linked = graph.snapshot();
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&second, &bank)
        )
        .is_err()
    );
    assert!(
        stem_cache_attachment::disconnect(&mut retriever, &registry, &mut graph, &stem, 99)
            .is_err()
    );
    assert_eq!(graph.snapshot(), linked);
    stem_cache_attachment::disconnect(&mut retriever, &registry, &mut graph, &stem, 2).unwrap();
    assert_eq!(graph.snapshot()["relations"], first);
    assert_eq!(stem.snapshot().unwrap(), snapshot);
    assert_eq!(format!("{registry:?}"), registry_before);
    image.pixels.fill(255);
    assert!(Arc::ptr_eq(
        &stem,
        &retriever
            .create_selected(&mut registry, &parts, &image, &p)
            .unwrap()
            .stem
            .unwrap()
    ));
    retriever.remove(&mut graph, &stem).unwrap();
    let removed = graph.snapshot();
    assert!(
        stem_cache_attachment::connect(
            &mut retriever,
            &registry,
            &mut graph,
            &stem,
            contact(&request, &bank)
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), removed);
    assert!(graph.snapshot()["relations"].as_array().unwrap().is_empty());
}
