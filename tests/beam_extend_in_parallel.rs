// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Geometry, Registry, SystemSpots},
    beam_extend_in_parallel::{ParallelOwner, ThroughParallel, side_through_parallel},
    beam_extend_to_beam::ExtensionScale,
    beam_extension_geometry::HorizontalSide,
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn geometry(w: usize, h: usize) -> Geometry {
    let grid = json!({"staffs":[{"id":1,"system_id":1,"lines":(0..5).map(|i|json!({"points":[[0,10+4*i],[w,10+4*i]]})).collect::<Vec<_>>()}],"systems":[{"id":1,"staff_ids":[1],"coordinates":[0,10,w,30,0],"area":{"x_range":[0,w],"vertical_margin":14.0,"north_neighbor_last_line":null,"south_neighbor_first_line":null}}]});
    Geometry::from_grid(w, h, &grid, 16).unwrap()
}
fn snapshot(s: &SystemCandidates) -> Value {
    json!({"active_ids":s.inters().iter().filter(|v|!v.is_removed()).map(|v|v.id()).collect::<Vec<_>>(),"removed":s.inters().iter().map(|v|v.is_removed()).collect::<Vec<_>>(),"index_present":s.inters().iter().map(|v|s.active(v.id()).is_some()).collect::<Vec<_>>(),"edge_count":s.exclusions().len()})
}
#[test]
fn original_parallel_scan_geometry_order_and_point_lifecycle_match() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extend-in-parallel.json"
    ))
    .unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 104);
    let mut accepted = 0;
    for (i, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let input = &c["input"];
        let seed = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
        let mut pixels = vec![255; 160 * 96];
        for p in input["black_pixels"].as_array().unwrap() {
            pixels[p[1].as_u64().unwrap() as usize * 160 + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(160, 96, pixels).unwrap();
        let mut s = SystemCandidates::empty(1);
        let mut r = Registry::default();
        for b in input["beams"].as_array().unwrap() {
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
        if let Some(id) = input["pre_removed"].as_u64() {
            s.remove(id as usize).unwrap();
        }
        for id in 2..=s.inters().len() {
            if s.active(id).is_some() {
                s.add_exclusion(1, id).unwrap();
            }
        }
        let before = s.clone();
        let reg = r.snapshot();
        let side = if input["side"] == "LEFT" {
            HorizontalSide::Left
        } else {
            HorizontalSide::Right
        };
        let scale = ExtensionScale {
            interline: 16,
            standard_height: 8.,
            small_height: None,
        };
        let geometry = geometry(
            input["system_size"][0].as_u64().unwrap() as usize,
            input["system_size"][1].as_u64().unwrap() as usize,
        );
        let report = if input["compose"] == true {
            assert_eq!(c["earlier_stem_spot_rejected"], true);
            match side_through_parallel(
                &mut s,
                &image,
                &mut r,
                1,
                side,
                scale,
                ParallelOwner {
                    free_glyphs: &SystemSpots {
                        system_id: 1,
                        glyph_ids: vec![],
                    },
                    geometry: &geometry,
                },
            )
            .unwrap()
            {
                ThroughParallel::FinishedSide { extension, .. } => extension,
                _ => panic!("unexpected earlier branch"),
            }
        } else {
            s.extend_in_parallel(&image, &mut r, 1, side, scale, &geometry)
                .unwrap()
        };
        assert_eq!(
            json!(report.ordered),
            c["ordered"],
            "{i} raw intersection order"
        );
        assert_eq!(
            json!(report.attempted_point),
            c["attempted_point"],
            "{i} final attempted candidate"
        );
        assert_eq!(
            json!(report.appended.is_some()),
            c["accepted"],
            "{i} accepted"
        );
        assert_eq!(
            report.attempts.len(),
            c["attempts"].as_array().unwrap().len(),
            "{i} attempt count"
        );
        for (a, b) in report
            .attempts
            .iter()
            .zip(c["attempts"].as_array().unwrap())
        {
            assert_eq!(json!(a.candidate), b["candidate"]);
            assert_eq!(json!(a.outcome), b["outcome"], "{i} reason");
            assert_eq!(json!(a.gap), b["gap"], "{i} gap");
            assert_eq!(json!(a.obstruction), b["obstruction"]);
            if let Some(p) = a.point {
                for j in 0..2 {
                    assert!((p[j] - b["point"][j].as_f64().unwrap()).abs() < 1e-11);
                }
            } else {
                assert!(b["point"].is_null());
            }
        }
        for k in ["active_ids", "removed", "index_present", "edge_count"] {
            assert_eq!(snapshot(&s)[k], c["after"][k], "{i} {k}");
        }
        if let Some(id) = report.appended {
            accepted += 1;
            let new = s.active(id).unwrap();
            assert_eq!(json!(new.item.median), c["merged"]["median"]);
            assert_eq!(json!(new.item.height), c["merged"]["height"]);
            assert!((new.impacts.grade - c["merged"]["grade"].as_f64().unwrap()).abs() < 1e-12);
            for old in before
                .inters()
                .iter()
                .filter(|v| !v.is_removed() && v.id() != 1)
            {
                assert!(s.active(old.id()).is_some());
            }
        } else {
            assert_eq!(s, before, "{i} no-op state");
            assert_eq!(r.snapshot(), reg, "{i} no-op registry");
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
fn invalid_target_is_a_noop_and_system_boundary_uses_both_endpoints() {
    let image = GrayImage::new(160, 96, vec![255; 160 * 96]).unwrap();
    let mut s = SystemCandidates::empty(1);
    let before = s.clone();
    let mut r = Registry::default();
    let g = geometry(160, 44);
    assert!(!g.is_beam_side_in_system(1, [24., 40.], 8.).unwrap());
    assert!(g.is_beam_side_in_system(1, [24., 39.999], 8.).unwrap());
    assert!(
        s.extend_in_parallel(
            &image,
            &mut r,
            1,
            HorizontalSide::Left,
            ExtensionScale {
                interline: 16,
                standard_height: 8.,
                small_height: None
            },
            &g
        )
        .is_err()
    );
    assert_eq!(s, before);
}
