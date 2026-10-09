//! Fixed-shape symbols (Audiveris SYMBOLS step): the image left after erasing
//! staff lines, heads, stems, beams, ledgers, bars and headers is split into
//! glyphs. Glyphs and small clusters of neighboring glyphs are classified;
//! the best compatible interpretations are kept.

use crate::beams::{Beam, Ledger};
use crate::binary::{Bitmap, Component, Rect, components};
use crate::classifier::classify;
use crate::grid::System;
use crate::heads::{Catalog, Head};
use crate::scale::Scale;
use crate::staff::Staff;
use crate::stems::Stem;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Symbol {
    pub shape: &'static str,
    pub grade: f64,
    pub bounds: Rect,
    pub staff: usize,
    /// Mass center.
    pub centroid: (f64, f64),
}

/// Erase all recognized items from the staff-free image.
#[allow(clippy::too_many_arguments)]
pub fn residual(
    nostaff: &Bitmap,
    heads: &[Head],
    catalog: &Catalog,
    stems: &[Stem],
    beams: &[Beam],
    ledgers: &[Ledger],
    systems: &[System],
    staves: &[Staff],
    header_items: &[Rect],
    scale: &Scale,
) -> Bitmap {
    let mut img = nostaff.clone();
    // Heads: erase template foreground (dilated by one pixel).
    // Only good heads are erased; weak ones may still be other symbols.
    for h in heads.iter().filter(|h| h.ctx >= 0.6) {
        let t = catalog.template(h);
        // Recover the MIDDLE_LEFT anchor from the slim bounds.
        let (ox, oy) = t.middle_left();
        let ax = h.bounds.x - t.slim.x + ox;
        let ay = h.bounds.y - t.slim.y + oy;
        let (ux, uy) = t.origin_at(ax, ay);
        for (kx, ky) in t.fore_points() {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    img.set(ux + kx + dx, uy + ky + dy, false);
                }
            }
        }
        let r = h.bounds.grow(1, 1);
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                img.set(x, y, false);
            }
        }
    }
    // Stems: erase thin horizontal runs on the stem column.
    let ms = scale.max_stem + 1;
    let good_stem = |s: &&Stem| s.heads.iter().any(|&h| heads[h].ctx >= 0.6);
    for s in stems.iter().filter(good_stem) {
        // Stem columns (covered by black over half the stem length at least):
        // erase their pixels where the horizontal run is stem-thin, so that
        // flags and dots touching the stem are kept.
        let len = s.bottom - s.top + 1;
        let is_col =
            |x: i32| ((s.top..=s.bottom).filter(|&y| nostaff.get(x, y)).count() as i32) * 2 >= len;
        // Contiguous stem columns around s.x only.
        let mut x0 = s.x;
        while x0 > s.x - ms && is_col(x0 - 1) {
            x0 -= 1;
        }
        let mut x1 = s.x;
        while x1 < s.x + ms && is_col(x1 + 1) {
            x1 += 1;
        }
        for x in x0..=x1 {
            for y in s.top..=s.bottom {
                if nostaff.get(x, y) {
                    let (a, b) = nostaff.hrun_at(x, y);
                    if b - a < ms {
                        img.set(x, y, false);
                    }
                }
            }
        }
    }
    for s in stems.iter().filter(good_stem) {
        for &bi in &s.beams {
            for &(x, y) in &beams[bi].pixels {
                img.set(x, y, false);
            }
        }
    }
    let thin = scale.max_line + 1;
    for l in ledgers {
        for x in l.x0..=l.x1 {
            let (a, b) = img.vrun_at(x, l.y.round() as i32);
            if b - a + 1 <= thin {
                for y in a..=b {
                    img.set(x, y, false);
                }
            }
        }
    }
    for sys in systems {
        let top = staves[sys.staves[0]].top(sys.left as f64).round() as i32 - 1;
        let bot = staves[*sys.staves.last().unwrap()]
            .bottom(sys.left as f64)
            .round() as i32
            + 1;
        for b in &sys.boundaries {
            for x in b.x0 - 1..=b.x1 + 1 {
                for y in top..=bot {
                    img.set(x, y, false);
                }
            }
        }
    }
    for r in header_items {
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                img.set(x, y, false);
            }
        }
    }
    img
}

/// Shapes interpreted by this pipeline.
fn relevant(shape: &str) -> bool {
    matches!(
        shape,
        "DOT_set"
            | "HW_REST_set"
            | "LONG_REST"
            | "BREVE_REST"
            | "QUARTER_REST"
            | "EIGHTH_REST"
            | "ONE_16TH_REST"
            | "ONE_32ND_REST"
            | "ONE_64TH_REST"
            | "ONE_128TH_REST"
            | "FLAT"
            | "NATURAL"
            | "SHARP"
            | "DOUBLE_SHARP"
            | "DOUBLE_FLAT"
            | "TUPLET_THREE"
            | "GRACE_NOTE"
            | "GRACE_NOTE_DOWN"
            | "GRACE_NOTE_SLASH"
            | "GRACE_NOTE_SLASH_DOWN"
            | "TUPLET_SIX"
            | "G_CLEF"
            | "G_CLEF_SMALL"
            | "G_CLEF_8VA"
            | "G_CLEF_8VB"
            | "C_CLEF"
            | "F_CLEF"
            | "F_CLEF_SMALL"
            | "F_CLEF_8VA"
            | "F_CLEF_8VB"
            | "DYNAMICS_P"
            | "DYNAMICS_PP"
            | "DYNAMICS_PPP"
            | "DYNAMICS_MP"
            | "DYNAMICS_F"
            | "DYNAMICS_FF"
            | "DYNAMICS_FFF"
            | "DYNAMICS_MF"
            | "DYNAMICS_FP"
            | "DYNAMICS_FZ"
            | "DYNAMICS_SF"
            | "DYNAMICS_SFZ"
            | "FERMATA"
            | "FERMATA_BELOW"
            | "ACCENT"
            | "TENUTO"
            | "STACCATISSIMO"
            | "MARCATO"
    ) || shape.starts_with("FLAG_")
        || shape.starts_with("TIME_")
        || shape == "COMMON_TIME"
        || shape == "CUT_TIME"
}

/// Geometric plausibility of a shape (Audiveris ShapeChecker), in interline units.
fn plausible(shape: &str, b: &Rect, il: f64) -> bool {
    let (w, h) = (b.w as f64 / il, b.h as f64 / il);
    match shape {
        "DOT_set" => {
            (0.2..=0.9).contains(&w) && (0.2..=0.9).contains(&h) && w / h <= 2.0 && h / w <= 2.0
        }
        "HW_REST_set" => (0.7..=2.0).contains(&w) && (0.25..=0.9).contains(&h),
        "QUARTER_REST" => (1.8..=3.6).contains(&h),
        "BREVE_REST" | "LONG_REST" => (0.8..=4.2).contains(&h) && w <= 1.5,
        s if s.ends_with("_REST") => (1.1..=6.0).contains(&h),
        "SHARP" | "NATURAL" | "DOUBLE_FLAT" => (1.8..=4.0).contains(&h),
        "FLAT" => (1.5..=3.5).contains(&h),
        "DOUBLE_SHARP" => (0.5..=1.5).contains(&h),
        s if s.starts_with("FLAG_") => h >= 1.0,
        _ => true,
    }
}

pub fn min_grade(shape: &str) -> f64 {
    match shape {
        "DOT_set" => 0.2,
        "BREVE_REST" | "LONG_REST" => 0.7,
        "HW_REST_set" => 0.15,
        s if s.starts_with("DYNAMICS") => 0.6,
        _ => 0.35,
    }
}

/// Classify residual glyphs near the staves.
pub fn recognize(residual: &Bitmap, staves: &[Staff], scale: &Scale) -> Vec<Symbol> {
    let il = scale.interline;
    let min_weight = ((il * il) as f64 * 0.04).max(4.0) as usize;
    let mut comps: Vec<Component> = components(residual, None)
        .into_iter()
        .filter(|c| c.weight() >= min_weight)
        .collect();
    // Keep glyphs whose center is close to a staff.
    let staff_of = |r: &Rect| -> Option<usize> {
        let (x, y) = (r.cx(), r.cy());
        staves
            .iter()
            .enumerate()
            .filter(|(_, s)| x >= (s.left - il) as f64 && x <= (s.right + il) as f64)
            .map(|(i, s)| {
                let d = if y < s.top(x) {
                    s.top(x) - y
                } else if y > s.bottom(x) {
                    y - s.bottom(x)
                } else {
                    0.0
                };
                (i, d)
            })
            .filter(|(_, d)| *d <= 4.0 * il as f64)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
    };
    comps.retain(|c| staff_of(&c.bounds).is_some() && c.bounds.w <= 6 * il && c.bounds.h <= 8 * il);
    comps.sort_by_key(|c| c.bounds.x);
    // Candidate groups: single glyphs and pairs/triples of close glyphs.
    let gap = (0.5 * il as f64) as i32;
    let near = |a: &Rect, b: &Rect| a.grow(gap, gap).intersects(b);
    type Cand = (Vec<usize>, &'static str, f64, Rect, (f64, f64));
    let mut cands: Vec<Cand> = Vec::new();
    let eval = |idx: Vec<usize>, comps: &[Component], cands: &mut Vec<Cand>| {
        let parts: Vec<&Component> = idx.iter().map(|&i| &comps[i]).collect();
        let g = Component::merge(&parts);
        if g.bounds.w > 6 * il || g.bounds.h > 8 * il {
            return;
        }
        let ev = classify(&g, il);
        if std::env::var("OMR_SYM").is_ok() {
            eprintln!("glyph {:?} -> {:?}", g.bounds, &ev[..2.min(ev.len())]);
        }
        if let Some(&(s, gr)) = ev.first()
            && relevant(s)
            && gr >= min_grade(s)
            && plausible(s, &g.bounds, il as f64)
        {
            let n = g.pixels.len() as f64;
            let c = (
                g.pixels.iter().map(|p| p.0 as f64).sum::<f64>() / n,
                g.pixels.iter().map(|p| p.1 as f64).sum::<f64>() / n,
            );
            cands.push((idx, s, gr, g.bounds, c));
        }
    };
    for i in 0..comps.len() {
        eval(vec![i], &comps, &mut cands);
        for j in i + 1..comps.len() {
            if comps[j].bounds.x > comps[i].bounds.right() + gap {
                break;
            }
            if near(&comps[i].bounds, &comps[j].bounds) {
                eval(vec![i, j], &comps, &mut cands);
                for k in j + 1..comps.len() {
                    if comps[k].bounds.x
                        > comps[j].bounds.right().max(comps[i].bounds.right()) + gap
                    {
                        break;
                    }
                    if near(&comps[i].bounds, &comps[k].bounds)
                        || near(&comps[j].bounds, &comps[k].bounds)
                    {
                        eval(vec![i, j, k], &comps, &mut cands);
                    }
                }
            }
        }
    }
    // Prefer larger groups at equal quality, then grade.
    cands.sort_by(|a, b| {
        (b.2 + 0.05 * b.0.len() as f64)
            .partial_cmp(&(a.2 + 0.05 * a.0.len() as f64))
            .unwrap()
    });
    let mut used = vec![false; comps.len()];
    let mut out = Vec::new();
    for (idx, shape, grade, bounds, centroid) in cands {
        if idx.iter().any(|&i| used[i]) {
            continue;
        }
        for &i in &idx {
            used[i] = true;
        }
        if let Some(si) = staff_of(&bounds) {
            // Time signatures and clefs lie within the staff height.
            let st = &staves[si];
            let inside = bounds.cy() > st.top(bounds.cx()) && bounds.cy() < st.bottom(bounds.cx());
            if (shape.starts_with("TIME_") || shape.ends_with("_TIME") || shape.contains("CLEF"))
                && !inside
            {
                continue;
            }
            // Whole rests hang from a line, half rests sit on a line.
            if shape == "HW_REST_set" {
                let x = bounds.cx();
                let on_line = (-6..=6).step_by(2).any(|p| {
                    let ly = st.pitch_y(p as f64, x);
                    (bounds.y as f64 - ly).abs() <= 0.3 * il as f64
                        || (bounds.bottom() as f64 - ly).abs() <= 0.3 * il as f64
                });
                if !on_line {
                    continue;
                }
            }
            out.push(Symbol {
                shape,
                grade,
                bounds,
                staff: si,
                centroid,
            });
        }
    }
    out.sort_by_key(|s| s.bounds.x);
    out
}

/// Number of flags on a beamless stem: glyphs of the residual image near the
/// stem tip, classified among flag shapes only.
pub fn stem_flags(residual: &Bitmap, stem: &Stem, scale: &Scale) -> u32 {
    let il = scale.interline;
    let len = stem.bottom - stem.top;
    // From the tip to just before the heads.
    let h = (len - il).clamp(il, 5 * il);
    let zone = if stem.up {
        Rect::new(stem.x - 2, stem.top - 2, il + il / 2, h)
    } else {
        Rect::new(stem.x - 2, stem.bottom - h, il + il / 2, h + 3)
    };
    let parts: Vec<Component> = components(residual, Some(zone))
        .into_iter()
        .filter(|c| c.weight() >= 6 && c.bounds.w >= 3)
        .collect();
    if parts.is_empty() {
        return 0;
    }
    let refs: Vec<&Component> = parts.iter().collect();
    let g = Component::merge(&refs);
    let ev = classify(&g, il);
    if std::env::var("OMR_FLAG").is_ok() {
        eprintln!(
            "flag stem x {} {}..{} glyph {:?} -> {:?}",
            stem.x,
            stem.top,
            stem.bottom,
            g.bounds,
            &ev[..3]
        );
    }
    let suffix_ok = |s: &str| s.ends_with("_DOWN") != stem.up;
    ev.first()
        .filter(|(s, g)| s.starts_with("FLAG_") && suffix_ok(s) && *g >= 0.2)
        .map(|(s, _)| {
            s.trim_start_matches("FLAG_")
                .trim_end_matches("_DOWN")
                .parse()
                .unwrap_or(0)
        })
        .unwrap_or(0)
}
