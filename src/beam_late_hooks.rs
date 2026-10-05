// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// BeamsBuilder.buildHooks/browseHooks/checkHookGlyph at pinned 9e1e55cd.
use crate::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::{Registry, SystemSpots},
    beam_extend_to_beam::ExtensionScale,
    beam_extend_to_spot::SpotPool,
    beam_impacts::{Evaluator, ItemAreas},
    beam_merge::MergeCandidate,
    beam_structure::{BeamItem, ItemParameters, VerticalSide, glyph_geometry},
};
use audiveris_staff_removal::Orientation;

#[derive(Clone, Debug, PartialEq)]
pub struct HookAttempt {
    pub beam: usize,
    pub side: VerticalSide,
    pub spot: usize,
    pub rejection: Option<&'static str>,
    pub appended: Option<usize>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HookPass {
    pub initial_ids: Vec<usize>,
    pub pool_ids: Vec<usize>,
    pub skipped_small: Vec<usize>,
    pub attempts: Vec<HookAttempt>,
}

fn area(item: &BeamItem) -> Result<audiveris_area::LinearArea, String> {
    let m = item.median;
    let h = item.height / 2.;
    audiveris_area::LinearArea::polygon(&[
        [m[0], m[1] - h],
        [m[2], m[3] - h],
        [m[2], m[3] + h],
        [m[0], m[1] + h],
    ])
}
fn centroid(g: &audiveris_staff_removal::GlyphPixels) -> [f64; 2] {
    let (mut weight, mut sx, mut sy) = (0i64, 0i64, 0i64);
    for r in &g.runs {
        let n = r.length as i64;
        let along = n * (2 * r.start as i64 + n - 1) / 2;
        let across = n * r.position as i64;
        weight += n;
        if g.orientation == Orientation::Vertical {
            sx += across;
            sy += along;
        } else {
            sx += along;
            sy += across;
        }
    }
    [
        g.left as f64 + sx as f64 / weight as f64,
        g.top as f64 + sy as f64 / weight as f64,
    ]
}

/// Fixed normal-beam snapshot, TOP then BOTTOM; fixed unassigned spot pool.
/// Assigned spots are recorded after acceptance, without pruning this stage's
/// pool. Existing beam/hook areas suppress duplicate center-contained items.
pub fn build_hooks(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    scale: ExtensionScale,
    owner: &SystemSpots,
) -> Result<HookPass, String> {
    if state.system_id() != owner.system_id {
        return Err("inconsistent_late_hook_owner".into());
    }
    let standard = ItemParameters::new(scale.interline, scale.standard_height, false)?;
    let small = scale
        .small_height
        .map(|h| ItemParameters::new(scale.interline, h, true))
        .transpose()?;
    let pool = SpotPool::prepare(owner, registry, state.assigned_spot_ids())?;
    let initial_ids = state
        .inters()
        .iter()
        .filter(|b| !b.is_removed() && matches!(b.kind, "BeamInter" | "SmallBeamInter"))
        .map(|b| b.id())
        .collect::<Vec<_>>();
    initial_ids
        .len()
        .checked_mul(pool.ordered_ids().len())
        .and_then(|n| n.checked_mul(2))
        .filter(|n| *n <= 1_000_000)
        .ok_or("unsupported_late_hook_lookup_work")?;
    let mut report = HookPass {
        initial_ids,
        pool_ids: pool.ordered_ids().to_vec(),
        skipped_small: Vec::new(),
        attempts: Vec::new(),
    };
    let mut sampling_work = 0usize;
    for &id in &report.initial_ids {
        let parent = state.active(id).unwrap().clone();
        let params = match small {
            Some(p)
                if (parent.item.height - standard.typical_height).abs()
                    > (parent.item.height - p.typical_height).abs() =>
            {
                p
            }
            _ => standard,
        };
        if params.is_small {
            report.skipped_small.push(id);
            continue;
        }
        for side in [VerticalSide::Top, VerticalSide::Bottom] {
            let offset =
                1.5 * parent.item.height * if side == VerticalSide::Top { -1. } else { 1. };
            let mut band = parent.item.clone();
            band.median[1] += offset;
            band.median[3] += offset;
            let band = area(&band)?;
            for &spot in pool.ordered_ids() {
                let glyph = &registry.get(spot)?.glyph;
                if !band.intersects_rect([
                    glyph.left as f64,
                    glyph.top as f64,
                    glyph.width as f64,
                    glyph.height as f64,
                ])? {
                    continue;
                }
                let mut attempt = HookAttempt {
                    beam: id,
                    side,
                    spot,
                    rejection: None,
                    appended: None,
                };
                let thickness = glyph_geometry(glyph)?.mean_thickness;
                let reason = if (glyph.width as f64) < params.min_hook_width_low {
                    Some("too narrow")
                } else if thickness < params.min_height_low {
                    Some("too slim")
                } else if thickness > params.max_height_high {
                    Some("too thick")
                } else {
                    None
                };
                if let Some(reason) = reason {
                    attempt.rejection = Some(reason);
                    report.attempts.push(attempt);
                    continue;
                }
                let c = centroid(glyph);
                let m = parent.item.median;
                let slope = (m[3] - m[1]) / (m[2] - m[0]);
                let x1 = glyph.left as f64;
                let x2 = x1 + glyph.width as f64;
                let item = BeamItem {
                    median: [
                        x1,
                        c[1] + slope * (x1 - c[0]) + 0.5,
                        x2,
                        c[1] + slope * (x2 - c[0]) + 0.5,
                    ],
                    height: parent.item.height,
                };
                let core = area(&item)?;
                let b = core.bounds();
                let center = [
                    b[0] as f64 + b[2] as f64 / 2.,
                    b[1] as f64 + b[3] as f64 / 2.,
                ];
                let mut overlap = false;
                for beam in state.inters().iter().filter(|b| !b.is_removed()) {
                    if core.intersects_rect(beam.bounds.map(f64::from))?
                        && area(&beam.item)?.contains(center)?
                    {
                        overlap = true;
                        break;
                    }
                }
                if overlap {
                    attempt.rejection = Some("overlap");
                    report.attempts.push(attempt);
                    continue;
                }
                let p = crate::beam_impacts::Parameters::new(scale.interline)?;
                let areas =
                    ItemAreas::new(&item, p.belt_margin_dx, p.belt_margin_dy, p.belt_margin_dy)?;
                for b in [areas.core_bounds(), areas.outer_bounds()] {
                    sampling_work = sampling_work
                        .checked_add(
                            (b[2] as usize)
                                .checked_mul(b[3] as usize)
                                .ok_or("unsupported_late_hook_sampling_work")?,
                        )
                        .filter(|n| *n <= 1_000_000_000)
                        .ok_or("unsupported_late_hook_sampling_work")?;
                }
                let evaluated = Evaluator::new(image, scale.interline)?.compute_hook_impacts(
                    &item,
                    parent.impacts.values[5],
                    &params,
                )?;
                if let Some(impacts) = evaluated.impacts.filter(|v| v.accepted_as_hook()) {
                    attempt.appended = Some(state.register_hook(
                        image,
                        registry,
                        &MergeCandidate {
                            item,
                            is_small: false,
                            impacts,
                        },
                        spot,
                    )?);
                } else {
                    attempt.rejection = Some("no good item");
                }
                report.attempts.push(attempt);
            }
        }
    }
    Ok(report)
}
