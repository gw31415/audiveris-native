// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

fn difference(end: (u32, u32), expected: (u32, u32)) -> Delta {
    Fraction::new(end.0, end.1)
        .unwrap()
        .difference(Fraction::new(expected.0, expected.1).unwrap())
        .unwrap()
}

#[test]
fn source_pickup_requires_consistent_nonnull_termination_of_all_part_voices() {
    let short = difference((1, 8), (1, 2));
    let common = common_termination([Some(short), None, Some(short)]);
    let pickup = next_identity(true, None, Some(Fraction::new(1, 8).unwrap()), common).unwrap();
    assert_eq!(pickup.number, 0);
    assert!(pickup.pickup);
    let unequal = common_termination([Some(short), Some(difference((1, 4), (1, 2)))]);
    assert!(unequal.is_none());
    let ordinary = next_identity(true, None, Some(Fraction::new(1, 4).unwrap()), unequal).unwrap();
    assert_eq!(ordinary.number, 1);
    assert!(!ordinary.pickup);
}

#[test]
fn source_null_and_zero_terminations_do_not_create_pickup() {
    assert!(common_termination([None, None]).is_none());
    let zero = difference((1, 2), (1, 2));
    for common in [None, common_termination([Some(zero), None, Some(zero)])] {
        let id = next_identity(true, None, Some(Fraction::new(1, 8).unwrap()), common).unwrap();
        assert_eq!(id.number, 1);
        assert!(!id.pickup);
    }
}

#[test]
fn later_short_measure_keeps_numbering_across_system_boundary() {
    let short = difference((1, 8), (1, 2));
    let id = next_identity(
        false,
        Some(4),
        Some(Fraction::new(1, 8).unwrap()),
        Some(short),
    )
    .unwrap();
    assert_eq!(id.number, 5);
    assert!(!id.pickup);
}

#[test]
fn original_empty_branch_precedes_pickup_and_unknown_timing_stays_explicit() {
    let short = difference((1, 8), (1, 2));
    assert_eq!(
        next_identity(true, None, Some(Fraction::ZERO), Some(short)).unwrap_err(),
        "music_page_empty_merge_or_cautionary_pending"
    );
    assert_eq!(
        next_identity(true, None, None, Some(short)).unwrap_err(),
        "music_page_actual_duration_required"
    );
}

#[test]
fn exact_seven_over_sixty_four_termination_is_not_rounded_to_a_tick() {
    let short = difference((7, 64), (1, 2));
    assert_eq!(short.to_string(), "-25/64");
    let id = next_identity(true, None, Some(Fraction::new(7, 64).unwrap()), Some(short)).unwrap();
    assert_eq!(id.number, 0);
    assert!(id.pickup);
}

#[test]
fn retained_special_objects_and_relations_are_explicit_frontiers() {
    for kind in ["MultipleRestInter", "MeasureRepeatInter"] {
        assert_eq!(
            guard_special_symbols(&json!({"inters":[{"kind":kind}],"relations":[]})).unwrap_err(),
            "music_page_multiple_rest_or_measure_repeat_pending"
        );
    }
    assert_eq!(
        guard_special_symbols(&json!({"inters":[{"kind":"RepeatDotInter"}],"relations":[]}))
            .unwrap_err(),
        "music_page_repeat_halves_pending"
    );
    assert_eq!(
        guard_special_symbols(&json!({"inters":[],"relations":[{"kind":"RepeatDotBarRelation"}]}))
            .unwrap_err(),
        "music_page_repeat_halves_pending"
    );
}

/// Original persisted PAGE flags/timing, not a runtime callback trace.
pub(crate) fn assert_der_winter_persisted_page_state(music: &OwnedMusic, observation: &Value) {
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-page-pickup/persisted-state.json"
    ))
    .unwrap();
    assert_eq!(observation["measure_count"], source["page_measure_count"]);
    assert_eq!(observation["delta_measure_id"], source["delta_measure_id"]);
    let rows = observation["stacks"].as_array().unwrap();
    assert_eq!(rows.len(), source["stacks"].as_array().unwrap().len());
    for ((row, expected), held) in rows
        .iter()
        .zip(source["stacks"].as_array().unwrap())
        .zip(&music.measure_layout.stacks)
    {
        for field in [
            "system",
            "stack_index",
            "number",
            "implicit",
            "expected",
            "actual",
            "excess",
        ] {
            assert_eq!(row[field], expected[field], "PAGE field {field}");
        }
        let id = held.page_identity.unwrap();
        assert_eq!(json!(id.number), expected["number"]);
        assert_eq!(json!(id.pickup), expected["implicit"]);
        assert_eq!(
            json!(held.measures.iter().map(|g| g.part.id).collect::<Vec<_>>()),
            expected["parts"]
        );
        for field in ["expected", "actual", "excess"] {
            assert_eq!(row["before"][field], row[field]);
        }
    }
}
