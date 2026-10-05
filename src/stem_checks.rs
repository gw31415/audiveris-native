// Copyright © Audiveris 2026. Rust translation 2026.
// SPDX-License-Identifier: AGPL-3.0-or-later
// StemChecker / Check / CheckSuite at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// No glyph detection, section assembly, fixture lookup, or Java runtime dependency.
use crate::GrayImage;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Candidate {
    pub bounds: [i64; 4],
    pub start: [f64; 2],
    pub stop: [f64; 2],
    pub line: [f64; 4],
    pub mean_distance: f64,
}

/// Original Glyph NearLine adapter. These are raw fitted pixel coordinates;
/// StemInter's distinct half-pixel center-line conversion is not a check input.
pub fn candidate_from_glyph(
    glyph: &audiveris_staff_removal::GlyphPixels,
) -> Result<Candidate, String> {
    let line = crate::beam_structure::glyph_stem_line(glyph)?
        .ok_or("unsupported_single_pixel_stem_check_glyph")?;
    let (start, stop) = if line[1] <= line[3] {
        ([line[0], line[1]], [line[2], line[3]])
    } else {
        ([line[2], line[3]], [line[0], line[1]])
    };
    Ok(Candidate {
        bounds: [
            glyph.left as i64,
            glyph.top as i64,
            glyph.width as i64,
            glyph.height as i64,
        ],
        start,
        stop,
        line,
        mean_distance: crate::beam_structure::glyph_mean_distance(glyph)?,
    })
}

/// Grade a caller-selected physical glyph against the supplied NO_STAFF raster.
/// Registration, selection, cache and shared SIG mutation belong to later stages.
pub fn check_glyph(
    image: &GrayImage,
    glyph: &audiveris_staff_removal::GlyphPixels,
    parameters: &Parameters,
) -> Result<CheckResult, String> {
    check(image, &candidate_from_glyph(glyph)?, parameters)
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Parameters {
    pub interline: usize,
    pub max_stem: usize,
    pub global_slope: f64,
    pub profile: u8,
}
impl Parameters {
    pub fn new(
        interline: usize,
        max_stem: usize,
        global_slope: f64,
        profile: u8,
    ) -> Result<Self, String> {
        if !(1..=128).contains(&interline) || !(1..=128).contains(&max_stem) {
            return Err("unsupported_stem_check_scale".into());
        }
        if profile > 3 {
            return Err("unsupported_stem_check_profile".into());
        }
        if !global_slope.is_finite() || global_slope.abs() > 1.0 {
            return Err("unsupported_stem_check_global_slope".into());
        }
        Ok(Self {
            interline,
            max_stem,
            global_slope,
            profile,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub raw_value: f64,
    pub low: f64,
    pub high: f64,
    pub covariant: bool,
    pub weight: f64,
    pub impact: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CheckResult {
    pub context_black: usize,
    pub context_white: usize,
    pub context_gap: usize,
    pub checks: [Check; 7],
    pub grade: f64,
    pub min_threshold: f64,
    pub accepted: bool,
}

pub(crate) fn impact(value: f64, low: f64, high: f64, covariant: bool) -> f64 {
    if covariant {
        if value < low {
            0.0
        } else if value >= high {
            1.0
        } else {
            ((value - low) / (high - low)).clamp(0.0, 1.0)
        }
    } else if value > high {
        0.0
    } else if value <= low {
        1.0
    } else {
        ((high - value) / (high - low)).clamp(0.0, 1.0)
    }
}

// Original LineUtil.xAtY intersects the side line with (0,y)-(1000,y).
// Preserve its determinant operations, rather than algebraically simplifying:
// subsequent ceil/floor can observe the difference at integer boundaries.
pub(crate) fn x_at_y(start: [f64; 2], stop: [f64; 2], y: f64) -> f64 {
    let other_start = [0.0, y];
    let other_stop = [1000.0, y];
    let denominator = (start[0] - stop[0]) * (other_start[1] - other_stop[1])
        - (start[1] - stop[1]) * (other_start[0] - other_stop[0]);
    let first = start[0] * stop[1] - start[1] * stop[0];
    let second = other_start[0] * other_stop[1] - other_start[1] * other_stop[0];
    (first * (other_start[0] - other_stop[0]) - (start[0] - stop[0]) * second) / denominator
}

// ImageJ ByteProcessor.get(x,y) uses a linear index; x itself is not checked.
// Thus a side belt can address the preceding/following row. Preserve valid
// linear accesses, but reject actual Java array-out-of-range cases explicitly.
fn pixel(image: &GrayImage, x: i64, y: i64) -> Result<u8, String> {
    let i = y
        .checked_mul(image.width as i64)
        .and_then(|v| v.checked_add(x))
        .ok_or("invalid_stem_check_pixel_index")?;
    if i < 0 {
        return Err("unsupported_stem_check_pixel_access".into());
    }
    image
        .pixels
        .get(i as usize)
        .copied()
        .ok_or_else(|| "unsupported_stem_check_pixel_access".into())
}

pub fn check(
    image: &GrayImage,
    candidate: &Candidate,
    parameters: &Parameters,
) -> Result<CheckResult, String> {
    Parameters::new(
        parameters.interline,
        parameters.max_stem,
        parameters.global_slope,
        parameters.profile,
    )?;
    if image.width == 0
        || image.height == 0
        || image.width.checked_mul(image.height) != Some(image.pixels.len())
        || image.pixels.len() > crate::MAX_PIXELS
    {
        return Err("invalid_stem_check_image".into());
    }
    let [x, y, width, height] = candidate.bounds;
    if x < 0
        || y < 0
        || width < 1
        || height < 1
        || x.checked_add(width).is_none_or(|n| n > image.width as i64)
        || y.checked_add(height)
            .is_none_or(|n| n > image.height as i64)
        || candidate
            .start
            .iter()
            .chain(candidate.stop.iter())
            .chain(candidate.line.iter())
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.0)
        || !candidate.mean_distance.is_finite()
        || candidate.mean_distance < 0.0
        || candidate.start[1] >= candidate.stop[1]
        || candidate.line[3] < candidate.line[1]
    {
        return Err("invalid_stem_check_geometry".into());
    }
    let dy = candidate.stop[1] - candidate.start[1];
    let inverse_slope = (candidate.stop[0] - candidate.start[0]) / dy;
    // The original sanity failure leaves 0/0 BlackRatio (NaN). This bounded API
    // rejects that untested/nonfinite case rather than fabricating check values.
    if !inverse_slope.is_finite() || inverse_slope.abs() > 0.5 {
        return Err("unsupported_stem_check_slope_sanity".into());
    }
    let interline = parameters.interline as f64;
    let belt = (interline * 0.15).round_ties_even() as i64;
    let half_width = (parameters.max_stem - 1) as f64 / 2.0;
    let left_start = [candidate.start[0] - half_width, candidate.start[1]];
    let left_stop = [candidate.stop[0] - half_width, candidate.stop[1]];
    let right_start = [candidate.start[0] + half_width, candidate.start[1]];
    let right_stop = [candidate.stop[0] + half_width, candidate.stop[1]];
    let (mut largest_gap, mut last_black, mut last_white) = (0_i64, -1_i64, -1_i64);
    let (mut white, mut left, mut right, mut both, mut clean) =
        (0_usize, 0_usize, 0_usize, 0_usize, 0_usize);
    let (y_min, y_max) = (y, y + height - 1);
    let mut accesses = 0_usize;
    for row in y_min..=y_max {
        let left_limit = (x_at_y(left_start, left_stop, row as f64).ceil() as i64)
            .clamp(0, image.width as i64 - 1);
        let right_limit = (x_at_y(right_start, right_stop, row as f64).floor() as i64)
            .clamp(0, image.width as i64 - 1);
        accesses = accesses.saturating_add(
            (right_limit - left_limit + 1).max(0) as usize + 2 * (belt + 1) as usize,
        );
        if accesses > 16_000_000 {
            return Err("unsupported_stem_check_work_limit".into());
        }
        if row != y_min && row != y_max {
            let mut empty = true;
            for column in left_limit..=right_limit {
                if pixel(image, column, row)? == 0 {
                    empty = false;
                    break;
                }
            }
            if empty {
                white += 1;
                last_white = row;
                continue;
            }
        }
        if last_white != -1 && last_black != -1 {
            largest_gap = largest_gap.max(last_white - last_black);
            last_white = -1;
        }
        last_black = row;
        let mut on_left = true;
        for column in ((left_limit - belt)..=left_limit).rev() {
            if pixel(image, column, row)? != 0 {
                on_left = false;
                break;
            }
        }
        let mut on_right = true;
        for column in right_limit..=(right_limit + belt) {
            if pixel(image, column, row)? != 0 {
                on_right = false;
                break;
            }
        }
        match (on_left, on_right) {
            (true, true) => both += 1,
            (true, false) => left += 1,
            (false, true) => right += 1,
            (false, false) => clean += 1,
        }
    }
    let black = both + left + right + clean;
    // ConstantSet.getConstant falls back through lower profiles. Black length
    // has only a p1 override, which also applies to head retries p2/p3.
    let p1 = parameters.profile >= 1;
    let profile = parameters.profile as usize;
    let values = [
        (-inverse_slope - parameters.global_slope).abs(),
        candidate.mean_distance / interline,
        (candidate.line[3] - candidate.line[1]) / interline,
        clean as f64 / interline,
        black as f64 / interline,
        black as f64 / (black + white) as f64,
        largest_gap as f64 / interline,
    ];
    let specifications = [
        ("Slope", 0.0, 0.08, false, 1.0),
        ("Straight", 0.0, 0.2, false, 1.0),
        ("Length", 1.25, 3.5, true, 2.0),
        ("Clean", [0.5, 0.3, 0.2, 0.0][profile], 2.0, true, 2.0),
        (
            "Black",
            if p1 { 0.625 } else { 1.25 },
            if p1 { 1.25 } else { 2.5 },
            true,
            1.0,
        ),
        ("BlackRatio", [0.5, 0.3, 0.2, 0.1][profile], 1.0, true, 1.0),
        ("Gap", 0.0, [0.0, 0.3, 0.6, 2.0][profile], false, 5.0),
    ];
    let checks = std::array::from_fn(|i| {
        let (name, low, high, covariant, weight) = specifications[i];
        Check {
            name,
            raw_value: values[i],
            low,
            high,
            covariant,
            weight,
            impact: impact(values[i], low, high, covariant),
        }
    });
    let (mut product, mut weight_sum) = (1.0_f64, 0.0_f64);
    for c in &checks {
        weight_sum += c.weight;
        if c.impact == 0.0 {
            product = 0.0;
        } else {
            product *= c.impact.powf(c.weight);
        }
    }
    let grade = product.powf(1.0 / weight_sum) * 0.8;
    Ok(CheckResult {
        context_black: black,
        context_white: white,
        context_gap: largest_gap as usize,
        checks,
        grade,
        min_threshold: 0.2,
        accepted: grade >= 0.2,
    })
}
