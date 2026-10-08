// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strong SymbolsFilter + bounded SymbolsBuilder/InterFactory continuation.
//! Isolated/two-part classification; compound musical Inter conflicts remain pending.
use crate::{
    GrayImage, beam_dispatch,
    music_measures::{Chord, OwnedMusic},
    shared_sig::{Key, Phase},
};
use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation, extract_glyphs};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
const MODEL: &[u8] = include_bytes!("../crates/audiveris-classifier/models/audiveris-5.11.0.json");
// RestInter.createValid: grow the rest glyph horizontally, intersect the
// actual HeadChord (including its stem tail), then compare center2D strictly.
fn rest_too_close_to_chord(b: [i64; 4], d: i64, c: &Chord) -> bool {
    let h = c.bounds;
    b[0] - d < h[0] + h[2]
        && h[0] < b[0] + b[2] + d
        && b[1] < h[1] + h[3]
        && h[1] < b[1] + b[3]
        && ((h[0] as f64 + h[2] as f64 / 2.) - (b[0] as f64 + b[2] as f64 / 2.)).abs() < d as f64
}

fn white(im: &mut GrayImage, x: i64, y: i64) {
    if x >= 0 && y >= 0 && (x as usize) < im.width && (y as usize) < im.height {
        im.pixels[y as usize * im.width + x as usize] = 255;
    }
}
fn glyph_erase(im: &mut GrayImage, v: &Value) -> Result<(), String> {
    let g = beam_dispatch::glyph_from_json(v)?;
    for r in &g.runs {
        for along in r.start..r.start + r.length {
            let (x, y) = match g.orientation {
                Orientation::Vertical => (r.position, along),
                Orientation::Horizontal => (along, r.position),
            };
            white(im, g.left as i64 + x as i64, g.top as i64 + y as i64);
        }
    }
    Ok(())
}
pub(crate) fn flat_staff_y(s: &Value, index: usize) -> Result<f64, String> {
    let p = s["lines"][index]["points"]
        .as_array()
        .ok_or("music_flat_staff_points_required")?;
    let y = p
        .first()
        .and_then(|a| a[1].as_f64())
        .ok_or("music_flat_staff_y_required")?;
    if p.len() < 2 || p.iter().any(|a| a[1].as_f64() != Some(y)) {
        return Err("music_symbols_curved_or_skew_staff_pending".into());
    }
    Ok(y)
}
fn header_rectangle(
    staff: &crate::grid_ir::StaffGeometry,
    left: i64,
    right: i64,
    interline: usize,
) -> Result<[i64; 4], String> {
    if staff.lines.len() != 5 || right < left || interline == 0 {
        return Err("music_actual_header_geometry_required".into());
    }
    let top = staff.lines[0].y(right as f64);
    let bottom = staff.lines[4].y(right as f64);
    if !top.is_finite() || !bottom.is_finite() || bottom < top {
        return Err("music_actual_header_geometry_required".into());
    }
    // PageCleaner uses a rectangle at the header's right ordinate, even when
    // the retained Staff lines curve across the rectangle's width.
    let margin = (interline as f64 * 0.5).round_ties_even() as i64;
    let top = top.round_ties_even() as i64 - margin;
    let bottom = bottom.round_ties_even() as i64 + margin;
    Ok([left, top, right - left + 1, bottom - top + 1])
}
fn erase_rectangle(im: &mut GrayImage, bounds: [i64; 4]) {
    for y in bounds[1]..bounds[1] + bounds[3] {
        for x in bounds[0]..bounds[0] + bounds[2] {
            white(im, x, y);
        }
    }
}
fn filter(m: &OwnedMusic) -> Result<GrayImage, String> {
    let o = &m.owner;
    let mut im = GrayImage::from_pgm(&o.upstream_output_pixels)?;
    let bank = o.catalog.bank()?;
    for h in o.upstream_report["headers"]["staffs"]
        .as_array()
        .ok_or("music_filter_headers_required")?
    {
        let s = o.upstream_report["grid_ir"]["staffs"]
            .as_array()
            .ok_or("music_filter_staffs_required")?
            .iter()
            .find(|s| s["id"] == h["staff_id"])
            .ok_or("music_filter_staff_required")?;
        if s["lines"].as_array().is_none_or(|a| a.len() != 5)
            || s["specific_interline"] != o.interline
        {
            return Err("music_symbols_uniform_five_lines_required".into());
        }
        let left = h["start"].as_i64().ok_or("music_header_start_required")?;
        let right = h["stop"].as_i64().ok_or("music_header_stop_required")?;
        let staff = o
            .staffs
            .iter()
            .find(|staff| {
                Some(staff.id) == h["staff_id"].as_u64()
                    && Some(staff.system) == h["system_id"].as_u64()
            })
            .ok_or("music_actual_header_staff_required")?;
        erase_rectangle(&mut im, header_rectangle(staff, left, right, o.interline)?);
        if h["clef"]["frozen"] != true {
            return Err("music_frozen_header_clef_required".into());
        }
        glyph_erase(&mut im, &h["clef"]["glyph"])?;
        if !h["time"].is_null() {
            // PageCleaner visits the actual TimeWhole glyph or TimePair numbers.
            // The pair has no synthetic physical glyph; upstream is provenance.
            let snapshot = o.graph.snapshot();
            let pair = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    n["kind"] == h["time"]["kind"]
                        && n["origin"]["system"] == h["system_id"]
                        && n["staff_id"] == h["staff_id"]
                        && n["source_header_id"] == h["time"]["id"]
                })
                .ok_or("music_owned_header_time_required")?;
            if pair["frozen"] != true
                || pair["origin"]["phase"] != "MUSIC_TIME"
                || pair["numerator"] != h["time"]["numerator"]
                || pair["denominator"] != h["time"]["denominator"]
            {
                return Err("music_stale_frozen_header_time".into());
            }
            let glyph_nodes = if pair["kind"] == "TimePairInter" {
                pair["members"]
                    .as_array()
                    .ok_or("music_time_pair_members_required")?
                    .as_slice()
            } else if pair["kind"] == "TimeWholeInter" {
                std::slice::from_ref(pair)
            } else {
                return Err("music_owned_header_time_kind_required".into());
            };
            for member in glyph_nodes {
                let actual = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == member["id"])
                    .ok_or("music_owned_time_number_required")?;
                if actual != member || actual["frozen"] != true {
                    return Err("music_stale_owned_time_number".into());
                }
                let gid = actual["glyph_id"]
                    .as_u64()
                    .and_then(|v| usize::try_from(v).ok())
                    .ok_or("music_time_number_glyph_id_required")?;
                if beam_dispatch::glyph_json(&o.prepared.registry.get(gid)?.glyph)
                    != actual["glyph"]
                {
                    return Err("music_stale_time_number_glyph_registry".into());
                }
                glyph_erase(&mut im, &actual["glyph"])?;
            }
        }
        if !h["key"].is_null() {
            // PageCleaner.visit(KeyInter) erases the actual member glyphs.
            // Source header IDs are provenance; canonical glyph/SIG IDs are owned.
            let snapshot = o.graph.snapshot();
            let key = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    n["kind"] == "KeyInter"
                        && n["origin"]["system"] == h["system_id"]
                        && n["staff_id"] == h["staff_id"]
                        && n["source_header_id"] == h["key"]["id"]
                })
                .ok_or("music_owned_header_key_required")?;
            if key["frozen"] != true || key["fifths"] != h["key"]["fifths"] {
                return Err("music_stale_frozen_header_key".into());
            }
            for member in key["members"]
                .as_array()
                .ok_or("music_header_key_members_required")?
            {
                let actual = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == member["id"])
                    .ok_or("music_owned_key_member_required")?;
                if actual != member || actual["frozen"] != true {
                    return Err("music_stale_owned_key_member".into());
                }
                let gid = actual["glyph_id"]
                    .as_u64()
                    .ok_or("music_key_glyph_id_required")? as usize;
                if beam_dispatch::glyph_json(&o.prepared.registry.get(gid)?.glyph)
                    != actual["glyph"]
                {
                    return Err("music_stale_key_glyph_registry".into());
                }
                glyph_erase(&mut im, &actual["glyph"])?;
            }
        }
    }
    for n in o.graph.snapshot()["inters"]
        .as_array()
        .ok_or("music_filter_SIG_required")?
    {
        match n["kind"].as_str().ok_or("music_cleaner_kind_required")? {
            "HeadInter" => {
                if n["contextual_grade"].as_f64().is_none_or(|g| g < 0.6) {
                    return Err("music_filter_weak_head_optionals_pending".into());
                }
                let a = bank
                    .get(
                        n["shape"].as_str().ok_or("music_head_shape_required")?,
                        o.catalog.stored_point_size,
                    )
                    .map_err(|e| e.to_string())?;
                let slim = a.slim_bounds();
                let x = n["bounds"][0]
                    .as_i64()
                    .ok_or("music_head_bounds_required")?
                    - slim[0] as i64;
                let y = n["bounds"][1]
                    .as_i64()
                    .ok_or("music_head_bounds_required")?
                    - slim[1] as i64;
                let mut fore = BTreeSet::new();
                for p in a.template().points() {
                    let xx = x + p.x as i64;
                    let yy = y + p.y as i64;
                    if p.distance == 0.
                        && xx >= 0
                        && yy >= 0
                        && (xx as usize) < im.width
                        && (yy as usize) < im.height
                        && im.pixels[yy as usize * im.width + xx as usize] == 0
                    {
                        fore.insert((xx, yy));
                    }
                }
                let d = (o.catalog.stored_point_size as f64 / 4. * 0.2).round_ties_even() as i64;
                for (x, y) in fore {
                    for yy in y - d..=y + d {
                        for xx in x - d..=x + d {
                            white(&mut im, xx, yy);
                        }
                    }
                }
            }
            "StemInter" => {
                if n["contextual_grade"].as_f64().is_none_or(|g| g < 0.7) {
                    return Err("music_filter_weak_stem_optionals_pending".into());
                }
                glyph_erase(&mut im, &n["glyph"])?;
            }
            "LedgerInter" => {
                // PageCleaner.visit(LedgerInter) -> processGlyph, using the
                // canonical foreground runs rather than theoretical bounds.
                // PageCleaner.canHide uses best grade / Grades.goodInterGrade.
                if n["shape"] != "LEDGER" {
                    return Err("music_ledger_shape_required".into());
                }
                let best = n["contextual_grade"].as_f64().or(n["grade"].as_f64());
                if n["frozen"] != true && best.is_none_or(|g| !g.is_finite() || g < 0.4) {
                    return Err("music_filter_weak_ledger_optionals_pending".into());
                }
                let id = n["glyph_id"]
                    .as_u64()
                    .and_then(|id| usize::try_from(id).ok())
                    .ok_or("music_ledger_glyph_id_required")?;
                let actual = &o.prepared.registry.get(id)?.glyph;
                if beam_dispatch::glyph_json(actual) != n["glyph"] {
                    return Err("stale_music_ledger_glyph_registry".into());
                }
                glyph_erase(&mut im, &n["glyph"])?;
            }
            "BarlineInter" => {
                if n["median"][0] != n["median"][2] {
                    return Err("music_cleaner_sloping_barline_raster_pending".into());
                }
                let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
                    .map_err(|_| "music_bar_bounds_required")?;
                // Observed axis-aligned raster from the saved prototype. Full
                // BasicStroke cap/join/area parity is deliberately not claimed.
                let d = (o.interline as f64 * 0.1).round_ties_even() as i64;
                for y in b[1] - d..b[1] + b[3] + d {
                    for x in b[0] - d..b[0] + b[2] + d {
                        white(&mut im, x, y);
                    }
                }
            }
            "BeamInter" | "BeamHookInter" => {
                if n["contextual_grade"].as_f64().is_none_or(|g| g < 0.7) {
                    return Err("music_filter_weak_beam_optionals_pending".into());
                }
                for [y, left, right] in crate::music_beams::area_rows(n, o.interline)? {
                    for x in left..right {
                        white(&mut im, x, y);
                    }
                }
            }
            "BeamGroupInter" => {}
            "HeadChordInter" | "StaffBarlineInter" | "ClefInter" => {}
            "TimePairInter" | "TimeNumberInter" | "TimeWholeInter" => {
                if n["frozen"] != true || n["origin"]["phase"] != "MUSIC_TIME" {
                    return Err("music_nonheader_pair_cleaner_pending".into());
                }
            }
            "KeyInter" | "KeyAlterInter" => {
                if n["frozen"] != true || n["origin"]["phase"] != "MUSIC_KEY" {
                    return Err("music_nonheader_key_cleaner_pending".into());
                }
            }
            _ => return Err("music_cleaner_beam_ledger_other_shapes_pending".into()),
        }
    }
    Ok(im)
}
pub(crate) fn time(shape: &str) -> Option<(u32, u32)> {
    match shape {
        "COMMON_TIME" | "TIME_FOUR_FOUR" => Some((4, 4)),
        "CUT_TIME" | "TIME_TWO_TWO" => Some((2, 2)),
        "TIME_TWO_FOUR" => Some((2, 4)),
        "TIME_THREE_FOUR" => Some((3, 4)),
        "TIME_FIVE_FOUR" => Some((5, 4)),
        "TIME_THREE_EIGHT" => Some((3, 8)),
        "TIME_SIX_EIGHT" => Some((6, 8)),
        _ => None,
    }
}
fn checked(
    shape: &str,
    g: &GlyphPixels,
    s: &Value,
    il: usize,
    nodes: &[Value],
) -> Result<Option<String>, String> {
    let y = g.top as f64 + g.height as f64 / 2.;
    let scale = il as f64;
    let pitch = 2. * (y - flat_staff_y(s, 2)?) / scale;
    let left = s["left"].as_f64().ok_or("music_staff_left_required")?;
    let right = s["right"].as_f64().ok_or("music_staff_right_required")?;
    if (g.left as f64 + g.width as f64) < left || g.left as f64 > right {
        return Ok(None);
    }
    if time(shape).is_some() {
        return Ok(
            (pitch.abs() < 4. && pitch.abs() <= 1. && g.height as f64 / scale <= 4.5)
                .then(|| shape.to_owned()),
        );
    }
    let rest = matches!(
        shape,
        "HW_REST_set"
            | "BREVE_REST"
            | "LONG_REST"
            | "QUARTER_REST"
            | "EIGHTH_REST"
            | "ONE_16TH_REST"
            | "ONE_32ND_REST"
            | "ONE_64TH_REST"
            | "ONE_128TH_REST"
    );
    if rest {
        let gap = (flat_staff_y(s, 0)? - (g.top as f64 + g.height as f64))
            .max(g.top as f64 - flat_staff_y(s, 4)?)
            .max(0.);
        if gap > (scale * 8.).round_ties_even() || (shape == "LONG_REST" && pitch.abs() > 0.5) {
            return Ok(None);
        }
        if shape == "HW_REST_set" {
            let d = (scale * 0.2).round_ties_even();
            if nodes.iter().filter(|n| n["kind"] == "StemInter").any(|n| {
                let b = &n["bounds"];
                let x = n["median"][0].as_f64().unwrap_or(f64::NAN);
                x >= g.left as f64 - d
                    && x < g.left as f64 + g.width as f64 + d
                    && y >= b[1].as_f64().unwrap_or(f64::NAN)
                    && y < b[1].as_f64().unwrap_or(f64::NAN) + b[3].as_f64().unwrap_or(0.)
            }) {
                return Ok(None);
            }
            return Ok(match (2. * pitch).round_ties_even() as i32 {
                -13 | -9 | -5 | -1 | 3 | 7 | 11 | 15 => Some("HALF_REST".into()),
                -15 | -11 | -7 | -3 | 1 | 5 | 9 | 13 => Some("WHOLE_REST".into()),
                _ => None,
            });
        }
    }
    // Other physical shapes remain observations; no unchecked musical Inter
    // is created. Their specific ShapeChecker/InterFactory dependencies remain.
    Ok(Some(shape.to_owned()))
}
// Glyphs.buildLinks + GlyphCluster.decompose, deliberately bounded to
// two-part components. Larger clusters refuse rather than discarding parts.
fn symbol_candidates(
    glyphs: &[GlyphPixels],
    il: usize,
) -> Result<Vec<(GlyphPixels, Vec<GlyphPixels>)>, String> {
    let minimum = (0.03 * (il * il) as f64).round_ties_even() as usize;
    let mut parts = glyphs
        .iter()
        .filter(|g| crate::clef_candidates::weight(g) >= minimum)
        .cloned()
        .collect::<Vec<_>>();
    parts.sort_by_key(|g| g.left);
    let mut neighbors = vec![Vec::new(); parts.len()];
    let gap = (0.8 * il as f64).ceil() as i64;
    for (i, a) in parts.iter().enumerate() {
        for (j, b) in parts.iter().enumerate().skip(i + 1) {
            if b.left as i64 >= a.left as i64 + a.width as i64 + gap {
                break;
            }
            if b.top as i64 >= a.top as i64 + a.height as i64 + gap
                || a.top as i64 - gap >= b.top as i64 + b.height as i64
            {
                continue;
            }
            let pair = [a.clone(), b.clone()];
            let links = crate::clef_candidates::links_at_distance(&pair, 0.8 * il as f64)
                .map_err(|e| e.to_string())?;
            if links[0] != 0 {
                neighbors[i].push(j);
                neighbors[j].push(i);
            }
        }
    }
    let mut visited = vec![false; parts.len()];
    let mut candidates = Vec::new();
    for start in 0..parts.len() {
        if visited[start] {
            continue;
        }
        let mut component = vec![start];
        visited[start] = true;
        let mut at = 0;
        while at < component.len() {
            for &j in &neighbors[component[at]] {
                if !visited[j] {
                    visited[j] = true;
                    component.push(j);
                }
            }
            at += 1;
        }
        if component.len() > 2 {
            return Err("music_symbols_more_than_two_cluster_parts_pending".into());
        }
        component.sort_by_key(|&i| std::cmp::Reverse(crate::clef_candidates::weight(&parts[i])));
        let group = component
            .iter()
            .map(|&i| parts[i].clone())
            .collect::<Vec<_>>();
        // Original stable heavier seed first: seed, its compound, next seed.
        let masks: &[u16] = if group.len() == 2 { &[1, 3, 2] } else { &[1] };
        for &mask in masks {
            let g = crate::clef_candidates::compound(&group, mask);
            let weight = crate::clef_candidates::weight(&g);
            if weight as f64 / ((il * il) as f64) < 0.04 {
                continue;
            }
            // SymbolAdapter isTooLarge applies to clustered glyphs only.
            // The left-of-system brace exception remains outside our scope.
            if group.len() == 2 && (g.width > 4 * il || g.height > 10 * il) {
                return Err("music_symbols_cluster_size_or_brace_pending".into());
            }
            candidates.push((
                g,
                group
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, g)| g.clone())
                    .collect(),
            ));
        }
    }
    Ok(candidates)
}
pub fn continue_symbols(mut m: OwnedMusic) -> Result<OwnedMusic, String> {
    if m.owner.prepared.report["schema"] != "audiveris-native.music-measures-head-chords.v1"
        || m.owner.prepared.report["shared_sig"] != m.owner.graph.snapshot()
    {
        return Err("retained_measures_chords_owner_required".into());
    }
    let im = filter(&m)?;
    let il = m.owner.interline;
    let glyphs = extract_glyphs(
        BinaryPage::new(im.width, im.height, &im.pixels).map_err(|e| e.to_string())?,
        Orientation::Vertical,
        (0, 0),
        Limits::default(),
    )
    .map_err(|e| e.to_string())?;
    let model = audiveris_classifier::Model::from_json(MODEL).map_err(|e| e.to_string())?;
    let initial = m.owner.graph.snapshot();
    let nodes = initial["inters"].as_array().unwrap();
    let staffs = m.owner.upstream_report["grid_ir"]["staffs"]
        .as_array()
        .ok_or("music_symbol_staffs_required")?;
    let mut graph = m.owner.graph.clone();
    let mut registry = m.owner.prepared.registry.clone();
    let mut trials = Vec::new();
    let mut accepted = Vec::new();
    let mut new_chords = Vec::new();
    let candidates = symbol_candidates(&glyphs, il)?;
    for (g, parts) in &candidates {
        let part_count = parts.len();
        let mut part_ids = Vec::new();
        for part in parts {
            let id = registry.register(part)?;
            registry.add_group(id, beam_dispatch::Group::Symbol)?;
            part_ids.push(id);
        }
        let weight: usize = g.runs.iter().map(|r| r.length).sum();
        if weight < (0.03 * (il * il) as f64).round_ties_even() as usize
            || (weight as f64) / ((il * il) as f64) < 0.04
        {
            continue;
        }
        let y = g.top as f64 + g.height as f64 / 2.;
        let s = staffs
            .iter()
            .map(|s| Ok(((y - flat_staff_y(s, 2)?).abs(), s)))
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .ok_or("missing_symbol_staff")?
            .1;
        let sid = s["system_id"].as_u64().ok_or("missing_symbol_system")?;
        let staff = s["id"].as_u64().ok_or("missing_symbol_staff_id")?;
        let features = audiveris_glyph_features::from_runs(
            audiveris_glyph_features::Geometry {
                width: g.width,
                height: g.height,
                left: g.left,
                top: g.top,
                interline: il as u32,
                orientation: audiveris_glyph_features::Orientation::Vertical,
            },
            &g.runs
                .iter()
                .map(|r| audiveris_glyph_features::Run {
                    position: r.position,
                    start: r.start,
                    length: r.length,
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;
        let scores = model.evaluate(&features).map_err(|e| e.to_string())?.scores;
        let mut ranks = (0..scores.len()).collect::<Vec<_>>();
        ranks.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
        let gid = registry.register(g)?;
        registry.add_group(gid, beam_dispatch::Group::Symbol)?;
        let mut evaluations = Vec::new();
        for i in ranks {
            if scores[i] < 0.15 || evaluations.len() == 2 {
                break;
            }
            let Some(shape) = checked(&model.output_labels()[i], g, s, il, nodes)? else {
                continue;
            };
            if evaluations.iter().any(|e: &Value| e["shape"] == shape) {
                continue;
            }
            evaluations.push(json!({"shape":shape,"score":scores[i],"supported_specific_checks":time(&shape).is_some() || shape.ends_with("_REST")}));
        }
        let mut created = Vec::new();
        for e in &evaluations {
            let shape = e["shape"].as_str().unwrap();
            let grade = 0.8 * e["score"].as_f64().unwrap();
            let b = [g.left as i64, g.top as i64, g.width as i64, g.height as i64];
            let rest = shape.ends_with("_REST");
            let meter = time(shape);
            if shape == "FLAT" {
                if m.owner.catalog.family != "Bravura" {
                    return Err("music_alter_font_area_pitch_offset_pending".into());
                }
                crate::music_alters::require_empty_links(shape, b, sid, il, &graph.snapshot())?;
                let inputs = crate::key_preparation::staff_inputs(
                    &m.owner.upstream_report["grid_ir"],
                    &m.owner.upstream_report["clef_registration"],
                    il,
                )?;
                let input = inputs
                    .iter()
                    .find(|s| s.staff_id == staff)
                    .ok_or("music_alter_staff_required")?;
                let mut data = crate::key_components::make_alter(
                    g,
                    e["score"].as_f64().unwrap(),
                    crate::key_slices::Shape::Flat,
                    input,
                )?;
                data["kind"] = json!("AlterInter");
                data["staff_id"] = json!(staff);
                data["glyph_id"] = json!(gid);
                data["abnormal"] = json!(true);
                let key = Key {
                    system: sid,
                    phase: Phase::Symbol,
                    local: gid * 2 + created.len(),
                };
                graph.vertex(key, data, false)?;
                created.push(key);
                continue;
            }
            if matches!(shape, "CAESURA" | "BREATH_MARK") {
                if let Some((chord, relation)) =
                    crate::music_pauses::lookup(b, sid, staff, il, &m.measures, &m.chords)?
                {
                    let key = Key {
                        system: sid,
                        phase: Phase::Symbol,
                        local: gid * 2 + created.len(),
                    };
                    graph.vertex(key,json!({"kind":if shape=="CAESURA"{"CaesuraInter"}else{"BreathMarkInter"},"shape":shape,"bounds":b,"glyph":beam_dispatch::glyph_json(g),"glyph_id":gid,"staff_id":staff,"grade":grade,"abnormal":false}),false)?;
                    graph.relations(sid, Phase::Symbol, vec![(chord, key, relation)])?;
                    created.push(key);
                }
                continue;
            }
            if let Some(flag_value) = crate::music_flags::value(shape) {
                let max = m.owner.upstream_report["stem_seeds"]["stem_scaling"]["stem_scale"]["max"]
                    .as_u64()
                    .ok_or("flag_max_stem_required")? as usize;
                if let Some((stem, rel)) =
                    crate::music_flags::lookup(shape, b, il, max, sid, &graph.snapshot())?
                {
                    let key = Key {
                        system: sid,
                        phase: Phase::Symbol,
                        local: gid * 2 + created.len(),
                    };
                    let data = json!({"kind":"FlagInter","shape":shape,"bounds":b,"glyph":beam_dispatch::glyph_json(g),"glyph_id":gid,"staff_id":staff,"grade":grade,"value":flag_value});
                    graph.vertex(key, data, false)?;
                    graph.relations(sid, Phase::Symbol, vec![(key, stem, rel)])?;
                    created.push(key);
                    accepted.push(
                        graph.snapshot()["inters"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|n| n["id"] == graph.id(key).unwrap())
                            .unwrap()
                            .clone(),
                    );
                }
                continue;
            }
            if meter.is_none() && !rest {
                continue;
            }
            let centroid = beam_dispatch::centroid(g)?;
            let cx = centroid[0] as i64;
            let mi = m.measures.iter().position(|mm| {
                mm.system == sid && mm.staff == staff && mm.left < cx && cx <= mm.right
            });
            if rest {
                let Some(mi) = mi else {
                    continue;
                };
                let d = (il as f64 * 0.5).round_ties_even() as i64;
                if m.chords
                    .iter()
                    .filter(|c| {
                        c.measure == mi
                            && c.members.first().is_some_and(|n| n["kind"] == "HeadInter")
                    })
                    .any(|c| rest_too_close_to_chord(b, d, c))
                {
                    continue;
                }
            }
            let key = Key {
                system: sid,
                phase: Phase::Symbol,
                local: gid * 2 + created.len(),
            };
            let mut data = json!({"kind":if rest{"RestInter"}else{"TimeWholeInter"},"shape":shape,"bounds":b,"glyph":beam_dispatch::glyph_json(g),"glyph_id":gid,"staff_id":staff,"grade":grade});
            if let Some((num, den)) = meter {
                data["numerator"] = json!(num);
                data["denominator"] = json!(den);
            } else {
                data["pitch"] = json!(2. * (centroid[1] as f64 - flat_staff_y(s, 2)?) / il as f64);
            }
            let id = graph.vertex(key, data.clone(), false)?;
            created.push(key);
            data["id"] = json!(id);
            data["origin"] = json!(key);
            accepted.push(data.clone());
            if rest {
                let mi = mi.unwrap();
                let ck = Key {
                    system: sid,
                    phase: Phase::Chord,
                    local: m
                        .chords
                        .iter()
                        .chain(&new_chords)
                        .filter(|c: &&Chord| c.key.system == sid)
                        .count()
                        + 1,
                };
                let cid=graph.vertex(ck,json!({"kind":"RestChordInter","bounds":b,"staff_id":staff,"part_id":m.measures[mi].part,"measure_index":m.measures[mi].index,"member_ids":[id],"duration":null,"voice":null}),false)?;
                graph.relations(
                    sid,
                    Phase::Chord,
                    vec![(ck, key, json!({"kind":"Containment"}))],
                )?;
                new_chords.push(Chord {
                    kind: crate::music_measures::ChordKind::Rest,
                    key: ck,
                    id: cid,
                    stem: None,
                    members: vec![data],
                    measure: mi,
                    bounds: b,
                });
            }
        }
        if created.len() == 2 {
            graph.relations(
                sid,
                Phase::Symbol,
                vec![(
                    created[0],
                    created[1],
                    json!({"kind":"Exclusion","cause":"OVERLAP"}),
                )],
            )?;
        }
        trials.push(json!({"glyph_id":gid,"bounds":[g.left,g.top,g.width,g.height],"weight":weight,"part_count":part_count,"part_glyph_ids":part_ids,"created_inter_ids":created.iter().map(|key| graph.id(*key).unwrap()).collect::<Vec<_>>(),"evaluations":evaluations}));
    }
    m.owner
        .prepared
        .registry
        .commit_symbol_extension(registry)?;
    m.owner.graph = graph;
    m.chords.extend(new_chords);
    let dot_late_checks = crate::music_dots::late_checks(
        &mut m.owner.graph,
        &m.owner.prepared.registry,
        &trials,
        staffs,
        &m.measures,
        &m.chords,
        il,
    )?;
    let previous = std::mem::take(&mut m.owner.prepared.report);
    m.owner.prepared.report = json!({"schema":"audiveris-native.music-symbols-isolated.v1","measures_head_chords":previous,"symbol_filter_PGM_SHA256":format!("{:x}",Sha256::digest(im.pgm())),"symbol_glyphs":glyphs.iter().map(beam_dispatch::glyph_json).collect::<Vec<_>>(),"classifier_trials":trials,"symbol_cluster_scope":"isolated-and-two-parts","symbols":accepted,"shared_sig":m.owner.graph.snapshot(),"scope":"strong heads/stems/ledgers, flat uniform staff and axis-aligned barline erasure; isolated/two-part classification with bounded unsupported-support-free Rest/TimeWhole SIG exclusions; actual Graph/registry/cache retained","pending":["clusters above two parts/other physical symbols and supported/frozen conflict reduction/fine small-chord glyphs/weak optionals","factory.lateChecks/symbol epilog/flags/dots/accidentals/LINKS/rhythm/export","general stroked areas/full symbol reduction/cleaner/epilog","nonzero skew/real scans/special staff/full24shapes/32sources"],"heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
    m.owner.prepared.report["dot_late_checks"] = dot_late_checks;
    Ok(m)
}

#[cfg(test)]
mod cluster_boundary {
    use super::*;
    #[test]
    fn original_rest_admission_uses_chord_tail_and_strict_dx_boundaries() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rest-chord-admission/controls.json"
        ))
        .unwrap();
        let head = json!({"id":1,"kind":"HeadInter","bounds":original["head_bounds"]});
        let stem = json!({"id":2,"kind":"StemInter","bounds":original["stem_bounds"]});
        let bounds = crate::head_chord_geometry::calculate(&[&head], Some(&stem))
            .unwrap()
            .bounds
            .unwrap();
        assert_eq!(json!(bounds), original["chord_bounds"]);
        let key = Key {
            system: 1,
            phase: Phase::Chord,
            local: 1,
        };
        let chord = Chord {
            kind: crate::music_measures::ChordKind::Head,
            key,
            id: 3,
            stem: Some(Key {
                system: 1,
                phase: Phase::Stem,
                local: 1,
            }),
            members: vec![head],
            measure: 0,
            bounds,
        };
        let d = (0.5 * original["interline"].as_f64().unwrap()).round_ties_even() as i64;
        assert_eq!(json!(d), original["minimum_dx"]);
        for case in original["cases"].as_array().unwrap() {
            let b = serde_json::from_value(case["rest_bounds"].clone()).unwrap();
            assert_eq!(
                !rest_too_close_to_chord(b, d, &chord),
                case["accepted"].as_bool().unwrap(),
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn original_two_part_cluster_order_and_larger_cluster_refusal() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-ledger/residue-parts.json"
        ))
        .unwrap();
        let parts = fixture
            .as_array()
            .unwrap()
            .iter()
            .map(beam_dispatch::glyph_from_json)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let candidates = symbol_candidates(&parts, 11).unwrap();
        assert_eq!(
            candidates
                .iter()
                .map(|(_, parts)| parts.len())
                .collect::<Vec<_>>(),
            vec![1, 2, 1]
        );
        assert_eq!(crate::clef_candidates::weight(&candidates[1].0), 10);
        let mut three = parts;
        let mut extra = three[0].clone();
        extra.left += 3;
        three.push(extra);
        assert_eq!(
            symbol_candidates(&three, 11).unwrap_err(),
            "music_symbols_more_than_two_cluster_parts_pending"
        );
        assert!(symbol_candidates(&[], 11).unwrap().is_empty());
    }
}

#[cfg(test)]
mod header_tests;
#[cfg(test)]
pub(crate) use header_tests::assert_real_headers;
