// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Original extendToStem selection + verified extendToPoint boundary.
use crate::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Group, Registry, SystemSpots},
    beam_extend_to_beam::ExtensionScale,
    beam_extension_geometry::{HorizontalSide, side_area_of},
    beam_extension_prefix::{SideDecision, side_prefix},
    beam_structure::{BeamItem, glyph_stem_line},
};
#[derive(Clone, Debug, PartialEq)]
pub struct StemLookup {
    pub reach: i32,
    pub vertical_margin: i32,
    pub ordered: Vec<usize>,
    pub selected: Option<usize>,
    pub point: Option<[f64; 2]>,
}
/// Owner free-glyph order is retained; only VERTICAL_SEED members are eligible.
/// Stable abscissa sorting selects LEFT last / RIGHT first, without fallback.
pub fn lookup_stem(
    item: &BeamItem,
    owner: &SystemSpots,
    registry: &Registry,
    side: HorizontalSide,
    max_extension: Option<i32>,
    interline: usize,
) -> Result<StemLookup, String> {
    if interline == 0
        || interline > crate::sections::MAX_DIMENSION
        || owner.glyph_ids.len() > crate::beam_dispatch::MAX_REGISTERED_GLYPHS
        || max_extension.is_some_and(|n| n < 0)
    {
        return Err("unsupported_beam_stem_lookup_size".into());
    }
    let reach = max_extension.map_or(interline as i32, |n| n.min(interline as i32));
    let vertical_margin = (0.8 * interline as f64).round_ties_even() as i32;
    let area = side_area_of(
        item.median,
        item.height,
        side,
        vertical_margin as f64,
        reach as f64,
        0.,
    )?;
    let mut seen = std::collections::BTreeSet::new();
    let mut ordered = Vec::new();
    for &id in &owner.glyph_ids {
        let entry = registry.get(id)?;
        if !seen.insert(id) || !entry.groups.contains(&Group::VerticalSeed) {
            continue;
        }
        let g = &entry.glyph;
        if area.intersects_rect([g.left as f64, g.top as f64, g.width as f64, g.height as f64])? {
            ordered.push(id);
        }
    }
    ordered.sort_by_key(|id| registry.get(*id).unwrap().glyph.left);
    let selected = match side {
        HorizontalSide::Left => ordered.last(),
        HorizontalSide::Right => ordered.first(),
    }
    .copied();
    let point = if let Some(id) = selected {
        let mut b = glyph_stem_line(&registry.get(id)?.glyph)?
            .ok_or("unsupported_single_pixel_stem_geometry")?;
        // Glyph.getStartPoint/StopPoint(VERTICAL) orders the center-line ends by y.
        if b[1] > b[3] {
            b = [b[2], b[3], b[0], b[1]];
        }
        let a = item.median;
        let den = (a[0] - a[2]) * (b[1] - b[3]) - (a[1] - a[3]) * (b[0] - b[2]);
        let ac = a[0] * a[3] - a[1] * a[2];
        let bc = b[0] * b[3] - b[1] * b[2];
        let point = [
            (ac * (b[0] - b[2]) - (a[0] - a[2]) * bc) / den,
            (ac * (b[1] - b[3]) - (a[1] - a[3]) * bc) / den,
        ];
        Some(point)
    } else {
        None
    };
    Ok(StemLookup {
        reach,
        vertical_margin,
        ordered,
        selected,
        point,
    })
}
#[derive(Clone, Debug, PartialEq)]
pub struct StemExtension {
    pub lookup: StemLookup,
    pub appended: Option<usize>,
}
#[derive(Clone, Copy, Debug)]
pub struct StemRequest {
    pub beam: usize,
    pub side: HorizontalSide,
    pub max_extension: Option<i32>,
}
impl SystemCandidates {
    pub fn extend_to_stem(
        &mut self,
        image: &GrayImage,
        registry: &mut Registry,
        request: StemRequest,
        scale: ExtensionScale,
        owner: &SystemSpots,
    ) -> Result<StemExtension, String> {
        let StemRequest {
            beam: target,
            side,
            max_extension,
        } = request;
        if self.system_id() != owner.system_id {
            return Err("inconsistent_beam_stem_owner".into());
        }
        let old = self.active(target).ok_or("invalid_beam_extension_target")?;
        let lookup = lookup_stem(
            &old.item,
            owner,
            registry,
            side,
            max_extension,
            scale.interline,
        )?;
        // Original parallel-glyph controls yield nonfinite intersections and reject.
        // Keep the chosen seed/point in the report; never fall back or coerce to zero.
        let appended = if let Some(point) = lookup.point.filter(|p| p.iter().all(|n| n.is_finite()))
        {
            self.extend_to_point(image, registry, target, side, point, scale)?
        } else {
            None
        };
        Ok(StemExtension { lookup, appended })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum ThroughStem {
    Merged(SideDecision),
    StemExtended {
        prefix: SideDecision,
        extension: StemExtension,
    },
    NeedSpot {
        prefix: SideDecision,
        extension: StemExtension,
    },
}
/// One side's original merge/side-gap prefix followed by the real stem branch.
/// A rejected stem stops at spot processing; no spot/parallel false stub is used.
pub fn side_through_stem(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    target: usize,
    side: HorizontalSide,
    scale: ExtensionScale,
    owner: &SystemSpots,
) -> Result<ThroughStem, String> {
    if state.system_id() != owner.system_id {
        return Err("inconsistent_beam_stem_owner".into());
    }
    let prefix = side_prefix(state, image, registry, target, side, scale)?;
    let SideDecision::NeedDownstream { max_extension, .. } = prefix else {
        return Ok(ThroughStem::Merged(prefix));
    };
    let extension = state.extend_to_stem(
        image,
        registry,
        StemRequest {
            beam: target,
            side,
            max_extension,
        },
        scale,
        owner,
    )?;
    if extension.appended.is_some() {
        Ok(ThroughStem::StemExtended { prefix, extension })
    } else {
        Ok(ThroughStem::NeedSpot { prefix, extension })
    }
}
