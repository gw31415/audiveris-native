// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{EditingLink, EditingOperation, Key};
use crate::music_measures::{Chord, ChordKind, OwnedMusic};
use serde_json::Value;

/// Retain the actual emptied Chord and its surviving links across removal.
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
            if !owner.chords[index].members.is_empty()
                || edges.iter().any(|eid| {
                    let e = &graph.edges[*eid - 1];
                    e.key.source != id || e.key.kind != "ChordStemRelation"
                })
            {
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
            for voice in owner
                .voices
                .iter_mut()
                .filter(|v| v.measure == chord.measure)
            {
                voice.chord_ids.retain(|cid| *cid != id);
            }
            owner.voices.retain(|v| {
                v.measure != chord.measure
                    || !v.chord_ids.is_empty()
                    || v.measure_rest_chord.is_some()
            });
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
