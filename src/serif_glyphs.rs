// SPDX-License-Identifier: AGPL-3.0-or-later
// BarFilamentBuilder(dy=0), BarFilamentFactory membership and toGlyph pixels.
use crate::{
    bar_sticks,
    sections::{Bounds, Orientation, Section},
    serif_peaks::Peak,
};
use audiveris_staff_removal::{GlyphPixels, Orientation as GlyphOrientation, Run};
/// Sections are supplied in original vLag-then-hLag order, within system bounds.
/// Stable abscissa sorting preserves that order on ties. No registry mutation.
pub fn build(
    peak: &Peak,
    sections: &[Section],
    interline: usize,
) -> Result<Option<GlyphPixels>, String> {
    if interline == 0
        || interline > 10000
        || sections.len() > 10000
        || peak.start < 0
        || peak.top < 0
        || peak.stop < peak.start
        || peak.bottom < peak.top
        || peak.stop > 16384
        || peak.bottom > 16384
    {
        return Err("invalid_serif_filament_input".into());
    }
    let mut work = 0usize;
    for s in sections {
        if s.runs.is_empty()
            || s.weight == 0
            || s.first_pos > 16384
            || s.runs.len() > 16384
            || s.runs
                .iter()
                .any(|r| r.length == 0 || r.start.checked_add(r.length).is_none_or(|n| n > 16384))
            || s.first_pos + s.runs.len() > 16384
            || s.runs.iter().map(|r| r.length).sum::<usize>() != s.weight
        {
            return Err("invalid_serif_section".into());
        }
        work = work.checked_add(s.weight).ok_or("serif_filament_budget")?;
    }
    if work > crate::MAX_PIXELS || sections.len().checked_pow(3).is_none_or(|n| n > 10_000_000) {
        return Err("serif_filament_budget".into());
    }
    let core = Bounds {
        x: peak.start as usize,
        y: peak.top as usize,
        width: (peak.stop - peak.start + 1) as usize,
        height: (peak.bottom - peak.top + 1) as usize,
    };
    let mut source = sections
        .iter()
        .filter(|s| {
            let b = s.bounds();
            b.width <= core.width && bar_sticks::intersects(b, core)
        })
        .collect::<Vec<_>>();
    source.sort_by_key(|s| s.bounds().x);
    let min_length = (0.5 * interline as f64).round_ties_even() as usize;
    let mut members = source
        .iter()
        .copied()
        .filter(|s| {
            let x = bar_sticks::centroid_x(s);
            s.bounds().height >= min_length && x >= core.x && x < core.x + core.width
        })
        .collect::<Vec<_>>();
    if members.is_empty() {
        return Ok(None);
    }
    loop {
        let next = source.iter().copied().find(|s| {
            !members.iter().any(|m| std::ptr::eq(*m, *s)) && {
                let b = s.bounds();
                b.x >= core.x
                    && b.x + b.width <= core.x + core.width
                    && members.iter().any(|m| bar_sticks::touch(m, s))
            }
        });
        if let Some(s) = next {
            members.push(s);
        } else {
            break;
        }
    }
    let left = members.iter().map(|s| s.bounds().x).min().unwrap();
    let top = members.iter().map(|s| s.bounds().y).min().unwrap();
    let right = members
        .iter()
        .map(|s| {
            let b = s.bounds();
            b.x + b.width
        })
        .max()
        .unwrap();
    let bottom = members
        .iter()
        .map(|s| {
            let b = s.bounds();
            b.y + b.height
        })
        .max()
        .unwrap();
    let width = right - left;
    let height = bottom - top;
    if width
        .checked_mul(height)
        .is_none_or(|n| n > crate::MAX_PIXELS)
    {
        return Err("serif_filament_budget".into());
    }
    let mut black = vec![false; width * height];
    for s in members {
        for (i, r) in s.runs.iter().enumerate() {
            for c in r.start..r.start + r.length {
                let (x, y) = if s.orientation == Orientation::Vertical {
                    (s.first_pos + i, c)
                } else {
                    (c, s.first_pos + i)
                };
                black[(y - top) * width + x - left] = true;
            }
        }
    }
    let mut runs = Vec::new();
    for x in 0..width {
        let mut y = 0;
        while y < height {
            if !black[y * width + x] {
                y += 1;
                continue;
            }
            let start = y;
            while y < height && black[y * width + x] {
                y += 1;
            }
            runs.push(Run {
                position: x,
                start,
                length: y - start,
            });
        }
    }
    Ok(Some(GlyphPixels {
        left: left as i32,
        top: top as i32,
        width,
        height,
        orientation: GlyphOrientation::Vertical,
        runs,
    }))
}
