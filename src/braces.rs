// Copyright © Audiveris 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// StaffProjector.findBracePeak/createBracePeak, BarsRetriever brace portions and compound.
use crate::{
    GrayImage,
    bar_sticks::{centroid_x, copy, intersects, touch},
    curves::{self, Curve},
    scale::Scale,
    sections::{Bounds, GridPreparation, Orientation, Section},
};
use serde_json::{Value, json};
#[derive(Clone)]
struct Portion {
    staff: usize,
    p: [i64; 4],
    m: Vec<usize>,
    kind: &'static str,
}
fn curve(v: &Value) -> Curve {
    Curve::interpolate(
        v["points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
            .collect(),
    )
    .unwrap()
}
fn find(
    staff: &Value,
    projection: &Value,
    mut min: i64,
    mut max: i64,
    il: usize,
) -> Option<[i64; 4]> {
    let blanks: Vec<[i64; 2]> = projection["blanks"]
        .as_array()?
        .iter()
        .map(|b| [b[0].as_i64().unwrap(), b[1].as_i64().unwrap()])
        .collect();
    let wide = (il as f64 * 2.0).round_ties_even() as i64;
    if let Some(b) = projection["ending_blanks"]["LEFT"].as_array() {
        let (a, z) = (b[0].as_i64()?, b[1].as_i64()?);
        if z + 2 >= max {
            max = a - 1;
            min = blanks
                .iter()
                .rev()
                .find(|b| (b[0] + b[1]) / 2 < max && b[1] - b[0] + 1 >= wide)
                .map(|b| b[1])
                .unwrap_or(min);
        } else {
            min = min.max(z);
        }
    }
    let values = projection["projection"].as_array()?;
    let val = |x: i64| {
        if x >= 0 && (x as usize) < values.len() {
            values[x as usize].as_i64().unwrap()
        } else {
            0
        }
    };
    let threshold = (il as f64 * 1.1).round_ties_even() as i64;
    let (mut start, mut stop, mut valley) = (-1, -1, false);
    for x in (min..=max).rev() {
        if val(x) >= threshold {
            if !valley {
                continue;
            }
            if stop == -1 {
                stop = x;
            }
            start = x;
        } else if !valley {
            valley = true;
        } else if stop != -1 {
            break;
        }
    }
    if start < 0 {
        return None;
    }
    start = blanks
        .iter()
        .take_while(|b| b[1] < start)
        .last()
        .map(|b| b[1])
        .unwrap_or(start);
    let mut v = val(start);
    for x in (0..start).rev() {
        if val(x) < v {
            v = val(x);
            start = x;
        } else {
            break;
        }
    }
    stop = (stop..=max).min_by_key(|&x| val(x))?;
    let x = ((start + stop) / 2) as f64;
    let lines = staff["lines"].as_array()?;
    Some([
        start,
        stop,
        curve(&lines[0]).y(x).round_ties_even() as i64,
        curve(lines.last()?).y(x).round_ties_even() as i64,
    ])
}
fn build_portion(all: &[Section], p: [i64; 4], sc: &Scale) -> Vec<usize> {
    let ext = (sc.interline.main as f64 * 0.5).round_ties_even() as usize;
    let core = Bounds {
        x: p[0] as usize,
        y: p[2] as usize,
        width: (p[1] - p[0] + 1) as usize,
        height: (p[3] - p[2] + 1) as usize,
    };
    let lookup = Bounds {
        x: core.x,
        y: core.y.saturating_sub(ext),
        width: core.width,
        height: core.height + ext + ext.min(core.y),
    };
    let source: Vec<_> = all
        .iter()
        .enumerate()
        .filter(|(_, s)| s.bounds().width <= core.width && intersects(s.bounds(), lookup))
        .map(|(i, _)| i)
        .collect();
    let mut m: Vec<_> = source
        .iter()
        .copied()
        .filter(|&i| {
            let s = &all[i];
            let b = s.bounds();
            let x = centroid_x(s);
            b.height >= ext && intersects(b, core) && x >= core.x && x < core.x + core.width
        })
        .collect();
    if m.is_empty() {
        return m;
    }
    loop {
        let next = source.iter().copied().find(|i| {
            !m.contains(i) && {
                let b = all[*i].bounds();
                b.x >= core.x
                    && b.x + b.width <= core.x + core.width
                    && m.iter().any(|&j| touch(&all[j], &all[*i]))
            }
        });
        if let Some(i) = next {
            m.push(i);
        } else {
            break;
        }
    }
    m.sort_by_key(|&i| {
        let b = all[i].bounds();
        (b.x, b.y, all[i].id.0)
    });
    m
}
fn look(
    staff: &Value,
    proj: &Value,
    right: i64,
    all: &[Section],
    sc: &Scale,
    first: bool,
    last: bool,
) -> Option<Portion> {
    let il = sc.interline.main;
    let px = |v: f64| (il as f64 * v).round_ties_even() as i64;
    let max = right - 1 - px(0.1);
    let min = (max - px(3.) - px(2.)).max(0);
    let p = find(staff, proj, min, max, il)?;
    if p[1] - p[0] + 1 > px(3.) {
        return None;
    }
    let m = build_portion(all, p, sc);
    if m.is_empty() {
        return None;
    }
    let b = curves::bounds(all, &m);
    if b.height < px(3.) as usize {
        return None;
    }
    let c = curves::compute_vertical(all, &m, il)?;
    if curves::mean_radius(&c.points) >= px(25.) as f64 {
        return None;
    }
    let half = sc.line.max as f64 / 2.;
    let above = p[2] as f64 - half - b.y as f64 > px(0.5) as f64 && !first;
    let below = (b.y + b.height - 1) as f64 - half - p[3] as f64 > px(0.5) as f64 && !last;
    Some(Portion {
        staff: staff["id"].as_u64()? as usize,
        p,
        m,
        kind: match (above, below) {
            (true, true) => "MIDDLE",
            (false, true) => "TOP",
            (true, false) => "BOTTOM",
            _ => "UNCLASSIFIED",
        },
    })
}
fn glyph(all: &[Section], m: &[usize]) -> Value {
    let b = curves::bounds(all, m);
    let mut pixels = std::collections::BTreeSet::new();
    for &i in m {
        let s = &all[i];
        for (j, r) in s.runs.iter().enumerate() {
            for v in r.start..r.start + r.length {
                let (x, y) = match s.orientation {
                    Orientation::Horizontal => (v, s.first_pos + j),
                    Orientation::Vertical => (s.first_pos + j, v),
                };
                pixels.insert((x - b.x, y - b.y));
            }
        }
    }
    let mut runs: Vec<[usize; 3]> = vec![];
    for (x, y) in pixels {
        if let Some(r) = runs.last_mut()
            && r[0] == x
            && r[1] + r[2] == y
        {
            r[2] += 1;
            continue;
        }
        runs.push([x, y, 1]);
    }
    json!({"left":b.x,"top":b.y,"width":b.width,"height":b.height,"orientation":"VERTICAL","runs":runs.iter().map(|r|json!({"position":r[0],"start":r[1],"length":r[2]})).collect::<Vec<_>>()})
}
fn full(all: &[Section], ps: &[Portion], sc: &Scale) -> Vec<usize> {
    let margin = (sc.interline.main as f64 * 0.5).round_ties_even();
    let mut left = vec![];
    let mut right = vec![];
    for p in ps {
        left.push([p.p[2] as f64, p.p[0] as f64 - margin]);
        left.push([(p.p[3] + 1) as f64, p.p[0] as f64 - margin]);
        right.push([p.p[2] as f64, (p.p[1] + 1) as f64]);
        right.push([(p.p[3] + 1) as f64, (p.p[1] + 1) as f64]);
    }
    let at = |v: &Vec<[f64; 2]>, y: f64| {
        let w = v.windows(2).find(|w| y >= w[0][0] && y <= w[1][0]).unwrap();
        if w[0][0] == w[1][0] {
            w[0][1]
        } else {
            w[0][1] + (w[1][1] - w[0][1]) * (y - w[0][0]) / (w[1][0] - w[0][0])
        }
    };
    let mut m: Vec<_> = ps.iter().flat_map(|p| p.m.iter().copied()).collect();
    m.sort_unstable();
    m.dedup();
    let source: Vec<_> = all
        .iter()
        .enumerate()
        .filter(|(i, s)| {
            if m.contains(i) {
                return false;
            }
            let b = s.bounds();
            let top = b.y as f64;
            let bottom = (b.y + b.height) as f64;
            if top < left[0][0] || bottom > left.last().unwrap()[0] {
                return false;
            }
            let ys = std::iter::once(top)
                .chain(std::iter::once(bottom))
                .chain(left.iter().map(|p| p[0]).filter(|&y| y > top && y < bottom));
            ys.into_iter()
                .all(|y| b.x as f64 >= at(&left, y) && (b.x + b.width) as f64 <= at(&right, y))
        })
        .map(|(i, _)| i)
        .collect();
    loop {
        let next = source
            .iter()
            .copied()
            .find(|i| !m.contains(i) && m.iter().any(|&j| touch(&all[j], &all[*i])));
        if let Some(i) = next {
            m.push(i);
        } else {
            break;
        }
    }
    m.sort_by_key(|&i| {
        let b = all[i].bounds();
        (b.x, b.y, all[i].id.0)
    });
    m
}
#[allow(clippy::too_many_arguments)]
pub fn analyze(
    image: &GrayImage,
    sc: &Scale,
    prep: &GridPreparation,
    staff: &Value,
    projection: &Value,
    sticks: &Value,
    systems: &Value,
    geometry: &Value,
) -> Result<Value, String> {
    let mut all: Vec<_> = prep
        .vertical
        .iter()
        .chain(&prep.horizontal)
        .map(copy)
        .chain(crate::line_completion::extra(image, prep)?)
        .filter(|s| s.bounds().width <= sc.interline.main)
        .collect();
    all.sort_by_key(|s| s.bounds().x);
    let mut output = vec![];
    let mut diagnostics = vec![];
    for (g, sys) in geometry
        .as_array()
        .unwrap()
        .iter()
        .zip(systems["systems"].as_array().unwrap())
    {
        let ids: Vec<_> = sys["staff_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        if ids.len() < 2 {
            continue;
        }
        let Some(start) = g["start_column"].as_u64().map(|x| x as usize) else {
            continue;
        };
        let mut removed = vec![];
        for c in g["columns"].as_array().unwrap().iter().skip(start + 1) {
            if c["full"] != true {
                removed.extend(c["peaks"].as_array().unwrap().iter().cloned());
            }
        }
        // purgeTooLeft + deleteRelatedColumns, in official order after partial columns.
        for &id in &ids {
            let key = g["columns"][start]["peaks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p[0] == id)
                .unwrap();
            let mut prev = key[1].as_i64().unwrap();
            let mut keys = vec![];
            for p in sticks["staffs"][id - 1]["sticks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p["peak"][0].as_i64().unwrap() < key[1].as_i64().unwrap())
                .rev()
            {
                let k = json!([id, p["peak"][0], p["peak"][1]]);
                if removed.contains(&k) {
                    continue;
                }
                if prev - p["peak"][1].as_i64().unwrap() + 1 > 2 * sc.interline.main as i64 {
                    keys.push(k);
                } else {
                    prev = p["peak"][0].as_i64().unwrap();
                }
            }
            for c in g["columns"].as_array().unwrap() {
                let ps = c["peaks"].as_array().unwrap();
                if ps.iter().any(|k| keys.contains(k)) {
                    removed.extend(ps.iter().cloned());
                }
            }
        }
        let mut portions: Vec<Option<Portion>> = vec![];
        for (index, &id) in ids.iter().enumerate() {
            let first = index == 0;
            let last = index + 1 == ids.len();
            let peaks: Vec<_> = sticks["staffs"][id - 1]["sticks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| !removed.contains(&json!([id, p["peak"][0], p["peak"][1]])))
                .collect();
            let startkey = g["columns"][start]["peaks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|k| k[0] == id)
                .unwrap();
            let startx = startkey[1].as_i64().unwrap();
            let mut valid = peaks;
            valid.sort_by_key(|p| p["peak"][0].as_i64().unwrap());
            let Some(p) = valid.first() else {
                portions.push(None);
                continue;
            };
            let st = &staff["staffs"][id - 1];
            let pr = &projection["staff_projections"][id - 1];
            let mut found = look(
                st,
                pr,
                p["peak"][0].as_i64().unwrap(),
                &all,
                sc,
                first,
                last,
            );
            if found.is_none() && p["peak"][0].as_i64().unwrap() < startx && valid.len() > 1 {
                found = look(
                    st,
                    pr,
                    valid[1]["peak"][0].as_i64().unwrap(),
                    &all,
                    sc,
                    first,
                    last,
                );
            }
            if let Some(p) = &found {
                diagnostics.push(json!({"staff_id":p.staff,"peak":p.p,"kind":p.kind,"sections":p.m.iter().map(|&i|json!({"id":all[i].id.0,"orientation":if all[i].orientation==Orientation::Horizontal{"HORIZONTAL"}else{"VERTICAL"}})).collect::<Vec<_>>()}));
            }
            portions.push(found);
        }
        let mut i = 0;
        while i < portions.len() {
            if portions[i].as_ref().is_some_and(|p| p.kind == "TOP") {
                let mut ps = vec![portions[i].as_ref().unwrap().clone()];
                let mut j = i + 1;
                while j < portions.len() {
                    let Some(p) = &portions[j] else {
                        return Err(format!(
                            "unsupported_brace_fallback_alignment: system {} staff {}",
                            sys["id"], ids[j]
                        ));
                    };
                    if p.kind != "MIDDLE" && p.kind != "BOTTOM" {
                        break;
                    }
                    ps.push(p.clone());
                    if p.kind == "BOTTOM" {
                        break;
                    }
                    j += 1;
                }
                if ps.len() > 1 {
                    let m = full(&all, &ps, sc);
                    output.push(json!({"system_id":sys["id"],"staff_ids":ps.iter().map(|p|p.staff).collect::<Vec<_>>(),"glyph":glyph(&all,&m),"grade":0.8}));
                    i = ids
                        .iter()
                        .position(|&id| id == ps.last().unwrap().staff)
                        .unwrap();
                }
            }
            i += 1;
        }
    }
    Ok(
        json!({"status":"brace_portions_and_compounds","braces":output,"portions":diagnostics,"unsupported":[],"bracket_support":"bar_filtering_stage"}),
    )
}
/// Attach actual brace glyphs/inters and default part/page topology to the native GRID IR.
/// Numeric IDs remain native; missing brace fallback/bracket recognition stays explicit.
#[derive(serde::Serialize)]
pub(crate) struct PartMembership {
    pub(crate) id: u64,
    pub(crate) staff_ids: Vec<u64>,
    #[serde(skip)]
    pub(crate) system: u64,
}
pub(crate) struct IntegratedGrid {
    pub(crate) physical_glyphs: Vec<std::sync::Arc<audiveris_staff_removal::GlyphPixels>>,
    pub(crate) report: Value,
    pub(crate) parts: Vec<PartMembership>,
}
pub fn integrate(
    ir: Value,
    completion: &Value,
    sc: &Scale,
    height: usize,
) -> Result<Value, String> {
    Ok(integrate_owned(ir, completion, sc, height)?.report)
}
pub(crate) fn integrate_owned(
    ir: Value,
    completion: &Value,
    sc: &Scale,
    height: usize,
) -> Result<IntegratedGrid, String> {
    let mut registry = crate::grid_ir::GlyphRegistry::default();
    for g in ir["glyphs"].as_array().unwrap() {
        registry.register(g);
    }
    integrate_registered(ir, completion, sc, height, registry)
}
// The recognition path moves its original registry through brace registration;
// the report-only facade above cannot replace those held physical objects.
pub(crate) fn integrate_registered(
    mut ir: Value,
    completion: &Value,
    sc: &Scale,
    height: usize,
    mut registry: crate::grid_ir::GlyphRegistry,
) -> Result<IntegratedGrid, String> {
    let mut actual_parts = Vec::new();
    let staffs = ir["staffs"].as_array().unwrap().clone();
    let systems = ir["systems"].as_array().unwrap().clone();
    let mut bars = ir["bar_semantics"].clone();
    let curves: Vec<Vec<Curve>> = staffs
        .iter()
        .map(|s| s["lines"].as_array().unwrap().iter().map(curve).collect())
        .collect();
    let margin = (sc.interline.main as f64 * 0.9).round_ties_even();
    let ay = |c: &Curve, x: f64| {
        if x < c.points[0][0] {
            c.points[0][1]
        } else if x > c.points.last().unwrap()[0] {
            c.points.last().unwrap()[1]
        } else {
            c.area_y(x)
        }
    };
    let next = bars["systems"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["inters"].as_array().unwrap())
        .map(|i| i["id"].as_u64().unwrap())
        .max()
        .unwrap_or(0)
        + 1;
    for (offset, brace) in completion["braces"]["braces"]
        .as_array()
        .ok_or("brace_stage_unavailable")?
        .iter()
        .enumerate()
    {
        let g = &brace["glyph"];
        let x = g["left"].as_f64().unwrap() + g["width"].as_f64().unwrap();
        let y = g["top"].as_f64().unwrap();
        let bottom = y + g["height"].as_f64().unwrap() - 1.;
        let sys = systems
            .iter()
            .find(|s| s["id"] == brace["system_id"])
            .unwrap();
        let ids = sys["staff_ids"].as_array().unwrap();
        let closest = |y: f64| {
            ids.iter()
                .filter_map(|id| {
                    let i = id.as_u64().unwrap() as usize - 1;
                    let n = if i == 0 {
                        0.
                    } else {
                        ay(curves[i - 1].last().unwrap(), x) + margin
                    };
                    let b = if i + 1 == curves.len() {
                        height as f64
                    } else {
                        ay(&curves[i + 1][0], x) - margin
                    };
                    if y < n || y >= b {
                        return None;
                    }
                    Some((
                        i,
                        (curves[i][0].y(x) - y).max(y - curves[i].last().unwrap().y(x)) as i32,
                    ))
                })
                .min_by_key(|p| p.1)
                .map(|p| p.0)
        };
        let a = closest(y).ok_or("brace_top_has_no_staff")?;
        let b = closest(bottom).ok_or("brace_bottom_has_no_staff")?;
        let top = curves[a][0].y(x).round_ties_even() as i64;
        let bottom = curves[b].last().unwrap().y(x).round_ties_even() as i64;
        let gid = registry.register(g);
        let sys = bars["systems"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["id"] == brace["system_id"])
            .unwrap();
        sys["inters"].as_array_mut().unwrap().push(json!({"id":next+offset as u64,"kind":"BraceInter","shape":"BRACE","staff_id":Value::Null,"glyph_id":gid,"bounds":[g["left"],json!(top),g["width"],json!(bottom-top+1)],"grade":0.8,"contextual_grade":0.8,"frozen":false}));
    }
    let edges = bars["remaining_peak_edges"].as_array().unwrap();
    let start = |id: usize| -> Option<i64> {
        completion["bar_filtering"]["left_end_peaks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|k| k[0] == id)
            .and_then(|k| k[1].as_i64())
    };
    let connected = |id: usize, below: bool| {
        start(id).is_some_and(|x| {
            edges.iter().any(|e| {
                let p = &e[if below { "top" } else { "bottom" }];
                e["connection"] == true && p[0] == id && p[1].as_i64().unwrap() > x
            })
        })
    };
    let mut output = systems;
    for sys in &mut output {
        let ids: Vec<_> = sys["staff_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let mut groups: Vec<Value> = vec![];
        let mut active = std::collections::BTreeMap::<usize, usize>::new();
        for &id in &ids {
            let Some(startx) = start(id) else {
                continue;
            };
            let mut level = 0usize;
            let peaks = completion["bar_filtering"]["remaining_peaks"]
                .as_array()
                .unwrap();
            for key in peaks
                .iter()
                .filter(|k| k[0] == id && k[1].as_i64().unwrap() < startx)
                .rev()
            {
                level += 1;
                let bracket = completion["bar_filtering"]["brackets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|b| b["peak"] == *key);
                let above = edges
                    .iter()
                    .any(|e| e["connection"] == true && e["bottom"] == *key);
                let below = edges
                    .iter()
                    .any(|e| e["connection"] == true && e["top"] == *key);
                let begin = bracket.map(|b| b["top"] == true).unwrap_or(!above && below);
                let end = bracket.map(|b| b["bottom"] == true).unwrap_or(!below);
                if begin {
                    groups.push(json!({"number":level,"symbol":if bracket.is_some(){"bracket"}else{"square"},"barline":connected(id,true),"first_staff_id":id,"last_staff_id":id}));
                    active.insert(level, groups.len() - 1);
                } else if let Some(&i) = active.get(&level) {
                    groups[i]["last_staff_id"] = json!(id);
                    if end {
                        active.remove(&level);
                    }
                }
            }
            if let Some(brace) = completion["braces"]["portions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|b| b["staff_id"] == id)
            {
                level += 1;
                if brace["kind"] == "TOP" {
                    groups.push(json!({"number":level,"symbol":"brace","barline":connected(id,true),"first_staff_id":id,"last_staff_id":id}));
                    active.insert(level, groups.len() - 1);
                } else if let Some(&i) = active.get(&level) {
                    groups[i]["last_staff_id"] = json!(id);
                    if brace["kind"] == "BOTTOM" {
                        active.remove(&level);
                    }
                }
            }
        }
        let mut spans: Vec<_> = completion["braces"]["braces"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["system_id"] == sys["id"])
            .filter_map(|b| {
                let a = b["staff_ids"].as_array().unwrap();
                if a.len() != 2 {
                    return None;
                }
                let first = a[0].as_u64().unwrap() as usize;
                let last = a[1].as_u64().unwrap() as usize;
                (!connected(first, false) && !connected(last, true) && connected(first, true))
                    .then_some((first, last))
            })
            .collect();
        spans.sort();
        groups.retain(|g| {
            g["symbol"] != "brace"
                || !spans.iter().any(|&(first, last)| {
                    g["first_staff_id"] == first && g["last_staff_id"] == last
                })
        });
        sys["part_groups"] = json!(groups);
        let mut parts = Vec::<PartMembership>::new();
        let mut next = ids[0];
        for (first, last) in spans {
            for id in next..first {
                parts.push(PartMembership {
                    id: parts.len() as u64 + 1,
                    staff_ids: vec![id as u64],
                    system: sys["id"].as_u64().unwrap(),
                });
            }
            if last >= next {
                parts.push(PartMembership {
                    id: parts.len() as u64 + 1,
                    staff_ids: (first.max(next)..=last).map(|id| id as u64).collect(),
                    system: sys["id"].as_u64().unwrap(),
                });
                next = last + 1;
            }
        }
        for id in next..=*ids.last().unwrap() {
            parts.push(PartMembership {
                id: parts.len() as u64 + 1,
                staff_ids: vec![id as u64],
                system: sys["id"].as_u64().unwrap(),
            });
        }
        sys["parts"] = json!(parts);
        actual_parts.extend(parts);
    }
    let slope = completion["global_slope"].as_f64().unwrap();
    let norm = (1. + slope * slope).sqrt();
    let xs: Vec<_> = output
        .iter()
        .map(|s| {
            (s["coordinates"][0].as_f64().unwrap() + slope * s["coordinates"][1].as_f64().unwrap())
                / norm
        })
        .collect();
    let indent = (sc.interline.main as f64 * 2.).round_ties_even();
    let mut pages: Vec<Value> = vec![];
    for (i, s) in output.iter_mut().enumerate() {
        let indented = (i > 0 && xs[i] - xs[i - 1] >= indent)
            || (i + 1 < xs.len() && xs[i] - xs[i + 1] >= indent);
        s["indented"] = json!(indented);
        if pages.is_empty() || indented {
            pages.push(json!({"id":pages.len()+1,"movement_start":indented,"system_ids":[]}));
        }
        s["page_id"] = json!(pages.len());
        pages.last_mut().unwrap()["system_ids"]
            .as_array_mut()
            .unwrap()
            .push(s["id"].clone());
    }
    if !completion["braces"]["unsupported"]
        .as_array()
        .unwrap()
        .is_empty()
    {
        actual_parts.clear();
        for sys in &mut output {
            sys["parts"] = Value::Null;
            sys["parts_status"] = json!("unsupported_brace_topology");
        }
    }
    ir["systems"] = json!(output);
    ir["pages"] = json!(pages);
    bars["pending"] = json!([
        "bracket font metrics beyond Bravura interline 11/16",
        "non-GRID SIG relation types and contexts"
    ]);
    bars["status"] = json!("bar_brace_bracket_semantics_partial");
    ir["bar_semantics"] = bars;
    let physical_glyphs = registry.physical_glyphs()?;
    ir["glyphs"] = json!(registry.entries);
    ir["pending"] = json!([
        "brace fallback alignment (explicitly rejected); natural line-root trigger unverified",
        "bracket font metrics beyond Bravura interline 11/16",
        "non-GRID SIG relation types and contexts"
    ]);
    Ok(IntegratedGrid {
        physical_glyphs,
        report: ir,
        parts: actual_parts,
    })
}
