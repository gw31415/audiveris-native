// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Staff-local KeyBuilder.process, before KeyColumn alignment and finalizeKey.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{
    GrayImage,
    clef_competition::{ClefNode, ExclusionEdge, competing_clef_order},
    curves::Curve,
    key_clef::{self, ClefCandidate, ClefKind, KeyAlter, KeyShape},
    key_components,
    key_preparation::StaffInput,
    key_slices::{Shape, ShapePreparation},
};
use serde_json::{Value, json};

#[derive(Clone)]
pub(crate) struct Edge {
    pub(crate) kind: &'static str,
    pub(crate) source: u64,
    pub(crate) target: u64,
}
pub(crate) struct State {
    pub(crate) staff_id: u64,
    pub(crate) clefs: Vec<Value>,
    pub(crate) working: Vec<u64>,
    pub(crate) shapes: Vec<Value>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) next_id: u64,
    pub(crate) input: StaffInput,
    pub(crate) prepared: Vec<ShapePreparation>,
    pub(crate) browse_rect: [i64; 4],
    pub(crate) final_shape: Option<usize>,
    pub(crate) key_stop: Option<i64>,
    pub(crate) alter_starts: Option<Vec<i64>>,
}
pub(crate) fn integer(v: &Value) -> Result<i64, String> {
    v.as_i64().ok_or("invalid_key_state_integer".into())
}
pub(crate) fn number(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|x| x.is_finite())
        .ok_or("invalid_key_state_number".into())
}
pub(crate) fn rect(v: &Value) -> Result<[i64; 4], String> {
    let a = v
        .as_array()
        .filter(|a| a.len() == 4)
        .ok_or("invalid_key_state_bounds")?;
    let mut b = [0; 4];
    for (i, x) in a.iter().enumerate() {
        b[i] = integer(x)?;
    }
    if b.iter().any(|x| !(-4_000_000..=4_000_000).contains(x)) || b[2] <= 0 || b[3] <= 0 {
        return Err("invalid_key_state_bounds".into());
    }
    Ok(b)
}
pub(crate) fn kind(clef: &Value) -> Result<ClefKind, String> {
    serde_json::from_value(clef["clef_kind"].clone())
        .map_err(|_| "unsupported_key_state_clef_kind".into())
}
pub(crate) fn key_shape(shape: Shape) -> KeyShape {
    match shape {
        Shape::Flat => KeyShape::Flat,
        Shape::Sharp => KeyShape::Sharp,
    }
}
fn shape_name(shape: Shape) -> &'static str {
    match shape {
        Shape::Flat => "FLAT",
        Shape::Sharp => "SHARP",
    }
}
fn contextual(grade: f64, contribution: f64) -> f64 {
    ((1. + contribution) * grade) / (1. + contribution * grade)
}
fn slice_alters(state: &Value) -> Result<Vec<Option<KeyAlter>>, String> {
    state["slices"]
        .as_array()
        .filter(|s| s.len() <= 7)
        .ok_or("invalid_key_state_slices")?
        .iter()
        .map(|s| {
            let a = &s["alter"];
            if a.is_null() {
                Ok(None)
            } else {
                Ok(Some(KeyAlter {
                    intrinsic_grade: number(&a["grade"])?,
                    measured_pitch: number(&a["measured_pitch"])?,
                }))
            }
        })
        .collect()
}
fn apply_pitch_impact(chosen: ClefKind, shape: Shape, state: &mut Value) -> Result<(), String> {
    let pitched = key_clef::compute_pitched_grades(chosen, key_shape(shape), &slice_alters(state)?)
        .map_err(|e| e.to_string())?;
    // The source ignores the returned alter_count. Earlier writes survive early rejection.
    for (slice, grade) in state["slices"]
        .as_array_mut()
        .ok_or("invalid_key_state_slices")?
        .iter_mut()
        .zip(pitched.grades)
    {
        if !slice["alter"].is_null() {
            slice["alter"]["grade"] = json!(grade);
        }
    }
    Ok(())
}
fn pitch_ordinate(staff: &StaffInput, x: f64, pitch: f64) -> Result<f64, String> {
    if staff.lines.len() != 5 || !x.is_finite() || !pitch.is_finite() {
        return Err("unsupported_key_state_staff_pitch".into());
    }
    let y = |i: usize| {
        Curve::interpolate(staff.lines[i].clone())
            .map(|c| c.y(x))
            .filter(|y| y.is_finite())
            .ok_or("invalid_key_state_curve")
    };
    let top = y(0)?;
    let bottom = y(4)?;
    if bottom <= top {
        return Err("invalid_key_state_staff_height".into());
    }
    Ok(if pitch.abs() <= 4. {
        0.5 * (top + bottom + pitch * (bottom - top) / 4.)
    } else if pitch < 0. {
        top + 0.5 * (pitch + 4.) * staff.specific_interline as f64
    } else {
        bottom + 0.5 * (pitch - 4.) * staff.specific_interline as f64
    })
}
/// KeyExtractor.isRatherEmpty, including its first-space-column branch behavior.
fn rather_empty(
    image: &GrayImage,
    b: [i64; 4],
    max_cumul: i32,
    min_width: i64,
) -> Result<bool, String> {
    if b.iter().any(|x| !(-4_000_000..=4_000_000).contains(x)) || b[2] <= 0 || b[3] <= 0 {
        return Err("unsupported_key_trailing_rectangle".into());
    }
    let left = b[0].max(0);
    let right = (b[0] + b[2]).min(image.width as i64);
    let top = b[1].max(0);
    let bottom = (b[1] + b[3]).min(image.height as i64);
    let mut start = None;
    for x in left..right {
        let mut count = 0;
        for y in top..bottom {
            if image.pixels[y as usize * image.width + x as usize] == 0 {
                count += 1;
            }
        }
        if count <= max_cumul {
            if let Some(first) = start {
                if x - first + 1 >= min_width {
                    return Ok(true);
                }
            } else {
                start = Some(x);
            }
        } else {
            start = None;
        }
    }
    Ok(false)
}
fn trailing_ok(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    state: &Value,
) -> Result<bool, String> {
    let slices = state["slices"]
        .as_array()
        .ok_or("invalid_key_state_slices")?;
    let last = slices
        .iter()
        .enumerate()
        .rev()
        .find(|(_, s)| !s["alter"].is_null() && s["stuffed"] != true);
    let Some((index, slice)) = last else {
        return Ok(true);
    };
    if index != 0 {
        return Ok(true);
    }
    let a = &slice["alter"];
    let b = rect(&a["bounds"])?;
    let x = b[0] + b[2];
    let y = pitch_ordinate(staff, x as f64, number(&a["measured_pitch"])?)?.round_ties_even();
    if !y.is_finite() || y.abs() > 4_000_000. {
        return Err("unsupported_key_trailing_ordinate".into());
    }
    let sil = staff.specific_interline as i64;
    let limit = (0.25 * sil as f64).round_ties_even() as i64;
    rather_empty(
        image,
        [x, y as i64 - sil / 2, sheet_interline as i64, sil],
        limit as i32,
        limit,
    )
}
impl State {
    fn new(staff: &StaffInput, row: &Value, shapes: &[ShapePreparation]) -> Result<Self, String> {
        let candidates = row["inters"]
            .as_array()
            .filter(|v| v.len() <= 8)
            .ok_or("unsupported_key_state_clefs")?;
        let mut clefs = Vec::with_capacity(candidates.len());
        for (i, c) in candidates.iter().enumerate() {
            let grade = number(&c["grade"])?;
            if !(0.0..=1.0).contains(&grade) {
                return Err("invalid_key_state_clef_grade".into());
            }
            let _ = rect(&c["bounds"])?;
            let _: ClefKind = serde_json::from_value(c["kind"].clone())
                .map_err(|_| "unsupported_key_state_clef_kind")?;
            clefs.push(json!({"id":i as u64+1,"kind":"ClefInter","clef_kind":c["kind"],"shape":c["shape"],
                "grade":grade,"contextual_grade":null,"bounds":c["bounds"],"pitch":c["pitch"],"glyph":c["glyph"],"removed":false}));
        }
        let mut edges = Vec::new();
        for e in row["exclusions"]
            .as_array()
            .filter(|v| v.len() <= 28)
            .ok_or("invalid_key_state_exclusions")?
        {
            let source = e["source"]
                .as_u64()
                .filter(|n| *n < candidates.len() as u64)
                .ok_or("invalid_key_state_exclusion_endpoint")?
                + 1;
            let target = e["target"]
                .as_u64()
                .filter(|n| *n < candidates.len() as u64)
                .ok_or("invalid_key_state_exclusion_endpoint")?
                + 1;
            if source == target {
                return Err("invalid_key_state_self_exclusion".into());
            }
            edges.push(Edge {
                kind: "Exclusion",
                source,
                target,
            });
        }
        if shapes.len() != 2 || shapes[0].shape != Shape::Flat || shapes[1].shape != Shape::Sharp {
            return Err("unsupported_key_state_shape_order".into());
        }
        let state_shapes=shapes.iter().map(|s|json!({"shape":s.shape,"range":{"valid":false,
            "browse_start":s.range.browse_start,"browse_stop":s.range.browse_stop,"start":null,"stop":null},
            "key_inter":null,"slices":[]})).collect();
        Ok(Self {
            staff_id: staff.staff_id,
            next_id: candidates.len() as u64 + 1,
            input: staff.clone(),
            prepared: shapes.to_vec(),
            browse_rect: [0, 0, 1, 1],
            final_shape: None,
            key_stop: None,
            alter_starts: None,
            clefs,
            working: vec![],
            shapes: state_shapes,
            edges,
        })
    }
    pub(crate) fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub(crate) fn clef(&self, id: u64) -> Result<&Value, String> {
        self.clefs
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("missing_key_state_clef".into())
    }
    fn ordered_clefs(&self, x: i64) -> Result<Vec<u64>, String> {
        let nodes = self
            .clefs
            .iter()
            .filter(|c| c["removed"] != true)
            .map(|c| {
                Ok(ClefNode {
                    id: c["id"].as_u64().ok_or("invalid_key_state_id")?,
                    staff_id: self.staff_id,
                    left: integer(&c["bounds"][0])?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let edges = self
            .edges
            .iter()
            .filter(|e| e.kind == "Exclusion")
            .map(|e| ExclusionEdge {
                source: e.source,
                target: e.target,
            })
            .collect::<Vec<_>>();
        competing_clef_order(self.staff_id, x, &nodes, &edges)
    }
    fn remove_clef(&mut self, id: u64) {
        if let Some(c) = self.clefs.iter_mut().find(|c| c["id"] == id) {
            c["removed"] = json!(true);
        }
        self.edges.retain(|e| e.source != id && e.target != id);
    }
    fn retain_clef(&mut self, winner: u64) {
        for id in self.working.clone() {
            if id != winner {
                self.remove_clef(id);
            }
        }
        // Retain duplicate references just as ArrayList.retainAll does.
        self.working.retain(|id| *id == winner);
    }
    pub(crate) fn decorate_alters(
        &mut self,
        state: &mut Value,
        previous: Option<&Value>,
    ) -> Result<(), String> {
        let slices = state["slices"]
            .as_array_mut()
            .filter(|s| s.len() <= 7)
            .ok_or("invalid_key_state_slices")?;
        for (i, slice) in slices.iter_mut().enumerate() {
            slice["id"] = json!(i + 1);
            if slice.get("stuffed").is_none() {
                slice["stuffed"] = json!(false);
            }
            if slice["alter"].is_null() {
                continue;
            }
            let old = previous
                .and_then(|p| p["slices"].get(i))
                .map(|s| &s["alter"]);
            let preserved = old
                .filter(|a| !a.is_null() && a["glyph"] == slice["alter"]["glyph"])
                .and_then(|a| a["id"].as_u64());
            let id = preserved
                .or_else(|| slice["alter"]["id"].as_u64())
                .unwrap_or_else(|| self.id());
            slice["alter"]["id"] = json!(id);
            slice["alter"]["kind"] = json!("KeyAlterInter");
            if slice["alter"].get("removed").is_none() {
                slice["alter"]["removed"] = json!(false);
            }
            if slice["alter"].get("frozen").is_none() {
                slice["alter"]["frozen"] = json!(false);
            }
            if slice["alter"].get("contextual_grade").is_none() {
                slice["alter"]["contextual_grade"] = Value::Null;
            }
        }
        Ok(())
    }
    fn create_key(&mut self, index: usize, state: &Value, chosen: u64) -> Result<(), String> {
        if !self.create_ensemble(index, state)? {
            return Err("unsupported_key_state_empty_ensemble".into());
        }
        self.link_clef(index, chosen, true)
    }
    pub(crate) fn link_clef(
        &mut self,
        index: usize,
        chosen: u64,
        compute_context: bool,
    ) -> Result<(), String> {
        let id = self.shapes[index]["key_inter"]["id"]
            .as_u64()
            .ok_or("missing_key_state_ensemble")?;
        self.edges.push(Edge {
            kind: "ClefKeyRelation",
            source: chosen,
            target: id,
        });
        if compute_context {
            let grade = number(&self.shapes[index]["key_inter"]["grade"])?;
            let cg = contextual(grade, number(&self.clef(chosen)?["grade"])? * 5.);
            self.shapes[index]["key_inter"]["contextual_grade"] = json!(cg);
        }
        Ok(())
    }
    pub(crate) fn create_ensemble(&mut self, index: usize, state: &Value) -> Result<bool, String> {
        let mut members = state["slices"]
            .as_array()
            .ok_or("invalid_key_state_slices")?
            .iter()
            .filter(|s| !s["alter"].is_null() && s["alter"]["removed"] != true)
            .map(|s| s["alter"].clone())
            .collect::<Vec<_>>();
        if members.is_empty() {
            return Ok(false);
        }
        for i in 0..members.len() {
            for j in i + 1..members.len() {
                self.edges.push(Edge {
                    kind: "KeyAltersRelation",
                    source: members[i]["id"].as_u64().unwrap(),
                    target: members[j]["id"].as_u64().unwrap(),
                });
            }
        }
        let id = self.id();
        for member in &members {
            self.edges.push(Edge {
                kind: "Containment",
                source: id,
                target: member["id"].as_u64().unwrap(),
            });
        }
        let mut bounded = members
            .into_iter()
            .map(|m| Ok((rect(&m["bounds"])?, m)))
            .collect::<Result<Vec<_>, String>>()?;
        bounded.sort_by_key(|(b, _)| b[0] + b[2] / 2);
        members = bounded.iter().map(|(_, m)| m.clone()).collect();
        let mut sum = 0.;
        let mut union = bounded[0].0;
        for (b, m) in &bounded {
            sum += m["contextual_grade"]
                .as_f64()
                .unwrap_or(number(&m["grade"])?);
            let right = (union[0] + union[2]).max(b[0] + b[2]);
            let bottom = (union[1] + union[3]).max(b[1] + b[3]);
            union[0] = union[0].min(b[0]);
            union[1] = union[1].min(b[1]);
            union[2] = right - union[0];
            union[3] = bottom - union[1];
        }
        let grade = sum / members.len() as f64;
        let fifths = if self.shapes[index]["shape"] == "FLAT" {
            -(members.len() as i64)
        } else {
            members.len() as i64
        };
        self.shapes[index]["key_inter"] = json!({"id":id,"kind":"KeyInter","shape":null,"grade":grade,"contextual_grade":null,
            "bounds":union,"removed":false,"frozen":false,"glyph":null,"fifths":fifths,"members":members});
        Ok(true)
    }
    pub(crate) fn destroy(&mut self, index: usize) {
        let mut ids = self.shapes[index]["slices"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|s| s["alter"]["id"].as_u64())
            .collect::<Vec<_>>();
        if let Some(id) = self.shapes[index]["key_inter"]["id"].as_u64() {
            ids.push(id);
        }
        self.edges
            .retain(|e| !ids.contains(&e.source) && !ids.contains(&e.target));
        self.shapes[index]["slices"] = json!([]);
        self.shapes[index]["key_inter"] = Value::Null;
    }
    pub(crate) fn snapshot(&self, phase: &str) -> Result<Value, String> {
        let builder = self
            .working
            .iter()
            .map(|id| self.clef(*id).cloned())
            .collect::<Result<Vec<_>, _>>()?;
        let shapes = self
            .shapes
            .iter()
            .map(|s| {
                let mut s = s.clone();
                if let Some(slices) = s["slices"].as_array_mut() {
                    for (i, slice) in slices.iter_mut().enumerate() {
                        slice["id"] = json!(i + 1);
                        slice["rect"] = slice["bounds"].clone();
                        if let Some(m) = slice.as_object_mut() {
                            m.remove("bounds");
                        }
                    }
                }
                s
            })
            .collect::<Vec<_>>();
        Ok(
            json!({"phase":phase,"builder_clefs":builder,"remaining_sig_clefs":self.clefs.iter().filter(|c|c["removed"]!=true).collect::<Vec<_>>(),
            "shape_states":shapes,"relations":self.edges.iter().map(|e|json!({"kind":e.kind,"source":e.source,"target":e.target})).collect::<Vec<_>>()}),
        )
    }
    fn select_best_clef(&mut self) -> Result<(), String> {
        if self.working.len() == 1 {
            return Ok(());
        }
        let stop = self
            .shapes
            .iter()
            .map(|s| {
                integer(if s["range"]["stop"].is_null() {
                    &s["range"]["browse_stop"]
                } else {
                    &s["range"]["stop"]
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .min()
            .ok_or("missing_key_state_shapes")?;
        self.working = self.ordered_clefs(stop)?;
        for id in self.working.clone() {
            let mut partners = Vec::new();
            for edge in self
                .edges
                .iter()
                .filter(|e| e.kind == "ClefKeyRelation" && e.source == id)
            {
                let key = self
                    .shapes
                    .iter()
                    .map(|s| &s["key_inter"])
                    .find(|k| k["id"] == edge.target)
                    .ok_or("missing_key_state_partner")?;
                partners.push(number(&key["grade"])?);
            }
            partners.sort_by(|a, b| b.total_cmp(a));
            let mut contribution = 0.;
            for p in partners {
                contribution += p * 5.;
            }
            let clef = self
                .clefs
                .iter_mut()
                .find(|c| c["id"] == id)
                .ok_or("missing_key_state_clef")?;
            clef["contextual_grade"] = json!(contextual(number(&clef["grade"])?, contribution));
        }
        let mut order = self
            .working
            .iter()
            .map(|id| Ok((*id, number(&self.clef(*id)?["contextual_grade"])?)))
            .collect::<Result<Vec<_>, String>>()?;
        order.sort_by(|a, b| b.1.total_cmp(&a.1));
        if let Some(&(winner, _)) = order.first() {
            for (id, _) in order.iter().skip(1) {
                self.remove_clef(*id);
            }
            self.working = vec![winner];
        }
        Ok(())
    }
}

/// Full native staff-local processing; graph IDs in the output are local to this
/// snapshot namespace, not original Java IDs. No input or GRID graph is mutated.
pub fn process_staff(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    registration: &Value,
) -> Result<Value, String> {
    process_retained(image, sheet_interline, staff, registration).map(|(state, report)| {
        debug_assert_eq!(state.input.staff_id, staff.staff_id);
        debug_assert_eq!(state.prepared.len(), 2);
        debug_assert!(state.browse_rect[2] > 0 && state.browse_rect[3] > 0);
        debug_assert!(
            state.final_shape.is_none() && state.key_stop.is_none() && state.alter_starts.is_none()
        );
        report
    })
}

pub(crate) fn process_retained(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    registration: &Value,
) -> Result<(State, Value), String> {
    let projection = crate::key_preparation::prepare_staff(image, sheet_interline, staff)?;
    let peaks = crate::key_peaks::retrieve(&projection, staff.specific_interline)?;
    let slicing =
        crate::key_slices::prepare_staff(image, sheet_interline, staff, &projection, &peaks)?;
    let mut live = State::new(staff, registration, &slicing.shapes)?;
    live.browse_rect = projection.browse_rect;
    let mut stages = Vec::new();
    let mut fill_reports = Vec::new();
    for (index, shape) in slicing.shapes.iter().enumerate() {
        live.shapes[index]["range"] =
            serde_json::to_value(&shape.range).map_err(|e| e.to_string())?;
        if !shape.starts.is_empty() {
            let mut state = key_components::with_empty_slice_fallback(
                image,
                sheet_interline,
                staff,
                shape,
                1.0559375,
            )?;
            live.decorate_alters(&mut state, None)?;
            live.working.extend(live.ordered_clefs(shape.starts[0])?);
            let mut compatible = true;
            if !live.working.is_empty() {
                let clefs = live
                    .working
                    .iter()
                    .map(|id| {
                        let c = live.clef(*id)?;
                        Ok(ClefCandidate {
                            kind: kind(c)?,
                            intrinsic_grade: number(&c["grade"])?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let decision = key_clef::check_with_clefs(
                    key_shape(shape.shape),
                    &clefs,
                    &slice_alters(&state)?,
                )
                .map_err(|e| e.to_string())?;
                if let Some(winner_index) = decision.best_clef {
                    let winner = live.working[winner_index];
                    live.retain_clef(winner);
                    if decision.compatible_winner {
                        let chosen = kind(live.clef(winner)?)?;
                        let old = state.clone();
                        let report = key_components::fill_missing_alters(
                            image,
                            sheet_interline,
                            staff,
                            shape,
                            chosen,
                            &mut state,
                        )?;
                        fill_reports.push(json!({"shape":shape.shape,"report":report}));
                        live.decorate_alters(&mut state, Some(&old))?;
                        apply_pitch_impact(chosen, shape.shape, &mut state)?;
                        live.create_key(index, &state, winner)?;
                    }
                }
                compatible = decision.compatible_winner;
            }
            live.shapes[index]["slices"] = state["slices"].clone();
            if !compatible || !trailing_ok(image, sheet_interline, staff, &state)? {
                live.destroy(index);
            }
        }
        stages.push(live.snapshot(&format!("after_{}.process", shape_name(shape.shape)))?);
    }
    live.select_best_clef()?;
    stages.push(live.snapshot("after_selectBestClef")?);
    let report = json!({"staff_id":staff.staff_id,"typical_glyph_height":(staff.specific_interline as f64*2.5).round_ties_even()as i64,
        "stages":stages,"pitch_fill_reports":fill_reports});
    Ok((live, report))
}

pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    let inputs = crate::key_preparation::staff_inputs(grid, registration, sheet_interline)?;
    let rows = registration["staffs"]
        .as_array()
        .ok_or("missing_key_state_registration")?;
    let mut staffs = Vec::new();
    for input in inputs {
        let row = rows
            .iter()
            .find(|r| r["staff_id"] == input.staff_id)
            .ok_or("missing_key_state_registration")?;
        let mut state = process_staff(image, sheet_interline, &input, row)?;
        state["system_id"] = grid["staffs"]
            .as_array()
            .ok_or("missing_key_state_staff")?
            .iter()
            .find(|s| s["id"] == input.staff_id)
            .ok_or("missing_key_state_staff")?["system_id"]
            .clone();
        staffs.push(state);
    }
    Ok(
        json!({"stage":"HEADERS.key_process_state","id_namespace":"native staff-local automatic header inter identities",
        "staffs":staffs,"key_column_finalization_complete":false,"key_recognition_complete":false,"headers_complete":false,
        "pending":["cross-staff key alignment and replication","key finalization and header assignment","time recognition and final clef/header freezing"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn live() -> State {
        State {
            staff_id: 1,
            clefs: vec![
                json!({"id":1,"kind":"ClefInter","clef_kind":"TREBLE","grade":0.4,"contextual_grade":null,"bounds":[10,10,5,10],"removed":false}),
            ],
            working: vec![1],
            shapes: vec![
                json!({"shape":"FLAT","range":{"stop":null,"browse_stop":100},"key_inter":null,"slices":[]}),
                json!({"shape":"SHARP","range":{"stop":null,"browse_stop":100},"key_inter":null,"slices":[]}),
            ],
            edges: vec![],
            next_id: 20,
            input: StaffInput {
                staff_id: 1,
                header_start: 0,
                header_stop: 0,
                clef_stop: None,
                specific_interline: 10,
                lines: (0..5)
                    .map(|i| vec![[0., 30. + 10. * i as f64], [119., 30. + 10. * i as f64]])
                    .collect(),
                barlines: vec![],
            },
            prepared: vec![],
            browse_rect: [0, 0, 1, 1],
            final_shape: None,
            key_stop: None,
            alter_starts: None,
        }
    }
    #[test]
    fn one_column_empty_space_does_not_bypass_original_else_if() {
        let image = GrayImage::new(3, 1, vec![255, 0, 255]).unwrap();
        assert!(!rather_empty(&image, [0, 0, 3, 1], 0, 1).unwrap());
        let image = GrayImage::new(3, 1, vec![255, 255, 0]).unwrap();
        assert!(rather_empty(&image, [0, 0, 3, 1], 0, 1).unwrap());
        assert!(!rather_empty(&image, [3, 0, 2, 1], 0, 1).unwrap());
    }
    #[test]
    fn singleton_clef_skips_refresh_but_duplicate_working_refs_trigger_it() {
        let mut live = live();
        live.shapes[0]["key_inter"] = json!({"id":10,"grade":0.5});
        live.edges.push(Edge {
            kind: "ClefKeyRelation",
            source: 1,
            target: 10,
        });
        live.select_best_clef().unwrap();
        assert!(live.clefs[0]["contextual_grade"].is_null());
        live.working.push(1);
        live.select_best_clef().unwrap();
        assert_eq!(live.working, vec![1]);
        assert_eq!(
            live.clefs[0]["contextual_grade"],
            json!(contextual(0.4, 2.5))
        );
    }
    #[test]
    fn ensemble_grade_uses_current_intrinsic_mean_and_member_center_order() {
        let mut live = live();
        let state = json!({"slices":[{"alter":{"id":11,"grade":0.6,"contextual_grade":null,"bounds":[30,10,4,10],"removed":false}},
            {"alter":{"id":12,"grade":0.2,"contextual_grade":null,"bounds":[10,12,4,10],"removed":false}}]});
        live.create_key(0, &state, 1).unwrap();
        let key = &live.shapes[0]["key_inter"];
        assert_eq!(key["grade"], json!(0.4));
        assert!(key["shape"].is_null());
        assert_eq!(key["fifths"], -2);
        assert_eq!(key["bounds"], json!([10, 10, 24, 12]));
        assert_eq!(key["members"][0]["id"], 12);
        assert_eq!(key["members"][1]["id"], 11);
        assert!(
            number(&key["grade"]).unwrap() < key_clef::compute_key_grade(2, &[0.6, 0.2]).unwrap()
        );
        assert_eq!(live.edges[0].kind, "KeyAltersRelation");
        assert_eq!(live.edges[0].source, 11);
        assert_eq!(live.edges[0].target, 12);
        assert_eq!(live.edges[1].kind, "Containment");
        assert_eq!(live.edges[1].target, 11);
    }
    #[test]
    fn destruction_keeps_selected_clef_and_duplicate_retention_is_exact() {
        let mut live = live();
        live.clefs.push(json!({"id":2,"removed":false}));
        live.working = vec![1, 1, 2];
        live.edges.push(Edge {
            kind: "Exclusion",
            source: 1,
            target: 2,
        });
        live.retain_clef(1);
        assert_eq!(live.working, vec![1, 1]);
        assert_eq!(live.clefs[1]["removed"], true);
        assert!(live.edges.is_empty());
        live.shapes[0]["slices"] = json!([{"alter":{"id":11}}]);
        live.shapes[0]["key_inter"] = json!({"id":12});
        live.edges = vec![
            Edge {
                kind: "Containment",
                source: 12,
                target: 11,
            },
            Edge {
                kind: "ClefKeyRelation",
                source: 1,
                target: 12,
            },
        ];
        live.destroy(0);
        assert!(live.edges.is_empty());
        assert_eq!(live.working, vec![1, 1]);
        assert_eq!(live.clefs[0]["removed"], false);
        assert_eq!(live.shapes[0]["slices"], json!([]));
        assert!(live.shapes[0]["key_inter"].is_null());
    }
    #[test]
    fn post_fill_pitch_rejection_keeps_partial_grade_writes() {
        let mut state = json!({"slices":[{"alter":{"grade":0.8,"measured_pitch":-4.0}},
            {"alter":{"grade":0.8,"measured_pitch":10.0}},{"alter":{"grade":0.8,"measured_pitch":-5.0}}]});
        apply_pitch_impact(ClefKind::Treble, Shape::Sharp, &mut state).unwrap();
        assert_eq!(state["slices"][0]["alter"]["grade"], json!(0.8));
        assert_eq!(state["slices"][1]["alter"]["grade"], json!(0.0));
        assert_eq!(state["slices"][2]["alter"]["grade"], json!(0.0));
    }
}
