// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native GlyphIndex content identity, SpotsBuilder dispatch and SystemInfo group
// lifecycle at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Native phase identities are not original Java numeric IDs. Registered values
// remain strongly owned through the native BEAMS phase; no JVM GC is simulated.
use crate::{MAX_PIXELS, curves::Curve};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_REGISTERED_GLYPHS: usize = 100_000;
pub const MAX_REGISTERED_RUNS: usize = 1_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Group {
    BeamSpot,
    BlackHeadSpot,
    BlackStackSpot,
    VerticalSeed,
    Symbol,
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub glyph: GlyphPixels,
    pub groups: BTreeSet<Group>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    left: i32,
    top: i32,
    width: usize,
    height: usize,
    vertical: bool,
    runs: Vec<[usize; 3]>,
}
fn key(g: &GlyphPixels) -> Result<Key, String> {
    if g.left.unsigned_abs() as usize > MAX_PIXELS
        || g.top.unsigned_abs() as usize > MAX_PIXELS
        || g.width == 0
        || g.height == 0
        || g.width.checked_mul(g.height).is_none_or(|n| n > MAX_PIXELS)
        || g.runs.is_empty()
        || g.runs.len() > MAX_REGISTERED_RUNS
    {
        return Err("invalid_beam_registry_glyph".into());
    }
    let vertical = g.orientation == Orientation::Vertical;
    let (positions, length) = if vertical {
        (g.width, g.height)
    } else {
        (g.height, g.width)
    };
    let mut last = None;
    for r in &g.runs {
        let stop = r
            .start
            .checked_add(r.length)
            .filter(|v| *v <= length)
            .ok_or("invalid_beam_registry_run")?;
        if r.length == 0
            || r.position >= positions
            || last.is_some_and(|(p, end)| r.position < p || (r.position == p && r.start <= end))
        {
            return Err("invalid_beam_registry_run_order".into());
        }
        last = Some((r.position, stop));
    }
    Ok(Key {
        left: g.left,
        top: g.top,
        width: g.width,
        height: g.height,
        vertical,
        runs: g
            .runs
            .iter()
            .map(|r| [r.position, r.start, r.length])
            .collect(),
    })
}
/// Original pixel barycenter, rounded by java.awt.Point assignment with Math.rint.
pub fn centroid(g: &GlyphPixels) -> Result<[i32; 2], String> {
    key(g)?;
    let (mut weight, mut sx, mut sy) = (0i64, 0i64, 0i64);
    for r in &g.runs {
        let n = r.length as i64;
        let along = n * (2 * r.start as i64 + n - 1) / 2;
        let across = n * r.position as i64;
        weight += n;
        if g.orientation == Orientation::Vertical {
            sx += across;
            sy += along;
        } else {
            sx += along;
            sy += across;
        }
    }
    Ok([
        (g.left as f64 + sx as f64 / weight as f64).round_ties_even() as i32,
        (g.top as f64 + sy as f64 / weight as f64).round_ties_even() as i32,
    ])
}
#[derive(Clone, Default, Debug)]
pub struct Registry {
    entries: Vec<Entry>,
    keys: BTreeMap<Key, usize>,
    runs: usize,
}
impl Registry {
    /// Original SymbolsFilter can add SYMBOL to an already registered glyph.
    /// Keep existing physical glyph/run allocations; only this group may grow.
    pub(crate) fn commit_symbol_extension(&mut self, mut next: Self) -> Result<(), String> {
        if next.entries.len() < self.entries.len()
            || self.entries.iter().zip(&next.entries).any(|(a, b)| {
                a.glyph != b.glyph
                    || !a.groups.is_subset(&b.groups)
                    || b.groups.difference(&a.groups).any(|g| *g != Group::Symbol)
            })
        {
            return Err("invalid_symbol_registry_transaction".into());
        }
        for (a, b) in self.entries.iter_mut().zip(&next.entries) {
            a.groups = b.groups.clone();
        }
        let additions = next.entries.split_off(self.entries.len());
        self.entries.extend(additions);
        self.keys = next.keys;
        self.runs = next.runs;
        Ok(())
    }
    /// Commit a validated append-only transaction without replacing existing
    /// physical glyphs or their run allocations. Refusal remains atomic.
    pub(crate) fn commit_extension(&mut self, mut next: Self) -> Result<(), String> {
        if next.entries.len() < self.entries.len()
            || self
                .entries
                .iter()
                .zip(&next.entries)
                .any(|(a, b)| a.glyph != b.glyph || a.groups != b.groups)
        {
            return Err("non_append_only_physical_registry_transaction".into());
        }
        let additions = next.entries.split_off(self.entries.len());
        self.entries.extend(additions);
        self.keys = next.keys;
        self.runs = next.runs;
        Ok(())
    }
    pub fn register(&mut self, g: &GlyphPixels) -> Result<usize, String> {
        let key = key(g)?;
        if let Some(id) = self.keys.get(&key) {
            return Ok(*id);
        }
        if self.entries.len() >= MAX_REGISTERED_GLYPHS
            || self
                .runs
                .checked_add(g.runs.len())
                .is_none_or(|v| v > MAX_REGISTERED_RUNS)
        {
            return Err("unsupported_beam_registry_budget".into());
        }
        let id = self.entries.len() + 1;
        self.runs += g.runs.len();
        self.entries.push(Entry {
            glyph: g.clone(),
            groups: BTreeSet::new(),
        });
        self.keys.insert(key, id);
        Ok(id)
    }
    pub fn add_group(&mut self, id: usize, group: Group) -> Result<(), String> {
        self.entries
            .get_mut(id.checked_sub(1).ok_or("invalid_beam_glyph_id")?)
            .ok_or("invalid_beam_glyph_id")?
            .groups
            .insert(group);
        Ok(())
    }
    pub fn get(&self, id: usize) -> Result<&Entry, String> {
        self.entries
            .get(id.checked_sub(1).ok_or("invalid_beam_glyph_id")?)
            .ok_or("invalid_beam_glyph_id".into())
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn snapshot(&self) -> Value {
        json!(
            self.entries
                .iter()
                .enumerate()
                .map(|(i, e)| json!({"id":i+1,"glyph":glyph_json(&e.glyph),"groups":e.groups}))
                .collect::<Vec<_>>()
        )
    }
}
pub fn glyph_json(g: &GlyphPixels) -> Value {
    json!({"bounds":[g.left as i64,g.top as i64,g.width as i64,g.height as i64],"orientation":if g.orientation==Orientation::Vertical{"VERTICAL"}else{"HORIZONTAL"},"runs":g.runs.iter().map(|r|[r.position,r.start,r.length]).collect::<Vec<_>>()})
}
/// Decode an existing native canonical glyph, retaining run orientation and
/// bounds exactly. No theoretical/inter bounds are substituted for glyph pixels.
pub fn glyph_from_json(v: &Value) -> Result<GlyphPixels, String> {
    let bounds = v["bounds"]
        .as_array()
        .filter(|v| v.len() == 4)
        .ok_or("invalid_beam_canonical_bounds")?;
    let left = integer(&bounds[0])?;
    let top = integer(&bounds[1])?;
    let width = integer(&bounds[2])?;
    let height = integer(&bounds[3])?;
    if width <= 0 || height <= 0 {
        return Err("invalid_beam_canonical_dimensions".into());
    }
    let orientation = match v["orientation"].as_str() {
        Some("VERTICAL") => Orientation::Vertical,
        Some("HORIZONTAL") => Orientation::Horizontal,
        _ => return Err("invalid_beam_canonical_orientation".into()),
    };
    let raw = v["runs"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= MAX_REGISTERED_RUNS)
        .ok_or("invalid_beam_canonical_runs")?;
    let mut runs = Vec::with_capacity(raw.len());
    for r in raw {
        let r = r
            .as_array()
            .filter(|r| r.len() == 3)
            .ok_or("invalid_beam_canonical_run")?;
        let mut n = [0usize; 3];
        for (i, v) in r.iter().enumerate() {
            n[i] = v
                .as_u64()
                .filter(|n| *n <= MAX_PIXELS as u64)
                .ok_or("invalid_beam_canonical_run_value")? as usize;
        }
        runs.push(Run {
            position: n[0],
            start: n[1],
            length: n[2],
        });
    }
    let glyph = GlyphPixels {
        left,
        top,
        width: width as usize,
        height: height as usize,
        orientation,
        runs,
    };
    key(&glyph)?;
    Ok(glyph)
}
fn number(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|n| n.is_finite() && n.abs() <= MAX_PIXELS as f64)
        .ok_or("invalid_beam_dispatch_coordinate".into())
}
fn integer(v: &Value) -> Result<i32, String> {
    v.as_i64()
        .filter(|v| v.unsigned_abs() <= MAX_PIXELS as u64)
        .map(|v| v as i32)
        .ok_or("invalid_beam_dispatch_integer".into())
}
pub(crate) fn curve(row: &Value, budget: &mut usize) -> Result<Curve, String> {
    let raw = row["points"]
        .as_array()
        .filter(|p| (2..=4096).contains(&p.len()))
        .ok_or("invalid_beam_dispatch_curve")?;
    *budget += raw.len();
    if *budget > 65536 {
        return Err("unsupported_beam_dispatch_curve_budget".into());
    }
    let points = raw
        .iter()
        .map(|p| {
            let p = p
                .as_array()
                .filter(|p| p.len() == 2)
                .ok_or("invalid_beam_dispatch_point")?;
            Ok([number(&p[0])?, number(&p[1])?])
        })
        .collect::<Result<Vec<_>, String>>()?;
    Curve::interpolate(points).ok_or("invalid_beam_dispatch_curve".into())
}
pub(crate) fn system_area(
    width: usize,
    height: usize,
    north: Option<&Curve>,
    south: Option<&Curve>,
    margin: f64,
) -> Result<audiveris_area::Area, String> {
    fn boundary(width: usize, curve: Option<&Curve>, shift: f64, flat: f64) -> Vec<Vec<[f64; 2]>> {
        let Some(curve) = curve else {
            return vec![vec![[0., flat], [width as f64, flat]]];
        };
        let first = curve.points[0];
        let last = *curve.points.last().unwrap();
        let mut segments = vec![vec![[0., first[1] + shift], [first[0], first[1] + shift]]];
        segments.extend(
            curve
                .path_segments()
                .iter()
                .map(|s| s.iter().map(|p| [p[0], p[1] + shift]).collect::<Vec<_>>()),
        );
        segments.push(vec![
            [last[0], last[1] + shift],
            [width as f64, last[1] + shift],
        ]);
        segments
    }
    let mut segments = boundary(width, north, margin, 0.);
    segments.extend(
        boundary(width, south, -margin, height as f64)
            .into_iter()
            .rev()
            .map(|mut segment| {
                segment.reverse();
                segment
            }),
    );
    audiveris_area::Area::from_closed_segments(&segments)
}
pub(crate) fn area_ends(value: &Value, width: usize) -> Result<[usize; 2], String> {
    let v = value
        .as_array()
        .filter(|v| v.len() == 2)
        .ok_or("invalid_system_area_ends")?;
    let l = v[0]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or("invalid_system_area_ends")?;
    let r = v[1]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or("invalid_system_area_ends")?;
    if l >= r || r > width {
        return Err("invalid_system_area_ends".into());
    }
    Ok([l, r])
}
/// Original intersection with the supplied horizontal area-end rectangle.
/// Full-width curved bands retain their existing path; clipped straight bands
/// reconstruct the exact line-only intersection polygon.
pub(crate) fn clipped_system_area(
    width: usize,
    height: usize,
    north: Option<&Curve>,
    south: Option<&Curve>,
    margin: f64,
    ends: [usize; 2],
) -> Result<audiveris_area::Area, String> {
    if ends == [0, width] {
        return system_area(width, height, north, south, margin);
    }
    if ends[0] >= ends[1] || ends[1] > width {
        return Err("invalid_system_area_ends".into());
    }
    if north.into_iter().chain(south).any(|c| c.points.len() != 2) {
        return Err("unsupported_curved_system_area_clip".into());
    }
    fn boundary(c: Option<&Curve>, shift: f64, flat: f64, ends: [usize; 2]) -> Vec<[f64; 2]> {
        let y = |x: f64| match c {
            None => flat,
            Some(c) => {
                let a = c.points[0];
                let b = *c.points.last().unwrap();
                if x < a[0] {
                    a[1] + shift
                } else if x > b[0] {
                    b[1] + shift
                } else {
                    c.y(x) + shift
                }
            }
        };
        let mut points = vec![[ends[0] as f64, y(ends[0] as f64)]];
        if let Some(c) = c {
            for p in &c.points {
                if p[0] > ends[0] as f64 && p[0] < ends[1] as f64 {
                    points.push([p[0], p[1] + shift]);
                }
            }
        }
        points.push([ends[1] as f64, y(ends[1] as f64)]);
        points
    }
    let top = boundary(north, margin, 0., ends);
    let mut bottom = boundary(south, -margin, height as f64, ends);
    bottom.reverse();
    let segments = top
        .windows(2)
        .chain(bottom.windows(2))
        .map(|p| p.to_vec())
        .collect::<Vec<_>>();
    audiveris_area::Area::from_closed_segments(&segments)
}
pub const MAX_AREA_QUERY_WORK: usize = 100_000_000;
#[derive(Debug)]
enum DispatchArea {
    Supplied(audiveris_area::Area),
    Native(crate::system_area_geometry::Band),
}
impl DispatchArea {
    fn contains(&self, p: [f64; 2]) -> Result<bool, String> {
        match self {
            Self::Supplied(a) => a.contains(p),
            Self::Native(a) => a.contains(p),
        }
    }
    fn query_work(&self) -> usize {
        match self {
            Self::Supplied(a) => a.query_work(),
            Self::Native(a) => a.query_work(),
        }
    }
}
#[derive(Debug)]
pub struct SystemArea {
    pub id: u64,
    pub left: i32,
    pub right: i32,
    area: DispatchArea,
}
#[derive(Debug)]
pub struct Geometry {
    pub systems: Vec<SystemArea>,
    width: usize,
    height: usize,
    query_work: usize,
}
impl Geometry {
    pub fn from_grid(
        width: usize,
        height: usize,
        grid: &Value,
        interline: usize,
    ) -> Result<Self, String> {
        if width == 0
            || height == 0
            || width.checked_mul(height).is_none_or(|n| n > MAX_PIXELS)
            || !(1..=128).contains(&interline)
        {
            return Err("invalid_beam_dispatch_dimensions".into());
        }
        let staffs = grid["staffs"]
            .as_array()
            .filter(|v| !v.is_empty() && v.len() <= 1024)
            .ok_or("invalid_beam_dispatch_staffs")?;
        let raw = grid["systems"]
            .as_array()
            .filter(|v| !v.is_empty() && v.len() <= 1024)
            .ok_or("invalid_beam_dispatch_systems")?;
        // Native GRID's declared stage owns discovered neighbor geometry.
        // Supplied-geometry controls can intentionally provide independent clips.
        if grid["schema"] == "audiveris-native.grid-ir.v1" {
            return Self::from_native_grid(width, height, grid, interline);
        }
        if grid.get("schema").is_some() {
            return Err("unsupported_beam_dispatch_grid_schema".into());
        }
        let mut curves = BTreeMap::new();
        let mut staff_systems = BTreeMap::new();
        let mut order = Vec::new();
        let mut budget = 0;
        for row in staffs {
            let id = row["id"].as_u64().ok_or("invalid_beam_dispatch_staff_id")?;
            let lines = row["lines"]
                .as_array()
                .filter(|v| v.len() == 5)
                .ok_or("unsupported_beam_dispatch_staff_lines")?;
            if curves
                .insert(
                    id,
                    (
                        curve(&lines[0], &mut budget)?,
                        curve(&lines[4], &mut budget)?,
                    ),
                )
                .is_some()
            {
                return Err("duplicate_beam_dispatch_staff".into());
            }
            staff_systems.insert(
                id,
                row["system_id"]
                    .as_u64()
                    .ok_or("invalid_beam_dispatch_staff_system")?,
            );
            order.push(id);
        }
        let margin = (0.9 * interline as f64).round_ties_even();
        let mut system_ids = BTreeSet::new();
        let mut members = Vec::new();
        let mut seen_members = BTreeSet::new();
        let mut systems = Vec::new();
        for (index, system) in raw.iter().enumerate() {
            let id = system["id"]
                .as_u64()
                .ok_or("invalid_beam_dispatch_system_id")?;
            if !system_ids.insert(id) {
                return Err("duplicate_beam_dispatch_system".into());
            }
            let ids = system["staff_ids"]
                .as_array()
                .filter(|v| !v.is_empty() && v.len() <= 1024)
                .ok_or("invalid_beam_dispatch_members")?;
            for member in ids {
                let sid = member.as_u64().ok_or("invalid_beam_dispatch_staff_id")?;
                if staff_systems.get(&sid) != Some(&id) || !seen_members.insert(sid) {
                    return Err("inconsistent_beam_dispatch_membership".into());
                }
                members.push(sid);
            }
            let ends = area_ends(&system["area"]["x_range"], width)?;
            if number(&system["area"]["vertical_margin"])? != margin {
                return Err("unsupported_beam_dispatch_area".into());
            }
            let coord = system["coordinates"]
                .as_array()
                .filter(|v| v.len() == 5)
                .ok_or("invalid_beam_dispatch_system_coordinates")?;
            let left = integer(&coord[0])?;
            let span = integer(&coord[2])?;
            if span <= 0 {
                return Err("invalid_beam_dispatch_system_span".into());
            }
            // Original getRight() returns left+width, deliberately inclusive here.
            let right = left
                .checked_add(span)
                .ok_or("invalid_beam_dispatch_system_span")?;
            let north_id = if index == 0 {
                Value::Null
            } else {
                raw[index - 1]["staff_ids"]
                    .as_array()
                    .and_then(|v| v.last())
                    .ok_or("invalid_beam_dispatch_neighbor")?
                    .clone()
            };
            let south_id = if index + 1 == raw.len() {
                Value::Null
            } else {
                raw[index + 1]["staff_ids"][0].clone()
            };
            if system["area"]["north_neighbor_last_line"] != north_id
                || system["area"]["south_neighbor_first_line"] != south_id
            {
                return Err("inconsistent_beam_dispatch_neighbor".into());
            }
            let north = if north_id.is_null() {
                None
            } else {
                Some(
                    curves
                        .get(&north_id.as_u64().ok_or("invalid_beam_dispatch_neighbor")?)
                        .ok_or("missing_beam_dispatch_neighbor")?
                        .1
                        .clone(),
                )
            };
            let south = if south_id.is_null() {
                None
            } else {
                Some(
                    curves
                        .get(&south_id.as_u64().ok_or("invalid_beam_dispatch_neighbor")?)
                        .ok_or("missing_beam_dispatch_neighbor")?
                        .0
                        .clone(),
                )
            };
            let area =
                clipped_system_area(width, height, north.as_ref(), south.as_ref(), margin, ends)?;
            systems.push(SystemArea {
                id,
                left,
                right,
                area: DispatchArea::Supplied(area),
            });
        }
        if order != members {
            return Err("inconsistent_beam_dispatch_staff_order".into());
        }
        let query_work = systems
            .iter()
            .try_fold(0usize, |n, s| n.checked_add(s.area.query_work()))
            .ok_or("unsupported_beam_dispatch_query_work")?;
        Ok(Self {
            systems,
            width,
            height,
            query_work,
        })
    }
    fn from_native_grid(
        width: usize,
        height: usize,
        grid: &Value,
        interline: usize,
    ) -> Result<Self, String> {
        let layout = crate::system_area_geometry::build(grid, width, height, interline)?;
        let margin = (0.9 * interline as f64).round_ties_even();
        let mut systems = Vec::new();
        for ((system, plan), area) in grid["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(layout.system_neighbors)
            .zip(layout.system_areas)
        {
            if area_ends(&system["area"]["x_range"], width)? != area_ends(&plan["x_range"], width)?
            {
                return Err("inconsistent_beam_dispatch_area_ends".into());
            }
            if number(&system["area"]["vertical_margin"])? != margin {
                return Err("unsupported_beam_dispatch_area".into());
            }
            for (field, key) in [
                ("north_neighbor_last_line", "north_staff_ids"),
                ("south_neighbor_first_line", "south_staff_ids"),
            ] {
                let ids = plan[key].as_array().unwrap();
                let expected = if ids.len() == 1 {
                    ids[0].clone()
                } else {
                    Value::Null
                };
                if system["area"].get(field) != Some(&expected)
                    || system["area"].get(key).is_some_and(|v| v != &plan[key])
                    || (ids.len() > 1 && system["area"].get(key).is_none())
                {
                    return Err("inconsistent_beam_dispatch_neighbor".into());
                }
            }
            let coord = system["coordinates"].as_array().unwrap();
            let left = integer(&coord[0])?;
            let right = left
                .checked_add(integer(&coord[2])?)
                .ok_or("invalid_beam_dispatch_system_span")?;
            systems.push(SystemArea {
                id: system["id"].as_u64().unwrap(),
                left,
                right,
                area: DispatchArea::Native(area),
            });
        }
        let query_work = systems
            .iter()
            .try_fold(0usize, |n, s| n.checked_add(s.area.query_work()))
            .ok_or("unsupported_beam_dispatch_query_work")?;
        Ok(Self {
            systems,
            width,
            height,
            query_work,
        })
    }
    pub fn validate_point_work(&self, count: usize) -> Result<(), String> {
        if count
            .checked_mul(self.query_work)
            .is_none_or(|n| n > MAX_AREA_QUERY_WORK)
        {
            return Err("unsupported_beam_dispatch_query_work".into());
        }
        Ok(())
    }
    /// Original BeamsBuilder.isSideInSystem checks both vertical beam ends.
    /// This uses the GRID area, including supplied system-area ends.
    pub fn is_beam_side_in_system(
        &self,
        id: u64,
        point: [f64; 2],
        height: f64,
    ) -> Result<bool, String> {
        if point.iter().any(|n| !n.is_finite()) || !height.is_finite() || height < 0. {
            return Err("invalid_beam_parallel_system_point".into());
        }
        self.validate_point_work(2)?;
        let system = self
            .systems
            .iter()
            .find(|s| s.id == id)
            .ok_or("missing_beam_parallel_system")?;
        for direction in [-1., 1.] {
            if !system
                .area
                .contains([point[0], point[1] + direction * (height / 2.)])?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub fn eligible(&self, point: [i32; 2]) -> Result<Vec<u64>, String> {
        let [x, y] = point;
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Ok(Vec::new());
        }
        self.validate_point_work(1)?;
        let mut out = Vec::new();
        for s in &self.systems {
            if x >= s.left && x <= s.right && s.area.contains([x as f64, y as f64])? {
                out.push(s.id);
            }
        }
        Ok(out)
    }
    pub fn register_closed(
        &self,
        registry: &mut Registry,
        g: &GlyphPixels,
    ) -> Result<Option<usize>, String> {
        if self.eligible(centroid(g)?)?.is_empty() {
            Ok(None)
        } else {
            registry.register(g).map(Some)
        }
    }
}
#[derive(Debug, Serialize)]
pub struct SystemSpots {
    pub system_id: u64,
    pub glyph_ids: Vec<usize>,
}
/// Dispatch in extracted-glyph order. Several overlapping system areas can own
/// the same canonical object; freeGlyphs is an insertion-ordered unique set.
pub fn dispatch(
    spots: &[GlyphPixels],
    geometry: &Geometry,
    registry: &mut Registry,
) -> Result<Vec<SystemSpots>, String> {
    let mut systems = geometry
        .systems
        .iter()
        .map(|s| SystemSpots {
            system_id: s.id,
            glyph_ids: Vec::new(),
        })
        .collect::<Vec<_>>();
    dispatch_into(spots, geometry, registry, &mut systems)?;
    Ok(systems)
}
/// Preserve existing free-glyph insertion order (for example VERTICAL_SEED)
/// when a raw beam spot reuses a previously registered canonical object.
pub fn dispatch_into(
    spots: &[GlyphPixels],
    geometry: &Geometry,
    registry: &mut Registry,
    systems: &mut [SystemSpots],
) -> Result<(), String> {
    if spots.len() > MAX_REGISTERED_GLYPHS {
        return Err("unsupported_beam_dispatch_spot_count".into());
    }
    geometry.validate_point_work(spots.len())?;
    let mut runs = 0usize;
    for glyph in spots {
        key(glyph)?;
        runs = runs
            .checked_add(glyph.runs.len())
            .filter(|r| *r <= MAX_REGISTERED_RUNS)
            .ok_or("unsupported_beam_dispatch_run_budget")?;
    }
    if systems.len() != geometry.systems.len() {
        return Err("invalid_beam_dispatch_initial_systems".into());
    }
    let mut seen = Vec::new();
    let mut references = 0usize;
    for (s, g) in systems.iter().zip(&geometry.systems) {
        if s.system_id != g.id {
            return Err("invalid_beam_dispatch_initial_systems".into());
        }
        references = references
            .checked_add(s.glyph_ids.len())
            .filter(|n| *n <= MAX_REGISTERED_GLYPHS)
            .ok_or("unsupported_beam_dispatch_reference_budget")?;
        let mut set = BTreeSet::new();
        for &id in &s.glyph_ids {
            registry.get(id)?;
            if !set.insert(id) {
                return Err("duplicate_beam_dispatch_initial_glyph".into());
            }
        }
        seen.push(set);
    }
    for glyph in spots {
        let ids = geometry.eligible(centroid(glyph)?)?;
        if ids.is_empty() {
            continue;
        }
        let gid = registry.register(glyph)?;
        registry.add_group(gid, Group::BeamSpot)?;
        for id in ids {
            let i = systems.iter().position(|s| s.system_id == id).unwrap();
            if seen[i].insert(gid) {
                references += 1;
                if references > MAX_REGISTERED_GLYPHS {
                    return Err("unsupported_beam_dispatch_reference_budget".into());
                }
                systems[i].glyph_ids.push(gid);
            }
        }
    }
    Ok(())
}
pub fn grouped_snapshot(
    systems: &[SystemSpots],
    registry: &Registry,
    group: Group,
) -> Result<Value, String> {
    let mut rows = Vec::new();
    for system in systems {
        let mut spots = Vec::new();
        for &id in &system.glyph_ids {
            let entry = registry.get(id)?;
            if entry.groups.contains(&group) {
                spots.push(json!({"glyph":glyph_json(&entry.glyph),"groups":entry.groups,"weight":entry.glyph.runs.iter().map(|r|r.length).sum::<usize>(),"centroid":centroid(&entry.glyph)?.map(|v|v as f64)}));
            }
        }
        rows.push(json!({"system_id":system.system_id,"grouped_spots":spots}));
    }
    Ok(json!(rows))
}
/// Original epilog drops only free glyphs whose sole group is BEAM_SPOT.
pub fn remove_exclusive_beam_spots(
    system: &mut SystemSpots,
    registry: &Registry,
) -> Result<(), String> {
    let mut kept = Vec::new();
    for &id in &system.glyph_ids {
        let groups = &registry.get(id)?.groups;
        if groups.len() != 1 || !groups.contains(&Group::BeamSpot) {
            kept.push(id);
        }
    }
    system.glyph_ids = kept;
    Ok(())
}

/// BeamsStep.doEpilog iterates every system after system processing. Validate
/// references before mutation, then retain each owner's free-glyph order.
/// Registry ownership/groups and SIG vertices/relations are unchanged.
pub fn finish_beams_epilog(
    systems: &mut [SystemSpots],
    registry: &Registry,
) -> Result<Vec<(u64, Vec<usize>)>, String> {
    for system in systems.iter() {
        for &id in &system.glyph_ids {
            registry.get(id)?;
        }
    }
    let mut removed = Vec::new();
    for system in systems {
        let before = system.glyph_ids.clone();
        remove_exclusive_beam_spots(system, registry)?;
        let kept = system.glyph_ids.iter().copied().collect::<BTreeSet<_>>();
        removed.push((
            system.system_id,
            before.into_iter().filter(|id| !kept.contains(id)).collect(),
        ));
    }
    Ok(removed)
}
