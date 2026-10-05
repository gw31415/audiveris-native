// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Rust adaptation of KeyColumn.retrieveKeys (lookup limits only),
// Staff.getBrowseStop, KeyBuilder.getBrowseRect and KeyExtractor.getProjection.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
// This stage does not recognize, slice, select or freeze a key signature.
use crate::{GrayImage, curves::Curve};
use serde::Serialize;
use serde_json::{Value, json};

/// A staff barline and its incident BarConnectionRelation state, before HEADERS purge.
#[derive(Clone, Debug)]
pub struct Barline {
    pub id: u64,
    pub bounds: [i64; 4],
    pub intrinsic_grade: f64,
    pub has_bar_connection: bool,
}

/// Header.stop must already include HeaderBuilder.setSystemClefStop propagation.
/// Lines and NO_STAFF pixels must come from the same native GRID snapshot.
#[derive(Clone, Debug)]
pub struct StaffInput {
    pub staff_id: u64,
    pub header_start: i64,
    pub header_stop: i64,
    pub clef_stop: Option<i64>,
    pub specific_interline: usize,
    pub lines: Vec<Vec<[f64; 2]>>,
    pub barlines: Vec<Barline>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Projection {
    pub staff_id: u64,
    pub measure_start: i64,
    pub browse_start: i64,
    pub browse_stop: i64,
    /// Original un-clamped KeyBuilder rectangle; x/y/width/height.
    pub browse_rect: [i64; 4],
    /// Sheet-clamped rectangle used for projection y limits.
    pub safe_rect: [i64; 4],
    pub projection_start: i64,
    /// Exact counts of zero-valued NO_STAFF pixels, one value per x.
    pub values: Vec<i32>,
    pub stopping_barline: Option<u64>,
}

/// Read canonical native GRID + registered-clef snapshots without changing either.
/// The current GRID path supports one specific interline, which must be explicit.
pub fn analyze(
    no_staff: &GrayImage,
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    let inputs = staff_inputs(grid, registration, sheet_interline)?;
    let mut output = Vec::with_capacity(inputs.len());
    for input in inputs {
        let sid = input.staff_id;
        // The shared parser checked staff identity and system membership.
        let system_id = &grid["staffs"]
            .as_array()
            .ok_or("missing_key_staffs")?
            .iter()
            .find(|s| s["id"] == sid)
            .ok_or("missing_key_staff")?["system_id"];
        let p = prepare_staff(no_staff, sheet_interline, &input)?;
        output.push(json!({
            "staff_id":sid,"system_id":system_id,"measure_start":p.measure_start,
            "clef_stop":input.clef_stop,"header_stop":input.header_stop,"global_width":15*sheet_interline,
            "browse_start":p.browse_start,"browse_stop":p.browse_stop,
            "raw_browse_rect":p.browse_rect,"clamped_browse_rect":p.safe_rect,
            "projection":{"x_min":p.projection_start,"x_max":p.projection_start+p.values.len() as i64-1,"values":p.values},
            "stopping_barline":p.stopping_barline
        }));
        // Carry staff scale only for mixed rows; preserve uniform-page JSON.
        if input.specific_interline != sheet_interline {
            output.last_mut().unwrap()["specific_interline"] = json!(input.specific_interline);
        }
    }
    Ok(
        json!({"stage":"HEADERS.key_preparation","staffs":output,"headers_complete":false,"key_recognition_complete":false,
        "pending":["HiLo peaks and sharp/flat slices","key alteration classification and clef compatibility","system key alignment and finalization"]}),
    )
}

/// Parse the common native GRID / clef-registration contract for header key stages.
pub fn staff_inputs(
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
) -> Result<Vec<StaffInput>, String> {
    if !(1..=128).contains(&sheet_interline) {
        return Err("unsupported_key_interline".into());
    }
    let staffs = grid["staffs"].as_array().ok_or("missing_key_staffs")?;
    let registered = registration["staffs"]
        .as_array()
        .ok_or("missing_key_registration")?;
    let systems = grid["bar_semantics"]["systems"]
        .as_array()
        .ok_or("missing_key_systems")?;
    if staffs.is_empty() || staffs.len() > 1024 {
        return Err("unsupported_key_staff_count".into());
    }
    if registered.len() > 1024 || systems.len() > 1024 {
        return Err("unsupported_key_lookup_budget".into());
    }
    let mut graph_items = 0usize;
    for system in systems {
        let inters = system["inters"].as_array().ok_or("missing_key_inters")?;
        let relations = system["relations"]
            .as_array()
            .ok_or("missing_key_relations")?;
        graph_items += inters.len().saturating_add(relations.len());
        if inters.len() > 32_768 || relations.len() > 65_536 || graph_items > 262_144 {
            return Err("unsupported_key_graph_budget".into());
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut connected_cache = std::collections::BTreeMap::new();
    let mut output = Vec::with_capacity(staffs.len());
    for staff in staffs {
        let sid = staff["id"].as_u64().ok_or("invalid_key_staff_id")?;
        if !seen.insert(sid) {
            return Err("duplicate_key_staff_id".into());
        }
        let system_id = staff["system_id"].as_u64().ok_or("invalid_key_system_id")?;
        let specific = staff["specific_interline"]
            .as_u64()
            .ok_or("unsupported_key_unknown_specific_interline")?;
        if specific != sheet_interline as u64
            && !(specific > 0 && specific < sheet_interline as u64 && staff["small"] == true)
        {
            return Err("unsupported_key_mixed_specific_interline".into());
        }
        let rows = registered
            .iter()
            .filter(|r| r["staff_id"] == sid)
            .collect::<Vec<_>>();
        if rows.len() != 1 {
            return Err("missing_or_duplicate_key_registration".into());
        }
        let row = rows[0];
        let system_rows = systems
            .iter()
            .filter(|s| s["id"] == system_id)
            .collect::<Vec<_>>();
        if system_rows.len() != 1 {
            return Err("missing_or_duplicate_key_system".into());
        }
        let system = system_rows[0];
        let relations = system["relations"]
            .as_array()
            .ok_or("missing_key_relations")?;
        let inters = system["inters"].as_array().ok_or("missing_key_inters")?;
        let connected = connected_cache.entry(system_id).or_insert_with(|| {
            relations
                .iter()
                .filter(|r| r["kind"] == "BarConnectionRelation")
                .flat_map(|r| [r["source"].as_u64(), r["target"].as_u64()])
                .flatten()
                .collect::<std::collections::BTreeSet<_>>()
        });
        let mut barlines = Vec::new();
        for bar in inters
            .iter()
            .filter(|b| b["kind"] == "BarlineInter" && b["staff_id"] == sid)
        {
            if barlines.len() == 4096 {
                return Err("unsupported_key_barline_budget".into());
            }
            let id = bar["id"].as_u64().ok_or("invalid_key_barline_id")?;
            let b = bar["bounds"]
                .as_array()
                .filter(|b| b.len() == 4)
                .ok_or("invalid_key_barline_bounds")?;
            let mut bounds = [0; 4];
            for (i, v) in b.iter().enumerate() {
                bounds[i] = v.as_i64().ok_or("invalid_key_barline_bounds")?;
            }
            barlines.push(Barline {
                id,
                bounds,
                intrinsic_grade: bar["grade"].as_f64().ok_or("invalid_key_barline_grade")?,
                has_bar_connection: connected.contains(&id),
            });
        }
        let lines = staff["lines"]
            .as_array()
            .ok_or("missing_key_staff_lines")?
            .as_slice();
        if lines.len() != 5 {
            return Err("unsupported_key_staff_configuration".into());
        }
        let lines = lines
            .iter()
            .map(|line| {
                let points = line["points"]
                    .as_array()
                    .ok_or("missing_key_curve_points")?;
                if points.len() > 4096 {
                    return Err("unsupported_key_curve_budget".into());
                }
                points
                    .iter()
                    .map(|point| {
                        let p = point
                            .as_array()
                            .filter(|a| a.len() == 2)
                            .ok_or("invalid_key_curve_point")?;
                        Ok([
                            p[0].as_f64().ok_or("invalid_key_curve_point")?,
                            p[1].as_f64().ok_or("invalid_key_curve_point")?,
                        ])
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()?;
        let clef_stop = if row["clef_stop"].is_null() {
            None
        } else {
            Some(row["clef_stop"].as_i64().ok_or("invalid_key_clef_stop")?)
        };
        let input = StaffInput {
            staff_id: sid,
            header_start: row["header_start"]
                .as_i64()
                .ok_or("missing_key_header_start")?,
            header_stop: row["header_stop"]
                .as_i64()
                .ok_or("missing_key_header_stop")?,
            clef_stop,
            specific_interline: specific as usize,
            lines,
            barlines,
        };
        output.push(input);
    }
    Ok(output)
}

/// Faithful default-constant projection on standard five-line staves.
/// Rejects unsupported inputs before returning a partial projection. No input is mutated.
pub fn prepare_staff(
    no_staff: &GrayImage,
    sheet_interline: usize,
    input: &StaffInput,
) -> Result<Projection, String> {
    let dimensions = no_staff.width.checked_mul(no_staff.height);
    if dimensions.is_none_or(|n| n == 0 || n > crate::MAX_PIXELS || n != no_staff.pixels.len()) {
        return Err("invalid_key_image_dimensions".into());
    }
    if no_staff.pixels.iter().any(|&p| p != 0 && p != 255) {
        return Err("unsupported_key_nonbinary_no_staff".into());
    }
    if !(1..=128).contains(&sheet_interline) || !(1..=128).contains(&input.specific_interline) {
        return Err("unsupported_key_interline".into());
    }
    if input.lines.len() != 5 {
        return Err("unsupported_key_staff_configuration".into());
    }
    if input.header_start < 0
        || input.header_start >= no_staff.width as i64
        || input.header_stop < input.header_start
        || input.header_stop > 4_000_000
        || input
            .clef_stop
            .is_some_and(|x| x < input.header_start || x > 4_000_000)
    {
        return Err("invalid_key_header_bounds".into());
    }
    if input.barlines.len() > 4096 {
        return Err("unsupported_key_barline_budget".into());
    }
    for bar in &input.barlines {
        if bar
            .bounds
            .iter()
            .any(|&v| !(-4_000_000..=4_000_000).contains(&v))
            || bar.bounds[2] <= 0
            || bar.bounds[3] <= 0
            || !(0.0..=1.0).contains(&bar.intrinsic_grade)
        {
            return Err("invalid_key_barline".into());
        }
    }
    for points in &input.lines {
        if points.len() > 4096
            || points
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 4_000_000.0)
            || Curve::interpolate(points.clone()).is_none()
        {
            return Err("invalid_key_staff_curve".into());
        }
    }

    let browse_start = input.clef_stop.unwrap_or(input.header_stop) + 1;
    // HeaderBuilder.maxHeaderWidth is 15.0 * sheet (large) interline.
    let (browse_stop, stopping_barline) = browse_stop(
        browse_start,
        input.header_start + 15 * sheet_interline as i64,
        &input.barlines,
    )?;
    if browse_stop < browse_start {
        return Err("unsupported_empty_key_browse_range".into());
    }
    let ordinate = |points: &Vec<[f64; 2]>| -> Result<i64, String> {
        let y = Curve::interpolate(points.clone())
            .ok_or("invalid_key_staff_curve")?
            .y(browse_start as f64);
        if !y.is_finite() || y.abs() > 4_000_000.0 {
            return Err("unsupported_key_staff_ordinate".into());
        }
        Ok(y.round_ties_even() as i64)
    };
    // Deliberately match the upstream loop: it repeatedly evaluates xMin, not x.
    let first = ordinate(&input.lines[0])?;
    let last = ordinate(&input.lines[4])?;
    if first >= last {
        return Err("invalid_key_staff_height".into());
    }
    let top = first - 2 * input.specific_interline as i64;
    let bottom = last + input.specific_interline as i64;
    let browse_rect = [
        browse_start,
        top,
        browse_stop - browse_start + 1,
        bottom - top + 1,
    ];
    let pixels = project_rectangle(no_staff, input.header_start, browse_rect)?;
    let PixelProjection {
        safe_rect,
        projection_start,
        values,
    } = pixels;
    Ok(Projection {
        staff_id: input.staff_id,
        measure_start: input.header_start,
        browse_start,
        browse_stop,
        browse_rect,
        safe_rect,
        projection_start,
        values,
        stopping_barline,
    })
}

/// The pixel-only part of KeyExtractor.getProjection, also used by refineAreaStart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelProjection {
    pub safe_rect: [i64; 4],
    pub projection_start: i64,
    pub values: Vec<i32>,
}

pub fn project_rectangle(
    no_staff: &GrayImage,
    measure_start: i64,
    raw: [i64; 4],
) -> Result<PixelProjection, String> {
    let dimensions = no_staff.width.checked_mul(no_staff.height);
    if dimensions.is_none_or(|n| n == 0 || n > crate::MAX_PIXELS || n != no_staff.pixels.len()) {
        return Err("invalid_key_image_dimensions".into());
    }
    if no_staff.pixels.iter().any(|&p| p != 0 && p != 255) {
        return Err("unsupported_key_nonbinary_no_staff".into());
    }
    if measure_start < 0
        || measure_start >= no_staff.width as i64
        || raw.iter().any(|&v| !(-4_000_000..=4_000_000).contains(&v))
        || raw[2] <= 0
        || raw[3] <= 0
    {
        return Err("invalid_key_projection_rectangle".into());
    }
    // Sheet.clamp -> Picture.clamp -> intersection with [0,width)x[0,height).
    let left = raw[0].max(0);
    let right = (raw[0] + raw[2]).min(no_staff.width as i64);
    let top = raw[1].max(0);
    let bottom = (raw[1] + raw[3]).min(no_staff.height as i64);
    if left >= right || top >= bottom {
        return Err("unsupported_empty_key_clamped_roi".into());
    }
    let safe_rect = [left, top, right - left, bottom - top];
    let projection_start = measure_start.min(left);
    let width = right - projection_start;
    if width > 8192 || width * (bottom - top) > 1_048_576 || bottom - top > i16::MAX as i64 {
        return Err("unsupported_key_projection_budget".into());
    }
    let mut values = Vec::with_capacity(width as usize);
    for x in projection_start..right {
        let mut count = 0;
        for y in top..bottom {
            if no_staff.pixels[y as usize * no_staff.width + x as usize] == 0 {
                count += 1;
            }
        }
        values.push(count);
    }
    Ok(PixelProjection {
        safe_rect,
        projection_start,
        values,
    })
}

/// Staff.getBrowseStop, preserving stable integer-center bar order.
pub fn browse_stop(
    x_min: i64,
    x_max: i64,
    barlines: &[Barline],
) -> Result<(i64, Option<u64>), String> {
    if !(-4_000_000..=4_000_000).contains(&x_min) || !(-4_000_000..=4_000_000).contains(&x_max) {
        return Err("invalid_key_browse_bounds".into());
    }
    if barlines.len() > 4096 {
        return Err("unsupported_key_barline_budget".into());
    }
    for bar in barlines {
        if bar
            .bounds
            .iter()
            .any(|&v| !(-4_000_000..=4_000_000).contains(&v))
            || bar.bounds[2] <= 0
            || bar.bounds[3] <= 0
            || !(0.0..=1.0).contains(&bar.intrinsic_grade)
        {
            return Err("invalid_key_barline".into());
        }
    }
    let mut bars = barlines.iter().collect::<Vec<_>>();
    bars.sort_by_key(|b| b.bounds[0] + b.bounds[2] / 2);
    for bar in bars {
        if bar.intrinsic_grade < 0.6 || !bar.has_bar_connection {
            continue;
        }
        let start = bar.bounds[0];
        if start > x_max {
            break;
        }
        if start > x_min {
            return Ok((start - 1, Some(bar.id)));
        }
    }
    Ok((x_max, None))
}
