// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    grid_ir::StaffGeometry,
    music_measures::OwnedMusic,
    music_time::Fraction,
    shared_sig::{Key, TupletFactor},
};
use std::sync::Arc;

/// References retained by the actual Tuplet and its first incoming Chord.
#[derive(Debug)]
pub struct TupletRead {
    pub factor: Arc<TupletFactor>,
    pub(crate) base: Option<Arc<Fraction>>,
    pub(crate) staff: Option<Arc<StaffGeometry>>,
    pub part: Option<u64>,
    pub voice: Option<Arc<()>>,
    pub chords: Vec<Key>,
}
impl TupletRead {
    pub fn base_duration(&self) -> Option<String> {
        self.base
            .as_ref()
            .map(|b| crate::music_time::format_fraction(**b))
    }
    pub fn staff_id(&self) -> Option<u64> {
        self.staff.as_ref().map(|s| s.id)
    }
}
impl OwnedMusic {
    fn require_tuplet_owner(&self, key: Key) -> Result<(), String> {
        self.owner.graph.tuplet_status(key)?;
        if !self.measures.iter().any(|m| {
            m.system == key.system && (m.tuplets.contains(&key) || m.stack_tuplets.contains(&key))
        }) {
            return Err("music_actual_tuplet_container_required".into());
        }
        for chord in self.owner.graph.tuplet_chords(key)? {
            let ch = self
                .chords
                .iter()
                .find(|c| c.key == chord)
                .ok_or("music_actual_tuplet_chord_owner_required")?;
            let m = self
                .measures
                .get(ch.measure)
                .ok_or("music_actual_tuplet_measure_required")?;
            self.owner
                .graph
                .validate_tuplet_chord_container(chord, ch.id, m.part, m.index, ch.kind)?;
            if m.system != key.system {
                return Err("music_actual_tuplet_chord_owner_required".into());
            }
        }
        Ok(())
    }
    pub fn read_rhythm_tuplet(&mut self, key: Key) -> Result<TupletRead, String> {
        self.require_tuplet_owner(key)?;
        let mut graph = self.owner.graph.clone();
        let factor = graph.tuplet_duration_factor(key)?;
        let base = graph.tuplet_base_duration(key)?;
        let staff = graph.tuplet_staff(key, &self.owner.staffs)?;
        let part = graph.tuplet_part(key, &self.owner.staffs)?;
        let voice = graph.tuplet_voice_identity(key)?;
        let mut chords = Vec::new();
        for chord in graph.tuplet_chords(key)? {
            let bounds = graph.retained_or_live_chord_bounds(chord)?;
            chords.push((chord, bounds[0]));
        }
        chords.sort_by_key(|(_, x)| *x);
        let chords = chords.into_iter().map(|(key, _)| key).collect();
        self.owner.graph = graph;
        Ok(TupletRead {
            factor,
            base,
            staff,
            part,
            voice,
            chords,
        })
    }
    pub fn set_rhythm_tuplet_staff(&mut self, key: Key, staff: Option<u64>) -> Result<(), String> {
        self.require_tuplet_owner(key)?;
        let staff = staff
            .map(|id| {
                self.owner
                    .staffs
                    .iter()
                    .find(|s| s.system == key.system && s.id == id)
                    .cloned()
                    .ok_or("music_actual_tuplet_staff_required")
            })
            .transpose()?;
        self.owner.graph.set_tuplet_staff(key, staff)
    }
    pub fn remove_implicit_tuplet_link(&mut self, edge: usize) -> Result<(), String> {
        let key = self.owner.graph.chord_tuplet_target(edge)?;
        self.require_tuplet_owner(key)?;
        self.owner.graph.remove_owned_chord_tuplet(edge)
    }
}
