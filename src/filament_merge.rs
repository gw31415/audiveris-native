// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// Horizontal default FilamentFactory merge/expand pipeline; before GRID slope/curvature purges.
use crate::{
    curves::{self, Curve},
    filaments,
    scale::Scale,
    sections::{Bounds, Section},
};
use serde_json::{Value, json};
#[derive(Clone)]
struct Fil {
    members: Vec<usize>,
    parent: Option<usize>,
    curve: Option<Curve>,
    bounds: Bounds,
}
impl Fil {
    fn new(i: usize, s: &[Section], il: usize) -> Self {
        let members = vec![i];
        Self {
            curve: curves::compute(s, &members, il),
            bounds: s[i].bounds(),
            members,
            parent: None,
        }
    }
    fn weight(&self, s: &[Section]) -> usize {
        self.members.iter().map(|&i| s[i].weight).sum()
    }
}
struct Params {
    gap: i64,
    pos: i64,
    overlap: i64,
    thick: i64,
    space: i64,
    expansion: i64,
    involving: i64,
    pos_slope: i64,
    long: i64,
    probe: i64,
}
impl Params {
    fn new(sc: &Scale) -> Self {
        let px = |v: f64| (v * sc.interline.main as f64).round_ties_even() as i64;
        Self {
            gap: px(1.7),
            pos: (sc.line.main as f64 * 0.75).round_ties_even() as i64,
            overlap: px(0.25),
            thick: (sc.line.main as f64 * 1.5).round_ties_even() as i64,
            space: px(0.16),
            expansion: px(0.02),
            involving: px(2.),
            pos_slope: px(0.1),
            long: px(10.),
            probe: px(0.5) / 2,
        }
    }
}
fn intersects(a: Bounds, b: Bounds, dx: i64, dy: i64) -> bool {
    (a.x as i64 - dx) < (b.x + b.width) as i64
        && (b.x as i64) < (a.x + a.width) as i64 + dx
        && (a.y as i64 - dy) < (b.y + b.height) as i64
        && (b.y as i64) < (a.y + a.height) as i64 + dy
}
fn thickness(x: f64, fs: &[&Fil], s: &[Section], p: &Params) -> f64 {
    if p.probe == 0 {
        return 0.;
    }
    let x = x.floor() as i64;
    let lo = fs.iter().map(|f| f.bounds.x as i64).min().unwrap();
    let hi = fs
        .iter()
        .map(|f| (f.bounds.x + f.bounds.width) as i64)
        .max()
        .unwrap();
    if x < lo || x >= hi {
        return 0.;
    }
    let (mut min, mut max) = (i64::MAX, i64::MIN);
    for f in fs {
        for &i in &f.members {
            let a = &s[i];
            for (j, r) in a.runs.iter().enumerate() {
                if (r.start as i64) < x + p.probe && ((r.start + r.length) as i64) > x - p.probe {
                    let y = (a.first_pos + j) as i64;
                    min = min.min(y);
                    max = max.max(y);
                }
            }
        }
    }
    if min == i64::MAX {
        0.
    } else {
        (max - min + 1) as f64
    }
}
fn contact(one: &Fil, two: &Fil, s: &[Section]) -> bool {
    let t = &s[two.members[0]];
    for &i in &one.members {
        let a = &s[i];
        for (j, r) in a.runs.iter().enumerate() {
            for (k, q) in t.runs.iter().enumerate() {
                if (a.first_pos + j).abs_diff(t.first_pos + k) <= 1
                    && r.start <= q.start + q.length
                    && q.start <= r.start + r.length
                    && (a.first_pos + j == t.first_pos + k
                        || (r.start < q.start + q.length && q.start < r.start + r.length))
                {
                    return true;
                }
            }
        }
    }
    false
}
fn can_merge(a: &Fil, b: &Fil, expanding: bool, s: &[Section], p: &Params) -> bool {
    let (Some(ca), Some(cb)) = (&a.curve, &b.curve) else {
        return false;
    };
    let a0 = ca.points[0];
    let a1 = *ca.points.last().unwrap();
    let b0 = cb.points[0];
    let b1 = *cb.points.last().unwrap();
    let start = a0[0].max(b0[0]);
    let stop = a1[0].min(b1[0]);
    let gap = start - stop - 1.;
    if gap > p.gap as f64 {
        return false;
    }
    if gap < 0. {
        let mean = a.weight(s) as f64 / a.bounds.width as f64;
        let consistent = if mean < 2. {
            2. * 1.7 * mean
        } else {
            1.7 * mean
        };
        let space = if expanding { p.expansion } else { p.space };
        let n = 3f64.min(1. - gap / 10.) as usize;
        for i in 1..=n {
            let x = start - i as f64 * gap / (n + 1) as f64;
            if (ca.y(x) - cb.y(x)).abs() > p.overlap as f64 {
                return false;
            }
            let thick = thickness(x, &[a, b], s, p);
            if thick > p.thick as f64 || (-gap <= p.involving as f64 && thick > consistent) {
                return false;
            }
            if thick - thickness(x, &[a], s, p) - thickness(x, &[b], s, p) > space as f64 {
                return false;
            }
            if expanding && space == 0 && !contact(a, b, s) {
                return false;
            }
        }
    } else {
        let (q, r) = if a0[0] < b0[0] { (a1, b0) } else { (b1, a0) };
        let margin = ((a.weight(s) as f64 / a.bounds.width as f64)
            .max(b.weight(s) as f64 / b.bounds.width as f64)
            / 2.)
            .round_ties_even();
        let pos = (r[1] - q[1]).abs() - margin;
        if pos > p.pos as f64 || pos > p.pos_slope as f64 && pos / gap > 0.5 {
            return false;
        }
    }
    if a1[0] - a0[0] + 1. >= p.long as f64
        && b1[0] - b0[0] + 1. >= p.long as f64
        && ((a1[1] - a0[1]) / (a1[0] - a0[0]) - (b1[1] - b0[1]) / (b1[0] - b0[0])).abs() > 0.01
    {
        return false;
    }
    true
}
fn steal(arena: &mut [Fil], head: usize, other: usize, s: &[Section], il: usize) {
    let added = arena[other].members.clone();
    arena[head].members.extend(added);
    curves::sorted_members(s, &mut arena[head].members);
    arena[head].bounds = curves::bounds(s, &arena[head].members);
    arena[head].curve = curves::compute(s, &arena[head].members, il);
    arena[other].parent = Some(head);
}
fn merge(
    arena: &mut [Fil],
    list: &mut Vec<usize>,
    s: &[Section],
    il: usize,
    p: &Params,
    events: &mut Vec<Value>,
) {
    list.sort_by_key(|&i| std::cmp::Reverse(arena[i].bounds.width));
    for &current in list.iter() {
        let mut candidate = current;
        'candidate: loop {
            for &head in list.iter() {
                if head == current {
                    break 'candidate;
                }
                if head != candidate
                    && arena[head].parent.is_none()
                    && intersects(arena[candidate].bounds, arena[head].bounds, p.gap, p.pos)
                    && can_merge(&arena[head], &arena[candidate], false, s, p)
                {
                    events.push(json!({"phase":"merge","head":head,"absorbed":candidate}));
                    steal(arena, head, candidate, s, il);
                    candidate = head;
                    continue 'candidate;
                }
            }
            break;
        }
    }
    list.retain(|&i| arena[i].parent.is_none());
}
pub fn analyze(width: usize, sc: &Scale, s: &[Section]) -> Value {
    if s.len() > 5000 {
        return json!({"status":"failed","error":"filament_section_limit","limit":5000,"staff_detection":"not_complete"});
    }
    let il = sc.interline.main;
    let p = Params::new(sc);
    let min = (il as f64 * 0.5).round_ties_even() as usize;
    let mut arena = Vec::new();
    let mut list = Vec::new();
    let mut used = vec![false; s.len()];
    let mut events = Vec::new();
    for (i, section) in s.iter().enumerate() {
        if section.bounds().width >= min && !filaments::is_fat(section, il, sc.line.main) {
            list.push(arena.len());
            arena.push(Fil::new(i, s, il));
            used[i] = true;
        }
    }
    merge(&mut arena, &mut list, s, il, &p, &mut events);
    let mut remaining: Vec<_> = (0..s.len())
        .filter(|&i| !used[i] && !filaments::is_fat(&s[i], il, sc.line.main))
        .collect();
    remaining.sort_by_key(|&i| s[i].first_pos);
    let mut candidates = Vec::new();
    for i in remaining {
        candidates.push(arena.len());
        arena.push(Fil::new(i, s, il));
    }
    list.sort_by_key(|&i| std::cmp::Reverse(arena[i].bounds.width));
    for &head in &list {
        let frozen_bounds = arena[head].bounds;
        loop {
            let found = candidates.iter().position(|&i| {
                intersects(frozen_bounds, arena[i].bounds, p.gap, p.pos)
                    && can_merge(&arena[head], &arena[i], true, s, &p)
            });
            let Some(k) = found else { break };
            let other = candidates.remove(k);
            events.push(json!({"phase":"expand","head":head,"absorbed":other}));
            steal(&mut arena, head, other, s, il);
        }
    }
    merge(&mut arena, &mut list, s, il, &p, &mut events);
    let filaments:Vec<_>=list.iter().map(|&i|{let f=&arena[i];json!({"id":i,"section_ids":f.members.iter().map(|&j|s[j].id.0).collect::<Vec<_>>(),"bounds":[f.bounds.x,f.bounds.y,f.bounds.width,f.bounds.height],"points":f.curve.as_ref().map(|c|&c.points),"sample_y":f.curve.as_ref().map(|c|(0..=8).map(|j|{let x=f.bounds.x as f64+(f.bounds.width-1)as f64*j as f64/8.;[x,c.y(x)]}).collect::<Vec<_>>())})}).collect();
    // LinesRetriever default curvature purge, top-length slope estimate, and slope purge.
    let mut curved_removed = Vec::new();
    list.retain(|&i| {
        let Some(c) = &arena[i].curve else {
            curved_removed.push(i);
            return false;
        };
        let a = c.points[0];
        let b = *c.points.last().unwrap();
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let px = (a[0] + b[0]) / 2. - a[0];
        let py = c.y((a[0] + b[0]) / 2.) - a[1];
        let dot = px * dx + py * dy;
        let distance = (px * px + py * py - dot * dot / (dx * dx + dy * dy)).max(0.);
        let rotation = 4. * (distance / ((dx * dx + dy * dy) / 4.)).sqrt().atan();
        if rotation > 0.1 {
            curved_removed.push(i);
            false
        } else {
            true
        }
    });
    let too_few = list.len() < 5;
    list.sort_by_key(|&i| std::cmp::Reverse(arena[i].bounds.width));
    let slope = |i: usize| {
        let c = arena[i].curve.as_ref().unwrap();
        let a = c.points[0];
        let b = *c.points.last().unwrap();
        (b[1] - a[1]) / (b[0] - a[0])
    };
    let top = ((list.len() as f64 * 0.1).round_ties_even() as usize).max(1);
    let global = if list.is_empty() {
        None
    } else {
        let mean = list.iter().take(top).map(|&i| slope(i)).sum::<f64>() / top as f64;
        Some(if mean.abs() >= 0.0002 { mean } else { 0. })
    };
    let mut sloped_removed = Vec::new();
    if let Some(sheet) = global {
        let min = if sheet > 0. { -0.025 / 2. } else { sheet };
        let max = if sheet > 0. { sheet } else { 0.025 / 2. };
        list.retain(|&i| {
            let value = slope(i);
            let bad = (sheet - value).abs() > 0.025
                && !(arena[i].bounds.width < il * 4 && value >= min && value <= max);
            if bad {
                sloped_removed.push(i);
            }
            !bad
        });
    }
    let filtering = json!({"curved_removed":curved_removed,"sloped_removed":sloped_removed,"global_slope":global,"too_few_after_curvature":too_few,"retained_ids":list});
    let count = ((width as f64 / il as f64).round_ties_even() as usize).saturating_sub(1);
    let dx = width as f64 / (count + 1) as f64;
    let margin = (il as f64 * 0.05).round_ties_even() as i64;
    let mut combs = Vec::new();
    for col in 1..=count {
        let x = (dx * col as f64).round_ties_even();
        let mut ys: Vec<_> = list
            .iter()
            .filter_map(|&i| {
                let f = &arena[i];
                let c = f.curve.as_ref()?;
                if x < c.points[0][0] || x > c.points.last()?[0] {
                    None
                } else {
                    Some((i, c.y(x)))
                }
            })
            .collect();
        ys.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut groups: Vec<Vec<(usize, f64)>> = Vec::new();
        let mut active = false;
        for pair in ys.windows(2) {
            let dy = (pair[1].1 - pair[0].1).round_ties_even() as i64;
            if dy >= sc.interline.min as i64 - margin && dy <= sc.interline.max as i64 + margin {
                if !active {
                    groups.push(vec![pair[0]]);
                    active = true;
                }
                groups.last_mut().unwrap().push(pair[1]);
            } else {
                active = false;
            }
        }
        for g in groups {
            combs.push(json!({"column":col,"x":x as usize,"filament_ids":g.iter().map(|p|p.0).collect::<Vec<_>>(),"ys":g.iter().map(|p|p.1).collect::<Vec<_>>()}));
        }
    }
    json!({"stage":"merged_curved_filaments_and_combs","staff_detection":"not_complete","scope":"horizontal default factory; curvature/slope filtered; before LineCluster network","filaments":filaments,"events":events,"combs":combs,"line_filtering":filtering})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Run,
        sections::{Orientation, SectionId},
    };
    #[test]
    fn contact_excludes_diagonal_corner_only() {
        let make = |id, x, y| Section {
            id: SectionId(id),
            orientation: Orientation::Horizontal,
            first_pos: y,
            runs: vec![Run {
                start: x,
                length: 2,
            }],
            weight: 2,
        };
        let s = vec![make(1, 0, 0), make(2, 2, 1), make(3, 1, 1), make(4, 2, 0)];
        let a = Fil::new(0, &s, 16);
        assert!(!contact(&a, &Fil::new(1, &s, 16), &s));
        assert!(contact(&a, &Fil::new(2, &s, 16), &s));
        assert!(contact(&a, &Fil::new(3, &s, 16), &s));
    }
}
