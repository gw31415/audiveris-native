// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// PeakGraph alignment, connection, pruning, getSystemTops and createSystems.
// Multi-column layouts and peaks requiring splitting are explicit unsupported boundaries.
use crate::{GrayImage, scale::Scale};
use serde_json::{Value, json};
#[derive(Clone)]
struct Peak {
    staff: usize,
    index: usize,
    xy: [i32; 4],
    grade: f64,
}
#[derive(Clone)]
struct Edge {
    a: usize,
    b: usize,
    align: f64,
    width: f64,
    grade: f64,
    connection: bool,
}
fn grade(values: &[(f64, f64)]) -> f64 {
    let mut product = 1.;
    let mut total = 0.;
    for &(v, w) in values {
        product *= v.clamp(0., 1.).powf(w);
        total += w;
    }
    0.8 * product.powf(1. / total)
}
fn snapshot(phase: &str, es: &[Edge], ps: &[Peak]) -> Value {
    json!({"phase":phase,"edges":es.iter().map(|e|{let a=&ps[e.a];let b=&ps[e.b];json!({"top":[a.staff+1,a.xy[0]as usize,a.xy[1]as usize],"bottom":[b.staff+1,b.xy[0]as usize,b.xy[1]as usize],"connection":e.connection,"grade":e.grade})}).collect::<Vec<_>>()})
}
fn core(image: &GrayImage, a: [i32; 4], b: [i32; 4]) -> (i32, f64) {
    let mut black = None;
    let mut white = None;
    let mut gap = 0;
    let mut count = 0;
    let y0 = a[3];
    let y1 = b[2];
    for y in y0..=y1 {
        let t = (y - y0) as f64 / (y1 - y0) as f64;
        let left = (a[0] as f64 + (b[0] - a[0]) as f64 * t).floor() as i32;
        let right = (a[1] as f64 + (b[1] - a[1]) as f64 * t).ceil() as i32;
        let empty = (left..=right).all(|x| {
            x < 0
                || y < 0
                || x >= image.width as i32
                || y >= image.height as i32
                || image.pixels[y as usize * image.width + x as usize] != 0
        });
        if empty {
            count += 1;
            white = Some(y);
            continue;
        }
        if let (Some(w), Some(b)) = (white, black) {
            gap = gap.max(w - b);
            white = None;
        }
        black = Some(y);
    }
    (gap, count as f64 / (y1 - y0 + 1) as f64)
}
fn group(ids: &[usize], staffs: &[Vec<usize>], ps: &[Peak], gap: i32) -> Vec<usize> {
    if ids.is_empty() {
        return Vec::new();
    }
    let first = ids[0];
    let all = &staffs[ps[first].staff];
    let i1 = ps[first].index;
    let last = *ids.last().unwrap();
    let i2 = ps[last].index;
    let (mut lo, mut hi, mut prev) = (i1, i2, first);
    for i in (0..i1).rev() {
        let p = all[i];
        if ps[prev].xy[0] - ps[p].xy[1] + 1 > gap {
            break;
        }
        lo = i;
        prev = p;
    }
    // Preserve upstream groupOf traversal's prevPeak state into its forward scan.
    for (i, &p) in all.iter().enumerate().skip(i2 + 1) {
        if ps[p].xy[0] - ps[prev].xy[1] + 1 > gap {
            break;
        }
        hi = i;
        prev = p;
    }
    all[lo..=hi].to_vec()
}
fn requires_split(staffs: &[Vec<usize>], ps: &[Peak], es: &[Edge], gap: i32) -> bool {
    for i in 0..ps.len() {
        if group(&[i], staffs, ps, gap).len() > 1 {
            continue;
        }
        for above in [true, false] {
            let mut partners: Vec<_> = es
                .iter()
                .filter(|e| e.connection && if above { e.b == i } else { e.a == i })
                .map(|e| if above { e.a } else { e.b })
                .collect();
            partners.sort_unstable();
            partners.dedup();
            let partners = group(&partners, staffs, ps, gap);
            if partners.len() != 2 {
                continue;
            }
            let a = ps[partners[0]].xy;
            let b = ps[partners[1]].xy;
            let w = ps[i].xy[1] - ps[i].xy[0] + 1;
            let w1 = a[1] - a[0] + 1;
            let w2 = b[1] - b[0] + 1;
            if w <= w1.max(w2) + 2 || b[0] - a[1] + 1 > gap {
                continue;
            }
            let total = w1 + w2;
            let span = b[1] - a[0] + 1;
            let ratio = ((total - w).abs() as f64 / total.max(w) as f64)
                .min((span - w).abs() as f64 / span.max(w) as f64);
            let mid =
                ps[i].xy[0] + (w as f64 * w1 as f64 / (w1 + w2) as f64).round_ties_even() as i32;
            if ratio <= 0.3 && mid > ps[i].xy[0] + 1 && mid < ps[i].xy[1] - 1 {
                return true;
            }
        }
    }
    false
}
pub fn analyze(
    image: &GrayImage,
    sc: &Scale,
    staff_result: &Value,
    sticks: &Value,
    slope: f64,
) -> Value {
    if sticks["status"] != "bar_sticks" {
        return json!({"status":"blocked","error":"bar_sticks_unavailable","grid_complete":false});
    }
    let st = staff_result["staffs"].as_array().unwrap();
    let neighbors = match alignment_neighbors(st) {
        Ok(v) => v,
        Err(error) => return json!({"status":"unsupported","error":error,"grid_complete":false}),
    };
    let px = |v: f64| (v * sc.interline.main as f64).round_ties_even() as i32;
    let mut ps = Vec::new();
    let mut staff_peaks = Vec::new();
    for (si, s) in sticks["staffs"].as_array().unwrap().iter().enumerate() {
        let mut ids = Vec::new();
        for (i, p) in s["sticks"].as_array().unwrap().iter().enumerate() {
            ids.push(ps.len());
            ps.push(Peak {
                staff: si,
                index: i,
                xy: std::array::from_fn(|k| p["peak"][k].as_i64().unwrap() as i32),
                grade: p["grade"].as_f64().unwrap(),
            });
        }
        staff_peaks.push(ids);
    }
    let mut es = Vec::new();
    for (si, ids) in staff_peaks.iter().enumerate() {
        for &a in ids {
            for &bi in &neighbors[si] {
                for &b in &staff_peaks[bi] {
                    let u = ps[a].xy;
                    let v = ps[b].xy;
                    let dy = (v[2] - u[3]) as f64;
                    let sl = ((v[0] - u[0]) as f64 / dy + slope)
                        .abs()
                        .min(((v[1] - u[1]) as f64 / dy + slope).abs());
                    let dw = (v[1] - v[0]) - (u[1] - u[0]);
                    if sl > 0.06 || dw.abs() > px(0.6) {
                        continue;
                    }
                    let align = 1. - sl / 0.06;
                    let width = 1. - dw.abs() as f64 / px(0.6) as f64;
                    es.push(Edge {
                        a,
                        b,
                        align,
                        width,
                        grade: grade(&[(align, 2.), (width, 1.)]),
                        connection: false,
                    });
                }
            }
        }
    }
    let mut stages = vec![snapshot("findAllAlignments", &es, &ps)];
    let mut connections = Vec::new();
    es.retain(|e| {
        let (gap, white) = core(image, ps[e.a].xy, ps[e.b].xy);
        if gap <= px(1.) && white <= 0.25 {
            let mut c = e.clone();
            c.connection = true;
            c.grade = grade(&[
                (e.align, 2.),
                (e.width, 1.),
                (1. - gap as f64 / px(1.) as f64, 2.),
                (1. - white / 0.25, 2.),
            ]);
            connections.push(c);
            false
        } else {
            true
        }
    });
    es.extend(connections);
    stages.push(snapshot("findConnections", &es, &ps));
    if requires_split(&staff_peaks, &ps, &es, px(0.4)) {
        return json!({"status":"unsupported","error":"merged_peak_split_required","grid_complete":false,"stages":stages});
    }
    stages.push(snapshot("splitMergedGroups", &es, &ps));
    let mut remove = std::collections::BTreeSet::new();
    for p in 0..ps.len() {
        for incoming in [true, false] {
            let indices: Vec<_> = es
                .iter()
                .enumerate()
                .filter(|(_, e)| if incoming { e.b == p } else { e.a == p })
                .map(|(i, _)| i)
                .collect();
            let mut best = None;
            let mut best_grade = 0.;
            for &i in &indices {
                let e = &es[i];
                let ctx = e.grade * ps[if incoming { e.a } else { e.b }].grade;
                let choose = match best {
                    None => true,
                    Some(b) => {
                        let prev: &Edge = &es[b];
                        if prev.connection {
                            e.connection && ctx > best_grade
                        } else {
                            e.connection || ctx > best_grade
                        }
                    }
                };
                if choose {
                    best = Some(i);
                    best_grade = ctx;
                }
            }
            for i in indices {
                if Some(i) != best {
                    remove.insert(i);
                }
            }
        }
    }
    es = es
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !remove.contains(i))
        .map(|(_, e)| e)
        .collect();
    stages.push(snapshot("purgeAlignments", &es, &ps));
    let mut tops = vec![None; st.len()];
    let mut connections: Vec<_> = es.iter().filter(|e| e.connection).collect();
    connections.sort_by_key(|e| (ps[e.a].staff, ps[e.a].xy[0], ps[e.b].staff, ps[e.b].xy[0]));
    for e in connections {
        let top = ps[e.a].staff;
        let bottom = ps[e.b].staff;
        if tops[top].is_none() {
            tops[top] = Some(top + 1);
        }
        if tops[bottom].is_none()
            && (ps[e.b].xy[0] as f64 - st[bottom]["left"].as_f64().unwrap()) > px(2.) as f64
        {
            let a = staff_peaks[top].last();
            let b = staff_peaks[bottom].last();
            let right = es
                .iter()
                .any(|e| e.connection && Some(&e.a) == a && Some(&e.b) == b);
            if Some(&e.b) == b || !right {
                continue;
            }
        }
        tops[bottom] = tops[top];
    }
    let tops: Vec<_> = tops
        .into_iter()
        .enumerate()
        .map(|(i, v)| v.unwrap_or(i + 1))
        .collect();
    let mut systems: Vec<Vec<usize>> = Vec::new();
    let mut prev = None;
    for (i, &t) in tops.iter().enumerate() {
        if prev.is_none_or(|p| p < t) {
            systems.push(vec![i + 1]);
            prev = Some(t);
        } else {
            systems.last_mut().unwrap().push(i + 1);
        }
    }
    json!({"status":if systems.is_empty(){"failed"}else{"initial_systems"},"error":if systems.is_empty(){Some("no_system_found")}else{None},"grid_complete":false,"scope":"default five-line original vertical neighbor chains and short-status compatibility; merged peak splitting and later BarsRetriever/completeLines boundaries remain","stages":stages,"system_tops":tops,"systems":systems.iter().enumerate().map(|(i,s)|json!({"id":i+1,"staff_ids":s})).collect::<Vec<_>>()})
}

/// Exact findAllAlignments candidate staff traversal. The first vertical
/// neighbor alone controls short compatibility; its full sorted chain is used.
pub fn alignment_neighbors(staffs: &[Value]) -> Result<Vec<Vec<usize>>, String> {
    if staffs.is_empty() {
        return Ok(Vec::new());
    }
    let plans = crate::system_area_geometry::staff_neighbors(staffs)?;
    if staffs
        .iter()
        .enumerate()
        .any(|(i, s)| s["id"].as_u64() != Some((i + 1) as u64))
    {
        return Err("unsupported_peak_graph_staff_id_order".into());
    }
    let short = plans
        .iter()
        .map(|p| !p["left"].is_null() || !p["right"].is_null())
        .collect::<Vec<_>>();
    let mut result = Vec::new();
    for (i, p) in plans.iter().enumerate() {
        let below = p["south"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize - 1)
            .collect::<Vec<_>>();
        if below.first().is_some_and(|&j| short[j] == short[i]) {
            result.push(below);
        } else {
            result.push(Vec::new());
        }
    }
    Ok(result)
}
