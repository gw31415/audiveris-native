// SPDX-License-Identifier: AGPL-3.0-or-later
// Default single-system/five-line staff; empty or supplied straight ledger curves.
use crate::{
    Run,
    beam_candidates::CandidateInter,
    curves::Curve,
    multiple_rest_admission::StaffPosition,
    sections::{Orientation, Section, SectionId},
    serif_peaks::StaffGeometry,
};
use serde_json::Value;
pub struct Mapping {
    pub id: u64,
    pub interline: usize,
    pub rows: Vec<[i32; 2]>,
    pub middle: Vec<i32>,
    pub thickness: Vec<f64>,
    pub abscissas: [i32; 2],
    pub sections: Vec<Section>,
    lines: Vec<Curve>,
    ledgers: std::collections::BTreeMap<i32, Curve>,
}
impl Mapping {
    pub fn from_grid(
        grid: &Value,
        width: usize,
        height: usize,
        system: u64,
    ) -> Result<Self, String> {
        let staffs = grid["staffs"]
            .as_array()
            .filter(|v| v.len() == 1)
            .ok_or("unsupported_rest_multi_staff_mapping")?;
        let systems = grid["systems"]
            .as_array()
            .filter(|v| v.len() == 1)
            .ok_or("unsupported_rest_multi_system_mapping")?;
        let staff = &staffs[0];
        let area = &systems[0]["area"];
        if systems[0]["id"] != system
            || staff["system_id"] != system
            || staff["tablature"] == true
            || area["x_range"] != serde_json::json!([0, width])
            || !area["north_neighbor_last_line"].is_null()
            || !area["south_neighbor_first_line"].is_null()
        {
            return Err("unsupported_rest_staff_area_mapping".into());
        }
        let mut ledgers = std::collections::BTreeMap::new();
        if !staff["ledger_lines"].is_null() {
            let raw_ledgers = staff["ledger_lines"]
                .as_array()
                .filter(|v| v.len() <= 1024)
                .ok_or("invalid_rest_ledger_map")?;
            for ledger in raw_ledgers {
                let index = ledger["index"]
                    .as_i64()
                    .and_then(|v| i32::try_from(v).ok())
                    .filter(|v| *v != 0 && v.unsigned_abs() <= 1024)
                    .ok_or("invalid_rest_ledger_index")?;
                let points = ledger["points"]
                    .as_array()
                    .filter(|v| v.len() == 2)
                    .ok_or("unsupported_rest_non_linear_ledger_mapping")?
                    .iter()
                    .map(|p| {
                        Ok([
                            p[0].as_f64().ok_or("invalid_rest_ledger_point")?,
                            p[1].as_f64().ok_or("invalid_rest_ledger_point")?,
                        ])
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let curve = Curve::interpolate(points).ok_or("invalid_rest_ledger_curve")?;
                if ledgers.insert(index, curve).is_some() {
                    return Err("duplicate_rest_ledger_index".into());
                }
            }
        }
        let raw = staff["lines"]
            .as_array()
            .filter(|v| v.len() == 5)
            .ok_or("unsupported_rest_staff_lines")?;
        if width == 0
            || height == 0
            || width
                .checked_mul(height)
                .is_none_or(|n| n > crate::MAX_PIXELS)
        {
            return Err("invalid_rest_mapping_dimensions".into());
        }
        let mut lines = Vec::new();
        let mut thickness = Vec::new();
        for l in raw {
            let points = l["points"]
                .as_array()
                .ok_or("invalid_rest_staff_curve")?
                .iter()
                .map(|p| {
                    Ok([
                        p[0].as_f64().ok_or("invalid_rest_staff_point")?,
                        p[1].as_f64().ok_or("invalid_rest_staff_point")?,
                    ])
                })
                .collect::<Result<Vec<_>, String>>()?;
            lines.push(Curve::interpolate(points).ok_or("invalid_rest_staff_curve")?);
            thickness.push(
                l["thickness"]
                    .as_f64()
                    .filter(|n| n.is_finite() && *n > 0.)
                    .ok_or("invalid_rest_staff_thickness")?,
            );
        }
        let rows = (0..width)
            .map(|x| {
                [
                    lines[0].y(x as f64).round_ties_even() as i32,
                    lines[4].y(x as f64).round_ties_even() as i32,
                ]
            })
            .collect::<Vec<_>>();
        let middle = (0..width)
            .map(|x| lines[2].y(x as f64).round_ties_even() as i32)
            .collect::<Vec<_>>();
        if rows
            .iter()
            .any(|r| r[0] < 0 || r[1] < r[0] || r[1] >= height as i32)
        {
            return Err("unsupported_rest_staff_edge_mapping".into());
        }
        let mut sections = Vec::new();
        for (field, orientation) in [
            ("vlag", Orientation::Vertical),
            ("hlag", Orientation::Horizontal),
        ] {
            for s in grid[field].as_array().ok_or("missing_rest_native_lag")? {
                let runs = s["runs"]
                    .as_array()
                    .ok_or("invalid_rest_native_section")?
                    .iter()
                    .map(|r| {
                        Ok(Run {
                            start: r[0]
                                .as_u64()
                                .and_then(|v| usize::try_from(v).ok())
                                .ok_or("invalid_rest_native_run")?,
                            length: r[1]
                                .as_u64()
                                .and_then(|v| usize::try_from(v).ok())
                                .ok_or("invalid_rest_native_run")?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                if runs.is_empty() {
                    return Err("empty_rest_native_section".into());
                }
                let weight = runs
                    .iter()
                    .try_fold(0usize, |n, r| n.checked_add(r.length))
                    .ok_or("rest_native_section_budget")?;
                sections.push(Section {
                    id: SectionId(
                        s["id"]
                            .as_u64()
                            .and_then(|v| u32::try_from(v).ok())
                            .ok_or("invalid_rest_native_section_id")?,
                    ),
                    orientation,
                    first_pos: s["first_pos"]
                        .as_u64()
                        .and_then(|v| usize::try_from(v).ok())
                        .ok_or("invalid_rest_native_section_position")?,
                    runs,
                    weight,
                });
            }
        }
        Ok(Self {
            id: staff["id"]
                .as_u64()
                .filter(|v| *v > 0)
                .ok_or("invalid_rest_native_staff_id")?,
            interline: staff["specific_interline"]
                .as_u64()
                .and_then(|v| usize::try_from(v).ok())
                .filter(|v| *v > 0)
                .ok_or("invalid_rest_native_staff_interline")?,
            rows,
            middle,
            thickness,
            abscissas: [
                staff["left"]
                    .as_f64()
                    .ok_or("invalid_rest_native_abscissa")?
                    .round_ties_even() as i32,
                staff["right"]
                    .as_f64()
                    .ok_or("invalid_rest_native_abscissa")?
                    .round_ties_even() as i32,
            ],
            sections,
            lines,
            ledgers,
        })
    }
    pub fn geometry(&self, fore: usize) -> StaffGeometry<'_> {
        StaffGeometry {
            rows: &self.rows,
            middle: &self.middle,
            line_thickness: &self.thickness,
            abscissas: self.abscissas,
            interline: self.interline,
            fore,
        }
    }
    /// Original Staff.pitchPositionOf: traverse consecutive signed ledger keys,
    /// interpolate the enclosing pair, or fall back from the last visited line.
    pub fn pitch_at(&self, x: f64, y: f64) -> Result<f64, String> {
        if !x.is_finite() || !y.is_finite() {
            return Err("invalid_rest_pitch_point".into());
        }
        let top = self.lines[0].y(x);
        let bottom = self.lines[4].y(x);
        if !top.is_finite() || !bottom.is_finite() || bottom <= top {
            return Err("invalid_rest_pitch_staff_span".into());
        }
        Ok(if y >= top && y <= bottom {
            4. * (2. * y - bottom - top) / (bottom - top)
        } else {
            let (direction, mut edge) = if y < top { (-1, top) } else { (1, bottom) };
            let mut pitch = f64::from(4 * direction);
            let mut index = direction;
            while let Some(ledger) = self.ledgers.get(&index) {
                let ordinate = ledger.y(x);
                if !ordinate.is_finite() || f64::from(direction) * (ordinate - edge) <= 0. {
                    return Err("unsupported_rest_ledger_order".into());
                }
                if f64::from(direction) * (ordinate - y) >= 0. {
                    return Ok(pitch + f64::from(2 * direction) * (y - edge) / (ordinate - edge));
                }
                edge = ordinate;
                pitch += f64::from(2 * direction);
                index += direction;
            }
            pitch + 2. * (y - edge) / self.interline as f64
        })
    }
    pub fn position(&self, beam: &CandidateInter) -> Result<Option<StaffPosition>, String> {
        let b = beam.bounds;
        let x = f64::from(b[0] + b[2] / 2);
        let y = f64::from(b[1] + b[3] / 2);
        if y < self.lines[0].y(x) {
            return Ok(None);
        }
        let mut pitches = [0.; 2];
        for (i, (x, y)) in [
            (beam.item.median[0], beam.item.median[1]),
            (beam.item.median[2], beam.item.median[3]),
        ]
        .into_iter()
        .enumerate()
        {
            pitches[i] = self.pitch_at(x, y)?;
        }
        Ok(Some(StaffPosition {
            id: self.id,
            tablature: false,
            endpoint_pitches: pitches,
            mid_line_thickness: self.thickness[2],
        }))
    }
}

/// Original StaffManager.computeStaffArea/getClosestStaff and
/// SystemInfo.getStaffAtOrAbove for one full-width vertical staff column.
pub struct SystemMapping {
    pub staffs: Vec<Mapping>,
    pub sections: Vec<Section>,
    areas: Vec<RestArea>,
}
enum RestArea {
    Supplied(audiveris_area::Area),
    Native(crate::system_area_geometry::Band),
}
impl RestArea {
    fn contains(&self, p: [f64; 2]) -> Result<bool, String> {
        match self {
            Self::Supplied(a) => a.contains(p),
            Self::Native(a) => a.contains(p),
        }
    }
}
impl SystemMapping {
    pub fn from_grid(
        grid: &Value,
        width: usize,
        height: usize,
        system: u64,
    ) -> Result<Self, String> {
        if grid["schema"] == "audiveris-native.grid-ir.v1" {
            return Self::from_native_grid(grid, width, height, system);
        }
        if grid.get("schema").is_some() {
            return Err("unsupported_rest_grid_schema".into());
        }
        let systems = grid["systems"]
            .as_array()
            .filter(|s| !s.is_empty() && s.len() <= 32)
            .ok_or("unsupported_rest_system_count")?;
        let target = systems
            .iter()
            .position(|s| s["id"] == system)
            .ok_or("invalid_rest_mapping_system")?;
        let raw = grid["staffs"]
            .as_array()
            .filter(|s| !s.is_empty() && s.len() <= 32)
            .ok_or("unsupported_rest_staff_count")?;
        if width
            .checked_mul(raw.len())
            .is_none_or(|n| n > crate::MAX_PIXELS)
        {
            return Err("unsupported_rest_staff_mapping_budget".into());
        }
        let margin = if raw.len() == 1 {
            0.
        } else {
            systems[target]["area"]["vertical_margin"]
                .as_f64()
                .filter(|m| m.is_finite() && *m >= 0.)
                .ok_or("invalid_rest_staff_area_margin")?
        };
        let mut order = Vec::new();
        let mut system_ids = std::collections::BTreeSet::new();
        for sys in systems {
            if !system_ids.insert(sys["id"].as_u64().ok_or("invalid_rest_mapping_system")?) {
                return Err("duplicate_rest_mapping_system".into());
            }
            crate::beam_dispatch::area_ends(&sys["area"]["x_range"], width)?;
            if raw.len() > 1 && sys["area"]["vertical_margin"].as_f64() != Some(margin) {
                return Err("unsupported_rest_system_area_margin".into());
            }
            if let Some(ids) = sys["staff_ids"].as_array() {
                for id in ids {
                    order.push((id.clone(), sys["id"].clone()));
                }
            } else if systems.len() == 1 && raw.len() == 1 {
                order.push((raw[0]["id"].clone(), sys["id"].clone()));
            } else {
                return Err("unsupported_rest_staff_order".into());
            }
        }
        if order
            != raw
                .iter()
                .map(|s| (s["id"].clone(), s["system_id"].clone()))
                .collect::<Vec<_>>()
        {
            return Err("unsupported_rest_staff_order".into());
        }
        let mut single = grid.clone();
        let mut global = Vec::new();
        let mut sections = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for row in raw {
            let sid = row["system_id"]
                .as_u64()
                .ok_or("invalid_rest_mapping_system")?;
            single["systems"] = serde_json::json!([{"id":sid,"area":{"x_range":[0,width]}}]);
            single["staffs"] = serde_json::json!([row]);
            let mut mapping = Mapping::from_grid(&single, width, height, sid)?;
            if !ids.insert(mapping.id) {
                return Err("duplicate_rest_mapping_staff".into());
            }
            if global.is_empty() {
                sections = std::mem::take(&mut mapping.sections);
                single["vlag"] = serde_json::json!([]);
                single["hlag"] = serde_json::json!([]);
            }
            global.push(mapping);
        }
        let common_left = global.iter().map(|s| s.abscissas[0]).max().unwrap();
        let common_right = global.iter().map(|s| s.abscissas[1]).min().unwrap();
        if common_left >= common_right {
            return Err("unsupported_rest_staff_column_mapping".into());
        }
        for pair in global.windows(2) {
            for x in 0..width {
                if pair[0].lines[4].y(x as f64) >= pair[1].lines[0].y(x as f64) {
                    return Err("unsupported_rest_overlapping_staff_bands".into());
                }
            }
        }
        let owned = raw
            .iter()
            .enumerate()
            .filter(|(_, s)| s["system_id"] == system)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        if owned.is_empty() {
            return Err("missing_rest_system_staff".into());
        }
        let first = owned[0];
        let last = *owned.last().unwrap();
        let ends = crate::beam_dispatch::area_ends(&systems[target]["area"]["x_range"], width)?;
        let system_area = crate::beam_dispatch::clipped_system_area(
            width,
            height,
            first.checked_sub(1).map(|i| &global[i].lines[4]),
            global.get(last + 1).map(|s| &s.lines[0]),
            margin,
            ends,
        )?;
        let [low, high] = system_area
            .path_y_bounds()
            .ok_or("empty_rest_system_area")?;
        let low = low.floor();
        let high = high.ceil();
        if low < 0. || high > height as f64 || high <= low {
            return Err("unsupported_rest_system_area_bounds".into());
        }
        let bounds = crate::sections::Bounds {
            x: ends[0],
            y: low as usize,
            width: ends[1] - ends[0],
            height: (high - low) as usize,
        };
        sections.retain(|s| crate::bar_sticks::intersects(s.bounds(), bounds));
        let areas = owned
            .iter()
            .map(|&i| {
                crate::beam_dispatch::clipped_system_area(
                    width,
                    height,
                    i.checked_sub(1).map(|n| &global[n].lines[4]),
                    global.get(i + 1).map(|s| &s.lines[0]),
                    margin,
                    ends,
                )
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut staffs = Vec::new();
        for (i, mapping) in global.into_iter().enumerate() {
            if owned.contains(&i) {
                staffs.push(mapping);
            }
        }
        Ok(Self {
            staffs,
            sections,
            areas: areas.into_iter().map(RestArea::Supplied).collect(),
        })
    }
    /// Native page adapter with the actual sheet main scale. Staff selection
    /// uses page-scale areas; pitch/serif geometry retains each staff scale.
    /// The legacy from_grid contract still refuses unknown mixed-scale pages.
    pub fn from_grid_with_main(
        grid: &Value,
        width: usize,
        height: usize,
        system: u64,
        main: usize,
    ) -> Result<Self, String> {
        if !(1..=128).contains(&main) {
            return Err("invalid_rest_native_interline".into());
        }
        if grid["schema"] != "audiveris-native.grid-ir.v1" {
            return Self::from_grid(grid, width, height, system);
        }
        let staffs = grid["staffs"]
            .as_array()
            .filter(|s| !s.is_empty())
            .ok_or("unsupported_rest_staff_count")?;
        for s in staffs {
            let specific = s["specific_interline"]
                .as_u64()
                .filter(|n| (1..=128).contains(n))
                .ok_or("invalid_rest_native_interline")?;
            if specific != main as u64 && !(specific < main as u64 && s["small"] == true) {
                return Err("unsupported_rest_native_mixed_interline".into());
            }
        }
        Self::from_native_grid_at_scale(grid, width, height, system, main)
    }
    fn from_native_grid(
        grid: &Value,
        width: usize,
        height: usize,
        system: u64,
    ) -> Result<Self, String> {
        // Legacy uniform-scale contract infers main from the first staff.
        let main = grid["staffs"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s["specific_interline"].as_u64())
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("invalid_rest_native_interline")?;
        if grid["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["specific_interline"].as_u64() != Some(main as u64))
        {
            return Err("unsupported_rest_native_mixed_interline".into());
        }
        Self::from_native_grid_at_scale(grid, width, height, system, main)
    }
    fn from_native_grid_at_scale(
        grid: &Value,
        width: usize,
        height: usize,
        system: u64,
        main: usize,
    ) -> Result<Self, String> {
        let interline = grid["systems"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s["area"]["vertical_margin"].as_f64());
        if interline != Some((main as f64 * 0.9).round_ties_even()) {
            return Err("inconsistent_rest_native_margin".into());
        }
        // Validate the native metadata against the same original area plan used by dispatch.
        crate::beam_dispatch::Geometry::from_grid(width, height, grid, main)?;
        let layout = crate::system_area_geometry::build(grid, width, height, main)?;
        let systems = grid["systems"].as_array().unwrap();
        let target = systems
            .iter()
            .position(|s| s["id"] == system)
            .ok_or("invalid_rest_mapping_system")?;
        let [x, y, w, h] = layout.system_areas[target].bounds()?;
        if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > width as i32 || y + h > height as i32 {
            return Err("unsupported_rest_system_area_bounds".into());
        }
        let bounds = crate::sections::Bounds {
            x: x as usize,
            y: y as usize,
            width: w as usize,
            height: h as usize,
        };
        let mut single = grid.clone();
        let mut staffs = Vec::new();
        let mut areas = Vec::new();
        let mut sections = Vec::new();
        let rows = grid["staffs"].as_array().unwrap();
        if width
            .checked_mul(rows.len())
            .is_none_or(|n| n > crate::MAX_PIXELS)
        {
            return Err("unsupported_rest_staff_mapping_budget".into());
        }
        for (row, area) in rows.iter().zip(layout.staff_areas) {
            if row["system_id"] != system {
                continue;
            }
            single["systems"] = serde_json::json!([{"id":system,"area":{"x_range":[0,width]}}]);
            single["staffs"] = serde_json::json!([row]);
            let mut mapping = Mapping::from_grid(&single, width, height, system)?;
            if staffs.is_empty() {
                sections = std::mem::take(&mut mapping.sections);
                single["vlag"] = serde_json::json!([]);
                single["hlag"] = serde_json::json!([]);
            }
            staffs.push(mapping);
            areas.push(RestArea::Native(area));
        }
        if staffs.is_empty() {
            return Err("missing_rest_system_staff".into());
        }
        sections.retain(|s| crate::bar_sticks::intersects(s.bounds(), bounds));
        Ok(Self {
            staffs,
            areas,
            sections,
        })
    }
    pub fn closest_staff(&self, x: f64, y: f64) -> Result<Option<usize>, String> {
        if !x.is_finite() || !y.is_finite() {
            return Err("invalid_rest_staff_point".into());
        }
        let mut closest = None;
        let mut distance = i32::MAX;
        for (index, (staff, area)) in self.staffs.iter().zip(&self.areas).enumerate() {
            if area.contains([x, y])? {
                let d = (staff.lines[0].y(x) - y).max(y - staff.lines[4].y(x)) as i32;
                if closest.is_none() || d < distance {
                    closest = Some(index);
                    distance = d;
                }
            }
        }
        Ok(closest)
    }
    pub fn staff_at_or_above(&self, x: f64, y: f64) -> Result<Option<usize>, String> {
        Ok(self.closest_staff(x, y)?.and_then(|i| {
            if self.staffs[i].lines[0].y(x) <= y {
                Some(i)
            } else {
                i.checked_sub(1)
            }
        }))
    }
    pub fn position(&self, beam: &CandidateInter) -> Result<Option<StaffPosition>, String> {
        let b = beam.bounds;
        let x = f64::from(b[0] + b[2] / 2);
        let y = f64::from(b[1] + b[3] / 2);
        let Some(index) = self.staff_at_or_above(x, y)? else {
            return Ok(None);
        };
        let staff = &self.staffs[index];
        Ok(Some(StaffPosition {
            id: staff.id,
            tablature: false,
            endpoint_pitches: [
                staff.pitch_at(beam.item.median[0], beam.item.median[1])?,
                staff.pitch_at(beam.item.median[2], beam.item.median[3])?,
            ],
            mid_line_thickness: staff.thickness[2],
        }))
    }
}
