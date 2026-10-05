// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// CurvedFilament.computeLine, NaturalSpline, GeoPath.yAtX (horizontal automatic endpoints).
use crate::sections::{Bounds, Section};
pub type Point = [f64; 2];
#[derive(Debug, Clone)]
pub struct Curve {
    pub points: Vec<Point>,
    segments: Vec<Vec<Point>>,
}
fn derivatives(z: &[f64]) -> Vec<f64> {
    let n = z.len() - 1;
    let mut g = vec![0.; n + 1];
    g[0] = 0.5;
    for i in 1..n {
        g[i] = 1. / (4. - g[i - 1]);
    }
    g[n] = 1. / (2. - g[n - 1]);
    let mut d = vec![0.; n + 1];
    d[0] = 3. * (z[1] - z[0]) * g[0];
    for i in 1..n {
        d[i] = (3. * (z[i + 1] - z[i - 1]) - d[i - 1]) * g[i];
    }
    d[n] = (3. * (z[n] - z[n - 1]) - d[n - 1]) * g[n];
    for i in (0..n).rev() {
        d[i] -= g[i] * d[i + 1];
    }
    d
}
impl Curve {
    pub fn interpolate(points: Vec<Point>) -> Option<Self> {
        if points.iter().flatten().any(|v| !v.is_finite())
            || points.len() < 2
            || points.windows(2).any(|w| w[0][0] >= w[1][0])
        {
            return None;
        }
        let n = points.len() - 1;
        let segments = match n {
            1 => vec![points.clone()],
            2 => vec![vec![
                points[0],
                [
                    2. * points[1][0] - (points[0][0] + points[2][0]) / 2.,
                    2. * points[1][1] - (points[0][1] + points[2][1]) / 2.,
                ],
                points[2],
            ]],
            _ => {
                let dx = derivatives(&points.iter().map(|p| p[0]).collect::<Vec<_>>());
                let dy = derivatives(&points.iter().map(|p| p[1]).collect::<Vec<_>>());
                (0..n)
                    .map(|i| {
                        vec![
                            points[i],
                            [points[i][0] + dx[i] / 3., points[i][1] + dy[i] / 3.],
                            [
                                points[i + 1][0] - dx[i + 1] / 3.,
                                points[i + 1][1] - dy[i + 1] / 3.,
                            ],
                            points[i + 1],
                        ]
                    })
                    .collect()
            }
        };
        Some(Self { points, segments })
    }
    /// Stored Bezier segments of the interpolated path, for native area assembly.
    pub fn path_segments(&self) -> &[Vec<Point>] {
        &self.segments
    }

    /// Evaluate the actual Bezier boundary for java.awt.Area-style containment.
    /// GeoPath.yAtX uses endpoint-normalized t; area containment instead solves x(t).
    pub fn area_y(&self, x: f64) -> f64 {
        let s = self
            .segments
            .iter()
            .find(|s| x <= s.last().unwrap()[0])
            .unwrap_or(self.segments.last().unwrap());
        let value = |t: f64, axis: usize| {
            let u = 1. - t;
            match s.len() {
                2 => s[0][axis] * u + s[1][axis] * t,
                3 => s[0][axis] * u * u + 2. * s[1][axis] * t * u + s[2][axis] * t * t,
                _ => {
                    s[0][axis] * u * u * u
                        + 3. * s[1][axis] * t * u * u
                        + 3. * s[2][axis] * t * t * u
                        + s[3][axis] * t * t * t
                }
            }
        };
        let (mut low, mut high) = (0., 1.);
        for _ in 0..60 {
            let t = (low + high) / 2.;
            if value(t, 0) < x {
                low = t;
            } else {
                high = t;
            }
        }
        value((low + high) / 2., 1)
    }
    pub fn derivative(&self, x: f64) -> f64 {
        let s = self
            .segments
            .iter()
            .find(|s| x <= s.last().unwrap()[0])
            .unwrap_or(self.segments.last().unwrap());
        let p = s[0];
        let q = *s.last().unwrap();
        let dx = q[0] - p[0];
        let t = (x - p[0]) / dx;
        let u = 1. - t;
        match s.len() {
            2 => (q[1] - p[1]) / dx,
            3 => (-2. * p[1] * u + 2. * s[1][1] * (1. - 2. * t) + 2. * q[1] * t) / dx,
            _ => {
                (-3. * p[1] * u * u
                    + 3. * s[1][1] * (u * u - 2. * u * t)
                    + 3. * s[2][1] * (2. * t * u - t * t)
                    + 3. * q[1] * t * t)
                    / dx
            }
        }
    }
    /// GeoPath affine Y translation, preserving spline control points.
    pub fn translated_y(&self, delta: f64) -> Self {
        let mut result = self.clone();
        for p in &mut result.points {
            p[1] += delta;
        }
        for segment in &mut result.segments {
            for p in segment {
                p[1] += delta;
            }
        }
        result
    }
    pub fn y(&self, x: f64) -> f64 {
        let p = self.points[0];
        let q = *self.points.last().unwrap();
        if x < p[0] || x > q[0] {
            return p[1] + ((q[1] - p[1]) / (q[0] - p[0])) * (x - p[0]);
        }
        let s = self
            .segments
            .iter()
            .find(|s| x <= s.last().unwrap()[0])
            .unwrap();
        let p = s[0];
        let q = *s.last().unwrap();
        let t = (x - p[0]) / (q[0] - p[0]);
        let u = 1. - t;
        match s.len() {
            2 => p[1] + t * (q[1] - p[1]),
            3 => p[1] * u * u + 2. * s[1][1] * t * u + q[1] * t * t,
            _ => {
                p[1] * u * u * u
                    + 3. * s[1][1] * t * u * u
                    + 3. * s[2][1] * t * t * u
                    + q[1] * t * t * t
            }
        }
    }
}
pub fn bounds(sections: &[Section], members: &[usize]) -> Bounds {
    let mut b = sections[members[0]].bounds();
    for &i in &members[1..] {
        let a = sections[i].bounds();
        let right = (b.x + b.width).max(a.x + a.width);
        let bottom = (b.y + b.height).max(a.y + a.height);
        b.x = b.x.min(a.x);
        b.y = b.y.min(a.y);
        b.width = right - b.x;
        b.height = bottom - b.y;
    }
    b
}
pub fn sorted_members(sections: &[Section], members: &mut Vec<usize>) {
    members.sort_by_key(|&i| {
        let b = sections[i].bounds();
        (b.x, b.y, sections[i].id.0)
    });
    members.dedup();
}
fn centroid(
    sections: &[Section],
    members: &[usize],
    left: i64,
    width: usize,
    min_weight: usize,
) -> Option<Point> {
    let (mut n, mut sx, mut sy) = (0usize, 0., 0.);
    for &i in members {
        let s = &sections[i];
        for (j, r) in s.runs.iter().enumerate() {
            let start = (r.start as i64).max(left);
            let stop = ((r.start + r.length) as i64).min(left + width as i64);
            for x in start..stop {
                n += 1;
                sx += x as f64;
                sy += (s.first_pos + j) as f64;
            }
        }
    }
    if n >= min_weight {
        Some([sx / n as f64, sy / n as f64])
    } else {
        None
    }
}
pub fn compute(sections: &[Section], members: &[usize], interline: usize) -> Option<Curve> {
    let b = bounds(sections, members);
    let start = b.x as f64;
    let stop = (b.x + b.width - 1) as f64;
    let len = b.width as f64;
    let probe = (interline as f64 * 0.5).round_ties_even() as usize;
    let min_weight = ((interline as f64 * 0.2).round_ties_even() as usize).max(1);
    let count = (len / (interline * 4) as f64).round_ties_even() as usize;
    let dx = len / count as f64;
    let first = centroid(sections, members, start.ceil() as i64, probe, 1)?;
    let mut points = vec![[start, first[1]]];
    for i in 1..count {
        if let Some(p) = centroid(
            sections,
            members,
            (start + i as f64 * dx).round_ties_even() as i64,
            probe,
            min_weight,
        ) {
            points.push(p);
        }
    }
    let last = centroid(
        sections,
        members,
        (stop - probe as f64 + 1.).floor() as i64,
        probe,
        1,
    )?;
    points.push([stop, last[1]]);
    Curve::interpolate(points)
}
/// CurvedFilament automatic endpoints with vertical orientation and one-interline segments.
/// Member order is retained from the absolute abscissa/ordinate/id compound ordering.
pub fn compute_vertical(
    sections: &[Section],
    members: &[usize],
    interline: usize,
) -> Option<Curve> {
    use crate::sections::Orientation;
    let b = bounds(sections, members);
    let (start, stop, len) = (b.y as f64, (b.y + b.height - 1) as f64, b.height as f64);
    let probe = (interline as f64 * 0.5).round_ties_even() as usize;
    let min_weight = ((interline as f64 * 0.2).round_ties_even() as usize).max(1);
    let count = (len / interline as f64).round_ties_even() as usize;
    let dx = len / count as f64;
    let centroid = |left: i64, minimum: usize| {
        let (mut n, mut sx, mut sy) = (0usize, 0., 0.);
        for &i in members {
            let section = &sections[i];
            for (j, r) in section.runs.iter().enumerate() {
                for v in r.start..r.start + r.length {
                    let (x, y) = match section.orientation {
                        Orientation::Horizontal => (v, section.first_pos + j),
                        Orientation::Vertical => (section.first_pos + j, v),
                    };
                    if (y as i64) >= left && (y as i64) < left + probe as i64 {
                        n += 1;
                        sx += y as f64;
                        sy += x as f64;
                    }
                }
            }
        }
        if n >= minimum {
            Some([sx / n as f64, sy / n as f64])
        } else {
            None
        }
    };
    let first = centroid(start.ceil() as i64, 1)?;
    let mut points = vec![[start, first[1]]];
    for i in 1..count {
        if let Some(p) = centroid((start + i as f64 * dx).round_ties_even() as i64, min_weight) {
            points.push(p);
        }
    }
    let last = centroid((stop - probe as f64 + 1.).floor() as i64, 1)?;
    points.push([stop, last[1]]);
    Curve::interpolate(points)
}
/// CurvedFilament.getMeanCurvature, including upstream infinity/NaN behavior for straight segments.
pub fn mean_radius(points: &[Point]) -> f64 {
    let bisector = |a: Point, b: Point| {
        let dx = (b[0] - a[0]) / 2.;
        let dy = (b[1] - a[1]) / 2.;
        let mx = a[0] + dx;
        let my = a[1] + dy;
        [mx + dy, my - dx, mx - dy, my + dx]
    };
    let mut total = 0.;
    let mut n = 0;
    for w in points.windows(3) {
        let [x1, y1, x2, y2] = bisector(w[0], w[1]);
        let [x3, y3, x4, y4] = bisector(w[1], w[2]);
        let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        let a = x1 * y2 - y1 * x2;
        let b = x3 * y4 - y3 * x4;
        let x = (a * (x3 - x4) - (x1 - x2) * b) / den;
        let y = (a * (y3 - y4) - (y1 - y2) * b) / den;
        total += 1. / (x - w[2][0]).hypot(y - w[2][1]);
        n += 1;
    }
    if n > 0 {
        1. / (total / n as f64)
    } else {
        f64::INFINITY
    }
}

/// CurvedFilament.computeLine with explicit ending points (setEndingPoints).
pub fn compute_ended(
    sections: &[Section],
    members: &[usize],
    interline: usize,
    a: Point,
    b: Point,
) -> Option<Curve> {
    let len = b[0] - a[0] + 1.;
    let count = (len / (interline * 4) as f64).round_ties_even() as usize;
    let dx = len / count as f64;
    let probe = (interline as f64 * 0.5).round_ties_even() as usize;
    let min_weight = ((interline as f64 * 0.2).round_ties_even() as usize).max(1);
    let mut points = vec![a];
    for i in 1..count {
        if let Some(p) = centroid(
            sections,
            members,
            (a[0] + i as f64 * dx).round_ties_even() as i64,
            probe,
            min_weight,
        ) {
            points.push(p);
        }
    }
    points.push(b);
    Curve::interpolate(points)
}

/// CurvedFilament.getRadius: circumcenter distance to the middle defining point.
pub fn point_radius(w: &[Point]) -> f64 {
    let bisector = |a: Point, b: Point| {
        let dx = (b[0] - a[0]) / 2.;
        let dy = (b[1] - a[1]) / 2.;
        let mx = a[0] + dx;
        let my = a[1] + dy;
        [mx + dy, my - dx, mx - dy, my + dx]
    };
    let [x1, y1, x2, y2] = bisector(w[0], w[1]);
    let [x3, y3, x4, y4] = bisector(w[1], w[2]);
    let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let a = x1 * y2 - y1 * x2;
    let b = x3 * y4 - y3 * x4;
    let x = (a * (x3 - x4) - (x1 - x2) * b) / den;
    let y = (a * (y3 - y4) - (y1 - y2) * b) / den;
    (x - w[1][0]).hypot(y - w[1][1])
}
