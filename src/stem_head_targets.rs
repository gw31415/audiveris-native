// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HeadLinker beam-group lookup and CLinker target choice.
//! Neighborhoods and complete beam-group membership are declared inputs.
//! This does not discover image candidates or update cache/SIG.
use crate::stem_lookup_area::{self, Geometry, HeadRequest};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Beam {
    pub id: usize,
    pub group: Option<usize>,
    pub median: [f64; 4],
    pub top: [f64; 4],
    pub bottom: [f64; 4],
    pub height: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: usize,
    pub members: Vec<usize>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub head: HeadRequest,
    pub min_distance: i32,
    pub small_head: bool,
    pub candidates: Vec<usize>,
}
#[derive(Debug, Serialize)]
pub struct Lookup {
    pub selected_beams: Vec<usize>,
    pub selected_groups: Vec<usize>,
}
#[derive(Debug, Serialize)]
pub struct Selected {
    pub selected_beams: Vec<usize>,
    pub selected_groups: Vec<usize>,
    pub target_beam: Option<usize>,
    pub initial_line: [f64; 4],
    pub area: Geometry,
    pub target: [f64; 2],
}
pub struct Catalog<'a> {
    beams: BTreeMap<usize, &'a Beam>,
    groups: BTreeMap<usize, &'a Group>,
}
fn finite(v: f64) -> bool {
    v.is_finite() && v.abs() <= 4_000_000.
}
fn line_ok(l: [f64; 4]) -> bool {
    l.iter().all(|v| finite(*v)) && l[0] != l[2]
}
fn intersection(a: [f64; 4], b: [f64; 4]) -> [f64; 2] {
    let [x1, y1, x2, y2] = a;
    let [x3, y3, x4, y4] = b;
    let d = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let p = x1 * y2 - y1 * x2;
    let q = x3 * y4 - y3 * x4;
    [
        (p * (x3 - x4) - (x1 - x2) * q) / d,
        (p * (y3 - y4) - (y1 - y2) * q) / d,
    ]
}
fn near(b: &Beam, dir: i8) -> [f64; 4] {
    if dir > 0 { b.top } else { b.bottom }
}
fn far(b: &Beam, dir: i8) -> [f64; 4] {
    if dir > 0 { b.bottom } else { b.top }
}
fn hit(reference: [f64; 2], line: [f64; 4]) -> [f64; 2] {
    intersection(
        [
            reference[0],
            reference[1],
            reference[0],
            reference[1] + 100.,
        ],
        line,
    )
}
// java.awt.geom.Line2D.relativeCCW, including collinear segment endpoints.
fn ccw(l: [f64; 4], p: [f64; 2]) -> i32 {
    let dx = l[2] - l[0];
    let dy = l[3] - l[1];
    let mut px = p[0] - l[0];
    let mut py = p[1] - l[1];
    let mut c = px * dy - py * dx;
    if c == 0. {
        c = px * dx + py * dy;
        if c > 0. {
            px -= dx;
            py -= dy;
            c = px * dx + py * dy;
            if c < 0. {
                c = 0.;
            }
        }
    }
    if c < 0. {
        -1
    } else if c > 0. {
        1
    } else {
        0
    }
}
fn intersects(a: [f64; 4], b: [f64; 4]) -> bool {
    ccw(a, [b[0], b[1]]) * ccw(a, [b[2], b[3]]) <= 0
        && ccw(b, [a[0], a[1]]) * ccw(b, [a[2], a[3]]) <= 0
}
impl<'a> Catalog<'a> {
    pub fn new(beams: &'a [Beam], groups: &'a [Group]) -> Result<Self, String> {
        if beams.len() > 4096 || groups.len() > 4096 {
            return Err("target catalog budget".into());
        }
        let mut bmap = BTreeMap::new();
        let mut gmap = BTreeMap::new();
        for b in beams {
            if b.id == 0
                || b.group == Some(0)
                || bmap.insert(b.id, b).is_some()
                || !line_ok(b.median)
                || !line_ok(b.top)
                || !line_ok(b.bottom)
                || !b.height.is_finite()
                || !(0.0..=1024.).contains(&b.height)
            {
                return Err("invalid target beam catalog".into());
            }
        }
        let mut members = BTreeSet::new();
        for g in groups {
            if g.id == 0
                || g.members.is_empty()
                || g.members.len() > 4096
                || gmap.insert(g.id, g).is_some()
            {
                return Err("invalid target group catalog".into());
            }
            for &id in &g.members {
                let b = bmap.get(&id).ok_or("missing target group member")?;
                if b.group != Some(g.id) || !members.insert(id) {
                    return Err("inconsistent target group membership".into());
                }
            }
        }
        for b in beams {
            if b.group.is_some() && !members.contains(&b.id) {
                return Err("incomplete target group membership".into());
            }
        }
        Ok(Self {
            beams: bmap,
            groups: gmap,
        })
    }
    pub fn lookup(
        &self,
        reference: [f64; 2],
        direction: i8,
        min_distance: i32,
        candidates: &[usize],
        skew: f64,
    ) -> Result<Lookup, String> {
        if ![-1, 1].contains(&direction)
            || skew != 0.
            || !reference.iter().all(|v| finite(*v))
            || !(0..=1_000_000).contains(&min_distance)
            || candidates.len() > 4096
        {
            return Err("unsupported target lookup input".into());
        }
        let mut ordered = Vec::new();
        for &id in candidates {
            let b = self.beams.get(&id).ok_or("unknown target candidate")?;
            let d = direction as f64 * (hit(reference, near(b, direction))[1] - reference[1]);
            if !finite(d) {
                return Err("unsupported target intersection".into());
            }
            if d > 0. {
                let key = direction as f64 * (hit(reference, far(b, direction))[1] - reference[1]);
                if !finite(key) {
                    return Err("unsupported target sort intersection".into());
                }
                ordered.push((id, key));
            }
        }
        // Comparator is a single finite distance key: stable ordering retains duplicate/tied beams.
        ordered.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut selected_groups = Vec::new();
        let mut seen = BTreeSet::new();
        for &(id, _) in &ordered {
            let d = direction as f64
                * (hit(reference, near(self.beams[&id], direction))[1] - reference[1]);
            if selected_groups.is_empty() && d < min_distance as f64 {
                continue;
            }
            let group = self.beams[&id]
                .group
                .ok_or("unsupported ungrouped target beam")?;
            if seen.insert(group) {
                selected_groups.push(group);
            }
        }
        Ok(Lookup {
            selected_beams: ordered.into_iter().map(|(id, _)| id).collect(),
            selected_groups,
        })
    }
    pub fn select(&self, r: &Request) -> Result<Selected, String> {
        if r.head.limit.is_some() {
            return Err("target selection requires initial part limit".into());
        }
        let mut area = stem_lookup_area::head(&r.head)?;
        let initial_line = area.theoretical_line;
        let lookup = self.lookup(
            r.head.reference,
            r.head.y_direction,
            r.min_distance,
            &r.candidates,
            r.head.slope,
        )?;
        let mut target = [initial_line[2], initial_line[3]];
        let mut target_beam = None;
        for &id in &lookup.selected_groups {
            let mut members = self.groups[&id].members.clone();
            members.sort_by(|a, b| {
                let dy = |id: &usize| {
                    r.head.y_direction as f64
                        * (hit(r.head.reference, far(self.beams[id], r.head.y_direction))[1]
                            - r.head.reference[1])
                };
                dy(a).total_cmp(&dy(b))
            });
            if !members
                .iter()
                .any(|id| intersects(self.beams[id].median, initial_line))
            {
                continue;
            }
            let beam = self.beams[if r.small_head {
                &members[0]
            } else {
                members.last().unwrap()
            }];
            let mut limit = if r.small_head {
                near(beam, r.head.y_direction)
            } else {
                if r.head.y_direction > 0 {
                    beam.bottom
                } else {
                    beam.top
                }
            };
            target = hit(r.head.reference, limit);
            if !r.small_head {
                target_beam = Some(beam.id);
                limit[1] += r.head.y_direction as f64 * beam.height;
                limit[3] += r.head.y_direction as f64 * beam.height;
            }
            let mut head = r.head.clone();
            head.limit = Some(limit);
            area = stem_lookup_area::head(&head)?;
            break;
        }
        if !target.iter().all(|v| finite(*v)) {
            return Err("unsupported target result".into());
        }
        Ok(Selected {
            selected_beams: lookup.selected_beams,
            selected_groups: lookup.selected_groups,
            target_beam,
            initial_line,
            area,
            target,
        })
    }
}
