// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// HeaderTimeBuilder / TimeBuilder.Parameters at
// 9e1e55cd2746037d059345881c53e6a6754bffbd. No time recognition implied.
use crate::{
    GrayImage,
    curves::Curve,
    key_preparation::{StaffInput, browse_stop, project_rectangle},
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Parameters {
    pub max_eval_rank: usize,
    pub roi_width: i64,
    pub max_first_space_width: i64,
    pub max_inner_space: i64,
    pub y_margin: i64,
    pub min_time_width: i64,
    pub max_time_width: i64,
    pub max_halves_dx: i64,
    pub min_part_weight: i64,
    pub max_part_gap: f64,
    pub min_whole_time_weight: i64,
    pub min_half_time_weight: i64,
    pub max_space_cumul: i32,
}
impl Parameters {
    pub fn new(sheet_interline: usize, specific_interline: usize) -> Result<Self, String> {
        if !(1..=128).contains(&sheet_interline) || !(1..=128).contains(&specific_interline) {
            return Err("unsupported_time_interline".into());
        }
        let large = sheet_interline as f64;
        let specific = specific_interline as f64;
        let pixels = |v: f64| v.round_ties_even() as i64;
        Ok(Self {
            max_eval_rank: 3,
            roi_width: pixels(4.0 * large),
            max_first_space_width: pixels(2.5 * large),
            max_inner_space: pixels(0.5 * large),
            y_margin: pixels(0.1 * specific),
            min_time_width: pixels(specific),
            max_time_width: pixels(2.0 * specific),
            max_halves_dx: pixels(specific),
            min_part_weight: pixels(0.01 * specific * specific),
            max_part_gap: pixels(specific) as f64,
            min_whole_time_weight: pixels(specific * specific),
            min_half_time_weight: pixels(0.75 * specific * specific),
            max_space_cumul: pixels(0.4 * specific) as i32,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Space {
    pub start: i64,
    pub stop: i64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Preparation {
    pub staff_id: u64,
    pub browse_start: i64,
    pub browse_stop: i64,
    pub raw_roi: [i64; 4],
    pub roi: [i64; 4],
    pub projection: Vec<i32>,
    pub spaces: Vec<Space>,
    pub start: Option<i64>,
    pub stop: Option<i64>,
    pub candidate_rectangles: Option<[[i64; 4]; 3]>,
    pub stopping_barline: Option<u64>,
    pub parameters: Parameters,
}

/// Fresh automatic HEADERS only: header_stop must include system key propagation.
/// Existing interactive timeRange overrides are deliberately outside this API.
pub fn prepare_staff(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
) -> Result<Preparation, String> {
    let parameters = Parameters::new(sheet_interline, staff.specific_interline)?;
    if staff.lines.len() != 5 || staff.header_stop < 0 || staff.header_stop >= image.width as i64 {
        return Err("invalid_time_staff_geometry".into());
    }
    let mut curves = Vec::new();
    for points in &staff.lines {
        if points.len() > 4096
            || points
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 4_000_000.0)
        {
            return Err("invalid_time_staff_curve".into());
        }
        curves.push(Curve::interpolate(points.clone()).ok_or("invalid_time_staff_curve")?);
    }
    let start = staff.header_stop; // No +1: HeaderTimeColumn.allocateBuilder.
    let (stop, stopping_barline) =
        browse_stop(start, start + parameters.roi_width - 1, &staff.barlines)?;
    let y = |line: usize, x: i64| -> Result<i64, String> {
        let ordinate = curves[line].y(x as f64);
        if !ordinate.is_finite() || ordinate.abs() > 4_000_000.0 {
            return Err("invalid_time_staff_ordinate".into());
        }
        Ok(ordinate.round_ties_even() as i64)
    };
    let top = y(0, start)?.min(y(0, stop)?);
    // Upstream evaluates the bottom line at stop twice, not at both endpoints.
    let bottom = y(4, stop)?;
    let raw_roi = [start, top, stop - start + 1, bottom - top + 1];
    let projected = project_rectangle(image, start, raw_roi)?;
    let roi = projected.safe_rect;
    let projection = projected.values;
    let spaces = find_spaces(roi[0], &projection, parameters.max_space_cumul)?;
    let refined =
        if spaces.len() >= 2 && spaces[0].stop - roi[0] <= parameters.max_first_space_width {
            Some((spaces[0].stop, spaces.last().unwrap().start))
        } else {
            None
        };
    let candidate_rectangles = refined
        .filter(|(a, b)| b - a + 1 >= parameters.min_time_width)
        .map(|(a, b)| {
            let margin = parameters.y_margin;
            let width = b - a + 1;
            let half = roi[3] / 2;
            [
                [a, roi[1] + margin, width, roi[3] - 2 * margin],
                [a, roi[1] + margin, width, half - 2 * margin],
                [a, roi[1] + roi[3] - half + margin, width, half - 2 * margin],
            ]
        });
    if candidate_rectangles
        .as_ref()
        .is_some_and(|r| r.iter().any(|r| r[3] <= 0))
    {
        return Err("unsupported_time_empty_candidate_rectangle".into());
    }
    Ok(Preparation {
        staff_id: staff.staff_id,
        browse_start: start,
        browse_stop: stop,
        raw_roi,
        roi,
        projection,
        spaces,
        start: refined.map(|r| r.0),
        stop: refined.map(|r| r.1),
        candidate_rectangles,
        stopping_barline,
        parameters,
    })
}

/// Includes single-column and terminal spaces. Range boundaries retain whitespace.
pub fn find_spaces(x_min: i64, values: &[i32], max_cumul: i32) -> Result<Vec<Space>, String> {
    if !(0..=4_000_000).contains(&x_min)
        || values.len() > 8192
        || max_cumul < 0
        || values.iter().any(|&n| !(0..=32767).contains(&n))
    {
        return Err("invalid_time_projection".into());
    }
    let mut spaces = Vec::new();
    let mut start = None;
    for (i, &value) in values.iter().enumerate() {
        let x = x_min + i as i64;
        if value <= max_cumul {
            start.get_or_insert(x);
        } else if let Some(a) = start.take() {
            spaces.push(Space {
                start: a,
                stop: x - 1,
            });
        }
    }
    if let Some(a) = start {
        spaces.push(Space {
            start: a,
            stop: x_min + values.len() as i64 - 1,
        });
    }
    Ok(spaces)
}

/// Join native GRID/clef geometry with completed native system-key propagation.
pub fn after_key_staffs(
    grid: &serde_json::Value,
    registration: &serde_json::Value,
    key_column: &serde_json::Value,
    sheet_interline: usize,
) -> Result<Vec<StaffInput>, String> {
    if key_column["stage"] != "HEADERS.key_column" || key_column["key_column_complete"] != true {
        return Err("incomplete_time_key_dependency".into());
    }
    let mut inputs = crate::key_preparation::staff_inputs(grid, registration, sheet_interline)?;
    let systems = key_column["systems"]
        .as_array()
        .filter(|s| s.len() <= 1024)
        .ok_or("invalid_time_key_systems")?;
    let mut stops = std::collections::BTreeMap::new();
    for system in systems {
        let rows = system["after_system_key_stop"]
            .as_array()
            .filter(|s| s.len() <= 1024)
            .ok_or("invalid_time_key_rows")?;
        for row in rows {
            let id = row["staff_id"]
                .as_u64()
                .ok_or("invalid_time_key_staff_id")?;
            let start = row["header_start"]
                .as_i64()
                .ok_or("invalid_time_key_header_start")?;
            let stop = row["header_stop"]
                .as_i64()
                .ok_or("invalid_time_key_header_stop")?;
            if stops.insert(id, (start, stop)).is_some() {
                return Err("duplicate_time_key_staff".into());
            }
        }
    }
    if stops.len() != inputs.len() {
        return Err("incomplete_time_key_staffs".into());
    }
    for input in &mut inputs {
        let (start, stop) = stops.get(&input.staff_id).ok_or("missing_time_key_staff")?;
        if *start != input.header_start || *stop < input.header_stop || *stop > 4_000_000 {
            return Err("inconsistent_time_key_header".into());
        }
        input.header_stop = *stop;
    }
    Ok(inputs)
}
