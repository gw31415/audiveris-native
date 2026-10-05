// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native ClefBuilder candidate stage, before key-dependent registration/selection.
use crate::{GrayImage, curves::Curve};
use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation, Run};
use serde_json::{Value, json};
use std::collections::BTreeMap;

type Error = Box<dyn std::error::Error>;
const MODEL: &[u8] = include_bytes!("../crates/audiveris-classifier/models/audiveris-5.11.0.json");
const KINDS: [&str; 8] = [
    "TREBLE",
    "BASS",
    "BARITONE",
    "TENOR",
    "ALTO",
    "MEZZO_SOPRANO",
    "SOPRANO",
    "PERCUSSION",
];
/// Missing scale retains the legacy uniform-staff contract. An explicit scale
/// must be bounded; malformed or unknown values never fall back silently.
pub(crate) fn staff_interline(staff: &Value, main: usize) -> Result<usize, String> {
    match staff.get("specific_interline") {
        None => Ok(main),
        Some(value) => value
            .as_u64()
            .filter(|n| (1..=128).contains(n))
            .map(|n| n as usize)
            .ok_or_else(|| "unsupported_clef_specific_interline".into()),
    }
}
pub(crate) fn weight(g: &GlyphPixels) -> usize {
    g.runs.iter().map(|r| r.length).sum()
}
pub(crate) fn bounds(g: &GlyphPixels) -> [i64; 4] {
    [g.left as i64, g.top as i64, g.width as i64, g.height as i64]
}
fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
pub(crate) fn pixels(g: &GlyphPixels) -> impl Iterator<Item = (i64, i64)> + '_ {
    g.runs.iter().flat_map(move |r| {
        (r.start..r.start + r.length)
            .map(move |y| (g.left as i64 + r.position as i64, g.top as i64 + y as i64))
    })
}
pub(crate) fn glyph(g: &GlyphPixels) -> Value {
    json!({"bounds":bounds(g),"orientation":"VERTICAL","runs":g.runs.iter().map(|r|[r.position,r.start,r.length]).collect::<Vec<_>>()})
}
fn line(staff: &Value, index: usize) -> Curve {
    Curve::interpolate(
        staff["lines"][index]["points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
            .collect(),
    )
    .unwrap()
}
fn parts(
    image: &GrayImage,
    rect: [i64; 4],
    inner: [i64; 4],
    first: bool,
    il: usize,
) -> Result<Vec<GlyphPixels>, Error> {
    let [x, y, w, h] = rect;
    if x < 0
        || y < 0
        || w <= 0
        || h <= 0
        || x + w > image.width as i64
        || y + h > image.height as i64
    {
        return Err("unsupported_clef_roi_outside_image".into());
    }
    if w * h > 262144 {
        return Err("unsupported_clef_roi_budget".into());
    }
    let mut data = vec![255; (w * h) as usize];
    for j in 0..h as usize {
        data[j * w as usize..(j + 1) * w as usize].copy_from_slice(
            &image.pixels[(y as usize + j) * image.width + x as usize
                ..(y as usize + j) * image.width + (x + w) as usize],
        );
    }
    let mut out = audiveris_staff_removal::extract_glyphs(
        BinaryPage::new(w as usize, h as usize, &data)?,
        Orientation::Vertical,
        (x as i32, y as i32),
        Limits::default(),
    )?;
    let minimum = (il.pow(2) as f64 * 0.01).round_ties_even() as usize;
    out.retain(|g| weight(g) >= minimum && (!first || intersects(bounds(g), inner)));
    if out.len() > 8 {
        out.sort_by_key(|g| std::cmp::Reverse(weight(g)));
        out.truncate(8);
    }
    Ok(out)
}
fn links(parts: &[GlyphPixels], il: usize) -> Result<Vec<u16>, Error> {
    links_at_distance(parts, il as f64 * 0.75)
}
pub(crate) fn links_at_distance(parts: &[GlyphPixels], distance: f64) -> Result<Vec<u16>, Error> {
    if !distance.is_finite() || !(0.0..=192.0).contains(&distance) || parts.len() > 8 {
        return Err("unsupported_glyph_link_budget".into());
    }
    use audiveris_template_match::{Storage, Target, compute};
    let mut adj = vec![0; parts.len()];
    let gap = distance.ceil() as i64;
    for (i, g) in parts.iter().enumerate() {
        let b = bounds(g);
        let fat = [b[0] - gap, b[1] - gap, b[2] + 2 * gap, b[3] + 2 * gap];
        if fat[2] * fat[3] > 262144 {
            return Err("unsupported_clef_distance_budget".into());
        }
        let mut data = vec![255; (fat[2] * fat[3]) as usize];
        for (x, y) in pixels(g) {
            data[((y - fat[1]) * fat[2] + x - fat[0]) as usize] = 0;
        }
        let table = compute(
            fat[2] as usize,
            fat[3] as usize,
            &data,
            Target::Foreground,
            Storage::JavaShort,
            audiveris_template_match::Limits::default(),
        )?;
        for (j, other) in parts.iter().enumerate().skip(i + 1) {
            if !intersects(fat, bounds(other)) {
                continue;
            }
            let d = pixels(other)
                .filter(|&(x, y)| {
                    x >= fat[0] && x < fat[0] + fat[2] && y >= fat[1] && y < fat[1] + fat[3]
                })
                .filter_map(|(x, y)| table.get((x - fat[0]) as usize, (y - fat[1]) as usize))
                .min()
                .unwrap_or(i32::MAX);
            if d as f64 / 3.0 <= distance {
                adj[i] |= 1 << j;
                adj[j] |= 1 << i;
            }
        }
    }
    Ok(adj)
}
pub(crate) fn compound(parts: &[GlyphPixels], mask: u16) -> GlyphPixels {
    if mask.count_ones() == 1 {
        return parts[mask.trailing_zeros() as usize].clone();
    }
    let mut pts = std::collections::BTreeSet::new();
    for (i, g) in parts.iter().enumerate() {
        if mask & (1 << i) != 0 {
            pts.extend(pixels(g));
        }
    }
    let left = pts.iter().map(|p| p.0).min().unwrap();
    let right = pts.iter().map(|p| p.0).max().unwrap();
    let top = pts.iter().map(|p| p.1).min().unwrap();
    let bottom = pts.iter().map(|p| p.1).max().unwrap();
    let mut runs: Vec<Run> = vec![];
    for (x, y) in pts {
        let pos = (x - left) as usize;
        let yy = (y - top) as usize;
        if let Some(r) = runs.last_mut()
            && r.position == pos
            && r.start + r.length == yy
        {
            r.length += 1;
        } else {
            runs.push(Run {
                position: pos,
                start: yy,
                length: 1,
            });
        }
    }
    GlyphPixels {
        left: left as i32,
        top: top as i32,
        width: (right - left + 1) as usize,
        height: (bottom - top + 1) as usize,
        orientation: Orientation::Vertical,
        runs,
    }
}
fn kind(shape: &str, g: &GlyphPixels, staff: &Value, il: usize) -> Option<&'static str> {
    match shape {
        "G_CLEF" | "G_CLEF_8VA" | "G_CLEF_8VB" => Some("TREBLE"),
        "PERCUSSION_CLEF" => Some("PERCUSSION"),
        "C_CLEF" | "F_CLEF" => {
            // GeoUtil.center2D uses rectangle's half width/height, not last-pixel center.
            let x = g.left as f64 + g.width as f64 / 2.;
            let y = g.top as f64 + g.height as f64 / 2.;
            let top = line(staff, 0).y(x);
            let bottom = line(staff, 4).y(x);
            let pp = if y < top {
                -4. + 2. * (y - top) / il as f64
            } else if y > bottom {
                4. + 2. * (y - bottom) / il as f64
            } else {
                4. * (2. * y - bottom - top) / (bottom - top)
            };
            // Fixed official Bravura pitch offset (font200); recorded by original-JAR observer.
            let offset = if shape == "F_CLEF" { -1.491875 } else { 0. };
            let pitch = (2. * ((pp + offset) / 2.).round_ties_even()) as i64;
            if shape == "F_CLEF" {
                Some(if pitch.clamp(-2, 0) == -2 {
                    "BASS"
                } else {
                    "BARITONE"
                })
            } else {
                Some(match pitch.clamp(-2, 4) {
                    -2 => "TENOR",
                    0 => "ALTO",
                    2 => "MEZZO_SOPRANO",
                    _ => "SOPRANO",
                })
            }
        }
        _ => None,
    }
}
fn best(
    parts: &[GlyphPixels],
    staff: &Value,
    il: usize,
    model: &audiveris_classifier::Model,
) -> Result<(Vec<Value>, usize), Error> {
    let mut sorted = parts.to_vec();
    sorted.sort_by_key(|g| g.left);
    let adj = links(&sorted, il)?;
    let mut best: BTreeMap<&str, Value> = BTreeMap::new();
    let mut trials = 0;
    for mask in 1u16..(1u16 << sorted.len()) {
        let mut reached = 1 << mask.trailing_zeros();
        loop {
            let mut next = reached;
            for (i, a) in adj.iter().enumerate() {
                if reached & (1 << i) != 0 {
                    next |= a & mask;
                }
            }
            if next == reached {
                break;
            }
            reached = next;
        }
        if reached != mask {
            continue;
        }
        let g = compound(&sorted, mask);
        if weight(&g) < il * il || g.height as f64 > 9. * il as f64 {
            continue;
        }
        trials += 1;
        let runs: Vec<_> = g
            .runs
            .iter()
            .map(|r| audiveris_glyph_features::Run {
                position: r.position,
                start: r.start,
                length: r.length,
            })
            .collect();
        let features = audiveris_glyph_features::from_runs(
            audiveris_glyph_features::Geometry {
                width: g.width,
                height: g.height,
                left: g.left,
                top: g.top,
                interline: il as u32,
                orientation: audiveris_glyph_features::Orientation::Vertical,
            },
            &runs,
        )?;
        let scores = model.evaluate(&features)?.scores;
        let mut order: Vec<_> = (0..scores.len()).collect();
        order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
        for i in order
            .into_iter()
            .take(3)
            .take_while(|&i| scores[i] >= 0.03 / 0.8)
        {
            let shape = model.output_labels()[i].as_str();
            if let Some(k) = kind(shape, &g, staff, il) {
                let grade = 0.8 * scores[i];
                let new = json!({"kind":k,"shape":shape,"grade":grade,"glyph":glyph(&g)});
                if let Some(old) = best.get(k) {
                    let before = old["grade"].as_f64().unwrap();
                    if grade == before && old["glyph"] != new["glyph"] {
                        return Err("unsupported_clef_equal_grade_tie_order".into());
                    }
                    if before >= grade {
                        continue;
                    }
                }
                best.insert(k, new);
            }
        }
    }
    let mut all: Vec<_> = KINDS.iter().filter_map(|k| best.remove(k)).collect();
    all.sort_by(|a, b| {
        b["grade"]
            .as_f64()
            .unwrap()
            .total_cmp(&a["grade"].as_f64().unwrap())
    });
    'outer: for i in 0..all.len() {
        for j in i + 1..all.len() {
            let g = all[j]["grade"].as_f64().unwrap();
            if all[i]["grade"].as_f64().unwrap() > 5. * g / (1. + 4. * g) {
                all.truncate(j);
                break 'outer;
            }
        }
    }
    all.sort_by_key(|v| {
        KINDS
            .iter()
            .position(|k| *k == v["kind"].as_str().unwrap())
            .unwrap()
    });
    Ok((all, trials))
}
/// Requires native NO_STAFF raster and native GRID/header-start snapshots.
/// Standard five-line staves with original neighbor chains and pinned defaults.
pub fn analyze(
    residual: &GrayImage,
    grid: &Value,
    headers: &Value,
    interline: usize,
) -> Result<Value, Error> {
    if interline == 0 || interline > 128 {
        return Err("unsupported_clef_interline".into());
    }
    let staffs = grid["staffs"].as_array().ok_or("missing_clef_staffs")?;
    for s in staffs {
        if s["lines"].as_array().is_none_or(|a| a.len() != 5) {
            return Err("unsupported_clef_staff_configuration".into());
        }
    }
    let model = audiveris_classifier::Model::from_json(MODEL)?;
    let rectangles = browse_rectangles(grid, headers, residual.height, interline)?;
    let mut out = Vec::new();
    for (staff, roi) in staffs.iter().zip(&rectangles) {
        let outer: [i64; 4] = serde_json::from_value(roi["outer"].clone())?;
        let inner: [i64; 4] = serde_json::from_value(roi["inner"].clone())?;
        let specific = staff_interline(staff, interline)?;
        let first = parts(residual, outer, inner, true, specific)?;
        let second = parts(residual, inner, inner, false, specific)?;
        let (fb, ft) = best(&first, staff, specific, &model)?;
        let (sb, st) = best(&second, staff, specific, &model)?;
        out.push(json!({"staff_id":staff["id"],"outer":outer,"inner":inner,"first_parts":first.iter().map(glyph).collect::<Vec<_>>(),"second_parts":second.iter().map(glyph).collect::<Vec<_>>(),"first_best":fb,"second_best":sb,"first_trials":ft,"second_trials":st,"selected_pass":if fb.is_empty(){2}else{1}}));
    }
    Ok(
        json!({"stage":"HEADERS.clef_candidates","staffs":out,"clef_selection_complete":false,"headers_complete":false,"note_recognition_complete":false,"pending":["register clef candidates and exclusions","key-dependent clef contextualization and final selection","key and time signatures"]}),
    )
}
/// Original ClefBuilder.getOuterRect/getInnerRect, with full StaffManager
/// vertical neighbor chains and strict midpoint-in-abscissa tests.
pub fn browse_rectangles(
    grid: &Value,
    headers: &Value,
    height: usize,
    interline: usize,
) -> Result<Vec<Value>, String> {
    if interline == 0 || interline > 128 || height == 0 || height > 4_000_000 {
        return Err("unsupported_clef_interline".into());
    }
    let staffs = grid["staffs"].as_array().ok_or("missing_clef_staffs")?;
    let neighbors = crate::system_area_geometry::staff_neighbors(staffs)?;
    if headers["staff_headers"].as_array().is_none() {
        return Err("missing_clef_headers".into());
    }
    let px = |n: f64| (interline as f64 * n).round_ties_even() as i64;
    let mut out = Vec::new();
    for (i, staff) in staffs.iter().enumerate() {
        let header = headers["staff_headers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["staff_id"] == staff["id"])
            .ok_or("missing_clef_header")?;
        let start = header["start"]
            .as_i64()
            .filter(|n| n.unsigned_abs() <= 4_000_000)
            .ok_or("invalid_clef_header_start")?;
        let end = start + px(4.5);
        let mid = (start + end) / 2;
        let top = line(staff, 0).y(mid as f64).round_ties_even() as i64;
        let bottom = line(staff, 4).y(mid as f64).round_ties_even() as i64;
        let specific = staff_interline(staff, interline)? as f64;
        // Original keeps the horizontal/core margins at the sheet main scale.
        let mut y0 = 0.max(top - (specific * 3.).round_ties_even() as i64);
        let mut y1 = (height as i64 - 1).min(bottom + (specific * 3.25).round_ties_even() as i64);
        for (above, key) in [(true, "north"), (false, "south")] {
            for id in neighbors[i][key].as_array().unwrap() {
                let s = staffs
                    .iter()
                    .find(|s| s["id"] == *id)
                    .ok_or("missing_clef_neighbor")?;
                if s["left"].as_f64().unwrap().round_ties_even() < mid as f64
                    && s["right"].as_f64().unwrap().round_ties_even() > mid as f64
                {
                    let y = line(s, if above { 4 } else { 0 })
                        .y(mid as f64)
                        .round_ties_even() as i64;
                    if above {
                        y0 = y0.max(((y + top + 1) as f64 * 0.5).ceil() as i64);
                    } else {
                        y1 = y1.min(((bottom + y - 1) as f64 * 0.5).floor() as i64);
                    }
                }
            }
        }
        let outer = [
            start + px(0.15),
            y0,
            end - start + 1 - 2 * px(0.15),
            y1 - y0 + 1,
        ];
        let inner = [
            outer[0] + px(0.2),
            outer[1] + px(0.5),
            outer[2] - px(0.2),
            outer[3] - 2 * px(0.5),
        ];
        out.push(json!({"staff_id":staff["id"],"outer":outer,"inner":inner}));
    }
    Ok(out)
}
