// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Graph;
use crate::music_voices::Voice;
use std::sync::Arc;

impl Graph {
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
    pub(crate) fn retained_voice_owner(&self, identity: &Arc<()>) -> Option<&Voice> {
        self.retained_voices
            .iter()
            .find(|voice| Arc::ptr_eq(&voice.identity, identity))
    }
}
