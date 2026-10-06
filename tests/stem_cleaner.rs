// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_cleaner_controls.rs"]
mod controls;
use audiveris_native::{
    beam_dispatch,
    head_stem_relation::HeadSide,
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_cleaner::{self, Link},
    stem_glyph,
};
use serde_json::{Value, json};
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-cleaner/controls.json"
    ))
    .unwrap()
}
#[test]
fn original_eight_hundred_cleaner_checks_match_ordered_ties_canonical_sharing_and_sig_callbacks() {
    let f = fixture();
    let a = controls::observe(&f);
    assert_eq!(a["cases"], f["cases"]);
    assert_eq!(f["cases"].as_array().unwrap().len(), 800);
    assert!(
        f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["kept"].as_array().unwrap().len() == 2)
    );
}
#[test]
fn cleaner_refuses_duplicate_nonfinite_stale_inputs_and_preserves_single_stem_parallel_edges() {
    let l = Link {
        relation_id: 1,
        median: [40., 50., 40., 90.],
        stem_grade: 0.8,
        target_ratio: 11.,
        head_side: HeadSide::Left,
        dy: 0.,
    };
    let bounds = [40, 50, 15, 12];
    assert!(stem_cleaner::select_removals(bounds, &[l.clone(), l.clone()]).is_err());
    let mut invalid = l.clone();
    invalid.stem_grade = f64::NAN;
    assert!(stem_cleaner::select_removals(bounds, &[invalid]).is_err());
    assert!(stem_cleaner::select_removals([40, 50, 0, 12], &[]).is_err());
    let mut graph = Graph::default();
    graph.sync_heads(1,&[json!({"id":1,"kind":"HeadInter","shape":"NOTEHEAD_BLACK","bounds":bounds,"grade":0.8,"abnormal":true,"removed":false})],&[],&[]).unwrap();
    let head = Key {
        system: 1,
        phase: Phase::Head,
        local: 1,
    };
    let before = graph.snapshot();
    let a = stem_cleaner::clean_head(&mut graph, head, &[]).unwrap();
    assert_eq!(a["removed_relation_ids"], json!([]));
    assert_eq!(graph.snapshot(), before);
    let missing = Key {
        system: 1,
        phase: Phase::Stem,
        local: 1,
    };
    assert!(stem_cleaner::clean_head(&mut graph, head, &[missing]).is_err());
    assert_eq!(graph.snapshot(), before);
    let f = fixture();
    let c = f["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["stems"].as_array().unwrap().len() == 1 && !c["stems"][0]["relation"].is_null())
        .unwrap();
    let stem = &c["stems"][0];
    let glyph = beam_dispatch::glyph_from_json(&stem["parts"][0]).unwrap();
    let record = stem_glyph::record(1, &glyph, stem["grade"].as_f64().unwrap()).unwrap();
    let mut relation = stem["relation"].clone();
    relation["kind"] = json!("HeadStemRelation");
    relation["profile"] = json!(0);
    relation["manual"] = json!(false);
    let parallel = (1..=2)
        .map(|local| HeadStemAttachment {
            local,
            head,
            stem_local: 1,
            relation: relation.clone(),
        })
        .collect::<Vec<_>>();
    graph
        .sync_stems(1, std::slice::from_ref(&record), &parallel)
        .unwrap();
    let prior = graph.snapshot();
    assert_eq!(
        stem_cleaner::clean_head(&mut graph, head, &[missing]).unwrap()["removed_relation_ids"],
        json!([])
    );
    assert_eq!(graph.snapshot(), prior);
    graph.sync_stems(1, &[record], &parallel[..1]).unwrap();
    let prior = graph.snapshot();
    assert!(stem_cleaner::clean_head(&mut graph, head, &[missing, missing]).is_err());
    assert_eq!(graph.snapshot(), prior);
    graph.sync_stems(1, &[], &[]).unwrap();
    let prior = graph.snapshot();
    assert!(stem_cleaner::clean_head(&mut graph, head, &[missing]).is_err());
    assert_eq!(graph.snapshot(), prior);
}
