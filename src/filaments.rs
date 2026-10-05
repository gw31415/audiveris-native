// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// 5.11.0: FilamentFactory initial selection/isSectionFat, BasicLine, ClustersRetriever combs.
use crate::{
    scale::Scale,
    sections::{Orientation, Section},
};
use serde_json::{Value, json};
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}
impl Line {
    pub fn from_section(s: &Section) -> Option<Self> {
        let (mut n, mut sx, mut sy, mut xx, mut yy, mut xy) = (0., 0., 0., 0., 0., 0.);
        for (i, r) in s.runs.iter().enumerate() {
            for x in r.start..r.start + r.length {
                let (x, y) = (x as f64, (s.first_pos + i) as f64);
                n += 1.;
                sx += x;
                sy += y;
                xx += x * x;
                yy += y * y;
                xy += x * y;
            }
        }
        if n < 2. {
            return None;
        }
        let h = n * xx - sx * sx;
        let v = n * yy - sy * sy;
        let (mut a, mut b, mut c) = if h.abs() >= v.abs() {
            ((n * xy - sx * sy) / h, -1., (sy * xx - sx * xy) / h)
        } else {
            (-1., (n * xy - sx * sy) / v, (sx * yy - sy * xy) / v)
        };
        let norm = a.hypot(b);
        a /= norm;
        b /= norm;
        c /= norm;
        if [a, b, c].iter().all(|x| x.is_finite()) {
            Some(Self { a, b, c })
        } else {
            None
        }
    }
    pub fn slope(self) -> f64 {
        -self.a / self.b
    }
    pub fn y(self, x: f64) -> f64 {
        (-self.a * x - self.c) / self.b
    }
}
fn pixels_in(s: &Section, x: i64, y: i64, w: i64, h: i64) -> usize {
    s.runs
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            let pos = (s.first_pos + i) as i64;
            if pos < y || pos >= y + h {
                return None;
            }
            Some(
                ((r.start + r.length) as i64)
                    .min(x + w)
                    .saturating_sub((r.start as i64).max(x))
                    .max(0) as usize,
            )
        })
        .sum()
}
/// Horizontal factory only. Error/undefined lines are fat, matching the Java catch branch.
pub fn is_fat(s: &Section, interline: usize, line_height: usize) -> bool {
    let b = s.bounds();
    let thickness = s.weight as f64 / b.width as f64;
    if thickness <= 1. {
        return false;
    }
    if b.width as f64 / thickness < 3. {
        return true;
    }
    let max = (line_height as f64 * 1.5).round_ties_even() as i64;
    let Some(line) = Line::from_section(s) else {
        return true;
    };
    if line.slope().abs() < std::f64::consts::FRAC_PI_4 {
        let start = (b.x + b.width / 4) as i64;
        let stop = (b.x + 3 * b.width / 4) as i64;
        let half =
            (((interline as f64 * 0.5).round_ties_even() as usize) / 2).min(b.width / 4) as i64;
        let probe = |x: i64| {
            let y = line.y(x as f64).round_ties_even() as i64;
            let count = pixels_in(s, x - half, y - max, half * 2, max * 2);
            (count as f64 / (half * 2) as f64).round_ties_even() as i64
        };
        probe(start) > max || probe(stop) > max
    } else {
        b.height as i64 > max
    }
}
#[derive(Debug, Clone)]
pub struct FlatSeed {
    pub section_id: u32,
    pub x0: usize,
    pub x1: usize,
    pub y: f64,
}
/// Exact retrieveCombs sampling/grouping on an explicit list of constant-ordinate filaments.
/// This adapter is deliberately limited to rectangular seed sections, before merge/expand.
pub fn combs(width: usize, sc: &Scale, seeds: &[FlatSeed]) -> Vec<Value> {
    let count =
        ((width as f64 / sc.interline.main as f64).round_ties_even() as usize).saturating_sub(1);
    let dx = width as f64 / (count + 1) as f64;
    let margin = (sc.interline.main as f64 * 0.05).round_ties_even() as i64;
    let (min, max) = (
        sc.interline.min as i64 - margin,
        sc.interline.max as i64 + margin,
    );
    let mut result = Vec::new();
    for col in 1..=count {
        let x = (dx * col as f64).round_ties_even() as usize;
        let mut crossing: Vec<_> = seeds.iter().filter(|s| x >= s.x0 && x <= s.x1).collect();
        crossing.sort_by(|a, b| a.y.total_cmp(&b.y));
        let mut groups: Vec<Vec<&FlatSeed>> = Vec::new();
        let mut active = false;
        for pair in crossing.windows(2) {
            let dy = (pair[1].y - pair[0].y).round_ties_even() as i64;
            if dy >= min && dy <= max {
                if !active {
                    groups.push(vec![pair[0]]);
                    active = true;
                }
                groups.last_mut().unwrap().push(pair[1]);
            } else {
                active = false;
            }
        }
        for group in groups {
            result.push(json!({"column":col,"x":x,"section_ids":group.iter().map(|s|s.section_id).collect::<Vec<_>>(),"ys":group.iter().map(|s|s.y).collect::<Vec<_>>()}));
        }
    }
    result
}
pub fn analyze(width: usize, sc: &Scale, sections: &[Section]) -> Value {
    let min = (sc.interline.main as f64 * 0.5).round_ties_even() as usize;
    let mut seeds = Vec::new();
    let mut decisions = Vec::new();
    let mut flat = Vec::new();
    for s in sections {
        assert_eq!(s.orientation, Orientation::Horizontal);
        let b = s.bounds();
        let reason = if b.width < min {
            "too_short"
        } else if is_fat(s, sc.interline.main, sc.line.main) {
            "too_fat"
        } else {
            "seed"
        };
        decisions.push(json!({"section_id":s.id.0,"decision":reason}));
        if reason != "seed" {
            continue;
        }
        let line = Line::from_section(s);
        let rectangular = s
            .runs
            .iter()
            .all(|r| r.start == s.runs[0].start && r.length == s.runs[0].length);
        seeds.push(json!({"section_id":s.id.0,"bounds":[b.x,b.y,b.width,b.height],"line_abc":line.map(|l|[l.a,l.b,l.c]),"comb_eligible":rectangular}));
        if rectangular {
            flat.push(FlatSeed {
                section_id: s.id.0,
                x0: b.x,
                x1: b.x + b.width - 1,
                y: b.y as f64 + (b.height - 1) as f64 / 2.,
            });
        }
    }
    let combs = combs(width, sc, &flat);
    let mut groups: Vec<Vec<u32>> = Vec::new();
    for comb in &combs {
        let ids: Vec<u32> = comb["section_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap() as u32)
            .collect();
        if ids.len() == 5 && !groups.contains(&ids) {
            groups.push(ids);
        }
    }
    json!({"stage":"initial_filament_seeds_and_restricted_combs","staff_detection":"not_complete","merge_expand":"not_implemented","curve_interpolation":"not_implemented","comb_input_scope":"rectangular_unmerged_seeds_only","min_core_section_length":min,"max_thickness":(sc.line.main as f64*1.5).round_ties_even() as usize,"decisions":decisions,"seeds":seeds,"combs":combs,"five_line_seed_groups":groups,"group_semantics":"diagnostic unique five-member combs; NOT upstream LineCluster or Staff"})
}
