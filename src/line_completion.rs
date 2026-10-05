// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit partial port: BarsRetriever column/end geometry and LinesRetriever completion.
use crate::{
    GrayImage,
    curves::{self, Curve, Point},
    position_runs,
    scale::Scale,
    sections::{self, GridPreparation, JunctionPolicy, Orientation, RunTable, Section, SectionId},
};
use serde_json::{Value, json};
fn n(v: &Value) -> f64 {
    v.as_f64().unwrap()
}
fn pts(v: &Value) -> Vec<Point> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| [n(&p[0]), n(&p[1])])
        .collect()
}
#[derive(Clone)]
struct Line {
    m: Vec<usize>,
    c: Curve,
}
struct Staff {
    id: usize,
    interline: usize,
    small: bool,
    left: f64,
    right: f64,
    lines: Vec<Line>,
}
fn snapshot(st: &[Staff], ss: &[Section], phase: &str) -> Value {
    let records:Vec<_>=st.iter().map(|s| {
        let lines:Vec<_>=s.lines.iter().map(|l| {
            let mut max=-1i64;let mut holes=0i64;let min=ss[l.m[0]].bounds().x as i64;
            for &i in &l.m {let b=ss[i].bounds();let start=b.x as i64;let stop=(b.x+b.width-1)as i64;if max != -1 && start>max {holes+=start-max;}max=max.max(stop);}
            let thickness=l.m.iter().map(|&i|ss[i].weight).sum::<usize>() as f64/(max-min+1-holes)as f64;
            json!({"points":l.c.points,"thickness":thickness,"section_ids":l.m.iter().map(|&i|ss[i].id.0).collect::<Vec<_>>()})
        }).collect();
        let mut value=json!({"id":s.id,"left":s.left,"right":s.right,"lines":lines});
        if s.small {value["interline"]=json!(s.interline);value["small"]=json!(true);}
        value
    }).collect();
    json!({"phase":phase,"staffs":records})
}

fn blank(p: &Value, x: f64, width: i32) -> Option<[i32; 2]> {
    p["blanks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| [b[0].as_i64().unwrap() as i32, b[1].as_i64().unwrap() as i32])
        .find(|b| (b[0] + b[1]) / 2 > x as i32 && b[1] - b[0] + 1 >= width)
}
pub(crate) fn extra(image: &GrayImage, prep: &GridPreparation) -> Result<Vec<Section>, String> {
    let mut short = GrayImage::new(image.width, image.height, vec![255; image.pixels.len()])?;
    for x in 0..image.width {
        for r in position_runs(image, true, x).filter(|r| r.length < prep.min_vertical_length) {
            for y in r.start..r.start + r.length {
                short.pixels[y * image.width + x] = 0;
            }
        }
    }
    let t = RunTable::from_binary(&short, Orientation::Horizontal, |r| {
        r.length < prep.min_horizontal_length
    })?;
    let mut ss = sections::build(&t, JunctionPolicy::Ratio(1.25))?;
    for s in &mut ss {
        s.id = SectionId(s.id.0 + prep.horizontal.len() as u32);
    }
    Ok(ss)
}
fn define(st: &mut [Staff], ss: &[Section], sc: &Scale, image: &GrayImage) -> Result<(), String> {
    let il = sc.interline.main;
    let fore = sc.line.main as f64;
    for s in st {
        let mut sum = 0.;
        let mut count = 0;
        let mut x = s.left;
        while x <= s.right {
            for w in s.lines.windows(2) {
                sum += w[1].c.y(x) - w[0].c.y(x);
                count += 1;
            }
            x += il as f64;
        }
        let dy = sum / count as f64;
        let mut endings = [vec![], vec![]];
        for (side, out) in endings.iter_mut().enumerate() {
            let x = if side == 0 { s.left } else { s.right };
            let ep: Vec<_> = s
                .lines
                .iter()
                .map(|l| {
                    if side == 0 {
                        l.c.points[0]
                    } else {
                        *l.c.points.last().unwrap()
                    }
                })
                .collect();
            let mut slopes: Vec<_> = s
                .lines
                .iter()
                .zip(&ep)
                .map(|(l, p)| l.c.derivative(p[0]))
                .collect();
            slopes.sort_by(f64::total_cmp);
            let slope = slopes[1..slopes.len() - 1].iter().sum::<f64>() / (slopes.len() - 2) as f64;
            let mut top = vec![];
            let mut ends = vec![None; ep.len()];
            let mut best = 0;
            let mut distance = f64::MAX;
            for (i, p) in ep.iter().enumerate() {
                let dx = x - p[0];
                if dx.abs() <= il as f64 {
                    let y = p[1] + dx * slope;
                    ends[i] = Some([x, y]);
                    top.push(y - i as f64 * dy);
                } else if dx.abs() < distance {
                    best = i;
                    distance = dx.abs();
                }
            }
            if ends.iter().any(Option::is_none) {
                let uly = if top.is_empty() {
                    ep[best][1] + (x - ep[best][0]) * slope - best as f64 * dy
                } else {
                    top.iter().sum::<f64>() / top.len() as f64
                };
                let px = if side == 0 { x } else { x - il as f64 } as i32;
                let jitter = (il as f64 * 0.25).round_ties_even() as i32;
                let (mut shift, mut bestdy, mut ratio) = (0i32, 0i32, 0.);
                for iter in 1..=1 + 2 * ((jitter + 1) / 2) {
                    let (mut matches, mut trials) = (0, 0);
                    for i in 0..ep.len() {
                        let mid = uly + shift as f64 + (i * il) as f64;
                        for y in (mid - fore / 2.).ceil() as i32..=(mid + fore / 2.).floor() as i32
                        {
                            for xx in px..px + il as i32 {
                                trials += 1;
                                if xx >= 0
                                    && y >= 0
                                    && xx < image.width as i32
                                    && y < image.height as i32
                                    && image.pixels[y as usize * image.width + xx as usize] == 0
                                {
                                    matches += 1;
                                }
                            }
                        }
                    }
                    let r = matches as f64 / trials as f64;
                    if r > ratio {
                        ratio = r;
                        bestdy = shift;
                    }
                    shift = if shift == 0 {
                        1
                    } else {
                        shift + (-shift).signum() * iter
                    };
                }
                for (i, e) in ends.iter_mut().enumerate() {
                    if e.is_none() {
                        *e = Some([x, uly + bestdy as f64 + i as f64 * dy]);
                    }
                }
            }
            *out = ends.into_iter().map(Option::unwrap).collect();
        }
        for (i, l) in s.lines.iter_mut().enumerate() {
            l.c = curves::compute_ended(ss, &l.m, il, endings[0][i], endings[1][i])
                .ok_or("invalid_explicit_curve")?;
        }
    }
    Ok(())
}
fn fill(st: &mut [Staff], il: usize) -> Result<(), String> {
    for s in st {
        for pos in 0..s.lines.len() {
            let original = s.lines[pos].c.clone();
            let mut points = vec![original.points[0]];
            for w in original.points.windows(2) {
                let gap = w[1][0] - w[0][0];
                if gap > (6 * il) as f64 {
                    let insert = (gap / (5 * il) as f64).round_ties_even() as i32 - 1;
                    for i in 1..=insert {
                        let x = (w[0][0] + i as f64 * gap / (insert + 1) as f64).round_ties_even();
                        let find = |j: usize| {
                            s.lines[j]
                                .c
                                .points
                                .iter()
                                .filter(|p| (p[0] - x).abs() <= (5 * il / 2) as f64)
                                .min_by(|a, b| (a[0] - x).abs().total_cmp(&(b[0] - x).abs()))
                                .copied()
                                .map(|p| (j, p))
                        };
                        let above = (0..pos).rev().find_map(find);
                        let below = (pos + 1..s.lines.len()).find_map(find);
                        let p = if let (Some((a, p)), Some((b, q))) = (above, below) {
                            let r = (pos - a) as f64 / (b - a) as f64;
                            [(1. - r) * p[0] + r * q[0], (1. - r) * p[1] + r * q[1]]
                        } else {
                            [x, original.y(x)]
                        };
                        points.push(p);
                    }
                }
                points.push(w[1]);
            }
            s.lines[pos].c = Curve::interpolate(points).ok_or("invalid_hole_curve")?;
        }
    }
    Ok(())
}
fn include(st: &mut [Staff], ss: &[Section], sc: &Scale, thick: bool) -> Result<(), String> {
    let limit = (sc.interline.main.pow(2) as f64 * 0.06).round_ties_even() as usize;
    let fore = sc.line.main as f64;
    let mut candidates: Vec<_> = (0..ss.len())
        .filter(|&i| (ss[i].weight > limit) == thick)
        .collect();
    candidates.sort_by_key(|&i| ss[i].first_pos);
    let mut used = vec![false; ss.len()];
    let probe = (sc.interline.main as f64 * 0.5).round_ties_even() as i64 / 2;
    for s in st {
        for l in &mut s.lines {
            let a = l.c.points[0];
            let b = *l.c.points.last().unwrap();
            let bounds = curves::bounds(ss, &l.m);
            let miny = (bounds.y as f64).min(a[1].min(b[1]).floor()) - fore;
            let maxy = ((bounds.y + bounds.height) as f64).max(a[1].max(b[1]).ceil()) + fore;
            let mut add = vec![];
            for &i in &candidates {
                let sec = &ss[i];
                if used[i] || (sec.first_pos as f64) < miny || sec.first_pos as f64 > maxy {
                    continue;
                }
                let boxx = sec.bounds();
                let cx = sec
                    .runs
                    .iter()
                    .map(|r| r.length * (2 * r.start + r.length))
                    .sum::<usize>()
                    / (2 * sec.weight);
                if (cx as f64) < a[0] || cx as f64 > b[0] || boxx.height > sc.line.max {
                    continue;
                }
                let x = (boxx.x + boxx.width / 2) as f64;
                let y = l.c.y(x);
                let cy = sec
                    .runs
                    .iter()
                    .enumerate()
                    .map(|(j, r)| r.length * (sec.first_pos + j))
                    .sum::<usize>() as f64
                    / sec.weight as f64;
                if (y - cy).abs() - fore / 2. > fore * 0.25
                    || (y - boxx.y as f64)
                        .abs()
                        .max(((boxx.y + boxx.height) as f64 - y).abs())
                        .round_ties_even()
                        > (fore * 1.2).ceil()
                {
                    continue;
                }
                let (mut low, mut high) = (i64::MAX, i64::MIN);
                for &k in l.m.iter().chain(std::iter::once(&i)) {
                    for (j, r) in ss[k].runs.iter().enumerate() {
                        if (r.start as i64) < x as i64 + probe
                            && (r.start + r.length) as i64 > x as i64 - probe
                        {
                            let y = (ss[k].first_pos + j) as i64;
                            low = low.min(y);
                            high = high.max(y);
                        }
                    }
                }
                if low != i64::MAX && high - low + 1 > sc.line.max as i64 {
                    continue;
                }
                add.push(i);
            }
            if !add.is_empty() {
                for i in add {
                    l.m.push(i);
                    used[i] = true;
                }
                curves::sorted_members(ss, &mut l.m);
                l.c =
                    curves::compute(ss, &l.m, sc.interline.main).ok_or("invalid_included_curve")?;
            }
            let a = [a[0], l.c.y(a[0])];
            let b = [b[0], l.c.y(b[0])];
            l.c = curves::compute_ended(ss, &l.m, sc.interline.main, a, b)
                .ok_or("invalid_restored_curve")?;
        }
    }
    Ok(())
}

fn stickers(st: &mut [Staff], ss: &[Section], sc: &Scale) -> Result<(), String> {
    let mut list: Vec<_> = (0..ss.len())
        .filter(|i| !st.iter().flat_map(|s| &s.lines).any(|l| l.m.contains(i)))
        .collect();
    list.sort_by_key(|&i| (ss[i].first_pos, ss[i].runs[0].start));
    let overlap = |a: &crate::Run, b: &crate::Run| {
        ((a.start + a.length).min(b.start + b.length) as i64 - a.start.max(b.start) as i64).max(0)
            as usize
    };
    let max = (sc.interline.main as f64 * 0.05).round_ties_even() as usize;
    let available: Vec<_> = list
        .iter()
        .copied()
        .filter(|&i| {
            let a = &ss[i];
            a.runs.len() == 1
                && [false, true].iter().all(|&above| {
                    let pos = if above {
                        a.first_pos as i64 - 1
                    } else {
                        (a.first_pos + a.runs.len()) as i64
                    };
                    let run = if above {
                        &a.runs[0]
                    } else {
                        a.runs.last().unwrap()
                    };
                    list.iter()
                        .filter(|&&j| ss[j].first_pos as i64 == pos)
                        .map(|&j| {
                            overlap(
                                run,
                                if above {
                                    ss[j].runs.last().unwrap()
                                } else {
                                    &ss[j].runs[0]
                                },
                            )
                        })
                        .sum::<usize>()
                        <= max
                })
        })
        .collect();
    for s in st {
        for l in &mut s.lines {
            let mut add = vec![];
            for &i in &l.m {
                let a = &ss[i];
                for above in [true, false] {
                    let pos = if above {
                        a.first_pos as i64 - 1
                    } else {
                        (a.first_pos + a.runs.len()) as i64
                    };
                    let run = if above {
                        &a.runs[0]
                    } else {
                        a.runs.last().unwrap()
                    };
                    for &j in &available {
                        if ss[j].first_pos as i64 == pos
                            && overlap(run, &ss[j].runs[0]) > 0
                            && !add.contains(&j)
                        {
                            add.push(j);
                        }
                    }
                }
            }
            if !add.is_empty() {
                let a = l.c.points[0];
                let b = *l.c.points.last().unwrap();
                l.m.extend(add);
                curves::sorted_members(ss, &mut l.m);
                l.c = curves::compute_ended(ss, &l.m, sc.interline.main, a, b)
                    .ok_or("invalid_sticker_curve")?;
            }
        }
    }
    Ok(())
}
fn polish(st: &mut [Staff], ss: &[Section], sc: &Scale) -> Result<(), String> {
    for s in st {
        for l in &mut s.lines {
            let a = l.c.points[0];
            let b = *l.c.points.last().unwrap();
            loop {
                let mut radius = i32::MAX as f64;
                let mut idx = 0;
                for (i, w) in l.c.points.windows(3).enumerate() {
                    let r = curves::point_radius(w);
                    if radius > r {
                        radius = r;
                        idx = i + 1;
                    }
                }
                if radius >= (sc.interline.main * 12) as f64 {
                    break;
                }
                if idx == 1 {
                    idx = 0;
                } else if idx == l.c.points.len() - 2 {
                    idx += 1;
                }
                let p = l.c.points[idx];
                let width = (sc.interline.main as f64 * 0.5).round_ties_even();
                let mut found: Vec<_> =
                    l.m.iter()
                        .copied()
                        .filter(|&i| {
                            let b = ss[i].bounds();
                            (b.x as f64) < p[0] + width / 2.
                                && (b.x + b.width) as f64 > p[0] - width / 2.
                                && (b.y as f64) < p[1] + width / 2.
                                && (b.y + b.height) as f64 > p[1] - width / 2.
                        })
                        .collect();
                let distance = |i: usize| {
                    let s = &ss[i];
                    let x = s
                        .runs
                        .iter()
                        .map(|r| r.length * (2 * r.start + r.length))
                        .sum::<usize>()
                        / (2 * s.weight);
                    let y = s
                        .runs
                        .iter()
                        .enumerate()
                        .map(|(j, r)| r.length * (s.first_pos + j))
                        .sum::<usize>()
                        / s.weight;
                    (p[0] - x as f64).hypot(p[1] - y as f64)
                };
                found.sort_by(|&i, &j| distance(i).total_cmp(&distance(j)));
                let Some(&remove) = found.first() else {
                    break;
                };
                l.m.retain(|&i| i != remove);
                if l.m.is_empty() {
                    return Err("curvature_removed_entire_line".into());
                }
                l.c = curves::compute_ended(ss, &l.m, sc.interline.main, a, b)
                    .ok_or("invalid_polished_curve")?;
            }
        }
    }
    Ok(())
}
fn discarded(st: &mut [Staff], ss: &[Section], sc: &Scale, staff: &Value) -> Result<(), String> {
    let mut candidates: Vec<Line> = staff["completion_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["points"].is_array())
        .map(|c| Line {
            m: c["section_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    ss.iter()
                        .position(|s| s.id.0 as u64 == id.as_u64().unwrap())
                        .unwrap()
                })
                .collect(),
            c: Curve::interpolate(pts(&c["points"])).unwrap(),
        })
        .collect();
    candidates.sort_by_key(|l| curves::bounds(ss, &l.m).y);
    let mut used = vec![false; candidates.len()];
    let fore = sc.line.main as f64;
    let probe = (sc.interline.main as f64 * 0.5).round_ties_even() as i64 / 2;
    for s in st {
        for l in &mut s.lines {
            let a = l.c.points[0];
            let b = *l.c.points.last().unwrap();
            let bounds = curves::bounds(ss, &l.m);
            let miny = (bounds.y as f64).min(a[1].min(b[1]).floor()) - fore;
            let maxy = ((bounds.y + bounds.height) as f64).max(a[1].max(b[1]).ceil()) + fore;
            let mut changed = false;
            for (i, c) in candidates.iter().enumerate() {
                let boxx = curves::bounds(ss, &c.m);
                if used[i] || (boxx.y as f64) < miny || boxx.y as f64 > maxy {
                    continue;
                }
                let weight = c.m.iter().map(|&j| ss[j].weight).sum::<usize>();
                let cx =
                    c.m.iter()
                        .flat_map(|&j| &ss[j].runs)
                        .map(|r| r.length * (2 * r.start + r.length))
                        .sum::<usize>()
                        / (2 * weight);
                if (cx as f64) < a[0]
                    || cx as f64 > b[0]
                    || weight as f64 / boxx.width as f64 > sc.line.max as f64
                {
                    continue;
                }
                let x = (boxx.x + boxx.width / 2) as f64;
                if (l.c.y(x) - c.c.y(x)).abs() - fore / 2. > fore * 0.25 {
                    continue;
                }
                let p = c.c.points[0];
                let q = *c.c.points.last().unwrap();
                if (p[1] - l.c.y(p[0]))
                    .abs()
                    .max((q[1] - l.c.y(q[0])).abs())
                    .round_ties_even()
                    > (fore * 1.2).ceil()
                {
                    continue;
                }
                let (mut lo, mut hi) = (i64::MAX, i64::MIN);
                for &j in l.m.iter().chain(&c.m) {
                    for (k, r) in ss[j].runs.iter().enumerate() {
                        if (r.start as i64) < x as i64 + probe
                            && (r.start + r.length) as i64 > x as i64 - probe
                        {
                            let y = (ss[j].first_pos + k) as i64;
                            lo = lo.min(y);
                            hi = hi.max(y);
                        }
                    }
                }
                if lo != i64::MAX && hi - lo + 1 > sc.line.max as i64 {
                    continue;
                }
                l.m.extend(&c.m);
                curves::sorted_members(ss, &mut l.m);
                l.c = curves::compute(ss, &l.m, sc.interline.main)
                    .ok_or("invalid_discarded_curve")?;
                used[i] = true;
                changed = true;
            }
            if changed {
                l.c = curves::compute_ended(ss, &l.m, sc.interline.main, a, b)
                    .ok_or("invalid_discarded_ends")?;
            }
        }
    }
    Ok(())
}
fn glyphs(st: &[Staff], ss: &[Section]) -> Vec<Value> {
    st.iter().flat_map(|s|s.lines.iter().enumerate().map(|(idx,l)|{let mut b=curves::bounds(ss,&l.m);for p in [l.c.points[0],*l.c.points.last().unwrap()]{let right=((b.x+b.width)as f64).max(p[0]).ceil()as usize;let bottom=((b.y+b.height)as f64).max(p[1]).ceil()as usize;b.x=(b.x as f64).min(p[0]).floor()as usize;b.y=(b.y as f64).min(p[1]).floor()as usize;b.width=right-b.x;b.height=bottom-b.y;}let mut pixels=std::collections::BTreeSet::new();for &i in &l.m{for(j,r)in ss[i].runs.iter().enumerate(){for x in r.start..r.start+r.length{pixels.insert((ss[i].first_pos+j-b.y,x-b.x));}}}let mut runs:Vec<[usize;3]>=vec![];for(y,x)in pixels{if let Some(r)=runs.last_mut() && r[0]==y&&r[1]+r[2]==x{r[2]+=1;continue;}runs.push([y,x,1]);}json!({"staff_id":s.id,"line_index":idx,"left":b.x,"top":b.y,"width":b.width,"height":b.height,"orientation":"HORIZONTAL","runs":runs.iter().map(|r|json!({"position":r[0],"start":r[1],"length":r[2]})).collect::<Vec<_>>()})})).collect()
}
fn endpoints(
    st: &mut [Staff],
    sc: &Scale,
    projection: &Value,
    sticks: &Value,
    systems: &Value,
    slope: f64,
    _width: usize,
) -> Value {
    let mut geometry_out = Vec::new();
    let px = |r: f64| (sc.interline.main as f64 * r).round_ties_even() as i32;
    let edges = systems["stages"].as_array().unwrap().last().unwrap()["edges"]
        .as_array()
        .unwrap();
    for system in systems["systems"].as_array().unwrap() {
        let ids: Vec<_> = system["staff_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let mut peaks: Vec<(usize, Vec<i32>, bool)> = vec![];
        for &id in &ids {
            for v in sticks["staffs"][id - 1]["sticks"].as_array().unwrap() {
                peaks.push((
                    id,
                    v["peak"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_i64().unwrap() as i32)
                        .collect(),
                    v["brace"] == true,
                ));
            }
        }
        let mut groups: Vec<Vec<usize>> = (0..peaks.len()).map(|i| vec![i]).collect();
        for e in edges {
            let find = |v: &Value| {
                peaks.iter().position(|(id, p, _)| {
                    *id == v[0].as_u64().unwrap() as usize
                        && p[0] == v[1].as_i64().unwrap() as i32
                        && p[1] == v[2].as_i64().unwrap() as i32
                })
            };
            if let (Some(a), Some(b)) = (find(&e["top"]), find(&e["bottom"])) {
                let ia = groups.iter().position(|g| g.contains(&a)).unwrap();
                let ib = groups.iter().position(|g| g.contains(&b)).unwrap();
                if ia != ib {
                    let g = groups.remove(ib);
                    let ia = groups.iter().position(|g| g.contains(&a)).unwrap();
                    groups[ia].extend(g);
                }
            }
        }
        let x = |i: usize| {
            let p = &peaks[i].1;
            ((p[0] + p[1]) as f64 / 2. + slope * (p[2] + p[3]) as f64 / 2.)
                / (1. + slope * slope).sqrt()
        };
        for g in &mut groups {
            g.sort_by_key(|&i| (peaks[i].0, peaks[i].1[0]));
        }
        groups.sort_by(|a, b| x(a[0]).total_cmp(&x(b[0])));
        let mut columns: Vec<Vec<usize>> = vec![];
        for g in groups {
            if let Some(c) = columns.last_mut() {
                let avg = c.iter().map(|&i| x(i)).sum::<f64>() / c.len() as f64;
                if (x(g[0]) - avg).abs() <= px(0.75) as f64
                    && !g
                        .iter()
                        .any(|&i| c.iter().any(|&j| peaks[i].0 == peaks[j].0))
                {
                    c.extend(g);
                    continue;
                }
            }
            columns.push(g);
        }
        let geometry = |c: &Vec<usize>| {
            (
                c.iter().map(|&i| x(i)).sum::<f64>() / c.len() as f64,
                c.iter()
                    .map(|&i| (peaks[i].1[1] - peaks[i].1[0] + 1) as f64)
                    .sum::<f64>()
                    / c.len() as f64,
            )
        };
        let mut start: Option<usize> = None;
        for (k, c) in columns.iter().enumerate() {
            if c.len() != ids.len() || c.iter().any(|&i| peaks[i].2) {
                continue;
            }
            if let Some(j) = start {
                let (a, wa) = geometry(&columns[j]);
                let (b, wb) = geometry(c);
                if b - wb / 2. - a - wa / 2. > px(if k == 1 { 2. } else { 0.75 }) as f64 {
                    break;
                }
            }
            let connected = ids.windows(2).all(|w| {
                let a = c.iter().find(|&&i| peaks[i].0 == w[0]);
                let b = c.iter().find(|&&i| peaks[i].0 == w[1]);
                match (a, b) {
                    (Some(&a), Some(&b)) => edges.iter().any(|e| {
                        e["connection"] == true
                            && e["top"] == json!([peaks[a].0, peaks[a].1[0], peaks[a].1[1]])
                            && e["bottom"] == json!([peaks[b].0, peaks[b].1[0], peaks[b].1[1]])
                    }),
                    _ => false,
                }
            });
            if connected {
                start = Some(k);
            }
        }
        if let Some(k) = start {
            let valid = columns[k].iter().all(|&i| {
                let (id, p, _) = &peaks[i];
                let left = st[*id - 1].left;
                p[0] as f64 - left <= px(0.15) as f64
                    && (left <= p[1] as f64
                        || blank(
                            &projection["staff_projections"][*id - 1],
                            p[1] as f64,
                            px(1.),
                        )
                        .is_none_or(|b| b[0] as f64 > left))
            });
            if valid {
                for &i in &columns[k] {
                    st[peaks[i].0 - 1].left = peaks[i].1[1] as f64;
                }
            }
        }
        geometry_out.push(json!({"system_id":system["id"],"columns":columns.iter().map(|c|json!({"full":c.len()==ids.len()&&!c.iter().any(|&i|peaks[i].2),"peaks":c.iter().map(|&i|json!([peaks[i].0,peaks[i].1[0],peaks[i].1[1]])).collect::<Vec<_>>()})).collect::<Vec<_>>(),"start_column":start.filter(|&k|columns[k].iter().all(|&i|st[peaks[i].0-1].left==peaks[i].1[1]as f64))}));
    }
    json!(geometry_out)
}
#[allow(clippy::too_many_arguments)] // Mirrors the explicitly staged diagnostic pipeline.
pub fn analyze(
    image: &GrayImage,
    sc: &Scale,
    prep: &GridPreparation,
    staff: &Value,
    projection: &Value,
    sticks: &Value,
    systems: &Value,
    slope: f64,
) -> Value {
    if systems["status"] != "initial_systems" {
        return json!({"status":"blocked","grid_complete":false});
    }
    let run = || -> Result<Value, String> {
        let mut ss: Vec<_> = prep
            .horizontal
            .iter()
            .map(|s| Section {
                id: s.id,
                orientation: s.orientation,
                first_pos: s.first_pos,
                weight: s.weight,
                runs: s
                    .runs
                    .iter()
                    .map(|r| crate::Run {
                        start: r.start,
                        length: r.length,
                    })
                    .collect(),
            })
            .collect();
        ss.extend(extra(image, prep)?);
        let mut st: Vec<_> = staff["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| Staff {
                interline: s["interline"].as_u64().unwrap_or(sc.interline.main as u64) as usize,
                small: s["small"] == true,
                id: s["id"].as_u64().unwrap() as usize,
                left: n(&s["left"]),
                right: n(&s["right"]),
                lines: s["lines"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|l| Line {
                        m: l["section_ids"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|id| {
                                ss.iter()
                                    .position(|s| s.id.0 as u64 == id.as_u64().unwrap())
                                    .unwrap()
                            })
                            .collect(),
                        c: Curve::interpolate(pts(&l["points"])).unwrap(),
                    })
                    .collect(),
            })
            .collect();
        let bar_geometry = endpoints(&mut st, sc, projection, sticks, systems, slope, image.width);
        let braces = crate::braces::analyze(
            image,
            sc,
            prep,
            staff,
            projection,
            sticks,
            systems,
            &bar_geometry,
        )?;
        let initial_ends = snapshot(&st, &ss, "detectStartColumns");
        let all_sections = ss
            .iter()
            .chain(&prep.vertical)
            .map(crate::bar_sticks::copy)
            .collect::<Vec<_>>();
        let bar_filtering = crate::bar_semantics::filter(
            sc,
            sticks,
            systems,
            &json!({"endpoint_geometry":initial_ends,"bar_geometry":bar_geometry,"braces":braces}),
            projection,
            image.width,
            &all_sections,
        )?;
        for (s, g) in st
            .iter_mut()
            .zip(bar_filtering["staffs"].as_array().unwrap())
        {
            s.left = n(&g["left"]);
            s.right = n(&g["right"]);
        }
        let ends = snapshot(&st, &ss, "endpointGeometry");
        let mut phases = vec![];
        define(&mut st, &ss, sc, image)?;
        phases.push(snapshot(&st, &ss, "defineEndPoints"));
        discarded(&mut st, &ss, sc, staff)?;
        phases.push(snapshot(&st, &ss, "includeDiscardedFilaments"));
        fill(&mut st, sc.interline.main)?;
        phases.push(snapshot(&st, &ss, "fillHoles_1"));
        include(&mut st, &ss, sc, true)?;
        phases.push(snapshot(&st, &ss, "includeThick"));
        include(&mut st, &ss, sc, false)?;
        phases.push(snapshot(&st, &ss, "includeThin"));
        polish(&mut st, &ss, sc)?;
        phases.push(snapshot(&st, &ss, "polishCurvatures"));
        fill(&mut st, sc.interline.main)?;
        phases.push(snapshot(&st, &ss, "fillHoles_2"));
        stickers(&mut st, &ss, sc)?;
        phases.push(snapshot(&st, &ss, "includeStickers"));
        fill(&mut st, sc.interline.main)?;
        phases.push(snapshot(&st, &ss, "fillHoles_3"));

        Ok(
            json!({"status":"partial_line_completion","grid_complete":false,"global_slope":slope,"braces":braces,"bar_geometry":bar_geometry,"bar_filtering":bar_filtering,"endpoint_geometry":ends,"line_phases":phases,"staff_line_glyphs":glyphs(&st,&ss),"pending":["brace fallback and nondefault GRID branches","non-GRID SIG relation types and contexts"]}),
        )
    };
    match run() {
        Ok(v) => v,
        Err(e) => json!({"status":"failed","error":e,"grid_complete":false}),
    }
}
