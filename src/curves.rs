//! Straight-line curves (Audiveris CURVES step: SegmentsBuilder,
//! WedgesBuilder, EndingsBuilder).
//!
//! Straight segments are taken from the residual glyphs: each glyph gives its
//! upper and lower envelope (one line, or the two legs of a hairpin drawn in
//! one stroke). Hairpins pair two segments whose ends are close on one side
//! and apart on the other; the remaining long horizontal segments high above
//! a staff are endings (voltas).

use crate::binary::{Bitmap, Rect, components};
use crate::grid::System;
use crate::scale::Scale;
use crate::staff::Staff;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Wedge {
    pub crescendo: bool,
    /// Left and right abscissae.
    pub x0: f64,
    pub x1: f64,
    /// Closest staff to the closed end.
    pub staff: usize,
    pub grade: f64,
    pub bounds: Rect,
}

struct Segment {
    left: (f64, f64),
    right: (f64, f64),
    /// Distance impact (1 - mean distance / max segment distance).
    dist: f64,
    /// Mean stroke thickness.
    thick: f64,
}

/// Least-squares line through the points: (slope, intercept, mean distance).
fn fit(pts: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = pts.len() as f64;
    let (sx, sy) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
    let (mx, my) = (sx / n, sy / n);
    let (mut sxx, mut sxy) = (0.0, 0.0);
    for p in pts {
        sxx += (p.0 - mx) * (p.0 - mx);
        sxy += (p.0 - mx) * (p.1 - my);
    }
    let a = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    let b = my - a * mx;
    let norm = (1.0 + a * a).sqrt();
    let d = pts.iter().map(|p| (p.1 - a * p.0 - b).abs() / norm).sum::<f64>() / n;
    (a, b, d)
}

fn segments(residual: &Bitmap, staves: &[Staff], scale: &Scale) -> Vec<Segment> {
    let il = scale.interline as f64;
    let max_dist = 0.3 * il;
    let (max_run_dist, min_run_ratio) = (0.2 * il, 0.75);
    let max_slope = 20f64.to_radians().tan();
    let min_len = 2.0 * il;
    // Segment ends must lie outside any staff height.
    let in_staff = |(x, y): (f64, f64)| {
        staves.iter().any(|s| x >= s.left as f64 && x <= s.right as f64 && y >= s.top(x) && y <= s.bottom(x))
    };
    let mut out = Vec::new();
    for c in components(residual, None) {
        let b = c.bounds;
        if (b.w as f64) < min_len || b.h as f64 > 4.0 * il {
            continue;
        }
        let mut cols: Vec<Vec<i32>> = vec![Vec::new(); b.w as usize];
        for &(x, y) in &c.pixels {
            cols[(x - b.x) as usize].push(y);
        }
        let (mut top, mut bot) = (Vec::new(), Vec::new());
        for (i, col) in cols.iter_mut().enumerate() {
            if col.is_empty() {
                continue;
            }
            col.sort();
            let x = (b.x + i as i32) as f64;
            // First and last vertical runs of the column.
            let first_end = col.windows(2).position(|w| w[1] > w[0] + 1).map_or(*col.last().unwrap(), |k| col[k]);
            let last_start = col.windows(2).rposition(|w| w[1] > w[0] + 1).map_or(col[0], |k| col[k + 1]);
            top.push((x, (col[0] + first_end) as f64 / 2.0));
            bot.push((x, (last_start + *col.last().unwrap()) as f64 / 2.0));
        }
        let columns = cols.iter().filter(|c| !c.is_empty()).count().max(1) as f64;
        let mut legs = vec![top];
        let (ta, tb, _) = fit(&legs[0]);
        let (ba, bb, _) = fit(&bot);
        let (x0, x1) = (b.x as f64, b.right() as f64 - 1.0);
        let sep = |x: f64| (ba * x + bb - ta * x - tb).abs();
        if sep(x0).max(sep(x1)) >= 0.5 * il {
            legs.push(bot);
        }
        let thick = c.pixels.len() as f64 / columns / legs.len() as f64;
        for pts in legs {
            let (a, k, d) = fit(&pts);
            if d > max_dist || a.abs() > max_slope {
                continue;
            }
            let (l, r) = (pts[0].0, pts[pts.len() - 1].0);
            // Runs close to the line over most of the width (maxRunDistance,
            // minRunRatio of SegmentsBuilder.retrieveGlyph).
            let close = pts.iter().filter(|p| (p.1 - a * p.0 - k).abs() <= max_run_dist).count();
            if (close as f64) < min_run_ratio * (r - l + 1.0) {
                continue;
            }
            let left = (l, a * l + k);
            let right = (r, a * r + k);
            if in_staff(left) || in_staff(right) {
                continue;
            }
            out.push(Segment { left, right, dist: 1.0 - d / max_dist, thick });
        }
    }
    out
}

/// Hairpins and endings of the sheet.
pub fn build(
    residual: &Bitmap,
    binary: &Bitmap,
    staves: &[Staff],
    systems: &[System],
    scale: &Scale,
) -> (Vec<Wedge>, Vec<Ending>) {
    let mut segs = segments(residual, staves, scale);
    let wedges = wedges(&mut segs, staves, scale);
    let endings = endings(&segs, binary, staves, systems, scale);
    (wedges, endings)
}

fn wedges(segs: &mut Vec<Segment>, staves: &[Staff], scale: &Scale) -> Vec<Wedge> {
    let il = scale.interline as f64;
    let closed_max_dx = (0.2 * il).round();
    let closed_max_dy = (0.5 * il).round();
    let (open_low, open_high) = ((0.5 * il).round(), (1.5 * il).round());
    let (len_low, len_high) = ((2.0 * il).round(), (3.0 * il).round());
    let open_max_bias = 20f64.to_radians().tan();
    let mut wedges = Vec::new();
    for rev in [true, false] {
        // Closed end: left for a crescendo (rev), right for a diminuendo.
        let end = |s: &Segment, closed: bool| if closed == rev { s.left } else { s.right };
        segs.sort_by(|a, b| end(a, true).0.partial_cmp(&end(b, true).0).unwrap());
        let mut i = 0;
        while i < segs.len() {
            let c1 = end(&segs[i], true);
            let ax = if rev { c1.0 } else { c1.0 - closed_max_dx + 1.0 };
            let half = (1.0 + closed_max_dy) / 2.0;
            let mut found = None;
            for j in i + 1..segs.len() {
                let c2 = end(&segs[j], true);
                if c2.0 >= ax && c2.0 < ax + closed_max_dx && (c2.1 - c1.1).abs() <= half {
                    let (s1, s2) = (&segs[i], &segs[j]);
                    let closed_dy = (c1.1 - c2.1).abs();
                    let (o1, o2) = (end(s1, false), end(s2, false));
                    let open_dy = (o1.1 - o2.1).abs();
                    let inv_slope = if open_dy > 0.0 { (o1.0 - o2.0).abs() / open_dy } else { f64::INFINITY };
                    let width = (s1.right.0 - s1.left.0).min(s2.right.0 - s2.left.0) + 1.0;
                    if closed_dy > closed_max_dy || open_dy < open_low || inv_slope > open_max_bias || width < len_low {
                        continue;
                    }
                    let impacts = [
                        s1.dist,
                        s2.dist,
                        1.0 - closed_dy / closed_max_dy,
                        (open_dy - open_low) / (open_high - open_low),
                        1.0 - inv_slope / open_max_bias,
                        (width - len_low) / (len_high - len_low),
                    ];
                    let grade = if impacts.iter().any(|&v| v <= 0.0) {
                        0.0
                    } else {
                        0.8 * impacts.iter().product::<f64>().powf(1.0 / 6.0)
                    };
                    if grade >= 0.08 {
                        found = Some((j, grade));
                        break;
                    }
                } else if c2.0 > ax + closed_max_dx {
                    break;
                }
            }
            if let Some((j, grade)) = found {
                let (s1, s2) = (&segs[i], &segs[j]);
                let closed = end(s1, true);
                let staff = (0..staves.len())
                    .min_by(|&a, &b| staff_dist(&staves[a], closed).partial_cmp(&staff_dist(&staves[b], closed)).unwrap())
                    .unwrap();
                wedges.push(Wedge {
                    crescendo: rev,
                    x0: s1.left.0.min(s2.left.0),
                    x1: s1.right.0.max(s2.right.0),
                    staff,
                    grade,
                    bounds: {
                        let ys = [s1.left.1, s1.right.1, s2.left.1, s2.right.1];
                        let (y0, y1) = (ys.iter().cloned().fold(f64::MAX, f64::min), ys.iter().cloned().fold(f64::MIN, f64::max));
                        let x0 = s1.left.0.min(s2.left.0);
                        Rect::new(x0 as i32, y0 as i32, (s1.right.0.max(s2.right.0) - x0) as i32 + 1, (y1 - y0) as i32 + 1)
                    },
                });
                segs.remove(j);
                segs.remove(i);
            } else {
                i += 1;
            }
        }
    }
    wedges
}

fn staff_dist(s: &Staff, (x, y): (f64, f64)) -> f64 {
    let dx = (s.left as f64 - x).max(x - s.right as f64).max(0.0);
    let dy = (s.top(x) - y).max(y - s.bottom(x)).max(0.0);
    dx.hypot(dy)
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Ending {
    pub x0: f64,
    pub x1: f64,
    pub y: f64,
    /// Staff below.
    pub staff: usize,
    /// Leg (vertical hook) on the right end: stop, else discontinue.
    pub right_leg: bool,
    pub grade: f64,
}

/// Endings (Audiveris EndingsBuilder / LeggedIntersBuilder): a thin
/// horizontal segment of at least 6 il, at least 2.5 il above the staff
/// below, covering most of its measure, with a vertical leg below its left
/// end (and maybe its right end).
fn endings(segs: &[Segment], binary: &Bitmap, staves: &[Staff], systems: &[System], scale: &Scale) -> Vec<Ending> {
    let il = scale.interline as f64;
    let (len_low, len_high) = (6.0 * il, 10.0 * il);
    let max_slope = 0.02;
    let max_thick = scale.max_stem as f64 * 1.4 + 1.0;
    let mut out = Vec::new();
    for s in segs {
        let length = s.right.0 - s.left.0;
        let slope = ((s.right.1 - s.left.1) / length.max(1.0)).abs();
        if length < len_low || slope > max_slope || s.thick > max_thick {
            continue;
        }
        let (cx, cy) = ((s.left.0 + s.right.0) / 2.0, (s.left.1 + s.right.1) / 2.0);
        // Staff at or below.
        let Some(si) = (0..staves.len())
            .filter(|&i| staves[i].top(cx) > cy && cx >= staves[i].left as f64 && cx <= staves[i].right as f64)
            .min_by(|&a, &b| staves[a].top(cx).partial_cmp(&staves[b].top(cx)).unwrap())
        else {
            continue;
        };
        if staves[si].top(cx) - cy < 2.5 * il {
            continue;
        }
        // Measure under the center.
        let Some(sys) = systems.iter().find(|y| y.staves.contains(&si)) else { continue };
        let Some(mi) = sys.boundaries.windows(2).position(|w| cx > w[0].x1 as f64 && cx < w[1].x0 as f64) else {
            continue;
        };
        let width = (sys.boundaries[mi + 1].x0 - sys.boundaries[mi].x1) as f64;
        let ratio = if mi == 0 { 0.6 } else { 0.8 };
        if length < ratio * width {
            continue;
        }
        let impacts = [s.dist, 1.0 - slope / max_slope, (length - len_low) / (len_high - len_low)];
        let grade = 0.8 * impacts.iter().map(|v| v.max(1e-6)).product::<f64>().powf(1.0 / 3.0);
        if grade < 0.08 {
            continue;
        }
        let leg = |(x, y): (f64, f64)| {
            let top = staves[si].top(x);
            ((x - 0.5 * il) as i32..=(x + 0.5 * il) as i32).any(|lx| {
                (y as i32..=(y + 0.5 * il) as i32).any(|ly| {
                    let (a, e) = binary.vrun_at(lx, ly);
                    a as f64 <= y + 0.5 * il && (e - a) as f64 >= il && (e as f64) < top
                })
            })
        };
        // A left leg is required (improvement: lyric extenders and lone
        // hairpin legs have none).
        if !leg(s.left) {
            continue;
        }
        out.push(Ending { x0: s.left.0, x1: s.right.0, y: cy, staff: si, right_leg: leg(s.right), grade });
    }
    out.sort_by(|a, b| (a.staff, a.x0 as i64).cmp(&(b.staff, b.x0 as i64)));
    out
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct OctaveShift {
    pub staff: usize,
    pub x0: f64,
    pub x1: f64,
    /// 8, 15 or 22.
    pub size: u32,
    /// Sign above the staff (notes sound higher).
    pub above: bool,
}

/// Octave shifts (Audiveris OctaveShiftInter): the 8va / 15ma / 22ma sign
/// outside the staff height, linked to the chords of its staff under its
/// line. Improvement: the dashed line is followed in the residual image to
/// find the end, and is required (Audiveris assumes a default length and has
/// OCR to discard text digits).
pub fn octave_shifts(residual: &Bitmap, symbols: &[crate::symbols::Symbol], staves: &[Staff], scale: &Scale) -> Vec<OctaveShift> {
    let il = scale.interline as f64;
    let mut out = Vec::new();
    for s in symbols {
        let size = match s.shape {
            "OTTAVA" => 8,
            "QUINDICESIMA" => 15,
            "VENTIDUESIMA" => 22,
            _ => continue,
        };
        let st = &staves[s.staff];
        let (cx, cy) = (s.bounds.cx(), s.bounds.cy());
        let above = cy < st.top(cx);
        if !above && cy <= st.bottom(cx) {
            continue;
        }
        // Dashes: thin short horizontal glyphs in a band at the sign's height.
        let band = Rect::new(s.bounds.right(), s.bounds.y, (40.0 * il) as i32, s.bounds.h);
        let mut dashes: Vec<Rect> = components(residual, Some(band))
            .into_iter()
            .map(|c| c.bounds)
            .filter(|b| b.h as f64 <= 0.3 * il && b.w as f64 <= 1.5 * il && b.w >= b.h)
            .collect();
        dashes.sort_by_key(|b| b.x);
        let mut end = s.bounds.right() as f64;
        let mut count = 0;
        for d in dashes {
            if d.x as f64 - end > 1.5 * il {
                break;
            }
            end = d.right() as f64;
            count += 1;
        }
        if count < 2 {
            continue;
        }
        out.push(OctaveShift { staff: s.staff, x0: s.bounds.x as f64, x1: end, size, above });
    }
    out
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Slur {
    /// Left and right end points.
    pub left: (f64, f64),
    pub right: (f64, f64),
    /// Arc bulging upwards (heads lie below its ends).
    pub above: bool,
    pub grade: f64,
    pub bounds: Rect,
    /// Center line.
    #[serde(skip)]
    pub pts: Vec<(f64, f64)>,
}

/// Circle through the points (algebraic least squares): (cx, cy, r, mean distance).
fn fit_circle(pts: &[(f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    let n = pts.len() as f64;
    let (mx, my) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0 / n, a.1 + p.1 / n));
    let (mut suu, mut svv, mut suv, mut suuu, mut svvv, mut suvv, mut svuu) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(x, y) in pts {
        let (u, v) = (x - mx, y - my);
        suu += u * u;
        svv += v * v;
        suv += u * v;
        suuu += u * u * u;
        svvv += v * v * v;
        suvv += u * v * v;
        svuu += v * u * u;
    }
    let det = suu * svv - suv * suv;
    if det.abs() < 1e-9 {
        return None;
    }
    let (b1, b2) = (0.5 * (suuu + suvv), 0.5 * (svvv + svuu));
    let uc = (b1 * svv - b2 * suv) / det;
    let vc = (suu * b2 - suv * b1) / det;
    let r = (uc * uc + vc * vc + (suu + svv) / n).sqrt();
    let (cx, cy) = (uc + mx, vc + my);
    let d = pts.iter().map(|p| ((p.0 - cx).hypot(p.1 - cy) - r).abs()).sum::<f64>() / n;
    Some((cx, cy, r, d))
}

/// Mean distance of the points to their side models (Audiveris
/// SlursBuilder): circles fitted on the first and last 6 interlines.
fn side_distance(pts: &[(f64, f64)], il: f64) -> Option<f64> {
    let len = 6.0 * il;
    let (x0, x1) = (pts[0].0, pts[pts.len() - 1].0);
    let left: Vec<(f64, f64)> = pts.iter().copied().filter(|p| p.0 - x0 <= len).collect();
    let right: Vec<(f64, f64)> = pts.iter().copied().filter(|p| x1 - p.0 <= len).collect();
    if left.len() < 3 || right.len() < 3 {
        return None;
    }
    Some((fit_circle(&left)?.3 + fit_circle(&right)?.3) / 2.0)
}

/// A thin piece of curve: its center points, one per column with a single run.
struct Piece {
    pts: Vec<(f64, f64)>,
    bounds: Rect,
}

/// Slurs and ties (Audiveris CURVES step, before SYMBOLS): thin curved
/// strokes of the residual image (chains of thin vertical runs, joined across
/// small gaps) whose center line fits a circle, graded as in
/// SlursBuilder.computeImpacts. Only slurs with both ends near a note (heads
/// and stem), or one end near a note and the other near a staff end, are
/// kept. Their pixels are returned as a mask, so that glyphs lying on them
/// are not read as symbols.
pub fn slurs(
    residual: &Bitmap,
    heads: &[crate::heads::Head],
    stems: &[crate::stems::Stem],
    staves: &[Staff],
    symbols: &[crate::symbols::Symbol],
    scale: &Scale,
) -> (Vec<Slur>, Bitmap) {
    let il = scale.interline as f64;
    // Notes: a stem with its heads, or a stemless head.
    let mut notes: Vec<Rect> = stems
        .iter()
        .filter(|st| !st.heads.is_empty())
        .map(|st| {
            st.heads
                .iter()
                .map(|&h| heads[h].bounds)
                .fold(Rect::new(st.x, st.top, 1, st.bottom - st.top + 1), |a, b| a.union(&b))
        })
        .collect();
    let stemmed: Vec<usize> = stems.iter().flat_map(|st| st.heads.iter().copied()).collect();
    notes.extend((0..heads.len()).filter(|h| !stemmed.contains(h)).map(|h| heads[h].bounds));
    let near = |end: (f64, f64), left: bool, above: bool| -> Option<usize> {
        let (x0, x1) = if left { (end.0 - 2.0 * il, end.0 + 0.5 * il) } else { (end.0 - 0.5 * il, end.0 + 2.0 * il) };
        let (y0, y1) = if above { (end.1 - 0.5 * il, end.1 + 3.0 * il) } else { (end.1 - 3.0 * il, end.1 + 0.5 * il) };
        notes.iter().position(|b| {
            (b.right() as f64) >= x0 && (b.x as f64) <= x1 && (b.bottom() as f64) >= y0 && (b.y as f64) <= y1
        })
    };
    let staff_end = |p: (f64, f64), left: bool| {
        staves.iter().any(|st| {
            p.1 >= st.top(p.0) - 4.0 * il
                && p.1 <= st.bottom(p.0) + 4.0 * il
                && if left {
                    p.0 - st.header_stop.max(st.left) as f64 <= 6.0 * il
                } else {
                    st.right as f64 - p.0 <= 6.0 * il
                }
        })
    };
    let max_thick = il.max(3.0) as i32;
    let mut mask = Bitmap::new(residual.width, residual.height);
    let mut out = Vec::new();
    // An end touching a stem is a flag, not a slur.
    let on_stem = |p: (f64, f64)| {
        stems.iter().any(|st| {
            (p.0 - st.x as f64).abs() <= 2.0 && p.1 >= st.top as f64 && p.1 <= st.bottom as f64
        })
    };
    for s in slur_candidates(residual, symbols, scale) {
        if on_stem(s.left) || on_stem(s.right) {
            continue;
        }
        let (l, r) = (near(s.left, true, s.above), near(s.right, false, s.above));
        let keep = match (l, r) {
            (Some(a), Some(b)) => a != b,
            (Some(_), None) => staff_end(s.right, false),
            (None, Some(_)) => staff_end(s.left, true),
            _ => false,
        };
        if !keep {
            continue;
        }
        for &(x, y) in &s.pts {
            let (a, b) = residual.vrun_at(x as i32, y.round() as i32);
            if b - a < max_thick {
                for yy in a..=b {
                    mask.set(x as i32, yy, true);
                }
            }
        }
        out.push(s);
    }
    (out, mask)
}

fn slur_candidates(residual: &Bitmap, symbols: &[crate::symbols::Symbol], scale: &Scale) -> Vec<Slur> {
    let il = scale.interline as f64;
    let max_thick = il.max(3.0);
    let mut pieces: Vec<Piece> = Vec::new();
    // Thin vertical runs of every column, chained column to column while the
    // chain does not branch (an approximation of skeleton arcs between
    // junctions, so that a slur touching another glyph is still found).
    let (w, h) = (residual.width as i32, residual.height as i32);
    let mut runs: Vec<Vec<(i32, i32)>> = vec![Vec::new(); w as usize];
    for x in 0..w {
        let mut y = 0;
        while y < h {
            if residual.get(x, y) {
                let a = y;
                while y < h && residual.get(x, y) {
                    y += 1;
                }
                if ((y - a) as f64) <= max_thick {
                    runs[x as usize].push((a, y - 1));
                }
            }
            y += 1;
        }
    }
    let touch = |a: (i32, i32), b: (i32, i32)| a.0 <= b.1 + 1 && b.0 <= a.1 + 1;
    let right_of = |x: usize, r: (i32, i32)| -> Vec<usize> {
        if x + 1 >= runs.len() {
            return Vec::new();
        }
        (0..runs[x + 1].len()).filter(|&j| touch(r, runs[x + 1][j])).collect()
    };
    let left_count = |x: usize, r: (i32, i32)| -> usize {
        if x == 0 {
            return 0;
        }
        runs[x - 1].iter().filter(|&&q| touch(r, q)).count()
    };
    let mut seen: Vec<Vec<bool>> = runs.iter().map(|c| vec![false; c.len()]).collect();
    for x in 0..runs.len() {
        for i in 0..runs[x].len() {
            if seen[x][i] {
                continue;
            }
            seen[x][i] = true;
            let mut pts = Vec::new();
            let mut thick = 0.0;
            let (mut cx, mut ci) = (x, i);
            loop {
                let r = runs[cx][ci];
                thick += (r.1 - r.0 + 1) as f64;
                pts.push((cx as f64, (r.0 + r.1) as f64 / 2.0));
                let next = right_of(cx, r);
                if next.len() != 1 {
                    break;
                }
                let n = runs[cx + 1][next[0]];
                if left_count(cx + 1, n) != 1 || seen[cx + 1][next[0]] {
                    break;
                }
                cx += 1;
                ci = next[0];
                seen[cx][ci] = true;
            }
            // Short pieces must be thin (cut pieces of a curve, not dots).
            if pts.len() < 3 || ((pts.len() as f64) < 0.5 * il && thick / pts.len() as f64 > 0.25 * il) {
                continue;
            }
            let (x0, x1) = (pts[0].0 as i32, pts[pts.len() - 1].0 as i32);
            let (y0, y1) = pts.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
            let bounds = Rect::new(x0, y0 as i32 - 1, x1 - x0 + 1, (y1 - y0) as i32 + 3);
            // Not part of a recognized symbol (but dots and rest bars may be
            // pieces of a slur cut by staff lines).
            if symbols.iter().any(|s| {
                s.shape != "DOT_set"
                    && s.shape != "HW_REST_set"
                    && s.bounds.intersection(&bounds).is_some_and(|i| i.area() as f64 >= 0.5 * bounds.area() as f64)
            }) {
                continue;
            }
            pieces.push(Piece { pts, bounds });
        }
    }
    pieces.sort_by_key(|p| p.bounds.x);
    // Join pieces end to end when the union still fits a circle.
    let mut used = vec![false; pieces.len()];
    let mut out = Vec::new();
    for i in 0..pieces.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        let mut pts = pieces[i].pts.clone();
        let mut bounds = pieces[i].bounds;
        // Last state passing the final check (extensions are accepted with
        // the looser intermediate distance, maxArcsDistance 0.15).
        let mut best = grade_slur(&pts, il).map(|s| Slur { bounds, ..s });
        loop {
            let end = *pts.last().unwrap();
            let next = (0..pieces.len()).filter(|&j| !used[j]).find(|&j| {
                let p0 = pieces[j].pts[0];
                p0.0 - end.0 >= -1.0
                    && p0.0 - end.0 <= 2.0 * il
                    && (p0.1 - end.1).abs() <= 0.5 * il
                    && {
                        let mut u = pts.clone();
                        u.extend(&pieces[j].pts);
                        side_distance(&u, il).is_some_and(|d| d <= 0.15 * il)
                    }
            });
            let Some(j) = next else { break };
            used[j] = true;
            pts.extend(&pieces[j].pts);
            bounds = bounds.union(&pieces[j].bounds);
            if let Some(s) = grade_slur(&pts, il) {
                best = Some(Slur { bounds, ..s });
            }
        }
        if let Some(s) = best {
            out.push(s);
        }
    }
    out
}

fn grade_slur(pts: &[(f64, f64)], il: f64) -> Option<Slur> {
    let (cx, cy, _r, _) = fit_circle(pts)?;
    let dist = side_distance(pts, il)?;
    let max_dist = 0.1 * il;
    if dist > max_dist {
        return None;
    }
    let (p0, p1, p2) = (pts[0], pts[pts.len() / 2], pts[pts.len() - 1]);
    let ang = |p: (f64, f64)| (p.1 - cy).atan2(p.0 - cx);
    let (a0, a1, a2) = (ang(p0), ang(p1), ang(p2));
    // Arc from p0 to p2 through p1.
    let norm = |a: f64| a.rem_euclid(2.0 * std::f64::consts::PI);
    let (d01, d02) = (norm(a1 - a0), norm(a2 - a0));
    let arc = if d01 <= d02 { d02 } else { 2.0 * std::f64::consts::PI - d02 }.to_degrees();
    if arc > 190.0 {
        return None;
    }
    let mut mid = a1.rem_euclid(std::f64::consts::PI);
    if mid < 0.0 {
        mid += std::f64::consts::PI;
    }
    let from_vertical = mid.min(std::f64::consts::PI - mid).to_degrees();
    if from_vertical < 10.0 {
        return None;
    }
    let width = (p2.0 - p0.0).abs();
    if width < 0.7 * il {
        return None;
    }
    let (dx, dy) = (p2.0 - p0.0, p2.1 - p0.1);
    let height = ((p1.0 - p0.0) * dy - (p1.1 - p0.1) * dx).abs() / dx.hypot(dy);
    if height < 0.07 * il {
        return None;
    }
    let impacts = [
        (1.0 - dist / max_dist, 3.0),
        ((190.0 - arc) / 20.0, 1.0),
        ((width - 0.7 * il) / (0.8 * il), 1.0),
        ((height - 0.07 * il) / (0.93 * il), 1.0),
        ((from_vertical - 10.0) / 15.0, 1.0),
    ];
    let grade = if impacts.iter().any(|(v, _)| *v <= 0.0) {
        0.0
    } else {
        0.8 * impacts.iter().map(|(v, w)| v.powf(*w)).product::<f64>().powf(1.0 / 7.0)
    };
    if grade < 0.08 {
        return None;
    }
    Some(Slur { left: p0, right: p2, above: p1.1 < cy, grade, bounds: Rect::default(), pts: pts.to_vec() })
}
