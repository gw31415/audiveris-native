// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original 5.11 filterHeadParts pixel test. Its removal is VIP-dependent;
//! preserve this observed branch, separately from later bbox/item filtering.
use crate::beam_dispatch::{self, Registry};
use audiveris_staff_removal::{GlyphPixels, Orientation};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Filtered {
    pub remaining_weights: Vec<Option<usize>>,
    pub retained_ids: Vec<usize>,
}
fn points(g: &GlyphPixels, mut visit: impl FnMut(i32, i32)) {
    for r in &g.runs {
        for k in 0..r.length {
            let (x, y) = if g.orientation == Orientation::Vertical {
                (r.position, r.start + k)
            } else {
                (r.start + k, r.position)
            };
            visit(g.left + x as i32, g.top + y as i32);
        }
    }
}
fn contains(g: &GlyphPixels, x: i32, y: i32) -> bool {
    let (rx, ry) = (x - g.left, y - g.top);
    if rx < 0 || ry < 0 || rx as usize >= g.width || ry as usize >= g.height {
        return false;
    }
    let (position, coordinate) = if g.orientation == Orientation::Vertical {
        (rx as usize, ry as usize)
    } else {
        (ry as usize, rx as usize)
    };
    // Validated run order permits membership lookup without materializing
    // every head pixel. Preserve physical holes and either table orientation.
    let i = g
        .runs
        .partition_point(|r| (r.position, r.start) <= (position, coordinate));
    i > 0
        && g.runs[i - 1].position == position
        && coordinate < g.runs[i - 1].start + g.runs[i - 1].length
}
/// Registry IDs are canonical caller identities. Count chunk pixels within
/// the physical head glyph only on positive GeoUtil y-overlap. Remove a
/// candidate iff remaining weight <15 AND head VIP is true. Zero overlap is
/// retained, even if the pixel count would be small. Repeated IDs retain their
/// list positions; registry, cache, head and SIG state remain unchanged.
pub fn filter(
    registry: &Registry,
    ids: &[usize],
    head: &GlyphPixels,
    vip: bool,
) -> Result<Filtered, String> {
    if ids.len() > 4096 {
        return Err("stem_chunk_head_filter_budget".into());
    }
    beam_dispatch::centroid(head)?;
    let weight = |g: &GlyphPixels| g.runs.iter().map(|r| r.length).sum::<usize>();
    let mut pixels = weight(head);
    for &id in ids {
        pixels = pixels
            .checked_add(weight(&registry.get(id)?.glyph))
            .filter(|p| *p <= 8_000_000)
            .ok_or("stem_chunk_head_filter_pixel_budget")?;
    }
    let head_bottom = head.top as i64 + head.height as i64 - 1;
    let mut remaining_weights = Vec::with_capacity(ids.len());
    let mut retained_ids = Vec::new();
    for &id in ids {
        let g = &registry.get(id)?.glyph;
        let bottom = g.top as i64 + g.height as i64 - 1;
        let overlap = head_bottom.min(bottom) - (head.top as i64).max(g.top as i64) + 1;
        if overlap <= 0 {
            remaining_weights.push(None);
            retained_ids.push(id);
            continue;
        }
        let mut contained = 0;
        points(g, |x, y| {
            if y as i64 >= head.top as i64 && y as i64 <= head_bottom && contains(head, x, y) {
                contained += 1;
            }
        });
        let remaining = weight(g) - contained;
        remaining_weights.push(Some(remaining));
        if remaining >= 15 || !vip {
            retained_ids.push(id);
        }
    }
    Ok(Filtered {
        remaining_weights,
        retained_ids,
    })
}
