// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only export access to real Head containment and live Slur relations.
use super::{Graph, Key, Phase};
use crate::music_slurs::lookup::Side;
impl Graph {
    pub(crate) fn slur_export_head(&self, chord: Key, note_id: usize) -> Result<Key, String> {
        let cid = self.endpoint(chord)?;
        let note = self
            .vertices
            .get(
                note_id
                    .checked_sub(1)
                    .ok_or("musicxml_slur_actual_Head_required")?,
            )
            .ok_or("musicxml_slur_actual_Head_required")?;
        if note.removed
            || note.key.phase != Phase::Head
            || note.data["kind"] != "HeadInter"
            || chord.system != note.key.system
            || self.vertices[cid - 1].data["kind"] != "HeadChordInter"
        {
            return Err("musicxml_slur_actual_Head_Chord_required".into());
        }
        if !self.incident_edges(cid).into_iter().any(|eid| {
            let e = &self.edges[eid - 1];
            e.key.kind == "Containment" && e.key.source == cid && e.key.target == note_id
        }) {
            return Err("musicxml_slur_actual_Head_Containment_required".into());
        }
        self.validate_chord_identity(cid, note_id, "Containment")?;
        Ok(note.key)
    }
    pub(crate) fn slur_export_incident(
        &self,
        head: Key,
    ) -> Result<Vec<(Key, usize, Side)>, String> {
        let hid = self.endpoint(head)?;
        let mut rows = Vec::new();
        for eid in self.incident_edges(hid) {
            let e = &self.edges[eid - 1];
            if e.key.kind != "SlurHeadRelation" {
                continue;
            }
            if e.key.target != hid || self.vertices[e.key.source - 1].data["kind"] != "SlurInter" {
                return Err("musicxml_slur_actual_relation_direction_required".into());
            }
            let side = match e.data["side"].as_str() {
                Some("LEFT") => Side::Left,
                Some("RIGHT") => Side::Right,
                _ => return Err("musicxml_slur_actual_relation_side_required".into()),
            };
            rows.push((self.vertices[e.key.source - 1].key, eid, side));
        }
        Ok(rows)
    }
    pub(crate) fn slur_export_relation(&self, id: usize) -> Result<&serde_json::Value, String> {
        let edge = self
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("musicxml_slur_actual_relation_id_required")?,
            )
            .ok_or("musicxml_slur_actual_relation_id_required")?;
        if edge.removed
            || edge.key.kind != "SlurHeadRelation"
            || self.vertices[edge.key.source - 1].removed
            || self.vertices[edge.key.target - 1].removed
        {
            return Err("musicxml_slur_live_relation_required".into());
        }
        Ok(&edge.data)
    }
    #[cfg(test)]
    pub(crate) fn declare_stale_slur_export_relation(&mut self, id: usize) {
        self.edges[id - 1].removed = true;
    }
}
