// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_length_controls.rs"]
mod controls;
use audiveris_native::stem_length_map::{self, Item};
use std::collections::BTreeMap;
#[test]
fn original_612_builders_gap_profile_threshold_empty_sticky_and_fractional_controls() {
    let f = serde_json::from_str(include_str!(
        "../fixtures/official-stem-length-map/controls.json"
    ))
    .unwrap();
    assert_eq!(controls::observe(&f)["controls"], 11205);
}
#[test]
fn gap_order_first_stop_null_bounds_and_invalid_inputs() {
    let line = Some(Item {
        gap: false,
        contrib: 10,
        line: Some([2.5, 10., 2.5, 20.]),
        head_bounds: Some([1, 8, 3, 4]),
    });
    let gap = |c| {
        Some(Item {
            gap: true,
            contrib: c,
            line: None,
            head_bounds: None,
        })
    };
    let gaps = BTreeMap::from([(0, 0), (1, 3), (2, 7)]);
    let init = BTreeMap::from([(0, 77)]);
    assert_eq!(
        stem_length_map::retrieve_lengths(
            &[line.clone(), gap(3), line.clone()],
            1,
            10.,
            &gaps,
            1,
            &init
        )
        .unwrap(),
        BTreeMap::from([(0, 77), (1, 10), (2, 10)])
    );
    assert_eq!(
        stem_length_map::retrieve_lengths(
            &[line.clone(), gap(4), gap(100)],
            1,
            10.,
            &gaps,
            1,
            &init
        )
        .unwrap(),
        BTreeMap::from([(0, 77), (1, 10)])
    );
    // The inner loop stops by key order at the first non-exceeded threshold.
    let unordered = BTreeMap::from([(0, 7), (1, 0), (2, 3)]);
    assert_eq!(
        stem_length_map::retrieve_lengths(
            &[line.clone(), gap(4)],
            1,
            10.,
            &unordered,
            2,
            &BTreeMap::new()
        )
        .unwrap(),
        BTreeMap::from([(2, 10)])
    );
    let mut no_line = line.clone().unwrap();
    no_line.line = None;
    assert_eq!(
        stem_length_map::prefix_lengths(&[None, Some(no_line), gap(100), line.clone()], -1, -0.5)
            .unwrap(),
        vec![0, 0, 0, 0, -8]
    );
    assert_eq!(
        stem_length_map::line_bounds([2.5, 20., 2.5, 10.]).unwrap(),
        [2, 10, 1, 10]
    );
    assert_eq!(
        stem_length_map::line_bounds([2., 10., 2., 10.]).unwrap(),
        [2, 10, 0, 0]
    );
    assert!(stem_length_map::retrieve_lengths(&[None], 1, 0., &gaps, 0, &init).is_err());
    assert!(stem_length_map::retrieve_lengths(&[], 1, 0., &gaps, 4, &init).is_err());
    assert!(stem_length_map::prefix_lengths(&[], 0, 0.).is_err());
    assert!(stem_length_map::prefix_lengths(&[], 1, f64::NAN).is_err());
    assert!(stem_length_map::line_bounds([0., 0., f64::INFINITY, 0.]).is_err());
    if let Some(mut bad) = line {
        bad.head_bounds = Some([0, 0, 1, -1]);
        assert!(stem_length_map::prefix_lengths(&[Some(bad)], 1, 0.).is_err());
    }
    assert_eq!(init, BTreeMap::from([(0, 77)]));
}
