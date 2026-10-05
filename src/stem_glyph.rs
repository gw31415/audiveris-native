// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit selected glyph parts only: original GlyphFactory.buildGlyph and
//! StemInter constructor geometry. No automatic STEMS section/seed selection.
use crate::{
    beam_dispatch::{self, Registry},
    beam_structure,
};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use std::collections::BTreeSet;
pub const MAX_PARTS: usize = 4096;
pub const MAX_PART_PIXELS: usize = 1_000_000;

/// Original vertical run-table union, even for a single horizontal part.
/// Includes overlap, gaps and duplicates; registry identity is content-based.
pub fn assemble(parts: &[GlyphPixels]) -> Result<GlyphPixels, String> {
    if parts.is_empty() || parts.len() > MAX_PARTS {
        return Err("invalid_stem_glyph_parts".into());
    }
    let mut input_pixels = 0usize;
    for g in parts {
        beam_dispatch::centroid(g)?;
        input_pixels = input_pixels
            .checked_add(g.runs.iter().map(|r| r.length).sum::<usize>())
            .filter(|n| *n <= MAX_PART_PIXELS)
            .ok_or("stem_glyph_pixel_budget")?;
    }
    let left = parts.iter().map(|g| g.left as i64).min().unwrap();
    let top = parts.iter().map(|g| g.top as i64).min().unwrap();
    let right = parts
        .iter()
        .map(|g| g.left as i64 + g.width as i64)
        .max()
        .unwrap();
    let bottom = parts
        .iter()
        .map(|g| g.top as i64 + g.height as i64)
        .max()
        .unwrap();
    let width = usize::try_from(right - left).map_err(|_| "invalid_stem_glyph_bounds")?;
    let height = usize::try_from(bottom - top).map_err(|_| "invalid_stem_glyph_bounds")?;
    if width
        .checked_mul(height)
        .is_none_or(|n| n > crate::MAX_PIXELS)
    {
        return Err("stem_glyph_bounds_budget".into());
    }
    let mut pixels = BTreeSet::new();
    for g in parts {
        for r in &g.runs {
            for delta in 0..r.length {
                let (x, y) = if g.orientation == Orientation::Vertical {
                    (r.position, r.start + delta)
                } else {
                    (r.start + delta, r.position)
                };
                pixels.insert((
                    (g.left as i64 + x as i64 - left) as usize,
                    (g.top as i64 + y as i64 - top) as usize,
                ));
            }
        }
    }
    let mut runs: Vec<Run> = Vec::new();
    for (x, y) in pixels {
        if let Some(last) = runs.last_mut()
            && last.position == x
            && last.start + last.length == y
        {
            last.length += 1;
            continue;
        }
        runs.push(Run {
            position: x,
            start: y,
            length: 1,
        });
    }
    let result = GlyphPixels {
        left: left as i32,
        top: top as i32,
        width,
        height,
        orientation: Orientation::Vertical,
        runs,
    };
    beam_dispatch::centroid(&result)?;
    Ok(result)
}
/// Validate/assemble before the single registry mutation. Existing content IDs
/// and VERTICAL_SEED groups are preserved by the shared physical glyph index.
pub fn acquire(registry: &mut Registry, parts: &[GlyphPixels]) -> Result<usize, String> {
    let glyph = assemble(parts)?;
    registry.register(&glyph)
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Geometry {
    pub median: [f64; 4],
    pub width: f64,
    pub bounds: [i32; 4],
}
/// Original vertical ribbon has horizontal half-width offsets. A horizontal
/// center line creates an empty area (original bounds [0,0,0,0]); it cannot be
/// registered as a supported vertical stem. Single-pixel fits are unsupported.
pub fn geometry(glyph: &GlyphPixels) -> Result<Geometry, String> {
    beam_dispatch::centroid(glyph)?;
    let median = beam_structure::glyph_geometry_signed(glyph)?
        .center_line
        .ok_or("unsupported_single_pixel_stem_geometry")?;
    let width = glyph.runs.iter().map(|r| r.length).sum::<usize>() as f64 / glyph.height as f64;
    let bounds = if median[1] == median[3] {
        [0, 0, 0, 0]
    } else {
        let left = (median[0].min(median[2]) - width / 2.).floor();
        let right = (median[0].max(median[2]) + width / 2.).ceil();
        let top = median[1].min(median[3]).floor();
        let bottom = median[1].max(median[3]).ceil();
        [
            left as i32,
            top as i32,
            (right - left) as i32,
            (bottom - top) as i32,
        ]
    };
    Ok(Geometry {
        median,
        width,
        bounds,
    })
}
/// Caller supplies an already assessed intrinsic stem grade. This is the
/// physical StemInter boundary, not the unported StemBuilder grade/selection.
pub fn record(local: usize, glyph: &GlyphPixels, grade: f64) -> Result<serde_json::Value, String> {
    if local == 0 || !grade.is_finite() || !(0.0..=1.0).contains(&grade) {
        return Err("invalid_stem_inter_record".into());
    }
    let geometry = geometry(glyph)?;
    if geometry.median[1] >= geometry.median[3]
        || geometry.bounds[2] <= 0
        || geometry.bounds[3] <= 0
    {
        return Err("unsupported_nonvertical_stem_geometry".into());
    }
    Ok(
        serde_json::json!({"id":local,"kind":"StemInter","shape":"STEM","bounds":geometry.bounds,"median":geometry.median,"width":geometry.width,"grade":grade,"glyph":beam_dispatch::glyph_json(glyph),"abnormal":true,"removed":false,"manual":false}),
    )
}
