// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Group, Registry, SystemSpots, glyph_from_json},
    beam_extend_to_beam::ExtensionScale,
    beam_extend_to_spot::{SpotPool, SpotRequest, ThroughSpot, lookup_spot, side_through_spot},
    beam_extension_geometry::HorizontalSide,
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn snapshot(s: &SystemCandidates) -> Value {
    json!({"active_ids":s.inters().iter().filter(|v|!v.is_removed()).map(|v|v.id()).collect::<Vec<_>>(),"removed":s.inters().iter().map(|v|v.is_removed()).collect::<Vec<_>>(),"index_present":s.inters().iter().map(|v|s.active(v.id()).is_some()).collect::<Vec<_>>(),"edge_count":s.exclusions().len()})
}
#[test]
fn original_spot_pool_selection_point_and_lifecycle_match() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extend-to-spot.json"
    ))
    .unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 140);
    let mut accepted = 0;
    for (i, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let input = &c["input"];
        let mut px = vec![255; 160 * 96];
        for p in input["black_pixels"].as_array().unwrap() {
            px[p[1].as_u64().unwrap() as usize * 160 + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(160, 96, px).unwrap();
        let seed = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
        let mut s = SystemCandidates::empty(1);
        let mut r = Registry::default();
        for key in ["first", "peer"] {
            let b = &input[key];
            s.register_merge(
                &seed,
                &mut r,
                &MergeCandidate {
                    item: BeamItem {
                        median: std::array::from_fn(|i| b["median"][i].as_f64().unwrap()),
                        height: b["height"].as_f64().unwrap(),
                    },
                    is_small: false,
                    impacts: Impacts::new([
                        1.,
                        1.,
                        1.,
                        1.,
                        1.,
                        b["distance_impact"].as_f64().unwrap(),
                    ])
                    .unwrap(),
                },
            )
            .unwrap();
        }
        s.add_exclusion(1, 2).unwrap();
        let side = if input["side"] == "LEFT" {
            HorizontalSide::Left
        } else {
            HorizontalSide::Right
        };
        let mut ids = Vec::new();
        let mut owner = SystemSpots {
            system_id: 1,
            glyph_ids: Vec::new(),
        };
        let mut assigned = Vec::new();
        for raw in input["spots"].as_array().unwrap() {
            let mut g = raw.clone();
            g["orientation"] = json!("HORIZONTAL");
            let id = r.register(&glyph_from_json(&g).unwrap()).unwrap();
            ids.push(id);
            if raw["grouped"].as_bool().unwrap_or(true) {
                r.add_group(id, Group::BeamSpot).unwrap();
            }
            if raw["owned"].as_bool().unwrap_or(true) {
                owner.glyph_ids.push(id);
            }
            if raw["assigned"].as_bool().unwrap_or(false) {
                assigned.push(id);
            }
        }
        let cap = input["max_extension"].as_i64().map(|n| n as i32);
        let pool = SpotPool::prepare(&owner, &r, &assigned).unwrap();
        let map = |id: usize| ids.iter().position(|n| *n == id).unwrap() + 1;
        assert_eq!(
            json!(
                pool.ordered_ids()
                    .iter()
                    .map(|id| map(*id))
                    .collect::<Vec<_>>()
            ),
            c["pool"],
            "{i} initial full-ordinate pool"
        );
        let lookup = lookup_spot(
            &s.active(1).unwrap().item,
            s.active(1).unwrap().glyph_id(),
            &pool,
            &r,
            side,
            cap,
            input["interline"].as_u64().unwrap() as usize,
        )
        .unwrap();
        assert_eq!(lookup.reach, c["reach"].as_i64().unwrap() as i32);
        assert_eq!(
            json!(lookup.ordered.iter().map(|id| map(*id)).collect::<Vec<_>>()),
            c["ordered"],
            "{i} selected order"
        );
        assert_eq!(
            json!(lookup.selected.map(map)),
            c["selected"],
            "{i} selected"
        );
        if let Some(point) = lookup.point {
            for j in 0..2 {
                assert!(
                    (point[j] - c["point"][j].as_f64().unwrap()).abs() < 1e-11,
                    "{i} point"
                );
            }
        } else {
            assert!(c["point"].is_null());
        }
        let before = s.clone();
        let before_pool = pool.clone();
        let scale = ExtensionScale {
            interline: input["interline"].as_u64().unwrap() as usize,
            standard_height: 8.,
            small_height: None,
        };
        for seed_raw in input["seeds"].as_array().into_iter().flatten() {
            let mut g = seed_raw.clone();
            g["orientation"] = json!("VERTICAL");
            let id = r.register(&glyph_from_json(&g).unwrap()).unwrap();
            r.add_group(id, Group::VerticalSeed).unwrap();
            owner.glyph_ids.push(id);
        }
        let registry = r.snapshot();
        let got = if input["compose"] == true {
            match side_through_spot(&mut s, &image, &mut r, 1, side, scale, &owner).unwrap() {
                ThroughSpot::SpotExtended { extension, .. }
                | ThroughSpot::NeedParallelOrNextSide { extension, .. } => {
                    assert_eq!(c["preceding_stem_accepted"], false);
                    assert_eq!(c["spot_method_called"], true);
                    extension.appended
                }
                ThroughSpot::Earlier(
                    audiveris_native::beam_extend_to_stem::ThroughStem::StemExtended {
                        extension,
                        ..
                    },
                ) => {
                    assert_eq!(c["preceding_stem_accepted"], true);
                    assert_eq!(c["spot_method_called"], false);
                    extension.appended
                }
                ThroughSpot::Earlier(_) => panic!("unexpected merge"),
            }
        } else {
            s.extend_to_spot(
                &image,
                &mut r,
                SpotRequest {
                    beam: 1,
                    side,
                    max_extension: cap,
                },
                scale,
                &pool,
            )
            .unwrap()
            .appended
        };
        assert_eq!(pool, before_pool);
        assert_eq!(json!(got.is_some()), c["accepted"], "{i} acceptance");
        for k in ["active_ids", "removed", "index_present", "edge_count"] {
            assert_eq!(snapshot(&s)[k], c["after"][k], "{i} {k}");
        }
        if let Some(id) = got {
            accepted += 1;
            assert_eq!(id, 3);
            let new = s.active(id).unwrap();
            assert_eq!(json!(new.item.median), c["merged"]["median"]);
            assert_eq!(json!(new.item.height), c["merged"]["height"]);
            assert_eq!(json!(new.kind), c["merged"]["kind"]);
            assert!(
                (new.impacts.grade - c["merged"]["grade"].as_f64().unwrap()).abs() < 1e-12,
                "{i} grade"
            );
            assert!(s.active(2).is_some());
        } else {
            assert_eq!(s, before, "{i} state no-op");
            assert_eq!(r.snapshot(), registry, "{i} registry no-op");
        }
    }
    assert_eq!(
        accepted,
        v["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["accepted"] == true)
            .count()
    );
}
#[test]
fn foreign_owner_and_negative_reach_are_noops() {
    let image = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
    let mut s = SystemCandidates::empty(1);
    let mut r = Registry::default();
    s.register_merge(
        &image,
        &mut r,
        &MergeCandidate {
            item: BeamItem {
                median: [30., 40., 70., 40.],
                height: 8.,
            },
            is_small: false,
            impacts: Impacts::new([1.; 6]).unwrap(),
        },
    )
    .unwrap();
    let scale = ExtensionScale {
        interline: 16,
        standard_height: 8.,
        small_height: None,
    };
    let before = s.clone();
    let reg = r.snapshot();
    let foreign = SpotPool::prepare(
        &SystemSpots {
            system_id: 2,
            glyph_ids: vec![],
        },
        &r,
        &[],
    )
    .unwrap();
    assert!(
        s.extend_to_spot(
            &image,
            &mut r,
            SpotRequest {
                beam: 1,
                side: HorizontalSide::Left,
                max_extension: None
            },
            scale,
            &foreign
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), reg);
    let pool = SpotPool::prepare(
        &SystemSpots {
            system_id: 1,
            glyph_ids: vec![],
        },
        &r,
        &[],
    )
    .unwrap();
    assert!(
        s.extend_to_spot(
            &image,
            &mut r,
            SpotRequest {
                beam: 1,
                side: HorizontalSide::Left,
                max_extension: Some(-1)
            },
            scale,
            &pool
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), reg);
}
