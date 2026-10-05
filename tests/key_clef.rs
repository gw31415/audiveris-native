// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled original-JAR method observations, not end-to-end key recognition.
use audiveris_native::key_clef::{self, ClefCandidate, ClefKind, KeyAlter, KeyShape};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Fixture {
    pitch_arrays: Vec<PitchArray>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct PitchArray {
    kind: ClefKind,
    shape: KeyShape,
    pitches: [i32; 7],
}

#[derive(Deserialize)]
struct Case {
    name: String,
    shape: KeyShape,
    slices: Vec<Option<KeyAlter>>,
    clefs: Vec<ClefCandidate>,
    expected: Value,
    observation: Value,
}

fn field<'a>(object: &'a Value, name: &str) -> &'a Value {
    object
        .get(name)
        .unwrap_or_else(|| panic!("missing oracle field {name}: {object}"))
}

fn float_bits(actual: f64, expected: &Value, context: &str) {
    let expected = expected
        .as_f64()
        .unwrap_or_else(|| panic!("non-numeric {context}: {expected}"));
    assert!(
        actual.is_finite() && expected.is_finite(),
        "non-finite {context}"
    );
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "{context}: {actual:?} != {expected:?}"
    );
}

fn optional_float_bits(actual: Option<f64>, expected: &Value, context: &str) {
    match actual {
        Some(actual) => float_bits(actual, expected, context),
        None => assert!(expected.is_null(), "expected absent {context}: {expected}"),
    }
}

fn optional_index(expected: &Value) -> Option<usize> {
    if expected.is_null() {
        None
    } else {
        Some(
            usize::try_from(
                expected
                    .as_u64()
                    .expect("oracle index must be a nonnegative integer"),
            )
            .unwrap(),
        )
    }
}

#[test]
fn original_controlled_key_clef_methods_match_exactly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture: Fixture = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/key-clef/official-controlled.json")).unwrap(),
    )
    .unwrap();
    assert!(!fixture.cases.is_empty(), "oracle fixture has no cases");
    assert_eq!(
        fixture.pitch_arrays.len(),
        14,
        "all seven pitched clefs for both shapes"
    );
    let mut pitch_table_names = std::collections::HashSet::new();
    for row in fixture.pitch_arrays {
        assert!(
            pitch_table_names.insert(format!("{:?}/{:?}", row.kind, row.shape)),
            "duplicate pitch table"
        );
        assert_eq!(key_clef::pitches(row.kind, row.shape), Some(&row.pitches));
    }
    let mut names = std::collections::HashSet::new();
    for case in fixture.cases {
        assert!(
            names.insert(case.name.clone()),
            "duplicate case {}",
            case.name
        );
        println!("checking {}", case.name);
        let actual = key_clef::check_with_clefs(case.shape, &case.clefs, &case.slices).unwrap();
        let candidates = field(&case.expected, "candidates").as_array().unwrap();
        assert_eq!(
            actual.evaluations.len(),
            case.clefs.len(),
            "{}: native clef count",
            case.name
        );
        assert_eq!(
            candidates.len(),
            case.clefs.len(),
            "{}: oracle clef count",
            case.name
        );
        for (index, (got, want)) in actual.evaluations.iter().zip(candidates).enumerate() {
            let context = format!("{}: clef {index}", case.name);
            assert_eq!(got.clef_index, index, "{context}: identity/order");
            assert_eq!(got.kind, case.clefs[index].kind, "{context}: input kind");
            let expected_kind: ClefKind =
                serde_json::from_value(field(want, "kind").clone()).unwrap();
            assert_eq!(got.kind, expected_kind, "{context}: oracle kind");
            assert_eq!(
                got.intrinsic_grade.to_bits(),
                case.clefs[index].intrinsic_grade.to_bits(),
                "{context}: intrinsic grade"
            );
            match &got.pitched {
                Some(pitched) => {
                    assert_eq!(
                        field(want, "percussion_excluded").as_bool(),
                        Some(false),
                        "{context}: pitched clef not excluded"
                    );
                    let expected_grades = field(want, "pitched_grades").as_array().unwrap();
                    assert_eq!(
                        pitched.grades.len(),
                        case.slices.len(),
                        "{context}: native slice count"
                    );
                    assert_eq!(
                        expected_grades.len(),
                        case.slices.len(),
                        "{context}: oracle slice count"
                    );
                    assert_eq!(
                        Some(pitched.alter_count as u64),
                        field(want, "alter_count").as_u64(),
                        "{context}: alter count"
                    );
                    for (slice_index, (&got_grade, want_grade)) in
                        pitched.grades.iter().zip(expected_grades).enumerate()
                    {
                        float_bits(
                            got_grade,
                            want_grade,
                            &format!("{context}: slice {slice_index}"),
                        );
                    }
                    assert_eq!(
                        key_clef::compute_pitched_grades(got.kind, case.shape, &case.slices)
                            .unwrap(),
                        *pitched,
                        "{context}: standalone pitched method"
                    );
                    if pitched.alter_count > 0 {
                        float_bits(
                            key_clef::compute_key_grade(pitched.alter_count, &pitched.grades)
                                .unwrap(),
                            field(want, "key_grade"),
                            &format!("{context}: standalone key grade"),
                        );
                    }
                }
                None => {
                    assert_eq!(
                        got.kind,
                        ClefKind::Percussion,
                        "{context}: only percussion is excluded"
                    );
                    assert_eq!(
                        field(want, "percussion_excluded").as_bool(),
                        Some(true),
                        "{context}: percussion exclusion"
                    );
                    assert_eq!(
                        field(want, "alter_count").as_u64(),
                        Some(0),
                        "{context}: excluded percussion count"
                    );
                    let unscored = field(want, "pitched_grades").as_array().unwrap();
                    assert_eq!(
                        unscored.len(),
                        case.slices.len(),
                        "{context}: excluded percussion slice count"
                    );
                    for zero in unscored {
                        float_bits(
                            0.0,
                            zero,
                            &format!("{context}: excluded percussion unscored array"),
                        );
                    }
                }
            }
            optional_float_bits(
                got.key_grade,
                field(want, "key_grade"),
                &format!("{context}: key grade"),
            );
            optional_float_bits(
                got.contextual_grade,
                field(want, "contextual_grade"),
                &format!("{context}: contextual grade"),
            );
        }
        assert_eq!(
            actual.best_compatible_clef,
            optional_index(field(&case.expected, "best_compatible_index")),
            "{}: best compatible index",
            case.name
        );
        if actual.best_compatible_clef.is_some() {
            float_bits(
                actual.best_compatible_clef_contextual_grade,
                field(&case.expected, "best_compatible_contextual_grade"),
                &format!("{}: best compatible contextual grade", case.name),
            );
        } else {
            // The observation schema uses null for absence; Java's local starts at +0.
            assert!(field(&case.expected, "best_compatible_contextual_grade").is_null());
            assert_eq!(
                actual.best_compatible_clef_contextual_grade.to_bits(),
                0.0_f64.to_bits()
            );
        }
        assert_eq!(
            actual.best_clef,
            optional_index(field(&case.expected, "best_clef_index")),
            "{}: final winner index",
            case.name
        );
        assert_eq!(
            Some(actual.compatible_winner),
            field(&case.expected, "key_accepted").as_bool(),
            "{}: compatible winner decision",
            case.name
        );
        if field(&case.observation, "stopped_at_original_retain_all")
            .as_bool()
            .unwrap()
        {
            // This winner is observed inside the original checkWithClefs method.
            // The best-compatible fields and accepted decision above are labeled
            // source replays of original numeric observations in the fixture.
            assert_eq!(
                actual.best_clef,
                optional_index(field(&case.observation, "original_selected_clef_index")),
                "{}: original method selected identity",
                case.name
            );
        } else {
            assert_eq!(
                Some(actual.compatible_winner),
                field(&case.observation, "check_with_clefs_return").as_bool(),
                "{}: original method return",
                case.name
            );
        }
    }
}
