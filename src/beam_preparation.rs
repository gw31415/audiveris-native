// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Bounded native adapter for SpotsBuilder pixel/glyph preparation at
// 9e1e55cd2746037d059345881c53e6a6754bffbd. No JVM/runtime fixture access.
use crate::{
    GrayImage,
    beam_spots::{self, HeaderRect, Parameters, PreparedSpots},
    curves::Curve,
    scale::Scale,
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn integer(value: &Value) -> Result<i32, String> {
    value
        .as_i64()
        .filter(|v| v.unsigned_abs() <= 4_000_000)
        .map(|v| v as i32)
        .ok_or("invalid_beam_header_integer".into())
}
fn curve(line: &Value, budget: &mut usize) -> Result<Curve, String> {
    let points = line["points"]
        .as_array()
        .filter(|p| (2..=4096).contains(&p.len()))
        .ok_or("invalid_beam_header_curve")?;
    *budget += points.len();
    if *budget > 65536 {
        return Err("unsupported_beam_header_spline_budget".into());
    }
    let points = points
        .iter()
        .map(|p| {
            let p = p
                .as_array()
                .filter(|p| p.len() == 2)
                .ok_or("invalid_beam_header_point")?;
            let mut out = [0.; 2];
            for (dst, src) in out.iter_mut().zip(p) {
                *dst = src
                    .as_f64()
                    .filter(|v| v.is_finite() && v.abs() <= 4_000_000.)
                    .ok_or("invalid_beam_header_coordinate")?;
            }
            Ok(out)
        })
        .collect::<Result<Vec<_>, String>>()?;
    Curve::interpolate(points).ok_or("invalid_beam_header_curve".into())
}

/// Original SpotsBuilder uses the first non-null staff header in each system,
/// the system AREA bounds (not staff-left), and the system's outer staff lines.
/// Clipped bounds require certified straight, non-crossing GRID system bands.
pub fn header_rectangles(
    image: &GrayImage,
    grid: &Value,
    headers: &Value,
    interline: usize,
) -> Result<(Vec<HeaderRect>, Value), String> {
    if headers["headers_complete"] != true {
        return Err("incomplete_beam_headers_dependency".into());
    }
    if !(1..=128).contains(&interline) {
        return Err("unsupported_beam_interline".into());
    }
    let rows = grid["staffs"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 1024)
        .ok_or("invalid_beam_staffs")?;
    let hs = headers["staffs"]
        .as_array()
        .filter(|v| v.len() == rows.len())
        .ok_or("invalid_beam_headers")?;
    let mut staffs = BTreeMap::new();
    let mut stops = BTreeMap::new();
    for row in rows {
        let id = row["id"].as_u64().ok_or("invalid_beam_staff_id")?;
        if staffs.insert(id, row).is_some() {
            return Err("duplicate_beam_staff".into());
        }
    }
    for header in hs {
        let id = header["staff_id"]
            .as_u64()
            .ok_or("invalid_beam_header_staff")?;
        if !staffs.contains_key(&id) || stops.insert(id, header).is_some() {
            return Err("invalid_beam_header_membership".into());
        }
    }
    let systems = grid["systems"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 1024)
        .ok_or("invalid_beam_systems")?;
    let clipped = systems
        .iter()
        .any(|s| s["area"]["x_range"] != json!([0, image.width]));
    let layout = if clipped {
        Some(crate::system_area_geometry::build(
            grid,
            image.width,
            image.height,
            interline,
        )?)
    } else {
        None
    };
    let mut seen_systems = BTreeSet::new();
    let mut seen_staffs = Vec::new();
    let mut seen = BTreeSet::new();
    let mut rectangles = Vec::new();
    let mut records = Vec::new();
    let mut budget = 0;
    let margin = (2. * interline as f64).round_ties_even() as i32;
    for (index, system) in systems.iter().enumerate() {
        let sid = system["id"].as_u64().ok_or("invalid_beam_system_id")?;
        if !seen_systems.insert(sid) {
            return Err("duplicate_beam_system".into());
        }
        let left = if let Some(layout) = &layout {
            let ends = crate::beam_dispatch::area_ends(&system["area"]["x_range"], image.width)?;
            if json!(ends) != layout.system_neighbors[index]["x_range"] {
                return Err("inconsistent_beam_header_area_ends".into());
            }
            layout.system_areas[index].header_left()? as i32
        } else {
            0
        };
        let ids = system["staff_ids"]
            .as_array()
            .filter(|v| !v.is_empty() && v.len() <= 1024)
            .ok_or("invalid_beam_system_staffs")?
            .iter()
            .map(|id| id.as_u64().ok_or("invalid_beam_staff_id"))
            .collect::<Result<Vec<_>, _>>()?;
        for id in &ids {
            let row = staffs.get(id).ok_or("missing_beam_staff")?;
            let header = stops.get(id).ok_or("missing_beam_header")?;
            if row["system_id"] != sid || header["system_id"] != sid || !seen.insert(*id) {
                return Err("inconsistent_beam_staff_membership".into());
            }
            seen_staffs.push(*id);
            if row["lines"].as_array().is_none_or(|v| v.len() != 5) {
                return Err("unsupported_beam_staff_lines".into());
            }
        }
        let first = staffs[&ids[0]];
        let last = staffs[ids.last().unwrap()];
        let stop = integer(&stops[&ids[0]]["stop"])?;
        let top = curve(&first["lines"][0], &mut budget)?
            .y(stop as f64)
            .round_ties_even();
        let bottom = curve(&last["lines"][4], &mut budget)?
            .y(stop as f64)
            .round_ties_even();
        if !top.is_finite()
            || !bottom.is_finite()
            || top.abs() > 4_000_000.
            || bottom.abs() > 4_000_000.
            || top >= bottom
            || stop < left
        {
            return Err("invalid_beam_header_geometry".into());
        }
        let rect = HeaderRect {
            left,
            top: top as i32 - margin,
            right: stop,
            bottom: bottom as i32 + margin,
        };
        rectangles.push(rect);
        records.push(json!({"system_id":sid,"first_header_staff_id":ids[0],"rect":[rect.left,rect.top,rect.right-rect.left+1,rect.bottom-rect.top+1]}));
    }
    let order = rows
        .iter()
        .map(|s| s["id"].as_u64().unwrap())
        .collect::<Vec<_>>();
    if order != seen_staffs {
        return Err("inconsistent_beam_staff_order".into());
    }
    Ok((rectangles, json!(records)))
}

/// The CLI uses the upstream default smallBeams=false; an actual small-beam
/// scale still lowers the closing diameter exactly as in buildSheetSpots.
pub fn effective_beam(scale: &Scale) -> Result<usize, String> {
    let beam = scale.beam.thickness.min(
        scale
            .small_beam
            .map_or(scale.beam.thickness, |v| v.thickness),
    );
    if !(1..=128).contains(&beam) {
        return Err("unsupported_beam_spot_thickness".into());
    }
    Ok(beam)
}
pub fn mask_summary(image: &GrayImage) -> Value {
    json!({"width":image.width,"height":image.height,"black_count":image.pixels.iter().filter(|&&v|v==0).count(),"sha256":format!("{:x}",Sha256::digest(&image.pixels))})
}
fn glyph_record(glyph: &GlyphPixels) -> Value {
    let mut weight = 0u64;
    let mut sx = 0u64;
    let mut sy = 0u64;
    let runs = glyph
        .runs
        .iter()
        .map(|r| {
            let len = r.length as u64;
            weight += len;
            sx += len * (glyph.left as u64 + r.position as u64);
            sy += len * (2 * (glyph.top as u64 + r.start as u64) + len - 1) / 2;
            [r.position, r.start, r.length]
        })
        .collect::<Vec<_>>();
    json!({"glyph":{"bounds":[glyph.left as i64,glyph.top as i64,glyph.width as i64,glyph.height as i64],"orientation":"VERTICAL","runs":runs},"weight":weight,"centroid":[(sx as f64/weight as f64).round_ties_even(),(sy as f64/weight as f64).round_ties_even()]})
}

/// Native image pipeline through BEAMS pixel/spot preparation. It intentionally
/// does not claim BlackHeadSizer, system dispatch, BeamInter or full BEAMS.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    headers: &Value,
    stem_seeds: &Value,
    scale: &Scale,
) -> Result<(PreparedSpots, Value), String> {
    if stem_seeds["stem_seeds_complete"] != true {
        return Err("incomplete_beam_stem_seeds_dependency".into());
    }
    let stem = stem_seeds["stem_scaling"]["stem_scale"]["max"]
        .as_u64()
        .filter(|v| *v > 0 && *v <= 128)
        .ok_or("invalid_beam_stem_scale")? as usize;
    let (rectangles, records) = header_rectangles(image, grid, headers, scale.interline.main)?;
    let effective = effective_beam(scale)?;
    let prepared = beam_spots::prepare(
        image,
        Parameters {
            max_stem: stem,
            effective_beam: effective as f64,
        },
        &rectangles,
    )?;
    let mut masks = serde_json::Map::new();
    for (name, mask) in [
        ("no_staff", image),
        ("filtered_horizontal", &prepared.filtered_horizontal),
        ("median", &prepared.median),
        ("gaussian", &prepared.gaussian),
        ("header_erased", &prepared.header_erased),
        ("closed", &prepared.closed),
        ("binarized", &prepared.binarized),
        ("head_spots", &prepared.head_spots),
    ] {
        masks.insert(name.into(), mask_summary(mask));
    }
    let report = json!({"stage":"BEAMS.native_spot_preparation","pixel_preparation_complete":true,"beams_complete":false,"musicxml_complete":false,"scale":{"interline":scale.interline.main,"fore":scale.line.main,"stem_max":stem,"beam_thickness":scale.beam.thickness,"small_beam_thickness":scale.small_beam.map(|b|b.thickness),"small_beams_switch":false,"effective_spot_beam":effective},"header_erase_rectangles":records,"masks":masks,"spot_count":prepared.glyphs.len(),"unassigned_spots":prepared.glyphs.iter().map(glyph_record).collect::<Vec<_>>(),"supported_scope":"default smallBeams=false; five-line GRID; legacy full-width and certified straight clipped system header bounds; four-million-pixel/resource bounds","pending":["BlackHeadSizer and glyph indexing/tagging","system spot dispatch","BeamStructure and BeamInter construction","multiple-measure rests","LEDGERS, HEADS, stem attachment, rhythm and MusicXML"]});
    Ok((prepared, report))
}
