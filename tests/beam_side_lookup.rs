// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_extension_geometry::HorizontalSide,
    beam_side_lookup::{LookupBeam, get_side_beam},
    beam_structure::BeamItem,
};
#[test]
fn original_side_lookup_and_rectangle_intersections_match() {
    let f: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-side-lookup.json")).unwrap();
    assert_eq!(f["native_outputs_used"], false);
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 300);
    let mut selected = 0;
    for (i, c) in cases.iter().enumerate() {
        let raw: Vec<_> = c["raw"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| LookupBeam {
                id: b["id"].as_u64().unwrap() as usize,
                item: BeamItem {
                    median: std::array::from_fn(|j| b["median"][j].as_f64().unwrap()),
                    height: b["height"].as_f64().unwrap(),
                },
                bounds: std::array::from_fn(|j| b["bounds"][j].as_i64().unwrap() as i32),
                removed: b["removed"].as_bool().unwrap(),
            })
            .collect();
        let before = raw.clone();
        let side = if c["side"] == "LEFT" {
            HorizontalSide::Left
        } else {
            HorizontalSide::Right
        };
        let il = c["interline"].as_u64().unwrap() as usize;
        assert_eq!(c["max_slope_difference"], 0.065);
        assert_eq!(
            c["max_y_gap"].as_f64().unwrap(),
            (0.25 * il as f64).round_ties_even()
        );
        assert_eq!(c["default_reach"], 4 * il);
        let got = get_side_beam(
            &raw,
            c["target"].as_u64().unwrap() as usize,
            side,
            c["reach"].as_f64(),
            il,
        )
        .unwrap();
        assert_eq!(
            serde_json::json!(got.intersected),
            c["intersected"],
            "case {i} intersections"
        );
        assert_eq!(
            serde_json::json!(got.selected),
            c["selected"],
            "case {i} selected"
        );
        assert_eq!(
            serde_json::json!(got.ordered),
            c["ordered"],
            "case {i} ordered"
        );
        assert_eq!(raw, before);
        selected += usize::from(got.selected.is_some());
    }
    assert!(selected > 0 && selected < 300);
    println!(
        "original side lookup cases=300 selected={selected} rejected={}",
        300 - selected
    );
}

#[test]
fn typed_state_lookup_preserves_state_and_ignores_removed_candidates() {
    use audiveris_native::{
        GrayImage, beam_candidates::SystemCandidates, beam_dispatch::Registry,
        beam_impacts::Impacts, beam_merge::MergeCandidate,
    };
    let image = GrayImage::new(100, 50, vec![0; 5000]).unwrap();
    let mut registry = Registry::default();
    let mut state = SystemCandidates::empty(1);
    for (a, b) in [(10., 30.), (34., 50.), (54., 70.)] {
        state
            .register_merge(
                &image,
                &mut registry,
                &MergeCandidate {
                    item: BeamItem {
                        median: [a, 20., b, 20.],
                        height: 4.,
                    },
                    is_small: false,
                    impacts: Impacts::new([1.; 6]).unwrap(),
                },
            )
            .unwrap();
    }
    let before = state.clone();
    let groups = registry.snapshot();
    assert_eq!(
        state
            .lookup_side(1, HorizontalSide::Right, None, 16)
            .unwrap()
            .selected,
        Some(2)
    );
    assert_eq!(state, before);
    assert_eq!(registry.snapshot(), groups);
    state.remove(2).unwrap();
    assert_eq!(
        state
            .lookup_side(1, HorizontalSide::Right, None, 16)
            .unwrap()
            .selected,
        Some(3)
    );
}
#[test]
fn invalid_inputs_fail_and_equal_gap_order_is_stable() {
    let item = BeamItem {
        median: [10., 20., 30., 20.],
        height: 4.,
    };
    let a = LookupBeam {
        id: 1,
        item: item.clone(),
        bounds: [10, 18, 20, 4],
        removed: false,
    };
    let b = LookupBeam {
        id: 2,
        item: BeamItem {
            median: [34., 20., 50., 20.],
            height: 4.,
        },
        bounds: [34, 18, 16, 4],
        removed: false,
    };
    let mut c = b.clone();
    c.id = 3;
    let raw = [a.clone(), b.clone(), c.clone()];
    assert_eq!(
        get_side_beam(&raw, 1, HorizontalSide::Right, None, 16)
            .unwrap()
            .ordered,
        vec![2, 3]
    );
    assert_eq!(
        get_side_beam(
            &[a.clone(), c, b.clone()],
            1,
            HorizontalSide::Right,
            None,
            16
        )
        .unwrap()
        .ordered,
        vec![3, 2]
    );
    assert!(get_side_beam(&[a.clone(), a], 1, HorizontalSide::Right, None, 16).is_err());
    assert!(get_side_beam(&raw, 1, HorizontalSide::Right, Some(f64::NAN), 16).is_err());
    assert!(get_side_beam(&raw, 99, HorizontalSide::Right, None, 16).is_err());
}
