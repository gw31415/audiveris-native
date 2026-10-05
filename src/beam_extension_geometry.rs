// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Port of BeamsBuilder.sideAreaOf/middleArea at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Pure search geometry only; no beam extension or graph mutation.
use audiveris_area::LinearArea;

use crate::beam_structure::intersection_at_x;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalSide {
    Left,
    Right,
}

/// Original side search parallelogram, including its one-pixel endpoint offset.
///
/// The native supported domain requires an increasing-x, finite median,
/// nonnegative height/margins/reach/overlap, and coordinates within the existing
/// LinearArea limit. Zero expanded height or coincident side endpoints returns
/// an empty area.
/// A reach below one pixel can reverse the endpoints: preserve the original
/// polygon rather than clamping it. Diagnostic Java attachments are omitted.
/// Direct original-method controls compare integer bounds and point membership
/// exactly; see `evidence/beam-side-area-checkpoint/original-parity.json`.
pub fn side_area_of(
    median: [f64; 4],
    height: f64,
    side: HorizontalSide,
    vertical_margin: f64,
    reach: f64,
    overlap: f64,
) -> Result<LinearArea, String> {
    const LIMIT: f64 = 16_000_000.0;
    if median.iter().any(|v| !v.is_finite() || v.abs() > LIMIT)
        || median[0] >= median[2]
        || [height, vertical_margin, reach, overlap]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=LIMIT).contains(v))
    {
        return Err("unsupported_beam_side_area_geometry".into());
    }
    let expanded_height = height + 2.0 * vertical_margin;
    let (near_x, far_x) = match side {
        HorizontalSide::Left => (median[0] - 1.0 + overlap, median[0] - reach),
        HorizontalSide::Right => (median[2] + 1.0 - overlap, median[2] + reach),
    };
    let near = intersection_at_x(median, near_x);
    let far = intersection_at_x(median, far_x);
    let (left, right) = match side {
        HorizontalSide::Left => (far, near),
        HorizontalSide::Right => (near, far),
    };
    // Preserve AreaUtil.horizontalParallelogramPath vertex and arithmetic order.
    let half_height = expanded_height / 2.0;
    LinearArea::polygon(&[
        [left[0], left[1] - half_height],
        [right[0], right[1] - half_height],
        [right[0], right[1] + half_height],
        [left[0], left[1] + half_height],
    ])
}

/// Original region between two beam ends, with horizontal-span-weighted height.
///
/// Both medians must be finite and increasing in x, with nonnegative finite
/// heights. Input values and generated vertices obey the existing LinearArea
/// coordinate limit. Touching ends or zero weighted height yield an empty area.
/// Overlapping beams retain the original reversed polygon. Equal starting x
/// selects the second beam's end and the first beam's start, as in the original;
/// do not sort the arguments or replace the strict comparison with <=.
pub fn middle_area(
    first_median: [f64; 4],
    first_height: f64,
    second_median: [f64; 4],
    second_height: f64,
) -> Result<LinearArea, String> {
    const LIMIT: f64 = 16_000_000.0;
    for (median, height) in [(first_median, first_height), (second_median, second_height)] {
        if median.iter().any(|v| !v.is_finite() || v.abs() > LIMIT)
            || median[0] >= median[2]
            || !height.is_finite()
            || !(0.0..=LIMIT).contains(&height)
        {
            return Err("unsupported_beam_middle_area_geometry".into());
        }
    }
    let first_width = first_median[2] - first_median[0];
    let second_width = second_median[2] - second_median[0];
    let height =
        (first_height * first_width + second_height * second_width) / (first_width + second_width);
    let (from, to) = if first_median[0] < second_median[0] {
        (
            [first_median[2], first_median[3]],
            [second_median[0], second_median[1]],
        )
    } else {
        (
            [second_median[2], second_median[3]],
            [first_median[0], first_median[1]],
        )
    };
    let half_height = height / 2.0;
    LinearArea::polygon(&[
        [from[0], from[1] - half_height],
        [to[0], to[1] - half_height],
        [to[0], to[1] + half_height],
        [from[0], from[1] + half_height],
    ])
}
