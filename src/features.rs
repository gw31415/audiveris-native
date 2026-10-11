// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright © Audiveris 2026. Original author: Hervé Bitteur.
// Rust translation 2026-10-02 of pinned 5.11.0 moment/descriptor routines.
//! Isolated binary glyph -> 110 raw features. No segmentation or staff removal.
use std::{fmt, sync::OnceLock};
const SIZE: usize = 101;
const PLANES: usize = 100;
const LIMIT: usize = 1_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    pub width: usize,
    pub height: usize,
    pub left: i32,
    pub top: i32,
    pub interline: u32,
    pub orientation: Orientation,
}
#[derive(Clone, Copy, Debug)]
pub struct Run {
    pub position: usize,
    pub start: usize,
    pub length: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Error(pub &'static str);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Error {}

/// Pixels are row-major booleans; true is foreground. The supplied box is
/// preserved (including padding) for aspect, as in the upstream RunTable.
pub fn from_bitmap(g: Geometry, pixels: &[bool]) -> Result<[f64; 110], Error> {
    validate_geometry(g)?;
    if g.width.checked_mul(g.height) != Some(pixels.len()) {
        return Err(Error("bitmap dimensions"));
    }
    let (seq, extent) = axes(g);
    let mut runs = Vec::new();
    for p in 0..seq {
        let mut start = None;
        for c in 0..=extent {
            let on = c < extent
                && match g.orientation {
                    Orientation::Horizontal => pixels[p * g.width + c],
                    Orientation::Vertical => pixels[c * g.width + p],
                };
            if on && start.is_none() {
                start = Some(c);
            }
            if !on && let Some(s) = start.take() {
                runs.push(Run {
                    position: p,
                    start: s,
                    length: c - s,
                });
            }
        }
    }
    from_runs(g, &runs)
}
/// Runs must be canonical: ascending position/start, nonempty, disjoint and
/// nonadjacent within each sequence. Pixels within each run are visited in
/// reverse order, matching RunTable.cumulate with no ROI.
pub fn from_runs(g: Geometry, runs: &[Run]) -> Result<[f64; 110], Error> {
    validate_geometry(g)?;
    let (seq, extent) = axes(g);
    let mut mass = 0usize;
    let mut prev: Option<&Run> = None;
    for r in runs {
        if r.position >= seq
            || r.length == 0
            || r.start.checked_add(r.length).is_none_or(|end| end > extent)
        {
            return Err(Error("run bounds"));
        }
        if let Some(p) = prev
            && (r.position < p.position
                || (r.position == p.position && r.start <= p.start + p.length))
        {
            return Err(Error("runs must be sorted, disjoint and nonadjacent"));
        }
        mass = mass.checked_add(r.length).ok_or(Error("mass overflow"))?;
        if mass > LIMIT {
            return Err(Error("glyph exceeds one million pixels"));
        }
        prev = Some(r);
    }
    if mass == 0 {
        return Err(Error("empty glyph"));
    }
    let mut points = Vec::with_capacity(mass);
    let (mut sx, mut sy) = (0i32, 0i32);
    for r in runs {
        for i in (0..r.length).rev() {
            let (x, y) = match g.orientation {
                Orientation::Horizontal => (r.start + i, r.position),
                Orientation::Vertical => (r.position, r.start + i),
            };
            let x = g
                .left
                .checked_add(i32::try_from(x).map_err(|_| Error("coordinate overflow"))?)
                .ok_or(Error("coordinate overflow"))?;
            let y = g
                .top
                .checked_add(i32::try_from(y).map_err(|_| Error("coordinate overflow"))?)
                .ok_or(Error("coordinate overflow"))?;
            sx = sx
                .checked_add(x)
                .ok_or(Error("Java centroid integer overflow"))?;
            sy = sy
                .checked_add(y)
                .ok_or(Error("Java centroid integer overflow"))?;
            points.push((x, y));
        }
    }
    let n = mass as f64;
    let (cx, cy) = (sx as f64 / n, sy as f64 / n);
    // Java Double.MIN_VALUE is the smallest positive subnormal, not MIN_POSITIVE.
    let (mut dxmax, mut dymax) = (f64::from_bits(1), f64::from_bits(1));
    for &(x, y) in &points {
        dxmax = dxmax.max((x as f64 - cx).abs());
        dymax = dymax.max((y as f64 - cy).abs());
    }
    let radius = dxmax.hypot(dymax);
    let tables = LUT.get_or_init(Tables::new);
    let mut re = [0.0; PLANES];
    let mut im = [0.0; PLANES];
    for &(x, y) in &points {
        let lx = ((x as f64 - cx) * 50.0) / radius + 50.0;
        let ly = ((y as f64 - cy) * 50.0) / radius + 50.0;
        if lx >= 0.0 && lx < SIZE as f64 && ly >= 0.0 && ly < SIZE as f64 {
            for k in 0..PLANES {
                re[k] += interpolate(&tables.real, k, lx, ly);
                im[k] -= interpolate(&tables.imag, k, lx, ly);
            }
        }
    }
    let mut out = [0.0; 110];
    for k in 1..PLANES {
        out[k - 1] = (im[k] / n).hypot(re[k] / n);
    }
    let geo = geometric(&points, g.interline);
    out[99..109].copy_from_slice(&geo);
    out[109] = g.height as f64 / g.width as f64;
    Ok(out)
}
fn axes(g: Geometry) -> (usize, usize) {
    match g.orientation {
        Orientation::Horizontal => (g.height, g.width),
        Orientation::Vertical => (g.width, g.height),
    }
}
fn validate_geometry(g: Geometry) -> Result<(), Error> {
    if g.width == 0 || g.height == 0 || g.width > i32::MAX as usize || g.height > i32::MAX as usize
    {
        return Err(Error("invalid bounding box"));
    }
    if g.interline == 0 || g.interline > 46340 {
        return Err(Error("interline outside safe Java i32 square range"));
    }
    Ok(())
}
struct Tables {
    real: Vec<f64>,
    imag: Vec<f64>,
}
static LUT: OnceLock<Tables> = OnceLock::new();
impl Tables {
    fn new() -> Self {
        let mut t = Self {
            real: vec![0.0; PLANES * SIZE * SIZE],
            imag: vec![0.0; PLANES * SIZE * SIZE],
        };
        for x in 0..SIZE {
            let tx = (x as f64 - 50.0) / 50.0;
            for y in 0..SIZE {
                let ty = (y as f64 - 50.0) / 50.0;
                let rad = tx.hypot(ty);
                if rad < 1.0 {
                    let angle = ty.atan2(tx);
                    for p in 0..20 {
                        for r in 0..5 {
                            let temp = (rad * std::f64::consts::PI * r as f64).cos();
                            let i = ((p * 5 + r) * SIZE + x) * SIZE + y;
                            t.real[i] = temp * (angle * p as f64).cos();
                            t.imag[i] = temp * (angle * p as f64).sin();
                        }
                    }
                }
            }
        }
        t
    }
}
fn interpolate(table: &[f64], k: usize, px: f64, py: f64) -> f64 {
    let x = px as usize;
    let y = py as usize;
    let ix = px - x as f64;
    let iy = py - y as f64;
    let at = |x, y| table[(k * SIZE + x) * SIZE + y];
    let v = at(x, y);
    if x == 100 {
        if y == 100 {
            v
        } else {
            v + iy * (at(x, y + 1) - v)
        }
    } else {
        let vx = v + ix * (at(x + 1, y) - v);
        if y == 100 {
            vx
        } else {
            let v1 = at(x, y + 1);
            let vx1 = v1 + ix * (at(x + 1, y + 1) - v1);
            vx + iy * (vx1 - vx)
        }
    }
}
fn geometric(points: &[(i32, i32)], unit: u32) -> [f64; 10] {
    let w = points.len() as f64;
    let w2 = w * w;
    let w3 = (w * w * w * w * w).sqrt();
    let (mut mx, mut my) = (0.0, 0.0);
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    for &(x, y) in points.iter().rev() {
        mx += x as f64;
        my += y as f64;
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        ymin = ymin.min(y);
        ymax = ymax.max(y);
    }
    mx /= w;
    my /= w;
    let (mut n20, mut n11, mut n02, mut n30, mut n21, mut n12, mut n03) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(x, y) in points.iter().rev() {
        let x = x as f64 - mx;
        let y = y as f64 - my;
        n11 += x * y;
        n12 += x * y * y;
        n21 += x * x * y;
        n20 += x * x;
        n02 += y * y;
        n30 += x * x * x;
        n03 += y * y * y;
    }
    [
        w / (unit * unit) as f64,
        (xmax as i64 - xmin as i64 + 1) as f64 / unit as f64,
        (ymax as i64 - ymin as i64 + 1) as f64 / unit as f64,
        n20 / w2,
        n11 / w2,
        n02 / w2,
        n30 / w3,
        n21 / w3,
        n12 / w3,
        n03 / w3,
    ]
}
