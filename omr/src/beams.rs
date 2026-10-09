//! Beams (Audiveris BEAMS step, simplified) and ledgers (LEDGERS step).
//!
//! Beam pixels are those whose vertical black run has a beam-like thickness.
//! Gaps left by crossing stems are closed horizontally, then each connected
//! component is fitted by a straight parallelogram (center line + thickness)
//! and kept when it is long, straight and solid enough.

use crate::binary::{Bitmap, Rect, components};
use crate::scale::Scale;
use crate::staff::Staff;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Beam {
    pub x0: f64,
    pub x1: f64,
    /// Center line y = a * x + b
    pub a: f64,
    pub b: f64,
    pub thickness: f64,
    #[serde(skip)]
    pub pixels: Vec<(i32, i32)>,
}

impl Beam {
    pub fn y_at(&self, x: f64) -> f64 {
        self.a * x + self.b
    }
    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }
    pub fn bounds(&self) -> Rect {
        let y0 = self.y_at(self.x0).min(self.y_at(self.x1)) - self.thickness / 2.0;
        let y1 = self.y_at(self.x0).max(self.y_at(self.x1)) + self.thickness / 2.0;
        Rect::new(self.x0.floor() as i32, y0.floor() as i32, (self.x1 - self.x0).ceil() as i32 + 1, (y1 - y0).ceil() as i32 + 1)
    }
}

pub fn detect(nostaff: &Bitmap, scale: &Scale) -> Vec<Beam> {
    let (w, h) = (nostaff.width, nostaff.height);
    let vr = nostaff.vertical_run_lengths();
    let bt = scale.beam as f64;
    let lo = (0.6 * bt).floor() as u16;
    let hi = (1.6 * bt).ceil() as u16;
    let mut mask = Bitmap::new(w, h);
    for i in 0..w * h {
        mask.data[i] = vr[i] >= lo.max(2) && vr[i] <= hi;
    }
    // Close small horizontal gaps (stems crossing the beam).
    let gap = (scale.max_stem + 2) as usize;
    for y in 0..h {
        let row = y * w;
        let mut last_black: Option<usize> = None;
        for x in 0..w {
            if mask.data[row + x] {
                if let Some(l) = last_black
                    && x - l > 1
                    && x - l <= gap + 1
                {
                    for xx in l + 1..x {
                        mask.data[row + xx] = true;
                    }
                }
                last_black = Some(x);
            }
        }
    }
    let il = scale.interline as f64;
    let mut beams = Vec::new();
    for c in components(&mask, None) {
        if (c.bounds.w as f64) < 0.9 * il || c.weight() < 6 {
            continue;
        }
        // Column-wise top/bottom extents.
        let x0 = c.bounds.x;
        let mut tops = vec![i32::MAX; c.bounds.w as usize];
        let mut bots = vec![i32::MIN; c.bounds.w as usize];
        for &(x, y) in &c.pixels {
            let i = (x - x0) as usize;
            tops[i] = tops[i].min(y);
            bots[i] = bots[i].max(y);
        }
        let cols: Vec<(f64, f64, f64)> = (0..tops.len())
            .filter(|&i| tops[i] != i32::MAX)
            .map(|i| ((x0 + i as i32) as f64, tops[i] as f64, bots[i] as f64))
            .collect();
        if cols.len() < 4 {
            continue;
        }
        // Least squares fit of the center line.
        let n = cols.len() as f64;
        let mx = cols.iter().map(|c| c.0).sum::<f64>() / n;
        let my = cols.iter().map(|c| (c.1 + c.2) / 2.0).sum::<f64>() / n;
        let sxx = cols.iter().map(|c| (c.0 - mx).powi(2)).sum::<f64>();
        let sxy = cols.iter().map(|c| (c.0 - mx) * ((c.1 + c.2) / 2.0 - my)).sum::<f64>();
        let a = if sxx > 0.0 { sxy / sxx } else { 0.0 };
        let b = my - a * mx;
        if a.abs() > 1.0 {
            continue;
        }
        let mut thick: Vec<f64> = cols.iter().map(|c| c.2 - c.1 + 1.0).collect();
        thick.sort_by(|p, q| p.partial_cmp(q).unwrap());
        let t = thick[thick.len() / 2];
        if t < 0.6 * bt || t > 1.6 * bt {
            continue;
        }
        // Straightness: center deviation and filling of the parallelogram.
        let dev = cols.iter().map(|c| ((c.1 + c.2) / 2.0 - (a * c.0 + b)).abs()).fold(0.0, f64::max);
        let fill = c.weight() as f64 / (t * cols.len() as f64);
        if dev > 0.25 * il + 1.0 || fill < 0.75 {
            continue;
        }
        beams.push(Beam {
            x0: c.bounds.x as f64,
            x1: (c.bounds.right() - 1) as f64,
            a,
            b,
            thickness: t,
            pixels: c.pixels,
        });
    }
    beams
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Ledger {
    pub staff: usize,
    /// -1, -2, ... above the staff; +1, +2, ... below.
    pub index: i32,
    pub x0: i32,
    pub x1: i32,
    pub y: f64,
}

impl Ledger {
    pub fn pitch(&self) -> i32 {
        if self.index < 0 { -4 + 2 * self.index } else { 4 + 2 * self.index }
    }
}

/// Ledgers: thin horizontal sticks at ledger positions above/below each staff.
pub fn ledgers(nostaff: &Bitmap, staves: &[Staff], si: usize, scale: &Scale) -> Vec<Ledger> {
    let s = &staves[si];
    let il = scale.interline;
    let thin = scale.max_line + 1;
    let min_len = (1.0 * il as f64).round() as i32;
    let mut out: Vec<Ledger> = Vec::new();
    for dir in [-1i32, 1] {
        let mut k = 1;
        loop {
            let mut found = Vec::new();
            let pitch = (4 + 2 * k) as f64 * dir as f64;
            let mut x = s.header_stop;
            while x < s.right {
                let ye = s.pitch_y(pitch, x as f64).round() as i32;
                // Look for a horizontal run crossing this column near ye.
                let mut hit = None;
                for dy in -thin..=thin {
                    if nostaff.get(x, ye + dy) {
                        let (a, b) = nostaff.hrun_at(x, ye + dy);
                        if b - a + 1 >= min_len {
                            hit = Some((a, b, ye + dy));
                            break;
                        }
                    }
                }
                let Some((a, b, y)) = hit else {
                    x += 1;
                    continue;
                };
                // Both ends must be thin (a beam or a head is thick everywhere).
                let ends_thin = |xs: &mut dyn Iterator<Item = i32>| {
                    let v: Vec<i32> = xs.collect();
                    let n = v.iter().filter(|&&xx| {
                        let (p, q) = nostaff.vrun_at(xx, y);
                        q - p + 1 <= thin
                    });
                    n.count() * 2 >= v.len()
                };
                let e = ((b - a + 1) / 6).max(2);
                let prev_ok = k == 1 || out.iter().any(|l| l.index == dir * (k - 1) && l.x0 <= b && l.x1 >= a);
                if prev_ok && ends_thin(&mut (a..a + e)) && ends_thin(&mut (b - e + 1..=b)) && b - a < 4 * il {
                    // Center ordinate: center of the thin run at the ends.
                    let (p, q) = nostaff.vrun_at(a + e / 2, y);
                    found.push(Ledger { staff: si, index: dir * k, x0: a, x1: b, y: (p + q) as f64 / 2.0 });
                }
                x = b + 1;
            }
            if found.is_empty() {
                break;
            }
            out.extend(found);
            k += 1;
        }
    }
    out
}
