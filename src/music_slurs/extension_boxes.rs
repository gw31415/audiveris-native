// SPDX-License-Identifier: AGPL-3.0-or-later
//! Group all current native pair alternatives before obstacle filtering.
//! This is not a reconstruction of original Skeleton reachable-arc ownership.
use super::fit;
use audiveris_staff_removal::{GlyphPixels, Orientation};
use std::collections::BTreeMap;
use std::sync::Arc;

fn has_pixel(glyph: &GlyphPixels, point: fit::Pixel) -> bool {
    let x = i64::from(point.x) - i64::from(glyph.left);
    let y = i64::from(point.y) - i64::from(glyph.top);
    if x < 0 || y < 0 {
        return false;
    }
    glyph.runs.iter().any(|run| {
        x as u64 == run.position as u64
            && y as u64 >= run.start as u64
            && (y as u64) < run.start as u64 + run.length as u64
    })
}

fn endpoint(
    candidate: &fit::Candidate,
    glyph: &GlyphPixels,
    right: bool,
) -> Result<fit::Pixel, String> {
    if glyph.orientation != Orientation::Vertical {
        return Err("slur_extension_actual_vertical_component_required".into());
    }
    let mut endpoint: Option<fit::Pixel> = None;
    for &point in &candidate.points {
        if !has_pixel(glyph, point) {
            continue;
        }
        if endpoint.is_none_or(|previous| {
            if right {
                point.x > previous.x
            } else {
                point.x < previous.x
            }
        }) {
            endpoint = Some(point);
        }
    }
    let endpoint = endpoint.ok_or("slur_extension_actual_component_endpoint_required")?;
    if candidate
        .points
        .iter()
        .any(|&point| point.x == endpoint.x && point != endpoint && has_pixel(glyph, point))
    {
        return Err("slur_extension_component_endpoint_ambiguous".into());
    }
    Ok(endpoint)
}

pub(crate) fn gap_ends(
    candidate: &fit::Candidate,
    components: &[Arc<GlyphPixels>],
) -> Result<(usize, fit::Pixel, fit::Pixel), String> {
    let [left_index, right_index] = candidate.component_indices.as_slice() else {
        return Err("slur_extension_pair_candidate_required".into());
    };
    if left_index == right_index {
        return Err("slur_extension_distinct_actual_components_required".into());
    }
    let left = components
        .get(*left_index)
        .ok_or("slur_extension_left_owner_required")?;
    let right = components
        .get(*right_index)
        .ok_or("slur_extension_right_owner_required")?;
    let curve_end = endpoint(candidate, left, true)?;
    let arc_end = endpoint(candidate, right, false)?;
    if arc_end.x <= curve_end.x {
        return Err("slur_extension_actual_left_to_right_pair_required".into());
    }
    Ok((*left_index, curve_end, arc_end))
}

/// Pass every unfiltered native pair alternative, before math/gap/obstacle rejection.
/// Returned arrays follow Java Rectangle: [x,y,width,height], not corner coordinates.
pub(crate) fn grouped_extension_boxes(
    candidates: &[fit::Candidate],
    components: &[Arc<GlyphPixels>],
) -> Result<BTreeMap<usize, [f64; 4]>, String> {
    let mut groups: BTreeMap<usize, [f64; 4]> = BTreeMap::new();
    let mut curve_ends = BTreeMap::new();
    for candidate in candidates {
        if candidate.component_indices.len() == 1 {
            continue;
        }
        let (left_index, curve_end, arc_end) = gap_ends(candidate, components)?;
        if let Some(previous) = curve_ends.insert(left_index, curve_end)
            && previous != curve_end
        {
            return Err("slur_extension_common_actual_curve_end_required".into());
        }
        let bounds =
            groups
                .entry(left_index)
                .or_insert([curve_end.x as f64, curve_end.y as f64, 1.0, 1.0]);
        let [x, y, width, height] = *bounds;
        let xmin = x.min(arc_end.x as f64);
        let ymin = y.min(arc_end.y as f64);
        let xmax = (x + width).max(arc_end.x as f64);
        let ymax = (y + height).max(arc_end.y as f64);
        *bounds = [xmin, ymin, xmax - xmin, ymax - ymin];
    }
    Ok(groups)
}
