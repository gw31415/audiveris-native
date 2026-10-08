// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use serde_json::json;
use std::sync::Arc;

#[cfg(test)]
pub(super) type CallbackOwners = Vec<(Key, Option<Arc<()>>, Option<Arc<()>>)>;

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct CallbackObservation {
    pub(crate) edge: usize,
    pub(crate) phase: &'static str,
    pub(crate) registration: serde_json::Value,
    pub(crate) snapshot: serde_json::Value,
    pub(crate) owners: CallbackOwners,
}

fn skewed_cross(median: [f64; 4], point: [f64; 2], skew: crate::sheet_skew::SheetSkew) -> [f64; 2] {
    let [x1, y1, x2, y2] = median;
    let [x3, y3, x4, y4] = skew.vertical(point);
    let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let v12 = x1 * y2 - y1 * x2;
    let v34 = x3 * y4 - y3 * x4;
    [
        (v12 * (x3 - x4) - (x1 - x2) * v34) / den,
        (v12 * (y3 - y4) - (y1 - y2) * v34) / den,
    ]
}

impl Graph {
    pub(crate) fn add_owned_beam_rest(
        &mut self,
        beam: Key,
        rest: Key,
        manual: bool,
        skew: crate::sheet_skew::SheetSkew,
        interline: usize,
    ) -> Result<usize, String> {
        let source = self.endpoint(beam)?;
        let target = self.endpoint(rest)?;
        if beam.phase != Phase::Beam
            || rest.phase != Phase::Symbol
            || beam.system != rest.system
            || self.vertices[source - 1].data["kind"] != "BeamInter"
            || self.vertices[target - 1].data["kind"] != "RestInter"
        {
            return Err("music_rest_actual_typed_addition_required".into());
        }
        let median: [f64; 4] =
            serde_json::from_value(self.vertices[source - 1].data["median"].clone())
                .map_err(|_| "music_rest_actual_beam_median_required")?;
        let bounds: [i32; 4] =
            serde_json::from_value(self.vertices[target - 1].data["bounds"].clone())
                .map_err(|_| "music_rest_actual_note_bounds_required")?;
        if !median.iter().all(|n| n.is_finite()) || bounds[2] <= 0 || bounds[3] <= 0 {
            return Err("music_rest_actual_geometry_required".into());
        }
        let point = [
            bounds[0].checked_add(bounds[2] / 2),
            bounds[1].checked_add(bounds[3] / 2),
        ];
        let point = [
            point[0].ok_or("music_rest_actual_note_bounds_required")? as f64,
            point[1].ok_or("music_rest_actual_note_bounds_required")? as f64,
        ];
        let cross = skewed_cross(median, point, skew);
        let distance = (point[0] - cross[0]).hypot(point[1] - cross[1]);
        let gap = i32::try_from(interline)
            .ok()
            .filter(|v| *v > 0)
            .ok_or("music_rest_measured_scale_required")? as f64;
        if !distance.is_finite() {
            return Err("music_rest_actual_geometry_required".into());
        }
        let dy = distance.round_ties_even() as i32;
        let portion = if point[0] < median[0] + gap {
            "LEFT"
        } else if point[0] > median[2] - gap {
            "RIGHT"
        } else {
            "CENTER"
        };
        // SIGraph accepts a fresh parallel relation. Reconciliation or revival
        // of an old registration would overwrite its manual status and identity.
        let ordinal = match self
            .edge_keys
            .keys()
            .filter(|k| {
                k.system == beam.system
                    && k.phase == Phase::Symbol
                    && k.source == source
                    && k.target == target
                    && k.kind == "BeamRestRelation"
            })
            .map(|k| k.ordinal)
            .max()
        {
            Some(last) => last
                .checked_add(1)
                .ok_or("shared_sig_relation_order_budget")?,
            None => 0,
        };
        self.relations(
            beam.system,
            Phase::Symbol,
            vec![(
                beam,
                rest,
                json!({"kind":"BeamRestRelation","manual":manual,
            "dy":0,"beam_portion":null,"_editing_registration_ordinal":ordinal}),
            )],
        )?;
        let id = *self
            .edge_keys
            .get(&super::EdgeKey {
                system: beam.system,
                phase: Phase::Symbol,
                source,
                target,
                kind: "BeamRestRelation".into(),
                ordinal,
            })
            .unwrap();
        #[cfg(test)]
        self.observe_beam_rest_callback(id, "entered");
        self.edges[id - 1].data["dy"] = json!(dy);
        self.edges[id - 1].data["beam_portion"] = json!(portion);
        self.check_beam_rest_abnormal(source);
        #[cfg(test)]
        self.observe_beam_rest_callback(id, "finished");
        Ok(id)
    }
    pub(crate) fn nearest_rest_beam(
        &self,
        group: Key,
        point: [i32; 2],
        skew: crate::sheet_skew::SheetSkew,
    ) -> Result<Option<(Key, f64)>, String> {
        let gid = self.endpoint(group)?;
        if group.phase != Phase::Group || self.vertices[gid - 1].data["kind"] != "BeamGroupInter" {
            return Err("music_rest_actual_group_required".into());
        }
        let point = [point[0] as f64, point[1] as f64];
        let mut nearest = None;
        for edge in self
            .live_edges()
            .into_iter()
            .filter(|e| e.key.source == gid && e.key.kind == "Containment")
        {
            let beam = &self.vertices[edge.key.target - 1];
            if beam.removed || beam.key.system != group.system {
                return Err("music_rest_actual_group_beam_required".into());
            }
            match beam.data["kind"].as_str() {
                Some("BeamHookInter" | "SmallBeamInter") => continue,
                Some("BeamInter") => {}
                _ => return Err("music_rest_actual_group_beam_required".into()),
            }
            let median: [f64; 4] = serde_json::from_value(beam.data["median"].clone())
                .map_err(|_| "music_rest_actual_beam_median_required")?;
            if !median.iter().all(|n| n.is_finite()) {
                return Err("music_rest_actual_beam_median_required".into());
            }
            let [x, y] = skewed_cross(median, point, skew);
            if x >= median[0] && x <= median[2] {
                let distance = (point[0] - x).hypot(point[1] - y);
                if nearest.is_none_or(|(_, best)| distance < best) {
                    nearest = Some((beam.key, distance));
                }
            }
        }
        Ok(nearest)
    }
    pub(crate) fn main_rest_median(&self, group: Key) -> Result<Option<[f64; 4]>, String> {
        let gid = self.endpoint(group)?;
        if group.phase != Phase::Group || self.vertices[gid - 1].data["kind"] != "BeamGroupInter" {
            return Err("music_rest_actual_group_required".into());
        }
        let mut main = None;
        for edge in self
            .live_edges()
            .into_iter()
            .filter(|e| e.key.source == gid && e.key.kind == "Containment")
        {
            let beam = &self.vertices[edge.key.target - 1];
            if beam.removed
                || beam.key.system != group.system
                || !matches!(
                    beam.data["kind"].as_str(),
                    Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
                )
            {
                return Err("music_rest_actual_group_beam_required".into());
            }
            let median: [f64; 4] = serde_json::from_value(beam.data["median"].clone())
                .map_err(|_| "music_rest_actual_beam_median_required")?;
            if !median.iter().all(|v| v.is_finite()) {
                return Err("music_rest_actual_beam_median_required".into());
            }
            // Main geometry includes Hook/Small; nearest candidates are full Beam only.
            if main.is_none_or(|old: [f64; 4]| old[2] - old[0] < median[2] - median[0]) {
                main = Some(median);
            }
        }
        Ok(main)
    }
    pub(crate) fn rest_chord_beams(&mut self, key: Key) -> Result<Arc<Vec<usize>>, String> {
        let id = self.endpoint(key)?;
        if key.phase != Phase::Chord || self.vertices[id - 1].data["kind"] != "RestChordInter" {
            return Err("music_rest_actual_chord_required".into());
        }
        if let Some(beams) = &self.vertices[id - 1].rest_beams {
            return Ok(beams.clone());
        }
        let beams = Arc::new(crate::music_voices::chord_beams(&self.snapshot(), id)?);
        self.vertices[id - 1].rest_beams = Some(beams.clone());
        Ok(beams)
    }
    pub(crate) fn actual_chord_beams(&mut self, key: Key) -> Result<Vec<usize>, String> {
        let id = self.endpoint(key)?;
        if self.vertices[id - 1].data["kind"] == "RestChordInter" {
            return Ok(self.rest_chord_beams(key)?.as_ref().clone());
        }
        if key.phase != Phase::Chord || self.vertices[id - 1].data["kind"] != "HeadChordInter" {
            return Err("music_rest_actual_chord_required".into());
        }
        if self.vertices[id - 1].chord_cache.is_some() {
            self.retained_chord_beams(key)
        } else {
            crate::music_voices::chord_beams(&self.snapshot(), id)
        }
    }
    pub(crate) fn beam_rest_relations(&self, key: Key) -> Result<Vec<(usize, Key, Key)>, String> {
        let id = self.endpoint(key)?;
        self.incident_edges(id)
            .into_iter()
            .filter(|eid| self.edges[*eid - 1].key.kind == "BeamRestRelation")
            .map(|eid| {
                let edge = &self.edges[eid - 1];
                let beam = &self.vertices[edge.key.source - 1];
                let rest = &self.vertices[edge.key.target - 1];
                if beam.removed
                    || rest.removed
                    || beam.key.phase != Phase::Beam
                    || rest.key.phase != Phase::Symbol
                    || beam.key.system != rest.key.system
                    || !matches!(
                        beam.data["kind"].as_str(),
                        Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
                    )
                    || rest.data["kind"] != "RestInter"
                {
                    return Err("music_rest_actual_typed_relation_required".into());
                }
                Ok((eid, beam.key, rest.key))
            })
            .collect()
    }
    pub(crate) fn beam_rest_distance(&self, id: usize) -> Result<i32, String> {
        self.edges
            .get(
                id.checked_sub(1)
                    .ok_or("music_rest_relation_identity_required")?,
            )
            .filter(|e| !e.removed && e.key.kind == "BeamRestRelation")
            .and_then(|e| e.data["dy"].as_i64())
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| "music_rest_actual_relation_distance_required".into())
    }
    pub(crate) fn rest_contains_candidate_vertical(
        &self,
        rest: Key,
        center: [i32; 2],
        skew: crate::sheet_skew::SheetSkew,
    ) -> Result<bool, String> {
        let id = self.endpoint(rest)?;
        let vertex = &self.vertices[id - 1];
        if vertex.data["kind"] != "RestInter" {
            return Err("music_rest_actual_note_required".into());
        }
        let bounds: [i32; 4] = serde_json::from_value(vertex.data["bounds"].clone())
            .map_err(|_| "music_rest_actual_note_bounds_required")?;
        let y = bounds[1]
            .checked_add(bounds[3] / 2)
            .ok_or("music_rest_actual_note_bounds_required")? as f64;
        let [x1, y1, x2, y2] = skew.vertical(center.map(f64::from));
        let (x3, y3, x4, y4) = (0., y, 1000., y);
        let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        let (a, b) = (x1 * y2 - y1 * x2, x3 * y4 - y3 * x4);
        let (x, y) = (
            (a * (x3 - x4) - (x1 - x2) * b) / den,
            (a * (y3 - y4) - (y1 - y2) * b) / den,
        );
        if !x.is_finite() || !y.is_finite() {
            return Err("music_rest_actual_geometry_required".into());
        }
        Ok(bounds[2] > 0
            && bounds[3] > 0
            && x >= bounds[0] as f64
            && y >= bounds[1] as f64
            && x < bounds[0] as f64 + bounds[2] as f64
            && y < bounds[1] as f64 + bounds[3] as f64)
    }
    pub(crate) fn remove_owned_beam_rest(&mut self, id: usize) -> Result<(), String> {
        self.remove_beam_rest_callback(id, true)
    }
    pub(crate) fn remove_lookup_beam_rest(&mut self, id: usize) -> Result<(), String> {
        self.remove_beam_rest_callback(id, false)
    }
    fn remove_beam_rest_callback(&mut self, id: usize, automatic_only: bool) -> Result<(), String> {
        let edge = self
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("music_rest_relation_identity_required")?,
            )
            .filter(|e| !e.removed && e.key.kind == "BeamRestRelation")
            .ok_or("music_rest_relation_identity_required")?;
        let source = edge.key.source;
        let beam = &self.vertices[source - 1];
        let rest = &self.vertices[edge.key.target - 1];
        if (automatic_only
            && (edge.data["manual"] == true || beam.data["kind"] == "SmallBeamInter"))
            || beam.removed
            || rest.removed
            || !matches!(
                beam.data["kind"].as_str(),
                Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
            )
            || rest.data["kind"] != "RestInter"
            || beam.key.system != rest.key.system
        {
            return Err(if automatic_only {
                "music_rest_automatic_typed_relation_required"
            } else {
                "music_rest_actual_typed_relation_required"
            }
            .into());
        }
        self.edges[id - 1].removed = true;
        #[cfg(test)]
        self.observe_beam_rest_callback(id, "remove_entered");
        self.check_beam_rest_abnormal(source);
        #[cfg(test)]
        self.observe_beam_rest_callback(id, "remove_finished");
        Ok(())
    }
    fn check_beam_rest_abnormal(&mut self, source: usize) {
        let relations = self
            .incident_edges(source)
            .into_iter()
            .map(|i| &self.edges[i - 1])
            .filter(|e| matches!(e.key.kind.as_str(), "BeamStemRelation" | "BeamRestRelation"))
            .collect::<Vec<_>>();
        let abnormal = if self.vertices[source - 1].data["kind"] == "BeamHookInter" {
            !relations.iter().any(|e| e.key.kind == "BeamStemRelation")
        } else {
            !relations.iter().any(|e| e.data["beam_portion"] == "LEFT")
                || !relations.iter().any(|e| e.data["beam_portion"] == "RIGHT")
        };
        // Original removal checks the beam without clearing RestChord beams or
        // clocks. Rhythm reset clears clocks; invalidation clears beam caches.
        self.vertices[source - 1].data["abnormal"] = json!(abnormal);
    }
    #[cfg(test)]
    pub(crate) fn beam_rest_registration_ordinal(&self, id: usize) -> usize {
        self.edges[id - 1].key.ordinal
    }
    #[cfg(test)]
    pub(crate) fn beam_rest_endpoints(&self, id: usize) -> Result<(usize, usize), String> {
        let e = self
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("music_rest_relation_identity_required")?,
            )
            .filter(|e| e.key.kind == "BeamRestRelation")
            .ok_or("music_rest_relation_identity_required")?;
        Ok((e.key.source, e.key.target))
    }
    #[cfg(test)]
    pub(crate) fn beam_rest_registration(&self, id: usize) -> serde_json::Value {
        let edge = &self.edges[id - 1];
        json!({"registered":!edge.removed,"manual":edge.data["manual"]==true,
            "dy":edge.data["dy"],"portion":edge.data["beam_portion"]})
    }
    #[cfg(test)]
    pub(crate) fn trace_beam_rest_callbacks(&mut self) {
        self.beam_rest_callbacks = Some(Vec::new());
    }
    #[cfg(test)]
    fn observe_beam_rest_callback(&mut self, edge: usize, phase: &'static str) {
        if self.beam_rest_callbacks.is_none() {
            return;
        }
        let row = CallbackObservation {
            edge,
            phase,
            registration: self.beam_rest_registration(edge),
            snapshot: self.snapshot(),
            owners: self
                .vertices
                .iter()
                .filter(|v| {
                    !v.removed
                        && matches!(
                            v.data["kind"].as_str(),
                            Some("HeadChordInter" | "RestChordInter")
                        )
                })
                .map(|v| (v.key, v.voice_owner.clone(), v.slot_owner.clone()))
                .collect(),
        };
        if let Some(observations) = &mut self.beam_rest_callbacks {
            observations.push(row);
        }
    }
    #[cfg(test)]
    pub(crate) fn take_beam_rest_callbacks(&mut self) -> Vec<CallbackObservation> {
        self.beam_rest_callbacks
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }
    #[cfg(test)]
    pub(crate) fn edge_manual_status(&self, id: usize) -> Result<bool, String> {
        self.edges
            .get(
                id.checked_sub(1)
                    .ok_or("music_rest_relation_identity_required")?,
            )
            .map(|e| e.data["manual"] == true)
            .ok_or_else(|| "music_rest_relation_identity_required".into())
    }
    #[cfg(test)]
    pub(crate) fn invalidate_owned_rest_cache(&mut self, key: Key) -> Result<(), String> {
        let id = self.endpoint(key)?;
        if self.vertices[id - 1].data["kind"] != "RestChordInter" {
            return Err("music_rest_actual_chord_required".into());
        }
        self.invalidate_note_chords(&std::collections::BTreeSet::from([id]))
    }
}
