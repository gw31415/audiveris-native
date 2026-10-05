// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Original unassigned sortedBeamSpots / extendToSpot boundary.
use crate::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Group, Registry, SystemSpots},
    beam_extend_to_beam::ExtensionScale,
    beam_extend_to_stem::{ThroughStem, side_through_stem},
    beam_extension_geometry::{HorizontalSide, side_area_of},
    beam_extension_prefix::SideDecision,
    beam_structure::{BeamItem, intersection_at_x},
};
#[derive(Clone, Debug, PartialEq)]
pub struct SpotPool {
    system_id: u64,
    ordered: Vec<usize>,
}
impl SpotPool {
    /// Original owning BEAM_SPOT list, full-ordinate ordering, then assigned removal.
    pub fn prepare(
        owner: &SystemSpots,
        registry: &Registry,
        assigned: &[usize],
    ) -> Result<Self, String> {
        if owner.glyph_ids.len() > crate::beam_dispatch::MAX_REGISTERED_GLYPHS
            || assigned.len() > crate::beam_dispatch::MAX_REGISTERED_GLYPHS
        {
            return Err("unsupported_beam_spot_pool_size".into());
        }
        let assigned = assigned
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let mut seen = std::collections::BTreeSet::new();
        let mut ordered = Vec::new();
        for &id in &owner.glyph_ids {
            let entry = registry.get(id)?;
            if seen.insert(id) && entry.groups.contains(&Group::BeamSpot) {
                ordered.push(id);
            }
        }
        ordered.sort_by_key(|id| {
            let g = &registry.get(*id).unwrap().glyph;
            (g.top, g.left, *id)
        });
        ordered.retain(|id| !assigned.contains(id));
        Ok(Self {
            system_id: owner.system_id,
            ordered,
        })
    }
    pub fn ordered_ids(&self) -> &[usize] {
        &self.ordered
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SpotLookup {
    pub reach: i32,
    pub ordered: Vec<usize>,
    pub selected: Option<usize>,
    pub point: Option<[f64; 2]>,
}
pub fn lookup_spot(
    item: &BeamItem,
    own_glyph: usize,
    pool: &SpotPool,
    registry: &Registry,
    side: HorizontalSide,
    max_extension: Option<i32>,
    interline: usize,
) -> Result<SpotLookup, String> {
    if interline == 0
        || interline > crate::sections::MAX_DIMENSION
        || max_extension.is_some_and(|n| n < 0)
    {
        return Err("unsupported_beam_spot_lookup_size".into());
    }
    let default = (0.5 * interline as f64).round_ties_even() as i32;
    let reach = max_extension.map_or(default, |n| n.min(default));
    let area = side_area_of(item.median, item.height, side, 0., reach as f64, 0.)?;
    let mut ordered = Vec::new();
    for &id in &pool.ordered {
        let g = &registry.get(id)?.glyph;
        if area.intersects_rect([g.left as f64, g.top as f64, g.width as f64, g.height as f64])? {
            ordered.push(id);
        }
    }
    ordered.sort_by_key(|id| registry.get(*id).unwrap().glyph.left);
    // The original removes the current beam's glyph after candidate sorting.
    ordered.retain(|id| *id != own_glyph);
    let selected = match side {
        HorizontalSide::Left => ordered.last(),
        HorizontalSide::Right => ordered.first(),
    }
    .copied();
    let point = selected.map(|id| {
        let g = &registry.get(id).unwrap().glyph;
        let x = match side {
            HorizontalSide::Left => g.left as f64,
            HorizontalSide::Right => g.left as f64 + g.width as f64 - 1.,
        };
        intersection_at_x(item.median, x)
    });
    Ok(SpotLookup {
        reach,
        ordered,
        selected,
        point,
    })
}
#[derive(Clone, Copy, Debug)]
pub struct SpotRequest {
    pub beam: usize,
    pub side: HorizontalSide,
    pub max_extension: Option<i32>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SpotExtension {
    pub lookup: SpotLookup,
    pub appended: Option<usize>,
}
impl SystemCandidates {
    pub fn extend_to_spot(
        &mut self,
        image: &GrayImage,
        registry: &mut Registry,
        request: SpotRequest,
        scale: ExtensionScale,
        pool: &SpotPool,
    ) -> Result<SpotExtension, String> {
        if self.system_id() != pool.system_id {
            return Err("inconsistent_beam_spot_owner".into());
        }
        let old = self
            .active(request.beam)
            .ok_or("invalid_beam_extension_target")?;
        let lookup = lookup_spot(
            &old.item,
            old.glyph_id(),
            pool,
            registry,
            request.side,
            request.max_extension,
            scale.interline,
        )?;
        let appended = if let Some(point) = lookup.point {
            self.extend_to_point(image, registry, request.beam, request.side, point, scale)?
        } else {
            None
        };
        Ok(SpotExtension { lookup, appended })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum ThroughSpot {
    Earlier(ThroughStem),
    SpotExtended {
        stem_path: ThroughStem,
        extension: SpotExtension,
    },
    NeedParallelOrNextSide {
        stem_path: ThroughStem,
        extension: SpotExtension,
    },
}
/// Original one-side merge/stem/spot schedule. Stops before any parallel branch.
pub fn side_through_spot(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    target: usize,
    side: HorizontalSide,
    scale: ExtensionScale,
    owner: &SystemSpots,
) -> Result<ThroughSpot, String> {
    if state.system_id() != owner.system_id {
        return Err("inconsistent_beam_spot_owner".into());
    }
    let pool = SpotPool::prepare(owner, registry, state.assigned_spot_ids())?;
    let stem_path = side_through_stem(state, image, registry, target, side, scale, owner)?;
    let ThroughStem::NeedSpot {
        prefix: SideDecision::NeedDownstream { max_extension, .. },
        ..
    } = stem_path
    else {
        return Ok(ThroughSpot::Earlier(stem_path));
    };
    let extension = state.extend_to_spot(
        image,
        registry,
        SpotRequest {
            beam: target,
            side,
            max_extension,
        },
        scale,
        &pool,
    )?;
    if extension.appended.is_some() {
        Ok(ThroughSpot::SpotExtended {
            stem_path,
            extension,
        })
    } else {
        Ok(ThroughSpot::NeedParallelOrNextSide {
            stem_path,
            extension,
        })
    }
}
