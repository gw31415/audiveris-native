// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Original BeamsBuilder.extendInParallel and geometric BeamGroupInter constraints.
use crate::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Geometry, Registry, SystemSpots},
    beam_extend_to_beam::ExtensionScale,
    beam_extend_to_spot::{ThroughSpot, side_through_spot},
    beam_extend_to_stem::ThroughStem,
    beam_extension_geometry::HorizontalSide,
    beam_extension_prefix::SideDecision,
    beam_structure::{BeamItem, intersection_at_x},
};
#[derive(Clone, Debug, PartialEq)]
pub struct NeighborGate {
    pub overlap: f64,
    pub vertical_distance: Option<f64>,
    pub slope_difference: Option<f64>,
    pub accepted: bool,
}
pub fn neighbor_gate(a: &BeamItem, b: &BeamItem, interline: usize) -> Result<NeighborGate, String> {
    if interline == 0 || interline > crate::sections::MAX_DIMENSION {
        return Err("unsupported_parallel_interline".into());
    }
    for item in [a, b] {
        crate::beam_extension_geometry::side_area_of(
            item.median,
            item.height,
            HorizontalSide::Left,
            0.,
            0.,
            0.,
        )?;
    }
    let a = a.median;
    let b = b.median;
    let left = a[0].max(b[0]);
    let right = a[2].min(b[2]);
    let overlap = right - left;
    let mut result = NeighborGate {
        overlap,
        vertical_distance: None,
        slope_difference: None,
        accepted: false,
    };
    if overlap < 0.7 * interline as f64 {
        return Ok(result);
    }
    let x = (left + right) / 2.;
    let distance = (intersection_at_x(b, x)[1] - intersection_at_x(a, x)[1]).abs();
    result.vertical_distance = Some(distance);
    if distance > 1.2 * interline as f64 {
        return Ok(result);
    }
    let difference = ((b[3] - b[1]) / (b[2] - b[0]) - (a[3] - a[1]) / (a[2] - a[0])).abs();
    result.slope_difference = Some(difference);
    result.accepted = difference <= 0.065;
    Ok(result)
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParallelAttempt {
    pub candidate: usize,
    pub outcome: &'static str,
    pub gap: Option<f64>,
    pub point: Option<[f64; 2]>,
    pub obstruction: Option<usize>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParallelExtension {
    pub ordered: Vec<usize>,
    pub attempts: Vec<ParallelAttempt>,
    pub attempted_point: Option<usize>,
    pub appended: Option<usize>,
}
impl SystemCandidates {
    pub fn extend_in_parallel(
        &mut self,
        image: &GrayImage,
        registry: &mut Registry,
        target: usize,
        side: HorizontalSide,
        scale: ExtensionScale,
        geometry: &Geometry,
    ) -> Result<ParallelExtension, String> {
        if scale.interline == 0 || scale.interline > crate::sections::MAX_DIMENSION {
            return Err("unsupported_parallel_interline".into());
        }
        geometry.validate_point_work(
            self.inters()
                .len()
                .checked_mul(self.inters().len().saturating_add(2))
                .ok_or("unsupported_parallel_work")?,
        )?;
        if !geometry.systems.iter().any(|s| s.id == self.system_id()) {
            return Err("missing_beam_parallel_system".into());
        }
        let old = self
            .active(target)
            .ok_or("invalid_beam_extension_target")?
            .clone();
        let m = old.item.median;
        let half = 1.2 * scale.interline as f64;
        let area = audiveris_area::LinearArea::polygon(&[
            [m[0], m[1] - half],
            [m[2], m[3] - half],
            [m[2], m[3] + half],
            [m[0], m[1] + half],
        ])?;
        let ordered = self
            .inters()
            .iter()
            .filter(|v| !v.is_removed() && v.id() != target)
            .filter_map(|v| match area.intersects_rect(v.bounds.map(|n| n as f64)) {
                Ok(true) => Some(Ok(v.id())),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut report = ParallelExtension {
            ordered,
            attempts: Vec::new(),
            attempted_point: None,
            appended: None,
        };
        let minimum = 2. * (0.2 * scale.interline as f64).round_ties_even();
        for &id in &report.ordered {
            let other = self.active(id).unwrap();
            let mut attempt = ParallelAttempt {
                candidate: id,
                outcome: "same_glyph",
                gap: None,
                point: None,
                obstruction: None,
            };
            if other.glyph_id() == old.glyph_id() {
                report.attempts.push(attempt);
                continue;
            }
            if !neighbor_gate(&old.item, &other.item, scale.interline)?.accepted {
                attempt.outcome = "not_neighbors";
                report.attempts.push(attempt);
                continue;
            }
            let n = other.item.median;
            let x = match side {
                HorizontalSide::Left => n[0],
                HorizontalSide::Right => n[2],
            };
            let gap = match side {
                HorizontalSide::Left => m[0] - x,
                HorizontalSide::Right => x - m[2],
            };
            attempt.gap = Some(gap);
            if gap < minimum {
                attempt.outcome = "insufficient_increment";
                report.attempts.push(attempt);
                continue;
            }
            let point = intersection_at_x(m, x);
            attempt.point = Some(point);
            if !geometry.is_beam_side_in_system(self.system_id(), point, old.item.height)? {
                attempt.outcome = "outside_system";
                report.attempts.push(attempt);
                continue;
            }
            let obstruction = self
                .lookup_side(target, side, Some(gap), scale.interline)?
                .selected;
            attempt.obstruction = obstruction;
            if obstruction.is_some() {
                attempt.outcome = "side_obstruction";
                report.attempts.push(attempt);
                continue;
            }
            attempt.outcome = "point_extension";
            report.attempts.push(attempt);
            report.attempted_point = Some(id);
            report.appended = self.extend_to_point(image, registry, target, side, point, scale)?;
            // The original returns this result immediately, including rejection: no fallback.
            return Ok(report);
        }
        Ok(report)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum ThroughParallel {
    Earlier(ThroughSpot),
    SkippedForSideNeighbor(ThroughSpot),
    FinishedSide {
        spot_path: ThroughSpot,
        extension: ParallelExtension,
    },
}
#[derive(Clone, Copy)]
pub struct ParallelOwner<'a> {
    pub free_glyphs: &'a SystemSpots,
    pub geometry: &'a Geometry,
}
/// One side only. Initial snapshot/next-side loop scheduling and CLI remain separate.
pub fn side_through_parallel(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    target: usize,
    side: HorizontalSide,
    scale: ExtensionScale,
    owner: ParallelOwner<'_>,
) -> Result<ThroughParallel, String> {
    let spot_path = side_through_spot(
        state,
        image,
        registry,
        target,
        side,
        scale,
        owner.free_glyphs,
    )?;
    let ThroughSpot::NeedParallelOrNextSide {
        stem_path:
            ThroughStem::NeedSpot {
                prefix:
                    SideDecision::NeedDownstream {
                        parallel_eligible_after_stem_and_spot,
                        ..
                    },
                ..
            },
        ..
    } = spot_path
    else {
        return Ok(ThroughParallel::Earlier(spot_path));
    };
    if !parallel_eligible_after_stem_and_spot {
        return Ok(ThroughParallel::SkippedForSideNeighbor(spot_path));
    }
    let extension =
        state.extend_in_parallel(image, registry, target, side, scale, owner.geometry)?;
    Ok(ThroughParallel::FinishedSide {
        spot_path,
        extension,
    })
}
