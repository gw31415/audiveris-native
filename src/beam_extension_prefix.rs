// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Bounded prefix of BeamsBuilder.extendBeams, pinned 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{
    GrayImage, beam_candidates::SystemCandidates, beam_dispatch::Registry,
    beam_extend_to_beam::ExtensionScale, beam_extension_geometry::HorizontalSide,
};

#[derive(Clone, Debug, PartialEq)]
pub enum SideDecision {
    Merged {
        beam: usize,
        neighbor: usize,
        appended: usize,
    },
    NeedDownstream {
        beam: usize,
        side: HorizontalSide,
        neighbor: Option<usize>,
        max_extension: Option<i32>,
        parallel_eligible_after_stem_and_spot: bool,
    },
}

/// The real stem/spot branches must run after NeedDownstream; parallel follows
/// only if both reject and no side beam exists. RIGHT never directly merges.
pub fn side_prefix(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    beam: usize,
    side: HorizontalSide,
    scale: ExtensionScale,
) -> Result<SideDecision, String> {
    if state.active(beam).is_none() {
        return Err("invalid_beam_extension_target".into());
    }
    let neighbor = state
        .lookup_side(beam, side, None, scale.interline)?
        .selected;
    if let Some(id) = neighbor
        && side == HorizontalSide::Left
        && let Some(appended) = state.extend_to_beam(image, registry, beam, id, scale)?
    {
        return Ok(SideDecision::Merged {
            beam,
            neighbor: id,
            appended,
        });
    }
    let max_extension = neighbor.map(|id| {
        let a = state.active(beam).unwrap().item.median;
        let b = state.active(id).unwrap().item.median;
        let gap = match side {
            HorizontalSide::Left => a[0] - b[2],
            HorizontalSide::Right => b[0] - a[2],
        };
        let margin = (0.25 * scale.interline as f64).round_ties_even();
        (gap - margin).max(0.).round_ties_even() as i32
    });
    Ok(SideDecision::NeedDownstream {
        beam,
        side,
        neighbor,
        max_extension,
        parallel_eligible_after_stem_and_spot: neighbor.is_none(),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergePrefix {
    pub initial_ids: Vec<usize>,
    pub skipped_removed: Vec<usize>,
    pub decisions: Vec<SideDecision>,
    pub stopped_for_downstream: bool,
}

/// Capture the initial active SIG order once. Merged beams append to raw state
/// but never become new iterations; removed snapshot entries are skipped.
/// Stop at the first unimplemented stem branch. No false branch result is
/// substituted, and this function is not a full extension pass.
pub fn initial_merge_prefix(
    state: &mut SystemCandidates,
    image: &GrayImage,
    registry: &mut Registry,
    scale: ExtensionScale,
) -> Result<MergePrefix, String> {
    let initial_ids = state
        .inters()
        .iter()
        .filter(|v| !v.is_removed())
        .map(|v| v.id())
        .collect::<Vec<_>>();
    // Include the possible appended entries in the repeated lookup budget.
    if state
        .inters()
        .len()
        .checked_add(initial_ids.len())
        .and_then(|n| n.checked_mul(initial_ids.len()))
        .is_none_or(|work| work > 1_000_000_000)
    {
        return Err("unsupported_beam_extension_lookup_work".into());
    }
    let mut report = MergePrefix {
        initial_ids,
        skipped_removed: Vec::new(),
        decisions: Vec::new(),
        stopped_for_downstream: false,
    };
    for &id in &report.initial_ids {
        if state.active(id).is_none() {
            report.skipped_removed.push(id);
            continue;
        }
        // HorizontalSide.values begins LEFT. A successful merge breaks its side loop.
        let decision = side_prefix(state, image, registry, id, HorizontalSide::Left, scale)?;
        let pending = matches!(decision, SideDecision::NeedDownstream { .. });
        report.decisions.push(decision);
        if pending {
            report.stopped_for_downstream = true;
            break;
        }
    }
    Ok(report)
}
