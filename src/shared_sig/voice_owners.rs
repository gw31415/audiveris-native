// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Graph;
use crate::music_voices::Voice;
use std::sync::Arc;

impl Graph {
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
    #[cfg(test)]
    pub(crate) fn retained_voice_owner(&self, identity: &Arc<()>) -> Option<&Voice> {
        self.retained_voices
            .iter()
            .find(|voice| Arc::ptr_eq(&voice.identity, identity))
    }
}
