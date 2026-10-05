// SPDX-License-Identifier: AGPL-3.0-or-later
// Pinned MultipleRestsBuilder.process admission and getSerifPeaks queries.
use crate::beam_candidates::CandidateInter;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaffPosition {
    pub id: u64,
    pub tablature: bool,
    pub endpoint_pitches: [f64; 2],
    pub mid_line_thickness: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SerifQuery {
    pub x: i32,
    pub left: bool,
    pub added_chunk: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Admission {
    pub rejection: Option<&'static str>,
    pub staff_id: Option<u64>,
    pub queries: Vec<SerifQuery>,
}
/// Staff association and endpoint pitch measurement are supplied by the caller.
/// This boundary never mutates candidates, groups or the glyph registry.
pub fn assess_candidate(
    beam: &CandidateInter,
    interline: usize,
    staff: Option<StaffPosition>,
) -> Result<Admission, String> {
    assess(
        beam.kind,
        beam.is_removed(),
        beam.bounds,
        beam.item.height,
        interline,
        staff,
    )
}
pub fn assess(
    kind: &str,
    removed: bool,
    bounds: [i32; 4],
    height: f64,
    interline: usize,
    staff: Option<StaffPosition>,
) -> Result<Admission, String> {
    if interline == 0
        || interline > 10000
        || bounds[2] <= 0
        || bounds[3] <= 0
        || bounds.iter().any(|&n| i64::from(n).abs() > 10_000_000)
        || !height.is_finite()
        || height <= 0.
        || height > 20000.
    {
        return Err("invalid_multiple_rest_admission_geometry".into());
    }
    if let Some(s) = staff
        && (s.id == 0
            || !s.endpoint_pitches.iter().all(|p| p.is_finite())
            || !s.mid_line_thickness.is_finite()
            || s.mid_line_thickness <= 0.
            || s.mid_line_thickness > 20000.)
    {
        return Err("invalid_multiple_rest_staff_measurement".into());
    }
    let reject = |reason| Admission {
        rejection: Some(reason),
        staff_id: None,
        queries: Vec::new(),
    };
    if removed || kind != "BeamInter" {
        return Ok(reject("not_active_normal_beam"));
    }
    let min_length = (4. * interline as f64).round_ties_even() as i32;
    if bounds[2] < min_length {
        return Ok(reject("too_short"));
    }
    let Some(staff) = staff else {
        return Ok(reject("no_staff"));
    };
    if staff.tablature {
        return Ok(reject("tablature"));
    }
    if staff.endpoint_pitches[0].abs() > 0.2 {
        return Ok(reject("left_pitch"));
    }
    if staff.endpoint_pitches[1].abs() > 0.2 {
        return Ok(reject("right_pitch"));
    }
    let added_chunk = (height - staff.mid_line_thickness).round_ties_even() as i32;
    Ok(Admission {
        rejection: None,
        staff_id: Some(staff.id),
        queries: vec![
            SerifQuery {
                x: bounds[0],
                left: true,
                added_chunk,
            },
            SerifQuery {
                x: bounds[0] + bounds[2] - 1,
                left: false,
                added_chunk,
            },
        ],
    })
}
/// Original getSerifPeaks retains the last LEFT peak and first RIGHT peak in
/// the projector's encounter order. Lists are not sorted or deduplicated here.
pub fn select_serif_peaks<'a, T>(left: &'a [T], right: &'a [T]) -> [Option<&'a T>; 2] {
    [left.last(), right.first()]
}
