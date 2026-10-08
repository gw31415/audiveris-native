// SPDX-License-Identifier: AGPL-3.0-or-later
use super::RhythmCandidates;
use crate::{
    music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, music_time::Fraction,
    shared_sig::Key,
};
use std::sync::Arc;

impl OwnedMusic {
    /// Explicit actual Voice/candidate inputs isolate shrink before ordinary resource binding.
    pub fn shrink_rhythm_voice(
        &mut self,
        candidates: &mut RhythmCandidates,
        identity: &Arc<()>,
        stop: Option<Key>,
        last_sync: Option<&str>,
        font: &TupletFontSymbols,
    ) -> Result<Vec<Key>, String> {
        let number = candidates.number;
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        let m = &self.measures[mi];
        if (m.system, m.part, m.index) != (candidates.system, candidates.part, candidates.measure) {
            return Err("music_tuplet_actual_candidate_measure_required".into());
        }
        let voice = self
            .voices
            .iter()
            .find(|v| Arc::ptr_eq(&v.identity, identity))
            .or_else(|| self.owner.graph.retained_voice_owner(identity))
            .filter(|v| v.measure == mi)
            .ok_or("music_actual_tuplet_voice_owner_required")?;
        let chords = voice
            .chord_ids
            .iter()
            .map(|id| {
                self.chords
                    .iter()
                    .find(|ch| ch.id == *id)
                    .map(|ch| ch.key)
                    .ok_or("music_actual_tuplet_chord_owner_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sync = last_sync.map(Fraction::parse).transpose()?;
        let mut first = 0;
        for (i, &key) in chords.iter().enumerate() {
            let time = self
                .owner
                .graph
                .actual_chord_time_offset(key)?
                .ok_or("music_tuplet_null_shrink_chord_time")?;
            if Some(time) == sync {
                first = i;
                break;
            }
        }
        let end = stop
            .and_then(|stop| chords.iter().position(|&ch| ch == stop))
            .unwrap_or(chords.len());
        let group = chords
            .get(first..end)
            .ok_or("music_tuplet_shrink_group_range_required")?;
        if group.is_empty() {
            return Ok(Vec::new());
        }
        let mut clocks = candidates.clocks.clone();
        let result = self.with_tuplet_owners(|owner| {
            let plan = owner.extend_rhythm_tuplet_group(number, group, &[])?;
            let extended = plan.extended;
            let mut generated = Vec::new();
            for group in plan.groups {
                generated.push(owner.generate_rhythm_tuplet(number, &group, font)?);
            }
            let mut previous = None;
            for &key in &extended {
                if let Some(prev) = previous {
                    let time = owner
                        .owner
                        .graph
                        .chord_end_time(prev)?
                        .as_deref()
                        .map(Fraction::parse)
                        .transpose()?
                        .ok_or("music_tuplet_null_shrink_chord_end")?;
                    let id = owner
                        .owner
                        .graph
                        .id(key)
                        .ok_or("music_actual_tuplet_chord_owner_required")?;
                    if owner.owner.graph.actual_chord_time_offset(key)? != Some(time) {
                        crate::music_chord_time::set_and_push(
                            &mut owner.owner.graph,
                            &owner.voices,
                            &owner.measures,
                            id,
                            Some(time),
                        )?;
                    }
                    // The original pushes Chord clocks before dereferencing its
                    // CompoundSlot and updating the first containing narrow member.
                    let slot = owner
                        .owner
                        .graph
                        .chord_slot_identity(key)
                        .ok_or("music_tuplet_null_shrink_slot")?;
                    let compound = candidates
                        .compounds
                        .iter()
                        .find(|s| Arc::ptr_eq(&s.identity, slot))
                        .ok_or("music_tuplet_actual_compound_slot_required")?;
                    let narrow = compound
                        .narrow_owner(id)
                        .ok_or("music_tuplet_actual_narrow_slot_required")?;
                    clocks.set_owner(narrow, Some(time))?;
                }
                previous = Some(key);
            }
            if let (Some(prev), Some(stop)) = (previous, stop) {
                let time = owner
                    .owner
                    .graph
                    .chord_end_time(prev)?
                    .as_deref()
                    .map(Fraction::parse)
                    .transpose()?
                    .ok_or("music_tuplet_null_shrink_chord_end")?;
                if owner.owner.graph.actual_chord_time_offset(stop)? != Some(time) {
                    let id = owner
                        .owner
                        .graph
                        .id(stop)
                        .ok_or("music_actual_tuplet_chord_owner_required")?;
                    crate::music_chord_time::set_and_push(
                        &mut owner.owner.graph,
                        &owner.voices,
                        &owner.measures,
                        id,
                        Some(time),
                    )?;
                }
            }
            Ok(generated)
        });
        if result.is_ok() {
            candidates.clocks = clocks;
        }
        result
    }
}
