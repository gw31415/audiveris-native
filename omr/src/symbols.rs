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
        for y in s.top..=s.bottom {
            for x in [s.x - 1, s.x, s.x + 1] {
                if img.get(x, y) {
                    let (a, b) = img.hrun_at(x, y);
                    if b - a + 1 <= ms {
                        for xx in a..=b {
                            img.set(xx, y, false);
                        }
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
        let bot = staves[*sys.staves.last().unwrap()].bottom(sys.left as f64).round() as i32 + 1;
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

pub fn min_grade(shape: &str) -> f64 {
    match shape {
        "DOT_set" => 0.2,
        s if s.starts_with("DYNAMICS") => 0.6,
        _ => 0.35,
    }
}

/// Classify residual glyphs near the staves.
pub fn recognize(residual: &Bitmap, staves: &[Staff], scale: &Scale) -> Vec<Symbol> {
    let il = scale.interline;
    let min_weight = ((il * il) as f64 * 0.04).max(4.0) as usize;
    let mut comps: Vec<Component> = components(residual, None).into_iter().filter(|c| c.weight() >= min_weight).collect();
    // Keep glyphs whose center is close to a staff.
    let staff_of = |r: &Rect| -> Option<usize> {
        let (x, y) = (r.cx(), r.cy());
        staves
            .iter()
            .enumerate()
            .filter(|(_, s)| x >= (s.left - il) as f64 && x <= (s.right + il) as f64)
            .map(|(i, s)| {
                let d = if y < s.top(x) { s.top(x) - y } else if y > s.bottom(x) { y - s.bottom(x) } else { 0.0 };
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
    let mut cands: Vec<(Vec<usize>, &'static str, f64, Rect)> = Vec::new();
    let mut eval = |idx: Vec<usize>, comps: &[Component], cands: &mut Vec<(Vec<usize>, &'static str, f64, Rect)>| {
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
        {
            cands.push((idx, s, gr, g.bounds));
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
                    if comps[k].bounds.x > comps[j].bounds.right().max(comps[i].bounds.right()) + gap {
                        break;
                    }
                    if near(&comps[i].bounds, &comps[k].bounds) || near(&comps[j].bounds, &comps[k].bounds) {
                        eval(vec![i, j, k], &comps, &mut cands);
                    }
                }
            }
        }
    }
    // Prefer larger groups at equal quality, then grade.
    cands.sort_by(|a, b| (b.2 + 0.05 * b.0.len() as f64).partial_cmp(&(a.2 + 0.05 * a.0.len() as f64)).unwrap());
    let mut used = vec![false; comps.len()];
    let mut out = Vec::new();
    for (idx, shape, grade, bounds) in cands {
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
            if (shape.starts_with("TIME_") || shape.ends_with("_TIME") || shape.contains("CLEF")) && !inside {
                continue;
            }
            out.push(Symbol { shape, grade, bounds, staff: si });
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
    let zone = if stem.up {
        Rect::new(stem.x - 2, stem.top - 2, il + il / 2, (len / 2).min(4 * il))
    } else {
        let h = (len / 2).min(4 * il);
        Rect::new(stem.x - 2, stem.bottom - h, il + il / 2, h + 3)
    };
    let parts: Vec<Component> = components(residual, Some(zone)).into_iter().filter(|c| c.weight() >= 6).collect();
    if parts.is_empty() {
        return 0;
    }
    let refs: Vec<&Component> = parts.iter().collect();
    let g = Component::merge(&refs);
    let ev = classify(&g, il);
    let suffix_ok = |s: &str| s.ends_with("_DOWN") != stem.up;
    ev.iter()
        .find(|(s, _)| s.starts_with("FLAG_") && !s.contains("SMALL") && suffix_ok(s))
        .filter(|(_, g)| *g >= 0.05)
        .map(|(s, _)| s.trim_start_matches("FLAG_").trim_end_matches("_DOWN").parse().unwrap_or(0))
        .unwrap_or(0)
}
