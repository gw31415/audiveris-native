// Copyright (c) 1998, 2018, Oracle and/or its affiliates. All rights reserved.
// Copyright (c) 1998, 2025, Oracle and/or its affiliates. All rights reserved.
// Rust translation and bounded assembly: 2026 audiveris-native contributors.
// SPDX-License-Identifier: GPL-2.0-only WITH Classpath-exception-2.0
// Derived numerical operations from OpenJDK Curve/Order1/Order2/Order3.
// The Classpath exception is retained for this modified independent library.
// Distributed WITHOUT ANY WARRANTY; see LICENSE and NOTICE.
mod fdlibm_hypot;
mod rectangle;
pub use fdlibm_hypot::hypot;

pub type Point = [f64; 2];
const MAX_SEGMENTS: usize = 131_072;
#[derive(Clone, Debug)]
struct Edge {
    points: Vec<Point>,
    x: [f64; 4],
    y: [f64; 4],
    direction: i32,
    xmin: f64,
    xmax: f64,
}
fn polynomial(c: [f64; 4], t: f64) -> f64 {
    ((c[3] * t + c[2]) * t + c[1]) * t + c[0]
}
fn coefficients(p: &[Point], axis: usize) -> [f64; 4] {
    let a = p[0][axis];
    match p.len() {
        2 => [a, p[1][axis] - a, 0., 0.],
        3 => {
            let b = p[1][axis];
            [a, b + b - a - a, a - b - b + p[2][axis], 0.]
        }
        _ => {
            let b = p[1][axis];
            let c = p[2][axis];
            [
                a,
                (b - a) * 3.,
                (c - b - b + a) * 3.,
                p[3][axis] - (c - b) * 3. - a,
            ]
        }
    }
}
fn quadratic_roots(c: f64, b: f64, a: f64) -> Vec<f64> {
    if a == 0. {
        return if b == 0. { Vec::new() } else { vec![-c / b] };
    }
    let mut d = b * b - 4. * a * c;
    if d < 0. {
        return Vec::new();
    }
    d = d.sqrt();
    if b < 0. {
        d = -d;
    }
    let q = (b + d) / -2.;
    let mut roots = vec![q / a];
    if q != 0. {
        roots.push(c / q);
    }
    roots
}
fn quadratic_t(target: f64, c: [f64; 4]) -> f64 {
    let constant = c[0] - target;
    for root in quadratic_roots(constant, c[1], c[2]) {
        if (0.0..=1.0).contains(&root) {
            return root;
        }
    }
    if 0. < (constant + (constant + c[1] + c[2])) / 2. {
        0.
    } else {
        1.
    }
}
fn refine(c: [f64; 4], target: f64, mut t: f64) -> f64 {
    if !t.is_finite() || !(-0.1..=1.1).contains(&t) {
        return -1.;
    }
    let mut y = polynomial(c, t);
    let (mut low, mut high) = if y < target { (t, 1.) } else { (0., t) };
    let mut slope_mode = true;
    let mut settled = false;
    for _ in 0..256 {
        if y == target {
            settled = true;
            break;
        }
        if slope_mode {
            let slope = (3. * c[3] * t + 2. * c[2]) * t + c[1];
            if slope == 0. {
                slope_mode = false;
                continue;
            }
            let next = t + (target - y) / slope;
            if next == t || next <= low || next >= high || !next.is_finite() {
                slope_mode = false;
                continue;
            }
            t = next;
        } else {
            let next = (low + high) / 2.;
            if next == low || next == high {
                settled = true;
                break;
            }
            t = next;
        }
        y = polynomial(c, t);
        if y < target {
            low = t;
        } else if y > target {
            high = t;
        } else {
            settled = true;
            break;
        }
    }
    if !settled || t > 1. || !t.is_finite() {
        -1.
    } else {
        t
    }
}
fn cubic_t(target: f64, c: [f64; 4]) -> Result<f64, String> {
    if c[3] == 0. {
        return Ok(quadratic_t(target, c));
    }
    let a = c[2] / c[3];
    let b = c[1] / c[3];
    let cc = (c[0] - target) / c[3];
    let mut q = (a * a - 3. * b) / 9.;
    let mut r = (2. * a * a * a - 9. * a * b + 27. * cc) / 54.;
    let r2 = r * r;
    let q3 = q * q * q;
    let third = a / 3.;
    let mut t;
    if r2 < q3 {
        let theta = (r / q3.sqrt()).acos();
        q = -2. * q.sqrt();
        t = refine(c, target, q * (theta / 3.).cos() - third);
        if t < 0. {
            t = refine(
                c,
                target,
                q * ((theta + std::f64::consts::PI * 2.) / 3.).cos() - third,
            );
        }
        if t < 0. {
            t = refine(
                c,
                target,
                q * ((theta - std::f64::consts::PI * 2.) / 3.).cos() - third,
            );
        }
    } else {
        let negative = r < 0.;
        let s = (r2 - q3).sqrt();
        if negative {
            r = -r;
        }
        let mut aa = (r + s).powf(1. / 3.);
        if !negative {
            aa = -aa;
        }
        let bb = if aa == 0. { 0. } else { q / aa };
        t = refine(c, target, (aa + bb) - third);
    }
    if t < 0. || !t.is_finite() {
        let (mut low, mut high) = (0., 1.);
        for _ in 0..256 {
            t = (low + high) / 2.;
            if t == low || t == high {
                return Ok(t);
            }
            let y = polynomial(c, t);
            if y < target {
                low = t;
            } else if y > target {
                high = t;
            } else {
                return Ok(t);
            }
        }
        return Err("unsupported_area_root_convergence".into());
    }
    Ok(t)
}
fn split(p: &[Point], t: f64) -> (Vec<Point>, Vec<Point>) {
    let mut work = p.to_vec();
    let mut left = vec![p[0]];
    let mut right = vec![*p.last().unwrap()];
    for level in (1..p.len()).rev() {
        for i in 0..level {
            let next = work[i + 1];
            for (coordinate, target) in work[i].iter_mut().zip(next) {
                *coordinate = *coordinate + (target - *coordinate) * t;
            }
        }
        left.push(work[0]);
        right.push(work[level - 1]);
    }
    right.reverse();
    (left, right)
}
/// Y extrema of a nonempty x-monotone Bezier segment clipped by two vertical
/// lines. Uses the pinned Order2/Order3 inverse and subdivision operations.
/// This supplies segment extrema, not a general boolean Area intersection.
pub fn monotone_clip_y_bounds(points: &[Point], left: f64, right: f64) -> Result<[f64; 2], String> {
    if !(2..=4).contains(&points.len())
        || points.iter().flatten().any(|v| !v.is_finite())
        || !left.is_finite()
        || !right.is_finite()
        || left >= right
        || points.windows(2).any(|w| w[0][0] > w[1][0])
    {
        return Err("unsupported_area_non_monotone_clip".into());
    }
    let start = points[0][0];
    let end = points.last().unwrap()[0];
    let left = left.max(start);
    let right = right.min(end);
    if left >= right {
        return Err("empty_area_segment_clip".into());
    }
    let c = coefficients(points, 0);
    let parameter = |x: f64| -> Result<f64, String> {
        if x == start {
            return Ok(0.);
        }
        if x == end {
            return Ok(1.);
        }
        match points.len() {
            2 => Ok((x - start) / (end - start)),
            3 => Ok(quadratic_t(x, c)),
            _ => cubic_t(x, c),
        }
    };
    let t0 = parameter(left)?;
    let t1 = parameter(right)?;
    if !(0.0..=1.0).contains(&t0) || !(0.0..=1.0).contains(&t1) || t0 >= t1 {
        return Err("unsupported_area_clip_parameters".into());
    }
    let mut segment = if t1 < 1. {
        split(points, t1).0
    } else {
        points.to_vec()
    };
    if t0 > 0. {
        segment = split(&segment, t0 / t1).1;
    }
    let flat = segment.iter().all(|p| p[1] == segment[0][1]);
    if flat {
        return Ok([segment[0][1], segment[0][1]]);
    }
    Area::from_closed_segments(&[segment])?
        .path_y_bounds()
        .ok_or("empty_area_clip_bounds".into())
}

fn horizontal_roots(p: &[Point]) -> Vec<f64> {
    if p.windows(2).all(|w| w[0][1] <= w[1][1]) {
        return Vec::new();
    }
    let mut roots = match p.len() {
        2 => Vec::new(),
        3 => {
            let a = p[0][1] - p[1][1];
            let b = p[2][1] - p[1][1];
            if a + b == 0. {
                Vec::new()
            } else {
                vec![a / (a + b)]
            }
        }
        _ => {
            let a = p[1][1] - p[0][1];
            let b = p[2][1] - p[1][1];
            let c = p[3][1] - p[2][1];
            quadratic_roots(a, (b - a) * 2., c - b - b + a)
        }
    };
    roots.retain(|t| *t > 0. && *t < 1.);
    roots.sort_by(f64::total_cmp);
    roots
}
fn insert(edges: &mut Vec<Edge>, mut points: Vec<Point>) -> Result<(), String> {
    let direction = if points[0][1] > points.last().unwrap()[1] {
        points.reverse();
        -1
    } else {
        1
    };
    let roots = horizontal_roots(&points);
    let mut current = points;
    let mut previous = 0.;
    for t in roots {
        let (left, right) = split(&current, (t - previous) / (1. - previous));
        add(edges, left, direction)?;
        current = right;
        previous = t;
    }
    add(edges, current, direction)
}
fn add(edges: &mut Vec<Edge>, mut points: Vec<Point>, mut direction: i32) -> Result<(), String> {
    let last = points.len() - 1;
    if points[0][1] == points[last][1] {
        return Ok(());
    }
    if points[0][1] > points[last][1] {
        points.reverse();
        direction = -direction;
    }
    if points.len() == 3 {
        points[1][1] = points[1][1].clamp(points[0][1], points[2][1]);
    }
    if points.len() == 4 {
        points[1][1] = points[1][1].max(points[0][1]);
        points[2][1] = points[2][1].min(points[3][1]);
    }
    let x = coefficients(&points, 0);
    let y = coefficients(&points, 1);
    let xmin = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let xmax = points
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    if edges.len() >= MAX_SEGMENTS || x.iter().chain(y.iter()).any(|v| !v.is_finite()) {
        return Err("unsupported_area_edge_budget".into());
    }
    edges.push(Edge {
        points,
        x,
        y,
        direction,
        xmin,
        xmax,
    });
    Ok(())
}
impl Edge {
    fn crossing(&self, point: Point) -> Result<i32, String> {
        let [x, y] = point;
        let first = self.points[0];
        let last = *self.points.last().unwrap();
        if y < first[1] || y >= last[1] || x >= self.xmax {
            return Ok(0);
        }
        if x < self.xmin {
            return Ok(self.direction);
        }
        let xx = if self.points.len() == 2 {
            if first[0] == last[0] || y <= first[1] {
                first[0]
            } else if y >= last[1] {
                last[0]
            } else {
                first[0] + (y - first[1]) * (last[0] - first[0]) / (last[1] - first[1])
            }
        } else if y <= first[1] {
            first[0]
        } else if y >= last[1] {
            last[0]
        } else {
            let t = if self.points.len() == 3 {
                quadratic_t(y, self.y)
            } else {
                cubic_t(y, self.y)?
            };
            polynomial(self.x, t)
        };
        Ok(if x < xx { self.direction } else { 0 })
    }
}
/// A closed path with nonzero winding, assembled from line/quadratic/cubic
/// Bezier segments. Edges are normalized to Y-monotone pieces once, not per query.
#[derive(Clone, Debug)]
pub struct Area {
    edges: Vec<Edge>,
}
impl Area {
    pub fn from_closed_segments(segments: &[Vec<Point>]) -> Result<Self, String> {
        if segments.is_empty() || segments.len() > MAX_SEGMENTS {
            return Err("invalid_area_segments".into());
        }
        let mut edges = Vec::new();
        let start = segments[0].first().copied().ok_or("invalid_area_segment")?;
        let mut previous = start;
        for segment in segments {
            if !(2..=4).contains(&segment.len())
                || segment
                    .iter()
                    .flatten()
                    .any(|v| !v.is_finite() || v.abs() > 16_000_000.)
            {
                return Err("invalid_area_segment".into());
            }
            if segment[0] != previous {
                insert(&mut edges, vec![previous, segment[0]])?;
            }
            insert(&mut edges, segment.clone())?;
            previous = *segment.last().unwrap();
        }
        if previous != start {
            insert(&mut edges, vec![previous, start])?;
        }
        Ok(Self { edges })
    }
    pub fn contains(&self, point: Point) -> Result<bool, String> {
        if point.iter().any(|v| !v.is_finite()) {
            return Err("invalid_area_point".into());
        }
        let mut crossings = 0;
        for edge in &self.edges {
            crossings += edge.crossing(point)?;
        }
        Ok(crossings != 0)
    }
    /// Y extents of the assembled normalized path edges, before cancellation.
    /// Native non-crossing full-width bands use these for Area.getBounds.
    pub fn path_y_bounds(&self) -> Option<[f64; 2]> {
        if self.edges.is_empty() {
            return None;
        }
        Some([
            self.edges
                .iter()
                .map(|e| e.points[0][1])
                .fold(f64::INFINITY, f64::min),
            self.edges
                .iter()
                .map(|e| e.points.last().unwrap()[1])
                .fold(f64::NEG_INFINITY, f64::max),
        ])
    }
    /// Conservative bounded query cost: at most three 256-step refinements and
    /// one 256-step fallback for each cubic edge, plus crossing/root overhead.
    pub fn query_work(&self) -> usize {
        self.edges.len().saturating_mul(1030).saturating_add(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rectangle_is_half_open_and_auto_closed() {
        let area =
            Area::from_closed_segments(&[vec![[0., 0.], [10., 0.]], vec![[10., 10.], [0., 10.]]])
                .unwrap();
        for p in [[0., 0.], [5., 5.], [0., 9.]] {
            assert!(area.contains(p).unwrap());
        }
        for p in [[10., 0.], [0., 10.], [10., 10.], [-1., 5.], [5., -1.]] {
            assert!(!area.contains(p).unwrap());
        }
    }
    #[test]
    fn sloped_boundary_and_winding_reversal_preserve_crossing_semantics() {
        let paths = vec![vec![[0., 0.], [10., 10.]], vec![[10., 20.], [0., 20.]]];
        let first = Area::from_closed_segments(&paths).unwrap();
        let reverse = paths
            .into_iter()
            .rev()
            .map(|mut p| {
                p.reverse();
                p
            })
            .collect::<Vec<_>>();
        let second = Area::from_closed_segments(&reverse).unwrap();
        for (p, wanted) in [([5., 5.], false), ([5., 6.], true), ([5., 4.], false)] {
            assert_eq!(first.contains(p).unwrap(), wanted);
            assert_eq!(second.contains(p).unwrap(), wanted);
        }
    }
    #[test]
    fn polynomial_degeneracy_reduces_to_original_line_boundary() {
        for top in [
            vec![[0., 0.], [1., 1.], [2., 2.]],
            vec![[0., 0.], [1., 1.], [2., 2.], [3., 3.]],
        ] {
            let end = top.last().unwrap()[0];
            let area = Area::from_closed_segments(&[top, vec![[end, 4.], [0., 4.]]]).unwrap();
            assert!(!area.contains([1., 1.]).unwrap());
            assert!(area.contains([1., 2.]).unwrap());
        }
    }
    #[test]
    fn malformed_paths_and_queries_fail_without_unbounded_work() {
        assert!(Area::from_closed_segments(&[]).is_err());
        assert!(Area::from_closed_segments(&[vec![]]).is_err());
        assert!(Area::from_closed_segments(&[vec![[0., 0.], [f64::NAN, 1.]]]).is_err());
        assert!(Area::from_closed_segments(&[vec![[0., 0.], [16_000_001., 1.]]]).is_err());
        assert!(
            Area::from_closed_segments(&vec![vec![[0., 0.], [1., 1.]]; MAX_SEGMENTS + 1]).is_err()
        );
        let area = Area::from_closed_segments(&[vec![[0., 0.], [1., 1.]]]).unwrap();
        assert!(area.contains([f64::INFINITY, 0.]).is_err());
        assert_eq!(refine([0., 1., 0., 0.], 0.5, f64::NAN), -1.);
    }
}

mod linear;
pub use linear::LinearArea;

mod polygon;
pub use polygon::polygon_intersects_rect;
