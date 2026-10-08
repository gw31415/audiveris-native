// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    music_measure_slots::{ClockState, CompoundSlot},
    music_measures::OwnedMusic,
};
use serde_json::{Value, json};

/// Actual in-pass candidates, distinct from the later timed Stack Slots.
#[derive(Clone)]
pub struct RhythmCandidates {
    pub(super) number: usize,
    pub(super) system: u64,
    pub(super) part: u64,
    pub(super) measure: usize,
    pub(crate) compounds: Vec<CompoundSlot>,
    pub(crate) clocks: ClockState,
}
impl RhythmCandidates {
    #[cfg(test)]
    pub(crate) fn clock_observation(&self, clocks: &ClockState) -> Result<Value, String> {
        self.compounds
            .iter()
            .map(|s| clocks.projection(s))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| json!(v))
    }
    pub fn observe(&self) -> Result<Value, String> {
        Ok(json!(
            self.compounds
                .iter()
                .map(|slot| Ok(
                    json!({"compound":slot.projection(),"clocks":self.clocks.projection(slot)?})
                ))
                .collect::<Result<Vec<_>, String>>()?
        ))
    }
}
impl OwnedMusic {
    /// Construct through actual retained Chords and bind the new CompoundSlot objects.
    pub fn bind_rhythm_candidates(&mut self, number: usize) -> Result<RhythmCandidates, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        let m = &self.measures[mi];
        let chords = self
            .chords
            .iter()
            .filter(|ch| ch.measure == mi)
            .collect::<Vec<_>>();
        let (narrow, wide) = crate::music_measure_slots::from_owned(
            &self.owner.graph,
            m,
            &chords,
            self.owner.interline,
        )?;
        let centers = chords
            .iter()
            .map(|ch| (ch.id, ch.bounds[1] + ch.bounds[3] / 2))
            .collect();
        let compounds =
            crate::music_measure_slots::build_compounds(&narrow.slots, &wide.slots, &centers)?;
        let clocks = ClockState::new(&compounds)?;
        let mut graph = self.owner.graph.clone();
        for slot in &compounds {
            for &id in &slot.chord_ids {
                let ch = chords
                    .iter()
                    .find(|ch| ch.id == id)
                    .ok_or("music_candidate_actual_chord_required")?;
                graph.set_chord_slot(ch.key, slot.id, &slot.identity)?;
            }
        }
        self.owner.graph = graph;
        Ok(RhythmCandidates {
            number,
            system: m.system,
            part: m.part,
            measure: m.index,
            compounds,
            clocks,
        })
    }
}
