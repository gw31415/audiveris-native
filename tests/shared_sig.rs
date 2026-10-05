// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::Registry,
    beam_groups::{self, GroupPass},
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
    multiple_rest_detection::SerifPlan,
    multiple_rest_registration::{Relation, RestGraph, RestInter, SerifInter},
    serif_peaks::Peak,
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
fn grid() -> Value {
    json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter","glyph_id":1},{"id":2,"kind":"BarlineInter","glyph_id":2}],"relations":[{"kind":"BarGroupRelation","source":1,"target":2}]}]}})
}
fn state() -> SystemCandidates {
    let image = GrayImage::new(250, 200, vec![0; 50000]).unwrap();
    let mut registry = Registry::default();
    let mut state = SystemCandidates::empty(1);
    for y in [153., 163.] {
        state
            .register_merge(
                &image,
                &mut registry,
                &MergeCandidate {
                    item: BeamItem {
                        median: [100., y, 200., y],
                        height: 8.,
                    },
                    is_small: false,
                    impacts: Impacts {
                        values: [1.; 6],
                        grade: 0.8,
                        non_finite: false,
                    },
                },
            )
            .unwrap();
    }
    state
}
fn endpoints(snapshot: &Value) {
    let ids = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_u64().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), snapshot["inters"].as_array().unwrap().len());
    for r in snapshot["relations"].as_array().unwrap() {
        assert!(ids.contains(&r["source"].as_u64().unwrap()));
        assert!(ids.contains(&r["target"].as_u64().unwrap()));
    }
}
#[test]
fn colliding_phase_ids_relations_and_replay_are_stable() {
    let s = state();
    let groups = beam_groups::populate_system(&s, 16).unwrap();
    let mut g = Graph::default();
    g.import_grid(&grid()).unwrap();
    g.sync_system(&s, &groups, &RestGraph::default()).unwrap();
    let first = g.snapshot();
    endpoints(&first);
    assert_eq!(
        g.id(Key {
            system: 1,
            phase: Phase::Grid,
            local: 1
        }),
        Some(1)
    );
    assert_eq!(
        g.id(Key {
            system: 1,
            phase: Phase::Beam,
            local: 1
        }),
        Some(3)
    );
    assert_eq!(
        g.id(Key {
            system: 1,
            phase: Phase::Group,
            local: 1
        }),
        Some(5)
    );
    g.import_grid(&grid()).unwrap();
    g.sync_system(&s, &groups, &RestGraph::default()).unwrap();
    assert!(g.sync_candidates(&s).is_err());
    assert_eq!(first, g.snapshot());
    assert_eq!(first, g.snapshot());
    let mut replay = Graph::default();
    replay.import_grid(&grid()).unwrap();
    replay
        .sync_system(&s, &groups, &RestGraph::default())
        .unwrap();
    assert_eq!(first, replay.snapshot());
}
#[test]
fn removed_beam_retains_identity_and_rest_relations_share_one_store() {
    let mut s = state();
    let mut groups = beam_groups::populate_system(&s, 16).unwrap();
    let mut g = Graph::default();
    g.import_grid(&grid()).unwrap();
    g.sync_system(&s, &groups, &RestGraph::default()).unwrap();
    let old = g
        .id(Key {
            system: 1,
            phase: Phase::Beam,
            local: 1,
        })
        .unwrap();
    let mut rests = RestGraph::default();
    rests.rests.push(RestInter {
        id: 1,
        source_beam: 1,
        staff_id: 7,
        median: [100., 153., 200., 153.],
        height: 8.,
        grade: 0.8,
    });
    for (id, left) in [(2, true), (3, false)] {
        rests.serifs.push(SerifInter {
            id,
            glyph_id: id,
            plan: SerifPlan {
                left,
                peak: Peak {
                    start: 100,
                    stop: 100,
                    top: 130,
                    bottom: 170,
                    impacts: [1.; 6],
                    grade: 0.8,
                },
                median: [100.5, 129.5, 100.5, 171.5],
                width: 1.,
            },
        });
        rests.relations.push(Relation {
            rest_id: 1,
            serif_id: id,
            left,
        });
    }
    rests.finish_cleanup(&mut s, &mut groups).unwrap();
    g.sync_system(&s, &groups, &rests).unwrap();
    let snap = g.snapshot();
    endpoints(&snap);
    assert_eq!(
        g.id(Key {
            system: 1,
            phase: Phase::Beam,
            local: 1
        }),
        Some(old)
    );
    assert!(
        snap["tombstones"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["id"] == old)
    );
    let rest = snap["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["kind"] == "MultipleRestInter")
        .unwrap();
    assert_eq!(rest["source_beam_id"], old);
    assert!(rest["id"] != old);
    let snapshot = snap.clone();
    g.sync_system(&s, &groups, &rests).unwrap();
    assert_eq!(snapshot, g.snapshot());
}
#[test]
fn dangling_relations_and_duplicate_sources_reject_transactionally() {
    let mut graph = Graph::default();
    graph.import_grid(&grid()).unwrap();
    let before = graph.snapshot();
    let mut bad = grid();
    bad["bar_semantics"]["systems"][0]["relations"][0]["target"] = json!(99);
    assert!(graph.import_grid(&bad).is_err());
    assert_eq!(graph.snapshot(), before);
    bad = grid();
    bad["bar_semantics"]["systems"][0]["inters"][1]["id"] = json!(1);
    assert!(graph.import_grid(&bad).is_err());
    assert_eq!(graph.snapshot(), before);
    let s = state();
    let bad_groups = GroupPass {
        ordered_ids: vec![1, 2],
        groups: vec![],
        supports: vec![(1, 99)],
    };
    assert!(
        graph
            .sync_system(&s, &bad_groups, &RestGraph::default())
            .is_err()
    );
    assert_eq!(graph.snapshot(), before);
}
#[test]
fn systems_with_same_local_ids_are_separate() {
    let mut value = grid();
    let mut other = value["bar_semantics"]["systems"][0].clone();
    other["id"] = json!(2);
    value["bar_semantics"]["systems"]
        .as_array_mut()
        .unwrap()
        .push(other);
    let mut g = Graph::default();
    g.import_grid(&value).unwrap();
    endpoints(&g.snapshot());
    assert_ne!(
        g.id(Key {
            system: 1,
            phase: Phase::Grid,
            local: 1
        }),
        g.id(Key {
            system: 2,
            phase: Phase::Grid,
            local: 1
        })
    );
}
