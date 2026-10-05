// SPDX-License-Identifier: AGPL-3.0-or-later
// MultipleRestsBuilder detection-to-registration planning boundary.
use crate::{
    multiple_rest_admission::{self, StaffPosition},
    serif_peaks::Peak,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SerifPlan {
    pub left: bool,
    pub peak: Peak,
    pub median: [f64; 4],
    pub width: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RestPlan {
    pub beam_id: usize,
    pub staff_id: u64,
    pub median: [f64; 4],
    pub height: f64,
    pub grade: f64,
    pub serifs: [SerifPlan; 2],
}
/// Admission geometry is supplied by the native staff association boundary.
/// Detection is performed LEFT then RIGHT, including when LEFT is empty, as in
/// getSerifPeaks. Plans have no glyph or graph mutation side effects.
pub struct BeamInput {
    pub id: usize,
    pub kind: &'static str,
    pub removed: bool,
    pub bounds: [i32; 4],
    pub median: [f64; 4],
    pub height: f64,
    pub grade: f64,
    pub staff: Option<StaffPosition>,
}
/// Collect all successful plans in candidate registration order before the
/// caller registers or removes any inter. Staff measurements remain supplied.
pub fn discover_system(
    state: &crate::beam_candidates::SystemCandidates,
    interline: usize,
    fore: usize,
    mut staff: impl FnMut(&crate::beam_candidates::CandidateInter) -> Option<StaffPosition>,
    mut detect: impl FnMut(u64, multiple_rest_admission::SerifQuery) -> Result<Vec<Peak>, String>,
) -> Result<Vec<RestPlan>, String> {
    if interline == 0 || interline > 10000 {
        return Err("invalid_multiple_rest_interline".into());
    }
    let mut plans = Vec::new();
    for candidate in state
        .inters()
        .iter()
        .filter(|b| !b.is_removed() && b.kind == "BeamInter")
    {
        if candidate.bounds[2] < (4. * interline as f64).round_ties_even() as i32 {
            continue;
        }
        let beam = BeamInput {
            id: candidate.id(),
            kind: candidate.kind,
            removed: false,
            bounds: candidate.bounds,
            median: candidate.item.median,
            height: candidate.item.height,
            grade: candidate.impacts.grade,
            staff: staff(candidate),
        };
        if let Some(plan) = discover(&beam, interline, fore, &mut detect)? {
            plans.push(plan);
        }
    }
    Ok(plans)
}
pub fn discover(
    beam: &BeamInput,
    interline: usize,
    fore: usize,
    mut detect: impl FnMut(u64, multiple_rest_admission::SerifQuery) -> Result<Vec<Peak>, String>,
) -> Result<Option<RestPlan>, String> {
    if beam.id == 0
        || fore == 0
        || fore > 20000
        || !beam
            .median
            .iter()
            .all(|v| v.is_finite() && v.abs() <= 10_000_000.)
        || beam.median[2] <= beam.median[0]
        || !beam.grade.is_finite()
        || !(0. ..=1.).contains(&beam.grade)
    {
        return Err("invalid_multiple_rest_plan_geometry".into());
    }
    let admission = multiple_rest_admission::assess(
        beam.kind,
        beam.removed,
        beam.bounds,
        beam.height,
        interline,
        beam.staff,
    )?;
    let Some(staff_id) = admission.staff_id else {
        return Ok(None);
    };
    let left = detect(staff_id, admission.queries[0])?;
    let right = detect(staff_id, admission.queries[1])?;
    let Some(left) = left.last().cloned() else {
        return Ok(None);
    };
    let Some(right) = right.first().cloned() else {
        return Ok(None);
    };
    for peak in [&left, &right] {
        if peak.stop < peak.start
            || peak.bottom < peak.top
            || [peak.start, peak.stop, peak.top, peak.bottom]
                .iter()
                .any(|&v| i64::from(v).abs() > 10_000_000)
            || !peak.grade.is_finite()
            || !peak.impacts.iter().all(|v| v.is_finite())
        {
            return Err("invalid_multiple_rest_peak".into());
        }
    }
    let make = |peak: Peak, left| {
        let width = f64::from(peak.stop - peak.start + 1);
        let x = f64::from(peak.start) + width / 2.;
        let median = [
            x,
            f64::from(peak.top) - fore as f64 / 2. + 0.5,
            x,
            f64::from(peak.bottom) + fore as f64 / 2. + 0.5,
        ];
        SerifPlan {
            left,
            peak,
            median,
            width,
        }
    };
    Ok(Some(RestPlan {
        beam_id: beam.id,
        staff_id,
        median: beam.median,
        height: beam.height,
        grade: beam.grade,
        serifs: [make(left, true), make(right, false)],
    }))
}
