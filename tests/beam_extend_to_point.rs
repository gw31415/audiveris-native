// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, beam_candidates::SystemCandidates, beam_dispatch::Registry,
    beam_extend_to_beam::ExtensionScale, beam_extend_to_point::point_gate,
    beam_extension_geometry::HorizontalSide, beam_impacts::Impacts, beam_merge::MergeCandidate,
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn snapshot(s: &SystemCandidates) -> Value {
    json!({"active_ids":s.inters().iter().filter(|v|!v.is_removed()).map(|v|v.id()).collect::<Vec<_>>(),"removed":s.inters().iter().map(|v|v.is_removed()).collect::<Vec<_>>(),"index_present":s.inters().iter().map(|v|s.active(v.id()).is_some()).collect::<Vec<_>>(),"edge_count":s.exclusions().len()})
}
#[test]
fn original_point_method_geometry_grade_and_lifecycle_match() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extend-to-point.json"
    ))
    .unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 38);
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
        let point = std::array::from_fn(|i| input["point"][i].as_f64().unwrap());
        let gate = point_gate(&image, &s.active(1).unwrap().item, side, point).unwrap();
        assert_eq!(json!(gate.gap), c["gap"], "{i} gap");
        assert_eq!(json!(gate.counts), c["counts"], "{i} mask");
        assert_eq!(json!(gate.accepted), c["gate"], "{i} gate");
        let before = s.clone();
        let registry = r.snapshot();
        let got = s
            .extend_to_point(
                &image,
                &mut r,
                1,
                side,
                point,
                ExtensionScale {
                    interline: 16,
                    standard_height: 8.,
                    small_height: input["small_height"].as_f64(),
                },
            )
            .unwrap();
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
    assert_eq!(accepted, 17);
}
#[test]
fn invalid_target_and_point_preserve_state() {
    let image = GrayImage::new(16, 16, vec![255; 256]).unwrap();
    let mut s = SystemCandidates::empty(1);
    let before = s.clone();
    let mut r = Registry::default();
    assert!(
        s.extend_to_point(
            &image,
            &mut r,
            99,
            HorizontalSide::Left,
            [0., 0.],
            ExtensionScale {
                interline: 16,
                standard_height: 8.,
                small_height: None
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert!(
        point_gate(
            &image,
            &BeamItem {
                median: [2., 8., 12., 8.],
                height: 4.
            },
            HorizontalSide::Left,
            [f64::NAN, 0.]
        )
        .is_err()
    );
}
