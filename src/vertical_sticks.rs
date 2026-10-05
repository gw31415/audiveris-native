// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native orientation-aware StickFactory at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Recovered from the authorized pinned local JAR; no runtime JVM.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SeedSection {
    pub id: u64,
    pub vertical: bool,
    pub first_pos: i64,
    pub runs: Vec<[i64; 2]>,
}
impl SeedSection {
    pub fn bounds(&self) -> [i64; 4] {
        let lo = self.runs.iter().map(|r| r[0]).min().unwrap();
        let hi = self.runs.iter().map(|r| r[0] + r[1]).max().unwrap();
        if self.vertical {
            [self.first_pos, lo, self.runs.len() as i64, hi - lo]
        } else {
            [lo, self.first_pos, hi - lo, self.runs.len() as i64]
        }
    }
    pub fn weight(&self) -> i64 {
        self.runs.iter().map(|r| r[1]).sum()
    }
    pub fn snapshot(&self) -> Value {
        json!({"id":self.id,"vertical":self.vertical,"first_pos":self.first_pos,"bounds":self.bounds(),"weight":self.weight(),"runs":self.runs})
    }
    fn key(&self) -> (i64, i64, u64) {
        let b = self.bounds();
        (b[0], b[1], self.id)
    }
    fn coord(&self) -> i64 {
        self.runs.iter().map(|r| r[0]).min().unwrap()
    }
    fn length(&self) -> i64 {
        self.bounds()[if self.vertical { 3 } else { 2 }]
    }
    fn pixels(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        self.runs.iter().enumerate().flat_map(move |(i, r)| {
            (r[0]..r[0] + r[1]).map(move |v| {
                if self.vertical {
                    (self.first_pos + i as i64, v)
                } else {
                    (v, self.first_pos + i as i64)
                }
            })
        })
    }
}
fn sort_members(members: &mut Vec<usize>, sections: &[SeedSection]) {
    members.sort_by_key(|&i| sections[i].key());
    members.dedup_by_key(|i| sections[*i].key());
}
fn box_of(m: &[usize], s: &[SeedSection]) -> [i64; 4] {
    let bs = m.iter().map(|&i| s[i].bounds()).collect::<Vec<_>>();
    let x = bs.iter().map(|b| b[0]).min().unwrap();
    let y = bs.iter().map(|b| b[1]).min().unwrap();
    [
        x,
        y,
        bs.iter().map(|b| b[0] + b[2]).max().unwrap() - x,
        bs.iter().map(|b| b[1] + b[3]).max().unwrap() - y,
    ]
}
fn thickness(m: &[usize], s: &[SeedSection], major: usize) -> i64 {
    (m.iter().map(|&i| s[i].weight()).sum::<i64>() as f64 / box_of(m, s)[major] as f64)
        .round_ties_even() as i64
}
fn common(a: [i64; 2], b: [i64; 2]) -> i64 {
    ((a[0] + a[1]).min(b[0] + b[1]) - a[0].max(b[0])).max(0)
}

/// Exact membership ordering; explicit budgets replace an unbounded allocation.
pub fn retrieve(
    vertical: &[SeedSection],
    horizontal: &[SeedSection],
    max_thickness: i64,
    min_core: i64,
) -> Result<Vec<Value>, String> {
    retrieve_oriented(vertical, horizontal, max_thickness, min_core, true, 0.4)
}

pub(crate) fn retrieve_oriented(
    vertical: &[SeedSection],
    horizontal: &[SeedSection],
    max_thickness: i64,
    min_core: i64,
    main_vertical: bool,
    min_side_ratio: f64,
) -> Result<Vec<Value>, String> {
    let major = if main_vertical { 3 } else { 2 };
    let coord = major - 2;
    if !min_side_ratio.is_finite() || !(0.0..=1.0).contains(&min_side_ratio) {
        return Err("invalid_stick_side_ratio".into());
    }
    if !(1..=128).contains(&max_thickness)
        || !(1..=8192).contains(&min_core)
        || vertical.len() + horizontal.len() > 32768
    {
        return Err("unsupported_vertical_stick_budget".into());
    }
    let mut total = 0;
    let mut run_count = 0;
    let mut ids = BTreeSet::new();
    for (wanted, sections) in [(main_vertical, vertical), (!main_vertical, horizontal)] {
        for s in sections {
            if s.vertical != wanted
                || !ids.insert((wanted, s.id))
                || s.runs.is_empty()
                || s.runs.len() > 8192
                || !(0..=4_000_000).contains(&s.first_pos)
                || s.first_pos + s.runs.len() as i64 > 4_000_000
                || s.runs.iter().any(|r| {
                    !(0..=4_000_000).contains(&r[0])
                        || !(1..=4_000_000).contains(&r[1])
                        || r[0] + r[1] > 4_000_000
                })
            {
                return Err("invalid_vertical_stick_section".into());
            }
            total += s.weight();
            run_count += s.runs.len();
            if total > 8_000_000 || run_count > 1_048_576 {
                return Err("unsupported_vertical_stick_pixel_budget".into());
            }
            if wanted != main_vertical && s.bounds()[if main_vertical { 2 } else { 3 }] != 1 {
                return Err("unsupported_vertical_opposite_sticker".into());
            }
        }
    }
    let vs = vertical.to_vec();
    let mut hs = horizontal.to_vec();
    hs.sort_by_key(SeedSection::coord);
    let n = vs.len();
    let mut sections = vs;
    sections.extend(hs);
    let mut edges = vec![[Vec::new(), Vec::new()]; n]; // sources then targets
    // SectionTally assumes ordered input but does not reorder it: preserve its
    // recorded increasing starts and contiguous sublist semantics exactly.
    let mut starts = BTreeMap::<i64, usize>::new();
    let mut previous = -1;
    for (i, s) in sections[..n].iter().enumerate() {
        if s.first_pos > previous {
            starts.insert(s.first_pos, i);
            previous = s.first_pos;
        }
    }
    let mut edge_count = 0;
    for i in 0..n {
        let s = &sections[i];
        let last = *s.runs.last().unwrap();
        let position = s.first_pos + s.runs.len() as i64;
        if let Some(&begin) = starts.get(&position) {
            let end = starts
                .range((
                    std::ops::Bound::Excluded(position),
                    std::ops::Bound::Unbounded,
                ))
                .next()
                .map(|(_, i)| *i)
                .unwrap_or(n);
            for j in begin..end {
                let first = sections[j].runs[0];
                if first[0] >= last[0] + last[1] {
                    break;
                }
                if first[0] + first[1] > last[0] {
                    edges[i][1].push(j);
                    edges[j][0].push(i);
                    edge_count += 1;
                    if edge_count > 262144 {
                        return Err("unsupported_vertical_stick_edge_budget".into());
                    }
                }
            }
        }
    }
    let mut opposite = BTreeMap::<i64, Vec<usize>>::new();
    for (i, s) in sections.iter().enumerate().skip(n) {
        opposite.entry(s.coord()).or_default().push(i);
    }
    let mut cores = (0..n)
        .filter(|&i| {
            sections[i].runs.len() as i64 <= max_thickness && sections[i].length() >= min_core
        })
        .collect::<Vec<_>>();
    cores.sort_by_key(|&i| std::cmp::Reverse(sections[i].length()));
    let mut processed = vec![false; n];
    let mut compound = vec![None; n];
    let mut answer = Vec::new();
    for core in cores {
        if processed[core] {
            continue;
        }
        let owner = answer.len();
        let mut members = vec![core];
        processed[core] = true;
        compound[core] = Some(owner);
        let mut closed = [false; 2];
        loop {
            let mut grown = false;
            for side in [0, 1] {
                if closed[side] {
                    continue;
                }
                let mean = thickness(&members, &sections, major);
                let sides = members
                    .iter()
                    .copied()
                    .filter(|&i| !edges[i][side].iter().any(|&j| compound[j] == Some(owner)))
                    .collect::<Vec<_>>();
                let mut lengths = BTreeMap::<usize, i64>::new();
                let mut candidates = Vec::new();
                let (mut sum_widths, mut sum_lengths) = (0i64, 0i64);
                for i in sides {
                    let run = if side == 0 {
                        sections[i].runs[0]
                    } else {
                        *sections[i].runs.last().unwrap()
                    };
                    edges[i][side].retain(|&j| {
                        !processed[j] && sections[j].runs.len() as i64 + mean <= max_thickness
                    });
                    for &j in &edges[i][side] {
                        let s = &sections[j];
                        let len = s.length();
                        sum_lengths += len;
                        sum_widths += s.runs.len() as i64 * len;
                        if sum_lengths > i32::MAX as i64 || sum_widths > i32::MAX as i64 {
                            return Err("unsupported_vertical_java_int_overflow".into());
                        }
                        let run2 = if side == 0 {
                            *s.runs.last().unwrap()
                        } else {
                            s.runs[0]
                        };
                        let value = lengths.entry(j).or_default();
                        *value += common(run, run2);
                        if *value > i32::MAX as i64 {
                            return Err("unsupported_vertical_java_int_overflow".into());
                        }
                        candidates.push(j);
                    }
                }
                // TreeSet byCoordinate keeps the first object at equal start coordinate.
                candidates.sort_by_key(|&i| sections[i].coord());
                candidates.dedup_by_key(|i| sections[*i].coord());
                let mut common_length = 0;
                if sum_lengths != 0 {
                    let consistent =
                        (sum_widths as f64 / sum_lengths as f64).round_ties_even() as i64;
                    candidates.retain(|&i| sections[i].runs.len() as i64 <= consistent);
                    for i in &candidates {
                        common_length += lengths[i];
                        if common_length > i32::MAX as i64 {
                            return Err("unsupported_vertical_java_int_overflow".into());
                        }
                    }
                }
                if common_length as f64 / box_of(&members, &sections)[major] as f64
                    >= min_side_ratio
                {
                    for i in candidates {
                        members.push(i);
                        processed[i] = true;
                    }
                    sort_members(&mut members, &sections);
                    grown = true;
                } else {
                    closed[side] = true;
                }
            }
            if !grown || thickness(&members, &sections, major) >= max_thickness {
                break;
            }
        }
        let mut stickers = Vec::new();
        for side in [0, 1] {
            for &i in &members {
                for &j in &edges[i][side] {
                    if compound[j].is_none()
                        && sections[j].runs.len() == 1
                        && edges[j][side].is_empty()
                        && !stickers.contains(&j)
                    {
                        stickers.push(j);
                    }
                }
                let s = &sections[i];
                let (run, x) = if side == 0 {
                    (s.runs[0], s.first_pos - 1)
                } else {
                    (*s.runs.last().unwrap(), s.first_pos + s.runs.len() as i64)
                };
                if let Some(hs) = opposite.get(&x) {
                    for &j in hs {
                        let b = sections[j].bounds();
                        if b[coord] < run[0] + run[1]
                            && run[0] < b[coord] + b[major]
                            && !stickers.contains(&j)
                        {
                            stickers.push(j);
                        }
                    }
                }
            }
        }
        for i in stickers {
            members.push(i);
            if i < n {
                compound[i] = Some(owner);
                processed[i] = true;
            }
        }
        sort_members(&mut members, &sections);
        answer.push(describe(&members, &sections, main_vertical)?);
    }
    Ok(answer)
}
fn describe(m: &[usize], sections: &[SeedSection], main_vertical: bool) -> Result<Value, String> {
    let b = box_of(m, sections);
    let mut sums = [0f64; 6];
    let mut pixels = BTreeMap::<i64, BTreeSet<i64>>::new();
    for &i in m {
        let mut local = [0f64; 6];
        for (x, y) in sections[i].pixels() {
            let (xf, yf) = (x as f64, y as f64);
            local[0] += 1.;
            local[1] += xf;
            local[2] += yf;
            local[3] += xf * xf;
            local[4] += yf * yf;
            local[5] += xf * yf;
            let (pos, coord) = if main_vertical { (x, y) } else { (y, x) };
            pixels.entry(pos).or_default().insert(coord);
        }
        for k in 0..6 {
            sums[k] += local[k];
        }
    }
    let [n, sx, sy, xx, yy, xy] = sums;
    let h = n * xx - sx * sx;
    let v = n * yy - sy * sy;
    let (mut a, mut bb, mut c) = if h.abs() >= v.abs() {
        ((n * xy - sx * sy) / h, -1., (sy * xx - sx * xy) / h)
    } else {
        (-1., (n * xy - sx * sy) / v, (sx * yy - sy * xy) / v)
    };
    let norm = a.hypot(bb);
    a /= norm;
    bb /= norm;
    c /= norm;
    if [a, bb, c].iter().any(|v| !v.is_finite()) {
        return Err("unsupported_vertical_undefined_line".into());
    }
    let distance = ((a * a * xx
        + bb * bb * yy
        + c * c * n
        + 2. * a * bb * xy
        + 2. * a * c * sx
        + 2. * bb * c * sy)
        / n)
        .max(0.)
        .sqrt();
    let endpoints = |vertical: bool| {
        if vertical {
            let y0 = b[1] as f64;
            let y1 = (b[1] + b[3] - 1) as f64;
            ([(-bb * y0 - c) / a, y0], [(-bb * y1 - c) / a, y1])
        } else {
            let x0 = b[0] as f64;
            let x1 = (b[0] + b[2] - 1) as f64;
            ([x0, (-a * x0 - c) / bb], [x1, (-a * x1 - c) / bb])
        }
    };
    let (start, stop) = endpoints(b[3] > b[2]);
    let (line_start, line_stop) = endpoints(v.abs() > h.abs());
    if start
        .iter()
        .chain(&stop)
        .chain(&line_start)
        .chain(&line_stop)
        .any(|v| !v.is_finite())
    {
        return Err("unsupported_vertical_endpoint_geometry".into());
    }
    let mut runs = Vec::new();
    let (pos_origin, coord_origin) = if main_vertical {
        (b[0], b[1])
    } else {
        (b[1], b[0])
    };
    for (x, ys) in pixels {
        let mut first = None;
        let mut last = 0;
        for y in ys {
            if first.is_some() && y != last + 1 {
                runs.push([
                    x - pos_origin,
                    first.unwrap() - coord_origin,
                    last - first.unwrap() + 1,
                ]);
                first = None;
            }
            first.get_or_insert(y);
            last = y;
        }
        if let Some(first) = first {
            runs.push([x - pos_origin, first - coord_origin, last - first + 1]);
        }
    }
    Ok(
        json!({"bounds":b,"members":m.iter().map(|&i|sections[i].snapshot()).collect::<Vec<_>>(),"center":[b[0]as f64+b[2]as f64/2.,b[1]as f64+b[3]as f64/2.],"start":start,"stop":stop,"line":[line_start[0],line_start[1],line_stop[0],line_stop[1]],"mean_distance":distance,"weight":n as usize,"basic_line":{"a":a,"b":bb,"c":c,"point_count":n as usize},"glyph":{"bounds":b,"orientation":if main_vertical { "VERTICAL" } else { "HORIZONTAL" },"runs":runs}}),
    )
}

/// Native GRID section dispatch and strict system-edge filtering, before seed checks.
pub fn analyze(grid: &Value, interline: usize, max_stem: usize) -> Result<Value, String> {
    if !(1..=128).contains(&interline) || !(1..=128).contains(&max_stem) {
        return Err("unsupported_vertical_stick_scale".into());
    }
    let mut total_runs = 0usize;
    let mut total_weight = 0i64;
    let mut parse = |key: &str, vertical: bool| -> Result<BTreeMap<u64, SeedSection>, String> {
        let values = grid[key]
            .as_array()
            .filter(|v| v.len() <= 32768)
            .ok_or("unsupported_stem_lag")?;
        let mut out = BTreeMap::new();
        for value in values {
            let raw_runs = value["runs"]
                .as_array()
                .filter(|r| !r.is_empty() && r.len() <= 8192)
                .ok_or("invalid_stem_runs")?;
            total_runs += raw_runs.len();
            if total_runs > 1_048_576 {
                return Err("unsupported_stem_lag_run_budget".into());
            }
            let s:SeedSection=serde_json::from_value(json!({"id":value["id"],"vertical":vertical,"first_pos":value["first_pos"],"runs":value["runs"]})).map_err(|_|"invalid_stem_section")?;
            if s.runs.is_empty()
                || s.runs.len() > 8192
                || !(0..=4_000_000).contains(&s.first_pos)
                || s.first_pos + s.runs.len() as i64 > 4_000_000
                || s.runs.iter().any(|r| {
                    !(0..=4_000_000).contains(&r[0])
                        || !(1..=4_000_000).contains(&r[1])
                        || r[0] + r[1] > 4_000_000
                })
            {
                return Err("invalid_stem_section".into());
            }
            total_weight += s.weight();
            if total_weight > 8_000_000 {
                return Err("unsupported_stem_lag_pixel_budget".into());
            }
            if out.insert(s.id, s).is_some() {
                return Err("duplicate_stem_section".into());
            }
        }
        Ok(out)
    };
    let v = parse("vlag", true)?;
    let h = parse("hlag", false)?;
    let systems = grid["systems"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("invalid_stem_systems")?;
    let mut seen = BTreeSet::new();
    let mut output = Vec::new();
    let mut total = 0;
    for system in systems {
        let sid = system["id"].as_u64().ok_or("invalid_stem_system_id")?;
        if !seen.insert(sid) {
            return Err("duplicate_stem_system_id".into());
        }
        let coords = system["coordinates"]
            .as_array()
            .filter(|c| c.len() >= 3)
            .ok_or("invalid_stem_system_bounds")?;
        let left = coords[0]
            .as_i64()
            .filter(|v| (0..=4_000_000).contains(v))
            .ok_or("invalid_stem_system_left")?;
        let width = coords[2]
            .as_i64()
            .filter(|v| (1..=4_000_000).contains(v))
            .ok_or("invalid_stem_system_width")?;
        let right = left + width; // Original SystemInfo.getRight is exclusive.
        let mut select = |key: &str,
                          bank: &BTreeMap<u64, SeedSection>,
                          vertical: bool|
         -> Result<Vec<SeedSection>, String> {
            let ids = system[key]
                .as_array()
                .filter(|a| a.len() <= 32768)
                .ok_or("invalid_stem_section_membership")?;
            total += ids.len();
            if total > 131072 {
                return Err("unsupported_stem_membership_budget".into());
            }
            let mut selected = Vec::new();
            let mut seen = BTreeSet::new();
            for id in ids {
                let id = id.as_u64().ok_or("invalid_stem_section_id")?;
                if !seen.insert(id) {
                    return Err("duplicate_stem_section_membership".into());
                }
                let s = bank.get(&id).ok_or("missing_stem_section")?;
                let b = s.bounds();
                let center = b[0] + b[2] / 2;
                if center > left && center < right && (vertical || b[2] == 1) {
                    selected.push(s.clone());
                }
            }
            selected.sort_by_key(|s| (s.first_pos, s.coord()));
            Ok(selected)
        };
        let vertical = select("vertical_section_ids", &v, true)?;
        let horizontal = select("horizontal_section_ids", &h, false)?;
        let candidates = retrieve(
            &vertical,
            &horizontal,
            max_stem as i64,
            (1.5 * interline as f64).round_ties_even() as i64,
        )?;
        output.push(json!({"system_id":sid,"left":left,"right":right,"candidates":candidates}));
    }
    Ok(json!({"stage":"STEM_SEEDS.vertical_sticks","systems":output,"stem_seeds_complete":false}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_multisection_line_fits_preserve_sampler_boundaries() {
        let bank: Value = serde_json::from_str(include_str!(
            "../fixtures/official-grid/stick-factory-controls.json"
        ))
        .unwrap();
        let cases = bank["direct_fit_cases"].as_array().unwrap();
        assert_eq!(cases.len(), 4);
        for case in cases {
            let sections: Vec<SeedSection> =
                serde_json::from_value(case["input_sections"].clone()).unwrap();
            let mut members = (0..sections.len()).collect::<Vec<_>>();
            sort_members(&mut members, &sections);
            let a = describe(&members, &sections, true).unwrap();
            let b = &case["filament"];
            for key in ["bounds", "weight", "members", "glyph"] {
                assert_eq!(a[key], b[key], "{} {key}", case["name"]);
            }
            for key in ["start", "stop", "line"] {
                for (x, y) in a[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(b[key].as_array().unwrap())
                {
                    assert!((x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-10);
                }
            }
            assert!(
                (a["mean_distance"].as_f64().unwrap() - b["mean_distance"].as_f64().unwrap()).abs()
                    < 1e-10
            );
            let line = a["line"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>();
            for sample in case["original_lineutil_samples"].as_array().unwrap() {
                let x = crate::stem_checks::x_at_y(
                    [line[0], line[1]],
                    [line[2], line[3]],
                    sample["y"].as_f64().unwrap(),
                );
                assert!((x - sample["x"].as_f64().unwrap()).abs() < 1e-10);
                assert_eq!(
                    x.ceil() as i64,
                    sample["ceil"].as_i64().unwrap(),
                    "{} ceil",
                    case["name"]
                );
                assert_eq!(
                    x.floor() as i64,
                    sample["floor"].as_i64().unwrap(),
                    "{} floor",
                    case["name"]
                );
            }
        }
    }
}
