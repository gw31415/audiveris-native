// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native STEM_SEEDS orchestration and original StaffManager clipped neighbor-chain geometry.
use crate::{GrayImage, curves::Curve, scale::Scale, stem_checks, stem_scale, vertical_sticks};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
struct Staff {
    id: u64,
    left: f64,
    right: f64,
    top: Curve,
    bottom: Curve,
}
fn finite(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|v| v.is_finite() && v.abs() <= 4_000_000.)
        .ok_or("invalid_seed_coordinate".into())
}
fn curve(line: &Value) -> Result<Curve, String> {
    let points = line["points"]
        .as_array()
        .filter(|p| p.len() <= 4096)
        .ok_or("invalid_seed_staff_points")?;
    let ps = points
        .iter()
        .map(|p| Ok([finite(&p[0])?, finite(&p[1])?]))
        .collect::<Result<Vec<_>, String>>()?;
    Curve::interpolate(ps).ok_or("invalid_seed_staff_curve".into())
}
/// Original StaffManager.getStavesOf area containment and getClosestStaff
/// integer distance selection, preserving system membership order on ties.
pub struct StaffMapping {
    staffs: Vec<Staff>,
    areas: Vec<crate::system_area_geometry::Band>,
    systems: BTreeMap<u64, Vec<u64>>,
}
impl StaffMapping {
    pub fn from_grid(
        grid: &Value,
        width: usize,
        height: usize,
        interline: usize,
    ) -> Result<Self, String> {
        let layout = crate::system_area_geometry::build(grid, width, height, interline)?;
        let rows = grid["staffs"].as_array().ok_or("invalid_seed_staffs")?;
        let mut staffs = Vec::new();
        let mut spline_points = 0usize;
        for row in rows {
            let lines = row["lines"]
                .as_array()
                .filter(|l| l.len() == 5)
                .ok_or("unsupported_seed_staff_lines")?;
            for i in [0, 4] {
                spline_points += lines[i]["points"]
                    .as_array()
                    .filter(|p| p.len() <= 4096)
                    .ok_or("invalid_seed_staff_points")?
                    .len();
                if spline_points > 65536 {
                    return Err("unsupported_seed_spline_budget".into());
                }
            }
            let staff = Staff {
                id: row["id"].as_u64().ok_or("invalid_seed_staff_id")?,
                left: finite(&row["left"])?,
                right: finite(&row["right"])?,
                top: curve(&lines[0])?,
                bottom: curve(&lines[4])?,
            };
            if staff.left >= staff.right || staff.top.points[0][1] >= staff.bottom.points[0][1] {
                return Err("invalid_seed_staff_span".into());
            }
            staffs.push(staff);
        }
        let mut systems = BTreeMap::new();
        for (system, neighbors) in grid["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(&layout.system_neighbors)
        {
            let ends = crate::beam_dispatch::area_ends(&system["area"]["x_range"], width)?;
            if json!(ends) != neighbors["x_range"] {
                return Err("inconsistent_seed_system_area_ends".into());
            }
            let id = system["id"].as_u64().ok_or("invalid_seed_system_id")?;
            let members = system["staff_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().ok_or("invalid_seed_member_id"))
                .collect::<Result<Vec<_>, _>>()?;
            systems.insert(id, members);
        }
        Ok(Self {
            staffs,
            areas: layout.staff_areas,
            systems,
        })
    }
    pub fn closest(&self, system_id: u64, point: [f64; 2]) -> Result<Option<u64>, String> {
        if point.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.) {
            return Err("invalid_seed_coordinate".into());
        }
        let members = self.systems.get(&system_id).ok_or("missing_seed_system")?;
        let mut best = None;
        let mut best_distance = i64::MAX;
        for id in members {
            let i = self
                .staffs
                .iter()
                .position(|s| s.id == *id)
                .ok_or("missing_seed_staff")?;
            if !self.areas[i].contains(point)? {
                continue;
            }
            let staff = &self.staffs[i];
            // Original Staff.distanceTo truncates its algebraic double to int.
            let distance = (staff.top.y(point[0]) - point[1])
                .max(point[1] - staff.bottom.y(point[0])) as i32 as i64;
            if distance < best_distance {
                best_distance = distance;
                best = Some(*id);
            }
        }
        Ok(best)
    }
}
fn array<const N: usize>(v: &Value) -> Result<[f64; N], String> {
    let a = v
        .as_array()
        .filter(|a| a.len() == N)
        .ok_or("invalid_seed_array")?;
    let mut out = [0.; N];
    for (i, v) in a.iter().enumerate() {
        out[i] = finite(v)?;
    }
    Ok(out)
}

/// Fresh default-profile (1) path. No runtime Java or oracle-result lookup.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    headers: &Value,
    scale: &Scale,
    global_slope: f64,
) -> Result<Value, String> {
    if headers["headers_complete"] != true {
        return Err("incomplete_seed_headers_dependency".into());
    }
    let rows = grid["staffs"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("invalid_seed_staffs")?;
    let hs = headers["staffs"]
        .as_array()
        .filter(|s| s.len() == rows.len())
        .ok_or("invalid_seed_headers")?;
    let mut stops = BTreeMap::new();
    for h in hs {
        let id = h["staff_id"].as_u64().ok_or("invalid_seed_staff_id")?;
        if stops.insert(id, finite(&h["stop"])?).is_some() {
            return Err("duplicate_seed_header".into());
        }
    }
    let mut memberships = BTreeMap::new();
    let mut membership_order = Vec::new();
    let mut system_ids = BTreeSet::new();
    for system in grid["systems"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("invalid_seed_systems")?
    {
        let sid = system["id"].as_u64().ok_or("invalid_seed_system_id")?;
        if !system_ids.insert(sid) {
            return Err("duplicate_seed_system".into());
        }
        for id in system["staff_ids"]
            .as_array()
            .filter(|s| !s.is_empty() && s.len() <= 1024)
            .ok_or("invalid_seed_membership")?
        {
            let id = id.as_u64().ok_or("invalid_seed_member_id")?;
            membership_order.push(id);
            if memberships.insert(id, sid).is_some() {
                return Err("duplicate_seed_staff_membership".into());
            }
        }
    }
    if memberships.len() != rows.len() {
        return Err("incomplete_seed_staff_membership".into());
    }
    let row_order = rows
        .iter()
        .map(|r| r["id"].as_u64().ok_or("invalid_seed_staff_id"))
        .collect::<Result<Vec<_>, _>>()?;
    if membership_order != row_order {
        return Err("inconsistent_seed_staff_order".into());
    }
    for row in rows {
        let id = row["id"].as_u64().ok_or("invalid_seed_staff_id")?;
        let system_id = row["system_id"]
            .as_u64()
            .ok_or("invalid_seed_staff_system")?;
        let header = hs
            .iter()
            .find(|h| h["staff_id"] == id)
            .ok_or("missing_seed_header")?;
        if memberships.get(&id) != Some(&system_id) || header["system_id"] != system_id {
            return Err("inconsistent_seed_staff_system".into());
        }
    }
    let mapping = StaffMapping::from_grid(grid, image.width, image.height, scale.interline.main)?;
    let scaling = stem_scale::analyze(image, scale)?;
    let sticks = vertical_sticks::analyze(grid, scale.interline.main, scaling.stem_scale.max)?;
    let params = stem_checks::Parameters::new(
        scale.interline.main,
        scaling.stem_scale.max,
        global_slope,
        1,
    )?;
    let mut systems = Vec::new();
    for system in sticks["systems"].as_array().unwrap() {
        let sid = system["system_id"]
            .as_u64()
            .ok_or("invalid_seed_system_id")?;
        let mut candidates = Vec::new();
        let mut accepted = Vec::new();
        let mut seen_glyphs = BTreeSet::new();
        for raw in system["candidates"].as_array().unwrap() {
            let mut row = raw.clone();
            let center = array::<2>(&raw["center"])?;
            let staff_id = mapping.closest(sid, center)?;
            let skipped = staff_id.is_none_or(|id| center[0] < stops[&id]);
            row["closest_staff_id"] = json!(staff_id);
            row["skipped_header_or_staff"] = json!(skipped);
            if !skipped {
                let b = array::<4>(&raw["bounds"])?;
                let candidate = stem_checks::Candidate {
                    bounds: b.map(|v| v as i64),
                    start: array(&raw["start"])?,
                    stop: array(&raw["stop"])?,
                    line: array(&raw["line"])?,
                    mean_distance: finite(&raw["mean_distance"])?,
                };
                let checked = stem_checks::check(image, &candidate, &params)?;
                if checked.accepted && seen_glyphs.insert(raw["glyph"].to_string()) {
                    accepted.push(raw["glyph"].clone());
                }
                row["check"] = serde_json::to_value(checked).map_err(|e| e.to_string())?;
            }
            candidates.push(row);
        }
        systems.push(json!({"system_id":system["system_id"],"profile":1,"left":system["left"],"right":system["right"],"candidates":candidates,"accepted_seeds":accepted}));
    }
    Ok(
        json!({"stage":"STEM_SEEDS.native","stem_scaling":scaling,"systems":systems,"stem_seeds_complete":true,"note_recognition_complete":false,"supported_scope":"fresh profile1, five-line native GRID with original neighbor-chain clipped staff areas; current 4M-pixel/resource limits","pending":["BEAMS","LEDGERS","automatic HEADS and stem attachment","rhythm and MusicXML"]}),
    )
}
