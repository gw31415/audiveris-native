// SPDX-License-Identifier: AGPL-3.0-or-later
//! StemBuilder section selection and raw StickFactory assembly at original 5.11.
//! The caller supplies the lookup polygon, theoretical line and optional stump.
//! This does not derive Head/Beam lookup geometry, filter chunks, register glyphs,
//! generate head relations or complete the page STEMS pipeline.
use crate::{
    beam_dispatch,
    vertical_sticks::{self, SeedSection},
};
use audiveris_area::LinearArea;
use audiveris_staff_removal::GlyphPixels;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Parameters {
    pub direction: i8,
    pub slope: f64,
    pub max_stem_thickness: i64,
    pub max_line_section_dx: f64,
    pub last_head_y: Option<f64>,
    pub theoretical_line: [f64; 4],
}
#[derive(Debug, Serialize)]
pub struct Selected {
    pub vertical_ids: Vec<u64>,
    pub horizontal_ids: Vec<u64>,
    /// Raw physical glyphs only; no registry identity or cache/SIG mutation.
    pub raw_glyphs: Vec<Value>,
}

fn validate_section(s: &SeedSection) -> Result<(), String> {
    if s.id == 0
        || s.runs.is_empty()
        || s.runs.len() > 8192
        || !(0..=4_000_000).contains(&s.first_pos)
        || s.first_pos + s.runs.len() as i64 > 4_000_000
        || s.runs.iter().any(|r| {
            !(0..=4_000_000).contains(&r[0])
                || !(1..=4_000_000).contains(&r[1])
                || r[0] + r[1] > 4_000_000
        })
    {
        return Err("invalid_stem_lookup_section".into());
    }
    if s.weight() > 8_000_000 {
        return Err("stem_lookup_pixel_budget".into());
    }
    Ok(())
}

/// BasicSection.getCentroid2D uses start+length/2 along a run, unlike the
/// pixel glyph centroid's start+(length-1)/2. Its intermediate products are Java
/// int before conversion to double. Refuse overflow outside the certified
/// range rather than silently substituting wider integer arithmetic.
pub fn section_centroid(s: &SeedSection) -> Result<[f64; 2], String> {
    validate_section(s)?;
    let (mut along, mut across) = (0f64, 0f64);
    for (i, r) in s.runs.iter().enumerate() {
        let a = r[1] * (2 * r[0] + r[1]);
        let b = r[1] * 2 * (s.first_pos + i as i64);
        if a > i32::MAX as i64 || b > i32::MAX as i64 {
            return Err("unsupported_stem_lookup_centroid_int_overflow".into());
        }
        along += a as f64;
        across += b as f64;
    }
    let divisor = (2 * s.weight()) as f64;
    if s.vertical {
        Ok([across / divisor, along / divisor])
    } else {
        Ok([along / divisor, across / divisor])
    }
}

fn distance(line: [f64; 4], point: [f64; 2]) -> f64 {
    // Preserve Line2D.ptLineDist's operation order, including clamp before sqrt.
    let (dx, dy) = (line[2] - line[0], line[3] - line[1]);
    let (px, py) = (point[0] - line[0], point[1] - line[1]);
    let dot = px * dx + py * dy;
    let projection = dot * dot / (dx * dx + dy * dy);
    (px * px + py * py - projection).max(0.).sqrt()
}

/// Stable Section.byFullPosition order and strict head-stop/inclusive line
/// distance selection. IDs are scoped per orientation. Inputs remain unchanged.
/// Only one linear polygon and observed zero skew are supported. The existing
/// StickFactory receives minCoreLength=1; positive runs make this equivalent to
/// original lookupParts' minCoreLength=0, checked on every oracle control.
/// Existing factory limits (including nonfinite/single-pixel line fits) apply.
pub fn collect(
    vertical: &[SeedSection],
    horizontal: &[SeedSection],
    polygon: &[[f64; 2]],
    stump: Option<&GlyphPixels>,
    p: &Parameters,
) -> Result<Selected, String> {
    if ![-1, 1].contains(&p.direction)
        || !p.slope.is_finite()
        || p.slope != 0.
        || !(1..=128).contains(&p.max_stem_thickness)
        || !p.max_line_section_dx.is_finite()
        || !(0.0..=1024.0).contains(&p.max_line_section_dx)
        || p.last_head_y
            .is_some_and(|y| !y.is_finite() || y.abs() > 4_000_000.)
        || p.theoretical_line
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || (p.theoretical_line[0] == p.theoretical_line[2]
            && p.theoretical_line[1] == p.theoretical_line[3])
    {
        return Err("unsupported_stem_lookup_parameters".into());
    }
    if vertical.len() + horizontal.len() > 32768 || polygon.len() > 128 {
        return Err("stem_lookup_input_budget".into());
    }
    let mut identities = BTreeSet::new();
    let (mut pixels, mut runs) = (0i64, 0usize);
    for (wanted, all) in [(true, vertical), (false, horizontal)] {
        for s in all {
            validate_section(s)?;
            if s.vertical != wanted || !identities.insert((wanted, s.id)) {
                return Err("invalid_stem_lookup_section_identity".into());
            }
            pixels += s.weight();
            runs += s.runs.len();
            if pixels > 8_000_000 || runs > 1_048_576 {
                return Err("stem_lookup_pixel_budget".into());
            }
        }
    }
    let area = if polygon.is_empty() {
        None
    } else {
        Some(LinearArea::polygon(polygon)?)
    };
    if let Some(glyph) = stump {
        beam_dispatch::centroid(glyph)?;
    }
    let stump_box = stump.map(|g| [g.left as i64, g.top as i64, g.width as i64, g.height as i64]);
    let (mut selected_v, mut selected_h) = (Vec::new(), Vec::new());
    for (wanted, all, selected) in [
        (true, vertical, &mut selected_v),
        (false, horizontal, &mut selected_h),
    ] {
        for section in all {
            let b = section.bounds();
            if !match &area {
                Some(a) => a.intersects_rect(b.map(|v| v as f64))?,
                None => false,
            } {
                continue;
            }
            if b[2] > if wanted { p.max_stem_thickness } else { 1 } {
                continue;
            }
            if wanted
                && stump_box.is_some_and(|a| {
                    (b[1] + b[3]).min(a[1] + a[3]) - b[1].max(a[1]) > 0 && b[3] < a[3]
                })
            {
                continue;
            }
            let center = section_centroid(section)?;
            if p.last_head_y
                .is_some_and(|y| match center[1].total_cmp(&y) {
                    std::cmp::Ordering::Less => p.direction < 0,
                    std::cmp::Ordering::Equal => true,
                    std::cmp::Ordering::Greater => p.direction > 0,
                })
            {
                continue;
            }
            if wanted && distance(p.theoretical_line, center) > p.max_line_section_dx {
                continue;
            }
            selected.push(section.clone());
        }
        selected.sort_by_key(|s| (s.first_pos, s.runs.iter().map(|r| r[0]).min().unwrap()));
    }
    // Preserve lookup order before StickFactory sorts its own horizontal copy.
    let sticks =
        vertical_sticks::retrieve_glyphs(&selected_v, &selected_h, p.max_stem_thickness, 1)?;
    Ok(Selected {
        vertical_ids: selected_v.iter().map(|s| s.id).collect(),
        horizontal_ids: selected_h.iter().map(|s| s.id).collect(),
        raw_glyphs: sticks,
    })
}
