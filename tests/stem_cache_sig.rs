// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_cache_sig_controls.rs"]
mod controls;
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_cache_sig, stem_checks, stem_glyph,
};
use serde_json::{Value, json};
use std::sync::Arc;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-cache-sig/controls.json"
    ))
    .unwrap()
}
fn canonical(mut cases: Value) -> Value {
    for c in cases.as_array_mut().unwrap() {
        assert_eq!(c["before_SIG"]["id"], 0);
        assert_eq!(c["before_SIG"]["in_sig"], false);
        let id = c["after_SIG"]["id"].clone();
        assert!(id.as_u64().unwrap() > 0);
        for stage in ["after_SIG", "after_cached_white", "after_removed"] {
            assert_eq!(c[stage]["id"], id);
            assert_eq!(c[stage]["in_sig"], true);
            c[stage]["id"] = json!(1);
        }
    }
    cases
}
fn near(a: &Value, b: &Value) {
    match b {
        Value::Number(n) if n.is_f64() => assert!(
            (a.as_f64().unwrap() - n.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {b}"
        ),
        Value::Object(m) => {
            assert_eq!(a.as_object().unwrap().len(), m.len());
            for (k, v) in m {
                near(&a[k], v);
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                near(x, y);
            }
        }
        _ => assert_eq!(a, b),
    }
}
#[test]
fn original_twelve_cache_sig_lifecycles_keep_glyph_grade_identity_and_removed_cache() {
    let f = fixture();
    let native = controls::observe(&f);
    let first = canonical(f["cases"].clone());
    let second: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-cache-sig/controls-certified.json"
    ))
    .unwrap();
    assert_eq!(first, canonical(second["cases"].clone()));
    near(&canonical(native["cases"].clone()), &first);
    assert_eq!(first.as_array().unwrap().len(), 12);
}
#[test]
fn cached_sig_rejects_foreign_stale_collision_and_wrong_registry_without_mutation_and_prunes_head_edges()
 {
    let f = fixture();
    let parts = f["cases"][0]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
        .collect::<Vec<_>>();
    let union = stem_glyph::assemble(&parts).unwrap();
    let mut image = GrayImage {
        width: 896,
        height: 160,
        pixels: vec![255; 896 * 160],
    };
    for r in &union.runs {
        for y in r.start..r.start + r.length {
            image.pixels[(union.top as usize + y) * 896 + union.left as usize + r.position] = 0;
        }
    }
    let p = stem_checks::Parameters::new(11, 5, 0.0, 0).unwrap();
    let mut registry = Registry::default();
    let mut cache = stem_cache_sig::Retriever::new(1).unwrap();
    let stem = cache
        .create_selected(&mut registry, &parts, &image, &p)
        .unwrap()
        .stem
        .unwrap();
    let mut graph = Graph::default();
    graph.sync_heads(1,&[json!({"id":1,"kind":"HeadInter","shape":"NOTEHEAD_BLACK","bounds":[40,50,15,10],"grade":0.8,"pitch":0.,"staff_id":1,"glyph":null,"abnormal":true,"removed":false})],&[],&[]).unwrap();
    let prior = graph.snapshot();
    let state = stem.snapshot().unwrap();
    assert!(cache.remove(&mut graph, &stem).is_err());
    let mut wrong = Registry::default();
    let mut shifted = parts[0].clone();
    shifted.left += 1;
    wrong.register(&shifted).unwrap();
    assert!(cache.register(&wrong, &mut graph, &stem).is_err());
    let mut foreign = stem_cache_sig::Retriever::new(2).unwrap();
    assert!(foreign.register(&registry, &mut graph, &stem).is_err());
    assert_eq!(graph.snapshot(), prior);
    assert_eq!(stem.snapshot().unwrap(), state);
    let id = cache.register(&registry, &mut graph, &stem).unwrap();
    let after = graph.snapshot();
    assert_eq!(cache.register(&registry, &mut graph, &stem).unwrap(), id);
    assert_eq!(graph.snapshot(), after);
    let key = Key {
        system: 1,
        phase: Phase::Stem,
        local: stem.glyph_id(),
    };
    let mut record = after["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap()
        .clone();
    record["id"] = json!(key.local);
    let attachment = HeadStemAttachment {
        local: 1,
        head: Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        },
        stem_local: key.local,
        relation: json!({"kind":"HeadStemRelation","profile":0,"head_side":"RIGHT","extension_point":[41.,50.],"dx":0.,"dy":0.,"grade":1.,"consistency":1.,"source_ratio":5.,"target_ratio":11.,"invading":true,"manual":false}),
    };
    graph
        .sync_stems(1, &[record.clone()], &[attachment])
        .unwrap();
    assert_eq!(graph.snapshot()["inters"][0]["abnormal"], false);
    cache.remove(&mut graph, &stem).unwrap();
    assert!(graph.snapshot()["relations"].as_array().unwrap().is_empty());
    assert_eq!(graph.snapshot()["inters"][0]["abnormal"], true);
    image.pixels.fill(255);
    let cached = cache
        .create_selected(&mut registry, &parts, &image, &p)
        .unwrap()
        .stem
        .unwrap();
    assert!(Arc::ptr_eq(&stem, &cached));
    assert_eq!(cached.snapshot().unwrap()["removed"], true);
    let removed = graph.snapshot();
    assert!(cache.register(&registry, &mut graph, &stem).is_err());
    cache.remove(&mut graph, &stem).unwrap();
    assert_eq!(graph.snapshot(), removed);
    // A competing boundary may not silently revive a removed cached object.
    record["removed"] = json!(false);
    graph.sync_stems(1, &[record], &[]).unwrap();
    let resurrected = graph.snapshot();
    assert!(cache.remove(&mut graph, &stem).is_err());
    assert_eq!(graph.snapshot(), resurrected);
    let mut other = stem_cache_sig::Retriever::new(1).unwrap();
    image.pixels.fill(255);
    for r in &union.runs {
        for y in r.start..r.start + r.length {
            image.pixels[(union.top as usize + y) * 896 + union.left as usize + r.position] = 0;
        }
    }
    let rival = other
        .create_selected(&mut registry, &parts, &image, &p)
        .unwrap()
        .stem
        .unwrap();
    assert!(other.register(&registry, &mut graph, &rival).is_err());
    assert_eq!(graph.snapshot(), resurrected);
}
