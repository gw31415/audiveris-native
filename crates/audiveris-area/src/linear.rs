// Copyright (c) 1998, 2022, Oracle and/or its affiliates. All rights reserved.
// Rust translation and bounded assembly: 2026 audiveris-native contributors.
// SPDX-License-Identifier: GPL-2.0-only WITH Classpath-exception-2.0
// Derived from OpenJDK AreaOp, Order1, Edge, CurveLink, ChainEnd,
// Area.getCachedBounds/contains and Rectangle2D/RectangularShape bounds.
// The Classpath exception is retained for this modified independent library.
// Distributed WITHOUT ANY WARRANTY; see LICENSE and NOTICE.
use crate::Point;
const MAX_EDGES: usize = 128;
// Stop before the original 1024-link intermediate-consumption branch;
// BeamItem parallelograms need only a small fraction of this bound.
const MAX_LINKS: usize = 1024;
#[derive(Clone, Copy, Debug)]
struct Line {
    a: Point,
    b: Point,
    direction: i32,
}
impl Line {
    fn x(&self, y: f64) -> f64 {
        if self.a[0] == self.b[0] || y <= self.a[1] {
            self.a[0]
        } else if y >= self.b[1] {
            self.b[0]
        } else {
            self.a[0] + (y - self.a[1]) * (self.b[0] - self.a[0]) / (self.b[1] - self.a[1])
        }
    }
    fn sub(&self, top: f64, bottom: f64, direction: i32) -> Self {
        if top == self.a[1] && bottom == self.b[1] {
            return Self { direction, ..*self };
        }
        if self.a[0] == self.b[0] {
            return Self {
                a: [self.a[0], top],
                b: [self.b[0], bottom],
                direction,
            };
        }
        let num = self.a[0] - self.b[0];
        let den = self.a[1] - self.b[1];
        Self {
            a: [self.a[0] + (top - self.a[1]) * num / den, top],
            b: [self.a[0] + (bottom - self.a[1]) * num / den, bottom],
            direction,
        }
    }
    fn compare(&self, other: &Self, yrange: &mut [f64; 2]) -> Result<i32, String> {
        yrange[1] = yrange[1].min(self.b[1]).min(other.b[1]);
        if yrange[1] <= yrange[0] {
            return Err("linear_area_backstep".into());
        }
        let xmin = self.a[0].min(self.b[0]);
        let xmax = self.a[0].max(self.b[0]);
        let oxmin = other.a[0].min(other.b[0]);
        let oxmax = other.a[0].max(other.b[0]);
        if xmax <= oxmin {
            return Ok(if xmin == oxmax { 0 } else { -1 });
        }
        if xmin >= oxmax {
            return Ok(1);
        }
        let dxa = self.b[0] - self.a[0];
        let dya = self.b[1] - self.a[1];
        let dxb = other.b[0] - other.a[0];
        let dyb = other.b[1] - other.a[1];
        let den = dxb * dya - dxa * dyb;
        let y = if den != 0.0 {
            let num = (self.a[0] - other.a[0]) * dya * dyb - self.a[1] * dxa * dyb
                + other.a[1] * dxb * dya;
            let y = num / den;
            if y <= yrange[0] {
                self.b[1].min(other.b[1])
            } else {
                if y < yrange[1] {
                    yrange[1] = y;
                }
                self.a[1].max(other.a[1])
            }
        } else {
            self.a[1].max(other.a[1])
        };
        Ok(order(self.x(y), other.x(y)))
    }
}
fn order(a: f64, b: f64) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}
#[derive(Clone, Debug)]
struct Edge {
    line: Line,
    tag: usize,
    etag: i32,
    active_y: f64,
    equivalent: usize,
    last: Option<usize>,
    last_result: i32,
    last_limit: f64,
}
impl Edge {
    fn new(line: Line, tag: usize) -> Self {
        Self {
            line,
            tag,
            etag: 0,
            active_y: 0.,
            equivalent: 0,
            last: None,
            last_result: 0,
            last_limit: 0.,
        }
    }
}
fn compare(edges: &mut [Edge], a: usize, b: usize, range: &mut [f64; 2]) -> Result<i32, String> {
    if edges[a].last == Some(b) && range[0] < edges[a].last_limit {
        range[1] = range[1].min(edges[a].last_limit);
        return Ok(edges[a].last_result);
    }
    if edges[b].last == Some(a) && range[0] < edges[b].last_limit {
        range[1] = range[1].min(edges[b].last_limit);
        return Ok(-edges[b].last_result);
    }
    let result = edges[a].line.compare(&edges[b].line, range)?;
    edges[a].last = Some(b);
    edges[a].last_result = result;
    edges[a].last_limit = range[1];
    Ok(result)
}
#[derive(Clone, Debug)]
struct Link {
    edge: usize,
    top: f64,
    bottom: f64,
    etag: i32,
    next: Option<usize>,
}
#[derive(Clone, Debug)]
struct Chain {
    head: usize,
    tail: usize,
    partner: usize,
    etag: i32,
}
struct Chains {
    links: Vec<Link>,
    ends: Vec<Chain>,
    active: Vec<usize>,
    complete: Vec<usize>,
}
impl Chains {
    fn new() -> Self {
        Self {
            links: vec![],
            ends: vec![],
            active: vec![],
            complete: vec![],
        }
    }
    fn x(&self, edges: &[Edge], id: usize) -> f64 {
        let end = &self.ends[id];
        let l = &self.links[if end.etag == 1 { end.tail } else { end.head }];
        edges[l.edge].line.x(l.bottom)
    }
    fn link_to(&mut self, a: usize, b: usize) -> Result<(), String> {
        if self.ends[a].etag == 0
            || self.ends[b].etag == 0
            || self.ends[a].etag == self.ends[b].etag
        {
            return Err("linear_area_chain_state".into());
        }
        let (enter, exit) = if self.ends[a].etag == 1 {
            (a, b)
        } else {
            (b, a)
        };
        self.ends[a].etag = 0;
        self.ends[b].etag = 0;
        self.links[self.ends[enter].tail].next = Some(self.ends[exit].head);
        self.ends[enter].tail = self.ends[exit].tail;
        if self.ends[a].partner == b {
            self.complete.push(self.ends[enter].head);
            return Ok(());
        }
        let other_enter = self.ends[exit].partner;
        let other_exit = self.ends[enter].partner;
        self.ends[other_enter].partner = other_exit;
        self.ends[other_exit].partner = other_enter;
        if self.links[self.ends[enter].head].top < self.links[self.ends[other_enter].head].top {
            self.links[self.ends[enter].tail].next = Some(self.ends[other_enter].head);
            self.ends[other_enter].head = self.ends[enter].head;
        } else {
            self.links[self.ends[other_exit].tail].next = Some(self.ends[enter].head);
            self.ends[other_exit].tail = self.ends[enter].tail;
        }
        Ok(())
    }
    fn finalize(&mut self) -> Result<(), String> {
        if !self.active.len().is_multiple_of(2) {
            return Err("linear_area_odd_chains".into());
        }
        let active = std::mem::take(&mut self.active);
        for pair in active.as_chunks::<2>().0 {
            self.link_to(pair[0], pair[1])?;
        }
        Ok(())
    }
    fn resolve(&mut self, edges: &[Edge], new_links: Vec<usize>) -> Result<(), String> {
        if !new_links.len().is_multiple_of(2) || !self.active.len().is_multiple_of(2) {
            return Err("linear_area_odd_links".into());
        }
        let old = std::mem::take(&mut self.active);
        let mut c = 0;
        let mut l = 0;
        while c < old.len() || l < new_links.len() {
            let mut connect_chains = l == new_links.len();
            let mut connect_links = c == old.len();
            if !connect_chains && !connect_links {
                let cx = self.x(edges, old[c]);
                let link = &self.links[new_links[l]];
                let lx = edges[link.edge].line.x(link.top);
                connect_chains = c % 2 == 0 && cx == self.x(edges, old[c + 1]);
                connect_links = l % 2 == 0
                    && lx == {
                        let n = &self.links[new_links[l + 1]];
                        edges[n.edge].line.x(n.top)
                    };
                if !connect_chains && !connect_links {
                    connect_chains =
                        c + 1 < old.len() && cx < lx && obstructs(self.x(edges, old[c + 1]), lx, c);
                    connect_links = l + 1 < new_links.len() && lx < cx && {
                        let n = &self.links[new_links[l + 1]];
                        obstructs(edges[n.edge].line.x(n.top), cx, l)
                    };
                }
            }
            if connect_chains {
                self.link_to(old[c], old[c + 1])?;
                c += 2;
            }
            if connect_links {
                let i = self.ends.len();
                let first = new_links[l];
                let second = new_links[l + 1];
                self.ends.push(Chain {
                    head: first,
                    tail: first,
                    partner: i + 1,
                    etag: self.links[first].etag,
                });
                self.ends.push(Chain {
                    head: second,
                    tail: second,
                    partner: i,
                    etag: self.links[second].etag,
                });
                self.active.extend([i, i + 1]);
                l += 2;
            }
            if !connect_chains && !connect_links {
                let id = old[c];
                let link = new_links[l];
                if self.ends[id].etag == 1 {
                    self.links[self.ends[id].tail].next = Some(link);
                    self.ends[id].tail = link;
                } else {
                    self.links[link].next = Some(self.ends[id].head);
                    self.ends[id].head = link;
                }
                self.active.push(id);
                c += 1;
                l += 1;
            }
        }
        Ok(())
    }
    fn result(mut self, edges: &[Edge]) -> Result<LinearArea, String> {
        self.finalize()?;
        let mut result = LinearArea {
            lines: vec![],
            cached_bounds: [0.; 4],
        };
        let mut initialized = false;
        for head in self.complete {
            let mut link = self.links[head].clone();
            // AreaOp emits an Order0 before each completed contour. Its point
            // comes from XforY before subcurve arithmetic and must be retained
            // for Area.getCachedBounds' sequential Rectangle2D.add rounding.
            let move_to = [edges[link.edge].line.x(link.top), link.top];
            if initialized {
                result.add_bounds_point(move_to);
            } else {
                result.cached_bounds = [move_to[0], move_to[1], 0., 0.];
                initialized = true;
            }
            let mut next = link.next;
            let mut steps = 0;
            while let Some(i) = next {
                steps += 1;
                if steps > MAX_LINKS {
                    return Err("linear_area_link_cycle".into());
                }
                let n = &self.links[i];
                if link.edge == n.edge
                    && link.etag == n.etag
                    && link.bottom >= n.top
                    && link.top <= n.bottom
                {
                    link.top = link.top.min(n.top);
                    link.bottom = link.bottom.max(n.bottom);
                } else {
                    result.add_line(edges[link.edge].line.sub(link.top, link.bottom, link.etag));
                    link = n.clone();
                }
                next = n.next;
            }
            result.add_line(edges[link.edge].line.sub(link.top, link.bottom, link.etag));
        }
        Ok(result)
    }
}
fn obstructs(a: f64, b: f64, phase: usize) -> bool {
    if phase.is_multiple_of(2) {
        a <= b
    } else {
        a < b
    }
}
#[derive(Clone, Copy)]
enum Op {
    Winding,
    Subtract,
}
struct Classifier {
    op: Op,
    count: i32,
    inside: [bool; 2],
    result: bool,
}
impl Classifier {
    fn classify(&mut self, e: &Edge) -> i32 {
        match self.op {
            Op::Winding => {
                let tag = if self.count == 0 { 1 } else { 0 };
                self.count += e.line.direction;
                self.result = self.count != 0;
                if self.count == 0 { -1 } else { tag }
            }
            Op::Subtract => {
                self.inside[e.tag] = !self.inside[e.tag];
                let result = self.inside[0] && !self.inside[1];
                if result == self.result {
                    0
                } else {
                    self.result = result;
                    if result { 1 } else { -1 }
                }
            }
        }
    }
}
fn calculate(mut edges: Vec<Edge>, op: Op) -> Result<LinearArea, String> {
    if edges.len() > MAX_EDGES {
        return Err("linear_area_edge_budget".into());
    }
    if edges.len() < 2 {
        return Ok(LinearArea {
            lines: vec![],
            cached_bounds: [0.; 4],
        });
    }
    let mut list: Vec<_> = (0..edges.len()).collect();
    list.sort_by(|a, b| {
        edges[*a].line.a[1]
            .partial_cmp(&edges[*b].line.a[1])
            .unwrap()
            .then_with(|| {
                edges[*a].line.a[0]
                    .partial_cmp(&edges[*b].line.a[0])
                    .unwrap()
            })
    });
    let n = list.len();
    let (mut left, mut right) = (0, 0);
    let mut range = [0., 0.];
    let mut chains = Chains::new();
    let mut iterations = 0;
    while left < n {
        iterations += 1;
        if iterations > MAX_LINKS {
            return Err("linear_area_row_budget".into());
        }
        let mut y = range[0];
        let mut next = right;
        for cur in (left..right).rev() {
            let e = list[cur];
            if edges[e].line.b[1] > y {
                next -= 1;
                list[next] = e;
            }
        }
        left = next;
        if left >= right {
            if right >= n {
                break;
            }
            y = edges[list[right]].line.a[1];
            if y > range[0] {
                chains.finalize()?;
            }
            range[0] = y;
        }
        while right < n && edges[list[right]].line.a[1] <= y {
            right += 1;
        }
        range[1] = edges[list[left]].line.b[1];
        if right < n {
            range[1] = range[1].min(edges[list[right]].line.a[1]);
        }
        let mut next_eq = 1;
        for cur in left..right {
            let e = list[cur];
            edges[e].equivalent = 0;
            let mut next = cur;
            while next > left {
                let prev = list[next - 1];
                let ordering = compare(&mut edges, e, prev, &mut range)?;
                if range[1] <= range[0] {
                    return Err("linear_area_backstep".into());
                }
                if ordering >= 0 {
                    if ordering == 0 {
                        let mut eq = edges[prev].equivalent;
                        if eq == 0 {
                            eq = next_eq;
                            next_eq += 1;
                            edges[prev].equivalent = eq;
                        }
                        edges[e].equivalent = eq;
                    }
                    break;
                }
                list[next] = prev;
                next -= 1;
            }
            list[next] = e;
        }
        let mut classifier = Classifier {
            op,
            count: 0,
            inside: [false; 2],
            result: false,
        };
        let mut new_links = vec![];
        let mut cur = left;
        while cur < right {
            let mut e = list[cur];
            let mut tag;
            let eq = edges[e].equivalent;
            if eq != 0 {
                let original = classifier.result;
                tag = if original { -1 } else { 1 };
                let mut active = None;
                let mut longest = e;
                let mut furthest = range[1];
                loop {
                    classifier.classify(&edges[e]);
                    if active.is_none() && edges[e].etag == tag && edges[e].active_y >= range[0] {
                        active = Some(e);
                    }
                    if edges[e].line.b[1] > furthest {
                        longest = e;
                        furthest = edges[e].line.b[1];
                    }
                    cur += 1;
                    if cur >= right || edges[list[cur]].equivalent != eq {
                        break;
                    }
                    e = list[cur];
                }
                if classifier.result == original {
                    tag = 0;
                } else {
                    e = active.unwrap_or(longest);
                }
            } else {
                tag = classifier.classify(&edges[e]);
                cur += 1;
            }
            if tag != 0 {
                if chains.links.len() >= MAX_LINKS {
                    return Err("linear_area_link_budget".into());
                }
                edges[e].etag = tag;
                edges[e].active_y = range[1];
                new_links.push(chains.links.len());
                chains.links.push(Link {
                    edge: e,
                    top: range[0],
                    bottom: range[1],
                    etag: tag,
                    next: None,
                });
            }
        }
        if classifier.result {
            return Err("linear_area_unbalanced".into());
        }
        chains.resolve(&edges, new_links)?;
        range[0] = range[1];
    }
    chains.result(&edges)
}
/// Bounded OpenJDK line-only Area normalization and subtraction. This API rejects
/// curves; use Area for general path containment. CAG clipping is essential for
/// exact integer-boundary membership, even for a parallelogram belt.
#[derive(Clone, Debug)]
pub struct LinearArea {
    lines: Vec<Line>,
    // Original Rectangle2D.Double [x, y, width, height], accumulated in
    // emitted contour order. Replacing it with extrema changes raster cells.
    cached_bounds: [f64; 4],
}
impl LinearArea {
    fn add_bounds_point(&mut self, [x, y]: Point) {
        let [rx, ry, width, height] = self.cached_bounds;
        let min_x = rx.min(x);
        let max_x = (rx + width).max(x);
        let min_y = ry.min(y);
        let max_y = (ry + height).max(y);
        self.cached_bounds = [min_x, min_y, max_x - min_x, max_y - min_y];
    }
    fn add_line(&mut self, line: Line) {
        // Order1.enlarge always visits the top endpoint before the bottom,
        // regardless of the contour traversal direction.
        self.add_bounds_point(line.a);
        self.add_bounds_point(line.b);
        self.lines.push(line);
    }
    pub fn polygon(points: &[Point]) -> Result<Self, String> {
        if points.len() < 3
            || points.len() > MAX_EDGES
            || points
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 16_000_000.)
        {
            return Err("invalid_linear_area_polygon".into());
        }
        let mut edges = vec![];
        for i in 0..points.len() {
            let (mut a, mut b) = (points[i], points[(i + 1) % points.len()]);
            if a[1] == b[1] {
                continue;
            }
            let direction = if a[1] > b[1] {
                std::mem::swap(&mut a, &mut b);
                -1
            } else {
                1
            };
            edges.push(Edge::new(Line { a, b, direction }, 0));
        }
        calculate(edges, Op::Winding)
    }
    pub fn subtract(&self, other: &Self) -> Result<Self, String> {
        let edges = self
            .lines
            .iter()
            .map(|l| Edge::new(*l, 0))
            .chain(other.lines.iter().map(|l| Edge::new(*l, 1)))
            .collect();
        calculate(edges, Op::Subtract)
    }
    pub fn contains(&self, [x, y]: Point) -> Result<bool, String> {
        if !x.is_finite() || !y.is_finite() {
            return Err("invalid_area_point".into());
        }
        let [rx, ry, width, height] = self.cached_bounds;
        if !(x >= rx && y >= ry && x < rx + width && y < ry + height) {
            return Ok(false);
        }
        let mut crossings = 0;
        for line in &self.lines {
            if y < line.a[1] || y >= line.b[1] || x >= line.a[0].max(line.b[0]) {
                continue;
            }
            if x < line.a[0].min(line.b[0]) || x < line.x(y) {
                crossings += 1;
            }
        }
        Ok(crossings % 2 == 1)
    }
    pub fn bounds(&self) -> [i32; 4] {
        let [rx, ry, width, height] = self.cached_bounds;
        let x = rx.floor();
        let y = ry.floor();
        [
            x as i32,
            y as i32,
            ((rx + width).ceil() - x) as i32,
            ((ry + height).ceil() - y) as i32,
        ]
    }
    /// OpenJDK Area.intersects / Order1.accumulateCrossings for line-only areas.
    /// Boundary-only contact is excluded; recorded left-side intervals use parity.
    pub fn intersects_rect(&self, [x, y, w, h]: [f64; 4]) -> Result<bool, String> {
        if [x, y, w, h]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 16_000_000.)
        {
            return Err("invalid_linear_area_rectangle".into());
        }
        let [rx, ry, rw, rh] = self.cached_bounds;
        if w <= 0.
            || h <= 0.
            || rw <= 0.
            || rh <= 0.
            || x + w <= rx
            || y + h <= ry
            || x >= rx + rw
            || y >= ry + rh
        {
            return Ok(false);
        }
        let mut endpoints = Vec::new();
        for line in &self.lines {
            if line.a[0].min(line.b[0]) >= x + w || line.b[1] <= y || line.a[1] >= y + h {
                continue;
            }
            let ys = line.a[1].max(y);
            let ye = line.b[1].min(y + h);
            let xs = line.x(ys);
            let xe = line.x(ye);
            if xs >= x + w && xe >= x + w {
                continue;
            }
            if xs > x || xe > x {
                return Ok(true);
            }
            endpoints.extend([ys, ye]);
        }
        endpoints.sort_by(f64::total_cmp);
        // XOR of interval endpoints: a nonempty pair means nonempty crossings.
        Ok(endpoints.as_chunks::<2>().0.iter().any(|p| p[0] < p[1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rectangle_contact_and_hole_intersections_are_distinct() {
        let outer = LinearArea::polygon(&[[0., 0.], [10., 0.], [10., 10.], [0., 10.]]).unwrap();
        let inner = LinearArea::polygon(&[[2., 2.], [8., 2.], [8., 8.], [2., 8.]]).unwrap();
        let belt = outer.subtract(&inner).unwrap();
        for r in [
            [10., 0., 1., 10.],
            [-1., 0., 1., 10.],
            [0., 10., 10., 1.],
            [3., 3., 2., 2.],
            [0., 0., 0., 10.],
        ] {
            assert!(!belt.intersects_rect(r).unwrap());
        }
        for r in [
            [0., 0., 1., 1.],
            [1., 3., 2., 2.],
            [-1., -1., 12., 12.],
            [8., 3., 1., 1.],
        ] {
            assert!(belt.intersects_rect(r).unwrap());
        }
        assert!(belt.intersects_rect([f64::NAN, 0., 1., 1.]).is_err());
    }
    #[test]
    fn rectangle_difference_retains_half_open_hole() {
        let outer = LinearArea::polygon(&[[0., 0.], [10., 0.], [10., 10.], [0., 10.]]).unwrap();
        let inner = LinearArea::polygon(&[[2., 2.], [8., 2.], [8., 8.], [2., 8.]]).unwrap();
        let belt = outer.subtract(&inner).unwrap();
        assert_eq!(belt.bounds(), [0, 0, 10, 10]);
        for p in [[0., 0.], [2., 1.], [8., 2.], [2., 8.], [9., 9.]] {
            assert!(belt.contains(p).unwrap());
        }
        for p in [[2., 2.], [7., 7.], [10., 1.], [-1., 1.]] {
            assert!(!belt.contains(p).unwrap());
        }
        assert_eq!(outer.subtract(&outer).unwrap().bounds(), [0; 4]);
    }
    #[test]
    fn resource_and_nonfinite_inputs_fail_closed() {
        assert!(LinearArea::polygon(&[[0., 0.]; 129]).is_err());
        assert!(LinearArea::polygon(&[[0., 0.], [1., f64::NAN], [1., 1.]]).is_err());
        assert!(LinearArea::polygon(&[[0., 0.], [1., 16_000_001.], [1., 1.]]).is_err());
        let area = LinearArea::polygon(&[[0., 0.], [1., 0.], [1., 1.]]).unwrap();
        assert!(area.contains([f64::INFINITY, 0.]).is_err());
    }
}
