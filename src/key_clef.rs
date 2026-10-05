// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure KeyBuilder pitch/grade and clef-decision port, using upstream defaults.
//!
//! Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
//! Sources: KeyInter pitch maps; KeyBuilder.ShapeBuilder.computePitchedGrades,
//! computeKeyGrade, and the two scoring passes of checkWithClefs; GradeUtil;
//! ClefKeyRelation, KeyAltersRelation, and Support.
//!
//! Inputs are already classified alterations with measured pitches and intrinsic
//! grades. No glyph recognition, graph mutation, missing-alter filling, key
//! creation, or final header/key recognition is performed here.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Native resource bound. The upstream pitch tables contain exactly seven items.
pub const MAX_SLICES: usize = 7;
/// Adapter resource bound; order and distinct candidates are preserved within it.
pub const MAX_CLEFS: usize = 64;

// Both subclasses override getSourceCoeff() with 5; Support() sets grade to 1.
// getSourceRatio() is 1 + coefficient * grade, not the superclass coefficient 0.
const KEY_ALTERS_SOURCE_RATIO: f64 = 6.0;
const CLEF_KEY_SOURCE_RATIO: f64 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClefKind {
    Treble,
    Bass,
    Baritone,
    Tenor,
    Alto,
    MezzoSoprano,
    Soprano,
    Percussion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum KeyShape {
    Flat,
    Sharp,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClefCandidate {
    pub kind: ClefKind,
    /// Already-scaled upstream Inter.getGrade(), not a raw classifier score.
    pub intrinsic_grade: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyAlter {
    /// Already-scaled upstream Inter.getGrade(), not a raw classifier score.
    pub intrinsic_grade: f64,
    pub measured_pitch: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyClefError {
    TooManySlices {
        count: usize,
    },
    TooManyClefs {
        count: usize,
    },
    InvalidAlterGrade {
        slice_index: usize,
    },
    InvalidMeasuredPitch {
        slice_index: usize,
    },
    InvalidClefGrade {
        clef_index: usize,
    },
    InvalidPitchedGrade {
        slice_index: usize,
    },
    InvalidAlterCount {
        alter_count: usize,
        slice_count: usize,
    },
    PercussionHasNoPitches,
}

impl fmt::Display for KeyClefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManySlices { count } => {
                write!(
                    f,
                    "key/clef scoring supports at most {MAX_SLICES} slices, got {count}"
                )
            }
            Self::TooManyClefs { count } => {
                write!(
                    f,
                    "key/clef scoring supports at most {MAX_CLEFS} clefs, got {count}"
                )
            }
            Self::InvalidAlterGrade { slice_index } => {
                write!(
                    f,
                    "slice {slice_index}: intrinsic grade must be finite and in [0, 1]"
                )
            }
            Self::InvalidMeasuredPitch { slice_index } => {
                write!(f, "slice {slice_index}: measured pitch must be finite")
            }
            Self::InvalidClefGrade { clef_index } => {
                write!(
                    f,
                    "clef {clef_index}: intrinsic grade must be finite and in [0, 1]"
                )
            }
            Self::InvalidPitchedGrade { slice_index } => {
                write!(
                    f,
                    "slice {slice_index}: pitched grade must be finite and in [0, 1]"
                )
            }
            Self::InvalidAlterCount {
                alter_count,
                slice_count,
            } => {
                write!(f, "alter count {alter_count} must be in 1..={slice_count}")
            }
            Self::PercussionHasNoPitches => write!(f, "percussion has no key pitch table"),
        }
    }
}

impl std::error::Error for KeyClefError {}

/// Exact seven-element KeyInter pitch tables. Percussion has no entry.
pub fn pitches(kind: ClefKind, shape: KeyShape) -> Option<&'static [i32; MAX_SLICES]> {
    use ClefKind::*;
    use KeyShape::*;
    Some(match (kind, shape) {
        (Treble, Sharp) => &[-4, -1, -5, -2, 1, -3, 0],
        (Bass, Sharp) => &[-2, 1, -3, 0, 3, -1, 2],
        (Baritone, Sharp) => &[0, 3, -1, 2, -2, 1, -3],
        (Tenor, Sharp) => &[2, -2, 1, -3, 0, -4, -1],
        (Alto, Sharp) => &[-3, 0, -4, -1, 2, -2, 1],
        (MezzoSoprano, Sharp) => &[-1, 2, -2, 1, -3, 0, -4],
        (Soprano, Sharp) => &[1, 4, 0, 3, -1, 2, -2],
        (Treble, Flat) => &[0, -3, 1, -2, 2, -1, 3],
        (Bass, Flat) => &[2, -1, 3, 0, 4, 1, 5],
        (Baritone, Flat) => &[-3, 1, -2, 2, -1, 3, 0],
        (Tenor, Flat) => &[-1, -4, 0, -3, 1, -2, 2],
        (Alto, Flat) => &[1, -2, 2, -1, 3, 0, 4],
        (MezzoSoprano, Flat) => &[-4, 0, -3, 1, -2, 2, -1],
        (Soprano, Flat) => &[-2, 2, -1, 3, 0, 4, 1],
        (Percussion, _) => return None,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PitchedGrades {
    /// Fresh zero-filled output array, including partial writes before rejection.
    pub grades: Vec<f64>,
    /// Exact Java return value: zero on any excessive pitch error; otherwise the
    /// number of non-null alterations, even if their pitched grades become zero.
    pub alter_count: usize,
    /// None for zero slices, which returns before calculating a threshold.
    pub max_delta_pitch: Option<f64>,
    /// First rejected slice, if any. This distinguishes rejection from absence.
    pub rejected_slice: Option<usize>,
}

fn valid_grade(grade: f64) -> bool {
    grade.is_finite() && (0.0..=1.0).contains(&grade)
}

fn validate_slices(slices: &[Option<KeyAlter>]) -> Result<(), KeyClefError> {
    if slices.len() > MAX_SLICES {
        return Err(KeyClefError::TooManySlices {
            count: slices.len(),
        });
    }
    for (slice_index, alter) in slices.iter().enumerate() {
        if let Some(alter) = alter {
            if !valid_grade(alter.intrinsic_grade) {
                return Err(KeyClefError::InvalidAlterGrade { slice_index });
            }
            if !alter.measured_pitch.is_finite() {
                return Err(KeyClefError::InvalidMeasuredPitch { slice_index });
            }
        }
    }
    Ok(())
}

/// Port of computePitchedGrades for a fresh zero-filled output array.
///
/// The threshold uses total slices (including nulls), not alter_count. An error
/// equal to the threshold is accepted and yields a zero pitched grade. Inputs
/// outside the bounded finite native contract produce explicit errors rather
/// than emulating Java exceptions or allowing NaN to change comparison results.
pub fn compute_pitched_grades(
    kind: ClefKind,
    shape: KeyShape,
    slices: &[Option<KeyAlter>],
) -> Result<PitchedGrades, KeyClefError> {
    validate_slices(slices)?;
    pitched_grades_validated(kind, shape, slices)
}

fn pitched_grades_validated(
    kind: ClefKind,
    shape: KeyShape,
    slices: &[Option<KeyAlter>],
) -> Result<PitchedGrades, KeyClefError> {
    let n = slices.len();
    let mut result = PitchedGrades {
        grades: vec![0.0; n],
        alter_count: 0,
        max_delta_pitch: None,
        rejected_slice: None,
    };
    if n == 0 {
        return Ok(result);
    }
    let max_delta_pitch = if n >= 4 {
        2.0
    } else {
        0.5 + ((2.0 - 0.5) * (n - 1) as f64) / 3.0
    };
    result.max_delta_pitch = Some(max_delta_pitch);
    let clef_pitches = pitches(kind, shape).ok_or(KeyClefError::PercussionHasNoPitches)?;
    for (i, alter) in slices.iter().enumerate() {
        if let Some(alter) = alter {
            result.alter_count += 1;
            let d_pitch = (alter.measured_pitch - f64::from(clef_pitches[i])).abs();
            if d_pitch > max_delta_pitch {
                // Java returns zero immediately; earlier writes remain visible.
                result.alter_count = 0;
                result.rejected_slice = Some(i);
                return Ok(result);
            }
            result.grades[i] = alter.intrinsic_grade * (1.0 - d_pitch / max_delta_pitch);
        }
    }
    Ok(result)
}

fn contribution_of(partner: f64, ratio: f64) -> f64 {
    partner * (ratio - 1.0)
}

fn contextual(inter: f64, contribution: f64) -> f64 {
    ((1.0 + contribution) * inter) / (1.0 + contribution * inter)
}

/// Port of computeKeyGrade. `alter_count` is the number of present alterations,
/// not the array length or number of nonzero grades. Zero-count calls are invalid:
/// checkWithClefs never invokes this method when computePitchedGrades returns zero.
pub fn compute_key_grade(alter_count: usize, pitched_grades: &[f64]) -> Result<f64, KeyClefError> {
    if pitched_grades.len() > MAX_SLICES {
        return Err(KeyClefError::TooManySlices {
            count: pitched_grades.len(),
        });
    }
    if alter_count == 0 || alter_count > pitched_grades.len() {
        return Err(KeyClefError::InvalidAlterCount {
            alter_count,
            slice_count: pitched_grades.len(),
        });
    }
    for (slice_index, grade) in pitched_grades.iter().copied().enumerate() {
        if !valid_grade(grade) {
            return Err(KeyClefError::InvalidPitchedGrade { slice_index });
        }
    }
    Ok(key_grade_validated(alter_count, pitched_grades))
}

fn key_grade_validated(alter_count: usize, pitched_grades: &[f64]) -> f64 {
    let contribs: Vec<f64> = pitched_grades
        .iter()
        .map(|&grade| contribution_of(grade, KEY_ALTERS_SOURCE_RATIO))
        .collect();
    let mut key_grade = 0.0;
    for (i, &grade) in pitched_grades.iter().enumerate() {
        let mut contribution = 0.0;
        // Keep Java's nested accumulation order, rather than subtracting from a
        // shared total (which would change floating-point rounding).
        for (p, &contrib) in contribs.iter().enumerate() {
            if p != i {
                contribution += contrib;
            }
        }
        key_grade += contextual(grade, contribution);
    }
    key_grade / alter_count as f64
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ClefEvaluation {
    pub clef_index: usize,
    pub kind: ClefKind,
    pub intrinsic_grade: f64,
    /// None only for percussion, which checkWithClefs excludes from pitch scoring.
    pub pitched: Option<PitchedGrades>,
    /// None for percussion, all-null/empty slices, or any excessive pitch error.
    pub key_grade: Option<f64>,
    pub contextual_grade: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ClefDecision {
    /// Same order as the supplied clef candidates; no sorting or deduplication.
    pub evaluations: Vec<ClefEvaluation>,
    pub best_compatible_clef: Option<usize>,
    /// Preserves the Java initial value zero when no compatible clef is chosen.
    pub best_compatible_clef_contextual_grade: f64,
    /// None if there was no best compatible clef; Java skips its second pass.
    pub best_clef: Option<usize>,
    pub best_clef_grade: Option<f64>,
    /// Decision only: does the supported compatible clef also win overall?
    /// A true value does not mean that key creation/recognition has run.
    pub compatible_winner: bool,
}

/// Pure scoring/decision portion of KeyBuilder.ShapeBuilder.checkWithClefs.
///
/// Pass one compares the contextual grades of compatible non-percussion clefs.
/// Pass two gives contextual support only to that winner, comparing it against
/// the intrinsic grades of every other candidate, including percussion. Both
/// passes use strict `>` updates, preserving the first candidate on equal grades.
pub fn check_with_clefs(
    shape: KeyShape,
    clefs: &[ClefCandidate],
    slices: &[Option<KeyAlter>],
) -> Result<ClefDecision, KeyClefError> {
    validate_slices(slices)?;
    if clefs.len() > MAX_CLEFS {
        return Err(KeyClefError::TooManyClefs { count: clefs.len() });
    }
    for (clef_index, clef) in clefs.iter().enumerate() {
        if !valid_grade(clef.intrinsic_grade) {
            return Err(KeyClefError::InvalidClefGrade { clef_index });
        }
    }

    let mut result = ClefDecision {
        evaluations: Vec::with_capacity(clefs.len()),
        best_compatible_clef: None,
        best_compatible_clef_contextual_grade: 0.0,
        best_clef: None,
        best_clef_grade: None,
        compatible_winner: false,
    };
    for (clef_index, clef) in clefs.iter().enumerate() {
        let mut evaluation = ClefEvaluation {
            clef_index,
            kind: clef.kind,
            intrinsic_grade: clef.intrinsic_grade,
            pitched: None,
            key_grade: None,
            contextual_grade: None,
        };
        if clef.kind != ClefKind::Percussion {
            let pitched = pitched_grades_validated(clef.kind, shape, slices)?;
            if pitched.alter_count > 0 {
                let key_grade = key_grade_validated(pitched.alter_count, &pitched.grades);
                let key_contribution = contribution_of(key_grade, CLEF_KEY_SOURCE_RATIO);
                let clef_ctx = contextual(clef.intrinsic_grade, key_contribution);
                evaluation.key_grade = Some(key_grade);
                evaluation.contextual_grade = Some(clef_ctx);
                if clef_ctx > result.best_compatible_clef_contextual_grade {
                    result.best_compatible_clef_contextual_grade = clef_ctx;
                    result.best_compatible_clef = Some(clef_index);
                }
            }
            evaluation.pitched = Some(pitched);
        }
        result.evaluations.push(evaluation);
    }

    if let Some(best_compatible) = result.best_compatible_clef {
        let mut best_clef_grade = -1.0;
        for (clef_index, clef) in clefs.iter().enumerate() {
            let grade = if clef_index == best_compatible {
                result.best_compatible_clef_contextual_grade
            } else {
                clef.intrinsic_grade
            };
            if grade > best_clef_grade {
                best_clef_grade = grade;
                result.best_clef = Some(clef_index);
            }
        }
        result.best_clef_grade = Some(best_clef_grade);
        result.compatible_winner = result.best_clef == result.best_compatible_clef;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alter(grade: f64, pitch: f64) -> Option<KeyAlter> {
        Some(KeyAlter {
            intrinsic_grade: grade,
            measured_pitch: pitch,
        })
    }

    fn clef(kind: ClefKind, grade: f64) -> ClefCandidate {
        ClefCandidate {
            kind,
            intrinsic_grade: grade,
        }
    }

    #[test]
    fn all_fourteen_pitch_tables_match_key_inter() {
        let rows = [
            (
                ClefKind::Treble,
                [-4, -1, -5, -2, 1, -3, 0],
                [0, -3, 1, -2, 2, -1, 3],
            ),
            (
                ClefKind::Bass,
                [-2, 1, -3, 0, 3, -1, 2],
                [2, -1, 3, 0, 4, 1, 5],
            ),
            (
                ClefKind::Baritone,
                [0, 3, -1, 2, -2, 1, -3],
                [-3, 1, -2, 2, -1, 3, 0],
            ),
            (
                ClefKind::Tenor,
                [2, -2, 1, -3, 0, -4, -1],
                [-1, -4, 0, -3, 1, -2, 2],
            ),
            (
                ClefKind::Alto,
                [-3, 0, -4, -1, 2, -2, 1],
                [1, -2, 2, -1, 3, 0, 4],
            ),
            (
                ClefKind::MezzoSoprano,
                [-1, 2, -2, 1, -3, 0, -4],
                [-4, 0, -3, 1, -2, 2, -1],
            ),
            (
                ClefKind::Soprano,
                [1, 4, 0, 3, -1, 2, -2],
                [-2, 2, -1, 3, 0, 4, 1],
            ),
        ];
        for (kind, sharps, flats) in rows {
            for (shape, expected) in [(KeyShape::Sharp, sharps), (KeyShape::Flat, flats)] {
                assert_eq!(pitches(kind, shape), Some(&expected));
                let slices: Vec<_> = expected
                    .iter()
                    .map(|&p| alter(0.64, f64::from(p)))
                    .collect();
                let actual = compute_pitched_grades(kind, shape, &slices).unwrap();
                assert_eq!(actual.grades, vec![0.64; 7]);
                assert_eq!(actual.alter_count, 7);
                assert_eq!(actual.rejected_slice, None);
            }
        }
        assert_eq!(pitches(ClefKind::Percussion, KeyShape::Flat), None);
        assert_eq!(pitches(ClefKind::Percussion, KeyShape::Sharp), None);
    }

    #[test]
    fn thresholds_use_slice_count_and_accept_equality_with_zero_grade() {
        for n in 1..=7 {
            let threshold = [0.5, 1.0, 1.5, 2.0, 2.0, 2.0, 2.0][n - 1];
            let mut slices = vec![None; n];
            slices[0] = alter(0.8, -4.0 + threshold);
            let equal = compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &slices).unwrap();
            assert_eq!(equal.max_delta_pitch, Some(threshold));
            assert_eq!(equal.alter_count, 1);
            assert_eq!(equal.grades, vec![0.0; n]);
            assert_eq!(equal.rejected_slice, None);
            slices[0] = alter(0.8, -4.0 - threshold);
            assert_eq!(
                compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &slices)
                    .unwrap()
                    .alter_count,
                1
            );
            slices[0] = alter(0.8, -4.0 + threshold + 1.0e-10);
            let over = compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &slices).unwrap();
            assert_eq!(over.alter_count, 0);
            assert_eq!(over.rejected_slice, Some(0));
            slices[0] = alter(0.8, -4.0 + threshold / 2.0);
            assert_eq!(
                compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &slices)
                    .unwrap()
                    .grades[0],
                0.4
            );
        }
    }

    #[test]
    fn rejection_preserves_earlier_writes_and_leaves_later_entries_zero() {
        let actual = compute_pitched_grades(
            ClefKind::Treble,
            KeyShape::Sharp,
            &[alter(0.8, -4.0), None, alter(0.6, 100.0), alter(0.5, -2.0)],
        )
        .unwrap();
        assert_eq!(actual.grades, vec![0.8, 0.0, 0.0, 0.0]);
        assert_eq!(actual.alter_count, 0);
        assert_eq!(actual.rejected_slice, Some(2));
    }

    #[test]
    fn support_uses_ratio_six_and_actual_non_null_count() {
        assert_eq!(compute_key_grade(1, &[0.8]).unwrap(), 0.8);
        assert_eq!(compute_key_grade(2, &[0.8, 0.0, 0.4]).unwrap(), 11.0 / 13.0);
        let actual = compute_pitched_grades(
            ClefKind::Treble,
            KeyShape::Sharp,
            &[alter(0.8, -4.0), None, alter(0.4, -5.0)],
        )
        .unwrap();
        assert_eq!(actual.alter_count, 2);
        assert_eq!(
            compute_key_grade(actual.alter_count, &actual.grades).unwrap(),
            compute_key_grade(2, &[0.8, 0.0, 0.4]).unwrap()
        );
    }

    #[test]
    fn empty_and_missing_slices_do_not_select_a_clef() {
        for slices in [vec![], vec![None], vec![None; 7]] {
            let actual =
                check_with_clefs(KeyShape::Sharp, &[clef(ClefKind::Treble, 0.8)], &slices).unwrap();
            assert_eq!(actual.best_compatible_clef, None);
            assert_eq!(actual.best_clef, None);
            assert!(!actual.compatible_winner);
        }
        let empty = compute_pitched_grades(ClefKind::Percussion, KeyShape::Sharp, &[]).unwrap();
        assert_eq!(empty.max_delta_pitch, None);
        assert_eq!(empty.alter_count, 0);
        let actual = check_with_clefs(KeyShape::Sharp, &[], &[alter(0.8, -4.0)]).unwrap();
        assert_eq!(actual.best_clef, None);
    }

    #[test]
    fn zero_pitched_grade_can_still_be_compatible_but_zero_clef_cannot_win() {
        let slices = [alter(0.8, -3.5)];
        let actual =
            check_with_clefs(KeyShape::Sharp, &[clef(ClefKind::Treble, 0.2)], &slices).unwrap();
        assert_eq!(actual.evaluations[0].key_grade, Some(0.0));
        assert_eq!(actual.best_compatible_clef, Some(0));
        assert!(actual.compatible_winner);
        let zero = check_with_clefs(
            KeyShape::Sharp,
            &[clef(ClefKind::Treble, 0.0)],
            &[alter(0.8, -4.0)],
        )
        .unwrap();
        assert_eq!(zero.evaluations[0].contextual_grade, Some(0.0));
        assert_eq!(zero.best_compatible_clef, None);
        assert_eq!(zero.best_clef, None);
    }

    #[test]
    fn compatible_clef_gets_contextual_support_and_first_tie_wins() {
        let clefs = [clef(ClefKind::Treble, 0.2), clef(ClefKind::Treble, 0.2)];
        let actual = check_with_clefs(KeyShape::Sharp, &clefs, &[alter(0.8, -4.0)]).unwrap();
        assert_eq!(actual.evaluations[0].key_grade, Some(0.8));
        assert_eq!(actual.best_compatible_clef_contextual_grade, 1.0 / 1.8);
        assert_eq!(actual.best_compatible_clef, Some(0));
        assert_eq!(actual.best_clef, Some(0));
        assert!(actual.compatible_winner);
    }

    #[test]
    fn incompatible_and_percussion_clefs_still_compete_in_second_pass() {
        for kind in [ClefKind::Bass, ClefKind::Percussion] {
            let clefs = [clef(ClefKind::Treble, 0.2), clef(kind, 0.8)];
            let actual = check_with_clefs(KeyShape::Sharp, &clefs, &[alter(0.8, -4.0)]).unwrap();
            assert_eq!(actual.best_compatible_clef, Some(0));
            assert_eq!(actual.best_clef, Some(1));
            assert_eq!(actual.best_clef_grade, Some(0.8));
            assert!(!actual.compatible_winner);
            assert_eq!(actual.evaluations[1].key_grade, None);
            assert_eq!(
                actual.evaluations[1].pitched.is_none(),
                kind == ClefKind::Percussion
            );
        }
    }

    #[test]
    fn overall_equal_grade_preserves_input_order_even_for_incompatible_clef() {
        // Equality boundary produces key grade 0, so treble remains exactly 0.5.
        let slices = [alter(0.8, -3.5)];
        for compatible_first in [false, true] {
            let mut clefs = [clef(ClefKind::Percussion, 0.5), clef(ClefKind::Treble, 0.5)];
            if compatible_first {
                clefs.reverse();
            }
            let actual = check_with_clefs(KeyShape::Sharp, &clefs, &slices).unwrap();
            assert_eq!(actual.best_clef, Some(0));
            assert_eq!(
                actual.best_compatible_clef,
                Some(usize::from(!compatible_first))
            );
            assert_eq!(actual.compatible_winner, compatible_first);
        }
    }

    #[test]
    fn all_percussion_is_excluded_without_pitch_table_error() {
        let actual = check_with_clefs(
            KeyShape::Flat,
            &[clef(ClefKind::Percussion, 1.0)],
            &[alter(0.8, 0.0)],
        )
        .unwrap();
        assert_eq!(actual.evaluations[0].pitched, None);
        assert_eq!(actual.best_compatible_clef, None);
        assert_eq!(actual.best_clef, None);
        assert_eq!(
            compute_pitched_grades(ClefKind::Percussion, KeyShape::Flat, &[None]),
            Err(KeyClefError::PercussionHasNoPitches)
        );
    }

    #[test]
    fn invalid_inputs_and_resource_bounds_are_explicit() {
        for grade in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
            assert_eq!(
                compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &[alter(grade, -4.0)]),
                Err(KeyClefError::InvalidAlterGrade { slice_index: 0 })
            );
            assert_eq!(
                check_with_clefs(KeyShape::Sharp, &[clef(ClefKind::Percussion, grade)], &[]),
                Err(KeyClefError::InvalidClefGrade { clef_index: 0 })
            );
            assert_eq!(
                compute_key_grade(1, &[grade]),
                Err(KeyClefError::InvalidPitchedGrade { slice_index: 0 })
            );
        }
        for pitch in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &[alter(0.8, pitch)]),
                Err(KeyClefError::InvalidMeasuredPitch { slice_index: 0 })
            );
        }
        assert_eq!(
            compute_pitched_grades(ClefKind::Treble, KeyShape::Sharp, &[None; 8]),
            Err(KeyClefError::TooManySlices { count: 8 })
        );
        assert_eq!(
            compute_key_grade(1, &[0.0; 8]),
            Err(KeyClefError::TooManySlices { count: 8 })
        );
        assert_eq!(
            compute_key_grade(0, &[]),
            Err(KeyClefError::InvalidAlterCount {
                alter_count: 0,
                slice_count: 0
            })
        );
        assert_eq!(
            compute_key_grade(2, &[0.5]),
            Err(KeyClefError::InvalidAlterCount {
                alter_count: 2,
                slice_count: 1
            })
        );
        assert!(
            check_with_clefs(
                KeyShape::Sharp,
                &[clef(ClefKind::Treble, 0.5); MAX_CLEFS],
                &[]
            )
            .is_ok()
        );
        assert_eq!(
            check_with_clefs(
                KeyShape::Sharp,
                &[clef(ClefKind::Treble, 0.5); MAX_CLEFS + 1],
                &[]
            ),
            Err(KeyClefError::TooManyClefs {
                count: MAX_CLEFS + 1
            })
        );
    }
}
