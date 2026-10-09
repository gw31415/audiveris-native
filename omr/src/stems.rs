//! Stems (Audiveris STEMS step) and the REDUCTION of head candidates.
//!
//! Each head candidate looks for a thin vertical stick touching its left or
//! right side and extending beyond it. Heads sharing a stick share a stem.
//! Conflicting head candidates are then reduced by contextual grade (a head
//! supported by a stem is stronger), and stem-requiring heads without stem
//! are discarded.

use crate::beams::Beam;
use crate::binary::Bitmap;
use crate::heads::{Head, overlaps};
use crate::scale::Scale;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Stem {
    pub x: i32,
    pub top: i32,
    pub bottom: i32,
    pub up: bool,
    pub heads: Vec<usize>,
    /// Beams attached, ordered from the free end towards the heads.
    pub beams: Vec<usize>,
    pub flags: u32,
}

impl Stem {
    pub fn tip(&self) -> i32 {
        if self.up { self.top } else { self.bottom }
    }
}

struct Link {
    x: i32,
    top: i32,
    bottom: i32,
}

fn find_stem(nostaff: &Bitmap, hr: &[u16], head: &Head, scale: &Scale) -> Option<Link> {
    let b = head.bounds;
    let il = scale.interline as f64;
    let ms = scale.max_stem;
    let min_len = (1.8 * il) as i32;
    let w = nostaff.width as i32;
    let mut best: Option<(i32, Link)> = None;
    let ranges = [(b.x - 3, b.x + ms + 1), (b.right() - ms - 2, b.right() + 2)];
    // Longest thin vertical stroke leaving the head on one side, upwards or downwards.
    let stroke = |x0: i32, x1: i32, up: bool| -> i32 {
        (x0..=x1)
            .map(|x| {
                let y = if up { b.y + b.h / 3 } else { b.bottom() - b.h / 3 };
                if !nostaff.get(x, y) {
                    return 0;
                }
                let (t, bt) = nostaff.vrun_at(x, y);
                if up { b.y - t } else { bt - b.bottom() }
            })
            .max()
            .unwrap_or(0)
    };
    for (x0, x1) in ranges {
        for x in x0..=x1 {
            let yc = b.y + b.h / 2;
            // Start from any black pixel near the head center row in this column.
            let Some(ys) = (0..=b.h / 3).flat_map(|d| [yc - d, yc + d]).find(|&y| nostaff.get(x, y)) else {
                continue;
            };
            let (top, bottom) = nostaff.vrun_at(x, ys);
            let up_len = b.y - top;
            let down_len = bottom - b.bottom();
            let len = up_len.max(down_len);
            if len < min_len {
                continue;
            }
            // The part beyond the head must be thin.
            let (ya, yb) = if up_len >= down_len { (top, b.y - 1) } else { (b.bottom() + 1, bottom) };
            let n = (yb - ya + 1).max(1);
            let thin = (ya..=yb).filter(|&y| y >= 0 && (hr[(y * w + x) as usize] as i32) <= ms + 2).count() as i32;
            if (thin as f64) < (1.0 * il).max(0.3 * n as f64) {
                continue;
            }
            if best.as_ref().is_none_or(|(l, _)| len > *l) {
                best = Some((len, Link { x, top, bottom }));
            }
        }
    }
    let link = best.map(|b| b.1)?;
    if head.shape.has_hole() {
        // Natural / sharp signs: strokes leave the "hole" on both sides in opposite directions.
        let up = b.y - link.top >= link.bottom - b.bottom();
        let other = (b.x..b.right()).filter(|&x| (x - link.x).abs() > ms + 1).map(|x| stroke(x, x, !up)).max().unwrap_or(0);
        if other as f64 >= 0.6 * il {
            return None;
        }
    }
    Some(link)
}

/// Link heads to stems, reduce conflicting heads, build stems.
pub fn link(nostaff: &Bitmap, heads: Vec<Head>, scale: &Scale) -> (Vec<Head>, Vec<Stem>) {
    let hr = nostaff.horizontal_run_lengths();
    let links: Vec<Option<Link>> = heads.iter().map(|h| if h.shape.stemless() { None } else { find_stem(nostaff, &hr, h, scale) }).collect();
    // Contextual grade (Audiveris GradeUtil.contextual): ctx = (1+C)g / (1+Cg),
    // where C = partner grade * (ratio - 1) for the head-stem support, the
    // ratio being 1 + 4 (headSupportCoeff) * relation grade. The stem grade
    // grows with its tail length (StemInter: 1.25 .. 3.5 interlines).
    let il = scale.interline as f64;
    let ctx: Vec<f64> = heads
        .iter()
        .zip(&links)
        .map(|(h, l)| {
            let c = match l {
                Some(l) => {
                    let cy = h.bounds.cy();
                    let tail = (cy - l.top as f64).max(l.bottom as f64 - cy);
                    let stem_grade = 0.4 + 0.5 * ((tail - 1.25 * il) / (2.25 * il)).clamp(0.0, 1.0);
                    stem_grade * 4.0 * 0.8
                }
                None => 0.0,
            };
            (1.0 + c) * h.grade / (1.0 + c * h.grade)
        })
        .collect();
    if let Ok(v) = std::env::var("OMR_PROBE") {
        let v: Vec<i32> = v.split(',').map(|x| x.parse().unwrap()).collect();
        for (i, h) in heads.iter().enumerate() {
            if h.bounds.intersects(&crate::binary::Rect::new(v[0] - 5, v[1] - 5, 10, 10)) {
                eprintln!("link {:?} {} ctx {:.2} link {:?}", h.shape, h.pitch, ctx[i], links[i].as_ref().map(|l| (l.x, l.top, l.bottom)));
            }
        }
    }
    let mut order: Vec<usize> = (0..heads.len()).filter(|&i| (links[i].is_some() || heads[i].shape.stemless()) && ctx[i] >= 0.5)
        .collect();
    order.sort_by(|&a, &b| ctx[b].partial_cmp(&ctx[a]).unwrap());
    let mut kept: Vec<usize> = Vec::new();
    for i in order {
        if kept.iter().any(|&k| overlaps(&heads[i], &heads[k])) {
            continue;
        }
        kept.push(i);
    }
    kept.sort_by_key(|&i| (heads[i].staff, heads[i].bounds.x, heads[i].pitch));
    let mut out_heads = Vec::new();
    let mut stems: Vec<Stem> = Vec::new();
    let tol = scale.max_stem + 1;
    for i in kept {
        let hi = out_heads.len();
        out_heads.push(Head { ctx: ctx[i], ..heads[i].clone() });
        if let Some(l) = &links[i] {
            let existing = stems.iter_mut().find(|s| (s.x - l.x).abs() <= tol && s.top <= l.bottom && l.top <= s.bottom);
            match existing {
                Some(s) => {
                    s.heads.push(hi);
                    s.top = s.top.min(l.top);
                    s.bottom = s.bottom.max(l.bottom);
                }
                None => stems.push(Stem { x: l.x, top: l.top, bottom: l.bottom, up: true, heads: vec![hi], beams: vec![], flags: 0 }),
            }
        }
    }
    for s in &mut stems {
        let hmin = s.heads.iter().map(|&h| out_heads[h].bounds.y).min().unwrap();
        let hmax = s.heads.iter().map(|&h| out_heads[h].bounds.bottom()).max().unwrap();
        s.up = (hmin - s.top) >= (s.bottom - hmax);
    }
    // Heads on the unusual side of the stem (right of an up stem, left of a
    // down stem) are only valid as the second of an interval of a second.
    let mut drop = vec![false; out_heads.len()];
    for s in &mut stems {
        let normal = |h: &Head| if s.up { h.bounds.cx() < s.x as f64 } else { h.bounds.cx() > s.x as f64 };
        let odd: Vec<usize> = s.heads.iter().copied().filter(|&h| !normal(&out_heads[h])).collect();
        for h in odd {
            let ok = s.heads.iter().any(|&o| normal(&out_heads[o]) && (out_heads[o].pitch - out_heads[h].pitch).abs() == 1);
            if !ok && s.heads.len() > 1 {
                drop[h] = true;
            }
        }
        s.heads.retain(|&h| !drop[h]);
    }
    // Minimum tail length after the last head (StemInter.minTailLength = 1.75):
    // heads lying within the tail are not on this stem's head side.
    stems.retain_mut(|s| {
        let min_tail = 1.75 * il;
        for &h in &s.heads {
            let cy = out_heads[h].bounds.cy();
            let tail = if s.up { cy - s.top as f64 } else { s.bottom as f64 - cy };
            if tail < min_tail {
                drop[h] = true;
            }
        }
        s.heads.retain(|&h| !drop[h]);
        !s.heads.is_empty()
    });
    // Re-index heads after dropping.
    let mut map = vec![usize::MAX; out_heads.len()];
    let mut kept_heads = Vec::new();
    for (i, h) in out_heads.into_iter().enumerate() {
        if !drop[i] {
            map[i] = kept_heads.len();
            kept_heads.push(h);
        }
    }
    for s in &mut stems {
        for h in &mut s.heads {
            *h = map[*h];
        }
    }
    (kept_heads, stems)
}

/// Attach beams to stems: a beam is attached when the stem abscissa lies in
/// the beam range and the beam crosses the stem near its free end.
pub fn attach_beams(stems: &mut [Stem], heads: &[Head], beams: &[Beam], scale: &Scale) {
    let il = scale.interline as f64;
    for s in stems.iter_mut() {
        let x = s.x as f64;
        let tol = (scale.max_stem + 2) as f64;
        let hy: Vec<f64> = s.heads.iter().map(|&h| heads[h].bounds.cy()).collect();
        let head_end = if s.up { hy.iter().cloned().fold(f64::MAX, f64::min) } else { hy.iter().cloned().fold(f64::MIN, f64::max) };
        let mut list: Vec<(f64, usize)> = Vec::new();
        for (bi, b) in beams.iter().enumerate() {
            if x < b.x0 - tol || x > b.x1 + tol {
                continue;
            }
            let y = b.y_at(x.clamp(b.x0, b.x1));
            // Beam must be on the stem, between its tip and its heads (with margin).
            let tip = s.tip() as f64;
            let ok = if s.up { y >= tip - 0.5 * il && y <= head_end - 1.0 * il } else { y <= tip + 0.5 * il && y >= head_end + 1.0 * il };
            if ok {
                list.push(((y - tip).abs(), bi));
            }
        }
        list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        s.beams = list.into_iter().map(|l| l.1).collect();
    }
}

/// Remove heads excluded by symbols (accidentals, rests) recognized on their
/// location, then stems left without heads.
pub fn purge_by_symbols(heads: &mut Vec<Head>, stems: &mut Vec<Stem>, symbols: &[crate::symbols::Symbol]) {
    let excluded: Vec<bool> = heads
        .iter()
        .map(|h| {
            symbols.iter().any(|s| {
                let competing = matches!(s.shape, "NATURAL" | "SHARP" | "FLAT" | "DOUBLE_SHARP" | "DOUBLE_FLAT")
                    || s.shape.ends_with("_REST")
                    || s.shape == "HW_REST_set";
                competing
                    && s.grade > h.ctx
                    && s.bounds.intersection(&h.bounds).is_some_and(|i| i.area() as f64 >= 0.4 * h.bounds.area() as f64)
            })
        })
        .collect();
    let mut map = vec![usize::MAX; heads.len()];
    let mut kept = Vec::new();
    for (i, h) in heads.drain(..).enumerate() {
        if !excluded[i] {
            map[i] = kept.len();
            kept.push(h);
        }
    }
    *heads = kept;
    for s in stems.iter_mut() {
        s.heads = s.heads.iter().filter(|&&h| map[h] != usize::MAX).map(|&h| map[h]).collect();
    }
    stems.retain(|s| !s.heads.is_empty());
}
