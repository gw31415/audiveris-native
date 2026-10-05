// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::beam_extension_geometry::middle_area;

#[test]
fn weighted_height_and_swapped_arguments_keep_exact_horizontal_edges() {
    let a = [0., 10., 10., 10.];
    let b = [14., 10., 34., 10.];
    for (m1, h1, m2, h2) in [(a, 2., b, 8.), (b, 8., a, 2.)] {
        let area = middle_area(m1, h1, m2, h2).unwrap();
        assert_eq!(area.bounds(), [10, 7, 4, 6]);
        assert!(area.contains([10., 7.]).unwrap());
        assert!(!area.contains([14., 10.]).unwrap());
        assert!(!area.contains([12., 13.]).unwrap());
    }
}

#[test]
fn sloped_bridge_uses_nearest_endpoints_in_both_argument_orders() {
    let a = [0., 0., 10., 5.];
    let b = [14., 9., 34., 19.];
    for (first, second) in [(a, b), (b, a)] {
        let area = middle_area(first, 4., second, 4.).unwrap();
        assert_eq!(area.bounds(), [10, 3, 4, 8]);
        assert!(area.contains([12., 7.]).unwrap());
        assert!(!area.contains([12., 9.5]).unwrap());
    }
}

#[test]
fn touching_zero_height_overlap_and_equal_starts_retain_original_geometry() {
    let a = [0., 10., 10., 10.];
    assert_eq!(
        middle_area(a, 2., [10., 10., 20., 10.], 2.)
            .unwrap()
            .bounds(),
        [0, 0, 0, 0]
    );
    assert_eq!(
        middle_area(a, 0., [14., 10., 20., 10.], 0.)
            .unwrap()
            .bounds(),
        [0, 0, 0, 0]
    );
    let overlap = middle_area(a, 2., [8., 10., 20., 10.], 2.).unwrap();
    assert_eq!(overlap.bounds(), [8, 9, 2, 2]);
    assert!(overlap.contains([9., 10.]).unwrap());
    let b = [0., 10., 20., 10.];
    assert_eq!(middle_area(a, 2., b, 2.).unwrap().bounds(), [0, 9, 20, 2]);
    assert_eq!(middle_area(b, 2., a, 2.).unwrap().bounds(), [0, 9, 10, 2]);
}

#[test]
fn unsupported_inputs_and_generated_vertices_are_rejected() {
    let a = [0., 10., 10., 10.];
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 16_000_001.] {
        for i in 0..4 {
            let mut b = [14., 10., 34., 10.];
            b[i] = bad;
            assert!(middle_area(a, 2., b, 2.).is_err());
            assert!(middle_area(b, 2., a, 2.).is_err());
        }
    }
    for height in [-1., f64::NAN, f64::INFINITY, 16_000_001.] {
        assert!(middle_area(a, height, a, 2.).is_err());
        assert!(middle_area(a, 2., a, height).is_err());
    }
    for b in [[14., 10., 14., 10.], [34., 10., 14., 10.]] {
        assert!(middle_area(a, 2., b, 2.).is_err());
    }
    assert!(
        middle_area(
            [0., 16_000_000., 1., 16_000_000.],
            2.,
            [2., 16_000_000., 3., 16_000_000.],
            2.
        )
        .is_err()
    );
}

#[test]
fn direct_official_middle_area_bounds_and_membership_match_exactly() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-middle-area.json")).unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 100);
    let mut queries = 0;
    for (index, case) in cases.iter().enumerate() {
        let first = std::array::from_fn(|i| case["first"][i].as_f64().unwrap());
        let second = std::array::from_fn(|i| case["second"][i].as_f64().unwrap());
        let area = middle_area(
            first,
            case["heights"][0].as_f64().unwrap(),
            second,
            case["heights"][1].as_f64().unwrap(),
        )
        .unwrap();
        let bounds: [i32; 4] = std::array::from_fn(|i| case["bounds"][i].as_i64().unwrap() as i32);
        assert_eq!(area.bounds(), bounds, "case {index} bounds");
        for query in case["queries"].as_array().unwrap() {
            let point = [query[0].as_f64().unwrap(), query[1].as_f64().unwrap()];
            assert_eq!(
                area.contains(point).unwrap(),
                query[2].as_bool().unwrap(),
                "case {index} at {point:?}"
            );
            queries += 1;
        }
    }
    assert_eq!(queries, fixture["query_count"].as_u64().unwrap());
}
