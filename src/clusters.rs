// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// Default five-line, single-interline ClustersRetriever/LineCluster pipeline.
use crate::{
    curves::{self, Curve},
    scale::Scale,
    sections::{Bounds, Section},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Clone, Default)]
struct Fil {
    members: Vec<usize>,
    parent: Option<usize>,
    cluster: Option<usize>,
    pos: i32,
    combs: BTreeMap<usize, usize>,
    curve: Option<Curve>,
    bounds: Option<Bounds>,
}
struct Comb {
    fils: Vec<usize>,
    processed: bool,
}
struct Cluster {
    seed: usize,
    interline: usize,
    lines: BTreeMap<i32, usize>,
    parent: Option<usize>,
    bounds: Option<Bounds>,
    length: Option<i64>,
}
struct Pass {
    clusters: Vec<usize>,
    phases: Vec<Value>,
    discarded: Vec<usize>,
}
struct State<'a> {
    s: &'a [Section],
    sc: &'a Scale,
    interline: usize,
    f: Vec<Fil>,
    combs: Vec<Comb>,
    c: Vec<Cluster>,
    events: Vec<Value>,
    slope: f64,
    depth: usize,
    failed: bool,
}
fn union(a: Bounds, b: Bounds) -> Bounds {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Bounds {
        x,
        y,
        width: (a.x + a.width).max(b.x + b.width) - x,
        height: (a.y + a.height).max(b.y + b.height) - y,
    }
}
fn overlap(a: Bounds, b: Bounds) -> i64 {
    (a.x + a.width).min(b.x + b.width) as i64 - a.x.max(b.x) as i64
}
fn near(a: Bounds, b: Bounds, dx: i64, dy: i64) -> bool {
    (a.x as i64 - dx) < (b.x + b.width) as i64
        && (b.x as i64) < (a.x + a.width) as i64 + dx
        && (a.y as i64 - dy) < (b.y + b.height) as i64
        && (b.y as i64) < (a.y + a.height) as i64 + dy
}
impl State<'_> {
    fn px(&self, v: f64) -> i64 {
        (self.interline as f64 * v).round_ties_even() as i64
    }
    fn anc(&self, mut f: usize) -> usize {
        while let Some(p) = self.f[f].parent {
            f = p;
        }
        f
    }
    fn ca(&self, mut c: usize) -> usize {
        while let Some(p) = self.c[c].parent {
            c = p;
        }
        c
    }
    fn fb(&self, f: usize) -> Bounds {
        self.f[f].bounds.unwrap()
    }
    fn cb(&mut self, c: usize) -> Bounds {
        if let Some(b) = self.c[c].bounds {
            return b;
        }
        let mut it = self.c[c].lines.values();
        let mut b = self.fb(*it.next().unwrap());
        for &f in it {
            b = union(b, self.fb(f));
        }
        self.c[c].bounds = Some(b);
        b
    }
    fn invalidate(&mut self, c: usize) {
        self.c[c].bounds = None;
        self.c[c].length = None;
    }
    fn true_length(&self, f: usize) -> i64 {
        let m = &self.f[f].members;
        let min = self.s[m[0]].bounds().x as i64;
        let mut max = -1;
        let mut holes = 0;
        for &i in m {
            let b = self.s[i].bounds();
            let start = b.x as i64;
            let stop = (b.x + b.width - 1) as i64;
            if max != -1 && start > max {
                holes += start - max;
            }
            max = max.max(stop);
        }
        max - min + 1 - holes
    }
    fn clength(&mut self, c: usize) -> i64 {
        if let Some(n) = self.c[c].length {
            return n;
        }
        let n = self.c[c]
            .lines
            .values()
            .map(|&f| self.true_length(f))
            .sum::<i64>()
            / self.c[c].lines.len() as i64;
        self.c[c].length = Some(n);
        n
    }
    fn combine(&mut self, head: usize, other: usize, phase: &str) {
        if head == other {
            return;
        }
        self.events
            .push(json!({"phase":phase,"head":head,"absorbed":other}));
        let m = self.f[other].members.clone();
        self.f[head].members.extend(m);
        curves::sorted_members(self.s, &mut self.f[head].members);
        let members = &self.f[head].members;
        let b = curves::bounds(self.s, members);
        let curve = curves::compute(self.s, members, self.sc.interline.main);
        let combs = self.f[other].combs.clone();
        self.f[head].bounds = Some(b);
        self.f[head].curve = curve;
        self.f[head].combs.extend(combs);
        self.f[other].parent = Some(head);
        self.f[other].cluster = self.f[head].cluster;
        self.f[other].pos = self.f[head].pos;
    }
    fn endpoints(&self, f: usize) -> ([f64; 2], [f64; 2]) {
        let p = &self.f[f].curve.as_ref().unwrap().points;
        (p[0], *p.last().unwrap())
    }
    fn y(&self, f: usize, x: f64) -> f64 {
        self.f[f].curve.as_ref().unwrap().y(x)
    }
    fn within(&self, f: usize, x: f64) -> bool {
        let (a, b) = self.endpoints(f);
        x >= a[0] && x <= b[0]
    }
    fn ordinate(&self, p: [f64; 2]) -> f64 {
        (-self.slope * p[0] + p[1]) / (1. + self.slope * self.slope).sqrt()
    }
    fn cordinate(&mut self, c: usize) -> f64 {
        let b = self.cb(c);
        self.ordinate([(b.x + b.width / 2) as f64, (b.y + b.height / 2) as f64])
    }
    fn sort_y(&mut self, list: &mut [usize]) {
        let keys: BTreeMap<_, _> = list.iter().map(|&c| (c, self.cordinate(c))).collect();
        list.sort_by(|a, b| keys[a].total_cmp(&keys[b]));
    }
    fn sort_length(&mut self, list: &mut [usize]) {
        let keys: BTreeMap<_, _> = list.iter().map(|&c| (c, self.clength(c))).collect();
        list.sort_by_key(|c| std::cmp::Reverse(keys[c]));
    }
    fn network(&mut self, all: &mut Vec<usize>) {
        for fil in all.clone() {
            let mut lines = BTreeMap::new();
            let maps: Vec<_> = self.f[fil].combs.values().copied().collect();
            for ci in maps {
                let fs = self.combs[ci].fils.clone();
                let pivot = fs
                    .iter()
                    .position(|&i| self.anc(i) == self.anc(fil))
                    .map(|n| n as i32)
                    .unwrap_or(-1);
                for (i, &f) in fs.iter().enumerate() {
                    let line = i as i32 - pivot;
                    if line == 0 {
                        continue;
                    }
                    if let Some(&prev) = lines.get(&line) {
                        let a = self.anc(prev);
                        let b = self.anc(f);
                        if a != b {
                            if self.fb(a).width >= self.fb(b).width {
                                self.combine(a, b, "network");
                            } else {
                                self.combine(b, a, "network");
                            }
                        }
                    } else {
                        lines.insert(line, f);
                    }
                }
            }
        }
        all.retain(|&f| self.f[f].parent.is_none());
    }
    fn include_cluster(&mut self, c: usize, other: usize, delta: i32) {
        let entries = self.c[other].lines.clone();
        for (pos, that) in entries {
            let p = pos + delta;
            if let Some(&head) = self.c[c].lines.get(&p) {
                self.combine(head, that, "cluster_include");
            } else {
                self.c[c].lines.insert(p, that);
                self.f[that].cluster = Some(c);
                self.f[that].pos = p;
            }
        }
        self.c[other].parent = Some(c);
        self.invalidate(c);
        self.events
            .push(json!({"phase":"cluster_merge","head":c,"absorbed":other,"delta":delta}));
    }
    fn include_pivot(&mut self, c: usize, pivot: usize, pivot_pos: i32) {
        self.depth += 1;
        if self.depth > 1024 {
            self.failed = true;
            self.depth -= 1;
            return;
        }
        let ancestor = self.anc(pivot);
        let cs: Vec<_> = self.f[pivot].combs.values().copied().collect();
        if cs.is_empty() {
            self.c[c].lines.insert(0, pivot);
            self.f[pivot].cluster = Some(c);
            self.f[pivot].pos = 0;
        } else {
            for ci in cs {
                if self.failed {
                    break;
                }
                if self.combs[ci].processed {
                    continue;
                }
                self.combs[ci].processed = true;
                let fs = self.combs[ci].fils.clone();
                let index = fs
                    .iter()
                    .position(|&f| self.anc(f) == self.anc(pivot))
                    .map(|x| x as i32)
                    .unwrap_or(-1);
                let delta = pivot_pos - index;
                for (i, f) in fs.into_iter().enumerate() {
                    let fil = self.anc(f);
                    if let Some(other) = self.f[fil].cluster {
                        if self.ca(other) != self.ca(c) {
                            self.include_cluster(c, other, i as i32 + delta - self.f[fil].pos);
                        }
                    } else {
                        let pos = i as i32 + delta;
                        if let Some(&line) = self.c[c].lines.get(&pos) {
                            self.combine(line, fil, "cluster_include");
                        } else {
                            self.c[c].lines.insert(pos, fil);
                        }
                        self.f[fil].cluster = Some(c);
                        self.f[fil].pos = pos;
                        if fil != ancestor {
                            self.include_pivot(c, fil, pos);
                        }
                    }
                }
            }
        }
        self.depth -= 1;
    }
    fn create(&mut self, all: &mut [usize]) -> Vec<usize> {
        all.sort_by_key(|&f| std::cmp::Reverse(self.fb(f).width));
        let mut result = Vec::new();
        for &f in all.iter() {
            let f = self.anc(f);
            if self.f[f].cluster.is_none() && !self.f[f].combs.is_empty() {
                let c = self.c.len();
                self.c.push(Cluster {
                    seed: f,
                    interline: self.interline,
                    lines: BTreeMap::new(),
                    parent: None,
                    bounds: None,
                    length: None,
                });
                self.include_pivot(c, f, 0);
                result.push(c);
            }
        }
        result.retain(|&c| self.c[c].parent.is_none());
        result
    }
    fn points_at(&self, c: usize, x: f64, margin: i64) -> Vec<Option<[f64; 2]>> {
        let mut result = BTreeMap::new();
        let mut holes = Vec::new();
        for (&pos, &f) in &self.c[c].lines {
            if self.within(f, x) {
                result.insert(pos, Some([x, self.y(f, x)]));
            } else {
                holes.push(pos);
            }
        }
        for pos in holes {
            let f = self.c[c].lines[&pos];
            let (a, b) = self.endpoints(f);
            let end = if x <= a[0] { a } else { b };
            let mut y = None;
            for dir in [-1, 1] {
                if let Some(&other) = self.c[c].lines.get(&(pos + dir))
                    && self.within(other, x)
                    && self.within(other, end[0])
                {
                    y = Some(self.y(other, x) + (self.y(f, end[0]) - self.y(other, end[0])));
                    break;
                }
            }
            if y.is_none() && (x - end[0]).abs() <= margin as f64 {
                y = Some(end[1] + (x - end[0]) * self.slope);
            }
            result.insert(pos, y.map(|y| [x, y]));
        }
        result.into_values().collect()
    }
    fn thickness(&self, x: f64, one: usize, two: usize) -> f64 {
        let coord = x.floor() as i64;
        let b = union(self.fb(one), self.fb(two));
        if coord < b.x as i64 || coord >= (b.x + b.width) as i64 {
            return 0.;
        }
        let half = self.px(0.5) / 2;
        let mut lo = i64::MAX;
        let mut hi = i64::MIN;
        for f in [one, two] {
            for &i in &self.f[f].members {
                let s = &self.s[i];
                for (j, r) in s.runs.iter().enumerate() {
                    if (r.start as i64) < coord + half
                        && ((r.start + r.length) as i64) > coord - half
                    {
                        let y = (s.first_pos + j) as i64;
                        lo = lo.min(y);
                        hi = hi.max(y);
                    }
                }
            }
        }
        if lo == i64::MAX {
            0.
        } else {
            (hi - lo + 1) as f64
        }
    }
    fn include_index(&mut self, c: usize, fil: usize, index: usize) -> bool {
        let (pos, line) = self.c[c]
            .lines
            .iter()
            .nth(index)
            .map(|(&p, &f)| (p, f))
            .unwrap();
        let fb = self.fb(fil);
        for &i in &self.f[line].members {
            let sb = self.s[i].bounds();
            let ov = overlap(fb, sb);
            if ov > 0
                && self.thickness((fb.x.max(sb.x) as i64 + ov / 2) as f64, fil, line)
                    > self.sc.line.max as f64
            {
                return false;
            }
        }
        self.combine(line, fil, "cluster_expand");
        self.f[fil].cluster = Some(c);
        self.f[fil].pos = pos;
        self.invalidate(c);
        true
    }
    fn expand_one(&mut self, c: usize, all: &[usize]) {
        let mut cb = None;
        for &f in all {
            let f = self.anc(f);
            if self.f[f].cluster.is_some() {
                continue;
            }
            let b = *cb.get_or_insert_with(|| self.cb(c));
            let fb = self.fb(f);
            let x = (fb.x + fb.width / 2) as f64;
            let y = self.y(f, x).round_ties_even();
            let dx = self.px(6.) as f64;
            let dy = self.px(2.) as f64;
            if x >= b.x as f64 - dx
                && x < (b.x + b.width) as f64 + dx
                && y >= b.y as f64 - dy
                && y < (b.y + b.height) as f64 + dy
            {
                let pts = self.points_at(c, x, self.px(2.));
                for (i, p) in pts.iter().enumerate() {
                    if let Some(p) = p
                        && (y - p[1]).abs() <= self.px(0.175) as f64
                        && self.include_index(c, f, i)
                    {
                        cb = None;
                        break;
                    }
                }
            }
        }
    }
    fn expand(&mut self, cs: &mut [usize], all: &[usize]) {
        let mut starts = all.to_vec();
        starts.sort_by(|&a, &b| self.endpoints(a).0[0].total_cmp(&self.endpoints(b).0[0]));
        let mut stops = starts.clone();
        stops.sort_by(|&a, &b| self.endpoints(a).1[0].total_cmp(&self.endpoints(b).1[0]));
        self.sort_length(cs);
        for &c in cs.iter() {
            self.expand_one(c, &stops);
            self.expand_one(c, &starts);
        }
    }
    fn best(&self, a: &[Option<[f64; 2]>], b: &[Option<[f64; 2]>]) -> (f64, i32) {
        let max = a.len().max(b.len()) as i32 - 1;
        let (mut best, mut delta) = (f64::MAX, 0);
        for d in -max..=max {
            let (mut sum, mut n) = (0., 0);
            for (i, x) in a.iter().enumerate() {
                let j = i as i32 + d;
                if j >= 0
                    && (j as usize) < b.len()
                    && let (Some(x), Some(y)) = (x, b[j as usize])
                {
                    n += 1;
                    sum += (self.ordinate(y) - self.ordinate(*x)).abs();
                }
            }
            if n > 0 && sum / (n as f64) < best {
                best = sum / n as f64;
                delta = d;
            }
        }
        (best, delta)
    }
    fn can_merge(&mut self, a: usize, b: usize) -> Option<i32> {
        let ab = self.cb(a);
        let bb = self.cb(b);
        let ar = ab.x + ab.width - 1;
        let br = bb.x + bb.width - 1;
        let gap = ab.x.max(bb.x) as i64 - ar.min(br) as i64;
        if gap > self.px(6.) {
            return None;
        }
        if gap <= 0 {
            let x = (ab.x.max(bb.x) + ar.min(br)) / 2;
            let pa = self.points_at(a, x as f64, self.px(6.));
            let pb = self.points_at(b, x as f64, self.px(6.));
            let (dist, delta) = self.best(&pa, &pb);
            if dist > self.px(0.4) as f64 {
                return None;
            }
            let aa: Vec<_> = self.c[a].lines.values().copied().collect();
            let bb: Vec<_> = self.c[b].lines.values().copied().collect();
            for (i, &fa) in aa.iter().enumerate() {
                let j = i as i32 + delta;
                if j >= 0 && (j as usize) < bb.len() {
                    let fb = bb[j as usize];
                    let ra = self.fb(fa);
                    let rb = self.fb(fb);
                    let ov = overlap(ra, rb);
                    if ov >= 0
                        && self.thickness((ra.x.max(rb.x) as i64 + ov / 2) as f64, fa, fb)
                            > self.sc.line.max as f64
                    {
                        return None;
                    }
                }
            }
            Some(delta)
        } else {
            let left = ab.x < bb.x;
            let pa: Vec<_> = self.c[a]
                .lines
                .values()
                .map(|&f| {
                    Some(if left {
                        self.endpoints(f).1
                    } else {
                        self.endpoints(f).0
                    })
                })
                .collect();
            let pb: Vec<_> = self.c[b]
                .lines
                .values()
                .map(|&f| {
                    Some(if left {
                        self.endpoints(f).0
                    } else {
                        self.endpoints(f).1
                    })
                })
                .collect();
            let (d, delta) = self.best(&pa, &pb);
            if d <= self.px(0.4) as f64 {
                Some(delta)
            } else {
                None
            }
        }
    }
    fn merge_with(&mut self, c: usize, other: usize, d: i32) {
        let delta = d + self.c[c].lines.first_key_value().unwrap().0
            - self.c[other].lines.first_key_value().unwrap().0;
        self.include_cluster(c, other, delta);
    }
    fn merge(&mut self, cs: &mut Vec<usize>, all: &mut Vec<usize>) {
        self.sort_y(cs);
        for current in cs.clone() {
            let candidate = current;
            'again: loop {
                let cb = self.cb(candidate);
                for &head in cs.iter() {
                    if head == current {
                        break 'again;
                    }
                    if head == candidate || self.c[head].parent.is_some() {
                        continue;
                    }
                    let hb = self.cb(head);
                    if near(cb, hb, self.px(6.), self.px(2.))
                        && let Some(delta) = self.can_merge(head, candidate)
                    {
                        self.merge_with(candidate, head, delta);
                        continue 'again;
                    }
                }
                break;
            }
        }
        cs.retain(|&c| self.c[c].parent.is_none());
        all.retain(|&f| self.f[f].parent.is_none());
    }
    fn trim(&mut self, cs: &mut [usize]) {
        self.sort_y(cs);
        for &c in cs.iter() {
            while self.c[c].lines.len() > 5 {
                let (&top, &tf) = self.c[c].lines.first_key_value().unwrap();
                let (&bot, &bf) = self.c[c].lines.last_key_value().unwrap();
                let p = if self.true_length(tf) < self.true_length(bf) {
                    top
                } else {
                    bot
                };
                let f = self.c[c].lines.remove(&p).unwrap();
                self.f[f].cluster = None;
                self.f[f].pos = 0;
                self.f[f].combs.clear();
                self.events.push(json!({"phase":"trim","cluster":c,"filament":f,"true_length":self.true_length(f)}));
            }
            let first = *self.c[c].lines.first_key_value().unwrap().0;
            if first != 0 {
                let old = self.c[c].lines.clone();
                self.c[c].lines.clear();
                for (pos, f) in old {
                    self.f[f].cluster = Some(c);
                    self.f[f].pos = pos - first;
                    self.c[c].lines.insert(pos - first, f);
                }
            }
            self.invalidate(c);
        }
    }
    fn destroy_non_five(&mut self, cs: &mut Vec<usize>) {
        cs.retain(|&c| {
            if self.c[c].lines.len() == 5 {
                return true;
            }
            for &f in self.c[c].lines.values() {
                self.f[f].cluster = None;
                self.f[f].pos = 0;
                self.f[f].combs.clear();
            }
            self.events
                .push(json!({"phase":"reject_size","cluster":c,"size":self.c[c].lines.len()}));
            false
        });
    }
    fn pairs(&mut self, cs: &mut Vec<usize>, all: &mut Vec<usize>) {
        if cs.is_empty() {
            return;
        }
        self.sort_y(cs);
        let mut lengths: Vec<_> = cs.iter().map(|&c| self.clength(c)).collect();
        lengths.sort();
        let min = lengths[lengths.len() / 2] as f64 * 0.2;
        let mut idx = 0;
        'whole: while idx < cs.len() {
            let c = cs[idx];
            let cb = self.cb(c);
            let max = self.cordinate(c) + self.px(1.) as f64;
            for j in idx + 1..cs.len() {
                let other = cs[j];
                if self.c[other].lines.len() != self.c[c].lines.len() {
                    continue;
                }
                if self.cordinate(other) > max {
                    break;
                }
                let ob = self.cb(other);
                if -overlap(cb, ob) > self.px(6.) {
                    continue;
                }
                self.merge_with(c, other, 0);
                cs.remove(j);
                continue 'whole;
            }
            let length = self.clength(c);
            if (length as f64) < min {
                self.events.push(json!({"phase":"reject_short_isolated","cluster":c,"true_length":length,"threshold":min}));
                cs.remove(idx);
            } else {
                idx += 1;
            }
        }
        all.retain(|&f| self.f[f].parent.is_none());
    }
    fn snapshot(&mut self, cs: &[usize], phase: &str) -> Value {
        let mut result = Vec::new();
        for &c in cs {
            // Debug observation must not prime upstream lazy caches.
            let saved = (self.c[c].bounds, self.c[c].length);
            let b = self.cb(c);
            let length = self.clength(c);
            self.c[c].bounds = saved.0;
            self.c[c].length = saved.1;
            result.push(json!({"id":c,"seed":self.c[c].seed,"lines":self.c[c].lines.iter().map(|(&pos,&f)|json!({"pos":pos,"filament":f,"sections":self.f[f].members.iter().map(|&i|self.s[i].id.0).collect::<Vec<_>>(),"true_length":self.true_length(f)})).collect::<Vec<_>>(),"bounds":[b.x,b.y,b.width,b.height],"true_length":length}));
        }
        json!({"phase":phase,"clusters":result})
    }
}
pub fn analyze(sc: &Scale, sections: &[Section], factory: &Value) -> Value {
    if factory["status"] == "failed" || factory["line_filtering"]["too_few_after_curvature"] == true
    {
        return json!({"status":"blocked","error":"filament_stage_failed"});
    }
    if sc.small_interline.is_some() {
        return json!({"status":"unsupported","error":"multiple_interline_requires_second_pass"});
    }
    let raw = factory["filaments"].as_array().unwrap();
    let max = raw
        .iter()
        .map(|x| x["id"].as_u64().unwrap() as usize)
        .max()
        .unwrap_or(0);
    let mut st = State {
        s: sections,
        sc,
        interline: sc.interline.main,
        f: vec![Fil::default(); max + 1],
        combs: Vec::new(),
        c: Vec::new(),
        events: Vec::new(),
        slope: factory["line_filtering"]["global_slope"]
            .as_f64()
            .unwrap_or(0.),
        depth: 0,
        failed: false,
    };
    let lookup: BTreeMap<_, _> = sections
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.0, i))
        .collect();
    for v in raw {
        let id = v["id"].as_u64().unwrap() as usize;
        let members: Vec<_> = v["section_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| lookup[&(x.as_u64().unwrap() as u32)])
            .collect();
        let bounds = curves::bounds(sections, &members);
        let curve = curves::compute(sections, &members, sc.interline.main);
        st.f[id] = Fil {
            members,
            bounds: Some(bounds),
            curve,
            ..Fil::default()
        };
    }
    let mut all: Vec<_> = factory["line_filtering"]["retained_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_u64().unwrap() as usize)
        .collect();
    let initial = all.clone();
    for v in factory["combs"].as_array().unwrap() {
        let col = v["column"].as_u64().unwrap() as usize;
        let fs: Vec<_> = v["filament_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap() as usize)
            .collect();
        let ci = st.combs.len();
        for &f in &fs {
            st.f[f].combs.insert(col, ci);
        }
        st.combs.push(Comb {
            fils: fs,
            processed: false,
        });
    }
    st.network(&mut all);
    let network = all.clone();
    let mut cs = st.create(&mut all);
    let mut phases = vec![st.snapshot(&cs, "create")];
    if st.failed {
        return json!({"status":"failed","error":"cluster_recursion_limit"});
    }
    st.expand(&mut cs, &all);
    phases.push(st.snapshot(&cs, "expand_1"));
    st.merge(&mut cs, &mut all);
    phases.push(st.snapshot(&cs, "merge"));
    st.trim(&mut cs);
    phases.push(st.snapshot(&cs, "trim"));
    st.destroy_non_five(&mut cs);
    phases.push(st.snapshot(&cs, "desired_size"));
    st.pairs(&mut cs, &mut all);
    phases.push(st.snapshot(&cs, "pair_and_length"));
    st.expand(&mut cs, &all);
    phases.push(st.snapshot(&cs, "expand_2"));
    let discarded: Vec<_> = all
        .iter()
        .filter(|&&f| st.f[f].cluster.is_none())
        .copied()
        .collect();
    all.retain(|&f| st.f[f].cluster.is_some() && st.f[f].parent.is_none());
    // LinesRetriever.buildStaves: default five-line branch (all one-line-only purges are inert).
    let min_staff = st.px(30.) as usize;
    cs.retain(|&c|{let width=st.cb(c).width;if width<min_staff{st.events.push(json!({"phase":"reject_min_staff_length","cluster":c,"width":width,"threshold":min_staff}));false}else{true}});
    let boxes: BTreeMap<_, _> = cs.iter().map(|&c| (c, st.cb(c))).collect();
    let ordinates: BTreeMap<_, _> = cs.iter().map(|&c| (c, st.cordinate(c))).collect();
    cs.sort_by(|a, b| {
        if overlap(boxes[a], boxes[b]) < 0 {
            (boxes[a].x + boxes[a].width / 2).cmp(&(boxes[b].x + boxes[b].width / 2))
        } else {
            ordinates[a].total_cmp(&ordinates[b])
        }
    });
    phases.push(st.snapshot(&cs, "build_staves_length_layout"));
    let spans: Vec<_> = cs
        .iter()
        .map(|&c| {
            let a = *st.c[c].lines.first_key_value().unwrap().1;
            let b = *st.c[c].lines.last_key_value().unwrap().1;
            (st.endpoints(a).0[1], st.endpoints(b).0[1])
        })
        .collect();
    let mut staffs = Vec::new();
    for (i, &c) in cs.iter().enumerate() {
        let ids: Vec<_> = st.c[c].lines.values().copied().collect();
        let mut lefts: Vec<_> = ids.iter().map(|&f| st.endpoints(f).0[0]).collect();
        let mut rights: Vec<_> = ids.iter().map(|&f| st.endpoints(f).1[0]).collect();
        lefts.sort_by(f64::total_cmp);
        rights.sort_by(f64::total_cmp);
        let short = spans
            .iter()
            .enumerate()
            .any(|(j, &(top, bottom))| i != j && bottom.min(spans[i].1) > top.max(spans[i].0));
        staffs.push(json!({"id":i+1,"cluster":c,"interline":sc.interline.main,"left":lefts[lefts.len()/2],"right":rights[rights.len()/2],"small":false,"short":short,"line_ids":ids,"lines":ids.iter().map(|&f|json!({"filament":f,"section_ids":st.f[f].members.iter().map(|&k|sections[k].id.0).collect::<Vec<_>>(),"points":st.f[f].curve.as_ref().unwrap().points})).collect::<Vec<_>>()}));
    }
    json!({"completion_candidates":st.f.iter().enumerate().filter(|(id,f)| f.bounds.is_some() && f.parent.is_none() && (discarded.contains(id) || factory["line_filtering"]["sloped_removed"].as_array().is_some_and(|a|a.contains(&json!(id))))).map(|(id,f)|json!({"id":id,"section_ids":f.members.iter().map(|&k|sections[k].id.0).collect::<Vec<_>>(),"points":f.curve.as_ref().map(|c|&c.points)})).collect::<Vec<_>>(),"status":"initial_staffs","scope":"default five-line single-interline initial Staff descriptors; later GRID completion and systems pending","staffs":staffs,"input_line_ids":initial,"network_line_ids":network,"remaining_line_ids":all,"discarded_line_ids":discarded,"phases":phases,"events":st.events,"initial_staff_construction":true})
}

/// Original first-pass cluster length consistency. Length is the horizontal
/// bounding-box extent, not the hole-adjusted true length used to order clusters.
pub fn cluster_lengths_consistent(lengths: &[usize]) -> Result<bool, String> {
    if lengths.is_empty()
        || lengths
            .iter()
            .any(|n| *n == 0 || *n > crate::sections::MAX_DIMENSION)
    {
        return Err("invalid_cluster_consistency_lengths".into());
    }
    let min = *lengths.iter().min().unwrap() as f64;
    let max = *lengths.iter().max().unwrap() as f64;
    Ok((max - min) / ((min + max) / 2.0) <= 0.5)
}
impl State<'_> {
    fn retrieve_combs(&mut self, width: usize, range: crate::scale::Range, all: &[usize]) {
        for &f in all {
            self.f[f].combs.clear();
        }
        let count =
            ((width as f64 / self.interline as f64).round_ties_even() as usize).saturating_sub(1);
        let dx = width as f64 / (count + 1) as f64;
        let margin = (self.interline as f64 * 0.05).round_ties_even() as i64;
        for col in 1..=count {
            let x = (dx * col as f64).round_ties_even();
            let mut ys: Vec<_> = all
                .iter()
                .filter_map(|&f| {
                    if self.within(f, x) {
                        Some((f, self.y(f, x)))
                    } else {
                        None
                    }
                })
                .collect();
            ys.sort_by(|a, b| a.1.total_cmp(&b.1));
            let mut groups: Vec<Vec<usize>> = Vec::new();
            let mut active = false;
            for pair in ys.windows(2) {
                let dy = (pair[1].1 - pair[0].1).round_ties_even() as i64;
                if dy >= range.min as i64 - margin && dy <= range.max as i64 + margin {
                    if !active {
                        groups.push(vec![pair[0].0]);
                        active = true;
                    }
                    groups.last_mut().unwrap().push(pair[1].0);
                } else {
                    active = false;
                }
            }
            for fils in groups {
                let ci = self.combs.len();
                for &f in &fils {
                    self.f[f].combs.insert(col, ci);
                }
                self.combs.push(Comb {
                    fils,
                    processed: false,
                });
            }
        }
    }
    fn pass(&mut self, all: &mut Vec<usize>, check_consistency: bool) -> Result<Pass, String> {
        self.network(all);
        let mut cs = self.create(all);
        let mut phases = vec![self.snapshot(&cs, "create")];
        if self.failed {
            return Err("cluster_recursion_limit".into());
        }
        self.expand(&mut cs, all);
        phases.push(self.snapshot(&cs, "expand_1"));
        self.merge(&mut cs, all);
        phases.push(self.snapshot(&cs, "merge"));
        self.trim(&mut cs);
        phases.push(self.snapshot(&cs, "trim"));
        self.destroy_non_five(&mut cs);
        phases.push(self.snapshot(&cs, "desired_size"));
        self.pairs(&mut cs, all);
        phases.push(self.snapshot(&cs, "pair_and_length"));
        if check_consistency {
            cs.retain(|&c| {
                let lengths:Vec<_>=self.c[c].lines.values().map(|&f|self.fb(f).width).collect();
                if cluster_lengths_consistent(&lengths).unwrap() {true} else {
                    for &f in self.c[c].lines.values(){self.f[f].cluster=None;self.f[f].pos=0;self.f[f].combs.clear();}
                    self.events.push(json!({"phase":"reject_inconsistent_lengths","cluster":c,"lengths":lengths}));false
                }
            });
            phases.push(self.snapshot(&cs, "length_consistency"));
        }
        self.expand(&mut cs, all);
        phases.push(self.snapshot(&cs, "expand_2"));
        let discarded: Vec<_> = all
            .iter()
            .filter(|&&f| self.f[f].cluster.is_none())
            .copied()
            .collect();
        all.retain(|&f| self.f[f].cluster.is_some() && self.f[f].parent.is_none());
        Ok(Pass {
            clusters: cs,
            phases,
            discarded,
        })
    }
    fn fil_snapshot(&self, ids: &[usize]) -> Vec<Value> {
        ids.iter().map(|&f|json!({"id":f,"filament":f,"section_ids":self.f[f].members.iter().map(|&k|self.s[k].id.0).collect::<Vec<_>>(),"points":self.f[f].curve.as_ref().unwrap().points})).collect()
    }
}
/// Selected default five-line initial-staff boundary, including original GRID
/// second pass. Page CLI integration waits for downstream multi-scale parity.
/// The filament factory and section inputs are actual runtime data, not fixtures.
pub fn analyze_two_pass(width: usize, sc: &Scale, sections: &[Section], factory: &Value) -> Value {
    if sc.small_interline.is_none() {
        return analyze(sc, sections, factory);
    }
    let small = sc.small_interline.unwrap();
    if width == 0
        || width > crate::sections::MAX_DIMENSION
        || small.min == 0
        || small.min > small.main
        || small.main > small.max
        || small.main >= sc.interline.main
        || sc.interline.main > 128
        || sc.interline.min == 0
        || sc.interline.min > sc.interline.main
        || sc.interline.main > sc.interline.max
    {
        return json!({"status":"unsupported","error":"invalid_second_pass_scale_or_width"});
    }
    if factory["status"] == "failed" || factory["line_filtering"]["too_few_after_curvature"] == true
    {
        return json!({"status":"blocked","error":"filament_stage_failed"});
    }
    let Some(raw) = factory["filaments"].as_array() else {
        return json!({"status":"unsupported","error":"invalid_second_pass_filaments"});
    };
    let Some(ids) = factory["line_filtering"]["retained_ids"].as_array() else {
        return json!({"status":"unsupported","error":"invalid_second_pass_retained_ids"});
    };
    if raw.len() > crate::sections::MAX_SECTIONS
        || ids.len().saturating_mul(width / small.main + 1) > 16_000_000
    {
        return json!({"status":"unsupported","error":"unsupported_second_pass_work_limit"});
    }
    if sections.len() > crate::sections::MAX_SECTIONS
        || sections.iter().any(|s| {
            s.orientation != crate::sections::Orientation::Horizontal
                || s.runs.is_empty()
                || s.first_pos
                    .checked_add(s.runs.len())
                    .is_none_or(|n| n > crate::sections::MAX_DIMENSION)
                || s.runs.iter().any(|r| {
                    r.length == 0
                        || r.start
                            .checked_add(r.length)
                            .is_none_or(|n| n > crate::sections::MAX_DIMENSION)
                })
                || s.runs.iter().map(|r| r.length).sum::<usize>() != s.weight
        })
    {
        return json!({"status":"unsupported","error":"invalid_second_pass_sections"});
    }
    let lookup: BTreeMap<_, _> = sections
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.0, i))
        .collect();
    if lookup.len() != sections.len() {
        return json!({"status":"unsupported","error":"invalid_second_pass_sections"});
    }
    let invalid = || json!({"status":"unsupported","error":"invalid_second_pass_filaments"});
    if raw.iter().any(|r| {
        r["id"]
            .as_u64()
            .is_none_or(|id| id > crate::sections::MAX_RUNS as u64)
    }) {
        return invalid();
    }
    let max = raw
        .iter()
        .map(|r| r["id"].as_u64().unwrap() as usize)
        .max()
        .unwrap_or(0);
    let mut st = State {
        s: sections,
        sc,
        interline: sc.interline.main,
        f: vec![Fil::default(); max + 1],
        combs: Vec::new(),
        c: Vec::new(),
        events: Vec::new(),
        slope: factory["line_filtering"]["global_slope"]
            .as_f64()
            .unwrap_or(0.),
        depth: 0,
        failed: false,
    };
    for v in raw {
        let Some(ms) = v["section_ids"].as_array() else {
            return invalid();
        };
        if ms.is_empty()
            || ms.iter().any(|id| {
                id.as_u64()
                    .is_none_or(|n| n > u32::MAX as u64 || !lookup.contains_key(&(n as u32)))
            })
        {
            return invalid();
        }
        let members: Vec<_> = ms
            .iter()
            .map(|id| lookup[&(id.as_u64().unwrap() as u32)])
            .collect();
        let id = v["id"].as_u64().unwrap() as usize;
        let bounds = curves::bounds(sections, &members);
        let curve = curves::compute(sections, &members, sc.interline.main);
        if st.f[id].bounds.is_some() {
            return invalid();
        }
        st.f[id] = Fil {
            members,
            bounds: Some(bounds),
            curve,
            ..Fil::default()
        };
    }
    if ids.iter().any(|id| {
        id.as_u64()
            .is_none_or(|id| id > max as u64 || st.f[id as usize].curve.is_none())
    }) {
        return invalid();
    }
    let mut all: Vec<_> = ids.iter().map(|id| id.as_u64().unwrap() as usize).collect();
    let input = all.clone();
    let mut unique = all.clone();
    unique.sort_unstable();
    unique.dedup();
    if unique.len() != all.len() {
        return invalid();
    }
    st.retrieve_combs(width, sc.interline, &all);
    let Ok(Pass {
        clusters: mut cs,
        phases: first_phases,
        mut discarded,
    }) = st.pass(&mut all, true)
    else {
        return json!({"status":"failed","error":"cluster_recursion_limit"});
    };
    let first_discarded = st.fil_snapshot(&discarded);
    discarded.sort_unstable();
    let second_input = st.fil_snapshot(&discarded);
    let mut second_phases = Vec::new();
    let mut second_ran = false;
    if !discarded.is_empty() {
        second_ran = true;
        st.interline = small.main;
        st.retrieve_combs(width, small, &discarded);
        let Ok(Pass {
            clusters: second,
            phases: p,
            discarded: d,
        }) = st.pass(&mut discarded, false)
        else {
            return json!({"status":"failed","error":"cluster_recursion_limit"});
        };
        cs.extend(second);
        second_phases = p;
        discarded = d;
    }
    let min_staff = (sc.interline.main as f64 * 30.).round_ties_even() as usize;
    cs.retain(|&c| st.cb(c).width >= min_staff);
    let boxes: BTreeMap<_, _> = cs.iter().map(|&c| (c, st.cb(c))).collect();
    let ordinates: BTreeMap<_, _> = cs.iter().map(|&c| (c, st.cordinate(c))).collect();
    cs.sort_by(|a, b| {
        if overlap(boxes[a], boxes[b]) < 0 {
            (boxes[a].x + boxes[a].width / 2).cmp(&(boxes[b].x + boxes[b].width / 2))
        } else {
            ordinates[a].total_cmp(&ordinates[b])
        }
    });
    let spans: Vec<_> = cs
        .iter()
        .map(|&c| {
            let a = *st.c[c].lines.first_key_value().unwrap().1;
            let b = *st.c[c].lines.last_key_value().unwrap().1;
            (st.endpoints(a).0[1], st.endpoints(b).0[1])
        })
        .collect();
    let mut staffs = Vec::new();
    for (i, &c) in cs.iter().enumerate() {
        let fs: Vec<_> = st.c[c].lines.values().copied().collect();
        let mut lefts: Vec<_> = fs.iter().map(|&f| st.endpoints(f).0[0]).collect();
        let mut rights: Vec<_> = fs.iter().map(|&f| st.endpoints(f).1[0]).collect();
        lefts.sort_by(f64::total_cmp);
        rights.sort_by(f64::total_cmp);
        let short = spans
            .iter()
            .enumerate()
            .any(|(j, &(top, bottom))| i != j && bottom.min(spans[i].1) > top.max(spans[i].0));
        staffs.push(json!({"id":i+1,"cluster":c,"interline":st.c[c].interline,"small":second_ran && st.c[c].interline==small.main,"short":short,"left":lefts[lefts.len()/2],"right":rights[rights.len()/2],"line_ids":fs,"lines":st.fil_snapshot(&fs)}));
    }
    let completion_candidates: Vec<_> = st.f.iter().enumerate().filter(|(id,f)| f.bounds.is_some() && f.parent.is_none() && (discarded.contains(id) || factory["line_filtering"]["sloped_removed"].as_array().is_some_and(|a|a.contains(&json!(id))))).map(|(id,f)|json!({"id":id,"section_ids":f.members.iter().map(|&k|sections[k].id.0).collect::<Vec<_>>(),"points":f.curve.as_ref().map(|c|&c.points)})).collect();
    json!({"completion_candidates":completion_candidates,"status":"initial_staffs","scope":"selected default five-line two-interline initial Staff construction; downstream GRID and HEADS/STEMS full-page parity pending","input_line_ids":input,"first_discarded":first_discarded,"second_input":second_input,"second_discarded":st.fil_snapshot(&discarded),"second_pass_ran":second_ran,"first_phases":first_phases,"second_phases":second_phases,"staffs":staffs,"events":st.events,"initial_staff_construction":true,"grid_complete":false,"heads_complete":false,"stems_complete":false})
}
