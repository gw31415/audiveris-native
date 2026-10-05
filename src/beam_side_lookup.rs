// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// BeamsBuilder.getSideBeam at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Read-only lookup only: no merge, extension loop or graph mutations.
use crate::{
    beam_extension_geometry::{HorizontalSide, side_area_of},
    beam_structure::BeamItem,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LookupBeam {
    pub id: usize,
    pub item: BeamItem,
    pub bounds: [i32; 4],
    pub removed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SideLookup {
    pub intersected: Vec<usize>,
    pub ordered: Vec<usize>,
    pub selected: Option<usize>,
}

/// Raw registration order is significant: equal gaps retain that order.
/// Bounds are the inter's rectangle, not a pixel intersection or the median.
/// Uses original default constants; diagnostic Java area attachments are omitted.
pub fn get_side_beam(
    raw: &[LookupBeam],
    target: usize,
    side: HorizontalSide,
    reach: Option<f64>,
    interline: usize,
) -> Result<SideLookup, String> {
    if raw.len() > 100_000 || interline == 0 || interline > crate::sections::MAX_DIMENSION {
        return Err("unsupported_beam_lookup_size".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    for b in raw {
        let m = b.item.median;
        if !ids.insert(b.id)
            || m.iter().any(|v| !v.is_finite() || v.abs() > 8_000_000.)
            || m[0] >= m[2]
            || !b.item.height.is_finite()
            || b.item.height < 0.
            || b.item.height > 8_000_000.
        {
            return Err("unsupported_beam_lookup_geometry".into());
        }
    }
    let current = raw
        .iter()
        .find(|b| b.id == target)
        .ok_or("unknown_beam_lookup_target")?;
    let m = current.item.median;
    let area = side_area_of(
        m,
        current.item.height,
        side,
        0.,
        reach.unwrap_or(4. * interline as f64),
        0.,
    )?;
    let endpoint = match side {
        HorizontalSide::Left => [m[0], m[1]],
        HorizontalSide::Right => [m[2], m[3]],
    };
    let slope = (m[3] - m[1]) / (m[2] - m[0]);
    let max_y = (0.25 * interline as f64).round_ties_even();
    let mut intersected = Vec::new();
    let mut ordered = Vec::new();
    for b in raw {
        if b.removed || !area.intersects_rect(b.bounds.map(f64::from))? {
            continue;
        }
        intersected.push(b.id);
        if b.id == target {
            continue;
        }
        let n = b.item.median;
        let dx = n[2] - n[0];
        let dy = n[3] - n[1];
        if (dy / dx - slope).abs() > 0.065 {
            continue;
        }
        // Preserve Line2D.ptLineDist arithmetic, including subtraction cancellation.
        let px = endpoint[0] - n[0];
        let py = endpoint[1] - n[1];
        let dot = px * dx + py * dy;
        let square = (px * px + py * py) - dot * dot / (dx * dx + dy * dy);
        if square.max(0.).sqrt() > max_y {
            continue;
        }
        let gap = match side {
            HorizontalSide::Left => endpoint[0] - n[2],
            HorizontalSide::Right => n[0] - endpoint[0],
        };
        ordered.push((b.id, gap));
    }
    ordered.sort_by(|a, b| a.1.total_cmp(&b.1));
    let ordered: Vec<_> = ordered.into_iter().map(|v| v.0).collect();
    Ok(SideLookup {
        selected: ordered.first().copied(),
        intersected,
        ordered,
    })
}
