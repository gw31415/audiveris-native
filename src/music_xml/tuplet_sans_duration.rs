// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_measures::OwnedMusic, shared_sig::Key};
use std::sync::Arc;

impl OwnedMusic {
    /// Source dotted duration before any Tuplet effect; clocks are untouched.
    pub fn sans_tuplet_rhythm_chord_duration(
        &mut self,
        key: Key,
    ) -> Result<Option<String>, String> {
        let ch = self
            .chords
            .iter()
            .find(|ch| ch.key == key)
            .ok_or("music_actual_tuplet_chord_owner_required")?;
        let m = self
            .measures
            .get(ch.measure)
            .ok_or("music_actual_tuplet_measure_required")?;
        self.owner
            .graph
            .validate_tuplet_chord_container(key, ch.id, m.part, m.index, ch.kind)?;
        if key.system != m.system {
            return Err("music_actual_tuplet_chord_owner_required".into());
        }
        let mut graph = self.owner.graph.clone();
        let duration = graph.sans_tuplet_chord_duration(key, false)?;
        self.owner.graph = graph;
        Ok(duration.map(crate::music_time::format_fraction))
    }
    /// Read the actual current or retained Voice object, never a reused number.
    pub fn sans_tuplet_rhythm_voice_duration(
        &mut self,
        number: usize,
        identity: &Arc<()>,
    ) -> Result<Option<String>, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        let voice = self
            .voices
            .iter()
            .find(|v| Arc::ptr_eq(&v.identity, identity))
            .or_else(|| self.owner.graph.retained_voice_owner(identity))
            .ok_or("music_actual_tuplet_voice_owner_required")?;
        if voice.measure != mi {
            return Err("music_actual_tuplet_voice_owner_required".into());
        }
        if voice.measure_rest_chord.is_some() {
            return Ok(None);
        }
        if !voice.chords_initialized {
            return Err("music_tuplet_null_voice_chord_list".into());
        }
        let ids = voice.chord_ids.clone();
        let mut graph = self.owner.graph.clone();
        let mut sum = crate::music_time::Fraction::ZERO;
        for id in ids {
            let ch = self
                .chords
                .iter()
                .find(|ch| ch.id == id)
                .ok_or("music_actual_tuplet_chord_owner_required")?;
            let m = self
                .measures
                .get(ch.measure)
                .ok_or("music_actual_tuplet_measure_required")?;
            graph.validate_tuplet_chord_container(ch.key, ch.id, m.part, m.index, ch.kind)?;
            if ch.key.system != self.measures[mi].system || ch.key.system != m.system {
                return Err("music_actual_tuplet_chord_owner_required".into());
            }
            // A retained old Voice can still reference this actual Chord after
            // reset/reassignment. Its duration sum does not consult Chord.voice.
            sum = sum.plus(
                graph
                    .sans_tuplet_chord_duration(ch.key, false)?
                    .ok_or("music_tuplet_null_voice_chord_duration")?,
            )?;
        }
        self.owner.graph = graph;
        Ok(Some(crate::music_time::format_fraction(sum)))
    }
}
