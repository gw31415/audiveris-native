// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::Registry,
    beam_extend_to_beam::{ExtensionScale, gap_gate},
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn item(v: &Value) -> BeamItem {
    BeamItem {
        median: std::array::from_fn(|i| v["median"][i].as_f64().unwrap()),
        height: v["height"].as_f64().unwrap(),
    }
}
fn snapshot(s: &SystemCandidates) -> Value {
    json!({"active_ids":s.inters().iter().filter(|v|!v.is_removed()).map(|v|v.id()).collect::<Vec<_>>(),
        "removed":s.inters().iter().map(|v|v.is_removed()).collect::<Vec<_>>(),
        "index_present":s.inters().iter().map(|v|s.active(v.id()).is_some()).collect::<Vec<_>>(),
        "edge_count":s.exclusions().len()})
}
#[test]
fn independent_original_gap_gate_and_extend_lifecycle_match() {
    let official: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extend-to-beam.json"
    ))
    .unwrap();
    assert_eq!(official["native_outputs_used"], false);
    let cases = official["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 61);
    let mut accepted = 0;
    for (index, c) in cases.iter().enumerate() {
        let input = &c["input"];
        let (w, h) = (
            input["width"].as_u64().unwrap() as usize,
            input["height"].as_u64().unwrap() as usize,
        );
        let mut pixels = vec![255; w * h];
        for p in input["black_pixels"].as_array().unwrap() {
            pixels[p[1].as_u64().unwrap() as usize * w + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(w, h, pixels).unwrap();
        let a = item(&input["first"]);
        let b = item(&input["second"]);
        let il = input["interline"].as_u64().unwrap() as usize;
        let gate = gap_gate(&image, &a, &b, il).unwrap();
        assert_eq!(gate.gap, c["gap"].as_f64().unwrap(), "{index} gap");
        assert_eq!(json!(gate.minimum_gap), c["minimum_gap"], "{index} minimum");
        assert_eq!(json!(gate.counts), c["counts"], "{index} mask total/black");
        assert_eq!(json!(gate.accepted), c["gate"], "{index} gate");
        if c["accepted"].is_null() {
            continue;
        }
        let mut state = SystemCandidates::empty(1);
        let mut registry = Registry::default();
        let seed_image = GrayImage::new(w, h, vec![0; w * h]).unwrap();
        for (key, geometry) in [("first", a), ("second", b)] {
            let mut values = [1.; 6];
            values[5] = input[key]["distance_impact"].as_f64().unwrap();
            state
                .register_merge(
                    &seed_image,
                    &mut registry,
                    &MergeCandidate {
                        item: geometry,
                        is_small: false,
                        impacts: Impacts::new(values).unwrap(),
                    },
                )
                .unwrap();
        }
        state.add_exclusion(1, 2).unwrap();
        let before = state.clone();
        let reg = registry.snapshot();
        let got = state
            .extend_to_beam(
                &image,
                &mut registry,
                1,
                2,
                ExtensionScale {
                    interline: il,
                    standard_height: 8.,
                    small_height: None,
                },
            )
            .unwrap();
        assert_eq!(json!(got.is_some()), c["accepted"], "{index} acceptance");
        for key in ["active_ids", "removed", "index_present", "edge_count"] {
            assert_eq!(
                snapshot(&before)[key],
                c["before"][key],
                "{index} before {key}"
            );
            assert_eq!(
                snapshot(&state)[key],
                c["after"][key],
                "{index} after {key}"
            );
        }
        if let Some(id) = got {
            accepted += 1;
            assert_eq!(id, 3);
            let merged = state.active(id).unwrap();
            for i in 0..4 {
                assert!(
                    (merged.item.median[i] - c["merged"]["median"][i].as_f64().unwrap()).abs()
                        < 1e-8,
                    "{index} median"
                );
            }
            assert!((merged.item.height - c["merged"]["height"].as_f64().unwrap()).abs() < 1e-8);
            assert!((merged.impacts.grade - c["merged"]["grade"].as_f64().unwrap()).abs() < 1e-8);
        } else {
            assert_eq!(state, before, "{index} rejection no-op");
            assert_eq!(registry.snapshot(), reg, "{index} registry no-op");
        }
        let ids: Vec<_> = state.inters().iter().map(|v| v.glyph_id()).collect();
        for i in 0..ids.len() {
            for j in 0..ids.len() {
                assert_eq!(
                    ids[i] == ids[j],
                    c["after"]["glyph_ids"][i] == c["after"]["glyph_ids"][j],
                    "{index} glyph identity"
                );
            }
        }
    }
    assert_eq!(accepted, 26);
}
#[test]
fn invalid_endpoints_fail_before_state_or_registry_mutation() {
    let image = GrayImage::new(100, 50, vec![0; 5000]).unwrap();
    let mut state = SystemCandidates::empty(1);
    let mut registry = Registry::default();
    let before = state.clone();
    let reg = registry.snapshot();
    for (a, b) in [(1, 1), (1, 2)] {
        assert!(
            state
                .extend_to_beam(
                    &image,
                    &mut registry,
                    a,
                    b,
                    ExtensionScale {
                        interline: 16,
                        standard_height: 8.,
                        small_height: None
                    }
                )
                .is_err()
        );
    }
    assert_eq!(state, before);
    assert_eq!(registry.snapshot(), reg);
}
