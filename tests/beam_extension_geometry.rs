// SPDX-License-Identifier: AGPL-3.0-or-later
// Hand-calculated controls plus separately generated direct official controls.
use audiveris_native::beam_extension_geometry::{HorizontalSide::*, side_area_of};

#[test]
fn direct_official_side_area_bounds_and_membership_match_exactly() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-side-area.json")).unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 120);
    let mut queries = 0;
    for (index, case) in cases.iter().enumerate() {
        let median = std::array::from_fn(|i| case["median"][i].as_f64().unwrap());
        let p: [f64; 4] = std::array::from_fn(|i| case["parameters"][i].as_f64().unwrap());
        let side = match case["side"].as_str().unwrap() {
            "LEFT" => Left,
            "RIGHT" => Right,
            other => panic!("unknown original side {other}"),
        };
        let area = side_area_of(median, p[0], side, p[1], p[2], p[3]).unwrap();
        let expected_bounds: [i32; 4] =
            std::array::from_fn(|i| case["bounds"][i].as_i64().unwrap() as i32);
        assert_eq!(area.bounds(), expected_bounds, "case {index} bounds");
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

#[test]
fn horizontal_sides_preserve_pixel_offset_and_half_open_edges() {
    let median = [10., 20., 30., 20.];
    for (side, bounds, inside, outside) in [
        (Left, [5, 17, 4, 6], [5., 17.], [9., 20.]),
        (Right, [31, 17, 4, 6], [31., 17.], [35., 20.]),
    ] {
        let area = side_area_of(median, 4., side, 1., 5., 0.).unwrap();
        assert_eq!(area.bounds(), bounds);
        assert!(area.contains(inside).unwrap());
        assert!(!area.contains(outside).unwrap());
        assert!(!area.contains([inside[0], 23.]).unwrap());
        assert!(area.contains([inside[0], 22.999]).unwrap());
    }
}

#[test]
fn positive_and_negative_slopes_use_parallelograms_on_both_sides() {
    // y = 15 + x/2 and y = 25 - x/2; half-height = 1.
    for (median, side, bounds, inside, outside) in [
        (
            [10., 20., 30., 30.],
            Left,
            [4, 16, 5, 5],
            [6., 18.],
            [6., 19.5],
        ),
        (
            [10., 20., 30., 30.],
            Right,
            [31, 29, 5, 5],
            [32., 31.],
            [32., 32.5],
        ),
        (
            [10., 20., 30., 10.],
            Left,
            [4, 19, 5, 5],
            [6., 22.],
            [6., 23.5],
        ),
        (
            [10., 20., 30., 10.],
            Right,
            [31, 6, 5, 5],
            [32., 9.],
            [32., 10.5],
        ),
    ] {
        let area = side_area_of(median, 2., side, 0., 6., 0.).unwrap();
        assert_eq!(area.bounds(), bounds);
        assert!(area.contains(inside).unwrap());
        assert!(!area.contains(outside).unwrap());
    }
}

#[test]
fn fractional_overlap_crosses_endpoint_and_keeps_integer_bounds() {
    let median = [10.25, 20.25, 30.25, 20.25];
    for (side, bounds, included, excluded) in [
        (Left, [5, 18, 7, 4], [11.749, 20.25], [11.75, 20.25]),
        (Right, [28, 18, 7, 4], [28.75, 20.25], [28.749, 20.25]),
    ] {
        let area = side_area_of(median, 3., side, 0.25, 4.5, 2.5).unwrap();
        assert_eq!(area.bounds(), bounds);
        assert!(area.contains(included).unwrap());
        assert!(!area.contains(excluded).unwrap());
    }
}

#[test]
fn collapsed_areas_are_empty_but_zero_reach_is_not_clamped() {
    let median = [10., 20., 30., 20.];
    for side in [Left, Right] {
        for (height, reach) in [(0., 5.), (2., 1.)] {
            let area = side_area_of(median, height, side, 0., reach, 0.).unwrap();
            assert_eq!(area.bounds(), [0, 0, 0, 0]);
            assert!(!area.contains([10., 20.]).unwrap());
        }
        let area = side_area_of(median, 2., side, 0., 0., 0.).unwrap();
        let x = if side == Left { 9. } else { 30. };
        assert_eq!(area.bounds(), [x as i32, 19, 1, 2]);
        assert!(area.contains([x + 0.5, 20.]).unwrap());
    }
}

#[test]
fn invalid_or_out_of_bounds_geometry_is_rejected() {
    let good = [10., 20., 30., 20.];
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 16_000_001.] {
        for i in 0..4 {
            let mut median = good;
            median[i] = bad;
            assert!(side_area_of(median, 2., Left, 0., 5., 0.).is_err());
        }
    }
    for bad in [f64::NAN, f64::INFINITY, -1., 16_000_001.] {
        for i in 0..4 {
            let mut params = [2., 0., 5., 0.];
            params[i] = bad;
            assert!(side_area_of(good, params[0], Left, params[1], params[2], params[3]).is_err());
        }
    }
    for median in [[10., 20., 10., 30.], [30., 20., 10., 20.]] {
        assert!(side_area_of(median, 2., Left, 0., 5., 0.).is_err());
    }
    // Inputs can be valid while extrapolated points leave the bounded domain.
    assert!(side_area_of([0., 0., 1., 16_000_000.], 2., Right, 0., 2., 0.).is_err());
}
