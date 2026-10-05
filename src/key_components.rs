// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// KeyExtractor connected-component candidate pass, before hard-slice fallback.
use crate::{
    GrayImage, clef_candidates as glyphs,
    curves::Curve,
    key_preparation::StaffInput,
    key_slices::{Shape, ShapePreparation},
};
use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation};
use serde_json::{Value, json};
const MODEL: &[u8] = include_bytes!("../crates/audiveris-classifier/models/audiveris-5.11.0.json");
#[derive(Clone)]
struct Candidate {
    glyph: GlyphPixels,
    parts: u16,
    score: f64,
}

fn crop_pixels(
    image: &GrayImage,
    rect: [i64; 4],
    erase: &[GlyphPixels],
) -> Result<Vec<u8>, String> {
    let [x, y, w, h] = rect;
    if w <= 0
        || h <= 0
        || w > 8192
        || h > 4096
        || w * h > 1_048_576
        || !(-4_000_000..=4_000_000).contains(&x)
        || !(-4_000_000..=4_000_000).contains(&y)
    {
        return Err("unsupported_key_component_roi".into());
    }
    let mut pixels = vec![255; (w * h) as usize];
    for j in 0..h {
        let source_y = y + j;
        if source_y < 0 || source_y >= image.height as i64 {
            continue;
        }
        for i in 0..w {
            let source_x = x + i;
            if source_x >= 0 && source_x < image.width as i64 {
                pixels[(j * w + i) as usize] =
                    image.pixels[source_y as usize * image.width + source_x as usize];
            }
        }
    }
    for glyph in erase {
        for (gx, gy) in glyphs::pixels(glyph) {
            if gx >= x && gx < x + w && gy >= y && gy < y + h {
                pixels[((gy - y) * w + gx - x) as usize] = 255;
            }
        }
    }
    Ok(pixels)
}
fn crop_parts(
    image: &GrayImage,
    rect: [i64; 4],
    interline: usize,
    erase: &[GlyphPixels],
) -> Result<Vec<GlyphPixels>, String> {
    let [x, y, w, h] = rect;
    let pixels = crop_pixels(image, rect, erase)?;
    let page = BinaryPage::new(w as usize, h as usize, &pixels).map_err(|e| e.to_string())?;
    let mut parts = audiveris_staff_removal::extract_glyphs(
        page,
        Orientation::Vertical,
        (x as i32, y as i32),
        Limits::default(),
    )
    .map_err(|e| e.to_string())?;
    let minimum = (interline.pow(2) as f64 * 0.01).round_ties_even() as usize;
    parts.retain(|g| glyphs::weight(g) >= minimum && g.left as i64 != x + w - 1);
    if parts.len() > 8 {
        parts.sort_by_key(|g| std::cmp::Reverse(glyphs::weight(g)));
        parts.truncate(8);
    }
    parts.sort_by_key(|g| g.left); // Glyphs.buildLinks vertex order.
    Ok(parts)
}
fn centroid(g: &GlyphPixels) -> [i64; 2] {
    let mut weight = 0i64;
    let mut xs = 0i64;
    let mut ys = 0i64;
    for r in &g.runs {
        let n = r.length as i64;
        weight += n;
        xs += (g.left as i64 + r.position as i64) * n;
        ys += (g.top as i64 + r.start as i64) * n + n * (n - 1) / 2;
    }
    [
        (xs as f64 / weight as f64).round_ties_even() as i64,
        (ys as f64 / weight as f64).round_ties_even() as i64,
    ]
}
fn pitch(staff: &StaffInput, x: f64, y: f64) -> Result<f64, String> {
    if staff.lines.len() != 5 {
        return Err("unsupported_key_component_staff".into());
    }
    let top = Curve::interpolate(staff.lines[0].clone())
        .ok_or("invalid_key_component_curve")?
        .y(x);
    let bottom = Curve::interpolate(staff.lines[4].clone())
        .ok_or("invalid_key_component_curve")?
        .y(x);
    if !top.is_finite() || !bottom.is_finite() || top >= bottom {
        return Err("invalid_key_component_staff_height".into());
    }
    let il = staff.specific_interline as f64;
    Ok(if y < top {
        -4. + 2. * (y - top) / il
    } else if y > bottom {
        4. + 2. * (y - bottom) / il
    } else {
        4. * (2. * y - bottom - top) / (bottom - top)
    })
}
fn slice_at(slices: &[[i64; 4]], x: i64) -> Option<usize> {
    slices.iter().position(|b| b[0] <= x && x < b[0] + b[2])
}
fn order(parts: &[GlyphPixels], adj: &[u16], interline: usize) -> Vec<u16> {
    // ConnectivityInspector BFS component order, followed by stable weight-sorted seeds.
    let mut visited = 0u16;
    let mut answer = Vec::new();
    for start in 0..parts.len() {
        if visited & (1 << start) != 0 {
            continue;
        }
        let mut component = vec![start];
        visited |= 1 << start;
        let mut pos = 0;
        while pos < component.len() {
            let i = component[pos];
            pos += 1;
            for j in 0..parts.len() {
                if adj[i] & (1 << j) != 0 && visited & (1 << j) == 0 {
                    visited |= 1 << j;
                    component.push(j);
                }
            }
        }
        // getSubGraph inserts incident edges in component iteration order.
        let mut edges = Vec::new();
        for &i in &component {
            for j in 0..parts.len() {
                if adj[i] & (1 << j) != 0 {
                    let e = (i.min(j), i.max(j));
                    if !edges.contains(&e) {
                        edges.push(e);
                    }
                }
            }
        }
        let mut neighbors = vec![Vec::new(); parts.len()];
        for (a, b) in edges {
            neighbors[a].push(b);
            neighbors[b].push(a);
        }
        let mut seeds = component;
        seeds.sort_by_key(|i| std::cmp::Reverse(glyphs::weight(&parts[*i])));
        let mut seen = 0;
        for seed in seeds {
            seen |= 1 << seed;
            walk(parts, &neighbors, vec![seed], seen, interline, &mut answer);
        }
    }
    answer
}
fn walk(
    parts: &[GlyphPixels],
    neighbors: &[Vec<usize>],
    current: Vec<usize>,
    seen: u16,
    il: usize,
    answer: &mut Vec<u16>,
) {
    let mask = current.iter().fold(0, |m, i| m | (1 << i));
    // Match original early rejection before materializing a compound pixel set.
    let weight = current
        .iter()
        .map(|i| glyphs::weight(&parts[*i]))
        .sum::<usize>();
    if weight > (il.pow(2) as f64 * 3.4).round_ties_even() as usize {
        return;
    }
    let left = current.iter().map(|i| parts[*i].left as i64).min().unwrap();
    let top = current.iter().map(|i| parts[*i].top as i64).min().unwrap();
    let right = current
        .iter()
        .map(|i| parts[*i].left as i64 + parts[*i].width as i64)
        .max()
        .unwrap();
    let bottom = current
        .iter()
        .map(|i| parts[*i].top as i64 + parts[*i].height as i64)
        .max()
        .unwrap();
    if (right - left) as f64 > 2. * il as f64 || (bottom - top) as f64 > 3.8 * il as f64 {
        return;
    }
    answer.push(mask);
    let mut outliers = Vec::new();
    for &i in &current {
        for &j in &neighbors[i] {
            if seen & (1 << j) == 0 && !outliers.contains(&j) {
                outliers.push(j);
            }
        }
    }
    let mut considered = seen;
    for other in outliers {
        considered |= 1 << other;
        let mut larger = current.clone();
        larger.push(other);
        walk(parts, neighbors, larger, considered, il, answer);
    }
}
fn evaluate(
    g: &GlyphPixels,
    interline: usize,
    model: &audiveris_classifier::Model,
    shape: &str,
    min_grade: f64,
) -> Result<Option<f64>, String> {
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
            interline: interline as u32,
            orientation: audiveris_glyph_features::Orientation::Vertical,
        },
        &runs,
    )
    .map_err(|e| e.to_string())?;
    let scores = model.evaluate(&features).map_err(|e| e.to_string())?.scores;
    let mut indices = (0..scores.len()).collect::<Vec<_>>();
    indices.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
    for i in indices
        .into_iter()
        .take(3)
        .take_while(|i| scores[*i] >= min_grade / 0.8)
    {
        if model.output_labels()[i] == shape {
            return Ok(Some(scores[i]));
        }
    }
    Ok(None)
}

/// Native component-pass candidates and provisional slice alterations only.
/// Hard slice fallback, clef compatibility, key selection and SIG feedback are not included.
pub fn retrieve_components(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    input: &ShapePreparation,
    flat_area_pitch_offset: f64,
) -> Result<Value, String> {
    if !(1..=128).contains(&sheet_interline)
        || !(1..=128).contains(&staff.specific_interline)
        || !flat_area_pitch_offset.is_finite()
        || flat_area_pitch_offset.abs() > 10.
    {
        return Err("unsupported_key_component_parameters".into());
    }
    if image
        .width
        .checked_mul(image.height)
        .is_none_or(|n| n == 0 || n > crate::MAX_PIXELS || n != image.pixels.len())
        || image.pixels.iter().any(|p| *p != 0 && *p != 255)
    {
        return Err("invalid_key_component_image".into());
    }
    let shape = match input.shape {
        Shape::Flat => "FLAT",
        Shape::Sharp => "SHARP",
    };
    if input.slices.is_empty() {
        return Ok(
            json!({"shape":shape,"parts":[],"candidates":[],"slices":[],"component_pass_complete":true,"hard_slice_fallback_complete":false,"key_recognition_complete":false}),
        );
    }
    if input.slices.len() > 7 || input.peaks.len() > 128 {
        return Err("unsupported_key_component_slice_budget".into());
    }
    if [
        input.range.start.unwrap_or(input.range.browse_start),
        input.range.stop.unwrap_or(input.range.browse_stop),
    ]
    .iter()
    .any(|v| !(-4_000_000..=4_000_000).contains(v))
        || input
            .slices
            .iter()
            .flatten()
            .any(|v| !(-4_000_000..=4_000_000).contains(v))
        || input.peaks.iter().any(|p| {
            ![p.min, p.main, p.max]
                .iter()
                .all(|v| (-4_000_000..=4_000_000).contains(v))
        })
    {
        return Err("invalid_key_component_geometry".into());
    }
    let x = input.range.start.unwrap_or(input.range.browse_start);
    let right = input.range.stop.unwrap_or(input.range.browse_stop);
    let rect = [x, input.slices[0][1], right - x + 1, input.slices[0][3]];
    for s in &input.slices {
        if s[1] != rect[1] || s[3] != rect[3] || s[2] <= 0 || s[0] < x || s[0] + s[2] - 1 > right {
            return Err("unsupported_key_component_slice_geometry".into());
        }
    }
    let il = staff.specific_interline;
    let parts = crop_parts(image, rect, il, &[])?;
    let adj = glyphs::links_at_distance(&parts, il as f64 * 1.5).map_err(|e| e.to_string())?;
    let model = audiveris_classifier::Model::from_json(MODEL).map_err(|e| e.to_string())?;
    let mut candidates = Vec::<Candidate>::new();
    for mask in order(&parts, &adj, il) {
        let glyph = glyphs::compound(&parts, mask);
        if glyphs::weight(&glyph) < (il.pow(2) as f64 * 0.2).round_ties_even() as usize
            || (glyph.width as f64) < 0.5 * il as f64
            || glyph.height < il
        {
            continue;
        }
        let gc = centroid(&glyph);
        if let Some(slice) = slice_at(&input.slices, gc[0]) {
            let b = input.slices[slice];
            if input.peaks.iter().any(|p| {
                let center = (p.min + p.max) as f64 * 0.5;
                b[0] as f64 <= center
                    && center <= (b[0] + b[2] - 1) as f64
                    && !(glyph.left as f64 <= center
                        && center <= (glyph.left as i64 + glyph.width as i64 - 1) as f64)
            }) {
                continue;
            }
        }
        if let Some(score) = evaluate(&glyph, sheet_interline, &model, shape, 0.1)? {
            candidates.push(Candidate {
                glyph,
                parts: mask,
                score,
            });
        }
    }
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut retained = Vec::<Candidate>::new();
    for c in candidates {
        if let Some(old) = retained.iter().find(|o| o.parts & c.parts != 0) {
            if old.score == c.score && glyphs::glyph(&old.glyph) != glyphs::glyph(&c.glyph) {
                return Err("unsupported_key_equal_grade_overlap_order".into());
            }
            continue;
        }
        retained.push(c);
    }
    let mut assigned = vec![None::<Candidate>; input.slices.len()];
    for c in &retained {
        let i = slice_at(&input.slices, centroid(&c.glyph)[0])
            .ok_or("unsupported_key_candidate_outside_slices")?;
        if assigned[i].as_ref().is_none_or(|old| old.score < c.score) {
            assigned[i] = Some(c.clone());
        } else if assigned[i].as_ref().is_some_and(|old| {
            old.score == c.score && glyphs::glyph(&old.glyph) != glyphs::glyph(&c.glyph)
        }) {
            return Err("unsupported_key_equal_grade_slice_order".into());
        }
    }
    let mut slices = Vec::new();
    for (b, c) in input.slices.iter().zip(assigned) {
        let eval = c.as_ref().map(|c| json!({"shape":shape,"grade":c.score}));
        let snapshot = c.as_ref().map(|c| glyphs::glyph(&c.glyph));
        let alter = if let Some(c) = c {
            let g = &c.glyph;
            let gc = centroid(g);
            let mass = pitch(staff, gc[0] as f64, gc[1] as f64)?;
            let measured = if input.shape == Shape::Flat {
                0.5 * ((mass + 0.65)
                    + (pitch(
                        staff,
                        g.left as f64 + g.width as f64 / 2.,
                        g.top as f64 + g.height as f64 / 2.,
                    )? + flat_area_pitch_offset))
            } else {
                mass
            };
            json!({"shape":shape,"grade":0.8*c.score,"bounds":glyphs::bounds(g),"pitch":measured.round_ties_even(),"measured_pitch":measured,"glyph":glyphs::glyph(g)})
        } else {
            Value::Null
        };
        slices.push(json!({"bounds":b,"eval":eval,"glyph":snapshot,"alter":alter}));
    }
    Ok(
        json!({"shape":shape,"parts":parts.iter().map(glyphs::glyph).collect::<Vec<_>>(),"candidates":retained.iter().map(|c|json!({"glyph":glyphs::glyph(&c.glyph),"part_indices":(0..parts.len()).filter(|i|c.parts&(1<<i)!=0).collect::<Vec<_>>(),"shape":shape,"eval_grade":c.score})).collect::<Vec<_>>(),"slices":slices,"component_pass_complete":true,"hard_slice_fallback_complete":false,"key_recognition_complete":false}),
    )
}

/// Exact original Bravura area-pitch offset, retained in component oracle provenance.
/// No font rasterization or Java is performed at runtime.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    analyze_mode(image, grid, registration, sheet_interline, false)
}
pub fn analyze_with_empty_slice_fallback(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    analyze_mode(image, grid, registration, sheet_interline, true)
}
fn analyze_mode(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
    hard_fallback: bool,
) -> Result<Value, String> {
    let inputs = crate::key_preparation::staff_inputs(grid, registration, sheet_interline)?;
    let mut staffs = Vec::new();
    for staff in inputs {
        let projection = crate::key_preparation::prepare_staff(image, sheet_interline, &staff)?;
        let peaks = crate::key_peaks::retrieve(&projection, staff.specific_interline)?;
        let slicing =
            crate::key_slices::prepare_staff(image, sheet_interline, &staff, &projection, &peaks)?;
        let mut shapes = Vec::new();
        for shape in &slicing.shapes {
            shapes.push(if hard_fallback {
                with_empty_slice_fallback(image, sheet_interline, &staff, shape, 1.0559375)?
            } else {
                retrieve_components(image, sheet_interline, &staff, shape, 1.0559375)?
            });
        }
        staffs.push(json!({"staff_id":staff.staff_id,"shapes":shapes}));
    }
    Ok(
        json!({"stage":if hard_fallback{"HEADERS.key_components_after_empty_slice_fallback"}else{"HEADERS.key_components"},"staffs":staffs,"hard_slice_fallback_complete":hard_fallback,"key_recognition_complete":false,"headers_complete":false,"pending":["hard slice fallback","clef-key compatibility and key selection","cross-staff consistency and key finalization"]}),
    )
}

fn read_glyph(v: &Value) -> Result<GlyphPixels, String> {
    let b = v["bounds"]
        .as_array()
        .filter(|a| a.len() == 4)
        .ok_or("invalid_key_snapshot_bounds")?;
    let mut rect = [0i64; 4];
    for (i, x) in b.iter().enumerate() {
        rect[i] = x
            .as_i64()
            .filter(|v| (-4_000_000..=4_000_000).contains(v))
            .ok_or("invalid_key_snapshot_bounds")?;
    }
    if rect[2] <= 0 || rect[3] <= 0 || v["orientation"] != "VERTICAL" {
        return Err("invalid_key_snapshot_geometry".into());
    }
    let mut runs = Vec::new();
    for r in v["runs"]
        .as_array()
        .filter(|a| a.len() <= 262144)
        .ok_or("invalid_key_snapshot_runs")?
    {
        let p = r[0]
            .as_u64()
            .filter(|p| *p < rect[2] as u64)
            .ok_or("invalid_key_snapshot_run")? as usize;
        let s = r[1]
            .as_u64()
            .filter(|s| *s < rect[3] as u64)
            .ok_or("invalid_key_snapshot_run")? as usize;
        let n = r[2]
            .as_u64()
            .filter(|n| *n > 0 && *n <= rect[3] as u64 - s as u64)
            .ok_or("invalid_key_snapshot_run")? as usize;
        runs.push(audiveris_staff_removal::Run {
            position: p,
            start: s,
            length: n,
        });
    }
    if runs.is_empty() {
        return Err("empty_key_snapshot_glyph".into());
    }
    Ok(GlyphPixels {
        left: rect[0] as i32,
        top: rect[1] as i32,
        width: rect[2] as usize,
        height: rect[3] as usize,
        orientation: Orientation::Vertical,
        runs,
    })
}
fn single_order(parts: &[GlyphPixels], adj: &[u16], il: usize) -> Vec<u16> {
    let mut neighbors = vec![Vec::new(); parts.len()];
    for i in 0..parts.len() {
        for j in i + 1..parts.len() {
            if adj[i] & (1 << j) != 0 {
                neighbors[i].push(j);
                neighbors[j].push(i);
            }
        }
    }
    let mut seeds = (0..parts.len()).collect::<Vec<_>>();
    seeds.sort_by_key(|i| std::cmp::Reverse(glyphs::weight(&parts[*i])));
    let mut seen = 0;
    let mut answer = Vec::new();
    for seed in seeds {
        seen |= 1 << seed;
        walk(parts, &neighbors, vec![seed], seen, il, &mut answer);
    }
    answer
}

/// Original extractEmptySlices stage. Newly accepted neighbors are erased for later attempts.
/// This does not implement the clef-conditioned fillMissingAlters pitch-window pass.
pub fn with_empty_slice_fallback(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    input: &ShapePreparation,
    flat_area_pitch_offset: f64,
) -> Result<Value, String> {
    let mut result =
        retrieve_components(image, sheet_interline, staff, input, flat_area_pitch_offset)?;
    result["component_slices"] = result["slices"].clone();
    let shape = match input.shape {
        Shape::Flat => "FLAT",
        Shape::Sharp => "SHARP",
    };
    let model = audiveris_classifier::Model::from_json(MODEL).map_err(|e| e.to_string())?;
    let il = staff.specific_interline;
    let mut attempts = Vec::new();
    for index in 0..input.slices.len() {
        if !result["slices"][index]["alter"].is_null() {
            continue;
        }
        let rect = input.slices[index];
        let mut erase = Vec::new();
        for neighbor in [
            index.checked_sub(1),
            (index + 1 < input.slices.len()).then_some(index + 1),
        ]
        .into_iter()
        .flatten()
        {
            let a = &result["slices"][neighbor]["alter"];
            if !a.is_null() {
                erase.push(read_glyph(&a["glyph"])?);
            }
        }
        use sha2::{Digest, Sha256};
        let raw_pixels = crop_pixels(image, rect, &[])?;
        let cropped_pixels = crop_pixels(image, rect, &erase)?;
        let erased_pixels = raw_pixels
            .iter()
            .zip(&cropped_pixels)
            .filter(|(a, b)| **a == 0 && **b == 255)
            .count();
        let raw_hash = format!("{:x}", Sha256::digest(&raw_pixels));
        let cropped_hash = format!("{:x}", Sha256::digest(&cropped_pixels));
        let parts = crop_parts(image, rect, il, &erase)?;
        let adj = glyphs::links_at_distance(&parts, il as f64 * 1.5).map_err(|e| e.to_string())?;
        let mut best = None::<Candidate>;
        for mask in single_order(&parts, &adj, il) {
            let glyph = glyphs::compound(&parts, mask);
            if glyphs::weight(&glyph) < (il.pow(2) as f64 * 0.2).round_ties_even() as usize
                || (glyph.width as f64) < 0.5 * il as f64
                || glyph.height < il
            {
                continue;
            }
            if input.peaks.iter().any(|p| {
                let c = (p.min + p.max) as f64 * 0.5;
                rect[0] as f64 <= c
                    && c <= (rect[0] + rect[2] - 1) as f64
                    && !(glyph.left as f64 <= c
                        && c <= (glyph.left as i64 + glyph.width as i64 - 1) as f64)
            }) {
                continue;
            }
            if let Some(score) = evaluate(&glyph, sheet_interline, &model, shape, 0.01)? {
                if best.as_ref().is_none_or(|b| b.score < score) {
                    best = Some(Candidate {
                        glyph,
                        parts: mask,
                        score,
                    });
                } else if best.as_ref().is_some_and(|b| {
                    b.score == score && glyphs::glyph(&b.glyph) != glyphs::glyph(&glyph)
                }) {
                    return Err("unsupported_key_fallback_equal_grade_order".into());
                }
            }
        }
        let assigned = if let Some(c) = best {
            let g = &c.glyph;
            let gc = centroid(g);
            let mass = pitch(staff, gc[0] as f64, gc[1] as f64)?;
            let measured = if input.shape == Shape::Flat {
                0.5 * ((mass + 0.65)
                    + (pitch(
                        staff,
                        g.left as f64 + g.width as f64 / 2.,
                        g.top as f64 + g.height as f64 / 2.,
                    )? + flat_area_pitch_offset))
            } else {
                mass
            };
            result["slices"][index]["eval"] = json!({"shape":shape,"grade":c.score});
            result["slices"][index]["glyph"] = glyphs::glyph(g);
            if 0.8 * c.score >= 0.01 {
                result["slices"][index]["alter"] = json!({"shape":shape,"grade":0.8*c.score,"bounds":glyphs::bounds(g),"pitch":measured.round_ties_even(),"measured_pitch":measured,"glyph":glyphs::glyph(g)});
                true
            } else {
                false
            }
        } else {
            false
        };
        attempts.push(json!({"slice_index":index,"parts":parts.iter().map(glyphs::glyph).collect::<Vec<_>>(),"neighbor_glyphs_considered":erase.len(),"neighbor_erased_pixels":erased_pixels,"raw_sha256":raw_hash,"cropped_sha256":cropped_hash,"assigned":assigned}));
    }
    result["fallback_attempts"] = json!(attempts);
    result["hard_slice_fallback_complete"] = json!(true);
    result["pitch_window_fallback_complete"] = json!(false);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn glyph(left: i32, top: i32, w: usize, h: usize) -> GlyphPixels {
        GlyphPixels {
            left,
            top,
            width: w,
            height: h,
            orientation: Orientation::Vertical,
            runs: (0..w)
                .map(|position| audiveris_staff_removal::Run {
                    position,
                    start: 0,
                    length: h,
                })
                .collect(),
        }
    }
    #[test]
    fn clip_background_and_erase_exact_neighbor_pixels() {
        let mut image = GrayImage::new(8, 8, vec![255; 64]).unwrap();
        for y in 1..5 {
            for x in 1..4 {
                image.pixels[y * 8 + x] = 0;
            }
        }
        let parts = crop_parts(&image, [-2, -2, 9, 9], 1, &[]).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(glyphs::bounds(&parts[0]), [1, 1, 3, 4]);
        let erased = crop_parts(&image, [0, 0, 7, 7], 1, &[glyph(1, 1, 2, 4)]).unwrap();
        assert_eq!(erased.len(), 1);
        assert_eq!(glyphs::bounds(&erased[0]), [3, 1, 1, 4]);
        assert!(crop_parts(&image, [i64::MIN, 0, 5, 5], 1, &[]).is_err());
    }
    #[test]
    fn tiny_parts_and_last_column_parts_follow_source_filter() {
        let mut image = GrayImage::new(10, 10, vec![255; 100]).unwrap();
        image.pixels[11] = 0;
        for y in 1..5 {
            image.pixels[y * 10 + 9] = 0;
            image.pixels[y * 10 + 5] = 0;
        }
        let parts = crop_parts(&image, [0, 0, 10, 10], 16, &[]).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(glyphs::bounds(&parts[0]), [5, 1, 1, 4]);
    }
    #[test]
    fn connected_decomposition_visits_each_connected_subset_once_and_prunes_weight() {
        let parts = vec![
            glyph(10, 10, 2, 10),
            glyph(13, 10, 2, 10),
            glyph(16, 10, 2, 10),
        ];
        let adj = [2, 5, 2];
        let mut actual = order(&parts, &adj, 16);
        let n = actual.len();
        actual.sort();
        actual.dedup();
        assert_eq!(n, actual.len());
        assert_eq!(actual, vec![1, 2, 3, 4, 6, 7]);
        let heavy = vec![glyph(0, 0, 200, 200)];
        assert!(order(&heavy, &[0], 16).is_empty());
    }
    #[test]
    fn glyph_centroid_rounds_integer_pixels_ties_even() {
        assert_eq!(centroid(&glyph(10, 20, 2, 2)), [10, 20]);
        assert_eq!(centroid(&glyph(11, 21, 2, 2)), [12, 22]);
    }
}

#[cfg(test)]
mod neighbor_oracle_test {
    use super::*;
    #[test]
    fn exact_original_cropped_pixels_erase_240_neighbor_pixels() {
        let control: Value = serde_json::from_slice(include_bytes!(
            "../fixtures/official-grid/key-neighbor-erasure-control.json"
        ))
        .unwrap();
        assert_eq!(control["controlled_input"], true);
        let b = &control["expanded_rect"];
        let (x, y, w, h) = (
            b[0].as_i64().unwrap(),
            b[1].as_i64().unwrap(),
            b[2].as_u64().unwrap() as usize,
            b[3].as_u64().unwrap() as usize,
        );
        let mut g = read_glyph(&control["previous_assigned_glyph"]).unwrap();
        g.left -= x as i32;
        g.top -= y as i32;
        let raw = include_bytes!("../fixtures/key-fallback/neighbor-raw.bin");
        let image = GrayImage::new(w, h, raw.to_vec()).unwrap();
        let actual = crop_pixels(&image, [0, 0, w as i64, h as i64], &[g]).unwrap();
        assert_eq!(
            &actual[..],
            include_bytes!("../fixtures/key-fallback/neighbor-cropped.bin")
        );
        assert_eq!(
            raw.iter()
                .zip(actual)
                .filter(|(a, b)| **a == 0 && *b == 255)
                .count(),
            240
        );
    }
}

fn pitch_ordinate(staff: &StaffInput, x: f64, target: i32) -> Result<f64, String> {
    if staff.lines.len() != 5 {
        return Err("unsupported_key_pitch_window_staff".into());
    }
    let top = Curve::interpolate(staff.lines[0].clone())
        .ok_or("invalid_key_pitch_window_curve")?
        .y(x);
    let bottom = Curve::interpolate(staff.lines[4].clone())
        .ok_or("invalid_key_pitch_window_curve")?
        .y(x);
    if !top.is_finite() || !bottom.is_finite() || bottom <= top {
        return Err("invalid_key_pitch_window_staff".into());
    }
    let p = target as f64;
    Ok(if target.abs() <= 4 {
        0.5 * (top + bottom + (p * (bottom - top) / 4.))
    } else if target < 0 {
        top + 0.5 * (p + 4.) * staff.specific_interline as f64
    } else {
        bottom + 0.5 * (p - 4.) * staff.specific_interline as f64
    })
}

/// KeySlice.setPitchRect on a standard pre-LEDGERS staff, preserving Java casts.
pub fn pitch_window_rect(
    staff: &StaffInput,
    old: [i64; 4],
    shape: Shape,
    clef: crate::key_clef::ClefKind,
    slice_index: usize,
    height: usize,
) -> Result<[i64; 4], String> {
    if height == 0
        || height > 4096
        || slice_index >= 7
        || old[2] <= 0
        || old.iter().any(|v| !(-4_000_000..=4_000_000).contains(v))
        || staff.lines.len() != 5
        || staff.lines.iter().any(|line| {
            line.len() > 4096
                || line
                    .iter()
                    .flatten()
                    .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        })
    {
        return Err("unsupported_key_pitch_window_geometry".into());
    }
    let key_shape = match shape {
        Shape::Flat => crate::key_clef::KeyShape::Flat,
        Shape::Sharp => crate::key_clef::KeyShape::Sharp,
    };
    let table = crate::key_clef::pitches(clef, key_shape)
        .ok_or("unsupported_percussion_key_pitch_window")?;
    let adjusted =
        (table[slice_index] as f64 - if shape == Shape::Flat { 1.0559375 } else { 0. }) as i32;
    let y = (pitch_ordinate(staff, old[0] as f64, adjusted)? - (height / 2) as f64)
        .round_ties_even() as i64;
    Ok([old[0], y, old[2], height as i64])
}
fn make_alter(
    g: &GlyphPixels,
    score: f64,
    shape: Shape,
    staff: &StaffInput,
) -> Result<Value, String> {
    let centroid = centroid(g);
    let mass = pitch(staff, centroid[0] as f64, centroid[1] as f64)?;
    let measured = if shape == Shape::Flat {
        let mass = mass + 0.65;
        let geo = pitch(
            staff,
            g.left as f64 + g.width as f64 / 2.,
            g.top as f64 + g.height as f64 / 2.,
        )? + 1.0559375;
        0.5 * (mass + geo)
    } else {
        mass
    };
    Ok(
        json!({"shape":if shape==Shape::Flat{"FLAT"}else{"SHARP"},"grade":0.8*score,"bounds":glyphs::bounds(g),"pitch":measured.round_ties_even(),"measured_pitch":measured,"glyph":glyphs::glyph(g),"contextual_grade":Value::Null}),
    )
}

/// Slice.extractAlter retry semantics: old eval/glyph survive unless strictly improved;
/// a same-glyph successful return leaves the old interpretation (including grade/cache) intact.
fn retry_slice(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    input: &ShapePreparation,
    index: usize,
    state: &mut Value,
) -> Result<Value, String> {
    extract_alter(image, sheet_interline, staff, input, index, state, true)
}
/// Original extractAlter with the phase-2 0.01 threshold; caller controls neighbor cropping.
/// The current mutable slice population may differ from the original geometric hypothesis.
pub fn extract_alter(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    input: &ShapePreparation,
    index: usize,
    state: &mut Value,
    crop_neighbors: bool,
) -> Result<Value, String> {
    if !(1..=128).contains(&sheet_interline)
        || !(1..=128).contains(&staff.specific_interline)
        || image
            .width
            .checked_mul(image.height)
            .is_none_or(|n| n == 0 || n > crate::MAX_PIXELS || n != image.pixels.len())
        || image.pixels.iter().any(|p| *p != 0 && *p != 255)
        || input.peaks.len() > 128
    {
        return Err("unsupported_key_retry_input".into());
    }
    let slice_count = state["slices"]
        .as_array()
        .filter(|a| a.len() <= 7)
        .ok_or("unsupported_key_retry_slice_count")?
        .len();
    if index >= slice_count {
        return Err("invalid_key_retry_slice_index".into());
    }
    let rect_value = &state["slices"][index]["bounds"];
    let a = rect_value
        .as_array()
        .filter(|a| a.len() == 4)
        .ok_or("invalid_key_retry_bounds")?;
    let mut rect = [0; 4];
    for (i, n) in a.iter().enumerate() {
        rect[i] = n
            .as_i64()
            .filter(|v| (-4_000_000..=4_000_000).contains(v))
            .ok_or("invalid_key_retry_bounds")?;
    }
    let il = staff.specific_interline;
    let shape = match input.shape {
        Shape::Flat => "FLAT",
        Shape::Sharp => "SHARP",
    };
    let old = state["slices"][index].clone();
    let mut best = if old["eval"].is_null() {
        None
    } else {
        if old["eval"]["shape"] != shape {
            return Err("unsupported_key_retry_eval_shape".into());
        }
        Some(Candidate {
            glyph: read_glyph(&old["glyph"])?,
            parts: 0,
            score: old["eval"]["grade"]
                .as_f64()
                .filter(|g| (0.0..=1.).contains(g))
                .ok_or("invalid_key_retry_eval_grade")?,
        })
    };
    let mut erase = Vec::new();
    for neighbor in [
        index.checked_sub(1),
        (index + 1 < slice_count).then_some(index + 1),
    ]
    .into_iter()
    .flatten()
    {
        let a = &state["slices"][neighbor]["alter"];
        if crop_neighbors && !a.is_null() {
            erase.push(read_glyph(&a["glyph"])?);
        }
    }
    let parts = crop_parts(image, rect, il, &erase)?;
    let adj = glyphs::links_at_distance(&parts, il as f64 * 1.5).map_err(|e| e.to_string())?;
    let model = audiveris_classifier::Model::from_json(MODEL).map_err(|e| e.to_string())?;
    let mut improved = false;
    for mask in single_order(&parts, &adj, il) {
        let g = glyphs::compound(&parts, mask);
        if glyphs::weight(&g) < (il.pow(2) as f64 * 0.2).round_ties_even() as usize
            || (g.width as f64) < 0.5 * il as f64
            || g.height < il
        {
            continue;
        }
        if input.peaks.iter().any(|p| {
            let c = (p.min + p.max) as f64 * 0.5;
            rect[0] as f64 <= c
                && c <= (rect[0] + rect[2] - 1) as f64
                && !(g.left as f64 <= c && c <= (g.left as i64 + g.width as i64 - 1) as f64)
        }) {
            continue;
        }
        if let Some(score) = evaluate(&g, sheet_interline, &model, shape, 0.01)? {
            if best.as_ref().is_none_or(|old| old.score < score) {
                best = Some(Candidate {
                    glyph: g,
                    parts: mask,
                    score,
                });
                improved = true;
            } else if improved
                && best.as_ref().is_some_and(|old| {
                    old.score == score && glyphs::glyph(&old.glyph) != glyphs::glyph(&g)
                })
            {
                return Err("unsupported_key_pitch_retry_equal_grade_order".into());
            }
        }
    }
    let mut replaced = false;
    let mut returned = Value::Null;
    if let Some(c) = best {
        if improved {
            state["slices"][index]["eval"] = json!({"shape":shape,"grade":c.score});
            state["slices"][index]["glyph"] = glyphs::glyph(&c.glyph);
        }
        if 0.8 * c.score >= 0.01 {
            if old["alter"].is_null() || old["alter"]["glyph"] != glyphs::glyph(&c.glyph) {
                state["slices"][index]["alter"] =
                    make_alter(&c.glyph, c.score, input.shape, staff)?;
                replaced = true;
            }
            returned = state["slices"][index]["alter"].clone();
        }
    }
    Ok(json!({"eval_improved":improved,"alter_replaced":replaced,"returned_alter":returned}))
}

/// Original fillMissingAlters pitch-window pass. Trigger grades are computed once,
/// then each retry sees any neighbor interpretation replaced by an earlier retry.
/// The caller applies pitch impact again and creates the key ensemble afterward.
pub fn fill_missing_alters(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    input: &ShapePreparation,
    chosen: crate::key_clef::ClefKind,
    state: &mut Value,
) -> Result<Value, String> {
    use crate::key_clef::{KeyAlter, KeyShape, compute_pitched_grades};
    if !(1..=128).contains(&sheet_interline)
        || !(1..=128).contains(&staff.specific_interline)
        || input.slices.len() > 7
    {
        return Err("unsupported_key_pitch_retry_parameters".into());
    }
    if image
        .width
        .checked_mul(image.height)
        .is_none_or(|n| n == 0 || n > crate::MAX_PIXELS || n != image.pixels.len())
        || image.pixels.iter().any(|p| *p != 0 && *p != 255)
    {
        return Err("invalid_key_pitch_retry_image".into());
    }
    let values = state["slices"]
        .as_array()
        .filter(|a| a.len() == input.slices.len())
        .ok_or("invalid_key_pitch_retry_slices")?;
    let mut alters = Vec::new();
    for s in values {
        let a = &s["alter"];
        alters.push(if a.is_null() {
            None
        } else {
            Some(KeyAlter {
                intrinsic_grade: a["grade"].as_f64().ok_or("missing_key_pitch_retry_grade")?,
                measured_pitch: a["measured_pitch"]
                    .as_f64()
                    .ok_or("missing_key_pitch_retry_measured")?,
            })
        });
    }
    let shape = match input.shape {
        Shape::Flat => KeyShape::Flat,
        Shape::Sharp => KeyShape::Sharp,
    };
    let grades = compute_pitched_grades(chosen, shape, &alters).map_err(|e| e.to_string())?;
    let height = (staff.specific_interline as f64 * 2.5).round_ties_even() as i64;
    let mut attempts = Vec::new();
    for i in 0..input.slices.len() {
        if alters[i].is_some() && grades.grades[i] >= 0.1 {
            continue;
        }
        let before = state["slices"][i].clone();
        let b = &before["bounds"];
        let old_rect = [
            b[0].as_i64().ok_or("invalid_key_pitch_retry_x")?,
            b[1].as_i64().ok_or("invalid_key_pitch_retry_y")?,
            b[2].as_i64().ok_or("invalid_key_pitch_retry_width")?,
            b[3].as_i64().ok_or("invalid_key_pitch_retry_height")?,
        ];
        state["slices"][i]["bounds"] = json!(pitch_window_rect(
            staff,
            old_rect,
            input.shape,
            chosen,
            i,
            height as usize
        )?);
        let outcome = retry_slice(image, sheet_interline, staff, input, i, state)?;
        attempts.push(
            json!({"slice_index":i,"before":before,"after":state["slices"][i],"outcome":outcome}),
        );
    }
    state["pitch_window_fallback_complete"] = json!(true);
    Ok(json!({"trigger_pitched_grades":grades,"attempts":attempts}))
}

#[cfg(test)]
mod retry_oracle_tests {
    use super::*;
    use crate::{
        bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
        header_preparation, key_peaks, key_preparation, key_slices, line_completion, page_glyphs,
        scale, sections, staff_projection, system_graph, vertical,
    };
    fn fixture(name: &str) -> (GrayImage, usize, StaffInput, key_slices::Preparation) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let image = GrayImage::decode(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}.png"))).unwrap(),
        )
        .unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        let sc = scale::analyze(&binary).result.unwrap();
        let prep = sections::prepare(&binary, &sc).unwrap();
        let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
        let st = clusters::analyze(&sc, &prep.horizontal, &factory);
        let projection = staff_projection::analyze(&binary, &sc, &prep.horizontal, &st);
        let sticks = bar_sticks::analyze(&binary, &sc, &prep, &projection);
        let slope = factory["line_filtering"]["global_slope"].as_f64().unwrap();
        let systems = system_graph::analyze(&binary, &sc, &st, &sticks, slope);
        let complete = line_completion::analyze(
            &binary,
            &sc,
            &prep,
            &st,
            &projection,
            &sticks,
            &systems,
            slope,
        );
        let (residual, _) =
            page_glyphs::extract(&binary, &complete, sc.interline.main as u32, None).unwrap();
        let grid = grid_ir::build(&residual, &sc, &prep, &complete, &systems, &sticks).unwrap();
        let headers = header_preparation::from_grid(&grid).unwrap();
        let candidates =
            clef_candidates::analyze(&residual, &grid, &headers, sc.interline.main).unwrap();
        let registered =
            clef_registration::analyze(&candidates, &grid, &headers, sc.interline.main).unwrap();
        let staff = key_preparation::staff_inputs(&grid, &registered, sc.interline.main)
            .unwrap()
            .remove(0);
        let p = key_preparation::prepare_staff(&residual, sc.interline.main, &staff).unwrap();
        let h = key_peaks::retrieve(&p, staff.specific_interline).unwrap();
        let shape =
            key_slices::prepare_staff(&residual, sc.interline.main, &staff, &p, &h).unwrap();
        (residual, sc.interline.main, staff, shape)
    }
    #[test]
    fn original_twenty_one_pitch_rectangles_preserve_java_casts_and_odd_heights() {
        let controls: Value = serde_json::from_slice(include_bytes!(
            "../fixtures/official-grid/key-pitch-window-controls.json"
        ))
        .unwrap();
        let cases = controls["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 21);
        let (_, _, sharp, _) = fixture("rendered-header-three-sharps");
        let (_, _, flat, _) = fixture("rendered-header-four-flats");
        for c in cases {
            let shape = if c["shape"] == "SHARP" {
                Shape::Sharp
            } else {
                Shape::Flat
            };
            let staff = if shape == Shape::Sharp { &sharp } else { &flat };
            let mut old = [0; 4];
            for (i, v) in c["original_rect"].as_array().unwrap().iter().enumerate() {
                old[i] = v.as_i64().unwrap();
            }
            let clef: crate::key_clef::ClefKind =
                serde_json::from_value(c["clef_kind"].clone()).unwrap();
            let actual = pitch_window_rect(
                staff,
                old,
                shape,
                clef,
                c["slice_index"].as_u64().unwrap() as usize,
                c["typical_height"].as_u64().unwrap() as usize,
            )
            .unwrap();
            assert_eq!(json!(actual), c["pitch_rect"]);
        }
    }
    #[test]
    fn original_three_extract_alter_controls_preserve_prior_eval_and_identity() {
        let controls: Value = serde_json::from_slice(include_bytes!(
            "../fixtures/official-grid/key-extract-retention-controls.json"
        ))
        .unwrap();
        let cases = controls["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 3);
        let (image, il, staff, slicing) = fixture("rendered-header-one-flat");
        let shape = slicing
            .shapes
            .iter()
            .find(|s| s.shape == Shape::Flat)
            .unwrap();
        for c in cases {
            let c = if c["retention"].is_object() {
                &c["retention"]
            } else {
                c
            };
            let b = &c["before"];
            let mut state = json!({"shape":"FLAT","slices":[{"bounds":b["rect"],"eval":b["eval"],"glyph":b["glyph"],"alter":b["alter"]}]});
            let outcome = retry_slice(&image, il, &staff, shape, 0, &mut state).unwrap();
            let a = &c["after"];
            assert_eq!(state["slices"][0]["bounds"], a["rect"], "{}", c["name"]);
            for field in ["eval", "glyph", "alter"] {
                assert_eq!(state["slices"][0][field], a[field], "{} {field}", c["name"]);
            }
            assert_eq!(
                outcome["returned_alter"].is_null(),
                c["return_is_null"].as_bool().unwrap()
            );
            assert_eq!(outcome["eval_improved"], false);
            assert_eq!(outcome["alter_replaced"], false);
        }
    }
}
