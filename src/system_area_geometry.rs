// SPDX-License-Identifier: AGPL-3.0-or-later
// Original StaffManager horiNeighbor/vertNeighbors/getGlobalLine and
// SystemManager verticalNeighbors/computeSystemArea point geometry.
use crate::curves::Curve;
use audiveris_area::Area;
use serde_json::{Value, json};
use std::collections::BTreeMap;
struct Node {
    id: u64,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}
fn horizontal(nodes: &[Node], i: usize, left: bool) -> Option<usize> {
    let overlap = |j: usize| nodes[i].top.max(nodes[j].top) < nodes[i].bottom.min(nodes[j].bottom);
    if left {
        (0..i).rev().find(|&j| overlap(j))
    } else {
        (i + 1..nodes.len()).find(|&j| overlap(j))
    }
}
fn vertical(nodes: &[Node], i: usize, top: bool, remove_self: bool) -> Vec<usize> {
    let overlap = |j: usize| nodes[i].left.max(nodes[j].left) < nodes[i].right.min(nodes[j].right);
    let first = if top {
        (0..i).rev().find(|&j| overlap(j))
    } else {
        (i + 1..nodes.len()).find(|&j| overlap(j))
    };
    let Some(first) = first else {
        return vec![];
    };
    let mut result = vec![first];
    for left in [true, false] {
        let mut n = first;
        while let Some(j) = horizontal(nodes, n, left) {
            result.push(j);
            n = j;
        }
    }
    if remove_self && let Some(n) = result.iter().position(|&n| n == i) {
        result.remove(n);
    }
    result.sort_by_key(|&j| nodes[j].id);
    result
}
/// Exact point containment for original Area intersect Rectangle.
/// Bounds require a non-crossing clip certificate; no general boolean edges.
#[derive(Debug)]
pub struct Band {
    path: Area,
    rectangle_certified: bool,
    ends: [usize; 2],
    width: usize,
    height: usize,
    header_left: Option<usize>,
    bounds: Option<[i32; 4]>,
}
impl Band {
    /// Original integer Area bounds for full-width paths or certified non-crossing straight/curved clips.
    pub fn bounds(&self) -> Result<[i32; 4], String> {
        self.bounds
            .ok_or("unsupported_rest_native_system_area_bounds".into())
    }
    pub fn query_work(&self) -> usize {
        self.path.query_work()
    }
    /// Exact Area.getBounds().x for legacy full-width bands or certified
    /// non-crossing straight/curved clipped bands. No general boolean-bounds claim.
    pub fn header_left(&self) -> Result<usize, String> {
        self.header_left
            .ok_or("unsupported_beam_header_area_bounds".into())
    }

    /// Original Area.intersects(Rectangle) over the retained noncrossing Band.
    pub(crate) fn intersects_rect(&self, mut r: [f64; 4]) -> Result<bool, String> {
        if r.iter().any(|v| !v.is_finite() || v.abs() > 16_000_000.) {
            return Err("invalid_system_area_rectangle".into());
        }
        if r[2] <= 0. || r[3] <= 0. {
            return Ok(false);
        }
        if !self.rectangle_certified {
            return Err("system_area_rectangle_noncrossing_certificate_required".into());
        }
        if self.ends != [0, self.width] {
            let right = (r[0] + r[2]).min(self.ends[1] as f64);
            let bottom = (r[1] + r[3]).min(self.height as f64);
            r[0] = r[0].max(self.ends[0] as f64);
            r[1] = r[1].max(0.);
            r[2] = right - r[0];
            r[3] = bottom - r[1];
        }
        self.path.intersects_rect_for_noncrossing_path(r)
    }

    pub fn contains(&self, p: [f64; 2]) -> Result<bool, String> {
        if p.iter().any(|v| !v.is_finite()) {
            return Err("invalid_system_area_point".into());
        }
        if self.ends != [0, self.width]
            && (p[0] < self.ends[0] as f64
                || p[0] >= self.ends[1] as f64
                || p[1] < 0.
                || p[1] >= self.height as f64)
        {
            return Ok(false);
        }
        self.path.contains(p)
    }
}
pub struct Layout {
    pub system_areas: Vec<Band>,
    pub staff_areas: Vec<Band>,
    pub system_neighbors: Vec<Value>,
    pub staff_neighbors: Vec<Value>,
}
fn boundary(curves: &[&Curve], width: usize, shift: f64, flat: f64) -> Vec<Vec<[f64; 2]>> {
    if curves.is_empty() {
        return vec![vec![[0., flat], [width as f64, flat]]];
    }
    let mut segments = vec![];
    let mut last = [0., curves[0].points[0][1] + shift];
    for c in curves {
        let first = c.points[0];
        let first = [first[0], first[1] + shift];
        segments.push(vec![last, first]);
        segments.extend(
            c.path_segments()
                .iter()
                .map(|s| s.iter().map(|p| [p[0], p[1] + shift]).collect::<Vec<_>>()),
        );
        let p = *c.points.last().unwrap();
        last = [p[0], p[1] + shift];
    }
    segments.push(vec![last, [width as f64, last[1]]]);
    segments
}
// Sufficient certificate: globally separated boundary control hulls guarantee
// positive filled height throughout the clip. No sampling or guessed intersections.
fn curved_clip_bounds(
    top: &[Vec<[f64; 2]>],
    bottom: &[Vec<[f64; 2]>],
    ends: [usize; 2],
    height: usize,
) -> Option<[i32; 4]> {
    let extents = |segments: &[Vec<[f64; 2]>]| -> Option<([f64; 2], [f64; 2])> {
        let mut curve = [f64::INFINITY, f64::NEG_INFINITY];
        let mut hull = [f64::INFINITY, f64::NEG_INFINITY];
        for s in segments {
            if s.windows(2).any(|p| p[0][0] > p[1][0]) {
                return None;
            }
            if s.last()?[0] < ends[0] as f64 || s[0][0] > ends[1] as f64 {
                continue;
            }
            for p in s {
                hull[0] = hull[0].min(p[1]);
                hull[1] = hull[1].max(p[1]);
            }
            if s[0][0] == s.last()?[0] {
                for p in s {
                    curve[0] = curve[0].min(p[1]);
                    curve[1] = curve[1].max(p[1]);
                }
            } else if s[0][0].max(ends[0] as f64) < s.last()?[0].min(ends[1] as f64) {
                let y = audiveris_area::monotone_clip_y_bounds(s, ends[0] as f64, ends[1] as f64)
                    .ok()?;
                curve[0] = curve[0].min(y[0]);
                curve[1] = curve[1].max(y[1]);
            }
        }
        if curve.iter().chain(&hull).any(|v| !v.is_finite()) {
            None
        } else {
            Some((curve, hull))
        }
    };
    let (north, nh) = extents(top)?;
    let (south, sh) = extents(bottom)?;
    if nh[1].max(0.) >= sh[0].min(height as f64) {
        return None;
    }
    let low = north[0].max(0.).floor() as i32;
    let high = south[1].min(height as f64).ceil() as i32;
    Some([ends[0] as i32, low, (ends[1] - ends[0]) as i32, high - low])
}

// Global X monotonicity and separated control hulls certify one simple filled
// band. This avoids applying raw path crossings to general Area boolean geometry.
fn rectangle_path_certificate(
    top: &[Vec<[f64; 2]>],
    bottom: &[Vec<[f64; 2]>],
    width: usize,
) -> bool {
    let monotone = |segments: &[Vec<[f64; 2]>]| {
        !segments.is_empty()
            && segments.first().unwrap().first().unwrap()[0] == 0.
            && segments.last().unwrap().last().unwrap()[0] == width as f64
            && segments
                .iter()
                .all(|s| s.windows(2).all(|p| p[0][0] <= p[1][0]))
            && segments.windows(2).all(|s| s[0].last() == s[1].first())
    };
    monotone(top)
        && monotone(bottom)
        && top
            .iter()
            .flatten()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max)
            < bottom
                .iter()
                .flatten()
                .map(|p| p[1])
                .fold(f64::INFINITY, f64::min)
}

fn band(
    north: &[&Curve],
    south: &[&Curve],
    width: usize,
    height: usize,
    margin: f64,
    ends: [usize; 2],
) -> Result<Band, String> {
    let top = boundary(north, width, margin, 0.);
    let bottom = boundary(south, width, -margin, height as f64);
    let mut header_left = if ends == [0, width] {
        Some(0)
    } else {
        // A simple straight x-monotone band with positive height throughout
        // the clip reaches its left edge. Its integer Area bound is exact.
        let straight = |segments: &[Vec<[f64; 2]>]| {
            segments.iter().all(|s| s.len() == 2 && s[0][0] <= s[1][0])
        };
        if straight(&top) && straight(&bottom) {
            let mut xs = vec![ends[0] as f64, ends[1] as f64];
            xs.extend(
                top.iter()
                    .chain(&bottom)
                    .flat_map(|s| s.iter().map(|p| p[0]))
                    .filter(|x| *x > ends[0] as f64 && *x < ends[1] as f64),
            );
            let ordinate = |segments: &[Vec<[f64; 2]>], x: f64| {
                segments
                    .iter()
                    .find(|s| s[0][0] <= x && x <= s[1][0] && s[0][0] < s[1][0])
                    .map(|s| s[0][1] + (x - s[0][0]) * (s[1][1] - s[0][1]) / (s[1][0] - s[0][0]))
            };
            if xs
                .into_iter()
                .all(|x| match (ordinate(&top, x), ordinate(&bottom, x)) {
                    (Some(t), Some(b)) => t.max(0.) < b.min(height as f64),
                    _ => false,
                })
            {
                Some(ends[0])
            } else {
                None
            }
        } else {
            None
        }
    };
    let mut clip_bounds = if header_left.is_some() && ends != [0, width] {
        let extrema = |segments: &[Vec<[f64; 2]>]| {
            let mut ys = Vec::new();
            for s in segments {
                if s[0][0] == s[1][0] {
                    continue;
                }
                let lo = s[0][0].max(ends[0] as f64);
                let hi = s[1][0].min(ends[1] as f64);
                if lo <= hi {
                    for x in [lo, hi] {
                        ys.push(
                            s[0][1] + (x - s[0][0]) * (s[1][1] - s[0][1]) / (s[1][0] - s[0][0]),
                        );
                    }
                }
            }
            ys
        };
        let low = extrema(&top)
            .into_iter()
            .fold(f64::INFINITY, f64::min)
            .max(0.)
            .floor() as i32;
        let high = extrema(&bottom)
            .into_iter()
            .fold(f64::NEG_INFINITY, f64::max)
            .min(height as f64)
            .ceil() as i32;
        Some([ends[0] as i32, low, (ends[1] - ends[0]) as i32, high - low])
    } else {
        None
    };
    if ends != [0, width] && top.iter().chain(&bottom).any(|s| s.len() > 2) {
        clip_bounds = curved_clip_bounds(&top, &bottom, ends, height);
        header_left = clip_bounds.map(|b| b[0] as usize);
    }
    let rectangle_certified = rectangle_path_certificate(&top, &bottom, width);
    let mut path = top;
    path.extend(bottom.into_iter().rev().map(|mut s| {
        s.reverse();
        s
    }));
    let path = Area::from_closed_segments(&path)?;
    let bounds = if ends == [0, width] {
        path.path_y_bounds().map(|[lo, hi]| {
            let low = lo.floor() as i32;
            [0, low, width as i32, hi.ceil() as i32 - low]
        })
    } else {
        clip_bounds
    };
    Ok(Band {
        path,
        rectangle_certified,
        bounds,
        header_left,
        ends,
        width,
        height,
    })
}
pub fn build(
    grid: &Value,
    width: usize,
    height: usize,
    interline: usize,
) -> Result<Layout, String> {
    if width == 0
        || height == 0
        || width
            .checked_mul(height)
            .is_none_or(|n| n > crate::MAX_PIXELS)
        || !(1..=128).contains(&interline)
    {
        return Err("invalid_system_area_dimensions".into());
    }
    let systems = grid["systems"]
        .as_array()
        .ok_or("invalid_system_area_systems")?;
    let ends = crate::system_area_ends::discover(systems, width)?;
    let staffs = grid["staffs"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("invalid_system_area_staffs")?;
    let (sn, curves, ids) = staff_geometry(staffs)?;
    if sn.iter().any(|n| n.right > width as f64) {
        return Err("invalid_system_area_staff_span".into());
    }
    let mut nodes = Vec::new();
    let mut members = Vec::new();
    let mut owned = Vec::new();
    for s in systems {
        let c = s["coordinates"].as_array().unwrap();
        let id = s["id"].as_u64().unwrap();
        nodes.push(Node {
            id,
            left: c[0].as_i64().unwrap() as f64,
            right: (c[0].as_i64().unwrap() + c[2].as_i64().unwrap() - 1) as f64,
            top: c[1].as_i64().unwrap() as f64,
            bottom: c[3].as_i64().unwrap() as f64,
        });
        let m = s["staff_ids"]
            .as_array()
            .filter(|m| !m.is_empty())
            .ok_or("invalid_system_area_membership")?
            .iter()
            .map(|v| {
                v.as_u64()
                    .and_then(|id| ids.get(&id).copied())
                    .ok_or("missing_system_area_staff")
            })
            .collect::<Result<Vec<_>, _>>()?;
        for &j in &m {
            if staffs[j]["system_id"].as_u64() != Some(id) || owned.contains(&j) {
                return Err("inconsistent_system_area_membership".into());
            }
            owned.push(j);
        }
        members.push(m);
    }
    if owned != (0..staffs.len()).collect::<Vec<_>>() {
        return Err("inconsistent_system_area_staff_order".into());
    }
    let margin = (interline as f64 * 0.9).round_ties_even();
    let mut system_areas = Vec::new();
    let mut system_neighbors = Vec::new();
    let mut staff_areas = Vec::new();
    let mut staff_neighbors = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let north = vertical(&nodes, i, true, false);
        let south = vertical(&nodes, i, false, false);
        let ns = north
            .iter()
            .map(|&j| *members[j].last().unwrap())
            .collect::<Vec<_>>();
        let ss = south.iter().map(|&j| members[j][0]).collect::<Vec<_>>();
        let e = crate::beam_dispatch::area_ends(&ends[i]["x_range"], width)?;
        system_areas.push(band(
            &ns.iter().map(|&j| &curves[j].1).collect::<Vec<_>>(),
            &ss.iter().map(|&j| &curves[j].0).collect::<Vec<_>>(),
            width,
            height,
            margin,
            e,
        )?);
        system_neighbors.push(json!({"id":n.id,"north":north.iter().map(|&j|nodes[j].id).collect::<Vec<_>>(),"south":south.iter().map(|&j|nodes[j].id).collect::<Vec<_>>(),"north_staff_ids":ns.iter().map(|&j|sn[j].id).collect::<Vec<_>>(),"south_staff_ids":ss.iter().map(|&j|sn[j].id).collect::<Vec<_>>(),"x_range":e}));
    }
    for (i, n) in sn.iter().enumerate() {
        let north = vertical(&sn, i, true, true);
        let south = vertical(&sn, i, false, true);
        let sys = members.iter().position(|m| m.contains(&i)).unwrap();
        let e = crate::beam_dispatch::area_ends(&ends[sys]["x_range"], width)?;
        staff_areas.push(band(
            &north.iter().map(|&j| &curves[j].1).collect::<Vec<_>>(),
            &south.iter().map(|&j| &curves[j].0).collect::<Vec<_>>(),
            width,
            height,
            margin,
            e,
        )?);
        staff_neighbors.push(json!({"id":n.id,"left":horizontal(&sn,i,true).map(|j|sn[j].id),"right":horizontal(&sn,i,false).map(|j|sn[j].id),"north":north.iter().map(|&j|sn[j].id).collect::<Vec<_>>(),"south":south.iter().map(|&j|sn[j].id).collect::<Vec<_>>()}));
    }
    Ok(Layout {
        system_areas,
        staff_areas,
        system_neighbors,
        staff_neighbors,
    })
}
type StaffGeometry = (Vec<Node>, Vec<(Curve, Curve)>, BTreeMap<u64, usize>);
fn staff_geometry(staffs: &[Value]) -> Result<StaffGeometry, String> {
    let mut curves = Vec::new();
    let mut sn = Vec::new();
    let mut ids = BTreeMap::new();
    let mut budget = 0;
    for (i, s) in staffs.iter().enumerate() {
        let id = s["id"].as_u64().ok_or("invalid_system_area_staff_id")?;
        if ids.insert(id, i).is_some() {
            return Err("duplicate_system_area_staff".into());
        }
        let lines = s["lines"]
            .as_array()
            .filter(|l| l.len() == 5)
            .ok_or("unsupported_system_area_staff_lines")?;
        let top = crate::beam_dispatch::curve(&lines[0], &mut budget)?;
        let bottom = crate::beam_dispatch::curve(&lines[4], &mut budget)?;
        let left = s["left"]
            .as_f64()
            .filter(|n| n.is_finite() && n.abs() <= 4_000_000.)
            .ok_or("invalid_system_area_staff_span")?
            .round_ties_even() as i64;
        let right = s["right"]
            .as_f64()
            .filter(|n| n.is_finite() && n.abs() <= 4_000_000.)
            .ok_or("invalid_system_area_staff_span")?
            .round_ties_even() as i64;
        if left < 0 || right <= left || right > 4_000_000 || top.points[0][1] >= bottom.points[0][1]
        {
            return Err("invalid_system_area_staff_span".into());
        }
        sn.push(Node {
            id,
            left: left as f64,
            right: right as f64,
            top: top.points[0][1],
            bottom: bottom.points[0][1],
        });
        curves.push((top, bottom));
    }
    Ok((sn, curves, ids))
}
/// StaffManager neighbors before PeakGraph creates systems. Uses actual line
/// LEFT endpoint spans and integer Staff abscissas; no short flag heuristic.
pub fn staff_neighbors(staffs: &[Value]) -> Result<Vec<Value>, String> {
    if staffs.is_empty() || staffs.len() > 1024 {
        return Err("invalid_system_area_staffs".into());
    }
    let (nodes, _, _) = staff_geometry(staffs)?;
    Ok(nodes.iter().enumerate().map(|(i,n)|json!({"id":n.id,"left":horizontal(&nodes,i,true).map(|j|nodes[j].id),"right":horizontal(&nodes,i,false).map(|j|nodes[j].id),"north":vertical(&nodes,i,true,true).iter().map(|&j|nodes[j].id).collect::<Vec<_>>(),"south":vertical(&nodes,i,false,true).iter().map(|&j|nodes[j].id).collect::<Vec<_>>()})).collect())
}
