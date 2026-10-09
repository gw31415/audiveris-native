// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_beam_lookup_area_controls.rs"]
mod controls;
use audiveris_native::stem_beam_lookup_area::{self, Request};
#[test]
fn original_beam_areas_actual_borders_parts_and_ratio_ulp_controls_match() {
    let original = serde_json::from_str(include_str!(
        "../fixtures/official-stem-beam-lookup-area/controls.json"
    ))
    .unwrap();
    let result = controls::observe(&original);
    assert_eq!(result["cases"].as_array().unwrap().len(), 72);
}
#[test]
fn beam_area_input_bounds_skew_and_part_empty_contracts() {
    let r = Request {
        reference: [20., 50.],
        border: [0., 48., 100., 48.],
        direction: 1,
        slope: 0.,
        slope_margin: 0.015,
        half_lookup_dx: 3.3,
        max_seed_dy_ratio: 0.25,
        gap_pixels: 13,
        system_bounds: [0, 0, 100, 200],
        surrounding_part_bounds: vec![[0, 10, 100, 91]],
        limit: None,
    };
    let before = serde_json::to_value(&r).unwrap();
    assert_eq!(
        stem_beam_lookup_area::generate(&r)
            .unwrap()
            .theoretical_line,
        [20., 50., 20., 100.]
    );
    let mut bad = r.clone();
    bad.slope = 0.01;
    assert!(stem_beam_lookup_area::generate(&bad).is_err());
    bad = r.clone();
    bad.border[2] = bad.border[0];
    assert!(stem_beam_lookup_area::generate(&bad).is_err());
    bad = r.clone();
    bad.limit = Some([0., 0., 0., 100.]);
    assert!(stem_beam_lookup_area::generate(&bad).is_err());
    bad = r.clone();
    bad.gap_pixels = -1;
    assert!(stem_beam_lookup_area::generate(&bad).is_err());
    bad = r.clone();
    bad.half_lookup_dx = f64::INFINITY;
    assert!(stem_beam_lookup_area::generate(&bad).is_err());
    bad = r.clone();
    bad.surrounding_part_bounds = vec![[0, 10, 100, 91]; 4097];
    assert!(stem_beam_lookup_area::generate(&bad).is_err());
    // Empty neighborhoods use the original initial system edge. These are
    // native boundary contracts, separate from the actual original controls.
    bad = r.clone();
    bad.surrounding_part_bounds.clear();
    bad.direction = -1;
    assert_eq!(
        stem_beam_lookup_area::generate(&bad)
            .unwrap()
            .theoretical_line,
        [20., 50., 20., 200.]
    );
    assert_eq!(serde_json::to_value(&r).unwrap(), before);
}

#[test]
fn actual_reversed_beam_limit_and_nonzero_area_boundaries_match_original() {
    let original: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-nonconvex-area/controls.json"
    ))
    .unwrap();
    let mut points = 0;
    let mut rectangles = 0;
    for case in original["cases"].as_array().unwrap() {
        let polygon: Vec<[f64; 2]> = serde_json::from_value(case["polygon"].clone()).unwrap();
        let actual = if !case["request"].is_null() {
            let request: Request = serde_json::from_value(case["request"].clone()).unwrap();
            let before = serde_json::to_value(&request).unwrap();
            let actual = stem_beam_lookup_area::generate(&request).unwrap();
            assert_eq!(serde_json::to_value(&request).unwrap(), before);
            assert_eq!(
                serde_json::json!(actual.theoretical_line),
                case["theoretical_line"]
            );
            assert_eq!(actual.polygon, polygon);
            actual.polygon
        } else {
            polygon
        };
        let area = audiveris_area::LinearArea::polygon(&actual).unwrap();
        assert_eq!(
            serde_json::json!(area.bounds()),
            case["bounds"],
            "{}",
            case["name"]
        );
        assert_eq!(
            area.intersects_area(&area).unwrap(),
            !case["empty"].as_bool().unwrap(),
            "{}",
            case["name"]
        );
        for query in case["points"].as_array().unwrap() {
            let point = serde_json::from_value(query["point"].clone()).unwrap();
            assert_eq!(
                area.contains(point).unwrap(),
                query["contains"].as_bool().unwrap(),
                "{} point {:?}",
                case["name"],
                point
            );
            points += 1;
        }
        for query in case["rectangles"].as_array().unwrap() {
            let rectangle = serde_json::from_value(query["rectangle"].clone()).unwrap();
            assert_eq!(
                area.intersects_rect(rectangle).unwrap(),
                query["intersects"].as_bool().unwrap(),
                "{} rectangle {:?}",
                case["name"],
                rectangle
            );
            rectangles += 1;
        }
    }
    assert_eq!((points, rectangles), (658, 159));
}
