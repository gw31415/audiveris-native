// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// HeaderTimeBuilder adapters at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Provisional candidates only: font bounds and time-column selection are separate.
use crate::{
    GrayImage, clef_candidates as glyphs,
    curves::Curve,
    key_preparation::StaffInput,
    time_preparation::{Preparation, prepare_staff},
};
use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation};
use serde_json::{Value, json};
const MODEL: &[u8] = include_bytes!("../crates/audiveris-classifier/models/audiveris-5.11.0.json");
const WHOLE: [&str; 11] = [
    "COMMON_TIME",
    "CUT_TIME",
    "TIME_FOUR_FOUR",
    "TIME_TWO_TWO",
    "TIME_TWO_FOUR",
    "TIME_THREE_FOUR",
    "TIME_FIVE_FOUR",
    "TIME_SIX_FOUR",
    "TIME_THREE_EIGHT",
    "TIME_SIX_EIGHT",
    "TIME_TWELVE_EIGHT",
];
const HALF: [&str; 10] = [
    "TIME_TWO",
    "TIME_THREE",
    "TIME_FOUR",
    "TIME_FIVE",
    "TIME_SIX",
    "TIME_SEVEN",
    "TIME_EIGHT",
    "TIME_NINE",
    "TIME_TWELVE",
    "TIME_SIXTEEN",
];

fn parts(image: &GrayImage, rect: [i64; 4], minimum: i64) -> Result<Vec<GlyphPixels>, String> {
    let [x, y, w, h] = rect;
    if w <= 0 || h <= 0 || w * h > 1_048_576 {
        return Err("unsupported_time_crop".into());
    }
    let mut pixels = vec![255; (w * h) as usize];
    for j in 0..h {
        for i in 0..w {
            let (sx, sy) = (x + i, y + j);
            if sx >= 0 && sy >= 0 && sx < image.width as i64 && sy < image.height as i64 {
                pixels[(j * w + i) as usize] =
                    image.pixels[sy as usize * image.width + sx as usize];
            }
        }
    }
    let page = BinaryPage::new(w as usize, h as usize, &pixels).map_err(|e| e.to_string())?;
    let mut parts = audiveris_staff_removal::extract_glyphs(
        page,
        Orientation::Vertical,
        (x as i32, y as i32),
        Limits::default(),
    )
    .map_err(|e| e.to_string())?;
    parts.retain(|g| glyphs::weight(g) as i64 >= minimum);
    // Upstream leaves this exponential search unbounded. Never silently discard parts.
    if parts.len() > 8 {
        return Err("unsupported_time_part_budget".into());
    }
    parts.sort_by_key(|g| g.left);
    Ok(parts)
}
fn masks(parts: &[GlyphPixels], adj: &[u16], max_width: i64, min_weight: i64) -> Vec<u16> {
    fn walk(
        parts: &[GlyphPixels],
        adj: &[u16],
        current: Vec<usize>,
        seen: u16,
        limits: (i64, i64),
        out: &mut Vec<u16>,
    ) {
        let left = current.iter().map(|&i| parts[i].left as i64).min().unwrap();
        let right = current
            .iter()
            .map(|&i| parts[i].left as i64 + parts[i].width as i64)
            .max()
            .unwrap();
        if right - left > limits.0 {
            return;
        }
        let weight = current
            .iter()
            .map(|&i| glyphs::weight(&parts[i]))
            .sum::<usize>();
        if weight as i64 >= limits.1 {
            out.push(current.iter().fold(0, |m, i| m | (1 << i)));
        }
        let mut neighbors = Vec::new();
        for &i in &current {
            for j in 0..parts.len() {
                if adj[i] & (1 << j) != 0 && seen & (1 << j) == 0 && !neighbors.contains(&j) {
                    neighbors.push(j);
                }
            }
        }
        let mut considered = seen;
        for j in neighbors {
            considered |= 1 << j;
            let mut next = current.clone();
            next.push(j);
            walk(parts, adj, next, considered, limits, out);
        }
    }
    let mut seeds = (0..parts.len()).collect::<Vec<_>>();
    seeds.sort_by_key(|&i| std::cmp::Reverse(glyphs::weight(&parts[i])));
    let mut seen = 0;
    let mut out = Vec::new();
    for seed in seeds {
        seen |= 1 << seed;
        walk(
            parts,
            adj,
            vec![seed],
            seen,
            (max_width, min_weight),
            &mut out,
        );
    }
    out
}
fn centroid_pitch(g: &GlyphPixels, staff: &StaffInput) -> Result<f64, String> {
    let mut sx = 0i64;
    let mut sy = 0i64;
    let mut n = 0i64;
    for (x, y) in glyphs::pixels(g) {
        sx += x;
        sy += y;
        n += 1;
    }
    let x = (sx as f64 / n as f64).round_ties_even();
    let y = (sy as f64 / n as f64).round_ties_even();
    let top = Curve::interpolate(staff.lines[0].clone())
        .ok_or("invalid_time_curve")?
        .y(x);
    let bottom = Curve::interpolate(staff.lines[4].clone())
        .ok_or("invalid_time_curve")?
        .y(x);
    if top >= bottom {
        return Err("invalid_time_staff_height".into());
    }
    Ok(if y < top {
        -4. + 2. * (y - top) / staff.specific_interline as f64
    } else if y > bottom {
        4. + 2. * (y - bottom) / staff.specific_interline as f64
    } else {
        4. * (2. * y - bottom - top) / (bottom - top)
    })
}
fn evaluate(
    g: &GlyphPixels,
    il: usize,
    model: &audiveris_classifier::Model,
) -> Result<Vec<(usize, f64)>, String> {
    let runs = g
        .runs
        .iter()
        .map(|r| audiveris_glyph_features::Run {
            position: r.position,
            start: r.start,
            length: r.length,
        })
        .collect::<Vec<_>>();
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
    )
    .map_err(|e| e.to_string())?;
    let scores = model.evaluate(&features).map_err(|e| e.to_string())?.scores;
    let mut order = (0..scores.len()).collect::<Vec<_>>();
    order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
    Ok(order
        .into_iter()
        .take(3)
        .take_while(|&i| scores[i] >= 0.1 / 0.8)
        .map(|i| (i, scores[i]))
        .collect())
}
/// Recomputes preparation from native pixels and the post-key staff header boundary.
pub fn find_candidates(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
) -> Result<Value, String> {
    let preparation = prepare_staff(image, sheet_interline, staff)?;
    candidates(image, staff, &preparation)
}
fn candidates(image: &GrayImage, staff: &StaffInput, prep: &Preparation) -> Result<Value, String> {
    let mut adapters = Vec::new();
    let metrics = crate::time_registration::builtin_metrics()?;
    if let Some(rects) = prep.candidate_rectangles {
        let model = audiveris_classifier::Model::from_json(MODEL).map_err(|e| e.to_string())?;
        for (kind, rect) in ["WHOLE", "NUM", "DEN"].iter().zip(rects) {
            let ps = parts(image, rect, prep.parameters.min_part_weight)?;
            let adj = glyphs::links_at_distance(&ps, prep.parameters.max_part_gap)
                .map_err(|e| e.to_string())?;
            let minimum = if *kind == "WHOLE" {
                prep.parameters.min_whole_time_weight
            } else {
                prep.parameters.min_half_time_weight
            };
            let order = masks(&ps, &adj, prep.parameters.max_time_width, minimum);
            let trials = order.len();
            let mut best = std::collections::BTreeMap::<usize, Value>::new();
            for mask in order {
                let g = glyphs::compound(&ps, mask);
                for (index, score) in evaluate(&g, staff.specific_interline, &model)? {
                    let shape = model.output_labels()[index].as_str();
                    let side = if *kind == "WHOLE" {
                        if !WHOLE.contains(&shape) {
                            break;
                        }
                        None
                    } else {
                        if !HALF.contains(&shape) {
                            continue;
                        }
                        let pitch = centroid_pitch(&g, staff)?;
                        if !(1.0..=3.0).contains(&pitch.abs()) {
                            continue;
                        }
                        Some(if pitch < 0.0 { "TOP" } else { "BOTTOM" })
                    };
                    let grade = 0.8 * score;
                    if best
                        .get(&index)
                        .is_none_or(|v| v["grade"].as_f64().unwrap() < grade)
                    {
                        let bounds = crate::time_registration::symbol_bounds(
                            shape,
                            staff.specific_interline,
                            glyphs::bounds(&g),
                            &metrics,
                        )?;
                        best.insert(index,json!({"shape":shape,"grade":grade,"side":side,"bounds":bounds,"glyph":glyphs::glyph(&g)}));
                    }
                }
            }
            adapters.push(json!({"kind":kind,"rectangle":rect,"parts":ps.iter().map(glyphs::glyph).collect::<Vec<_>>(),"trials":trials,"best":best.into_values().collect::<Vec<_>>() }));
        }
    }
    Ok(
        json!({"staff_id":staff.staff_id,"preparation":prep,"adapters":adapters,"time_recognition_complete":false,"headers_complete":false}),
    )
}

/// Per-staff diagnostics after native key-column propagation. Unlike TimeColumn,
/// this diagnostic visits every staff; system abort/selection belongs to time_column.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    key_column: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    let inputs =
        crate::time_preparation::after_key_staffs(grid, registration, key_column, sheet_interline)?;
    let staffs = inputs
        .iter()
        .map(|s| find_candidates(image, sheet_interline, s))
        .collect::<Result<Vec<_>, String>>()?;
    Ok(
        json!({"stage":"HEADERS.time_candidates","staffs":staffs,"system_selection_complete":false,"time_recognition_complete":false,"headers_complete":false}),
    )
}
