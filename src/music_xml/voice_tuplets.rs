// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_measures::OwnedMusic, music_voices::Voice, shared_sig::Key};
use std::sync::Arc;

impl OwnedMusic {
    fn actual_tuplet_voice(&self, number: usize, identity: &Arc<()>) -> Result<&Voice, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        self.voices
            .iter()
            .find(|v| Arc::ptr_eq(&v.identity, identity))
            .or_else(|| self.owner.graph.retained_voice_owner(identity))
            .filter(|v| v.measure == mi)
            .ok_or_else(|| "music_actual_tuplet_voice_owner_required".into())
    }
    fn actual_tuplet_voice_chord(&self, id: usize) -> Result<Key, String> {
        let ch = self
            .chords
            .iter()
            .find(|ch| ch.id == id)
            .ok_or("music_actual_tuplet_chord_owner_required")?;
        let m = self
            .measures
            .get(ch.measure)
            .ok_or("music_actual_tuplet_measure_required")?;
        self.owner
            .graph
            .validate_tuplet_chord_container(ch.key, ch.id, m.part, m.index, ch.kind)?;
        Ok(ch.key)
    }
    /// Voice.getTuplets sees only each Chord's first relation, deduplicated by actual Tuplet.
    pub fn rhythm_voice_tuplets(
        &self,
        number: usize,
        identity: &Arc<()>,
    ) -> Result<Vec<Key>, String> {
        let voice = self.actual_tuplet_voice(number, identity)?;
        if !voice.chords_initialized {
            return Err("music_tuplet_null_voice_chord_list".into());
        }
        let mut found = Vec::new();
        for &id in &voice.chord_ids {
            if let Some(tuplet) = self
                .owner
                .graph
                .first_chord_tuplet(self.actual_tuplet_voice_chord(id)?)?
                && !found.contains(&tuplet)
            {
                found.push(tuplet);
            }
        }
        Ok(found)
    }
    /// Any ChordTupletRelation counts here, including an explicit/manual one.
    pub fn last_rhythm_voice_tuplet_chord(
        &self,
        number: usize,
        identity: &Arc<()>,
    ) -> Result<Option<Key>, String> {
        let voice = self.actual_tuplet_voice(number, identity)?;
        let mut last = None;
        for &id in &voice.chord_ids {
            let key = self.actual_tuplet_voice_chord(id)?;
            if self.owner.graph.chord_has_tuplet_relation(key)? {
                last = Some(key);
            }
        }
        Ok(last)
    }
    /// Source indexOf returns -1 for an absent/null Chord, so its successor is first.
    pub fn rhythm_voice_chord_after(
        &self,
        number: usize,
        identity: &Arc<()>,
        input: Option<Key>,
    ) -> Result<Option<Key>, String> {
        let voice = self.actual_tuplet_voice(number, identity)?;
        if voice.chord_ids.is_empty() {
            return Ok(None);
        }
        let id = input
            .map(|key| {
                self.chords
                    .iter()
                    .find(|ch| ch.key == key)
                    .map(|ch| ch.id)
                    .ok_or("music_actual_tuplet_chord_owner_required")
            })
            .transpose()?;
        let index = id
            .and_then(|id| voice.chord_ids.iter().position(|&ch| ch == id))
            .map_or(0, |i| i + 1);
        voice
            .chord_ids
            .get(index)
            .map(|&id| self.actual_tuplet_voice_chord(id))
            .transpose()
    }
    pub fn rhythm_voices_with_implicit_tuplets(
        &self,
        number: usize,
    ) -> Result<Vec<Arc<()>>, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        let mut found = Vec::new();
        for voice in self.voices.iter().filter(|v| v.measure == mi) {
            for &id in &voice.chord_ids {
                let key = self.actual_tuplet_voice_chord(id)?;
                if let Some(tuplet) = self.owner.graph.first_chord_tuplet(key)?
                    && self.owner.graph.tuplet_status(tuplet)?.0
                {
                    if !found
                        .iter()
                        .any(|identity| Arc::ptr_eq(identity, &voice.identity))
                    {
                        found.push(voice.identity.clone());
                    }
                    break;
                }
            }
        }
        Ok(found)
    }
    /// Pure Measure.purgeVoices retains objects referenced by old Chords without renumbering.
    pub fn purge_rhythm_measure_voices(&mut self, number: usize) -> Result<(), String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        crate::music_voices::purge_measure_voices(&mut self.owner.graph, &mut self.voices, mi);
        Ok(())
    }
}
