//! Hairpins (Audiveris SegmentsBuilder and WedgesBuilder).
//!
//! Straight segments are taken from the residual glyphs: each glyph gives its
//! upper and lower envelope (one line, or the two legs of a hairpin drawn in
//! one stroke). Segments are pairs-matched as in WedgesBuilder: ends close on
//! one side, apart on the other.

use crate::binary::{Bitmap, Rect, components};
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
        let mut legs = vec![top];
        let (ta, tb, _) = fit(&legs[0]);
        let (ba, bb, _) = fit(&bot);
        let (x0, x1) = (b.x as f64, b.right() as f64 - 1.0);
        let sep = |x: f64| (ba * x + bb - ta * x - tb).abs();
        if sep(x0).max(sep(x1)) >= 0.5 * il {
            legs.push(bot);
        }
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
            out.push(Segment { left, right, dist: 1.0 - d / max_dist });
        }
    }
    out
}

pub fn detect(residual: &Bitmap, staves: &[Staff], scale: &Scale) -> Vec<Wedge> {
    let il = scale.interline as f64;
    let closed_max_dx = (0.2 * il).round();
    let closed_max_dy = (0.5 * il).round();
    let (open_low, open_high) = ((0.5 * il).round(), (1.5 * il).round());
    let (len_low, len_high) = ((2.0 * il).round(), (3.0 * il).round());
    let open_max_bias = 20f64.to_radians().tan();
    let mut segs = segments(residual, staves, scale);
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
