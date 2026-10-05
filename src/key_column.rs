// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// KeyColumn.retrieveKeys, replication and finalization, before time recognition.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{
    GrayImage, key_components,
    key_state::{self, State, integer, kind, number, rect},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn best_grade(inter: &Value) -> Result<f64, String> {
    if inter["contextual_grade"].is_null() {
        number(&inter["grade"])
    } else {
        number(&inter["contextual_grade"])
    }
}
fn shape_index(fifths: i64) -> Result<usize, String> {
    if fifths == 0 {
        return Err("unsupported_zero_key_fifths".into());
    }
    Ok(usize::from(fifths > 0))
}
fn slice_bounds(slice: &Value) -> Result<[i64; 4], String> {
    rect(&slice["bounds"])
}
fn slice_stop(slice: &Value) -> Result<i64, String> {
    let b = slice_bounds(slice)?;
    Ok(b[0] + b[2] - 1)
}

impl State {
    fn best_key(&self) -> Result<Option<usize>, String> {
        let mut best = None;
        let mut grade = 0.;
        for (i, shape) in self.shapes.iter().enumerate() {
            let key = &shape["key_inter"];
            if !key.is_null() {
                let g = best_grade(key)?;
                if best.is_none() || grade < g {
                    best = Some(i);
                    grade = g;
                }
            }
        }
        Ok(best)
    }
    fn get_alter(&self, id: u64) -> Option<Value> {
        for shape in &self.shapes {
            for s in shape["slices"].as_array().into_iter().flatten() {
                if s["alter"]["id"] == id {
                    return Some(s["alter"].clone());
                }
            }
            for a in shape["key_inter"]["members"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if a["id"] == id {
                    return Some(a.clone());
                }
            }
        }
        None
    }
    fn update_alter(&mut self, id: u64, update: impl Fn(&mut Value)) {
        for shape in &mut self.shapes {
            if let Some(slices) = shape["slices"].as_array_mut() {
                for s in slices {
                    if s["alter"]["id"] == id {
                        update(&mut s["alter"]);
                    }
                }
            }
            if let Some(members) = shape
                .get_mut("key_inter")
                .and_then(|k| k.get_mut("members"))
                .and_then(Value::as_array_mut)
            {
                for a in members {
                    if a["id"] == id {
                        update(a);
                    }
                }
            }
        }
    }
    fn refresh_key(&mut self, index: usize) -> Result<(), String> {
        let key = &self.shapes[index]["key_inter"];
        if key.is_null() || key["removed"] == true {
            return Ok(());
        }
        let id = key["id"].as_u64().ok_or("invalid_column_key_id")?;
        let ids = self
            .edges
            .iter()
            .filter(|e| e.kind == "Containment" && e.source == id)
            .map(|e| e.target)
            .collect::<Vec<_>>();
        let mut members = ids
            .into_iter()
            .map(|id| self.get_alter(id).ok_or("missing_column_member".into()))
            .collect::<Result<Vec<_>, String>>()?;
        members.sort_by_key(|a| {
            a["bounds"][0].as_i64().unwrap() + a["bounds"][2].as_i64().unwrap() / 2
        });
        let mut sum = 0.;
        let mut bounds = None;
        let mut fifths = 0;
        for member in &members {
            sum += best_grade(member)?;
            fifths += match member["shape"].as_str() {
                Some("FLAT") => -1,
                Some("SHARP") => 1,
                Some("NATURAL") => 0,
                _ => return Err("unsupported_column_member_shape".into()),
            };
            let b = rect(&member["bounds"])?;
            bounds = Some(if let Some(a) = bounds {
                let a: [i64; 4] = a;
                let left = a[0].min(b[0]);
                let top = a[1].min(b[1]);
                [
                    left,
                    top,
                    (a[0] + a[2]).max(b[0] + b[2]) - left,
                    (a[1] + a[3]).max(b[1] + b[3]) - top,
                ]
            } else {
                b
            });
        }
        let grade = if members.is_empty() {
            Value::Null
        } else {
            json!(sum / members.len() as f64)
        };
        let key = &mut self.shapes[index]["key_inter"];
        key["members"] = json!(members);
        key["grade"] = grade;
        key["bounds"] = json!(bounds);
        key["fifths"] = json!(fifths);
        // Containment invalidates intrinsic/cached geometry, not contextual grade.
        Ok(())
    }
    fn remove_key(&mut self, index: usize, extensive: bool) -> Result<(), String> {
        let key = &self.shapes[index]["key_inter"];
        if key.is_null() || key["removed"] == true {
            return Ok(());
        }
        let id = key["id"].as_u64().ok_or("invalid_column_key_id")?;
        let members = self
            .edges
            .iter()
            .filter(|e| e.kind == "Containment" && e.source == id)
            .map(|e| e.target)
            .collect::<Vec<_>>();
        self.shapes[index]["key_inter"]["removed"] = json!(true);
        if extensive {
            for member in members {
                if !self
                    .edges
                    .iter()
                    .any(|e| e.kind == "Containment" && e.target == member && e.source != id)
                {
                    self.remove_alter(member, false)?;
                }
            }
        }
        self.edges.retain(|e| e.source != id && e.target != id);
        self.shapes[index]["key_inter"]["members"] = json!([]);
        Ok(())
    }
    fn remove_alter(&mut self, id: u64, extensive: bool) -> Result<(), String> {
        if self.get_alter(id).is_none_or(|a| a["removed"] == true) {
            return Ok(());
        }
        let affected = self
            .shapes
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s["key_inter"]["id"].as_u64().map(|k| (i, k)))
            .filter(|(_, key)| {
                self.edges
                    .iter()
                    .any(|e| e.kind == "Containment" && e.source == *key && e.target == id)
            })
            .collect::<Vec<_>>();
        self.update_alter(id, |a| a["removed"] = json!(true));
        if extensive {
            for (i, key) in &affected {
                if self
                    .edges
                    .iter()
                    .filter(|e| e.kind == "Containment" && e.source == *key)
                    .count()
                    == 1
                {
                    self.remove_key(*i, false)?;
                }
            }
        }
        self.edges.retain(|e| e.source != id && e.target != id);
        for (i, _) in affected {
            self.refresh_key(i)?;
        }
        Ok(())
    }
    fn delete_slice(&mut self, index: usize, slice: usize) -> Result<(), String> {
        if let Some(id) = self.shapes[index]["slices"][slice]["alter"]["id"].as_u64() {
            self.remove_alter(id, true)?;
        }
        self.shapes[index]["slices"]
            .as_array_mut()
            .ok_or("invalid_column_slices")?
            .remove(slice);
        Ok(())
    }
    fn retry(
        &mut self,
        image: &GrayImage,
        il: usize,
        index: usize,
        slice: usize,
        crop: bool,
    ) -> Result<(), String> {
        let old = self.shapes[index].clone();
        let mut updated = old.clone();
        key_components::extract_alter(
            image,
            il,
            &self.input,
            &self.prepared[index],
            slice,
            &mut updated,
            crop,
        )?;
        self.decorate_alters(&mut updated, Some(&old))?;
        let old_id = old["slices"][slice]["alter"]["id"].as_u64();
        let new_id = updated["slices"][slice]["alter"]["id"].as_u64();
        if let Some(id) = old_id
            && old_id != new_id
        {
            self.remove_alter(id, true)?;
        }
        self.shapes[index]["slices"] = updated["slices"].clone();
        if let Some(id) = new_id {
            let updated = updated["slices"][slice]["alter"].clone();
            self.update_alter(id, |a| *a = updated.clone());
        }
        Ok(())
    }
    fn pitch_rect(&mut self, index: usize, slice: usize) -> Result<(), String> {
        let chosen = *self.working.first().ok_or("missing_column_clef")?;
        let b = slice_bounds(&self.shapes[index]["slices"][slice])?;
        let height = (2.5 * self.input.specific_interline as f64).round_ties_even() as usize;
        let b = key_components::pitch_window_rect(
            &self.input,
            b,
            self.prepared[index].shape,
            kind(self.clef(chosen)?)?,
            slice,
            height,
        )?;
        self.shapes[index]["slices"][slice]["bounds"] = json!(b);
        Ok(())
    }
    fn finalize(&mut self, image: &GrayImage, il: usize) -> Result<(), String> {
        let Some(index) = self.best_key()? else {
            return Ok(());
        };
        if let Some(&chosen) = self.working.first() {
            let table = crate::key_clef::pitches(
                kind(self.clef(chosen)?)?,
                key_state::key_shape(self.prepared[index].shape),
            )
            .ok_or("unsupported_column_percussion_key")?;
            let count = self.shapes[index]["slices"]
                .as_array()
                .ok_or("invalid_column_slices")?
                .len();
            if count > 7 {
                return Err("unsupported_column_slice_count".into());
            }
            for (i, &pitch) in table.iter().enumerate().take(count) {
                if self.shapes[index]["slices"][i]["alter"].is_null() {
                    self.retry(image, il, index, i, false)?;
                }
                let a = &self.shapes[index]["slices"][i]["alter"];
                if !a.is_null() && number(&a["pitch"])?.round_ties_even() != pitch as f64 {
                    let id = a["id"].as_u64().ok_or("invalid_column_alter_id")?;
                    self.update_alter(id, |a| a["pitch"] = json!(pitch as f64));
                }
            }
        }
        let last = self.shapes[index]["slices"]
            .as_array()
            .ok_or("invalid_column_slices")?
            .iter()
            .rev()
            .find(|s| s["stuffed"] != true && !s["alter"].is_null())
            .cloned();
        if let Some(last) = last {
            let b = rect(&last["alter"]["bounds"])?;
            let stop = b[0] + b[2] - 1;
            self.key_stop = Some(stop);
            self.final_shape = Some(index);
            self.shapes[index]["range"]["stop"] = json!(stop);
            self.shapes[index]["range"]["valid"] = json!(true);
            if self.shapes[index]["key_inter"].is_null() {
                let shape = self.shapes[index].clone();
                self.create_ensemble(index, &shape)?;
            }
            let slices = self.shapes[index]["slices"].as_array().unwrap();
            let ids = slices
                .iter()
                .filter_map(|s| s["alter"]["id"].as_u64())
                .collect::<Vec<_>>();
            let starts = slices
                .iter()
                .map(|s| integer(&s["bounds"][0]))
                .collect::<Result<Vec<_>, _>>()?;
            for id in ids {
                self.update_alter(id, |a| a["frozen"] = json!(true));
            }
            if !starts.is_empty() {
                self.alter_starts = Some(starts);
            }
        }
        self.destroy(1 - index);
        Ok(())
    }
    fn shrink(&mut self, index: usize) -> Result<(), String> {
        let slices = self.shapes[index]["slices"]
            .as_array_mut()
            .ok_or("invalid_column_slices")?;
        let last = slices
            .iter()
            .rposition(|s| s["stuffed"] != true && !s["alter"].is_null())
            .ok_or("unsupported_column_shrink_without_slice")?;
        // ROI removal and KeyInter.shrink refer independently to the last valid slice and last member.
        slices.remove(last);
        let last = self.shapes[index]["key_inter"]["members"]
            .as_array()
            .and_then(|m| m.last())
            .and_then(|m| m["id"].as_u64())
            .ok_or("unsupported_column_shrink_without_member")?;
        self.remove_alter(last, true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Ok,
    NoClef,
    NoReplicate,
    Shrink,
    Destroy,
}
fn closest(values: impl Iterator<Item = i64>, x: i64, limit: i64) -> Option<usize> {
    let mut best = None;
    let mut distance = i64::MAX;
    for (i, v) in values.enumerate() {
        let d = (x - v).abs();
        if d < distance {
            distance = d;
            best = Some(i);
        }
    }
    if distance <= limit { best } else { None }
}
fn nearest_slice(
    slices: &[Value],
    x: i64,
    limit: i64,
    stop: bool,
) -> Result<Option<usize>, String> {
    let values = slices
        .iter()
        .map(|s| {
            if stop {
                slice_stop(s)
            } else {
                integer(&s["bounds"][0])
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(closest(values.into_iter(), x, limit))
}
fn has_ink(image: &GrayImage, b: [i64; 4], il: usize) -> bool {
    let mut count = 0;
    for y in b[1].max(0)..(b[1] + b[3]).min(image.height as i64) {
        for x in b[0].max(0)..(b[0] + b[2]).min(image.width as i64) {
            if image.pixels[y as usize * image.width + x as usize] == 0 {
                count += 1;
            }
        }
    }
    count >= (0.2 * (il * il) as f64).round_ties_even() as usize
}
fn replicate(
    image: &GrayImage,
    il: usize,
    target: &mut State,
    index: usize,
    source: &Value,
    offsets: &[i64],
    max_dist: i64,
) -> Result<Status, String> {
    target.destroy(1 - index);
    let source_fifths = integer(&source["key_inter"]["fifths"])?;
    let own = &target.shapes[index]["key_inter"];
    if !own.is_null() && own["fifths"] == source["key_inter"]["fifths"] {
        return Ok(Status::Ok);
    }
    if target.working.is_empty() {
        return Ok(Status::NoClef);
    }
    let source_slices = source["slices"]
        .as_array()
        .filter(|s| s.len() <= 7)
        .ok_or("invalid_column_source_slices")?;
    let delta = (0.15 * il as f64).round_ties_even() as i64;
    let mut i = 0;
    while i < target.shapes[index]["slices"].as_array().unwrap().len() {
        let slice = &target.shapes[index]["slices"][i];
        let local = integer(&slice["bounds"][0])? - target.input.header_start;
        let matched = closest(offsets.iter().copied(), local, max_dist);
        if matched.is_none_or(|j| j >= source_slices.len()) {
            target.delete_slice(index, i)?;
            continue;
        }
        if slice["alter"].is_null() {
            let a = slice_bounds(slice)?;
            let b = slice_bounds(&source_slices[matched.unwrap()])?;
            if (a[0] - b[0]).abs() > delta || (a[2] - b[2]).abs() > delta {
                while i < target.shapes[index]["slices"].as_array().unwrap().len() {
                    target.delete_slice(index, i)?;
                }
                break;
            }
        }
        i += 1;
    }
    for (source_index, source_slice) in source_slices.iter().enumerate() {
        let offset = *offsets
            .get(source_index)
            .ok_or("unsupported_column_offset_count")?;
        let start = target.input.header_start + offset;
        let local = target.shapes[index]["slices"].as_array().unwrap();
        if nearest_slice(local, start, max_dist, false)?.is_some() {
            continue;
        }
        let start = if let Some(prev) = nearest_slice(local, start, max_dist, true)? {
            slice_stop(&local[prev])? + 1
        } else {
            start
        };
        let mut stop = start + integer(&source_slice["bounds"][2])? - 1;
        if let Some(next) = nearest_slice(local, stop + 1, max_dist, false)? {
            stop = integer(&local[next]["bounds"][0])? - 1;
        }
        if stop < start {
            return Err("unsupported_column_negative_slice".into());
        }
        let raw = target.browse_rect;
        // KeyRoi y/height are fixed at builder creation, not copied from source or last pitch window.
        let b = [start, raw[1], stop - start + 1, raw[3]];
        let slices = target.shapes[index]["slices"].as_array_mut().unwrap();
        if slices.len() >= 7 {
            return Err("unsupported_column_slice_count".into());
        }
        let at = slices
            .iter()
            .position(|s| slice_stop(s).is_ok_and(|x| stop <= x))
            .unwrap_or(slices.len());
        slices.insert(
            at,
            json!({"id":at+1,"bounds":b,"eval":null,"glyph":null,"alter":null,"stuffed":false}),
        );
        target.pitch_rect(index, at)?;
        let b = slice_bounds(&target.shapes[index]["slices"][at])?;
        if !has_ink(image, b, target.input.specific_interline) {
            if source_fifths.abs() > 1 && source_index + 1 == source_fifths.unsigned_abs() as usize
            {
                target.shapes[index]["slices"]
                    .as_array_mut()
                    .unwrap()
                    .remove(at);
                return Ok(Status::Shrink);
            }
            return Ok(Status::Destroy);
        }
    }
    let count = target.shapes[index]["slices"].as_array().unwrap().len();
    for i in 0..count {
        let alter = &target.shapes[index]["slices"][i]["alter"];
        if alter.is_null() || number(&alter["grade"])? < 0.1 {
            target.pitch_rect(index, i)?;
            target.retry(image, il, index, i, true)?;
        }
    }
    if !target.shapes[index]["key_inter"].is_null() {
        target.remove_key(index, true)?;
        target.shapes[index]["key_inter"] = Value::Null;
    }
    let current = target.shapes[index].clone();
    if !target.create_ensemble(index, &current)? {
        return Ok(Status::NoReplicate);
    }
    target.link_clef(index, target.working[0], false)?;
    Ok(Status::Ok)
}

/// Pinned Clustering.EM, Gaussian laws initialized with per-index means and sigma=1.
fn gaussian_fit(populations: &[Vec<f64>], values: &[f64]) -> Result<Vec<[f64; 3]>, String> {
    if populations.is_empty() {
        return Ok(vec![]);
    }
    if populations.len() > 7 || values.is_empty() || values.len() > 7168 {
        return Err("unsupported_column_em_budget".into());
    }
    let mut means = populations
        .iter()
        .map(|p| {
            if p.is_empty() {
                Err("invalid_column_population")
            } else {
                Ok(p.iter().sum::<f64>() / p.len() as f64)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let g = means.len();
    let n = values.len();
    let mut sigma = vec![1.; g];
    let mut pi = vec![1. / g as f64; g];
    let mut t = vec![vec![0.; n]; g];
    for _ in 0..10 {
        for (i, &x) in values.iter().enumerate() {
            let mut denominator = 0.;
            for k in 0..g {
                denominator += pi[k] * gaussian_probability(x, means[k], sigma[k]);
            }
            if denominator <= 0. || !denominator.is_finite() {
                return Err("unsupported_column_em_degenerate".into());
            }
            for k in 0..g {
                t[k][i] = pi[k] * gaussian_probability(x, means[k], sigma[k]) / denominator;
            }
        }
        let mut convergence = 0.;
        for k in 0..g {
            let old = pi[k];
            let mut new = 0.;
            for &weight in &t[k] {
                new += weight;
            }
            new /= n as f64;
            pi[k] = new;
            convergence += (new - old) * (new - old);
        }
        for k in 0..g {
            let mut weighted = 0.;
            let mut sum = 0.;
            for (i, &x) in values.iter().enumerate() {
                weighted += t[k][i] * x;
            }
            for &weight in &t[k] {
                sum += weight;
            }
            if sum <= 0. || !sum.is_finite() {
                return Err("unsupported_column_em_degenerate".into());
            }
            means[k] = weighted / sum;
            let mut variance = 0.;
            for (i, &x) in values.iter().enumerate() {
                variance += t[k][i] * (x - means[k]) * (x - means[k]);
            }
            sigma[k] = (variance / sum).sqrt();
        }
        if convergence < 1e-10 {
            break;
        }
    }
    Ok(means
        .into_iter()
        .zip(sigma)
        .zip(pi)
        .map(|((mean, sigma), pi)| [mean, sigma, pi])
        .collect())
}
fn gaussian_probability(x: f64, mean: f64, sigma: f64) -> f64 {
    let normalizer = (2. * std::f64::consts::PI).sqrt();
    let d = x - mean;
    if sigma <= 1e-10 {
        if d.abs() <= 1e-10 { 1. } else { 0. }
    } else {
        (-(d * d) / (2. * sigma * sigma)).exp() / (sigma * normalizer)
    }
}
fn gaussian_offsets(populations: &[Vec<f64>], values: &[f64]) -> Result<Vec<i64>, String> {
    gaussian_fit(populations, values)?
        .into_iter()
        .map(|[mean, _, _]| {
            if mean.is_finite() && mean.abs() <= 4_000_000. {
                Ok(mean.round_ties_even() as i64)
            } else {
                Err("unsupported_column_em_mean".into())
            }
        })
        .collect()
}

fn global_offsets(states: &[State]) -> Result<(Vec<i64>, i64), String> {
    let mut populations: Vec<Vec<f64>> = vec![];
    let mut values = vec![];
    let mut widths = 0;
    let mut count = 0;
    for state in states {
        if let Some(i) = state.best_key()? {
            for (j, slice) in state.shapes[i]["slices"]
                .as_array()
                .ok_or("invalid_column_slices")?
                .iter()
                .enumerate()
            {
                if j >= 7 {
                    return Err("unsupported_column_slice_count".into());
                }
                while populations.len() <= j {
                    populations.push(vec![]);
                }
                let b = slice_bounds(slice)?;
                let value = (b[0] - state.input.header_start) as f64;
                populations[j].push(value);
                values.push(value);
                widths += b[2];
                count += 1;
            }
        }
    }
    Ok((
        gaussian_offsets(&populations, &values)?,
        if count > 0 {
            (widths as f64 / count as f64).round_ties_even() as i64
        } else {
            0
        },
    ))
}

fn propagated_stop(start: i64, previous: i64, offset: i64) -> i64 {
    if offset > 0 { start + offset } else { previous }
}

/// Native fresh-GRID column processing. Per-staff IDs are local diagnostic identities.
/// `final_state` is before HeaderBuilder propagation; `after_system_key_stop` is after it.
/// Time recognition and final header clef assignment are deliberately still pending.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    il: usize,
) -> Result<Value, String> {
    let inputs = crate::key_preparation::staff_inputs(grid, registration, il)?;
    let registered = registration["staffs"]
        .as_array()
        .ok_or("missing_column_registration")?;
    let grid_staffs = grid["staffs"].as_array().ok_or("missing_column_staffs")?;
    let systems = grid["systems"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("unsupported_column_systems")?;
    let mut by_system: BTreeMap<u64, Vec<crate::key_preparation::StaffInput>> = BTreeMap::new();
    for input in inputs {
        let id = grid_staffs
            .iter()
            .find(|s| s["id"] == input.staff_id)
            .ok_or("missing_column_staff")?["system_id"]
            .as_u64()
            .ok_or("invalid_column_system")?;
        by_system.entry(id).or_default().push(input);
    }
    let mut output = vec![];
    for system in systems {
        let id = system["id"].as_u64().ok_or("invalid_column_system")?;
        let mut inputs = by_system
            .remove(&id)
            .ok_or("missing_column_system_staffs")?;
        inputs.sort_by_key(|s| s.staff_id);
        let mut states = Vec::new();
        for input in inputs {
            let row = registered
                .iter()
                .find(|r| r["staff_id"] == input.staff_id)
                .ok_or("missing_column_registration")?;
            states.push(key_state::process_retained(image, il, &input, row)?.0);
        }
        let parts = system["parts"]
            .as_array()
            .filter(|p| p.len() <= 1024)
            .ok_or("unsupported_column_parts")?;
        let mut seen = BTreeSet::new();
        let mut part_indices = vec![];
        for part in parts {
            part["id"].as_u64().ok_or("invalid_column_part_id")?;
            let ids = part["staff_ids"]
                .as_array()
                .filter(|a| !a.is_empty() && a.len() <= 1024)
                .ok_or("invalid_column_part")?;
            let mut indices = vec![];
            for sid in ids {
                let sid = sid.as_u64().ok_or("invalid_column_part_staff")?;
                if !seen.insert(sid) {
                    return Err("duplicate_column_part_staff".into());
                }
                indices.push(
                    states
                        .iter()
                        .position(|s| s.staff_id == sid)
                        .ok_or("unknown_column_part_staff")?,
                );
            }
            part_indices.push(indices);
        }
        if seen.len() != states.len() {
            return Err("incomplete_column_part_membership".into());
        }
        let max_dist = (0.5 * il as f64).round_ties_even() as i64;
        let mut offsets = None;
        let mut valid = true;
        let mut statuses = vec![];
        if states.len() > 1 {
            let (global, _mean) = global_offsets(&states)?;
            offsets = Some(global);
            valid = !offsets.as_ref().unwrap().is_empty();
            'parts: for part in &part_indices {
                if !valid {
                    break;
                }
                if part.len() <= 1 {
                    continue;
                }
                let mut best = None;
                let mut grade = -1.;
                for &i in part {
                    if let Some(shape) = states[i].best_key()? {
                        let g = best_grade(&states[i].shapes[shape]["key_inter"])?;
                        if best.is_none() || g > grade {
                            best = Some((i, shape));
                            grade = g;
                        }
                    }
                }
                let Some((source, best_index)) = best else {
                    continue;
                };
                let index = shape_index(integer(
                    &states[source].shapes[best_index]["key_inter"]["fifths"],
                )?)?;
                loop {
                    let mut modified = false;
                    for &target in part {
                        if target == source {
                            states[source].destroy(1 - index);
                            continue;
                        }
                        let source_shape = states[source].shapes[index].clone();
                        let status = replicate(
                            image,
                            il,
                            &mut states[target],
                            index,
                            &source_shape,
                            offsets.as_ref().unwrap(),
                            max_dist,
                        )?;
                        statuses.push(json!({"source_staff_id":states[source].staff_id,"target_staff_id":states[target].staff_id,"status":format!("{status:?}")}));
                        match status {
                            Status::Shrink => {
                                if offsets.as_mut().unwrap().pop().is_none() {
                                    return Err("unsupported_column_empty_shrink".into());
                                }
                                states[source].shrink(index)?;
                                modified = true;
                                break;
                            }
                            Status::Destroy => {
                                valid = false;
                                break 'parts;
                            }
                            _ => {}
                        }
                    }
                    if !modified {
                        break;
                    }
                }
            }
        }
        if valid {
            for state in &mut states {
                state.finalize(image, il)?;
            }
        } else {
            for state in &mut states {
                state.destroy(0);
                state.destroy(1);
            }
        }
        let returned = states
            .iter()
            .filter_map(|s| s.key_stop.map(|x| x - s.input.header_start))
            .max()
            .unwrap_or(0)
            .max(0);
        let mut staff_rows = vec![];
        let mut propagated = vec![];
        for state in &states {
            let header_key = state
                .final_shape
                .map(|i| state.shapes[i]["key_inter"].clone())
                .unwrap_or(Value::Null);
            let key_range = state
                .final_shape
                .map(|i| state.shapes[i]["range"].clone())
                .unwrap_or(Value::Null);
            let frozen = state
                .shapes
                .iter()
                .flat_map(|s| s["slices"].as_array().into_iter().flatten())
                .filter(|s| s["alter"]["frozen"] == true && s["alter"]["removed"] != true)
                .filter_map(|s| s["alter"]["id"].as_u64())
                .collect::<Vec<_>>();
            staff_rows.push(json!({"staff_id":state.staff_id,"header_start":state.input.header_start,"header_stop":state.input.header_stop,
                "clef_stop":state.input.clef_stop,"key_stop":state.key_stop,"header_key":header_key,"header_clef":null,"alter_starts":state.alter_starts,
                "key_range":key_range,"frozen_key_alter_ids":frozen,"builder_state":state.snapshot("column_checkpoint")?}));
            propagated.push(json!({"staff_id":state.staff_id,"header_start":state.input.header_start,"header_stop":propagated_stop(state.input.header_start,state.input.header_stop,returned),"clef_stop":state.input.clef_stop,"key_stop":state.key_stop,"key":header_key}));
        }
        output.push(json!({"system_id":id,"multi_staff":states.len()>1,"max_header_width":15*il,"max_slice_dist":max_dist,
            "clef_column_offset":states.iter().map(|s|s.input.header_stop-s.input.header_start).max().unwrap_or(0),
            "returned_key_column_offset":returned,"final_state":{"global_offsets":offsets,"parts":parts,"staffs":staff_rows},
            "replication_statuses":statuses,"after_system_key_stop":propagated}));
    }
    if !by_system.is_empty() {
        return Err("missing_column_system_record".into());
    }
    Ok(
        json!({"stage":"HEADERS.key_column","systems":output,"key_column_complete":true,"headers_complete":false,
        "pending":["time signatures","final clef selection and header freezing"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_preparation::StaffInput;
    fn state() -> State {
        let a = json!({"id":2,"kind":"KeyAlterInter","shape":"FLAT","grade":0.4,"contextual_grade":null,"bounds":[10,20,8,30],"removed":false});
        let b = json!({"id":3,"kind":"KeyAlterInter","shape":"FLAT","grade":0.8,"contextual_grade":0.9,"bounds":[30,20,8,30],"removed":false});
        State {
            staff_id: 1,
            clefs: vec![],
            working: vec![],
            shapes: vec![
                json!({"shape":"FLAT","slices":[{"bounds":[10,0,20,100],"alter":a},{"bounds":[30,0,20,100],"alter":b}],"key_inter":{"id":4,"kind":"KeyInter","grade":0.65,"contextual_grade":0.95,"bounds":[10,20,28,30],"fifths":-2,"members":[a,b],"removed":false}}),
                json!({"shape":"SHARP","slices":[],"key_inter":null}),
            ],
            edges: vec![
                key_state::Edge {
                    kind: "KeyAltersRelation",
                    source: 2,
                    target: 3,
                },
                key_state::Edge {
                    kind: "Containment",
                    source: 4,
                    target: 2,
                },
                key_state::Edge {
                    kind: "Containment",
                    source: 4,
                    target: 3,
                },
            ],
            next_id: 5,
            input: StaffInput {
                staff_id: 1,
                header_start: 0,
                header_stop: 0,
                clef_stop: None,
                specific_interline: 10,
                lines: vec![],
                barlines: vec![],
            },
            prepared: vec![],
            browse_rect: [0, 0, 100, 100],
            final_shape: None,
            key_stop: None,
            alter_starts: None,
        }
    }
    #[test]
    fn independent_original_clustering_controls() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-grid/clustering-controls.json"
        ))
        .unwrap();
        let cases = oracle["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 14);
        let mut finite = 0;
        for case in cases {
            let pops: Vec<Vec<f64>> = serde_json::from_value(case["populations"].clone()).unwrap();
            let values: Vec<f64> =
                serde_json::from_value(case["flattened_values"].clone()).unwrap();
            let expected = case["final_laws"].as_array().unwrap();
            let fit = gaussian_fit(&pops, &values);
            if expected.iter().any(|v| v["mean"].is_string()) {
                assert_eq!(fit.unwrap_err(), "unsupported_column_em_degenerate");
                continue;
            }
            finite += 1;
            let fit = fit.unwrap();
            assert_eq!(fit.len(), expected.len());
            let offsets = gaussian_offsets(&pops, &values).unwrap();
            for (i, (actual, expected)) in fit.iter().zip(expected).enumerate() {
                for (j, k) in ["mean", "sigma", "pi"].iter().enumerate() {
                    let target = expected[k].as_f64().unwrap();
                    assert!(
                        (actual[j] - target).abs() <= 1e-12,
                        "{} {k}: {} vs {target}",
                        case["name"],
                        actual[j]
                    );
                }
                assert_eq!(offsets[i], expected["rounded_offset"].as_i64().unwrap());
            }
        }
        assert_eq!(finite, 13);
        let controls = oracle["gaussian_probability_controls"].as_array().unwrap();
        assert_eq!(controls.len(), 20);
        for c in controls {
            let actual = gaussian_probability(
                c["x"].as_f64().unwrap(),
                c["mean"].as_f64().unwrap(),
                c["sigma"].as_f64().unwrap(),
            );
            let expected = c["proba"].as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 1e-12 * expected.abs().max(1.),
                "{c}"
            );
        }
    }
    #[test]
    fn nearest_is_inclusive_and_first_tie() {
        assert_eq!(closest([10, 20].into_iter(), 15, 5), Some(0));
        assert_eq!(closest([10, 20].into_iter(), 15, 4), None);
        assert_eq!(closest([].into_iter(), 15, 5), None);
    }
    #[test]
    fn em_handles_collapse_and_half_even_means() {
        assert_eq!(
            gaussian_offsets(&[vec![10., 10.], vec![30., 30.]], &[10., 30., 10., 30.]).unwrap(),
            [10, 30]
        );
        assert_eq!(
            gaussian_offsets(&[vec![10., 11.]], &[10., 11.]).unwrap(),
            [10]
        );
        assert_eq!(
            gaussian_offsets(&[vec![11., 12.]], &[11., 12.]).unwrap(),
            [12]
        );
        assert!(gaussian_offsets(&[vec![0.]], &[1e6]).is_err());
    }
    #[test]
    fn removing_member_keeps_stale_contextual_grade() {
        let mut state = state();
        state.remove_alter(3, true).unwrap();
        let key = &state.shapes[0]["key_inter"];
        assert_eq!(key["grade"], 0.4);
        assert_eq!(key["contextual_grade"], 0.95);
        assert_eq!(key["fifths"], -1);
        assert_eq!(key["bounds"], json!([10, 20, 8, 30]));
        assert_eq!(state.shapes[0]["slices"][1]["alter"]["removed"], true);
        assert!(state.shapes[1]["key_inter"].is_null());
    }
    #[test]
    fn extensive_key_removal_keeps_removed_slice_aliases() {
        let mut state = state();
        state.remove_key(0, true).unwrap();
        assert!(state.edges.is_empty());
        for s in state.shapes[0]["slices"].as_array().unwrap() {
            assert_eq!(s["alter"]["removed"], true);
        }
        assert_eq!(state.shapes[0]["key_inter"]["removed"], true);
        assert!(!state.create_ensemble(0, &state.shapes[0].clone()).unwrap());
        assert!(state.shapes[1]["key_inter"].is_null());
    }
    #[test]
    fn ink_threshold_and_image_clamp_are_exact() {
        let mut image = GrayImage::new(10, 10, vec![255; 100]).unwrap();
        for pixel in image.pixels.iter_mut().take(20) {
            *pixel = 0;
        }
        assert!(has_ink(&image, [-3, -3, 16, 16], 10));
        image.pixels[19] = 255;
        assert!(!has_ink(&image, [-3, -3, 16, 16], 10));
    }
    #[test]
    fn key_column_offset_preserves_previous_stop_when_no_key() {
        assert_eq!(propagated_stop(80, 140, 0), 140);
        assert_eq!(propagated_stop(80, 140, 190), 270);
        assert_eq!(propagated_stop(110, 150, 190), 300);
    }
}
