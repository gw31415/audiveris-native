// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected candidate-part alignment filter only. Candidate discovery and the
//! complete StemBuilder parts/section/linker expansion remain separate.
use crate::{beam_dispatch, stem_glyph};
use audiveris_staff_removal::{GlyphPixels, Orientation};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy)]
pub struct Part<'a> {
    pub id: usize,
    pub glyph: &'a GlyphPixels,
}
#[derive(Clone, Debug)]
pub struct Parameters {
    pub direction: i8,
    pub max_dx: f64,
    pub max_dy: f64,
    pub slope: f64,
}
impl Parameters {
    pub fn new(direction: i8, max_dx: f64, max_dy: f64, slope: f64) -> Result<Self, String> {
        if ![-1, 1].contains(&direction)
            || !max_dx.is_finite()
            || !max_dy.is_finite()
            || !(0.0..=1024.0).contains(&max_dx)
            || !(0.0..=1024.0).contains(&max_dy)
        {
            return Err("invalid_stem_alignment_parameters".into());
        }
        if !slope.is_finite() || slope != 0.0 {
            return Err("unsupported_stem_alignment_skew".into());
        }
        Ok(Self {
            direction,
            max_dx,
            max_dy,
            slope,
        })
    }
}
#[derive(Debug, serde::Serialize)]
pub struct Filtered {
    pub retained_order: Vec<usize>,
    pub removed_order: Vec<usize>,
}
fn centroid(g: &GlyphPixels) -> Result<[f64; 2], String> {
    beam_dispatch::centroid(g)?;
    let (mut weight, mut sx, mut sy) = (0usize, 0f64, 0f64);
    for r in &g.runs {
        weight = weight
            .checked_add(r.length)
            .filter(|v| *v <= stem_glyph::MAX_PART_PIXELS)
            .ok_or("stem_alignment_pixel_budget")?;
        let n = r.length as f64;
        let along = n * (2. * r.start as f64 + n - 1.) / 2.;
        let across = n * r.position as f64;
        match g.orientation {
            Orientation::Vertical => {
                sx += across;
                sy += along
            }
            Orientation::Horizontal => {
                sx += along;
                sy += across
            }
        }
    }
    Ok([
        g.left as f64 + sx / weight as f64,
        g.top as f64 + sy / weight as f64,
    ])
}
/// Original stable ordinate/reverse-bottom order, optional forced first stump,
/// short-part removal and caller collection removeAll. IDs represent caller
/// object identity, preserving duplicates and separate equal-physical objects.
/// Returns a plan without mutating parts, registry, cache or SIG. Observed
/// skew0 only; nonzero/NaN skew refuses until independently compared.
pub fn filter(
    parts: &[Part<'_>],
    stump: Option<Part<'_>>,
    parameters: &Parameters,
) -> Result<Filtered, String> {
    let parameters = Parameters::new(
        parameters.direction,
        parameters.max_dx,
        parameters.max_dy,
        parameters.slope,
    )?;
    if parts.len() > stem_glyph::MAX_PARTS {
        return Err("stem_alignment_part_budget".into());
    }
    let mut total_pixels = 0usize;
    let mut catalog = BTreeMap::<usize, (&GlyphPixels, [f64; 2])>::new();
    for p in parts.iter().copied().chain(stump) {
        if p.id == 0 {
            return Err("invalid_stem_alignment_part_id".into());
        }
        if let Some((g, _)) = catalog.get(&p.id) {
            if beam_dispatch::glyph_json(g) != beam_dispatch::glyph_json(p.glyph) {
                return Err("conflicting_stem_part_identity".into());
            }
        } else {
            let center = centroid(p.glyph)?;
            total_pixels = total_pixels
                .checked_add(p.glyph.runs.iter().map(|r| r.length).sum::<usize>())
                .filter(|v| *v <= stem_glyph::MAX_PART_PIXELS)
                .ok_or("stem_alignment_pixel_budget")?;
            catalog.insert(p.id, (p.glyph, center));
        }
    }
    let mut working = parts.iter().map(|p| p.id).collect::<Vec<_>>();
    working.sort_by_key(|id| {
        let g = catalog[id].0;
        if parameters.direction > 0 {
            g.top as i64
        } else {
            -(g.top as i64 + g.height as i64 - 1)
        }
    });
    if let Some(stump) = stump {
        if let Some(i) = working.iter().position(|id| *id == stump.id) {
            working.remove(i);
        }
        working.insert(0, stump.id);
    }
    let mut removed = Vec::new();
    let mut i = 0;
    while i + 1 < working.len() {
        let a = working[i];
        let b = working[i + 1];
        let (ga, ca) = catalog[&a];
        let (gb, cb) = catalog[&b];
        if (cb[1] - ca[1]).abs() <= parameters.max_dy && (cb[0] - ca[0]).abs() > parameters.max_dx {
            let drop = if ga.height < gb.height { a } else { b };
            removed.push(drop);
            let at = working.iter().position(|id| *id == drop).unwrap();
            working.remove(at);
        } else {
            i += 1;
        }
    }
    let dropped = removed.iter().copied().collect::<BTreeSet<_>>();
    Ok(Filtered {
        retained_order: parts
            .iter()
            .filter(|p| !dropped.contains(&p.id))
            .map(|p| p.id)
            .collect(),
        removed_order: removed,
    })
}
