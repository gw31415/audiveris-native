// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_measures::OwnedMusic, music_voices::Voice, shared_sig::Key};
use std::sync::Arc;

impl OwnedMusic {
    fn actual_voice_mut(&mut self, identity: &Arc<()>) -> Result<&mut Voice, String> {
        if let Some(i) = self
            .voices
            .iter()
            .position(|v| Arc::ptr_eq(&v.identity, identity))
        {
            return Ok(&mut self.voices[i]);
        }
        self.owner
            .graph
            .retained_voice_owner_mut(identity)
            .ok_or_else(|| "music_actual_tuplet_voice_owner_required".into())
    }
    /// Voice.resetChords leaves registration, cached Chord backrefs and clocks intact.
    pub fn reset_rhythm_voice_chords(&mut self, identity: &Arc<()>) -> Result<(), String> {
        self.actual_voice_mut(identity)?.reset_chords();
        Ok(())
    }
    /// Direct Voice.removeChord does not perform Measure.removeVoiceChord's purge.
    pub fn remove_rhythm_voice_chord(
        &mut self,
        identity: &Arc<()>,
        key: Key,
    ) -> Result<bool, String> {
        let id = self
            .chords
            .iter()
            .find(|ch| ch.key == key)
            .ok_or("music_actual_tuplet_chord_owner_required")?
            .id;
        let voice = self.actual_voice_mut(identity)?;
        if !voice.chords_initialized {
            return Ok(false);
        }
        let Some(i) = voice.chord_ids.iter().position(|&owned| owned == id) else {
            return Ok(false);
        };
        voice.chord_ids.remove(i);
        Ok(true)
    }
    /// Direct Voice.addChord keeps actual encounter ties when sorting by center x.
    pub fn add_rhythm_voice_chord(&mut self, identity: &Arc<()>, key: Key) -> Result<(), String> {
        let ch = self
            .chords
            .iter()
            .find(|ch| ch.key == key)
            .ok_or("music_actual_tuplet_chord_owner_required")?;
        let id = ch.id;
        let voice = self
            .voices
            .iter()
            .find(|v| Arc::ptr_eq(&v.identity, identity))
            .or_else(|| self.owner.graph.retained_voice_owner(identity))
            .ok_or("music_actual_tuplet_voice_owner_required")?;
        if voice.chord_ids.contains(&id) {
            return Ok(());
        }
        let mut ids = voice.chord_ids.clone();
        ids.push(id);
        let mut centers = std::collections::BTreeMap::new();
        for &id in &ids {
            let chord = self
                .chords
                .iter()
                .find(|ch| ch.id == id)
                .ok_or("music_actual_tuplet_chord_owner_required")?;
            let b = self.owner.graph.retained_or_live_chord_bounds(chord.key)?;
            centers.insert(id, b[0] + b[2] / 2);
        }
        ids.sort_by_key(|id| centers[id]);
        let voice = self.actual_voice_mut(identity)?;
        voice.chord_ids = ids;
        voice.chords_initialized = true;
        Ok(())
    }
}
