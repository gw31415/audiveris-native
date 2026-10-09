// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual singleton Rest ownership and weak removal.
use super::{Graph, Key, Phase};
use crate::{
    grid_ir::StaffGeometry,
    music_measures::{Chord, ChordKind, Measure},
    music_voices::Voice,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
struct StaffRests {
    staff: Arc<StaffGeometry>,
    members: Vec<Key>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct RestOwnership {
    staffs: BTreeMap<(u64, u64), StaffRests>,
    // The retained Staff pointer outlives membership removal, like AbstractInter.staff.
    owners: BTreeMap<Key, Arc<StaffGeometry>>,
    retired_chords: Vec<Chord>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RestRemovalStage {
    Staff,
    Measure,
    Edge,
    Vertex,
}
impl RestRemovalStage {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Staff => "StaffRemoved",
            Self::Measure => "MeasureRemoved",
            Self::Edge => "EdgeRemoved",
            Self::Vertex => "VertexRemoved",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct RestRemovalEvent {
    pub(crate) stage: RestRemovalStage,
    pub(crate) inter: Key,
    pub(crate) relation_id: Option<usize>,
}
type Observer<'a> =
    dyn FnMut(&Graph, &[Chord], &[Voice], RestRemovalEvent) -> Result<(), String> + 'a;

impl Graph {
    pub(super) fn requires_owned_rest_removal(&self, key: Key) -> bool {
        if self.rest_ownership.owners.contains_key(&key) {
            return true;
        }
        let Some(id) = self.id(key) else { return false };
        self.vertices[id - 1].data["kind"] == "RestChordInter"
            && self.edges.iter().any(|edge| {
                !edge.removed
                    && edge.key.kind == "Containment"
                    && edge.key.source == id
                    && self
                        .rest_ownership
                        .owners
                        .contains_key(&self.vertices[edge.key.target - 1].key)
            })
    }
    /// Call at the actual Rest added boundary, using CreatedRest.staff. This is
    /// a canonical Rest subset of Staff notes, never a projection-derived list.
    pub(crate) fn register_rest_membership(
        &mut self,
        key: Key,
        staff: Arc<StaffGeometry>,
    ) -> Result<(), String> {
        let id = self.endpoint(key)?;
        let vertex = &self.vertices[id - 1];
        if key.phase != Phase::Symbol
            || vertex.data["kind"] != "RestInter"
            || vertex.data["staff_id"] != staff.id
            || key.system != staff.system
            || vertex.removing
        {
            return Err("music_weak_rest_actual_added_Staff_required".into());
        }
        if self
            .rest_ownership
            .owners
            .get(&key)
            .is_some_and(|old| !Arc::ptr_eq(old, &staff))
            || self
                .rest_ownership
                .staffs
                .get(&(staff.system, staff.id))
                .is_some_and(|old| !Arc::ptr_eq(&old.staff, &staff))
        {
            return Err("music_weak_rest_Staff_identity_changed".into());
        }
        self.rest_ownership.owners.insert(key, staff.clone());
        let members = &mut self
            .rest_ownership
            .staffs
            .entry((staff.system, staff.id))
            .or_insert_with(|| StaffRests {
                staff,
                members: Vec::new(),
            })
            .members;
        if !members.contains(&key) {
            members.push(key);
        }
        Ok(())
    }
    pub(crate) fn retained_rest_staff(&self, key: Key) -> Result<&Arc<StaffGeometry>, String> {
        self.rest_ownership
            .owners
            .get(&key)
            .ok_or_else(|| "music_weak_rest_added_owner_required".into())
    }
    #[cfg(test)]
    pub(crate) fn staff_rest_members(&self, staff: &Arc<StaffGeometry>) -> Result<&[Key], String> {
        let owner = self
            .rest_ownership
            .staffs
            .get(&(staff.system, staff.id))
            .filter(|owner| Arc::ptr_eq(&owner.staff, staff))
            .ok_or("music_weak_rest_actual_Staff_required")?;
        Ok(&owner.members)
    }
    #[cfg(test)]
    pub(crate) fn retained_removed_rest_chord(&self, key: Key) -> Option<&Chord> {
        self.rest_ownership
            .retired_chords
            .iter()
            .find(|chord| chord.key == key)
    }
    /// (isRemoved, containsVertex): Graph.removing represents the original
    /// removed=true interval while callbacks still see the registered object.
    #[cfg(test)]
    pub(crate) fn rest_removal_relation(
        &self,
        id: usize,
    ) -> Result<(&serde_json::Value, bool), String> {
        let edge = self
            .edges
            .get(id.checked_sub(1).ok_or("music_rest_relation_required")?)
            .ok_or("music_rest_relation_required")?;
        Ok((&edge.data, edge.removed))
    }
    #[cfg(test)]
    pub(crate) fn weak_rest_registration(&self, key: Key) -> Result<(bool, bool), String> {
        let id = self
            .id(key)
            .ok_or("music_weak_rest_registered_identity_required")?;
        let vertex = &self.vertices[id - 1];
        Ok((vertex.removed || vertex.removing, !vertex.removed))
    }
    /// One captured-queue item. The outer common coordinator must also stage its
    /// whole queue: a later family failure must roll back earlier removals.
    pub(crate) fn remove_first_weak_rest(
        &mut self,
        key: Key,
        measures: &[Measure],
        chords: &mut Vec<Chord>,
        voices: &mut Vec<Voice>,
        mut observe: impl FnMut(&Graph, &[Chord], &[Voice], RestRemovalEvent) -> Result<(), String>,
    ) -> Result<Vec<Key>, String> {
        let mut graph = self.clone();
        let mut next_chords = chords.clone();
        let mut next_voices = voices.clone();
        let mut removed = Vec::new();
        graph.remove_weak_rest_inner(
            key,
            true,
            measures,
            &mut next_chords,
            &mut next_voices,
            &mut removed,
            &mut observe,
        )?;
        *self = graph;
        *chords = next_chords;
        *voices = next_voices;
        Ok(removed)
    }
    #[allow(clippy::too_many_arguments)]
    fn remove_weak_rest_inner(
        &mut self,
        key: Key,
        extensive: bool,
        measures: &[Measure],
        chords: &mut Vec<Chord>,
        voices: &mut Vec<Voice>,
        removed: &mut Vec<Key>,
        observe: &mut Observer<'_>,
    ) -> Result<(), String> {
        let id = self
            .id(key)
            .ok_or("music_weak_rest_registered_identity_required")?;
        if self.vertices[id - 1].removed || self.vertices[id - 1].removing {
            return Ok(());
        }
        if self.vertices[id - 1].data["frozen"] == true {
            return Err("music_weak_rest_frozen_cascade_pending".into());
        }
        let is_rest = self.vertices[id - 1].data["kind"] == "RestInter";
        let is_chord = self.vertices[id - 1].data["kind"] == "RestChordInter";
        if !(is_rest && key.phase == Phase::Symbol || is_chord && key.phase == Phase::Chord) {
            return Err("music_weak_rest_typed_inter_required".into());
        }
        let emit = |graph: &Graph,
                    chords: &[Chord],
                    voices: &[Voice],
                    stage,
                    relation_id,
                    observe: &mut Observer<'_>| {
            observe(
                graph,
                chords,
                voices,
                RestRemovalEvent {
                    stage,
                    inter: key,
                    relation_id,
                },
            )
        };
        if is_rest {
            let staff = self.retained_rest_staff(key)?.clone();
            if self.vertices[id - 1].data["staff_id"] != staff.id {
                return Err("music_weak_rest_raw_Staff_changed".into());
            }
            let membership = self
                .rest_ownership
                .staffs
                .get_mut(&(staff.system, staff.id))
                .filter(|owner| Arc::ptr_eq(&owner.staff, &staff))
                .ok_or("music_weak_rest_actual_Staff_required")?;
            membership.members.retain(|member| *member != key);
            emit(self, chords, voices, RestRemovalStage::Staff, None, observe)?;
            self.vertices[id - 1].removing = true;
            if extensive {
                let parents = self
                    .incident_edges(id)
                    .into_iter()
                    .filter_map(|eid| {
                        let edge = &self.edges[eid - 1];
                        (edge.key.target == id && edge.key.kind == "Containment")
                            .then_some(edge.key.source)
                    })
                    .collect::<Vec<_>>();
                for parent in parents {
                    self.validate_chord_identity(parent, id, "Containment")?;
                    let members = self
                        .incident_edges(parent)
                        .into_iter()
                        .filter(|eid| {
                            let edge = &self.edges[*eid - 1];
                            edge.key.source == parent && edge.key.kind == "Containment"
                        })
                        .count();
                    if members != 1 {
                        return Err("music_weak_rest_singleton_ensemble_required".into());
                    }
                    self.remove_weak_rest_inner(
                        self.vertices[parent - 1].key,
                        false,
                        measures,
                        chords,
                        voices,
                        removed,
                        observe,
                    )?;
                }
            }
        } else {
            let index = chords
                .iter()
                .position(|chord| chord.key == key && chord.id == id)
                .ok_or("music_weak_rest_actual_Chord_required")?;
            let chord = &chords[index];
            let measure = measures
                .get(chord.measure)
                .ok_or("music_weak_rest_actual_Measure_required")?;
            if chord.kind != ChordKind::Rest
                || chord.stem.is_some()
                || measure.system != key.system
                || self.vertices[id - 1].data["part_id"] != measure.part
                || self.vertices[id - 1].data["measure_index"] != measure.index
                || self
                    .rest_ownership
                    .retired_chords
                    .iter()
                    .any(|old| old.key == key)
            {
                return Err("music_weak_rest_Measure_membership_required".into());
            }
            let member_edges = self
                .incident_edges(id)
                .into_iter()
                .filter(|eid| {
                    let edge = &self.edges[*eid - 1];
                    edge.key.source == id && edge.key.kind == "Containment"
                })
                .collect::<Vec<_>>();
            if member_edges.len() != 1
                || chord.members.len() != 1
                || chord.members[0]["id"] != self.edges[member_edges[0] - 1].key.target
            {
                return Err("music_weak_rest_actual_singleton_Chord_required".into());
            }
            let member_id = self.edges[member_edges[0] - 1].key.target;
            self.validate_chord_identity(id, member_id, "Containment")?;
            let staff = self.retained_rest_staff(self.vertices[member_id - 1].key)?;
            if let Some(geometry) = &measure.geometry {
                if !geometry.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)) {
                    return Err("music_weak_rest_actual_Part_Staff_required".into());
                }
            } else if measure.staff != staff.id {
                return Err("music_weak_rest_declared_scalar_Staff_required".into());
            }
            let chord = chords.remove(index);
            let mut vi = 0;
            while vi < voices.len() {
                let voice = &mut voices[vi];
                if voice.measure != chord.measure || !voice.chords_initialized {
                    vi += 1;
                    continue;
                }
                let Some(ci) = voice.chord_ids.iter().position(|member| *member == id) else {
                    vi += 1;
                    continue;
                };
                voice.chord_ids.remove(ci);
                if voice.chord_ids.is_empty() {
                    self.retain_voice_owner(voices.remove(vi));
                    break;
                }
                vi += 1;
            }
            self.rest_ownership.retired_chords.push(chord);
            emit(
                self,
                chords,
                voices,
                RestRemovalStage::Measure,
                None,
                observe,
            )?;
            self.vertices[id - 1].removing = true;
            if extensive {
                let other_parent = self.incident_edges(member_id).into_iter().any(|eid| {
                    let edge = &self.edges[eid - 1];
                    edge.key.target == member_id
                        && edge.key.kind == "Containment"
                        && edge.key.source != id
                });
                if !other_parent {
                    self.remove_weak_rest_inner(
                        self.vertices[member_id - 1].key,
                        false,
                        measures,
                        chords,
                        voices,
                        removed,
                        observe,
                    )?;
                }
            }
        }
        // Graph.edgesOf is incoming-then-outgoing, each by live insertion order.
        for eid in self.incident_edges(id) {
            let edge = self.edges[eid - 1].clone();
            match edge.key.kind.as_str() {
                "Containment" => {
                    self.remove_chord_identity_edges(&[eid])?;
                    let source_key = self.vertices[edge.key.source - 1].key;
                    if let Some(chord) = self
                        .rest_ownership
                        .retired_chords
                        .iter_mut()
                        .find(|chord| chord.key == source_key)
                    {
                        chord
                            .members
                            .retain(|member| member["id"] != edge.key.target);
                    }
                }
                "AugmentationRelation" if is_rest && edge.key.target == id => {
                    self.remove_augmentation_edge(eid)?;
                }
                "BeamRestRelation" if is_rest && edge.key.target == id => {
                    self.remove_lookup_beam_rest(eid)?;
                }
                "ChordTupletRelation" if is_chord && edge.key.source == id => {
                    // Current typed API admits implicit tuplets only; an explicit
                    // survivor needs its original checkAbnormal callback first.
                    self.remove_owned_chord_tuplet(eid)?;
                }
                "Exclusion" | "NoExclusion" => self.edges[eid - 1].removed = true,
                "NextInVoiceRelation" if is_chord => self.edges[eid - 1].removed = true,
                _ => {
                    return Err(format!(
                        "music_weak_rest_relation_callback_pending:{}",
                        edge.key.kind
                    ));
                }
            }
            emit(
                self,
                chords,
                voices,
                RestRemovalStage::Edge,
                Some(eid),
                observe,
            )?;
        }
        self.vertices[id - 1].removed = true;
        self.vertices[id - 1].removing = false;
        removed.push(key);
        emit(
            self,
            chords,
            voices,
            RestRemovalStage::Vertex,
            None,
            observe,
        )
    }
}

#[cfg(test)]
mod tests;
