// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{EditingLink, EditingOperation, Key};
use crate::music_measures::{Chord, ChordKind, OwnedMusic};
use serde_json::Value;

/// An actual editing HeadChord can exist before it has any Part or Measure.
/// Its nullable geometry and lazy placement remain valid through raw undo.
#[derive(Clone, Debug)]
pub(crate) struct EditingHeadChord {
    pub key: Key,
    pub id: usize,
    pub stem: Option<Key>,
    pub members: Vec<Value>,
    pub bounds: Option<[i64; 4]>,
    pub measure: Option<usize>,
    pub in_measure: bool,
}

#[derive(Clone, Debug)]
pub struct ChordAdditionTask {
    key: Key,
    initial_bounds: [i64; 4],
}
impl ChordAdditionTask {
    pub(super) fn new(owner: &OwnedMusic, stem: Key) -> Result<Self, String> {
        let graph = &owner.owner.graph;
        let id = graph.endpoint(stem)?;
        let bounds = serde_json::from_value(graph.vertices[id - 1].data["bounds"].clone())
            .map_err(|_| "editing_stem_bounds_required")?;
        let local = graph
            .vertices
            .iter()
            .filter(|v| v.key.system == stem.system && v.key.phase == super::Phase::Chord)
            .map(|v| v.key.local)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("editing_chord_identity_budget")?;
        Ok(Self {
            key: Key {
                system: stem.system,
                phase: super::Phase::Chord,
                local,
            },
            initial_bounds: bounds,
        })
    }
    pub(super) fn key(&self) -> Key {
        self.key
    }
    pub(super) fn perform_owned(
        &mut self,
        owner: &mut OwnedMusic,
        op: EditingOperation,
    ) -> Result<(), String> {
        let graph = &mut owner.owner.graph;
        if matches!(op, EditingOperation::Undo) {
            let index = graph
                .editing_chords
                .iter()
                .position(|c| c.key == self.key)
                .ok_or("editing_added_chord_required")?;
            let id = graph.endpoint(self.key)?;
            // This raw task closure reverses Containment and ChordStem first.
            // Extensive child deletion requires a separately compared closure.
            if !graph.editing_chords[index].members.is_empty()
                || !graph.incident_edges(id).is_empty()
            {
                return Err("editing_empty_added_chord_required".into());
            }
            graph.editing_chords[index].in_measure = false;
            graph.vertices[id - 1].removed = true;
            #[cfg(test)]
            graph.dot_callback_event("vertexRemoved", id, "enter");
            #[cfg(test)]
            graph.dot_callback_event("vertexRemoved", id, "leave");
            return Ok(());
        }
        let first = graph.id(self.key).is_none();
        let id = if first {
            let id = graph.vertex(self.key, serde_json::json!({"kind":"HeadChordInter", "grade":null, "bounds":self.initial_bounds}), false)?;
            graph.vertices[id - 1].chord_cache = Some(super::chord_cache::ChordCache::default());
            graph.editing_chords.push(EditingHeadChord {
                key: self.key,
                id,
                stem: None,
                members: Vec::new(),
                bounds: Some(self.initial_bounds),
                measure: None,
                in_measure: false,
            });
            id
        } else {
            let id = graph.id(self.key).unwrap();
            if !graph.vertices[id - 1].removed {
                return Err("editing_removed_added_chord_required".into());
            }
            graph.vertices[id - 1].removed = false;
            graph.vertices[id - 1].removing = true;
            id
        };
        graph.vertices[id - 1].data["bounds"] = serde_json::json!(self.initial_bounds);
        graph.vertices[id - 1]
            .chord_cache
            .as_mut()
            .unwrap()
            .set_bounds(self.initial_bounds);
        graph
            .editing_chords
            .iter_mut()
            .find(|c| c.key == self.key)
            .unwrap()
            .bounds = Some(self.initial_bounds);
        #[cfg(test)]
        graph.dot_callback_event("vertexAdded", id, "enter");
        #[cfg(test)]
        graph.dot_callback_event("vertexAdded", id, "leave");
        graph.vertices[id - 1].removing = false;
        let chord = graph
            .editing_chords
            .iter_mut()
            .find(|c| c.key == self.key)
            .unwrap();
        chord.bounds = Some(self.initial_bounds);
        // A first addition is unplaced; redo uses cached placement only after
        // the SIG listeners, just as AbstractChord.added calls Measure.addInter.
        chord.in_measure = chord.measure.is_some();
        Ok(())
    }
}

/// Retain the actual Chord and its surviving links across nonextensive removal.
/// Head removal and extensive ensemble deletion require separate task closure.
#[derive(Clone, Debug)]
pub struct ChordRemovalTask {
    key: Key,
    initial_bounds: [i64; 4],
    links: Option<Vec<EditingLink>>,
    retained: Option<Chord>,
}
impl ChordRemovalTask {
    pub fn new(owner: &OwnedMusic, key: Key) -> Result<Self, String> {
        let chord = owner
            .chords
            .iter()
            .find(|c| c.key == key)
            .filter(|c| c.kind == ChordKind::Head)
            .ok_or("editing_actual_head_chord_required")?;
        owner.owner.graph.endpoint(key)?;
        Ok(Self {
            key,
            initial_bounds: chord.bounds,
            links: None,
            retained: None,
        })
    }
    pub(super) fn key(&self) -> Key {
        self.key
    }
    pub(super) fn perform_owned(
        &mut self,
        owner: &mut OwnedMusic,
        op: EditingOperation,
    ) -> Result<(), String> {
        if matches!(op, EditingOperation::Undo) {
            let chord = self
                .retained
                .take()
                .ok_or("editing_removed_chord_required")?;
            let graph = &mut owner.owner.graph;
            let id = graph
                .id(self.key)
                .ok_or("editing_chord_identity_required")?;
            if !graph.vertices[id - 1].removed || owner.chords.iter().any(|c| c.key == self.key) {
                return Err("editing_removed_chord_required".into());
            }
            graph.vertices[id - 1].data["bounds"] = serde_json::json!(self.initial_bounds);
            graph.vertices[id - 1]
                .chord_cache
                .as_mut()
                .ok_or("editing_retained_chord_cache_required")?
                .set_bounds(self.initial_bounds);
            graph.vertices[id - 1].removed = false;
            graph.vertices[id - 1].removing = true;
            #[cfg(test)]
            graph.dot_callback_event("vertexAdded", id, "enter");
            #[cfg(test)]
            graph.dot_callback_event("vertexAdded", id, "leave");
            // SIGraph.addVertex fires listeners before Inter.added clears the
            // removed flag and registers the chord back into its Measure.
            graph.vertices[id - 1].removing = false;
            // Measure.addInter appends the same chord. It does not put it back
            // into the old Voice list or erase its retained Voice/Slot clocks.
            owner.chords.push(chord);
            for link in self
                .links
                .as_mut()
                .ok_or("editing_removal_links_required")?
            {
                link.apply_relation(graph, self.key, true)?;
            }
            owner.sync_task_chord(self.key)?;
        } else {
            let index = owner
                .chords
                .iter()
                .position(|c| c.key == self.key)
                .ok_or("editing_actual_head_chord_required")?;
            let graph = &mut owner.owner.graph;
            let id = graph.endpoint(self.key)?;
            let edges = graph.incident_edges(id);
            if edges.iter().any(|eid| {
                let e = &graph.edges[*eid - 1];
                e.key.source != id
                    || !matches!(e.key.kind.as_str(), "ChordStemRelation" | "Containment")
            }) {
                return Err("editing_empty_chord_removal_required".into());
            }
            if self.links.is_none() {
                self.links = Some(
                    edges
                        .iter()
                        .map(|id| EditingLink::from_registered(graph, self.key, *id))
                        .collect::<Result<_, _>>()?,
                );
            }
            let chord = owner.chords.remove(index);
            let mut voice_index = 0;
            while voice_index < owner.voices.len() {
                let voice = &mut owner.voices[voice_index];
                if voice.measure != chord.measure {
                    voice_index += 1;
                    continue;
                }
                let Some(member) = voice.chord_ids.iter().position(|cid| *cid == id) else {
                    voice_index += 1;
                    continue;
                };
                voice.chord_ids.remove(member);
                if !voice.chord_ids.is_empty() {
                    voice_index += 1;
                    continue;
                }
                // Measure.removeInter removes an empty Voice from its list;
                // removed Chords still refer to that actual Voice and slots.
                let voice = owner.voices.remove(voice_index);
                graph.retain_voice_owner(voice);
                // Original removeVoiceChord stops after the first Voice becomes
                // empty; subsequent overlapping lists remain for the next task.
                break;
            }
            self.retained = Some(chord);
            graph.vertices[id - 1].removing = true;
            graph.remove_chord_identity_edges(&edges)?;
            graph.vertices[id - 1].removed = true;
            graph.vertices[id - 1].removing = false;
            #[cfg(test)]
            graph.dot_callback_event("vertexRemoved", id, "enter");
            #[cfg(test)]
            graph.dot_callback_event("vertexRemoved", id, "leave");
        }
        Ok(())
    }
}
impl OwnedMusic {
    pub(super) fn sync_task_chord(&mut self, key: Key) -> Result<(), String> {
        let graph = &mut self.owner.graph;
        let id = graph.endpoint(key)?;
        let mut members = graph
            .incident_edges(id)
            .into_iter()
            .filter_map(|eid| {
                let edge = &graph.edges[eid - 1];
                (edge.key.source == id && edge.key.kind == "Containment")
                    .then(|| graph.vertices[edge.key.target - 1].data.clone())
            })
            .collect::<Vec<Value>>();
        members.sort_by_key(|h| {
            std::cmp::Reverse(
                h["bounds"][1].as_i64().unwrap() + h["bounds"][3].as_i64().unwrap() / 2,
            )
        });
        let read = graph.retained_chord_getters(key)?;
        if let Some(chord) = graph.editing_chords.iter_mut().find(|c| c.key == key) {
            if chord.id != id {
                return Err("editing_chord_identity_changed".into());
            }
            // Placement belongs to the relation callback. This post-task read
            // verifies/synchronizes geometry without manufacturing membership.
            chord.members = members;
            chord.stem = read.stem;
            chord.bounds = read.bounds;
            return Ok(());
        }
        let chord = self
            .chords
            .iter_mut()
            .find(|c| c.key == key)
            .ok_or("editing_actual_music_chord_required")?;
        chord.members = members;
        chord.stem = read.stem;
        if let Some(bounds) = read.bounds {
            chord.bounds = bounds;
        }
        Ok(())
    }
}

impl super::Graph {
    pub(crate) fn chord_has_voice(&self, key: Key, voice: &crate::music_voices::Voice) -> bool {
        self.id(key).is_some_and(|id| {
            self.vertices[id - 1]
                .voice_owner
                .as_ref()
                .is_some_and(|identity| std::sync::Arc::ptr_eq(identity, &voice.identity))
        })
    }
    pub(crate) fn bind_chord_to_live_voice(
        &mut self,
        key: Key,
        voice: &crate::music_voices::Voice,
    ) -> Result<(), String> {
        let id = self.endpoint(key)?;
        if key.phase != super::Phase::Chord || self.vertices[id - 1].data["voice"] != voice.id {
            return Err("music_voice_actual_chord_required".into());
        }
        self.vertices[id - 1].retained_voice = None;
        self.vertices[id - 1].voice_owner = Some(voice.identity.clone());
        Ok(())
    }
    pub(crate) fn rename_voice_references(
        &mut self,
        voice: &crate::music_voices::Voice,
        id: usize,
    ) -> Result<(), String> {
        // Voice.setId changes its object, including cached Chord references
        // absent from its list; old list entries can point to another Voice.
        let bound = |v: &super::Vertex| {
            v.voice_owner
                .as_ref()
                .is_some_and(|identity| std::sync::Arc::ptr_eq(identity, &voice.identity))
        };
        if self
            .vertices
            .iter()
            .any(|v| bound(v) && v.data["voice"] != voice.id)
        {
            return Err("music_refine_ID_actual_chord_voice_required".into());
        }
        for v in self.vertices.iter_mut().filter(|v| bound(v)) {
            v.data["voice"] = serde_json::json!(id);
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn retained_voice_observations(&self) -> Value {
        serde_json::json!(self.retained_voices.iter().enumerate().map(|(key,v)|serde_json::json!({
            "key":key,"id":v.id,"measure":v.measure,"chord_ids":v.chord_ids,
            "termination":v.termination.map(crate::music_time::format_delta),
            "excess":v.excess.map(crate::music_time::format_fraction),
            "slots":v.slots.iter().map(|(id,s)|serde_json::json!({"id":id,"chord_id":s.chord_id,"status":if s.continuing {"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>()
        })).collect::<Vec<_>>())
    }
    #[cfg(test)]
    pub(super) fn editing_chord_observations(&self) -> Value {
        serde_json::json!(self.editing_chords.iter().map(|c| serde_json::json!({
            "id":c.id,"member_ids":c.members.iter().map(|h|h["id"].clone()).collect::<Vec<_>>(),
            "stem_id":c.stem.and_then(|key|self.id(key)),"bounds":c.bounds,
            "measure":c.measure,"in_measure":c.in_measure
        })).collect::<Vec<_>>())
    }
    pub(super) fn refresh_editing_chord_links(&mut self, id: usize) {
        let Some(index) = self.editing_chords.iter().position(|c| c.id == id) else {
            return;
        };
        let mut members = Vec::new();
        let mut stem = None;
        for eid in self.incident_edges(id) {
            let edge = &self.edges[eid - 1];
            if edge.key.source != id {
                continue;
            }
            if edge.key.kind == "Containment" {
                members.push(self.vertices[edge.key.target - 1].data.clone());
            }
            if edge.key.kind == "ChordStemRelation" && stem.is_none() {
                stem = Some(self.vertices[edge.key.target - 1].key);
            }
        }
        members.sort_by_key(|h| {
            std::cmp::Reverse(
                h["bounds"][1].as_i64().unwrap() + h["bounds"][3].as_i64().unwrap() / 2,
            )
        });
        self.editing_chords[index].members = members;
        self.editing_chords[index].stem = stem;
    }
    pub(super) fn invalidate_editing_chord(&mut self, id: usize) -> Result<(), String> {
        let Some(index) = self.editing_chords.iter().position(|c| c.id == id) else {
            return Ok(());
        };
        self.refresh_editing_chord_links(id);
        let bounds = serde_json::from_value(self.vertices[id - 1].data["bounds"].clone())
            .map_err(|_| "editing_nullable_chord_bounds_required")?;
        if self.editing_chords[index].measure.is_none()
            && !self.editing_chords[index].members.is_empty()
        {
            let (measure, ordinal) = self
                .dot_ownership
                .as_ref()
                .and_then(|owner| {
                    owner.measure_for_chord(
                        self.vertices[id - 1].key.system,
                        &self.vertices[id - 1].data,
                    )
                })
                .ok_or("editing_actual_head_measure_required")?;
            self.editing_chords[index].measure = Some(measure);
            self.editing_chords[index].in_measure = true;
            self.vertices[id - 1].data["measure_index"] = serde_json::json!(ordinal);
        }
        self.editing_chords[index].bounds = bounds;
        Ok(())
    }
}
