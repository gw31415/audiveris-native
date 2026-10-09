// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use serde_json::{Value, json};
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static FACTORIES: RefCell<Option<Vec<Value>>> = const { RefCell::new(None) };
}
pub(crate) fn begin_factory_observation() {
    FACTORIES.with(|rows| {
        assert!(rows.borrow().is_none());
        *rows.borrow_mut() = Some(Vec::new());
    });
}
pub(super) fn before(graph: &Graph) -> Option<Value> {
    FACTORIES.with(|rows| rows.borrow().as_ref().map(|_| graph.snapshot()))
}
pub(super) fn record(
    graph: &Graph,
    music: &OwnedMusic,
    system: u64,
    bounds: [i64; 4],
    grade: f64,
    found: &Lookup,
    before: Option<Value>,
) {
    let Some(before) = before else { return };
    let after = graph.snapshot();
    assert_eq!(
        after, before,
        "nullable factory lookup must retain the entire graph"
    );
    let actual_system = music.owner.systems.iter().find(|s| s.id == system).unwrap();
    let heads = graph.live_inter_keys(system, "HeadInter");
    let nearby = found.nearby.iter().map(|candidate| {
        let chord = music.chords.iter().find(|c| c.key == candidate.key).unwrap();
        let measure = &music.measures[chord.measure];
        let actual_measure = measure.geometry.as_ref().unwrap();
        let (body, removed) = graph.retained_inter(chord.key).unwrap();
        assert!(!removed);
        let members = heads.iter().filter(|h| graph.has_relation(chord.key, **h, "Containment"))
            .map(|key| {
                let (body, removed) = graph.retained_inter(*key).unwrap();
                assert!(!removed);
                json!({"key":key,"id":graph.id(*key),"bounds":body["bounds"],"manual":body["manual"]})
            }).collect::<Vec<_>>();
        json!({"key":candidate.key,"id":chord.id,"bounds":candidate.bounds,"heads_bounds":candidate.heads,
            "heads_intersect_lookup":intersects(candidate.heads,found.area),"y_overlap":y_overlap(candidate.heads,found.area),
            "actual_measure":{"system":measure.system,"part":measure.part,"measure_index":measure.index},
            "measure_pointer":Arc::as_ptr(actual_measure) as usize,
            "part_pointer":Arc::as_ptr(&actual_measure.part) as usize,
            "members":members,"voice":body["voice"],"slot":body["slot_id"],"time_offset":body["time_offset"],
            "manual":body["manual"]})
    }).collect::<Vec<_>>();
    let row = json!({"system":system,"system_pointer":Arc::as_ptr(actual_system) as usize,
        "bounds":bounds,"intrinsic_grade":grade,"profile":actual_system.profile,"lookup_bounds":found.area,
        "actual_factory_head_chord_count":found.head_chord_count,"nearby_actual_same_system_head_chords":nearby,
        "returns_null":found.link.is_none(),"selected":found.link.map(|(key,grade)|json!({"key":key,"grade":grade})),
        "vertex_delta":0,"edge_delta":0,"same_entire_graph":true});
    FACTORIES.with(|rows| rows.borrow_mut().as_mut().unwrap().push(row));
}

/// Called from the existing DerWinter image test after its ordinary SYMBOLS run.
/// All expected source data is test-only and is compared after native lookup.
pub(crate) fn assert_der_winter_null_factory(music: &OwnedMusic) -> Value {
    let rows = FACTORIES.with(|rows| {
        rows.borrow_mut()
            .take()
            .expect("enable observation before SYMBOLS")
    });
    let observation = json!({"stage":"actual native checked SYMBOLS factory before weak removal",
        "factory_candidates":rows,"original_observations_used_as_native_inputs":false});
    std::fs::write(
        "/tmp/real-rest-arpeggiato-native-factory.json",
        serde_json::to_vec_pretty(&observation).unwrap(),
    )
    .unwrap();
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-arpeggiato/der-winter-original.json"
    ))
    .unwrap();
    assert!(source["observer_errors"].as_array().unwrap().is_empty());
    let actual = observation["factory_candidates"].as_array().unwrap();
    let expected = source["factory_candidates"].as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    assert_eq!(expected.len(), 1);
    for (actual, expected) in actual.iter().zip(expected) {
        for field in [
            "system",
            "bounds",
            "profile",
            "lookup_bounds",
            "actual_factory_head_chord_count",
            "returns_null",
            "vertex_delta",
            "edge_delta",
        ] {
            assert_eq!(actual[field], expected[field], "{field}");
        }
        assert!(
            (actual["intrinsic_grade"].as_f64().unwrap()
                - expected["intrinsic_grade"].as_f64().unwrap())
            .abs()
                < 1e-12
        );
        assert_eq!(actual["same_entire_graph"], true);
        assert_eq!(actual["returns_null"], true);
        assert!(actual["selected"].is_null());
        assert!(expected["separate_original_lookup"].is_null());
        assert_eq!(expected["previous_vertex_edge_identity_retained"], true);
        let system = actual["system"].as_u64().unwrap();
        let held_system = music.owner.systems.iter().find(|s| s.id == system).unwrap();
        assert_eq!(
            actual["system_pointer"],
            json!(Arc::as_ptr(held_system) as usize)
        );
        let nearby = actual["nearby_actual_same_system_head_chords"]
            .as_array()
            .unwrap();
        let source_nearby = expected["nearby_actual_same_system_head_chords"]
            .as_array()
            .unwrap();
        assert_eq!(nearby.len(), source_nearby.len());
        for (chord, source_chord) in nearby.iter().zip(source_nearby) {
            for field in [
                "bounds",
                "heads_bounds",
                "heads_intersect_lookup",
                "y_overlap",
                "voice",
                "slot",
                "time_offset",
            ] {
                assert_eq!(chord[field], source_chord[field], "{field}");
            }
            for field in ["system", "part", "measure_index"] {
                assert_eq!(
                    chord["actual_measure"][field],
                    source_chord["actual_measure"][field]
                );
            }
            assert_eq!(
                chord["members"].as_array().unwrap().len(),
                source_chord["members"].as_array().unwrap().len()
            );
            let key: Key = serde_json::from_value(chord["key"].clone()).unwrap();
            let actual_chord = music.chords.iter().find(|c| c.key == key).unwrap();
            assert_eq!(Some(actual_chord.id), music.owner.graph.id(key));
            let held_measure = music.measures[actual_chord.measure]
                .geometry
                .as_ref()
                .unwrap();
            assert_eq!(
                chord["measure_pointer"],
                json!(Arc::as_ptr(held_measure) as usize)
            );
            assert_eq!(
                chord["part_pointer"],
                json!(Arc::as_ptr(&held_measure.part) as usize)
            );
        }
    }
    observation
}

fn candidate(local: usize, bounds: [i64; 4], heads: [i64; 4]) -> Candidate {
    Candidate {
        key: Key {
            system: 1,
            phase: crate::shared_sig::Phase::Chord,
            local,
        },
        bounds,
        heads,
    }
}

#[test]
fn head_only_overlap_excludes_a_stem_only_hit_and_open_rectangle_edges() {
    let area = lookup_area([805, 803, 1, 16], 11, 1).unwrap();
    assert_eq!(area, [806, 803, 28, 16]);
    let stem_only = candidate(1, [807, 758, 14, 67], [807, 758, 14, 12]);
    assert!(intersects(stem_only.bounds, area));
    assert_eq!(select(area, &[stem_only]), None);
    for b in [
        [834, 803, 1, 16],
        [806, 819, 14, 1],
        [792, 803, 14, 16],
        [806, 802, 14, 1],
    ] {
        assert_eq!(select(area, &[candidate(1, b, b)]), None);
    }
    let b = [833, 803, 1, 16];
    assert_eq!(select(area, &[candidate(1, b, b)]).unwrap().1, 1.);
}

#[test]
fn relation_admission_keeps_maximum_overlap_first_tie_and_one_tenth_boundary() {
    let area = lookup_area([0, 0, 5, 10], 11, 1).unwrap();
    let weak = candidate(1, [6, 9, 2, 1], [6, 9, 2, 1]);
    assert_eq!(
        select(area, std::slice::from_ref(&weak)),
        Some((weak.key, 0.1))
    );
    assert_eq!(select([5, 0, 28, 11], &[weak]), None);
    let first = candidate(2, [6, 2, 2, 6], [6, 2, 2, 6]);
    let second = candidate(3, [6, 0, 2, 6], [6, 0, 2, 6]);
    let third = candidate(4, [8, 0, 2, 5], [8, 0, 2, 5]);
    assert_eq!(
        select(area, &[first.clone(), second.clone(), third]),
        Some((first.key, 0.6))
    );
    assert_eq!(
        select(area, &[second.clone(), first]),
        Some((second.key, 0.6))
    );
}

#[test]
fn scale_rounding_and_profile_fallback_preserve_the_lookup_width() {
    let normal = lookup_area([0, 0, 5, 10], 11, 0).unwrap();
    assert_eq!(normal, [5, 0, 16, 10]);
    let far = candidate(1, [22, 0, 2, 10], [22, 0, 2, 10]);
    assert_eq!(select(normal, std::slice::from_ref(&far)), None);
    for profile in [1, 2, 3, 255] {
        let area = lookup_area([0, 0, 5, 10], 11, profile).unwrap();
        assert_eq!(area, [5, 0, 28, 10]);
        assert_eq!(
            select(area, std::slice::from_ref(&far)),
            Some((far.key, 1.))
        );
    }
}
