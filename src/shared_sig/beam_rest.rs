// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use serde_json::json;
use std::sync::Arc;

impl Graph {
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
        let vertical = skew.vertical(point);
        let mut nearest = None;
        for edge in self
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.source == gid && e.key.kind == "Containment")
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
            let [x1, y1, x2, y2] = median;
            let [x3, y3, x4, y4] = vertical;
            let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
            let v12 = x1 * y2 - y1 * x2;
            let v34 = x3 * y4 - y3 * x4;
            let x = (v12 * (x3 - x4) - (x1 - x2) * v34) / den;
            let y = (v12 * (y3 - y4) - (y1 - y2) * v34) / den;
            if x >= x1 && x <= x2 {
                let distance = (point[0] - x).hypot(point[1] - y);
                if nearest.is_none_or(|(_, best)| distance < best) {
                    nearest = Some((beam.key, distance));
                }
            }
        }
        Ok(nearest)
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
    pub(crate) fn remove_owned_beam_rest(&mut self, id: usize) -> Result<(), String> {
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
        if edge.data["manual"] == true
            || beam.removed
            || rest.removed
            || !matches!(
                beam.data["kind"].as_str(),
                Some("BeamInter" | "BeamHookInter")
            )
            || rest.data["kind"] != "RestInter"
            || beam.key.system != rest.key.system
        {
            return Err("music_rest_automatic_typed_relation_required".into());
        }
        self.edges[id - 1].removed = true;
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
        Ok(())
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
