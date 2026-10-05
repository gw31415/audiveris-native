// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Uninterrupted BeamsBuilder.extendBeams scheduling at pinned 9e1e55cd.
use crate::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::Registry,
    beam_extend_in_parallel::{ParallelOwner, ThroughParallel, side_through_parallel},
    beam_extend_to_beam::ExtensionScale,
    beam_extension_geometry::HorizontalSide,
    beam_structure::ItemParameters,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotExtension {
    pub initial_ids: Vec<usize>,
    pub skipped_removed: Vec<usize>,
    pub sides: Vec<(usize, HorizontalSide, ThroughParallel)>,
}

/// Copy the initial active SIG order once. Each surviving original beam tries
/// LEFT then RIGHT; any accepted replacement ends that beam's side loop.
/// Appended replacements remain available to lookups but are never revisited.
/// This boundary stops before late hooks, beam groups and multiple rests.
pub fn extend_beams(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    scale: ExtensionScale,
    owner: ParallelOwner<'_>,
) -> Result<SnapshotExtension, String> {
    ItemParameters::new(scale.interline, scale.standard_height, false)?;
    if let Some(height) = scale.small_height {
        ItemParameters::new(scale.interline, height, true)?;
    }
    if state.system_id() != owner.free_glyphs.system_id
        || !owner
            .geometry
            .systems
            .iter()
            .any(|s| s.id == state.system_id())
    {
        return Err("inconsistent_beam_extension_loop_owner".into());
    }
    let initial_ids = state
        .inters()
        .iter()
        .filter(|b| !b.is_removed())
        .map(|b| b.id())
        .collect::<Vec<_>>();
    // At most one appended replacement per snapshot entry and two side attempts.
    // Bound repeated parallel/side scans before any candidate mutation.
    let raw_bound = state
        .inters()
        .len()
        .checked_add(initial_ids.len())
        .ok_or("unsupported_beam_extension_loop_work")?;
    let work = raw_bound
        .checked_mul(raw_bound.saturating_add(owner.free_glyphs.glyph_ids.len()))
        .and_then(|n| n.checked_mul(initial_ids.len()))
        .and_then(|n| n.checked_mul(2))
        .filter(|n| *n <= 1_000_000_000)
        .ok_or("unsupported_beam_extension_loop_work")?;
    // Full area containment is queried at most twice per candidate on each side.
    // Pairwise lookup scans are not additional system-area point queries.
    let point_queries = initial_ids
        .len()
        .checked_mul(raw_bound)
        .and_then(|n| n.checked_mul(4))
        .ok_or("unsupported_beam_extension_loop_work")?;
    owner.geometry.validate_point_work(point_queries)?;
    let _ = work;
    let mut report = SnapshotExtension {
        initial_ids,
        skipped_removed: Vec::new(),
        sides: Vec::new(),
    };
    for &id in &report.initial_ids {
        if state.active(id).is_none() {
            report.skipped_removed.push(id);
            continue;
        }
        for side in [HorizontalSide::Left, HorizontalSide::Right] {
            let decision = side_through_parallel(state, image, registry, id, side, scale, owner)?;
            report.sides.push((id, side, decision));
            // All four original acceptance paths register a replacement and remove
            // the target. Rejection leaves it active and proceeds to the next side.
            if state.active(id).is_none() {
                break;
            }
        }
    }
    Ok(report)
}
