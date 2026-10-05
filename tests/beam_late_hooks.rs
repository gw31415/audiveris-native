// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Group, Registry, SystemSpots},
    beam_extend_to_beam::ExtensionScale,
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn compare(a: &Value, b: &Value) {
    match b {
        Value::Number(_) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-8,
            "{a} != {b}"
        ),
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                compare(x, y);
            }
        }
        Value::Object(v) => {
            for (k, y) in v {
                compare(&a[k], y);
            }
        }
        _ => assert_eq!(a, b),
    }
}
#[test]
fn uninterrupted_original_late_hooks_match_order_geometry_grade_and_assignment() {
    let data: Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-late-hooks.json")).unwrap();
    assert_eq!(data["native_outputs_used"], false);
    assert_eq!(data["cases"].as_array().unwrap().len(), 66);
    let mut accepted = 0;
    for (i, c) in data["cases"].as_array().unwrap().iter().enumerate() {
        let input = &c["input"];
        let seed = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
        let mut pixels = vec![255; 160 * 96];
        for b in input["rectangles"].as_array().unwrap() {
            let x = b[0].as_u64().unwrap() as usize;
            let y = b[1].as_u64().unwrap() as usize;
            let w = b[2].as_u64().unwrap() as usize;
            let h = b[3].as_u64().unwrap() as usize;
            for yy in y..y + h {
                for xx in x..x + w {
                    pixels[yy * 160 + xx] = 0;
                }
            }
        }
        let image = GrayImage::new(160, 96, pixels).unwrap();
        let mut state = SystemCandidates::empty(1);
        let mut registry = Registry::default();
        for b in input["beams"].as_array().unwrap() {
            state
                .register_merge(
                    &seed,
                    &mut registry,
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
                            b["distance"].as_f64().unwrap(),
                        ])
                        .unwrap(),
                    },
                )
                .unwrap();
        }
        let mut owner = SystemSpots {
            system_id: 1,
            glyph_ids: Vec::new(),
        };
        for g in input["spots"].as_array().unwrap() {
            let glyph = audiveris_native::beam_dispatch::glyph_from_json(
                &json!({"bounds":g["bounds"],"orientation":"HORIZONTAL","runs":g["runs"]}),
            )
            .unwrap();
            let id = registry.register(&glyph).unwrap();
            registry.add_group(id, Group::BeamSpot).unwrap();
            owner.glyph_ids.push(id);
        }
        let scale = ExtensionScale {
            interline: 16,
            standard_height: 8.,
            small_height: if input["small_parameters"] == true {
                Some(5.)
            } else {
                None
            },
        };
        let report = audiveris_native::beam_late_hooks::build_hooks(
            &mut state,
            &image,
            &mut registry,
            scale,
            &owner,
        )
        .unwrap();
        assert_eq!(
            report.initial_ids.len(),
            input["beams"].as_array().unwrap().len()
        );
        let raw=json!(state.inters().iter().map(|v|json!({"kind":v.kind,"median":v.item.median,"height":v.item.height,"grade":v.impacts.grade})).collect::<Vec<_>>());
        compare(&raw, &c["raw"]);
        let assigned = state
            .assigned_spot_ids()
            .iter()
            .map(|id| owner.glyph_ids.iter().position(|v| v == id).unwrap() + 1)
            .collect::<Vec<_>>();
        assert_eq!(json!(assigned), c["assigned"], "case {i} assignment/order");
        assert!(
            state.inters().iter().all(|v| !v.is_removed()),
            "case {i} removed parent"
        );
        accepted += report
            .attempts
            .iter()
            .filter(|x| x.appended.is_some())
            .count();
    }
    assert!(accepted > 20, "accepted {accepted}");
    println!("66 original cases, {accepted} accepted hooks");
}
#[test]
fn inconsistent_late_hook_owner_is_rejected_before_mutation() {
    let image = GrayImage::new(160, 96, vec![255; 160 * 96]).unwrap();
    let mut state = SystemCandidates::empty(1);
    let before = state.clone();
    let mut registry = Registry::default();
    assert!(
        audiveris_native::beam_late_hooks::build_hooks(
            &mut state,
            &image,
            &mut registry,
            ExtensionScale {
                interline: 16,
                standard_height: 8.,
                small_height: None
            },
            &SystemSpots {
                system_id: 2,
                glyph_ids: vec![]
            }
        )
        .is_err()
    );
    assert_eq!(state, before);
    assert!(registry.is_empty());
}
