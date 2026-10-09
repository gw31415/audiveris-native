// SPDX-License-Identifier: AGPL-3.0-or-later
// Proposal: include this file from fit.rs after copying the projected Java fixture.
#[cfg(test)]
mod original_source_observations {
    use super::{Pixel, fit_short};
    use serde_json::Value;

    fn scalar(actual: f64, expected: &Value, tolerance: f64, case: usize, field: &str) {
        match expected
            .as_str()
            .expect("original scalar decimal or nonfinite marker")
        {
            "NaN" => assert!(actual.is_nan(), "case {case} {field}: {actual:?} != NaN"),
            "Infinity" => assert_eq!(actual, f64::INFINITY, "case {case} {field}"),
            "-Infinity" => assert_eq!(actual, f64::NEG_INFINITY, "case {case} {field}"),
            decimal => {
                let expected = decimal.parse::<f64>().expect("finite original Java scalar");
                assert!(expected.is_finite());
                assert!(
                    actual.is_finite(),
                    "case {case} {field}: {actual:?} != {expected:?}"
                );
                assert!(
                    (actual - expected).abs() <= tolerance,
                    "case {case} {field}: {actual:?} != {expected:?}; tolerance {tolerance}"
                );
            }
        }
    }

    #[test]
    fn actual_java_sequences_preserve_nullable_geometry_and_math_decisions() {
        let original: Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/official-short-slur-fit/source-observations.json"
        )))
        .unwrap();
        assert_eq!(
            original["source_pin"],
            "9e1e55cd2746037d059345881c53e6a6754bffbd"
        );
        let interline = original["interline"].as_f64().unwrap();
        let rows = original["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 157);
        let exact = original["tolerances"]["center_radius_distance"]
            .as_f64()
            .unwrap();
        let rounded = original["tolerances"]["angles_bezier_grade"]
            .as_f64()
            .unwrap();
        assert_eq!(exact, 0.0);
        assert_eq!(rounded, 1e-12);
        let mut native_count = 0;
        let mut control_count = 0;

        for (case, row) in rows.iter().enumerate() {
            assert_eq!(row["id"].as_u64(), Some(case as u64));
            if row["kind"].as_str().unwrap().starts_with("control_") {
                control_count += 1;
            } else {
                native_count += 1;
            }
            let points = row["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| Pixel {
                    x: p[0].as_i64().unwrap().try_into().unwrap(),
                    y: p[1].as_i64().unwrap().try_into().unwrap(),
                })
                .collect::<Vec<_>>();
            let actual = fit_short(&points, interline)
                .unwrap_or_else(|error| panic!("case {case}: unexpected input failure {error:?}"));
            assert_eq!(
                actual.math_accepted_before_binary_and_head_links,
                row["accepted"].as_bool().unwrap(),
                "case {case}: math acceptance before glyph/owner/Head linkage"
            );

            let expected_circle = &row["circle"];
            assert_eq!(
                actual.selected_after_impacts.is_none(),
                expected_circle.is_null(),
                "case {case}: nullable selected Circle"
            );
            if let Some(circle) = actual.selected_after_impacts.as_ref() {
                scalar(
                    circle.center.x,
                    &expected_circle["center"][0],
                    exact,
                    case,
                    "center.x",
                );
                scalar(
                    circle.center.y,
                    &expected_circle["center"][1],
                    exact,
                    case,
                    "center.y",
                );
                scalar(
                    circle.radius,
                    &expected_circle["radius"],
                    exact,
                    case,
                    "radius",
                );
                scalar(
                    circle.distance,
                    &expected_circle["distance"],
                    exact,
                    case,
                    "post-impacts distance",
                );
                assert_eq!(
                    circle.ccw as i64,
                    expected_circle["ccw"].as_i64().unwrap(),
                    "case {case}: ccw"
                );
                assert_eq!(
                    circle.above,
                    expected_circle["above"].as_bool().unwrap(),
                    "case {case}: above"
                );
                scalar(
                    circle.first_angle,
                    &expected_circle["first_angle"],
                    rounded,
                    case,
                    "first angle",
                );
                scalar(
                    circle.last_angle,
                    &expected_circle["last_angle"],
                    rounded,
                    case,
                    "last angle",
                );
            }

            assert_eq!(
                actual.impacts.is_none(),
                row["grade"].is_null(),
                "case {case}: nullable Impacts"
            );
            if let Some(impacts) = actual.impacts.as_ref() {
                scalar(impacts.grade, &row["grade"], rounded, case, "grade");
            }
            assert_eq!(
                actual.bezier.is_none(),
                row["bezier"].is_null(),
                "case {case}: nullable Bezier"
            );
            if let Some(bezier) = actual.bezier.as_ref() {
                let expected = row["bezier"].as_array().unwrap();
                assert_eq!(expected.len(), 4);
                for (index, point) in bezier.iter().enumerate() {
                    scalar(
                        point.x,
                        &expected[index][0],
                        rounded,
                        case,
                        &format!("Bezier[{index}].x"),
                    );
                    scalar(
                        point.y,
                        &expected[index][1],
                        rounded,
                        case,
                        &format!("Bezier[{index}].y"),
                    );
                }
            }
        }
        assert_eq!((native_count, control_count), (153, 4));
    }
}
