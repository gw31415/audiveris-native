// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use crate::music_time::Fraction;

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct TupletRemovalObservation {
    pub edge: usize,
    pub tuplet: Key,
    pub phase: &'static str,
    pub measure_tuplets: Vec<Key>,
    pub stack_tuplets: Vec<Key>,
    pub state: serde_json::Value,
    pub snapshot: serde_json::Value,
    pub owners: super::beam_rest::CallbackOwners,
}

impl Graph {
    pub(crate) fn tuplet_chords(&self, key: Key) -> Result<Vec<Key>, String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        self.incident_edges(id)
            .into_iter()
            .filter(|eid| self.edges[*eid - 1].key.kind == "ChordTupletRelation")
            .map(|eid| {
                let edge = &self.edges[eid - 1];
                if edge.key.target != id {
                    return Err("music_actual_chord_tuplet_relation_required".into());
                }
                Ok(self.vertices[edge.key.source - 1].key)
            })
            .collect()
    }
    pub(crate) fn tuplet_status(&self, key: Key) -> Result<(bool, bool), String> {
        let id = self.endpoint(key)?;
        let v = &self.vertices[id - 1];
        if key.phase != Phase::Symbol
            || v.data["kind"] != "TupletInter"
            || !matches!(
                v.data["shape"].as_str(),
                Some("TUPLET_THREE" | "TUPLET_SIX")
            )
        {
            return Err("music_actual_tuplet_owner_required".into());
        }
        Ok((v.data["implicit"] == true, v.data["manual"] == true))
    }
    pub(super) fn exact_tuplet_duration(&self, id: usize) -> Result<Option<Fraction>, String> {
        let Some(mut duration) =
            crate::music_duration::live_intrinsic(&self.snapshot(), id)?.intrinsic
        else {
            return Ok(None);
        };
        if let Some(eid) = self
            .incident_edges(id)
            .into_iter()
            .find(|eid| self.edges[*eid - 1].key.kind == "ChordTupletRelation")
        {
            let edge = &self.edges[eid - 1];
            let tuplet = &self.vertices[edge.key.target - 1];
            if edge.key.source != id
                || tuplet.removed
                || tuplet.key.system != self.vertices[id - 1].key.system
            {
                return Err("music_actual_chord_tuplet_relation_required".into());
            }
            self.tuplet_status(tuplet.key)?;
            let factor = if tuplet.data["shape"] == "TUPLET_THREE" {
                Fraction::new(2, 3)?
            } else {
                Fraction::new(4, 6)?
            };
            duration = duration.times(factor)?;
        }
        Ok(Some(duration))
    }
    pub(crate) fn remove_owned_implicit_tuplet(
        &mut self,
        key: Key,
        measure: &[Key],
        stack: &[Key],
    ) -> Result<(), String> {
        if !self.tuplet_status(key)?.0 {
            return Err("music_actual_implicit_tuplet_required".into());
        }
        let id = self.endpoint(key)?;
        let incident = self.incident_edges(id);
        for &eid in &incident {
            let e = &self.edges[eid - 1];
            match e.key.kind.as_str() {
                "ChordTupletRelation" => {
                    let ch = &self.vertices[e.key.source - 1];
                    if e.key.target != id
                        || ch.removed
                        || ch.key.phase != Phase::Chord
                        || ch.key.system != key.system
                        || !matches!(
                            ch.data["kind"].as_str(),
                            Some("HeadChordInter" | "RestChordInter")
                        )
                    {
                        return Err("music_actual_chord_tuplet_relation_required".into());
                    }
                }
                "Exclusion" => {}
                _ => return Err("music_tuplet_extensive_removal_pending".into()),
            }
        }
        self.vertices[id - 1].removing = true;
        for eid in incident {
            self.edges[eid - 1].removed = true;
            if self.edges[eid - 1].key.kind == "ChordTupletRelation" {
                // Removed implicit tuplets skip checkAbnormal; old Chord clocks
                // and relation bodies survive while the vertex is registered.
                #[cfg(test)]
                for phase in ["entered", "finished"] {
                    self.observe_tuplet_removal(eid, key, phase, measure, stack);
                }
            }
        }
        self.vertices[id - 1].removing = false;
        self.vertices[id - 1].removed = true;
        #[cfg(not(test))]
        let _ = (measure, stack);
        Ok(())
    }
    #[cfg(test)]
    fn observe_tuplet_removal(
        &mut self,
        edge: usize,
        tuplet: Key,
        phase: &'static str,
        measure: &[Key],
        stack: &[Key],
    ) {
        let v = &self.vertices[self.edges[edge - 1].key.target - 1];
        let state = serde_json::json!({"registered":!v.removed,"removed":v.removing||v.removed,"manual":self.edges[edge-1].data["manual"]==true});
        let row = TupletRemovalObservation {
            edge,
            tuplet,
            phase,
            measure_tuplets: measure.to_vec(),
            stack_tuplets: stack.to_vec(),
            state,
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
        if let Some(rows) = &mut self.tuplet_removals {
            rows.push(row);
        }
    }
    #[cfg(test)]
    pub(crate) fn trace_tuplet_removals(&mut self) {
        self.tuplet_removals = Some(Vec::new());
    }
    #[cfg(test)]
    pub(crate) fn take_tuplet_removals(&mut self) -> Vec<TupletRemovalObservation> {
        self.tuplet_removals
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }
    #[cfg(test)]
    pub(crate) fn tuplet_registration(&self, key: Key) -> serde_json::Value {
        let v = &self.vertices[*self.keys.get(&key).unwrap() - 1];
        serde_json::json!({"registered":!v.removed,"removed":v.removing||v.removed,"implicit":v.data["implicit"]==true,"manual":v.data["manual"]==true})
    }
    #[cfg(test)]
    pub(crate) fn tuplet_relation_registration(&self, id: usize) -> serde_json::Value {
        let e = &self.edges[id - 1];
        serde_json::json!({"registered":!e.removed,"manual":e.data["manual"]==true})
    }
}
