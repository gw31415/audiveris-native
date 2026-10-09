//! Staff line retrieval (Audiveris LinesRetriever + ClustersRetriever).
//!
//! Thin vertical runs (thickness <= max line thickness) are sampled column by
//! column. At every sampled column we look for "combs": five such runs with
//! regular spacing close to the interline. Consecutive combs are chained into
//! staves; each staff line is then traced to its true horizontal ends.

use crate::binary::Bitmap;
use crate::scale::Scale;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Line {
    /// y = c0 + c1 * t + c2 * t^2, with t = (x - x0) / scale
    pub c: [f64; 3],
    pub x0: f64,
    pub xs: f64,
}

impl Line {
    pub fn y_at(&self, x: f64) -> f64 {
        let t = (x - self.x0) / self.xs;
        self.c[0] + self.c[1] * t + self.c[2] * t * t
    }

    /// Robust least-squares quadratic fit of line samples (outliers rejected).
    pub fn fit(pts: &[(f64, f64)]) -> Line {
        let x0 = pts.iter().map(|p| p.0).sum::<f64>() / pts.len() as f64;
        let xs = pts.iter().map(|p| (p.0 - x0).abs()).fold(1.0, f64::max);
        let mut used: Vec<(f64, f64)> = pts.to_vec();
        let mut line = Line { c: [used[0].1, 0.0, 0.0], x0, xs };
        // Robust start: linear fit with progressively tighter outlier rejection.
        for tol in [8.0, 4.0, 2.0, 1.5] {
            line.c = lsq(&used, x0, xs, if used.len() >= 2 { 1 } else { 0 });
            let keep: Vec<(f64, f64)> = pts.iter().copied().filter(|p| (p.1 - line.y_at(p.0)).abs() <= tol).collect();
            if keep.len() >= 2 {
                used = keep;
            }
        }
        // Mild curvature allowed when well supported.
        if used.len() >= 20 {
            let c = lsq(&used, x0, xs, 2);
            if c[2].abs() <= 3.0 {
                line.c = c;
            }
        }
        line
    }
}

fn lsq(pts: &[(f64, f64)], x0: f64, xs: f64, deg: usize) -> [f64; 3] {
    let n = deg + 1;
    let mut a = [[0.0f64; 4]; 3];
    for &(x, y) in pts {
        let t = (x - x0) / xs;
        let pw = [1.0, t, t * t];
        for i in 0..n {
            for j in 0..n {
                a[i][j] += pw[i] * pw[j];
            }
            a[i][3] += pw[i] * y;
        }
    }
    // Gaussian elimination.
    for i in 0..n {
        let p = (i..n)
            .max_by(|&r, &q| a[r][i].abs().partial_cmp(&a[q][i].abs()).unwrap())
            .unwrap();
        a.swap(i, p);
        if a[i][i].abs() < 1e-12 {
            continue;
        }
        for r in 0..n {
            if r != i {
                let f = a[r][i] / a[i][i];
                for c in i..4 {
                    a[r][c] -= f * a[i][c];
                }
            }
        }
    }
    let mut c = [0.0; 3];
    for i in 0..n {
        if a[i][i].abs() > 1e-12 {
            c[i] = a[i][3] / a[i][i];
        }
    }
    c
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Staff {
    pub id: usize,
    pub lines: Vec<Line>,
    pub left: i32,
    pub right: i32,
    /// Abscissa where the header (clef/key/time) ends; set later.
    pub header_stop: i32,
}

impl Staff {
    pub fn top(&self, x: f64) -> f64 {
        self.lines[0].y_at(x)
    }
    pub fn bottom(&self, x: f64) -> f64 {
        self.lines[4].y_at(x)
    }
    /// Local interline at abscissa x.
    pub fn interline_at(&self, x: f64) -> f64 {
        (self.bottom(x) - self.top(x)) / 4.0
    }
    /// Ordinate of a pitch position (Audiveris convention: -4 top line, 0 middle, +4 bottom line).
    pub fn pitch_y(&self, pitch: f64, x: f64) -> f64 {
        let p = pitch + 4.0; // 0..8 inside
        if (0.0..=8.0).contains(&p) {
            let i = ((p / 2.0).floor() as usize).min(3);
            let f = (p - 2.0 * i as f64) / 2.0;
            let (a, b) = (self.lines[i].y_at(x), self.lines[i + 1].y_at(x));
            a + (b - a) * f
        } else if p < 0.0 {
            self.top(x) + p * self.interline_at(x) / 2.0
        } else {
            self.bottom(x) + (p - 8.0) * self.interline_at(x) / 2.0
        }
    }
    /// Continuous pitch position of ordinate y.
    pub fn pitch_of(&self, y: f64, x: f64) -> f64 {
        let ys: Vec<f64> = self.lines.iter().map(|l| l.y_at(x)).collect();
        if y < ys[0] {
            return -4.0 - 2.0 * (ys[0] - y) / (ys[1] - ys[0]);
        }
        for i in 0..4 {
            if y <= ys[i + 1] {
                return -4.0 + 2.0 * i as f64 + 2.0 * (y - ys[i]) / (ys[i + 1] - ys[i]);
            }
        }
        4.0 + 2.0 * (y - ys[4]) / (ys[4] - ys[3])
    }
    pub fn mid_x(&self) -> f64 {
        (self.left + self.right) as f64 / 2.0
    }
}

struct Comb {
    x: i32,
    ys: [f64; 5],
}

/// Thin black runs centers at column x.
fn thin_runs(img: &Bitmap, x: i32, max_line: i32) -> Vec<f64> {
    let mut out = Vec::new();
    let mut y = 0;
    let h = img.height as i32;
    while y < h {
        if img.get(x, y) {
            let s = y;
            while y < h && img.get(x, y) {
                y += 1;
            }
            if y - s <= max_line {
                out.push((s + y - 1) as f64 / 2.0);
            }
        } else {
            y += 1;
        }
    }
    out
}

fn combs_at(ys: &[f64], il: f64) -> Vec<[f64; 5]> {
    let mut out = Vec::new();
    let (lo, hi) = (0.8 * il, 1.2 * il);
    for i in 0..ys.len() {
        // Greedy chaining: at each step pick the run whose gap is closest to the expected one.
        let mut chain = vec![ys[i]];
        let mut last = ys[i];
        let mut step = il;
        for _ in 0..4 {
            let best = ys
                .iter()
                .filter(|&&y| y - last >= lo && y - last <= hi)
                .min_by(|a, b| {
                    ((*a - last) - step)
                        .abs()
                        .partial_cmp(&((*b - last) - step).abs())
                        .unwrap()
                });
            match best {
                Some(&y) => {
                    if chain.len() > 1 {
                        step = (y - chain[0]) / chain.len() as f64;
                    } else {
                        step = y - last;
                    }
                    chain.push(y);
                    last = y;
                }
                None => break,
            }
        }
        if chain.len() == 5 {
            let gaps: Vec<f64> = chain.windows(2).map(|w| w[1] - w[0]).collect();
            let mean = gaps.iter().sum::<f64>() / 4.0;
            if gaps.iter().all(|g| (g - mean).abs() <= 0.12 * il + 1.0) {
                out.push([chain[0], chain[1], chain[2], chain[3], chain[4]]);
            }
        }
    }
    out
}

pub fn retrieve(img: &Bitmap, scale: &Scale) -> Vec<Staff> {
    let il = scale.interline as f64;
    let step = (scale.interline / 3).max(2);
    let max_line = scale.max_line + 1;
    // 1. Combs at sampled columns.
    let mut combs: Vec<Comb> = Vec::new();
    let mut x = 0;
    while x < img.width as i32 {
        let ys = thin_runs(img, x, max_line);
        for c in combs_at(&ys, il) {
            combs.push(Comb { x, ys: c });
        }
        x += step;
    }
    // 2. Chain combs into staff candidates.
    struct Chain {
        combs: Vec<usize>,
    }
    let mut chains: Vec<Chain> = Vec::new();
    let max_gap = (6.0 * il) as i32;
    for (ci, c) in combs.iter().enumerate() {
        let mut best: Option<(usize, f64)> = None;
        for (k, ch) in chains.iter().enumerate() {
            let last = &combs[*ch.combs.last().unwrap()];
            if last.x == c.x || c.x - last.x > max_gap {
                continue;
            }
            let d = (0..5).map(|i| (last.ys[i] - c.ys[i]).abs()).sum::<f64>() / 5.0;
            if d <= 0.3 * il && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((k, d));
            }
        }
        match best {
            Some((k, _)) => chains[k].combs.push(ci),
            None => chains.push(Chain { combs: vec![ci] }),
        }
    }
    // 2b. Merge chains that continue each other (same lines after a wide gap).
    chains.sort_by_key(|c| combs[c.combs[0]].x);
    let mut merged: Vec<Chain> = Vec::new();
    for ch in chains {
        let first = &combs[ch.combs[0]];
        let target = merged.iter_mut().find(|m| {
            let last = &combs[*m.combs.last().unwrap()];
            if last.x >= first.x {
                return false;
            }
            let fit = Line::fit(
                &m.combs
                    .iter()
                    .map(|&k| (combs[k].x as f64, combs[k].ys[2]))
                    .collect::<Vec<_>>(),
            );
            let dy = fit.y_at(first.x as f64) - first.ys[2];
            let spacing = (0..5)
                .map(|i| (last.ys[i] - last.ys[0]) - (first.ys[i] - first.ys[0]))
                .map(f64::abs)
                .fold(0.0, f64::max);
            dy.abs() <= 0.3 * il && spacing <= 0.25 * il
        });
        match target {
            Some(m) => m.combs.extend(ch.combs),
            None => merged.push(ch),
        }
    }
    let chains = merged;
    // 3. Keep significant chains, trace lines.
    let mut staves = Vec::new();
    for ch in chains {
        let first = &combs[ch.combs[0]];
        let last = &combs[*ch.combs.last().unwrap()];
        if ch.combs.len() < 6 || ((last.x - first.x) as f64) < 6.0 * il {
            continue;
        }
        let mut lines = Vec::new();
        let mut lefts = Vec::new();
        let mut rights = Vec::new();
        // The five lines share one smooth shape: fit it on all samples
        // shifted by each line's median offset to the middle line.
        let offsets: Vec<f64> = (0..5)
            .map(|i| {
                let mut d: Vec<f64> = ch.combs.iter().map(|&k| combs[k].ys[i] - combs[k].ys[2]).collect();
                d.sort_by(|a, b| a.partial_cmp(b).unwrap());
                d[d.len() / 2]
            })
            .collect();
        let pts: Vec<(f64, f64)> = ch
            .combs
            .iter()
            .flat_map(|&k| (0..5).map(move |i| (k, i)))
            .map(|(k, i)| (combs[k].x as f64, combs[k].ys[i] - offsets[i]))
            .collect();
        let mid = Line::fit(&pts);
        for &o in &offsets {
            let mut line = mid.clone();
            line.c[0] += o;
            lefts.push(trace_end(img, &line, first.x, -1, scale));
            rights.push(trace_end(img, &line, last.x, 1, scale));
            lines.push(line);
        }
        lefts.sort();
        rights.sort();
        let (left, right) = (lefts[2], rights[2]);
        staves.push(Staff {
            id: 0,
            lines,
            left,
            right,
            header_stop: left,
        });
    }
    // Drop staves overlapping a better (longer) one.
    staves.sort_by(|a, b| (b.right - b.left).cmp(&(a.right - a.left)));
    let mut kept: Vec<Staff> = Vec::new();
    for s in staves {
        let overlaps = kept.iter().any(|k| {
            let x0 = s.left.max(k.left);
            let x1 = s.right.min(k.right);
            if x1 <= x0 {
                return false;
            }
            let xm = (x0 + x1) as f64 / 2.0;
            s.top(xm) < k.bottom(xm) + il && k.top(xm) < s.bottom(xm) + il
        });
        if !overlaps {
            kept.push(s);
        }
    }
    kept.sort_by(|a, b| a.top(a.mid_x()).partial_cmp(&b.top(b.mid_x())).unwrap());
    for (i, s) in kept.iter_mut().enumerate() {
        s.id = i + 1;
    }
    kept
}

/// Follow a line from abscissa x0 in direction dir while black pixels remain near it.
fn trace_end(img: &Bitmap, line: &Line, x0: i32, dir: i32, scale: &Scale) -> i32 {
    let tol = (scale.line / 2 + 1).max(1);
    let max_miss = 2;
    let mut x = x0;
    let mut last_ok = x0;
    let mut misses = 0;
    loop {
        x += dir;
        if x < 0 || x >= img.width as i32 {
            break;
        }
        let y = line.y_at(x as f64);
        let yc = y.round() as i32;
        if (-tol..=tol).any(|d| img.get(x, yc + d)) {
            last_ok = x;
            misses = 0;
        } else {
            misses += 1;
            if misses > max_miss {
                break;
            }
        }
    }
    last_ok
}

/// Remove staff line pixels, keeping pixels of crossing objects (vertical
/// runs longer than a line thickness). Returns the "no-staff" image and the
/// mask of removed pixels.
pub fn remove_lines(img: &Bitmap, staves: &[Staff], scale: &Scale) -> (Bitmap, Bitmap) {
    let mut out = img.clone();
    let mut removed = Bitmap::new(img.width, img.height);
    let max = scale.max_line + 1;
    for s in staves {
        for line in &s.lines {
            for x in (s.left - 2)..=(s.right + 2) {
                let yc = line.y_at(x as f64).round() as i32;
                let mut done = std::collections::HashSet::new();
                for dy in -max..=max {
                    let y = yc + dy;
                    if !img.get(x, y) {
                        continue;
                    }
                    let (a, b) = img.vrun_at(x, y);
                    if !done.insert(a) {
                        continue;
                    }
                    // run must overlap the line core
                    if b < yc - scale.line || a > yc + scale.line {
                        continue;
                    }
                    if b - a + 1 <= max {
                        for yy in a..=b {
                            out.set(x, yy, false);
                            removed.set(x, yy, true);
                        }
                    }
                }
            }
        }
    }
    (out, removed)
}
