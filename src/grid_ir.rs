// Copyright © Audiveris 2026. Original author: Hervé Bitteur.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// StaffLine persistence, exact glyph registry, LagManager.rebuildHLag and
// single-column SystemManager section dispatch. Musical interpretations are separate.
use crate::{
    GrayImage,
    curves::{Curve, Point},
    position_runs,
    scale::Scale,
    sections::{self, GridPreparation, JunctionPolicy, Orientation, RunTable, Section},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct StaffGeometry {
    pub(crate) id: u64,
    pub(crate) system: u64,
    pub(crate) specific_interline: usize,
    pub(crate) abscissae: [i32; 2],
    pub(crate) lines: Vec<Arc<Curve>>,
}
impl StaffGeometry {
    pub(crate) fn pitch_at(
        &self,
        virtual_lines: &BTreeMap<i32, Arc<Curve>>,
        point: [f64; 2],
    ) -> Result<f64, String> {
        if self.lines.len() < 2
            || self.specific_interline == 0
            || point.iter().any(|p| !p.is_finite())
        {
            return Err("music_actual_fresh_staff_pitch_required".into());
        }
        let top = self.lines.first().unwrap().y(point[0]);
        let bottom = self.lines.last().unwrap().y(point[0]);
        if !top.is_finite() || !bottom.is_finite() || bottom <= top {
            return Err("music_actual_fresh_staff_pitch_required".into());
        }
        let span = (self.lines.len() - 1) as f64;
        let y = point[1];
        if y >= top && y <= bottom {
            Ok(span * (2. * y - bottom - top) / (bottom - top))
        } else {
            let dir = if y < top { -1 } else { 1 };
            let (mut line, mut pitch) = if dir < 0 {
                (top, -span)
            } else {
                (bottom, span)
            };
            for level in 1..=1024 {
                let Some(curve) = virtual_lines.get(&(dir * level)) else {
                    break;
                };
                let next = curve.y(point[0]);
                if !next.is_finite() || (next - line) * dir as f64 <= 0. {
                    return Err("music_actual_virtual_ledger_pitch_required".into());
                }
                if (next - y) * dir as f64 >= 0. {
                    return Ok(pitch + 2. * dir as f64 * (y - line) / (next - line));
                }
                line = next;
                pitch += 2. * dir as f64;
            }
            Ok(pitch + 2. * (y - line) / self.specific_interline as f64)
        }
    }
    pub(crate) fn middle_at(&self, x: i32) -> Result<i32, String> {
        let index = self
            .lines
            .len()
            .checked_sub(1)
            .ok_or("music_actual_staff_midline_required")?
            / 2;
        let y = self.lines[index].y(x as f64);
        if !y.is_finite() {
            return Err("music_actual_staff_midline_required".into());
        }
        Ok(y.round_ties_even() as i32)
    }
}
#[derive(Debug)]
pub(crate) struct PartGeometry {
    pub(crate) id: u64,
    pub(crate) system: u64,
    pub(crate) staffs: Vec<Arc<StaffGeometry>>,
    pub(crate) bounds: [i32; 4],
}
pub(crate) struct OwnedGrid {
    pub(crate) systems: Vec<Arc<SystemGeometry>>,
    pub(crate) physical_glyphs: Vec<Arc<audiveris_staff_removal::GlyphPixels>>,
    pub(crate) parts: Vec<Arc<PartGeometry>>,
    pub(crate) staff_areas: Vec<Arc<crate::system_area_geometry::Band>>,
    pub(crate) report: Value,
    pub(crate) staffs: Vec<Arc<StaffGeometry>>,
}
#[derive(Debug)]
pub(crate) struct SystemGeometry {
    pub(crate) id: u64,
    pub(crate) left: i32,
    pub(crate) area: Arc<crate::system_area_geometry::Band>,
    pub(crate) staffs: Vec<Arc<StaffGeometry>>,
}
#[derive(Default)]
pub struct GlyphRegistry {
    // Legacy registration returns an ID; owned GRID construction checks these
    // results before publishing its physical registry or any downstream owner.
    physical: Vec<Result<Arc<audiveris_staff_removal::GlyphPixels>, String>>,
    keys: BTreeMap<String, usize>,
    pub entries: Vec<Value>,
}
impl GlyphRegistry {
    pub fn register(&mut self, g: &Value) -> usize {
        let canonical = json!({"left":g["left"],"top":g["top"],"width":g["width"],"height":g["height"],"orientation":g["orientation"],"runs":g["runs"]});
        let key = canonical.to_string();
        if let Some(&id) = self.keys.get(&key) {
            return id;
        }
        let id = self.entries.len() + 1;
        let mut value = canonical;
        value["id"] = json!(id);
        let physical = (|| {
            let runs = g["runs"]
                .as_array()
                .ok_or("grid_physical_runs_required")?
                .iter()
                .map(|r| json!([r["position"], r["start"], r["length"]]))
                .collect::<Vec<_>>();
            crate::beam_dispatch::glyph_from_json(&json!({
                "bounds":[g["left"],g["top"],g["width"],g["height"]],
                "orientation":g["orientation"],"runs":runs
            }))
            .map(Arc::new)
        })();
        self.physical.push(physical);
        self.entries.push(value);
        self.keys.insert(key, id);
        id
    }
    pub(crate) fn physical_glyphs(
        &self,
    ) -> Result<Vec<Arc<audiveris_staff_removal::GlyphPixels>>, String> {
        self.physical.iter().cloned().collect()
    }
}
fn points(v: &Value) -> Vec<Point> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap() + 0.5])
        .collect()
}
fn simplify(p: Vec<Point>, segment: usize) -> Result<Curve, String> {
    let original = Curve::interpolate(p).ok_or("invalid_persistent_staff_curve")?;
    let left = original.points[0];
    let right = *original.points.last().unwrap();
    let mut defs = vec![left, right];
    loop {
        if (right[0] - left[0]) / ((defs.len() - 1) as f64) < segment as f64 {
            return Ok(original);
        }
        let spline = Curve::interpolate(defs.clone()).ok_or("invalid_simplified_curve")?;
        if original
            .points
            .iter()
            .all(|p| (spline.y(p[0]) - p[1]).abs() <= 1.)
        {
            return Ok(spline);
        }
        let mut expanded = vec![defs[0]];
        for w in defs.windows(2) {
            let x = 0.5 * (w[0][0] + w[1][0]);
            expanded.push([x, original.y(x)]);
            expanded.push(w[1]);
        }
        defs = expanded;
    }
}
pub(crate) fn centroid(s: &Section) -> [usize; 2] {
    let x = s
        .runs
        .iter()
        .map(|r| r.length * (2 * r.start + r.length))
        .sum::<usize>()
        / (2 * s.weight);
    let y = s
        .runs
        .iter()
        .enumerate()
        .map(|(i, r)| r.length * (s.first_pos + i))
        .sum::<usize>()
        / s.weight;
    match s.orientation {
        Orientation::Horizontal => [x, y],
        Orientation::Vertical => [y, x],
    }
}
fn section(s: &Section) -> Value {
    let b = s.bounds();
    json!({"id":s.id.0,"first_pos":s.first_pos,"centroid":centroid(s),"bounds":[b.x,b.y,b.width,b.height],"runs":s.runs.iter().map(|r|[r.start,r.length]).collect::<Vec<_>>()})
}
/// Input comes from native stages, never a Java fixture; output IDs have a native namespace.
pub fn build(
    residual: &GrayImage,
    sc: &Scale,
    prep: &GridPreparation,
    completion: &Value,
    systems: &Value,
    sticks: &Value,
) -> Result<Value, String> {
    Ok(build_owned(residual, sc, prep, completion, systems, sticks)?.report)
}
pub(crate) fn build_owned(
    residual: &GrayImage,
    sc: &Scale,
    prep: &GridPreparation,
    completion: &Value,
    systems: &Value,
    sticks: &Value,
) -> Result<OwnedGrid, String> {
    if systems["status"] != "initial_systems" {
        return Err("initial_systems_unavailable".into());
    }
    let final_staffs = completion["line_phases"]
        .as_array()
        .and_then(|p| p.last())
        .and_then(|p| p["staffs"].as_array())
        .ok_or("completed_line_geometry_unavailable")?;
    let glyphs = completion["staff_line_glyphs"]
        .as_array()
        .ok_or("line_membership_unavailable")?;
    let mut registry = GlyphRegistry::default();
    let bars = crate::bar_semantics::build(sc, sticks, systems, completion, &mut registry)?;
    let mut curves: Vec<Vec<Arc<Curve>>> = vec![];
    let mut staffs = vec![];
    let mut actual_staffs = Vec::new();
    let mut gi = 0;
    for staff in final_staffs {
        let mut lines = vec![];
        let mut staff_curves = vec![];
        for line in staff["lines"].as_array().unwrap() {
            let curve = Arc::new(simplify(points(&line["points"]), sc.interline.main * 4)?);
            let g = &glyphs[gi];
            gi += 1;
            let id = registry.register(g);
            lines.push(json!({"points":curve.points,"thickness":line["thickness"],"glyph_id":id}));
            staff_curves.push(curve);
        }
        let id = staff["id"].as_u64().unwrap();
        let system = systems["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["staff_ids"].as_array().unwrap().contains(&json!(id)))
            .ok_or("staff_without_system")?;
        let barline_ids: Vec<_> = bars["systems"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["inters"].as_array().unwrap())
            .filter(|v| v["kind"] == "BarlineInter" && v["staff_id"] == id)
            .map(|v| v["id"].clone())
            .collect();
        staffs.push(json!({"ledger_lines":[],"barline_ids":barline_ids,"specific_interline":staff["interline"].as_u64().unwrap_or(sc.interline.main as u64),"id":id,"system_id":system["id"],"left":staff["left"],"right":staff["right"],"lines":lines}));
        if staff["small"] == true {
            staffs.last_mut().unwrap()["small"] = json!(true);
        }
        actual_staffs.push(Arc::new(StaffGeometry {
            id,
            system: system["id"].as_u64().unwrap(),
            specific_interline: staff["interline"]
                .as_u64()
                .unwrap_or(sc.interline.main as u64) as usize,
            abscissae: [
                staff["left"]
                    .as_f64()
                    .ok_or("actual_staff_left_required")?
                    .round_ties_even() as i32,
                staff["right"]
                    .as_f64()
                    .ok_or("actual_staff_right_required")?
                    .round_ties_even() as i32,
            ],
            lines: staff_curves.clone(),
        }));
        curves.push(staff_curves);
    }
    // LagManager.dispatchRuns: discard long vertical runs from NO_STAFF, then build ALL horizontal sections.
    let mut short = GrayImage::new(
        residual.width,
        residual.height,
        vec![255; residual.pixels.len()],
    )?;
    for x in 0..residual.width {
        for r in position_runs(residual, true, x).filter(|r| r.length < prep.min_vertical_length) {
            for y in r.start..r.start + r.length {
                short.pixels[y * residual.width + x] = 0;
            }
        }
    }
    let table = RunTable::from_binary(&short, Orientation::Horizontal, |_| true)?;
    let horizontal = sections::build(&table, JunctionPolicy::Ratio(1.25))?;
    let margin = (sc.interline.main as f64 * 0.9).round_ties_even();
    let sys = systems["systems"].as_array().unwrap();
    let mut out = vec![];
    for (i, s) in sys.iter().enumerate() {
        let ids: Vec<_> = s["staff_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let first = ids[0] - 1;
        let last = *ids.last().unwrap() - 1;
        let left = ids
            .iter()
            .map(|&id| staffs[id - 1]["left"].as_f64().unwrap() as i64)
            .min()
            .unwrap();
        let right = ids
            .iter()
            .map(|&id| staffs[id - 1]["right"].as_f64().unwrap() as i64)
            .max()
            .unwrap();
        let top = curves[first][0].points[0][1];
        let bottom = curves[last].last().unwrap().points[0][1];
        let delta = curves[last][0].points[0][1] - top;
        out.push(json!({"id":s["id"],"staff_ids":s["staff_ids"],"coordinates":[left,top.round_ties_even()as i64,right-left+1,bottom.round_ties_even()as i64,delta.round_ties_even()as i64],"horizontal_section_ids":[],"vertical_section_ids":[],"area":{"x_range":[0,residual.width],"north_neighbor_last_line":if i==0{Value::Null}else{sys[i-1]["staff_ids"].as_array().unwrap().last().unwrap().clone()},"south_neighbor_first_line":if i+1==sys.len(){Value::Null}else{sys[i+1]["staff_ids"][0].clone()},"vertical_margin":margin}}));
    }
    let layout = crate::system_area_geometry::build(
        &json!({"staffs":staffs,"systems":out}),
        residual.width,
        residual.height,
        sc.interline.main,
    )?;
    for (i, system) in out.iter_mut().enumerate() {
        let plan = &layout.system_neighbors[i];
        system["area"]["x_range"] = plan["x_range"].clone();
        for (field, key) in [
            ("north_neighbor_last_line", "north_staff_ids"),
            ("south_neighbor_first_line", "south_staff_ids"),
        ] {
            let ids = plan[key].as_array().unwrap();
            system["area"][field] = if ids.len() == 1 {
                ids[0].clone()
            } else {
                Value::Null
            };
            if ids.len() > 1 {
                system["area"][key] = plan[key].clone();
            }
        }
        let contains = |s: &Section| {
            let [x, y] = centroid(s);
            layout.system_areas[i].contains([x as f64, y as f64])
        };
        let collect = |sections: &[Section]| -> Result<Vec<u32>, String> {
            let mut ids = Vec::new();
            for s in sections {
                if contains(s)? {
                    ids.push(s.id.0);
                }
            }
            Ok(ids)
        };
        system["horizontal_section_ids"] = json!(collect(&horizontal)?);
        system["vertical_section_ids"] = json!(collect(&prep.vertical)?);
    }
    let integrated = crate::braces::integrate_registered(
        json!({"schema":"audiveris-native.grid-ir.v1","grid_complete":false,"glyph_id_namespace":"native exact-pixel registry; not Java numeric IDs","section_namespaces":["post-removal horizontal","original vertical"],"staffs":staffs,"systems":out,"glyphs":registry.entries.clone(),"bar_semantics":bars,"hlag":horizontal.iter().map(section).collect::<Vec<_>>(),"vlag":prep.vertical.iter().map(section).collect::<Vec<_>>(),"pending":["complete BarsRetriever interpretations and contextualization","staff areas and page allocation"]}),
        completion,
        sc,
        residual.height,
        registry,
    )?;
    let mut parts = Vec::new();
    for membership in integrated.parts {
        let mut members = Vec::new();
        let mut area_bounds = None;
        for id in membership.staff_ids {
            let index = actual_staffs
                .iter()
                .position(|s| s.id == id && s.system == membership.system)
                .ok_or("grid_actual_part_staff_required")?;
            let b = layout.staff_areas[index].bounds()?;
            area_bounds = Some(match area_bounds {
                None => b,
                Some(a) => {
                    let a: [i32; 4] = a;
                    let x = a[0].min(b[0]);
                    let y = a[1].min(b[1]);
                    [
                        x,
                        y,
                        (a[0] + a[2]).max(b[0] + b[2]) - x,
                        (a[1] + a[3]).max(b[1] + b[3]) - y,
                    ]
                }
            });
            members.push(actual_staffs[index].clone());
        }
        parts.push(Arc::new(PartGeometry {
            id: membership.id,
            system: membership.system,
            staffs: members,
            bounds: area_bounds.ok_or("grid_actual_nonempty_part_required")?,
        }));
    }
    let actual_systems = out
        .iter()
        .zip(layout.system_areas)
        .map(|(system, area)| {
            let id = system["id"]
                .as_u64()
                .ok_or("grid_actual_system_id_required")?;
            Ok(Arc::new(SystemGeometry {
                id,
                left: system["coordinates"][0]
                    .as_i64()
                    .ok_or("grid_actual_system_left_required")? as i32,
                area: Arc::new(area),
                staffs: actual_staffs
                    .iter()
                    .filter(|s| s.system == id)
                    .cloned()
                    .collect(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(OwnedGrid {
        systems: actual_systems,
        physical_glyphs: integrated.physical_glyphs,
        report: integrated.report,
        staffs: actual_staffs,
        parts,
        staff_areas: layout.staff_areas.into_iter().map(Arc::new).collect(),
    })
}
