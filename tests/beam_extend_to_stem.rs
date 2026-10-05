// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Group, Registry, SystemSpots, glyph_from_json},
    beam_extend_to_beam::ExtensionScale,
    beam_extend_to_stem::{StemRequest, ThroughStem, lookup_stem, side_through_stem},
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
fn original_stem_selection_point_and_lifecycle_match() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extend-to-stem.json"
    ))
    .unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 126);
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
        for raw in input["seeds"].as_array().unwrap() {
            let mut g = raw.clone();
            g["orientation"] = json!("VERTICAL");
            let glyph = glyph_from_json(&g).unwrap();
            let id = r.register(&glyph).unwrap();
            ids.push(id);
            if raw["grouped"].as_bool().unwrap_or(true) {
                r.add_group(id, Group::VerticalSeed).unwrap();
            }
            if raw["owned"].as_bool().unwrap_or(true) {
                owner.glyph_ids.push(id);
            }
        }
        let cap = input["max_extension"].as_i64().map(|n| n as i32);
        let lookup = lookup_stem(&s.active(1).unwrap().item, &owner, &r, side, cap, 16).unwrap();
        assert_eq!(
            lookup.reach,
            c["reach"].as_i64().unwrap() as i32,
            "{i} reach"
        );
        assert_eq!(
            lookup.vertical_margin,
            c["vertical_margin"].as_i64().unwrap() as i32
        );
        let map = |id: usize| ids.iter().position(|n| *n == id).unwrap() + 1;
        assert_eq!(
            json!(lookup.ordered.iter().map(|id| map(*id)).collect::<Vec<_>>()),
            c["ordered"],
            "{i} order"
        );
        assert_eq!(
            json!(lookup.selected.map(map)),
            c["selected"],
            "{i} selected"
        );
        if let Some(point) = lookup.point {
            for j in 0..2 {
                let expected = &c["point"][j];
                if let Some(number) = expected.as_f64() {
                    assert!((point[j] - number).abs() < 1e-11, "{i} point {j}");
                } else {
                    let classification = if point[j].is_nan() {
                        "NaN"
                    } else if point[j].is_sign_positive() {
                        "Infinity"
                    } else {
                        "-Infinity"
                    };
                    assert_eq!(json!(classification), *expected, "{i} nonfinite point {j}");
                }
            }
        } else {
            assert!(c["point"].is_null());
        }
        let before = s.clone();
        let registry = r.snapshot();
        let scale = ExtensionScale {
            interline: 16,
            standard_height: 8.,
            small_height: None,
        };
        let result = if cap.is_none() || input["compose"].as_bool().unwrap_or(false) {
            match side_through_stem(&mut s, &image, &mut r, 1, side, scale, &owner).unwrap() {
                ThroughStem::StemExtended { prefix, extension }
                | ThroughStem::NeedSpot { prefix, extension } => {
                    let audiveris_native::beam_extension_prefix::SideDecision::NeedDownstream {
                        neighbor,
                        max_extension,
                        ..
                    } = prefix
                    else {
                        panic!("unexpected merge");
                    };
                    assert_eq!(
                        neighbor,
                        if input["compose"] == true {
                            Some(c["prefix_neighbor"].as_u64().unwrap() as usize)
                        } else if side == HorizontalSide::Right {
                            Some(2)
                        } else {
                            None
                        }
                    );
                    assert_eq!(
                        max_extension,
                        if input["compose"] == true {
                            Some(c["prefix_max_extension"].as_i64().unwrap() as i32)
                        } else if side == HorizontalSide::Right {
                            Some(36)
                        } else {
                            None
                        }
                    );
                    extension
                }
                ThroughStem::Merged(_) => panic!("unexpected merge"),
            }
        } else {
            s.extend_to_stem(
                &image,
                &mut r,
                StemRequest {
                    beam: 1,
                    side,
                    max_extension: cap,
                },
                ExtensionScale {
                    interline: 16,
                    standard_height: 8.,
                    small_height: None,
                },
                &owner,
            )
            .unwrap()
        };
        let got = result.appended;
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
    assert_eq!(accepted, 58);
}
#[test]
fn owner_mismatch_and_missing_seed_identity_preserve_state() {
    let image = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
    let mut s = SystemCandidates::empty(1);
    let mut r = Registry::default();
    let scale = ExtensionScale {
        interline: 16,
        standard_height: 8.,
        small_height: None,
    };
    let candidate = MergeCandidate {
        item: BeamItem {
            median: [30., 40., 70., 40.],
            height: 8.,
        },
        is_small: false,
        impacts: Impacts::new([1.; 6]).unwrap(),
    };
    s.register_merge(&image, &mut r, &candidate).unwrap();
    let before = s.clone();
    let registry = r.snapshot();
    assert!(
        s.extend_to_stem(
            &image,
            &mut r,
            StemRequest {
                beam: 1,
                side: HorizontalSide::Left,
                max_extension: None
            },
            scale,
            &SystemSpots {
                system_id: 2,
                glyph_ids: vec![]
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), registry);
    assert!(
        s.extend_to_stem(
            &image,
            &mut r,
            StemRequest {
                beam: 1,
                side: HorizontalSide::Left,
                max_extension: None
            },
            scale,
            &SystemSpots {
                system_id: 1,
                glyph_ids: vec![999]
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), registry);
}
#[test]
fn composition_stops_at_real_stem_or_pending_spot() {
    let image = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
    let mut s = SystemCandidates::empty(1);
    let mut r = Registry::default();
    let scale = ExtensionScale {
        interline: 16,
        standard_height: 8.,
        small_height: None,
    };
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
    let owner = SystemSpots {
        system_id: 1,
        glyph_ids: vec![],
    };
    let before = s.clone();
    let reg = r.snapshot();
    let got = side_through_stem(
        &mut s,
        &image,
        &mut r,
        1,
        HorizontalSide::Left,
        scale,
        &owner,
    )
    .unwrap();
    assert!(matches!(got, ThroughStem::NeedSpot { .. }));
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), reg);
}
#[test]
fn original_merge_prefix_success_skips_stem_branch() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extension-prefix.json"
    ))
    .unwrap();
    let mut checked = 0;
    for c in v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["input"]["mode"] == "side" && c["decisions"][0]["merged"] == true)
    {
        let input = &c["input"];
        let mut pixels = vec![255; 160 * 96];
        for p in input["black_pixels"].as_array().unwrap() {
            pixels[p[1].as_u64().unwrap() as usize * 160 + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(160, 96, pixels).unwrap();
        let seed = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
        let mut s = SystemCandidates::empty(1);
        let mut r = Registry::default();
        for b in input["beams"].as_array().unwrap() {
            s.register_merge(
                &seed,
                &mut r,
                &MergeCandidate {
                    item: BeamItem {
                        median: std::array::from_fn(|i| b["median"][i].as_f64().unwrap()),
                        height: 8.,
                    },
                    is_small: false,
                    impacts: Impacts::new([1.; 6]).unwrap(),
                },
            )
            .unwrap();
        }
        let result = side_through_stem(
            &mut s,
            &image,
            &mut r,
            input["target"].as_u64().unwrap() as usize,
            HorizontalSide::Left,
            ExtensionScale {
                interline: 16,
                standard_height: 8.,
                small_height: None,
            },
            &SystemSpots {
                system_id: 1,
                glyph_ids: vec![],
            },
        )
        .unwrap();
        assert!(matches!(result, ThroughStem::Merged(_)));
        for k in ["active_ids", "removed", "index_present", "edge_count"] {
            assert_eq!(snapshot(&s)[k], c["after"][k]);
        }
        checked += 1;
    }
    assert!(checked > 0);
}
