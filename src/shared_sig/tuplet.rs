// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use crate::music_time::Fraction;
use std::{
    cell::{OnceCell, RefCell},
    sync::Arc,
};

// DurationFactor keeps actual4/6 identity even when arithmetic reduces it to2/3.
#[derive(Debug)]
pub struct TupletFactor {
    pub actual_num: u32,
    pub actual_den: u32,
}
#[derive(Clone, Debug, Default)]
pub(super) struct TupletCache {
    factor: OnceCell<Arc<TupletFactor>>,
    base: RefCell<Option<Arc<Fraction>>>,
    staff: Option<Arc<crate::grid_ir::StaffGeometry>>,
}

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
    pub(crate) fn tuplet_duration_factor(&self, key: Key) -> Result<Arc<TupletFactor>, String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        let cache = self.vertices[id - 1]
            .tuplet_cache
            .as_ref()
            .ok_or("music_actual_tuplet_cache_required")?;
        Ok(cache
            .factor
            .get_or_init(|| {
                Arc::new(if self.vertices[id - 1].data["shape"] == "TUPLET_THREE" {
                    TupletFactor {
                        actual_num: 2,
                        actual_den: 3,
                    }
                } else {
                    TupletFactor {
                        actual_num: 4,
                        actual_den: 6,
                    }
                })
            })
            .clone())
    }
    pub(crate) fn tuplet_base_duration(
        &mut self,
        key: Key,
    ) -> Result<Option<Arc<Fraction>>, String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        let cache = self.vertices[id - 1]
            .tuplet_cache
            .as_ref()
            .ok_or("music_actual_tuplet_cache_required")?;
        if cache.base.borrow().is_some() {
            return Ok(cache.base.borrow().clone());
        }
        for chord in self.tuplet_chords(key)? {
            let raw = self.raw_chord_duration(chord)?;
            let mut base = self.vertices[id - 1]
                .tuplet_cache
                .as_ref()
                .unwrap()
                .base
                .borrow_mut();
            match (base.as_ref(), raw) {
                (None, raw) => *base = raw.map(Arc::new),
                (Some(_), None) => return Err("music_tuplet_null_raw_duration_after_base".into()),
                (Some(old), Some(raw)) if **old > raw => *base = Some(Arc::new(raw)),
                _ => {}
            }
        }
        let base = self.vertices[id - 1]
            .tuplet_cache
            .as_ref()
            .unwrap()
            .base
            .borrow()
            .clone();
        Ok(base)
    }
    pub(crate) fn tuplet_voice_identity(&self, key: Key) -> Result<Option<Arc<()>>, String> {
        let Some(chord) = self.tuplet_chords(key)?.first().copied() else {
            return Ok(None);
        };
        let id = self.endpoint(chord)?;
        // The first actual source's null Voice is the result, not a reason to
        // choose a later source or reuse a musical number.
        Ok(self.vertices[id - 1].voice_owner.clone())
    }
    pub(crate) fn tuplet_staff(
        &mut self,
        key: Key,
        staffs: &[Arc<crate::grid_ir::StaffGeometry>],
    ) -> Result<Option<Arc<crate::grid_ir::StaffGeometry>>, String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        let cache = self.vertices[id - 1]
            .tuplet_cache
            .as_ref()
            .ok_or("music_actual_tuplet_cache_required")?;
        if cache.staff.is_some() {
            return Ok(cache.staff.clone());
        }
        if let Some(staff) = self.vertices[id - 1].data["staff_id"].as_u64() {
            let owner = staffs
                .iter()
                .find(|s| s.system == key.system && s.id == staff)
                .ok_or("music_actual_tuplet_staff_required")?
                .clone();
            self.vertices[id - 1].tuplet_cache.as_mut().unwrap().staff = Some(owner.clone());
            return Ok(Some(owner));
        }
        for chord in self.tuplet_chords(key)? {
            let cid = self.endpoint(chord)?;
            let mut staff = self.vertices[cid - 1].data["staff_id"].as_u64();
            if staff.is_none() {
                let mut notes = self
                    .incident_edges(cid)
                    .into_iter()
                    .filter_map(|eid| {
                        let e = &self.edges[eid - 1];
                        (e.key.source == cid && e.key.kind == "Containment").then_some(e.key.target)
                    })
                    .collect::<Vec<_>>();
                let mut centers = std::collections::BTreeMap::new();
                for &note in &notes {
                    let n = &self.vertices[note - 1];
                    if n.removed
                        || n.key.system != key.system
                        || !matches!(n.data["kind"].as_str(), Some("HeadInter" | "RestInter"))
                    {
                        return Err("music_actual_tuplet_staff_note_required".into());
                    }
                    let b: [i64; 4] = serde_json::from_value(n.data["bounds"].clone())
                        .map_err(|_| "music_actual_tuplet_staff_note_required")?;
                    centers.insert(note, b[1] + b[3] / 2);
                }
                notes.sort_by_key(|n| std::cmp::Reverse(centers[n]));
                if let (Some(first), Some(last)) = (notes.first(), notes.last()) {
                    let bottom = self.vertices[*first - 1].data["staff_id"].as_u64();
                    let top = self.vertices[*last - 1].data["staff_id"].as_u64();
                    if bottom == top {
                        staff = top;
                        self.vertices[cid - 1].data["staff_id"] = serde_json::json!(staff);
                    }
                }
            }
            if let Some(staff) = staff {
                let owner = staffs
                    .iter()
                    .find(|s| s.system == key.system && s.id == staff)
                    .ok_or("music_actual_tuplet_staff_required")?
                    .clone();
                self.vertices[id - 1].data["staff_id"] = serde_json::json!(staff);
                self.vertices[id - 1].tuplet_cache.as_mut().unwrap().staff = Some(owner.clone());
                return Ok(Some(owner));
            }
        }
        Ok(None)
    }
    pub(crate) fn set_tuplet_staff(
        &mut self,
        key: Key,
        staff: Option<Arc<crate::grid_ir::StaffGeometry>>,
    ) -> Result<(), String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        if staff.as_ref().is_some_and(|s| s.system != key.system) {
            return Err("music_actual_tuplet_staff_required".into());
        }
        self.vertices[id - 1].data["staff_id"] = serde_json::json!(staff.as_ref().map(|s| s.id));
        self.vertices[id - 1].tuplet_cache.as_mut().unwrap().staff = staff;
        // AbstractInter.setStaff leaves an already cached Part untouched.
        Ok(())
    }
    pub(crate) fn tuplet_part(&mut self, key: Key) -> Result<Option<u64>, String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        // Inherited getPart reads the existing Staff field; it must not invoke
        // virtual getStaff and change Stack.addInter's unassigned boundary.
        self.part_for_inter(id)
    }
    pub(crate) fn validate_tuplet_chord_container(
        &self,
        key: Key,
        id: usize,
        part: u64,
        index: usize,
        kind: crate::music_measures::ChordKind,
    ) -> Result<(), String> {
        if self.endpoint(key)? != id {
            return Err("music_actual_tuplet_chord_owner_required".into());
        }
        let node = &self.vertices[id - 1].data;
        if node["part_id"] != part
            || node["measure_index"] != index
            || node["kind"]
                != match kind {
                    crate::music_measures::ChordKind::Head => "HeadChordInter",
                    crate::music_measures::ChordKind::Rest => "RestChordInter",
                }
        {
            return Err("music_actual_tuplet_chord_owner_required".into());
        }
        Ok(())
    }
    pub(crate) fn chord_tuplet_target(&self, eid: usize) -> Result<Key, String> {
        let e = self
            .edges
            .get(
                eid.checked_sub(1)
                    .ok_or("music_actual_chord_tuplet_relation_required")?,
            )
            .ok_or("music_actual_chord_tuplet_relation_required")?;
        if e.key.kind != "ChordTupletRelation" {
            return Err("music_actual_chord_tuplet_relation_required".into());
        }
        Ok(self.vertices[e.key.target - 1].key)
    }
    pub(crate) fn remove_owned_chord_tuplet(&mut self, eid: usize) -> Result<(), String> {
        let e = self
            .edges
            .get(
                eid.checked_sub(1)
                    .ok_or("music_actual_chord_tuplet_relation_required")?,
            )
            .ok_or("music_actual_chord_tuplet_relation_required")?;
        if e.key.kind != "ChordTupletRelation" {
            return Err("music_actual_chord_tuplet_relation_required".into());
        }
        if e.removed {
            return Ok(());
        }
        let key = self.vertices[e.key.target - 1].key;
        if !self.tuplet_status(key)?.0 {
            return Err("music_tuplet_explicit_relation_removal_pending".into());
        }
        self.edges[eid - 1].removed = true;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn tuplet_cache_observation(&self, key: Key) -> serde_json::Value {
        let c = self.vertices[self.id(key).unwrap() - 1]
            .tuplet_cache
            .as_ref()
            .unwrap();
        serde_json::json!({"factor":c.factor.get().map(|f|[f.actual_num,f.actual_den]),"base":c.base.borrow().as_ref().map(|b|**b),"staff":c.staff.as_ref().map(|s|s.id)})
    }
    pub(crate) fn first_chord_tuplet(&self, key: Key) -> Result<Option<Key>, String> {
        let id = self.endpoint(key)?;
        let chord = &self.vertices[id - 1];
        if key.phase != Phase::Chord
            || !matches!(
                chord.data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("music_actual_tuplet_chord_required".into());
        }
        for eid in self.incident_edges(id) {
            let edge = &self.edges[eid - 1];
            if edge.key.kind == "ChordTupletRelation" {
                if edge.key.source != id {
                    return Err("music_actual_chord_tuplet_relation_required".into());
                }
                let tuplet = self.vertices[edge.key.target - 1].key;
                if tuplet.system != key.system {
                    return Err("music_actual_chord_tuplet_relation_required".into());
                }
                self.tuplet_status(tuplet)?;
                return Ok(Some(tuplet));
            }
        }
        Ok(None)
    }
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
                let chord = &self.vertices[edge.key.source - 1];
                if chord.removed
                    || chord.key.system != key.system
                    || chord.key.phase != Phase::Chord
                    || !matches!(
                        chord.data["kind"].as_str(),
                        Some("HeadChordInter" | "RestChordInter")
                    )
                {
                    return Err("music_actual_chord_tuplet_relation_required".into());
                }
                Ok(chord.key)
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
            let cached = self.tuplet_duration_factor(tuplet.key)?;
            let factor = Fraction::new(cached.actual_num, cached.actual_den)?;
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
