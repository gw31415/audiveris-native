// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
#[test]
fn original_single_foreground_pixel_dilation_preserves_integer_point_size_scaling() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-head-cleaner-dilation/observed.json"
    ))
    .unwrap();
    for case in source["cases"].as_array().unwrap() {
        let margin = head_dilation(case["point_size"].as_u64().unwrap() as u16);
        let point: [i64; 2] =
            serde_json::from_value(case["single_foreground_point"].clone()).unwrap();
        let side = 2 * margin + 1;
        assert_eq!(
            json!([point[0] - margin, point[1] - margin, side, side]),
            case["dilated_bounds"]
        );
        assert_eq!(json!(side * side), case["dilated_pixels"]);
    }
}
