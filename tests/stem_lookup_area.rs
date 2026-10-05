// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_lookup_area_controls.rs"]
mod controls;
use audiveris_native::stem_lookup_area::{self, HeadRequest};
#[test]
fn original_head_area_actual_references_part_edges_and_limit_ulp_controls_match() {
    let original = serde_json::from_str(include_str!(
        "../fixtures/official-stem-lookup-area/controls.json"
    ))
    .unwrap();
    let result = controls::observe(&original);
    assert_eq!(result["cases"].as_array().unwrap().len(), 3000);
}
#[test]
fn area_refuses_skew_invalid_limits_and_crossing_without_mutation() {
    let r = HeadRequest {
        reference: [20., 10.],
        x_direction: 1,
        y_direction: 1,
        slope: 0.,
        slope_margin: 0.015,
        max_head_in_dx: 4,
        max_head_out_dx: 3,
        part_bounds: [0, 0, 100, 101],
        limit: None,
    };
    let before = serde_json::to_value(&r).unwrap();
    let g = stem_lookup_area::head(&r).unwrap();
    assert_eq!(g.theoretical_line, [20., 10., 20., 100.]);
    assert_eq!(
        g.polygon,
        vec![[23., 10.], [16., 10.], [14.65, 100.], [24.35, 100.]]
    );
    let mut bad = r.clone();
    bad.slope = 0.01;
    assert!(stem_lookup_area::head(&bad).is_err());
    bad = r.clone();
    bad.limit = Some([0., 0., 0., 100.]);
    assert!(stem_lookup_area::head(&bad).is_err());
    bad = r.clone();
    bad.reference[0] = f64::NAN;
    assert!(stem_lookup_area::head(&bad).is_err());
    bad = r.clone();
    bad.part_bounds[3] = 0;
    assert!(stem_lookup_area::head(&bad).is_err());
    bad = r.clone();
    bad.x_direction = 0;
    assert!(stem_lookup_area::head(&bad).is_err());
    bad = r.clone();
    bad.limit = Some([0., 10., 100., 10.]);
    assert!(stem_lookup_area::head(&bad).unwrap().polygon.is_empty());
    bad = r.clone();
    bad.limit = Some([0., -1000., 100., -1000.]);
    assert!(stem_lookup_area::head(&bad).is_err());
    assert_eq!(serde_json::to_value(&r).unwrap(), before);
}
