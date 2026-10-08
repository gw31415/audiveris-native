// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Graph;
use crate::music_voices::Voice;
use std::sync::Arc;

impl Graph {
    pub(crate) fn actual_chord_time_offset(
        &self,
        key: super::Key,
    ) -> Result<Option<crate::music_time::Fraction>, String> {
        let id = self.endpoint(key)?;
        let chord = &self.vertices[id - 1];
        if key.phase != super::Phase::Chord
            || !matches!(
                chord.data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("music_tuplet_actual_chord_clock_required".into());
        }
        let value = &chord.data["time_offset"];
        if value.is_null() {
            return Ok(None);
        }
        Ok(Some(crate::music_time::Fraction::parse(
            value
                .as_str()
                .ok_or("music_tuplet_actual_chord_clock_required")?,
        )?))
    }
    pub(crate) fn set_chord_slot(
        &mut self,
        key: super::Key,
        id: usize,
        identity: &Arc<()>,
    ) -> Result<(), String> {
        let endpoint = self.endpoint(key)?;
        let chord = &mut self.vertices[endpoint - 1];
        if key.phase != super::Phase::Chord
            || !matches!(
                chord.data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("music_slot_actual_chord_required".into());
        }
        chord.data["slot_id"] = serde_json::json!(id);
        chord.slot_owner = Some(identity.clone());
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn chord_slot_identity(&self, key: super::Key) -> Option<&Arc<()>> {
        self.vertices[self.id(key)? - 1].slot_owner.as_ref()
    }
    pub(crate) fn chord_voice_owner<'a>(
        &'a self,
        key: super::Key,
        voices: &'a [Voice],
    ) -> Result<Option<&'a Voice>, String> {
        let id = self.endpoint(key)?;
        let chord = &self.vertices[id - 1];
        let Some(identity) = &chord.voice_owner else {
            return if chord.data["voice"].is_null() {
                Ok(None)
            } else {
                Err("music_refine_actual_chord_voice_required".into())
            };
        };
        let voice = voices
            .iter()
            .chain(&self.retained_voices)
            .find(|voice| Arc::ptr_eq(&voice.identity, identity))
            .ok_or("music_refine_actual_chord_voice_required")?;
        Ok(Some(voice))
    }
    pub(crate) fn retain_voice_owner(&mut self, voice: Voice) -> usize {
        let key = self
            .retained_voices
            .iter()
            .position(|old| Arc::ptr_eq(&old.identity, &voice.identity))
            .unwrap_or(self.retained_voices.len());
        for vertex in &mut self.vertices {
            if vertex
                .voice_owner
                .as_ref()
                .is_some_and(|identity| Arc::ptr_eq(identity, &voice.identity))
            {
                vertex.retained_voice = Some(key);
            }
        }
        // A restored Voice can leave the live list again. Its cached references
        // still identify the same owner, with the latest actual fields.
        if key == self.retained_voices.len() {
            self.retained_voices.push(voice);
        } else {
            self.retained_voices[key] = voice;
        }
        key
    }
    #[cfg(test)]
    pub(crate) fn cached_chord_manual_status(&self, key: super::Key) -> bool {
        let id = self.id(key).unwrap();
        assert_eq!(key.phase, super::Phase::Chord);
        self.vertices[id - 1].data["manual"] == true
    }
    pub(crate) fn retained_voice_owner(&self, identity: &Arc<()>) -> Option<&Voice> {
        self.retained_voices
            .iter()
            .find(|voice| Arc::ptr_eq(&voice.identity, identity))
    }
    pub(crate) fn retained_voice_owner_mut(&mut self, identity: &Arc<()>) -> Option<&mut Voice> {
        self.retained_voices
            .iter_mut()
            .find(|voice| Arc::ptr_eq(&voice.identity, identity))
    }
}
