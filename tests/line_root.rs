// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::bar_semantics::line_root_correction;
use serde_json::Value;
#[test]
fn controlled_original_projector_positive_root_correction() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/line-root-cases.json"
    ))
    .unwrap();
    let mut positive = 0;
    for case in oracle["cases"].as_array().unwrap() {
        let correction = line_root_correction(
            &oracle,
            case["first_start"].as_i64(),
            case["has_start"].as_bool().unwrap(),
            case["has_brace"].as_bool().unwrap(),
            oracle["interline"].as_u64().unwrap() as usize,
        );
        let final_left = correction.unwrap_or(case["initial_left"].as_i64().unwrap());
        assert_eq!(final_left, case["final_left"].as_i64().unwrap());
        assert_eq!(
            case["has_start"].as_bool().unwrap() && correction.is_none(),
            case["final_has_start"].as_bool().unwrap()
        );
        if correction.is_some() {
            positive += 1;
        }
    }
    assert_eq!(
        positive, 1,
        "oracle must exercise the correction, not only no-op cases"
    );
}
