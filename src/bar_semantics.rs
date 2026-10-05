// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Partial BarsRetriever: ordinary bar peak pruning, width partition, inters and relations.
// Bracket ends/middles and semantic purge/endpoint ordering; full GRID remains incomplete.
use crate::{grid_ir::GlyphRegistry, scale::Scale};
use serde_json::{Value, json};
#[derive(Clone)]
struct Peak {
    staff: usize,
    p: [i64; 4],
    grade: f64,
    sections: Vec<Value>,
    key: Value,
    brace: bool,
    alive: bool,
    thick: bool,
    bracket: [bool; 3],
}
fn glyph(p: &Peak) -> Value {
    let mut pixels = std::collections::BTreeSet::new();
    for s in &p.sections {
        let pos = s["first_pos"].as_u64().unwrap() as usize;
        for (i, r) in s["runs"].as_array().unwrap().iter().enumerate() {
            let start = r[0].as_u64().unwrap() as usize;
            let len = r[1].as_u64().unwrap() as usize;
            for v in start..start + len {
                let (x, y) = if s["orientation"] == "horizontal" {
                    (v, pos + i)
                } else {
                    (pos + i, v)
                };
                pixels.insert((x, y));
            }
        }
    }
    let left = pixels.iter().map(|p| p.0).min().unwrap();
    let right = pixels.iter().map(|p| p.0).max().unwrap();
    let top = pixels.iter().map(|p| p.1).min().unwrap();
    let bottom = pixels.iter().map(|p| p.1).max().unwrap();
    let mut runs: Vec<[usize; 3]> = vec![];
    for (x, y) in pixels {
        if let Some(r) = runs.last_mut()
            && r[0] == x - left
            && r[1] + r[2] == y - top
        {
            r[2] += 1;
            continue;
        }
        runs.push([x - left, y - top, 1]);
    }
    json!({"left":left,"top":top,"width":right-left+1,"height":bottom-top+1,"orientation":"VERTICAL","runs":runs.iter().map(|r|json!({"position":r[0],"start":r[1],"length":r[2]})).collect::<Vec<_>>()})
}
fn ribbon(x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> [i64; 4] {
    let x = (x1.min(x2) - width / 2.).floor();
    let y = y1.min(y2).floor();
    [
        x as i64,
        y as i64,
        (x1.max(x2) + width / 2.).ceil() as i64 - x as i64,
        y1.max(y2).ceil() as i64 - y as i64,
    ]
}
fn load_peaks(sticks: &Value) -> Vec<Peak> {
    let mut ps = vec![];
    for s in sticks["staffs"].as_array().unwrap() {
        for v in s["sticks"].as_array().unwrap() {
            let staff = s["staff_id"].as_u64().unwrap() as usize;
            let p = std::array::from_fn(|i| v["peak"][i].as_i64().unwrap());
            ps.push(Peak {
                staff,
                p,
                grade: v["grade"].as_f64().unwrap(),
                sections: v["sections"].as_array().unwrap().clone(),
                key: json!([staff, p[0], p[1]]),
                brace: v["brace"] == true,
                alive: true,
                thick: false,
                bracket: [false; 3],
            });
        }
    }
    ps
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn filter(
    sc: &Scale,
    sticks: &Value,
    systems: &Value,
    completion: &Value,
    projection: &Value,
    width: usize,
    sections: &[crate::sections::Section],
) -> Result<Value, String> {
    let px = |r: f64| (sc.interline.main as f64 * r).round_ties_even() as i64;
    let mut ps = load_peaks(sticks);
    let mut edges = systems["stages"].as_array().unwrap().last().unwrap()["edges"]
        .as_array()
        .unwrap()
        .clone();
    let geom = completion["bar_geometry"].as_array().unwrap();
    let mut staffgeo = completion["endpoint_geometry"]["staffs"]
        .as_array()
        .unwrap()
        .clone();
    let mut left_ends = geom
        .iter()
        .filter_map(|g| {
            g["start_column"]
                .as_u64()
                .map(|i| g["columns"][i as usize]["peaks"].as_array().unwrap())
        })
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let mut removed = vec![];
    let remove = |ps: &mut Vec<Peak>,
                  keys: &[Value],
                  phase: &str,
                  edges: &mut Vec<Value>,
                  events: &mut Vec<Value>| {
        // deleteRelatedColumns removes every peak in each affected column.
        let mut keys = keys.to_vec();
        if phase != "purgeUnalignedBars" {
            for g in geom {
                for c in g["columns"].as_array().unwrap() {
                    let members = c["peaks"].as_array().unwrap();
                    if members.iter().any(|k| keys.contains(k)) {
                        for k in members {
                            if !keys.contains(k) {
                                keys.push(k.clone());
                            }
                        }
                    }
                }
            }
        }
        for p in ps.iter_mut().filter(|p| p.alive && keys.contains(&p.key)) {
            p.alive = false;
            events.push(json!({"phase":phase,"peak":p.key}));
        }
        edges.retain(|e| !keys.contains(&e["top"]) && !keys.contains(&e["bottom"]));
    };
    for g in geom {
        let start = g["start_column"].as_u64().map(|v| v as usize);
        for (i, c) in g["columns"].as_array().unwrap().iter().enumerate() {
            if start.is_none_or(|s| i > s) && c["full"] != true {
                remove(
                    &mut ps,
                    c["peaks"].as_array().unwrap(),
                    "purgePartialColumns",
                    &mut edges,
                    &mut removed,
                );
            }
        }
    }
    for s in &staffgeo {
        let id = s["id"].as_u64().unwrap() as usize;
        let mut keys = vec![];
        let Some(start) = ps
            .iter()
            .find(|p| p.alive && p.staff == id && left_ends.contains(&p.key))
        else {
            continue;
        };
        let left = start.p[1];
        let mut prev = start.p[0];
        for p in ps
            .iter()
            .filter(|p| p.staff == id && p.alive && p.p[1] < left)
            .rev()
        {
            if prev - p.p[1] + 1 > px(2.) {
                keys.push(p.key.clone());
            } else {
                prev = p.p[0];
            }
        }
        remove(&mut ps, &keys, "purgeTooLeft", &mut edges, &mut removed);
    }
    // purgeLeftOfBraces, before root validation and ordinary left-peak purge.
    for portion in completion["braces"]["portions"].as_array().unwrap() {
        let id = portion["staff_id"].as_u64().unwrap() as usize;
        let start = ps
            .iter()
            .find(|p| p.alive && p.staff == id && left_ends.contains(&p.key))
            .map(|p| p.p[0]);
        if let Some(start) = start {
            let brace_start = portion["peak"][0].as_i64().unwrap();
            let keys = ps
                .iter()
                .filter(|p| p.alive && p.staff == id && p.p[0] < start && p.p[1] < brace_start)
                .map(|p| p.key.clone())
                .collect::<Vec<_>>();
            remove(
                &mut ps,
                &keys,
                "purgeLeftOfBraces",
                &mut edges,
                &mut removed,
            );
        }
    }
    // verifyLinesRoot: only one-staff systems; start status is no longer inferred
    // from old geometry after this correction.
    for system in systems["systems"].as_array().unwrap() {
        let ids = system["staff_ids"].as_array().unwrap();
        if ids.len() != 1 {
            continue;
        }
        let id = ids[0].as_u64().unwrap() as usize;
        if completion["braces"]["portions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["staff_id"] == id)
        {
            continue;
        }
        let active = ps
            .iter()
            .filter(|p| p.alive && p.staff == id)
            .collect::<Vec<_>>();
        if let Some(left) = line_root_correction(
            &projection["staff_projections"][id - 1],
            active.first().map(|p| p.p[0]),
            active.iter().any(|p| left_ends.contains(&p.key)),
            false,
            sc.interline.main,
        ) {
            staffgeo[id - 1]["left"] = json!(left);
            left_ends.retain(|key| key[0] != id);
        }
    }
    let bracket_serifs = detect_brackets(&mut ps, &edges, &left_ends, systems, sc, sections)?;
    // Ordinary-bar branch: retain brace candidates separately rather than invent brace inters.
    let keys: Vec<_> = ps
        .iter()
        .filter(|p| {
            p.alive
                && !p.brace
                && !p.bracket.iter().any(|v| *v)
                && p.p[0] <= staffgeo[p.staff - 1]["left"].as_f64().unwrap() as i64
                && !left_ends.contains(&p.key)
        })
        .map(|p| p.key.clone())
        .collect();
    remove(&mut ps, &keys, "purgeLeftPeaks", &mut edges, &mut removed);
    for s in systems["systems"].as_array().unwrap() {
        let ids = s["staff_ids"].as_array().unwrap();
        if ids.len() > 1 {
            let keys: Vec<_> = ps
                .iter()
                .filter(|p| {
                    p.alive
                        && ids.contains(&json!(p.staff))
                        && !edges
                            .iter()
                            .any(|e| e["top"] == p.key || e["bottom"] == p.key)
                })
                .map(|p| p.key.clone())
                .collect();
            remove(
                &mut ps,
                &keys,
                "purgeUnalignedBars",
                &mut edges,
                &mut removed,
            );
        }
        if ids.len() < 4 {
            for (top, id) in [
                (true, ids[0].as_u64().unwrap() as usize),
                (false, ids.last().unwrap().as_u64().unwrap() as usize),
            ] {
                let left = staffgeo[id - 1]["left"].as_f64().unwrap() as i64;
                let keys: Vec<_> = ps
                    .iter()
                    .filter(|p| p.alive && p.staff == id && p.p[0] > left)
                    .filter(|p| {
                        let g = glyph(p);
                        let y = g["top"].as_f64().unwrap();
                        let h = g["height"].as_f64().unwrap();
                        let ext = if top {
                            p.p[2] as f64 - sc.line.max as f64 / 2. - y
                        } else {
                            y + h - 1. - sc.line.max as f64 / 2. - p.p[3] as f64
                        };
                        ext > px(1.) as f64
                    })
                    .map(|p| p.key.clone())
                    .collect();
                remove(
                    &mut ps,
                    &keys,
                    "purgeExtendingPeaks",
                    &mut edges,
                    &mut removed,
                );
            }
        }
    }

    // Right endpoints must see peaks surviving the preceding semantic purges.
    let mut right_ends = vec![];
    for s in &mut staffgeo {
        let id = s["id"].as_u64().unwrap() as usize;
        let right = s["right"]
            .as_i64()
            .unwrap_or_else(|| s["right"].as_f64().unwrap() as i64);
        let last = ps.iter().rfind(|p| p.alive && p.staff == id);
        let end = last.filter(|p| (p.p[0] + p.p[1]) / 2 >= right);
        let at = end.map(|p| p.p[1]).unwrap_or(right);
        let xmax = select_blank(&projection["staff_projections"][id - 1], at, px(0.1), false)
            .map(|b| b[0] - 1)
            .unwrap_or(width as i64 - 1);
        s["right"] = json!(match end {
            Some(p) if xmax - p.p[1] <= px(0.3) => {
                right_ends.push(p.key.clone());
                (p.p[0] + p.p[1]) / 2
            }
            _ => xmax,
        });
    }
    let before_cclef = ps
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.key.clone())
        .collect::<Vec<_>>();
    for s in &staffgeo {
        let id = s["id"].as_u64().unwrap() as usize;
        let left = s["left"].as_f64().unwrap() as i64;
        let mut measure_start = left;
        let mut i = 0;
        loop {
            let indices: Vec<_> = ps
                .iter()
                .enumerate()
                .filter(|(_, p)| p.alive && p.staff == id)
                .map(|(i, _)| i)
                .collect();
            if i >= indices.len() {
                break;
            }
            let p1 = &ps[indices[i]];
            if p1.p[0] > measure_start {
                if !left_ends.contains(&p1.key)
                    && !right_ends.contains(&p1.key)
                    && !p1.brace
                    && !p1.bracket.iter().any(|v| *v)
                    && p1.p[1] - p1.p[0] + 1 >= px(0.3)
                {
                    let gap = p1.p[0] - measure_start;
                    let min_gap = if measure_start == left { 0 } else { px(0.75) };
                    let connected = |p: &Peak| {
                        edges.iter().any(|e| {
                            e["connection"] == true && (e["top"] == p.key || e["bottom"] == p.key)
                        })
                    };
                    if gap > min_gap && gap < px(2.) && !connected(p1) {
                        if let Some(&j) = indices.get(i + 1) {
                            let p2 = &ps[j];
                            if p2.p[1] - p2.p[0] < px(0.3)
                                && p2.p[0] - p1.p[1] - 1 <= px(0.75)
                                && !connected(p2)
                            {
                                let xbreak = (p2.p[0] + p2.p[1]) / 2 + px(2.);
                                let tails: Vec<_> = indices
                                    .iter()
                                    .skip(i + 2)
                                    .map(|&j| &ps[j])
                                    .take_while(|p| (p.p[0] + p.p[1]) / 2 < xbreak)
                                    .collect();
                                if !tails.iter().any(|p| connected(p)) {
                                    let mut keys = vec![p1.key.clone(), p2.key.clone()];
                                    keys.extend(tails.iter().map(|p| p.key.clone()));
                                    // Preserve source list-mutation/index behavior, including its skip.
                                    i += keys.len() - 1;
                                    remove(&mut ps, &keys, "purgeCClefs", &mut edges, &mut removed);
                                }
                            }
                        }
                    } else {
                        measure_start = p1.p[1] + 1;
                    }
                } else {
                    measure_start = p1.p[1] + 1;
                }
            }
            i += 1;
        }
    }
    Ok(
        json!({"removed_peaks":removed,"remaining_peak_edges":edges,"staffs":staffgeo,
        "left_end_peaks":left_ends,"bracket_serifs":bracket_serifs,"brackets":ps.iter().filter(|p|p.alive&&p.bracket.iter().any(|v|*v)).map(|p|json!({"peak":p.key,"top":p.bracket[0],"middle":p.bracket[1],"bottom":p.bracket[2]})).collect::<Vec<_>>(),"right_end_peaks":right_ends,"before_cclef":before_cclef,
        "remaining_peaks":ps.iter().filter(|p|p.alive).map(|p|p.key.clone()).collect::<Vec<_>>()}),
    )
}
/// StaffProjector.checkLinesRoot; returns the corrected abscissa and signals
/// that the caller must unset the existing STAFF_LEFT_END attribute.
pub fn line_root_correction(
    projection: &Value,
    first_start: Option<i64>,
    has_start: bool,
    has_brace: bool,
    interline: usize,
) -> Option<i64> {
    if has_brace || !has_start {
        return None;
    }
    let first = first_start?;
    let min_width = (interline as f64 * 0.1).round_ties_even() as i64;
    let blank = select_blank(projection, first, min_width, true)?;
    (first - 1 - blank[1] > interline as i64).then_some(blank[1] + 1)
}

fn select_blank(projection: &Value, start: i64, min_width: i64, left: bool) -> Option<[i64; 2]> {
    let mut blanks = projection["blanks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| [v[0].as_i64().unwrap(), v[1].as_i64().unwrap()])
        .collect::<Vec<_>>();
    if left {
        blanks.reverse();
    }
    blanks.into_iter().find(|b| {
        let mid = (b[0] + b[1]) / 2;
        (if left { mid < start } else { mid > start }) && b[1] - b[0] + 1 >= min_width
    })
}
pub fn build(
    sc: &Scale,
    sticks: &Value,
    systems: &Value,
    completion: &Value,
    registry: &mut GlyphRegistry,
) -> Result<Value, String> {
    let px = |r: f64| (sc.interline.main as f64 * r).round_ties_even() as i64;
    let mut ps = load_peaks(sticks);
    let filtering = &completion["bar_filtering"];
    let removed = filtering["removed_peaks"].as_array().unwrap().clone();
    for p in &mut ps {
        p.alive = !removed.iter().any(|v| v["peak"] == p.key);
        if let Some(b) = filtering["brackets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["peak"] == p.key)
        {
            p.bracket = [b["top"] == true, b["middle"] == true, b["bottom"] == true];
        }
    }
    let edges = filtering["remaining_peak_edges"]
        .as_array()
        .unwrap()
        .clone();
    let staffgeo = filtering["staffs"].as_array().unwrap();
    let serif_metrics: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/bracket-font-metrics.json"
    ))
    .unwrap();
    let metrics = serif_metrics
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["interline"] == sc.interline.main);
    if ps.iter().any(|p| p.alive && p.bracket.iter().any(|v| *v)) && metrics.is_none() {
        return Err("unsupported_bracket_font_metrics: Bravura interline 11/16 only".into());
    }
    // groupBarPeaks + partitionWidths; brace candidates break ordinary-bar groups.
    for s in staffgeo {
        let id = s["id"].as_u64().unwrap() as usize;
        let mut groups: Vec<Vec<usize>> = vec![];
        let mut prev = None;
        for (i, p) in ps
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive && p.staff == id)
        {
            if p.brace || p.bracket.iter().any(|v| *v) {
                prev = None;
                continue;
            }
            if let Some(j) = prev {
                let q: &Peak = &ps[j];
                if p.p[0] - q.p[1] - 1 <= px(0.75) {
                    groups.last_mut().unwrap().push(i);
                } else {
                    groups.push(vec![i]);
                }
            } else {
                groups.push(vec![i]);
            }
            prev = Some(i);
        }
        for g in groups {
            let min = g
                .iter()
                .map(|&i| ps[i].p[1] - ps[i].p[0] + 1)
                .min()
                .unwrap();
            let max = g
                .iter()
                .map(|&i| ps[i].p[1] - ps[i].p[0] + 1)
                .max()
                .unwrap();
            if (max - min) as f64 / sc.interline.main as f64 >= 0.2 {
                for i in g {
                    let w = ps[i].p[1] - ps[i].p[0] + 1;
                    ps[i].thick = w - min > max - w;
                }
            }
        }
    }
    let mut out = vec![];
    let mut next_id = 1usize;
    for system in systems["systems"].as_array().unwrap() {
        let ids = system["staff_ids"].as_array().unwrap();
        let members: Vec<_> = ps
            .iter()
            .filter(|p| p.alive && !p.brace && ids.contains(&json!(p.staff)))
            .collect();
        let mut inters = vec![];
        let mut relations = vec![];
        let mut keys = vec![];
        let mut frozen_peaks = vec![];
        for p in &members {
            let width = (p.p[1] - p.p[0] + 1) as f64;
            let x = p.p[0] as f64 + width / 2.;
            let half = sc.line.main as f64 / 2.;
            let gid = registry.register(&glyph(p));
            let inter_id = next_id;
            next_id += 1;
            keys.push((p.key.clone(), inter_id));
            let bracket = p.bracket.iter().any(|v| *v);
            let mut bounds = ribbon(
                x,
                p.p[2] as f64 - half + 0.5,
                x,
                p.p[3] as f64 + half + 0.5,
                width,
            );
            if bracket {
                let metrics = &metrics.unwrap()["bounds"];
                let top_y = p.p[2] as f64 - half + 0.5;
                let bottom_y = p.p[3] as f64 + half + 0.5;
                let upper = &metrics["BRACKET_UPPER_SERIF"];
                let lower = &metrics["BRACKET_LOWER_SERIF"];
                let top = if p.bracket[0] {
                    top_y - upper[3].as_f64().unwrap()
                } else {
                    top_y
                };
                let bottom = if p.bracket[2] {
                    bottom_y + lower[3].as_f64().unwrap()
                } else {
                    bottom_y
                };
                let w = width
                    .max(if p.bracket[0] {
                        upper[2].as_f64().unwrap()
                    } else {
                        0.
                    })
                    .max(if p.bracket[2] {
                        lower[2].as_f64().unwrap()
                    } else {
                        0.
                    });
                bounds = [
                    p.p[0],
                    top.floor() as i64,
                    (p.p[0] as f64 + w).ceil() as i64 - p.p[0],
                    bottom.ceil() as i64 - top.floor() as i64,
                ];
            }
            inters.push(json!({"id":inter_id,"kind":if bracket{"BracketInter"}else{"BarlineInter"},"shape":if bracket{"BRACKET"}else if p.thick{"THICK_BARLINE"}else{"THIN_BARLINE"},"staff_id":p.staff,"glyph_id":gid,"bounds":bounds,"median":[x,p.p[2] as f64-half+0.5,x,p.p[3] as f64+half+0.5],"width":width,"grade":p.grade,"contextual_grade":p.grade,"source_peak":p.key,"frozen":false}));
            if bracket {
                inters.last_mut().unwrap()["bracket_kind"] =
                    json!(match (p.bracket[0], p.bracket[2]) {
                        (true, true) => "BOTH",
                        (true, false) => "TOP",
                        (false, true) => "BOTTOM",
                        _ => "NONE",
                    });
            }
        }
        for e in &edges {
            if e["connection"] != true {
                continue;
            }
            let a = members.iter().find(|p| p.key == e["top"]);
            let b = members.iter().find(|p| p.key == e["bottom"]);
            if let (Some(a), Some(b)) = (a, b) {
                let id = next_id;
                next_id += 1;
                let aid = keys.iter().find(|(k, _)| *k == a.key).unwrap().1;
                let bid = keys.iter().find(|(k, _)| *k == b.key).unwrap().1;
                let wa = (a.p[1] - a.p[0] + 1) as f64;
                let wb = (b.p[1] - b.p[0] + 1) as f64;
                let half = sc.line.main as f64 / 2.;
                // BarConnectorInter overrides isGood: the threshold is 0.65,
                // not AbstractInter's intrinsicRatio-scaled goodInterGrade.
                let frozen = !a.bracket.iter().any(|v| *v) && e["grade"].as_f64().unwrap() >= 0.65;
                if frozen && !frozen_peaks.contains(&a.key) {
                    frozen_peaks.push(a.key.clone());
                }
                inters.push(json!({"id":id,"kind":if a.bracket.iter().any(|v|*v){"BracketConnectorInter"}else{"BarConnectorInter"},"shape":if a.bracket.iter().any(|v|*v){"BRACKET_CONNECTOR"}else if a.thick{"THICK_CONNECTOR"}else{"THIN_CONNECTOR"},"staff_id":Value::Null,"glyph_id":Value::Null,"bounds":ribbon(a.p[0]as f64+wa/2.,a.p[3]as f64+half+0.5,b.p[0]as f64+wb/2.,b.p[2]as f64-half+0.5,(wa+wb)/2.),"median":[a.p[0]as f64+wa/2.,a.p[3]as f64+half+0.5,b.p[0]as f64+wb/2.,b.p[2]as f64-half+0.5],"width":(wa+wb)/2.,"grade":e["grade"],"contextual_grade":e["grade"],"frozen":frozen}));
                relations.push(json!({"kind":"NoExclusion","source":aid,"target":id}));
                relations.push(json!({"kind":"NoExclusion","source":id,"target":bid}));
                relations.push(json!({"kind":"BarConnectionRelation","source":aid,"target":bid,"grade":e["grade"]}));
            }
        }
        // BarsRetriever.extendConnection walks all surviving alignments,
        // including edges which are not physical connections.
        let mut cursor = 0;
        while cursor < frozen_peaks.len() {
            let key = frozen_peaks[cursor].clone();
            for e in &edges {
                if e["top"] == key || e["bottom"] == key {
                    for side in ["top", "bottom"] {
                        if !frozen_peaks.contains(&e[side]) {
                            frozen_peaks.push(e[side].clone());
                        }
                    }
                }
            }
            cursor += 1;
        }
        for inter in &mut inters {
            if !inter["source_peak"].is_null() {
                inter["frozen"] = json!(frozen_peaks.contains(&inter["source_peak"]));
            }
        }
        for id in ids {
            let m: Vec<_> = members
                .iter()
                .filter(|p| json!(p.staff) == *id && !p.bracket.iter().any(|v| *v))
                .collect();
            for w in m.windows(2) {
                let gap = w[1].p[0] - w[0].p[1] - 1;
                if gap <= px(0.75) {
                    let a = keys.iter().find(|(k, _)| *k == w[0].key).unwrap().1;
                    let b = keys.iter().find(|(k, _)| *k == w[1].key).unwrap().1;
                    relations.push(json!({"kind":"BarGroupRelation","source":a,"target":b,"x_gap":gap as f64/sc.interline.main as f64}));
                }
            }
        }
        crate::sig_context::contextualize(&mut inters, &relations)?;
        out.push(json!({"id":system["id"],"inters":inters,"relations":relations}));
    }
    Ok(
        json!({"status":"bar_bracket_semantics_partial","systems":out,"left_end_peaks":filtering["left_end_peaks"],"right_end_peaks":filtering["right_end_peaks"],"remaining_peak_edges":edges,"removed_peaks":removed,"pending":["non-GRID SIG relation types and contexts","uncovered nondefault and brace fallback branches"],"grid_complete":false}),
    )
}

// BarsRetriever.detectBracketEnds/getSerif/buildSerifFilament/detectBracketMiddles.
fn detect_brackets(
    ps: &mut [Peak],
    edges: &[Value],
    left_ends: &[Value],
    systems: &Value,
    sc: &Scale,
    all: &[crate::sections::Section],
) -> Result<Vec<Value>, String> {
    use crate::{bar_sticks::touch, sections::Orientation};
    let px = |v: f64| (sc.interline.main as f64 * v).round_ties_even() as i64;
    let min_weight = (sc.interline.main.pow(2) as f64 * 0.2).round_ties_even() as usize;
    let mut serifs = vec![];
    for sys in systems["systems"].as_array().unwrap() {
        let ids = sys["staff_ids"].as_array().unwrap();
        if ids.len() < 2 {
            continue;
        }
        for id in ids {
            let id = id.as_u64().unwrap() as usize;
            let active = ps
                .iter()
                .enumerate()
                .filter(|(_, p)| p.alive && p.staff == id)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let Some(start) = active.iter().position(|&i| left_ends.contains(&ps[i].key)) else {
                continue;
            };
            for k in (0..start).rev() {
                let i = active[k];
                let p = &ps[i];
                if p.brace {
                    break;
                }
                if p.p[1] - p.p[0] + 1 < px(0.25) {
                    continue;
                }
                let g = glyph(p);
                for top in [true, false] {
                    let ext = if top {
                        p.p[2] as f64 - sc.line.max as f64 / 2. - g["top"].as_f64().unwrap()
                    } else {
                        g["top"].as_f64().unwrap() + g["height"].as_f64().unwrap()
                            - 1.
                            - sc.line.max as f64 / 2.
                            - p.p[3] as f64
                    };
                    if ext > px(1.25) as f64 {
                        continue;
                    }
                    let half = (sc.line.max as f64 / 2.).ceil() as i64;
                    let roi = [
                        p.p[1] + 1,
                        if top {
                            p.p[2] - half - px(2.)
                        } else {
                            p.p[3] + half
                        },
                        px(2.),
                        px(2.),
                    ];
                    let excluded = ps[i]
                        .sections
                        .iter()
                        .chain(&ps[active[k + 1]].sections)
                        .collect::<Vec<_>>();
                    let candidates = all
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| {
                            let b = s.bounds();
                            let intersects = (b.x as i64) < roi[0] + roi[2]
                                && roi[0] < (b.x + b.width) as i64
                                && (b.y as i64) < roi[1] + roi[3]
                                && roi[1] < (b.y + b.height) as i64;
                            let orientation = if s.orientation == Orientation::Horizontal {
                                "horizontal"
                            } else {
                                "vertical"
                            };
                            intersects
                                && !excluded
                                    .iter()
                                    .any(|e| e["id"] == s.id.0 && e["orientation"] == orientation)
                        })
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
                    if candidates.len() > 512 {
                        return Err("bracket_serif_candidate_budget_exceeded".into());
                    }
                    if candidates.is_empty() {
                        continue;
                    }
                    let mut components: Vec<Vec<usize>> = vec![];
                    let mut unvisited = candidates;
                    while let Some(seed) = unvisited.pop() {
                        let mut component = vec![seed];
                        let mut at = 0;
                        while at < component.len() {
                            let mut j = 0;
                            while j < unvisited.len() {
                                if touch(&all[component[at]], &all[unvisited[j]]) {
                                    component.push(unvisited.remove(j));
                                } else {
                                    j += 1;
                                }
                            }
                            at += 1;
                        }
                        components.push(component);
                    }
                    let vertex = [
                        roi[0] as f64,
                        (roi[1] + if top { roi[3] - 1 } else { 0 }) as f64,
                    ];
                    let distance = |m: &Vec<usize>| {
                        let (mut sx, mut sy, mut count) = (0usize, 0usize, 0usize);
                        for &i in m {
                            let s = &all[i];
                            for (j, r) in s.runs.iter().enumerate() {
                                let along = r.length * (2 * r.start + r.length - 1);
                                let across = 2 * r.length * (s.first_pos + j);
                                let (x, y) = if s.orientation == Orientation::Horizontal {
                                    (along, across)
                                } else {
                                    (across, along)
                                };
                                sx += x;
                                sy += y;
                                count += r.length;
                            }
                        }
                        let x = (sx as f64 / (2 * count) as f64).round_ties_even();
                        let y = (sy as f64 / (2 * count) as f64).round_ties_even();
                        (x - vertex[0]).hypot(y - vertex[1])
                    };
                    components.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
                    let mut chosen = vec![];
                    let mut weight = 0;
                    for m in components {
                        weight += m.iter().map(|&i| all[i].weight).sum::<usize>();
                        chosen.extend(m);
                        if weight >= min_weight {
                            break;
                        }
                    }
                    if weight < min_weight {
                        continue;
                    }
                    serifs.push(json!({"peak":p.key,"side":if top {"TOP"}else{"BOTTOM"},"weight":weight,
                        "section_keys":chosen.iter().map(|&i|json!([if all[i].orientation==Orientation::Horizontal {"HORIZONTAL"}else{"VERTICAL"},all[i].id.0])).collect::<Vec<_>>()}));
                }
            }
        }
    }
    for s in &serifs {
        if let Some(p) = ps.iter_mut().find(|p| p.key == s["peak"]) {
            p.bracket[if s["side"] == "TOP" { 0 } else { 2 }] = true;
        }
    }
    let tops = ps
        .iter()
        .enumerate()
        .filter(|(_, p)| p.alive && p.bracket[0])
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    for top in tops {
        let mut current = top;
        let mut visited = std::collections::BTreeSet::new();
        while visited.insert(current) {
            let mut next = None;
            let mut bottom = false;
            let current_key = ps[current].key.clone();
            for e in edges
                .iter()
                .filter(|e| e["connection"] == true && e["top"] == current_key)
            {
                if let Some(i) = ps.iter().position(|p| p.alive && p.key == e["bottom"]) {
                    if ps[i].bracket[2] {
                        bottom = true;
                        break;
                    }
                    ps[i].bracket[1] = true;
                    next = Some(i);
                }
            }
            if bottom {
                break;
            }
            let Some(i) = next else {
                break;
            };
            current = i;
        }
    }
    Ok(serifs)
}
