// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Group, Registry},
    beam_impacts::Impacts,
    beam_merge::{MergeCandidate, MergeInput, merge_of},
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn candidate(a: f64, b: f64) -> MergeCandidate {
    MergeCandidate {
        item: BeamItem {
            median: [a, 40., b, 40.],
            height: 8.,
        },
        is_small: false,
        impacts: Impacts {
            values: [1.; 6],
            grade: 0.8,
            non_finite: false,
        },
    }
}
fn image(black: bool) -> GrayImage {
    let mut pixels = vec![255; 128 * 96];
    if black {
        for x in 10..100 {
            for y in 36..44 {
                pixels[y * 128 + x] = 0;
            }
        }
    }
    GrayImage::new(128, 96, pixels).unwrap()
}
fn snapshot(s: &SystemCandidates, r: &Registry) -> Value {
    json!({"active_ids":s.inters().iter().filter(|v|!v.is_removed()).map(|v|v.id()).collect::<Vec<_>>(),
        "removed":s.inters().iter().map(|v|v.is_removed()).collect::<Vec<_>>(),
        "index_present":s.inters().iter().map(|v|s.active(v.id()).is_some()).collect::<Vec<_>>(),
        "edge_count":s.exclusions().len(),
        "glyph_groups":s.inters().iter().map(|v|r.get(v.glyph_id()).unwrap().groups.iter().map(|g|match g {Group::BeamSpot=>"BEAM_SPOT",_=>"unexpected"}).collect::<Vec<_>>()).collect::<Vec<_>>()})
}
#[test]
fn original_registration_removal_and_rejection_match() {
    let official: Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-lifecycle.json")).unwrap();
    assert_eq!(official["native_outputs_used"], false);
    assert_eq!(official["duplicate_add_return"], false);
    assert_eq!(official["unregistered_merge_id"], 0);
    assert_eq!(official["new_beam_group_is_null"], true);
    let mut s = SystemCandidates::empty(1);
    let mut r = Registry::default();
    let black = image(true);
    let a = candidate(10., 30.);
    let b = candidate(34., 70.);
    let first = s.register_merge(&black, &mut r, &a).unwrap();
    let second = s.register_merge(&black, &mut r, &b).unwrap();
    let glyph = s.active(first).unwrap().glyph_id();
    r.add_group(glyph, Group::BeamSpot).unwrap();
    assert_eq!(r.register(&s.active(first).unwrap().glyph).unwrap(), glyph);
    s.add_exclusion(first, second).unwrap();
    let mut states = vec![snapshot(&s, &r)];
    let inputs = (
        MergeInput {
            item: a.item.clone(),
            distance_impact: 1.,
        },
        MergeInput {
            item: b.item.clone(),
            distance_impact: 1.,
        },
    );
    let reject = merge_of(&image(false), &inputs.0, &inputs.1, 16, 8., None).unwrap();
    assert!(reject.is_none());
    let before = s.clone();
    let reg = r.snapshot();
    assert_eq!(
        s.apply_merge(&black, &mut r, first, second, reject.as_ref())
            .unwrap(),
        None
    );
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), reg);
    states.push(snapshot(&s, &r));
    let merged = merge_of(&black, &inputs.0, &inputs.1, 16, 8., None)
        .unwrap()
        .unwrap();
    s.register_merge(&black, &mut r, &merged).unwrap();
    states.push(snapshot(&s, &r));
    s.remove(first).unwrap();
    states.push(snapshot(&s, &r));
    s.remove(second).unwrap();
    s.remove(second).unwrap();
    states.push(snapshot(&s, &r));
    let extra = s.register_merge(&black, &mut r, &a).unwrap();
    assert_eq!(extra, 4);
    assert_eq!(s.active(extra).unwrap().glyph_id(), glyph);
    states.push(snapshot(&s, &r));
    for (got, expected) in states.iter().zip(official["states"].as_array().unwrap()) {
        for key in [
            "active_ids",
            "removed",
            "index_present",
            "edge_count",
            "glyph_groups",
        ] {
            assert_eq!(got[key], expected[key], "{key}");
        }
    }
    let ids: Vec<_> = s.inters().iter().map(|v| v.glyph_id()).collect();
    let last = &official["states"][5];
    for i in 0..4 {
        for j in 0..4 {
            assert_eq!(
                ids[i] == ids[j],
                last["glyph_ids"][i] == last["glyph_ids"][j]
            );
        }
    }
    let originals = last["original_ids"].as_array().unwrap();
    assert!(originals.windows(2).all(|w| w[0].as_u64() < w[1].as_u64()));
    println!("6 original lifecycle snapshots matched; native IDs normalized by registration order");
}
#[test]
fn accepted_application_is_atomic_for_invalid_endpoints_and_preserves_raw_order() {
    let mut s = SystemCandidates::empty(1);
    let mut r = Registry::default();
    let black = image(true);
    let a = candidate(10., 30.);
    let b = candidate(34., 70.);
    let merged = candidate(10., 70.);
    s.register_merge(&black, &mut r, &a).unwrap();
    s.register_merge(&black, &mut r, &b).unwrap();
    s.add_exclusion(1, 2).unwrap();
    let before = s.clone();
    let reg = r.snapshot();
    assert!(s.apply_merge(&black, &mut r, 1, 99, Some(&merged)).is_err());
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), reg);
    assert_eq!(
        s.apply_merge(&black, &mut r, 1, 2, Some(&merged)).unwrap(),
        Some(3)
    );
    assert!(s.active(1).is_none() && s.active(2).is_none() && s.active(3).is_some());
    assert!(s.exclusions().is_empty());
    assert_eq!(
        s.inters().iter().map(|v| v.id()).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}
