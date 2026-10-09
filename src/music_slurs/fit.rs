// SPDX-License-Identifier: AGPL-3.0-or-later
//! Short component-derived Slur candidate math; acceptance requires owned endpoint lookup.
//! Inputs are actual retained scale and glyph runs; no original mask, coordinates or IDs.
use audiveris_area::hypot as java_hypot;
use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl From<Pixel> for Point {
    fn from(p: Pixel) -> Self {
        Self {
            x: p.x as f64,
            y: p.y as f64,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    pub distance: f64,
    pub ccw: i32,
    pub above: bool,
    pub first_angle: f64,
    pub last_angle: f64,
}
#[derive(Clone, Debug)]
pub struct Impacts {
    pub grade: f64,
}
#[derive(Clone, Debug)]
pub struct Fit {
    pub selected_after_impacts: Option<Circle>,
    pub impacts: Option<Impacts>,
    pub bezier: Option<[Point; 4]>,
    pub math_accepted_before_binary_and_head_links: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitError {
    TooFewPoints,
    OutsideShortPointCount,
    InvalidInterline,
    SingularMatrix,
    InvalidGlyphRun,
}
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct VerticalRun {
    pub x: i32,
    pub y: i32,
    pub length: i32,
}
#[derive(Clone, Debug)]
pub struct GlyphRuns {
    pub bounds: Rect,
    pub runs: Vec<VerticalRun>,
}
#[derive(Clone, Debug)]
pub struct Candidate {
    pub component_indices: Vec<usize>,
    pub points: Vec<Pixel>,
}
#[derive(Clone, Debug)]
pub struct Extraction {
    pub candidates: Vec<Candidate>,
}
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a <= b {
        a
    } else {
        b
    }
}
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a >= b {
        a
    } else {
        b
    }
}
fn dot<const N: usize>(a: &[f64; N], b: &[f64; N]) -> f64 {
    let mut s = 0.;
    for k in 0..N {
        s += a[k] * b[k];
    }
    s
}
fn multiply<const N: usize>(a: &[[f64; N]; N], b: &[[f64; N]; N]) -> [[f64; N]; N] {
    let mut result = [[0.; N]; N];
    for j in 0..N {
        let mut col = [0.; N];
        for k in 0..N {
            col[k] = b[k][j];
        }
        for i in 0..N {
            result[i][j] = dot(&a[i], &col);
        }
    }
    result
}
// Preserve JAMA's indexed operation order used by the original fit comparison.
#[allow(clippy::needless_range_loop)]
fn inverse4(mut lu: [[f64; 4]; 4]) -> Result<[[f64; 4]; 4], FitError> {
    let mut piv = [0, 1, 2, 3];
    for j in 0..4 {
        let mut col = [0.; 4];
        for i in 0..4 {
            col[i] = lu[i][j];
        }
        for i in 0..4 {
            let mut sum = 0.;
            for k in 0..i.min(j) {
                sum += lu[i][k] * col[k];
            }
            col[i] -= sum;
            lu[i][j] = col[i];
        }
        let mut p = j;
        for i in j + 1..4 {
            if col[i].abs() > col[p].abs() {
                p = i;
            }
        }
        if p != j {
            lu.swap(p, j);
            piv.swap(p, j);
        }
        if lu[j][j] != 0. {
            for i in j + 1..4 {
                lu[i][j] /= lu[j][j];
            }
        }
    }
    if (0..4).any(|j| lu[j][j] == 0.) {
        return Err(FitError::SingularMatrix);
    }
    let mut x = [[0.; 4]; 4];
    for i in 0..4 {
        x[i][piv[i]] = 1.;
    }
    for k in 0..4 {
        for i in k + 1..4 {
            for j in 0..4 {
                x[i][j] -= x[k][j] * lu[i][k];
            }
        }
    }
    for k in (0..4).rev() {
        for j in 0..4 {
            x[k][j] /= lu[k][k];
        }
        for i in 0..k {
            for j in 0..4 {
                x[i][j] -= x[k][j] * lu[i][k];
            }
        }
    }
    Ok(x)
}
fn relative_ccw(first: Pixel, middle: Pixel, last: Pixel) -> i32 {
    let x2 = (middle.x - first.x) as f64;
    let y2 = (middle.y - first.y) as f64;
    let mut px = (last.x - first.x) as f64;
    let mut py = (last.y - first.y) as f64;
    let mut v = px * y2 - py * x2;
    if v == 0. {
        v = px * x2 + py * y2;
        if v > 0. {
            px -= x2;
            py -= y2;
            v = px * x2 + py * y2;
            if v < 0. {
                v = 0.;
            }
        }
    }
    if v < 0. {
        -1
    } else if v > 0. {
        1
    } else {
        0
    }
}
fn bisector(a: Point, b: Point) -> (Point, Point) {
    let dx = (b.x - a.x) / 2.;
    let dy = (b.y - a.y) / 2.;
    let mx = a.x + dx;
    let my = a.y + dy;
    (
        Point {
            x: mx + dy,
            y: my - dx,
        },
        Point {
            x: mx - dy,
            y: my + dx,
        },
    )
}
fn intersection(a: Point, b: Point, c: Point, d: Point) -> Point {
    let den = (a.x - b.x) * (c.y - d.y) - (a.y - b.y) * (c.x - d.x);
    let v12 = a.x * b.y - a.y * b.x;
    let v34 = c.x * d.y - c.y * d.x;
    Point {
        x: (v12 * (c.x - d.x) - (a.x - b.x) * v34) / den,
        y: (v12 * (c.y - d.y) - (a.y - b.y) * v34) / den,
    }
}
impl Circle {
    fn new(center: Point, radius: f64, points: &[Pixel]) -> Self {
        let first = points[0];
        let last = points[points.len() - 1];
        let ccw = relative_ccw(first, points[points.len() / 2], last);
        Self {
            center,
            radius,
            distance: 0.,
            ccw,
            above: if first.x < last.x {
                ccw == -1
            } else {
                ccw == 1
            },
            first_angle: (first.y as f64 - center.y).atan2(first.x as f64 - center.x),
            last_angle: (last.y as f64 - center.y).atan2(last.x as f64 - center.x),
        }
    }
    pub fn rough(points: &[Pixel]) -> Self {
        let a = Point::from(points[0]);
        let mid = Point::from(points[points.len() / 2]);
        let b = Point::from(points[points.len() - 1]);
        let (u, v) = bisector(a, mid);
        let (w, z) = bisector(mid, b);
        let center = intersection(u, v, w, z);
        let r = java_hypot(center.x - mid.x, center.y - mid.y);
        Self::new(center, r, points)
    }
    pub fn fitted(points: &[Pixel]) -> Result<Self, FitError> {
        if points.len() < 3 {
            return Err(FitError::TooFewPoints);
        }
        let design: Vec<[f64; 4]> = points
            .iter()
            .map(|p| {
                let x = p.x as f64;
                let y = p.y as f64;
                [x * x + y * y, x, y, 1.]
            })
            .collect();
        let mut scatter = [[0.; 4]; 4];
        for j in 0..4 {
            for i in 0..4 {
                let mut sum = 0.;
                for row in &design {
                    sum += row[i] * row[j];
                }
                scatter[i][j] = sum;
            }
        }
        let mut matrix = [[0.; 4]; 4];
        let mut rhs = [0.; 4];
        for i in 0..4 {
            rhs[i] = -scatter[0][i];
            for j in 1..4 {
                matrix[i][j - 1] = scatter[i][j];
            }
        }
        matrix[0][3] = -0.5;
        // Original constructs inverse first, then multiplies; direct solve changes rounding.
        let inverse = inverse4(matrix)?;
        let mut solution = [0.; 4];
        for i in 0..4 {
            solution[i] = dot(&inverse[i], &rhs);
        }
        let center = Point {
            x: -solution[0] / 2.,
            y: -solution[1] / 2.,
        };
        let radius = (center.x * center.x + center.y * center.y - solution[2]).sqrt();
        let mut circle = Self::new(center, radius, points);
        circle.distance = circle.residual_sum(points).sqrt() / points.len() as f64;
        Ok(circle)
    }
    fn residual_sum(&self, points: &[Pixel]) -> f64 {
        let mut sum = 0.;
        for p in points {
            let delta =
                java_hypot(p.x as f64 - self.center.x, p.y as f64 - self.center.y) - self.radius;
            sum += delta * delta;
        }
        sum
    }
    pub fn distance_rms(&mut self, points: &[Pixel]) -> f64 {
        self.distance = (self.residual_sum(points) / points.len() as f64).sqrt();
        self.distance
    }
    pub fn arc_angle(&self) -> f64 {
        let mut arc = if self.ccw == 1 {
            self.first_angle - self.last_angle
        } else {
            self.last_angle - self.first_angle
        };
        if arc < 0. {
            arc += 2. * PI;
        }
        arc
    }
    pub fn mid_angle(&self) -> f64 {
        let half = self.arc_angle() / 2.;
        let mut mid = if self.ccw == 1 {
            self.first_angle - half
        } else {
            self.first_angle + half
        };
        if mid < -PI {
            mid += 2. * PI;
        }
        if mid > PI {
            mid -= 2. * PI;
        }
        mid
    }
    pub fn bezier(&self) -> Option<[Point; 4]> {
        if self.first_angle.is_nan() || self.last_angle.is_nan() {
            return None;
        }
        let arc = self.arc_angle();
        let x0 = (arc / 2.).cos();
        let y0 = (arc / 2.).sin();
        let x1 = (4. - x0) / 3.;
        let y1 = ((1. - x0) * (3. - x0)) / (3. * y0);
        let theta = self.mid_angle();
        let rotation = [
            [theta.cos(), -theta.sin(), 0.],
            [theta.sin(), theta.cos(), 0.],
            [0., 0., 1.],
        ];
        let scaling = [[self.radius, 0., 0.], [0., self.radius, 0.], [0., 0., 1.]];
        let translation = [
            [1., 0., self.center.x],
            [0., 1., self.center.y],
            [0., 0., 1.],
        ];
        let op = multiply(&multiply(&translation, &scaling), &rotation);
        let uv = [[x0, y0, 1.], [x1, y1, 1.], [x1, -y1, 1.], [x0, -y0, 1.]];
        let mut pts = [Point { x: 0., y: 0. }; 4];
        for i in 0..4 {
            pts[i] = Point {
                x: dot(&op[0], &uv[i]),
                y: dot(&op[1], &uv[i]),
            };
        }
        if pts[0].x > pts[3].x {
            pts.reverse();
        }
        Some(pts)
    }
}
fn selected_model(
    rough: &Circle,
    fitted: &Result<Circle, FitError>,
    points: &[Pixel],
    interline: f64,
    is_seed: bool,
) -> Option<Circle> {
    if rough.radius < (if is_seed { 0.3 } else { 0.4 }) * interline {
        return None;
    }
    if !rough.radius.is_infinite() && rough.arc_angle() > 190f64.to_radians() {
        return None;
    }
    let fitted = fitted.as_ref().ok()?;
    let chosen = if rough.radius.is_infinite()
        || (rough.radius - fitted.radius).abs() / java_max(rough.radius, fitted.radius) <= 0.25
    {
        fitted.clone()
    } else {
        let mut r = rough.clone();
        r.distance_rms(points);
        r
    };
    if chosen.distance > 0.15 * interline {
        None
    } else {
        Some(chosen)
    }
}
fn short_impacts(circle: &mut Circle, points: &[Pixel], interline: f64) -> Option<Impacts> {
    // Both actual short side models reference the same global model; each call writes its distance.
    let d1 = circle.distance_rms(points);
    let d2 = circle.distance_rms(points);
    let dist = (d1 + d2) / 2.;
    if dist > 0.1 * interline {
        return None;
    }
    let arc = circle.arc_angle();
    if arc > 190f64.to_radians() {
        return None;
    }
    let mut mid = circle.mid_angle();
    if mid < 0. {
        mid += 2. * PI;
    }
    mid %= PI;
    let vertical = java_min(mid.abs(), (PI - mid).abs());
    if vertical < 10f64.to_radians() {
        return None;
    }
    let first = points[0];
    let middle = points[points.len() / 2];
    let last = points[points.len() - 1];
    let width = (last.x - first.x).abs() as f64;
    if width < 0.7 * interline {
        return None;
    }
    // Line2D.ptLineDist: projection subtraction order, including max(0,roundoff).
    let dx = (last.x - first.x) as f64;
    let dy = (last.y - first.y) as f64;
    let px = (middle.x - first.x) as f64;
    let py = (middle.y - first.y) as f64;
    let prod = px * dx + py * dy;
    let mut length_sq = px * px + py * py - prod * prod / (dx * dx + dy * dy);
    if length_sq < 0. {
        length_sq = 0.;
    }
    let height = length_sq.sqrt();
    if height < 0.07 * interline {
        return None;
    }
    let raw = [
        1. - dist / (0.1 * interline),
        (190f64.to_radians() - arc) / (190f64.to_radians() - 170f64.to_radians()),
        (width - 0.7 * interline) / (1.5 * interline - 0.7 * interline),
        (height - 0.07 * interline) / (1.0 * interline - 0.07 * interline),
        (vertical - 10f64.to_radians()) / (25f64.to_radians() - 10f64.to_radians()),
    ];
    let mut values = [0.; 5];
    let weights = [3., 1., 1., 1., 1.];
    let mut global = 1.;
    let mut total = 0.;
    for i in 0..5 {
        let v = raw[i];
        values[i] = v.clamp(0., 1.);
        total += weights[i];
        if values[i] == 0. {
            global = 0.;
        } else {
            global *= values[i].powf(weights[i]);
        }
    }
    Some(Impacts {
        grade: 0.8 * global.powf(1. / total),
    })
}
pub fn fit_short(points: &[Pixel], interline: f64) -> Result<Fit, FitError> {
    if !interline.is_finite() || interline <= 0. {
        return Err(FitError::InvalidInterline);
    }
    if points.len() < 3 {
        return Err(FitError::TooFewPoints);
    }
    let side_length = (6. * interline).round_ties_even() as usize;
    if points.len() > side_length {
        return Err(FitError::OutsideShortPointCount);
    }
    let rough = Circle::rough(points);
    let fitted = Circle::fitted(points);
    let selected_before_impacts = selected_model(&rough, &fitted, points, interline, false);
    let mut selected_after_impacts = selected_before_impacts.clone();
    let impacts = selected_after_impacts
        .as_mut()
        .and_then(|c| short_impacts(c, points, interline));
    let bezier = selected_after_impacts.as_ref().and_then(Circle::bezier);
    let accepted = impacts.as_ref().is_some_and(|i| i.grade >= 0.8 * 0.1) && bezier.is_some();
    Ok(Fit {
        selected_after_impacts,
        impacts,
        bezier,
        math_accepted_before_binary_and_head_links: accepted,
    })
}
/// Broad alternatives only. Every candidate still requires original BINARY run support;
/// pairs additionally require source true-gap validation, then real Head/Chord linkage.
pub fn extract_short(glyphs: &[GlyphRuns], interline: f64) -> Result<Extraction, FitError> {
    if !interline.is_finite() || interline <= 0. {
        return Err(FitError::InvalidInterline);
    }
    let max_width = (6. * interline).round_ties_even() as i32;
    let mut singles = Vec::new();
    for (idx, g) in glyphs.iter().enumerate() {
        if g.bounds.width <= 0
            || g.bounds.height <= 0
            || g.runs.iter().any(|r| {
                r.x < 0
                    || r.x >= g.bounds.width
                    || r.y < 0
                    || r.length <= 0
                    || r.y
                        .checked_add(r.length)
                        .is_none_or(|end| end > g.bounds.height)
            })
        {
            return Err(FitError::InvalidGlyphRun);
        }
        if g.bounds.width > max_width {
            continue;
        }
        let mut columns: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
        for r in &g.runs {
            for y in r.y..r.y + r.length {
                columns
                    .entry(g.bounds.x + r.x)
                    .or_default()
                    .insert(g.bounds.y + y);
            }
        }
        if columns.len() < 3 {
            continue;
        }
        if columns.values().any(|ys| {
            let span = ys.last().unwrap() - ys.first().unwrap();
            span as usize + 1 != ys.len() || span as f64 > 0.5 * interline
        }) {
            continue;
        }
        let points = columns
            .iter()
            .map(|(&x, ys)| Pixel {
                x,
                y: (ys.iter().map(|&y| y as f64).sum::<f64>() / ys.len() as f64).round_ties_even()
                    as i32,
            })
            .collect();
        singles.push(Candidate {
            component_indices: vec![idx],
            points,
        });
    }
    let mut candidates = singles.clone();
    for i in 0..singles.len() {
        for j in i + 1..singles.len() {
            let a = &singles[i];
            let b = &singles[j];
            let (left, right) = if a.points[0].x < b.points[0].x {
                (a, b)
            } else {
                (b, a)
            };
            let end = left.points.last().unwrap();
            let start = right.points[0];
            if start.x <= end.x {
                continue;
            }
            let gap = java_hypot((start.x - end.x) as f64, (start.y - end.y) as f64);
            if gap > interline {
                continue;
            }
            if right.points.last().unwrap().x - left.points[0].x + 1 > max_width {
                continue;
            }
            let mut points = left.points.clone();
            points.extend_from_slice(&right.points);
            candidates.push(Candidate {
                component_indices: vec![left.component_indices[0], right.component_indices[0]],
                points,
            });
        }
    }
    Ok(Extraction { candidates })
}

include!("fit_source_observations.rs");
