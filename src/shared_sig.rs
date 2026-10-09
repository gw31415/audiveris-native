// SPDX-License-Identifier: AGPL-3.0-or-later
// Deterministic native identities and relation store across existing phases.
use crate::{
    beam_candidates::SystemCandidates, beam_groups::GroupPass,
    multiple_rest_registration::RestGraph,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
mod alter_callbacks;
mod articulation_callbacks;
mod beam_rest;
#[cfg(test)]
mod context_projection_image_tests;
mod direction_callbacks;
mod explicit_tuplet;
#[cfg(test)]
mod flag_rest_weak_image_tests;
mod header_binding;
mod ornament;
mod tuplet;
mod tuplet_addition;
#[cfg(test)]
pub(crate) use beam_rest::CallbackObservation;
pub use tuplet::TupletFactor;
#[cfg(test)]
pub(crate) use tuplet::TupletRemovalObservation;
#[cfg(test)]
pub(crate) use tuplet_addition::{TupletAdditionObservation, TupletAdditionOwners};
mod book_state;
mod chord_cache;
mod chord_duration;
pub(crate) mod chord_tasks;
mod dot_ownership;
mod editing_task;
mod manual_chord_stem;
#[cfg(test)]
mod stem_refinement_tests;
mod symbol_cleaner;
mod voice_owners;
pub use chord_tasks::{ChordAdditionTask, ChordRemovalTask};
pub use dot_ownership::NoteOwnerEdit;
pub use editing_task::{
    DefaultEditingTask, EditingLink, EditingOperation, EditingSequence, EditingTask, RelationTask,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Phase {
    Grid,
    Beam,
    Group,
    Rest,
    Ledger,
    Head,
    Stem,
    MusicHeader,
    StaffBarline,
    Chord,
    Symbol,
    MusicKey,
    MusicTime,
}

#[cfg(test)]
mod weak_foundation_tests {
    use super::*;
    #[test]
    fn actual_grid_bar_removal_updates_staff_membership_and_preserves_registry() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-foundation-barline-removal/observed.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/rest-augmentation-unconfirmed.png"
        ))
        .unwrap();
        let owned = crate::stem_image_owned::prepare_items(&image).unwrap();
        let snapshot = owned.graph.snapshot();
        let bar = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "BarlineInter" && n["bounds"] == original["bar"]["bounds"])
            .unwrap();
        let k: Key = serde_json::from_value(bar["origin"].clone()).unwrap();
        assert_eq!(
            k,
            Key {
                system: 1,
                phase: Phase::Grid,
                local: 2
            }
        );
        assert_eq!(bar["staff_id"], original["bar"]["staff_id"]);
        let bounds =
            |bars: Vec<Value>| bars.iter().map(|b| b["bounds"].clone()).collect::<Vec<_>>();
        assert_eq!(
            bounds(owned.graph.staff_barlines(1, 1)),
            original["before_staff_barlines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b["bounds"].clone())
                .collect::<Vec<_>>()
        );
        let mut graph = owned.graph.clone();
        graph.remove_foundation_grid_member(k).unwrap();
        assert_eq!(
            bounds(graph.staff_barlines(1, 1)),
            original["after_staff_barlines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b["bounds"].clone())
                .collect::<Vec<_>>()
        );
        assert!(graph.vertices[bar["id"].as_u64().unwrap() as usize - 1].removed);
        let after = graph.snapshot();
        graph.remove_foundation_grid_member(k).unwrap();
        assert_eq!(graph.snapshot(), after);
        assert_eq!(graph.id(k), Some(bar["id"].as_u64().unwrap() as usize));
        assert_eq!(bar["glyph_id_namespace"], "GRID");
        let physical = owned.upstream_report["grid_ir"]["glyphs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["id"] == bar["glyph_id"])
            .unwrap()
            .clone();
        assert_eq!(physical["left"], bar["bounds"][0]);
        assert_eq!(
            graph.vertices[bar["id"].as_u64().unwrap() as usize - 1].data["glyph_id"],
            physical["id"]
        );
        for property in ["manual", "frozen"] {
            let mut protected = owned.graph.clone();
            let mut n = bar.clone();
            n[property] = json!(true);
            protected.vertex(k, n, false).unwrap();
            let before = protected.snapshot();
            assert!(protected.remove_foundation_grid_member(k).is_err());
            assert_eq!(protected.snapshot(), before);
        }
        let mut ensemble = owned.graph.clone();
        let parent = Key {
            system: 1,
            phase: Phase::StaffBarline,
            local: 1,
        };
        ensemble
            .vertex(
                parent,
                json!({"kind":"StaffBarlineInter","staff_id":1,"bounds":bar["bounds"]}),
                false,
            )
            .unwrap();
        ensemble
            .relations(
                1,
                Phase::StaffBarline,
                vec![(parent, k, json!({"kind":"Containment"}))],
            )
            .unwrap();
        let before = ensemble.snapshot();
        assert!(ensemble.remove_foundation_grid_member(k).is_err());
        assert_eq!(ensemble.snapshot(), before);
        let owner = crate::music_links::prepare_beam_linking(owned).unwrap();
        let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
        let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
        assert_eq!(
            bounds(owner.graph.staff_barlines(1, 1)),
            original["after_staff_barlines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b["bounds"].clone())
                .collect::<Vec<_>>()
        );
        assert!(
            owner.upstream_report["grid_ir"]["glyphs"]
                .as_array()
                .unwrap()
                .contains(&physical)
        );
        let m = crate::music_measures::continue_measures(owner).unwrap();
        assert_eq!(m.layout()["measures"].as_array().unwrap().len(), 2);
    }
    fn add(g: &mut Graph, local: usize, data: Value) {
        g.vertex(
            Key {
                system: 1,
                phase: Phase::Head,
                local,
            },
            data,
            false,
        )
        .unwrap();
    }
    #[test]
    fn weak_threshold_frozen_ledger_and_incident_edges_retain_identities() {
        let mut g = Graph::default();
        add(
            &mut g,
            1,
            json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK","contextual_grade":0.5}),
        );
        add(
            &mut g,
            2,
            json!({"kind":"HeadInter","contextual_grade":0.5_f64.next_down()}),
        );
        add(
            &mut g,
            3,
            json!({"kind":"HeadInter","contextual_grade":0.1,"frozen":true}),
        );
        add(&mut g, 4, json!({"kind":"LedgerInter","shape":"LEDGER"}));
        add(
            &mut g,
            5,
            json!({"kind":"StemInter","contextual_grade":0.6}),
        );
        for source in [1, 2] {
            g.edges.push(Edge {
                live_order: source as u64,
                key: EdgeKey {
                    system: 1,
                    phase: Phase::Stem,
                    source,
                    target: 5,
                    kind: "HeadStemRelation".into(),
                    ordinal: source,
                },
                data: json!({"id":source,"source":source,"target":5,"kind":"HeadStemRelation"}),
                removed: false,
            });
        }
        assert_eq!(g.purge_weak_foundations().unwrap(), vec![2]);
        let s = g.snapshot();
        assert_eq!(s["tombstones"][0]["id"], 2);
        assert_eq!(s["relations"].as_array().unwrap().len(), 1);
        assert_eq!(s["inters"][0]["id"], 1);
        assert_eq!(s["inters"][0]["abnormal"], false);
        assert_eq!(s["inters"][3]["abnormal"], false);
        assert!(g.purge_weak_foundations().unwrap().is_empty());
        assert_eq!(s, g.snapshot());
        assert_eq!(
            g.id(Key {
                system: 1,
                phase: Phase::Head,
                local: 2
            }),
            Some(2)
        );
        assert!(
            Graph::default()
                .purge_weak_foundations()
                .unwrap()
                .is_empty()
        );
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-weak-stem-purge/controls.json"
        ))
        .unwrap();
        for case in original["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            graph
                .vertex(
                    Key {
                        system: 1,
                        phase: Phase::Stem,
                        local: 1,
                    },
                    json!({"kind":"StemInter","shape":"STEM",
                    "contextual_grade":case["grade"],"frozen":case["frozen"]}),
                    false,
                )
                .unwrap();
            let before = graph.snapshot();
            let (staged, removed) = graph.prepare_weak_foundations().unwrap();
            assert_eq!(!removed.is_empty(), case["selected_for_removal"]);
            assert_eq!(
                !staged.snapshot()["tombstones"]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                case["same_object_removed"]
            );
            assert_eq!(
                !staged.snapshot()["inters"].as_array().unwrap().is_empty(),
                case["still_in_SIG"]
            );
            assert_eq!(graph.snapshot(), before);
        }
    }
    #[test]
    fn weak_stem_or_invalid_context_refuses_before_any_mutation() {
        for bad in [
            json!({"kind":"StemInter","contextual_grade":0.1}),
            json!({"kind":"HeadInter"}),
            json!({"kind":"HeadInter","contextual_grade":1.01}),
            json!({"kind":"BeamInter","contextual_grade":0.1}),
        ] {
            let mut g = Graph::default();
            add(
                &mut g,
                1,
                json!({"kind":"HeadInter","contextual_grade":0.1}),
            );
            add(&mut g, 2, bad);
            let before = g.snapshot();
            assert!(g.purge_weak_foundations().is_err());
            assert_eq!(before, g.snapshot());
        }
    }
    #[test]
    fn automatic_foundation_plan_failure_keeps_private_identity_store_unchanged() {
        use crate::stem_head_foundation_plan::{Event, Plan};
        let mut g = Graph::default();
        add(
            &mut g,
            1,
            json!({"kind":"HeadInter","grade":0.8,"contextual_grade":0.8}),
        );
        let before = g.snapshot();
        let bad = Plan {
            events: vec![Event::RemoveVertex(1), Event::RemoveVertex(999)],
            final_state: json!({"inters":[],"relations":[]}),
            stages: Value::Null,
        };
        assert!(g.apply_head_foundation_plan(&bad).is_err());
        assert_eq!(before, g.snapshot());
        let mut changed = before.clone();
        changed["inters"][0]["grade"] = json!(0.9);
        let bad = Plan {
            events: vec![],
            final_state: changed,
            stages: Value::Null,
        };
        assert!(g.apply_head_foundation_plan(&bad).is_err());
        assert_eq!(before, g.snapshot());
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Key {
    pub system: u64,
    pub phase: Phase,
    pub local: usize,
}
/// Stable caller-owned relation identity distinguishes parallel relations when
/// one is removed. Head keys use their existing native HEAD registration key.
pub struct HeadStemAttachment {
    pub local: usize,
    pub head: Key,
    pub stem_local: usize,
    pub relation: Value,
}
#[derive(Clone, Debug)]
struct Vertex {
    key: Key,
    data: Value,
    removed: bool,
    removing: bool,
    cached_part: Option<u64>,
    retained_voice: Option<usize>,
    voice_owner: Option<std::sync::Arc<()>>,
    slot_owner: Option<std::sync::Arc<()>>,
    chord_cache: Option<chord_cache::ChordCache>,
    rest_beams: Option<std::sync::Arc<Vec<usize>>>,
    tuplet_cache: Option<tuplet::TupletCache>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeKey {
    system: u64,
    phase: Phase,
    source: usize,
    target: usize,
    kind: String,
    ordinal: usize,
}
#[derive(Clone, Debug)]
struct Edge {
    live_order: u64,
    key: EdgeKey,
    data: Value,
    removed: bool,
}
#[derive(Clone, Default, Debug)]
pub struct Graph {
    native_header_bindings: BTreeMap<Key, header_binding::HeaderBinding>,
    native_header_relation_bindings:
        BTreeMap<usize, std::sync::Arc<crate::native_headers::NativeHeaderRelation>>,
    // This certificate is held by the live graph; reports cannot supply it.
    refined_stem_geometry: BTreeMap<Key, Value>,
    pub(crate) editing_chords: Vec<chord_tasks::EditingHeadChord>,
    retained_voices: Vec<crate::music_voices::Voice>,
    vertices: Vec<Vertex>,
    keys: BTreeMap<Key, usize>,
    edges: Vec<Edge>,
    edge_keys: BTreeMap<EdgeKey, usize>,
    edge_clock: u64,
    #[cfg(test)]
    beam_rest_callbacks: Option<Vec<beam_rest::CallbackObservation>>,
    #[cfg(test)]
    tuplet_removals: Option<Vec<tuplet::TupletRemovalObservation>>,
    #[cfg(test)]
    tuplet_removal_containers: Option<tuplet::TupletContainers>,
    #[cfg(test)]
    tuplet_observer_order: usize,
    #[cfg(test)]
    tuplet_additions: Option<Vec<tuplet_addition::TupletAdditionObservation>>,
    #[cfg(test)]
    tuplet_candidate_clocks: Option<Value>,
    #[cfg(test)]
    tuplet_addition_failure: Option<usize>,
    #[cfg(test)]
    tuplet_registration_failure: Option<usize>,
    dot_removals: Vec<Value>,
    edit_publications: Vec<Value>,
    edit_impacts: Vec<Value>,
    dot_ownership: Option<dot_ownership::Ownership>,
    editing_book: Option<book_state::BookState>,
    #[cfg(test)]
    trace_dot_callbacks: bool,
    #[cfg(test)]
    dot_callback_depth: usize,
    #[cfg(test)]
    dot_callback_trace: Vec<Value>,
    #[cfg(test)]
    observed_chord: Option<Key>,
    #[cfg(test)]
    observed_chords: Vec<Key>,
    #[cfg(test)]
    observed_voice_bindings: Vec<(Key, std::sync::Arc<()>)>,
}
impl Graph {
    pub(crate) fn next_symbol_key(&self, system: u64) -> Result<Key, String> {
        // Internal DotFactory interpretations can be registered without being
        // returned by InterFactory. Glyph-based slots would alias that vertex.
        let local = self
            .keys
            .keys()
            .filter(|k| k.system == system && k.phase == Phase::Symbol)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("music_symbol_identity_budget")?;
        Ok(Key {
            system,
            phase: Phase::Symbol,
            local,
        })
    }
    fn require_pre_heads(&self, system: u64) -> Result<(), String> {
        if self.vertices.iter().any(|v| {
            v.key.system == system && matches!(v.key.phase, Phase::Head | Phase::Stem) && !v.removed
        }) {
            return Err("complete_shared_sig_heads_state_required".into());
        }
        Ok(())
    }
    pub(crate) fn live_inter_keys(&self, system: u64, kind: &str) -> Vec<Key> {
        self.vertices
            .iter()
            .filter(|v| !v.removed && v.key.system == system && v.data["kind"] == kind)
            .map(|v| v.key)
            .collect()
    }
    pub(crate) fn has_relation(&self, source: Key, target: Key, kind: &str) -> bool {
        let (Some(source), Some(target)) = (self.id(source), self.id(target)) else {
            return false;
        };
        self.edges.iter().any(|e| {
            !e.removed && e.key.source == source && e.key.target == target && e.key.kind == kind
        })
    }
    pub(crate) fn retained_inter(&self, key: Key) -> Result<(&Value, bool), String> {
        let id = self.id(key).ok_or("missing_actual_retained_inter")?;
        let vertex = &self.vertices[id - 1];
        Ok((&vertex.data, vertex.removed))
    }
    pub fn id(&self, key: Key) -> Option<usize> {
        self.keys.get(&key).copied()
    }
    pub(crate) fn vertex(
        &mut self,
        key: Key,
        mut data: Value,
        removed: bool,
    ) -> Result<usize, String> {
        if key.system == 0 || key.local == 0 || !data.is_object() {
            return Err("invalid_shared_sig_vertex".into());
        }
        let id = if let Some(id) = self.id(key) {
            id
        } else {
            if self.vertices.len() >= 100000 {
                return Err("shared_sig_vertex_budget".into());
            }
            let id = self.vertices.len() + 1;
            self.keys.insert(key, id);
            self.vertices.push(Vertex {
                key,
                data: Value::Null,
                removed,
                removing: false,
                cached_part: None,
                retained_voice: None,
                voice_owner: None,
                slot_owner: None,
                chord_cache: None,
                rest_beams: None,
                tuplet_cache: None,
            });
            id
        };
        data["id"] = json!(id);
        data["origin"] = json!(key);
        // Context lives on the registered Inter, not the glyph cache. Re-registering
        // the same intrinsic stem/head/dot must retain it, as original addVertex does.
        let old = &self.vertices[id - 1];
        if (matches!(key.phase, Phase::Head | Phase::Stem)
            || (key.phase == Phase::Symbol
                && data["kind"] == "AugmentationDotInter"
                && old.data["kind"] == data["kind"]))
            && !removed
            && old.data["grade"] == data["grade"]
            && data["contextual_grade"].is_null()
            && !old.data["contextual_grade"].is_null()
        {
            data["contextual_grade"] = old.data["contextual_grade"].clone();
        }
        let added = old.data.is_null() || old.removed;
        let data_slot_unchanged = data["slot_id"] == old.data["slot_id"];
        let data_voice_unchanged = data["voice"] == old.data["voice"];
        let retained_voice = if data_voice_unchanged {
            old.retained_voice
        } else {
            None
        };
        let tuplet_cache = if data["kind"] == "TupletInter" {
            Some(old.tuplet_cache.clone().unwrap_or_default())
        } else {
            None
        };
        self.vertices[id - 1] = Vertex {
            key,
            data,
            removed,
            removing: old.removing,
            cached_part: old.cached_part,
            retained_voice,
            voice_owner: if data_voice_unchanged {
                old.voice_owner.clone()
            } else {
                None
            },
            slot_owner: if data_slot_unchanged {
                old.slot_owner.clone()
            } else {
                None
            },
            chord_cache: old.chord_cache.clone(),
            rest_beams: old.rest_beams.clone(),
            tuplet_cache,
        };
        if !removed && added && self.vertices[id - 1].data["kind"] == "AugmentationDotInter" {
            self.add_dot_to_measure(id)?;
            self.vertices[id - 1].data["abnormal"] = json!(true);
        }
        if !removed && added && self.vertices[id - 1].data["kind"] == "AlterInter" {
            self.vertices[id - 1].data["abnormal"] = json!(true);
        }
        if !removed
            && added
            && matches!(
                self.vertices[id - 1].data["kind"].as_str(),
                Some("ArticulationInter" | "PedalInter")
            )
        {
            self.vertices[id - 1].data["abnormal"] = json!(true);
        }
        Ok(id)
    }
    fn endpoint(&self, key: Key) -> Result<usize, String> {
        let id = self.id(key).ok_or("missing_shared_sig_endpoint")?;
        if self.vertices[id - 1].removed {
            return Err("removed_shared_sig_endpoint".into());
        }
        Ok(id)
    }
    fn phase(&mut self, system: u64, phase: Phase) {
        for v in &mut self.vertices {
            if v.key.system == system && v.key.phase == phase {
                v.removed = true;
            }
        }
        for e in &mut self.edges {
            if e.key.system == system && e.key.phase == phase {
                e.removed = true;
            }
        }
    }
    /// Staff membership is a view of the live owner, not the historical GRID report.
    #[cfg(test)]
    pub(crate) fn staff_barlines(&self, system: u64, staff: u64) -> Vec<Value> {
        self.vertices
            .iter()
            .filter(|v| {
                !v.removed
                    && v.key.system == system
                    && v.data["kind"] == "BarlineInter"
                    && v.data["staff_id"] == staff
            })
            .map(|v| v.data.clone())
            .collect()
    }

    fn remove_foundation_grid_member(&mut self, key: Key) -> Result<(), String> {
        let id = self.id(key).ok_or("missing_shared_sig_endpoint")?;
        let v = &self.vertices[id - 1];
        if key.phase != Phase::Grid
            || !matches!(
                v.data["kind"].as_str(),
                Some("BarlineInter" | "BarConnectorInter" | "BraceInter")
            )
            || (v.data["kind"] == "BarlineInter" && v.data["staff_id"].as_u64().is_none())
        {
            return Err("music_foundation_owned_GRID_member_required".into());
        }
        if v.removed {
            return Ok(());
        }
        if v.data["frozen"] == true
            || v.data["is_frozen"] == true
            || v.data["manual"] == true
            || v.data["is_manual"] == true
        {
            return Err("music_foundation_protected_GRID_member_removal_pending".into());
        }
        // Manual Brace removal splits actual Parts; recognition never substitutes
        // a tombstone for that structural callback. Keep it protected above.
        // Post-MEASURES closure, grouped bars and support callbacks need their
        // own original owners; removing a bare GRID member cannot cover them.
        if self.edges.iter().any(|e| {
            !e.removed
                && (e.key.source == id || e.key.target == id)
                && !matches!(e.key.kind.as_str(), "Exclusion" | "NoExclusion")
        }) {
            return Err("music_foundation_GRID_ensemble_or_support_removal_pending".into());
        }
        self.vertices[id - 1].removed = true;
        self.prune();
        Ok(())
    }

    /// Bounded ledger/beam/symbol/container removals retain IDs and incident tombstones.
    /// Cached stems and head callbacks must use their dedicated removal path.
    pub(crate) fn remove_music_inter(&mut self, key: Key) -> Result<(), String> {
        if !matches!(
            key.phase,
            Phase::Beam | Phase::Group | Phase::Symbol | Phase::Chord | Phase::Ledger
        ) {
            return Err("music_removal_requires_uncached_ledger_beam_symbol_or_chord".into());
        }
        let id = self.id(key).ok_or("missing_shared_sig_endpoint")?;
        // Original Inter.remove is idempotent; a dying ensemble may already
        // have been removed by the last member's callback.
        if self.vertices[id - 1].removed || self.vertices[id - 1].removing {
            return Ok(());
        }
        if self.vertices[id - 1].data["kind"] == "AugmentationDotInter" {
            let mut next = self.clone();
            next.remove_dot_vertex(id, None)?;
            *self = next;
            return Ok(());
        }
        if key.phase == Phase::Group && self.vertices[id - 1].data["kind"] != "BeamGroupInter" {
            return Err("music_removal_only_beam_group_supported".into());
        }
        if key.phase == Phase::Ledger && self.vertices[id - 1].data["kind"] != "LedgerInter" {
            return Err("music_removal_only_ledger_supported".into());
        }
        if self.vertices[id - 1].data["frozen"] == true {
            return Err("music_removal_frozen_inter".into());
        }
        let mut next = self.clone();
        let mut removed = vec![id as u64];
        if key.phase == Phase::Group {
            // Extensive ensemble deletion removes only members not owned by
            // another live ensemble. Physical glyphs/cache objects are retained.
            for edge in self
                .edges
                .iter()
                .filter(|e| !e.removed && e.key.kind == "Containment" && e.key.source == id)
            {
                let member = &self.vertices[edge.key.target - 1];
                if member.key.phase != Phase::Beam {
                    return Err("beam ensemble member phase required".into());
                }
                if !self.edges.iter().any(|e| {
                    !e.removed
                        && e.key.kind == "Containment"
                        && e.key.target == edge.key.target
                        && e.key.source != id
                }) {
                    if member.data["frozen"] == true {
                        return Err("music_removal_frozen_inter".into());
                    }
                    removed.push(edge.key.target as u64);
                }
            }
        }
        let kind = self.vertices[id - 1].data["kind"].as_str();
        if kind == Some("RestInter") {
            // AbstractInter.remove(true): delete only an incoming ensemble
            // whose current actual member list has exactly one entry. This is
            // ensemble.remove(false), so no recursive extensive cascade.
            for e in self
                .edges
                .iter()
                .filter(|e| !e.removed && e.key.kind == "Containment" && e.key.target == id)
            {
                self.validate_chord_identity(e.key.source, id, "Containment")?;
                let parent = &self.vertices[e.key.source - 1];
                if parent.removed {
                    return Err("chord_identity_removed_parent".into());
                }
                if self
                    .edges
                    .iter()
                    .filter(|r| {
                        !r.removed && r.key.kind == "Containment" && r.key.source == e.key.source
                    })
                    .count()
                    == 1
                {
                    if parent.data["frozen"] == true {
                        return Err("music_removal_frozen_inter".into());
                    }
                    removed.push(e.key.source as u64);
                }
            }
        } else if matches!(kind, Some("HeadChordInter" | "RestChordInter")) {
            // ensemble.remove(true) calls member.remove(false) only for
            // unshared members; a different ensemble keeps its same vertex.
            for e in self
                .edges
                .iter()
                .filter(|e| !e.removed && e.key.kind == "Containment" && e.key.source == id)
            {
                self.validate_chord_identity(id, e.key.target, "Containment")?;
                let member = &self.vertices[e.key.target - 1];
                if member.removed {
                    return Err("chord_identity_removed_member".into());
                }
                if !self.edges.iter().any(|r| {
                    !r.removed
                        && r.key.kind == "Containment"
                        && r.key.target == e.key.target
                        && r.key.source != id
                }) {
                    if member.data["frozen"] == true {
                        return Err("music_removal_frozen_inter".into());
                    }
                    removed.push(e.key.target as u64);
                }
            }
        }
        // AbstractInter extensive child removal is nonextensive: a retained
        // cached stem is not a Containment member and must stay registered.
        // HeadStemRelation.removed checks only surviving endpoints; collect
        // their actual IDs before pruning, then refresh abnormal after removal.
        let mut affected_head_stem = BTreeSet::new();
        for e in self.edges.iter().filter(|e| {
            !e.removed
                && e.key.kind == "HeadStemRelation"
                && (removed.contains(&(e.key.source as u64))
                    || removed.contains(&(e.key.target as u64)))
        }) {
            let head = &self.vertices[e.key.source - 1];
            let stem = &self.vertices[e.key.target - 1];
            if head.removed
                || stem.removed
                || head.key.phase != Phase::Head
                || stem.key.phase != Phase::Stem
                || head.data["kind"] != "HeadInter"
                || stem.data["kind"] != "StemInter"
                || head.key.system != stem.key.system
            {
                return Err("music_removal_typed_head_stem_required".into());
            }
            affected_head_stem.insert(e.key.source);
            affected_head_stem.insert(e.key.target);
        }
        for id in &removed {
            next.vertices[*id as usize - 1].removed = true;
        }
        let chord_edges = next
            .edges
            .iter()
            .filter(|e| {
                !e.removed
                    && matches!(e.key.kind.as_str(), "Containment" | "ChordStemRelation")
                    && matches!(
                        next.vertices[e.key.source - 1].data["kind"].as_str(),
                        Some("HeadChordInter" | "RestChordInter")
                    )
                    && (next.vertices[e.key.source - 1].removed
                        || next.vertices[e.key.target - 1].removed)
            })
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        next.remove_chord_identity_edges(&chord_edges)?;
        next.prune();
        next.refresh_head_stem_abnormal(&affected_head_stem.into_iter().collect::<Vec<_>>());
        next.refresh_beam_groups(&removed)?;
        *self = next;
        Ok(())
    }
    pub(crate) fn remove_beam_head_supports(&mut self, ids: &[usize]) -> Result<(), String> {
        for id in ids {
            let e = self
                .edges
                .get(id.checked_sub(1).ok_or("invalid beam head support id")?)
                .ok_or("missing beam head support")?;
            if e.removed || e.key.kind != "BeamHeadRelation" {
                return Err("invalid live beam head support".into());
            }
        }
        for id in ids {
            self.edges[*id - 1].removed = true;
        }
        Ok(())
    }

    /// HeadStemsCleaner -> SIGraph.removeEdge -> HeadStemRelation.removed.
    /// Delete only validated actual edges; preserve vertices, other relations,
    /// cache/registry identities and intrinsic/manual/frozen metadata.
    pub(crate) fn remove_head_stem_edges(
        &mut self,
        head: Key,
        ids: &[usize],
    ) -> Result<(), String> {
        let hid = self.endpoint(head)?;
        if head.phase != Phase::Head || self.vertices[hid - 1].data["kind"] != "HeadInter" {
            return Err("cleaner_typed_owned_head_required".into());
        }
        let mut unique = BTreeSet::new();
        let mut ordered = Vec::new();
        for id in ids {
            let e = self
                .edges
                .get(id.checked_sub(1).ok_or("invalid_cleaner_edge_id")?)
                .ok_or("missing_cleaner_edge")?;
            let stem = &self.vertices[e.key.target - 1];
            if e.removed
                || e.key.kind != "HeadStemRelation"
                || e.key.source != hid
                || e.key.system != head.system
                || stem.removed
                || stem.key.system != head.system
                || stem.key.phase != Phase::Stem
                || stem.data["kind"] != "StemInter"
            {
                return Err("cleaner_typed_live_head_stem_edge_required".into());
            }
            if unique.insert(*id) {
                ordered.push(*id);
            }
        }
        // Validate the whole request before mutation, then let each callback see
        // that edge removed and subsequent edges still live, in cleaner order.
        for id in ordered {
            let endpoints = [self.edges[id - 1].key.source, self.edges[id - 1].key.target];
            self.edges[id - 1].removed = true;
            #[cfg(test)]
            self.dot_callback_event("edgeRemoved", id, "enter");
            self.refresh_head_stem_abnormal(&endpoints);
            #[cfg(test)]
            self.dot_callback_event("edgeRemoved", id, "leave");
        }
        Ok(())
    }
    fn refresh_head_stem_abnormal(&mut self, affected: &[usize]) {
        let linked = self
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.kind == "HeadStemRelation")
            .flat_map(|e| [e.key.source, e.key.target])
            .collect::<BTreeSet<_>>();
        for id in affected.iter().copied() {
            let v = &self.vertices[id - 1];
            if !v.removed
                && (v.data["kind"] == "StemInter"
                    || (v.data["kind"] == "HeadInter"
                        && v.data["shape"].as_str().is_some_and(|s| {
                            crate::head_seed_search::ALL_STEM_SHAPES.contains(&s)
                        })))
            {
                self.set_owned_abnormal(id, !linked.contains(&id));
            }
        }
    }
    pub(crate) fn relations(
        &mut self,
        system: u64,
        phase: Phase,
        relations: Vec<(Key, Key, Value)>,
    ) -> Result<(), String> {
        let chords = relations.iter().any(|(a, _, r)| {
            matches!(
                r["kind"].as_str(),
                Some("Containment" | "ChordStemRelation")
            ) && self.id(*a).is_some_and(|id| {
                matches!(
                    self.vertices[id - 1].data["kind"].as_str(),
                    Some("HeadChordInter" | "RestChordInter")
                )
            })
        });
        let dots = relations.iter().any(|(_, _, r)| {
            matches!(
                r["kind"].as_str(),
                Some("AugmentationRelation" | "DoubleDotRelation")
            )
        });
        let retained_stems = self.editing_book.is_some()
            && relations
                .iter()
                .any(|(_, _, r)| r["kind"] == "HeadStemRelation");
        let alters = relations
            .iter()
            .any(|(_, _, r)| r["kind"] == "AlterHeadRelation");
        let articulations = relations
            .iter()
            .any(|(_, _, r)| r["kind"] == "ChordArticulationRelation");
        let directions = relations.iter().any(|(_, _, r)| {
            matches!(
                r["kind"].as_str(),
                Some("ChordDynamicsRelation" | "ChordPedalRelation")
            )
        });
        if chords || dots || retained_stems || alters || articulations || directions {
            // Original identity callbacks operate on the actual registered
            // chord. Validate/update a stage before changing the retained SIG.
            let mut next = self.clone();
            next.relations_impl(system, phase, relations, None)?;
            *self = next;
            Ok(())
        } else {
            self.relations_impl(system, phase, relations, None)
        }
    }
    fn relations_impl(
        &mut self,
        system: u64,
        phase: Phase,
        relations: Vec<(Key, Key, Value)>,
        owned_manual_chord: Option<Key>,
    ) -> Result<(), String> {
        let mut counts = BTreeMap::new();
        let mut affected_chords = BTreeSet::new();
        for (a, b, mut data) in relations {
            if a.system != system || b.system != system || !data.is_object() {
                return Err("invalid_shared_sig_relation".into());
            }
            let source = self.endpoint(a)?;
            let target = self.endpoint(b)?;
            let kind = data["kind"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("invalid_shared_sig_relation_kind")?
                .to_owned();
            if kind == "AlterHeadRelation" {
                self.validate_alter_head(source, target, &data)?;
            }
            if kind == "ChordArticulationRelation" {
                self.validate_chord_articulation(source, target, &data)?;
            }
            if kind == "ChordOrnamentRelation" {
                self.validate_chord_ornament(source, target, &data)?;
            }
            if matches!(
                kind.as_str(),
                "ChordDynamicsRelation" | "ChordPedalRelation"
            ) {
                self.validate_chord_direction(source, target, &data)?;
            }
            let dot_support = matches!(kind.as_str(), "AugmentationRelation" | "DoubleDotRelation");
            if dot_support {
                let a = &self.vertices[source - 1];
                let b = &self.vertices[target - 1];
                if a.key.phase != Phase::Symbol
                    || a.data["kind"] != "AugmentationDotInter"
                    || !matches!(
                        (kind.as_str(), b.key.phase, b.data["kind"].as_str()),
                        ("AugmentationRelation", Phase::Head, Some("HeadInter"))
                            | ("AugmentationRelation", Phase::Symbol, Some("RestInter"))
                            | (
                                "DoubleDotRelation",
                                Phase::Symbol,
                                Some("AugmentationDotInter")
                            )
                    )
                {
                    return Err("music_dot_typed_support_required".into());
                }
            }
            let chord_identity = matches!(
                self.vertices[source - 1].data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            ) && matches!(kind.as_str(), "Containment" | "ChordStemRelation");
            if chord_identity {
                self.validate_chord_identity(source, target, &kind)?;
            }
            let count = counts
                .entry((source, target, kind.clone()))
                .or_insert(0usize);
            let editing_ordinal = if matches!(phase, Phase::Symbol | Phase::Chord) {
                data.as_object_mut()
                    .unwrap()
                    .remove("_editing_registration_ordinal")
                    .map(|v| {
                        v.as_u64()
                            .and_then(|v| usize::try_from(v).ok())
                            .ok_or("editing_relation_identity_required")
                    })
                    .transpose()?
            } else {
                None
            };
            let owned_chord_ordinal = if phase == Phase::Chord
                && kind == "ChordStemRelation"
                && owned_manual_chord == Some(a)
            {
                // HeadChord.setStem creates a fresh relation when no live
                // partner relation exists; removed registration is not replay.
                Some(
                    match self
                        .edge_keys
                        .keys()
                        .filter(|k| {
                            k.system == system
                                && k.phase == phase
                                && k.source == source
                                && k.target == target
                                && k.kind == kind
                        })
                        .map(|k| k.ordinal)
                        .max()
                    {
                        Some(last) => last
                            .checked_add(1)
                            .ok_or("shared_sig_relation_order_budget")?,
                        None => 0,
                    },
                )
            } else {
                None
            };
            let key = EdgeKey {
                system,
                phase,
                source,
                target,
                kind,
                ordinal: if phase == Phase::Stem {
                    data["local_relation_id"]
                        .as_u64()
                        .and_then(|i| usize::try_from(i).ok())
                        .filter(|i| *i > 0)
                        .ok_or("invalid_shared_stem_relation_id")?
                } else {
                    owned_chord_ordinal.or(editing_ordinal).unwrap_or(*count)
                },
            };
            *count += 1;
            let id = if let Some(id) = self.edge_keys.get(&key) {
                *id
            } else {
                if self.edges.len() >= 1000000 {
                    return Err("shared_sig_relation_budget".into());
                }
                let id = self.edges.len() + 1;
                self.edge_keys.insert(key.clone(), id);
                self.edges.push(Edge {
                    live_order: 0,
                    key: key.clone(),
                    data: Value::Null,
                    removed: false,
                });
                id
            };
            data["id"] = json!(id);
            data["source"] = json!(source);
            data["target"] = json!(target);
            data["system_id"] = json!(system);
            let added = self.edges[id - 1].removed || self.edges[id - 1].data.is_null();
            let manual_count = key.kind == "AugmentationRelation"
                && (data["manual"] == true || self.vertices[source - 1].data["manual"] == true);
            if chord_identity && added && self.vertices[source - 1].chord_cache.is_none() {
                affected_chords.insert(source);
            }
            let live_order = if added {
                self.edge_clock = self
                    .edge_clock
                    .checked_add(1)
                    .ok_or("shared_sig_relation_order_budget")?;
                self.edge_clock
            } else {
                self.edges[id - 1].live_order
            };
            self.edges[id - 1] = Edge {
                live_order,
                key,
                data,
                removed: false,
            };
            if chord_identity && added && self.vertices[source - 1].chord_cache.is_some() {
                self.refresh_editing_chord_links(source);
                #[cfg(test)]
                self.dot_callback_event("edgeAdded", id, "enter");
                self.invalidate_note_chords(&BTreeSet::from([source]))?;
                #[cfg(test)]
                self.dot_callback_event("edgeAdded", id, "leave");
            }
            if added
                && self.editing_book.is_some()
                && self.edges[id - 1].key.kind == "HeadStemRelation"
            {
                let head = &self.vertices[source - 1];
                let stem = &self.vertices[target - 1];
                if head.key.phase != Phase::Head
                    || head.data["kind"] != "HeadInter"
                    || stem.key.phase != Phase::Stem
                    || stem.data["kind"] != "StemInter"
                {
                    return Err("retained_head_stem_typed_endpoints_required".into());
                }
                let manual = [&self.edges[id - 1].data, &head.data, &stem.data]
                    .iter()
                    .any(|v| v["manual"] == true || v["is_manual"] == true);
                if manual && owned_manual_chord.is_none() {
                    return Err("retained_manual_head_stem_owner_update_pending".into());
                }
                #[cfg(test)]
                self.dot_callback_event("edgeAdded", id, "enter");
                if manual {
                    self.replace_manual_chord_stem(source, target, owned_manual_chord.unwrap())?;
                }
                self.refresh_head_stem_abnormal(&[source, target]);
                #[cfg(test)]
                self.dot_callback_event("edgeAdded", id, "leave");
            }
            if added && self.edges[id - 1].key.kind == "AlterHeadRelation" {
                self.refresh_alter_abnormal(&[source]);
            }
            if added && self.edges[id - 1].key.kind == "ChordPedalRelation" {
                self.refresh_pedal_abnormal(target);
            }
            if added && self.edges[id - 1].key.kind == "ChordArticulationRelation" {
                self.refresh_articulation_abnormal(target);
            }
            if added && dot_support {
                #[cfg(test)]
                self.dot_callback_event("edgeAdded", id, "enter");
                self.refresh_augmentation_abnormal(&BTreeSet::from([source]));
                // SigListener completes this callback before the next relation;
                // a manual relation can synchronously delete its automatic dot.
                if manual_count {
                    self.recount_augmentation_note(target)?;
                }
                #[cfg(test)]
                self.dot_callback_event("edgeAdded", id, "leave");
            }
        }
        self.invalidate_note_chords(&affected_chords)?;
        Ok(())
    }

    fn validate_chord_identity(
        &self,
        source: usize,
        target: usize,
        kind: &str,
    ) -> Result<(), String> {
        let a = &self.vertices[source - 1];
        let b = &self.vertices[target - 1];
        let (phase, class) = match (a.data["kind"].as_str(), kind) {
            (Some("HeadChordInter"), "Containment") => (Phase::Head, "HeadInter"),
            (Some("HeadChordInter"), "ChordStemRelation") => (Phase::Stem, "StemInter"),
            (Some("RestChordInter"), "Containment") => (Phase::Symbol, "RestInter"),
            _ => return Err("typed_owned_chord_identity_required".into()),
        };
        if a.key.phase != Phase::Chord
            || b.key.phase != phase
            || b.data["kind"] != class
            || a.key.system != b.key.system
            || source == target
        {
            return Err("typed_owned_chord_identity_required".into());
        }
        Ok(())
    }
    /// Containment / ChordStemRelation added/removed -> AbstractChord.invalidateCache.
    /// Original setGrade changes intrinsic mean only, leaving an existing ctx
    /// grade intact. No cache object or physical glyph is replaced.
    fn invalidate_note_chords(&mut self, ids: &BTreeSet<usize>) -> Result<(), String> {
        for id in ids
            .iter()
            .copied()
            .filter(|id| !self.vertices[*id - 1].removed && !self.vertices[*id - 1].removing)
            .collect::<Vec<_>>()
        {
            self.vertices[id - 1].rest_beams = None;
            if self.vertices[id - 1].chord_cache.is_some() {
                self.vertices[id - 1].chord_cache = Some(chord_cache::ChordCache::default());
            }
            let mut heads = Vec::new();
            let mut stem = None;
            for e in self
                .edges
                .iter()
                .filter(|e| !e.removed && e.key.source == id)
            {
                if matches!(e.key.kind.as_str(), "Containment" | "ChordStemRelation") {
                    self.validate_chord_identity(id, e.key.target, &e.key.kind)?;
                    if self.vertices[e.key.target - 1].removed {
                        return Err("chord_identity_removed_member".into());
                    }
                    if e.key.kind == "Containment" {
                        heads.push(&self.vertices[e.key.target - 1].data);
                    } else if stem.is_none() {
                        stem = Some(&self.vertices[e.key.target - 1].data);
                    }
                }
            }
            let geometry = if self.vertices[id - 1].data["kind"] == "RestChordInter" {
                crate::head_chord_geometry::calculate_rest(&heads)?
            } else {
                crate::head_chord_geometry::calculate(&heads, stem)?
            };
            // AbstractChord.getMembers is stable bottom-up for both note kinds.
            heads.sort_by_key(|h| {
                std::cmp::Reverse(
                    h["bounds"][1].as_i64().unwrap() + h["bounds"][3].as_i64().unwrap() / 2,
                )
            });
            let mut sum = 0.;
            for head in &heads {
                let grade = if head["contextual_grade"].is_null() {
                    &head["grade"]
                } else {
                    &head["contextual_grade"]
                };
                sum += grade
                    .as_f64()
                    .filter(|g| (0.0..=1.0).contains(g))
                    .ok_or("chord_member_best_grade_required")?;
            }
            let members = heads.iter().map(|h| h["id"].clone()).collect::<Vec<_>>();
            let mean = (!heads.is_empty()).then(|| sum / heads.len() as f64);
            let stem_id = stem.map(|s| s["id"].clone()).unwrap_or(Value::Null);
            let data = &mut self.vertices[id - 1].data;
            data["grade"] = json!(mean);
            data["member_ids"] = json!(members);
            data["stem_id"] = stem_id;
            data["bounds"] = json!(geometry.bounds);
            data["leading_note_id"] = json!(geometry.leading_id);
            data["head_location"] = json!(geometry.head_location);
            data["tail_location"] = json!(geometry.tail_location);
            self.read_chord_placement(self.vertices[id - 1].key)?;
            self.invalidate_editing_chord(id)?;
        }
        Ok(())
    }
    /// Remove actual typed identity edges, not caller-supplied member metadata.
    /// Used by vertex cleanup; cached head/stem vertex removal is still owned by
    /// its dedicated lifecycle. Direct relation deletion preserves the vertices.
    pub(crate) fn remove_chord_identity_edges(&mut self, ids: &[usize]) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut affected = BTreeSet::new();
        let mut remove = BTreeSet::new();
        for id in ids {
            let e = self
                .edges
                .get(id.checked_sub(1).ok_or("invalid_chord_identity_edge")?)
                .ok_or("missing_chord_identity_edge")?;
            if !matches!(e.key.kind.as_str(), "Containment" | "ChordStemRelation") {
                return Err("typed_owned_chord_identity_required".into());
            }
            self.validate_chord_identity(e.key.source, e.key.target, &e.key.kind)?;
            if !e.removed {
                affected.insert(e.key.source);
                remove.insert(*id);
            }
        }
        let mut next = self.clone();
        for id in remove {
            let source = next.edges[id - 1].key.source;
            next.edges[id - 1].removed = true;
            if next.vertices[source - 1].chord_cache.is_some() {
                next.refresh_editing_chord_links(source);
                #[cfg(test)]
                next.dot_callback_event("edgeRemoved", id, "enter");
                next.invalidate_note_chords(&BTreeSet::from([source]))?;
                #[cfg(test)]
                next.dot_callback_event("edgeRemoved", id, "leave");
                affected.remove(&source);
            }
        }
        next.invalidate_note_chords(&affected)?;
        *self = next;
        Ok(())
    }
    pub(crate) fn remove_augmentation_edge(&mut self, id: usize) -> Result<(), String> {
        let edge = self
            .edges
            .get(id.checked_sub(1).ok_or("invalid_dot_relation_id")?)
            .ok_or("missing_dot_relation")?;
        let source = &self.vertices[edge.key.source - 1];
        let target = &self.vertices[edge.key.target - 1];
        if source.data["kind"] != "AugmentationDotInter"
            || source.key.phase != Phase::Symbol
            || target.key.system != source.key.system
            || !matches!(
                (edge.key.kind.as_str(), target.data["kind"].as_str()),
                ("AugmentationRelation", Some("RestInter" | "HeadInter"))
                    | ("DoubleDotRelation", Some("AugmentationDotInter"))
            )
        {
            return Err("music_dot_typed_note_or_double_removal_required".into());
        }
        if edge.removed {
            return Ok(());
        }
        if source.data["frozen"] == true || target.data["frozen"] == true {
            return Err("music_dot_frozen_competition_pending".into());
        }
        let mut next = self.clone();
        next.edges[id - 1].removed = true;
        #[cfg(test)]
        next.dot_callback_event("edgeRemoved", id, "enter");
        next.refresh_augmentation_abnormal(&BTreeSet::from([edge.key.source]));
        if edge.key.kind == "AugmentationRelation" && !target.removed && !target.removing {
            next.recount_augmentation_note(edge.key.target)?;
        }
        #[cfg(test)]
        next.dot_callback_event("edgeRemoved", id, "leave");
        *self = next;
        Ok(())
    }
    fn remove_dot_vertex(&mut self, id: usize, cause: Option<Value>) -> Result<(), String> {
        if self.vertices[id - 1].data["frozen"] == true {
            return Err("music_removal_frozen_inter".into());
        }
        // JGraphT edgesOf copies incoming edges before outgoing edges. The
        // removed Inter stays registered while those callbacks can reenter.
        let incident = self.incident_edges(id);
        for i in &incident {
            if !matches!(
                self.edges[i - 1].key.kind.as_str(),
                "AugmentationRelation" | "DoubleDotRelation" | "Exclusion"
            ) {
                return Err("music_dot_typed_vertex_removal_required".into());
            }
        }
        let outgoing = incident
            .iter()
            .map(|i| &self.edges[i - 1])
            .filter(|e| e.key.source == id && e.key.kind != "Exclusion")
            .collect::<Vec<_>>();
        if outgoing.len() > 1 && outgoing.iter().any(|e| e.key.kind == "DoubleDotRelation") {
            return Err("music_dot_shared_vertex_callback_pending".into());
        }
        self.remove_dot_from_measure(id)?;
        let dot = &self.vertices[id - 1];
        let mut removal = json!({"id":id,"origin":dot.key,"kind":dot.data["kind"],"shape":dot.data["shape"],"glyph_id":dot.data["glyph_id"],"bounds":dot.data["bounds"],"grade":dot.data["grade"],"contextual_grade":dot.data["contextual_grade"],"reason":"Inter.remove"});
        if let Some(cause) = cause {
            removal.as_object_mut().unwrap().extend(
                cause
                    .as_object()
                    .ok_or("music_dot_removal_cause_required")?
                    .clone(),
            );
        }
        self.vertices[id - 1].removing = true;
        for edge in incident {
            if self.edges[edge - 1].removed {
                continue;
            }
            if self.edges[edge - 1].key.kind == "Exclusion" {
                self.edges[edge - 1].removed = true;
            } else {
                self.remove_augmentation_edge(edge)?;
            }
        }
        self.vertices[id - 1].removing = false;
        self.vertices[id - 1].removed = true;
        self.dot_removals.push(removal);
        #[cfg(test)]
        {
            self.dot_callback_event("vertexRemoved", id, "enter");
            self.dot_callback_event("vertexRemoved", id, "leave");
        }
        Ok(())
    }
    pub(crate) fn pending_dot_removals(&self) -> &[Value] {
        &self.dot_removals
    }
    pub(crate) fn take_dot_removals(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.dot_removals)
    }
    fn remove_counted_dot(&mut self, id: usize, cause: Value) -> Result<(), String> {
        if self.vertices[id - 1].removed || self.vertices[id - 1].removing {
            return Ok(());
        }
        let mut next = self.clone();
        next.remove_dot_vertex(id, Some(cause))?;
        *self = next;
        Ok(())
    }
    /// Original uncached ordinary duration; does not mutate clocks or ownership.
    pub fn chord_duration(&self, key: Key) -> Result<Option<String>, String> {
        let id = self
            .id(key)
            .ok_or("musicxml_duration_live_chord_required")?;
        Ok(self.exact_tuplet_duration(id)?.map(|d| d.to_string()))
    }
    /// Original getEndTime uses the current duration and retained timeOffset.
    pub fn chord_end_time(&self, key: Key) -> Result<Option<String>, String> {
        let id = self
            .id(key)
            .ok_or("musicxml_duration_live_chord_required")?;
        let Some(duration) = self.exact_tuplet_duration(id)? else {
            return Ok(None);
        };
        let Some(offset) = self.vertices[id - 1].data["time_offset"].as_str() else {
            return Ok(None);
        };
        let end = crate::music_time::Fraction::parse(offset)?.plus(duration)?;
        Ok(Some(end.to_string()))
    }
    pub(crate) fn count_owned_augmentation(&mut self, chord: Key) -> Result<(), String> {
        let id = self.endpoint(chord)?;
        let mut next = self.clone();
        next.count_augmentation_chord(id)?;
        *self = next;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn enable_dot_callback_trace(&mut self) {
        self.trace_dot_callbacks = true;
        self.dot_callback_trace.clear();
        self.dot_callback_depth = 0;
    }
    #[cfg(test)]
    pub(crate) fn dot_callback_trace(&self) -> &[Value] {
        &self.dot_callback_trace
    }
    #[cfg(test)]
    pub(crate) fn observe_chord_callbacks(&mut self, chord: Key) {
        self.enable_dot_callback_trace();
        self.observed_chord = Some(chord);
    }
    #[cfg(test)]
    fn dot_callback_event(&mut self, event: &str, id: usize, stage: &str) {
        if !self.trace_dot_callbacks {
            return;
        }
        if stage == "leave" {
            self.dot_callback_depth -= 1;
        }
        // The original listener reads getPart/getStaff before its state row.
        // Those lazy reads can resolve a newly contained actual head at entry.
        for chord in self.observed_chords.clone() {
            self.read_chord_placement(chord).unwrap();
        }
        let mut state = self.snapshot();
        state["editing_chord_owners"] = self.editing_chord_observations();
        state["retained_voice_owners"] = self.retained_voice_observations();
        state["inters"] = json!(
            self.vertices
                .iter()
                .map(|v| {
                    let mut data = v.data.clone();
                    if let Some(cache) = &v.chord_cache {
                        data["retained_chord_cache"] = cache.snapshot();
                    }
                    data["lifecycle_removed"] = json!(v.removed || v.removing);
                    data["registered"] = json!(!v.removed);
                    data["retained_voice_key"] = json!(v.retained_voice);
                    if let Some((_, identity)) = self
                        .observed_voice_bindings
                        .iter()
                        .find(|(key, _)| *key == v.key)
                    {
                        data["same_observed_voice"] = json!(
                            v.voice_owner
                                .as_ref()
                                .is_some_and(|owner| std::sync::Arc::ptr_eq(owner, identity))
                        );
                    }
                    data
                })
                .collect::<Vec<_>>()
        );
        self.dot_callback_trace.push(json!({"event":event,"id":id,"stage":stage,"depth":self.dot_callback_depth,"state":state}));
        if event.starts_with("edge") {
            self.dot_callback_trace.last_mut().unwrap()["kind"] =
                json!(self.edges[id - 1].key.kind);
        }
        if let Some(chord) = self.observed_chord {
            // The oracle listener calls real getters after taking its raw
            // cache snapshot; these reads also warm the next callback's cache.
            let observation = self.retained_chord_observation(chord).unwrap();
            self.dot_callback_trace.last_mut().unwrap()["chord_observation"] = observation;
        }
        let observations = self.observed_chords.clone().into_iter().filter_map(|chord| {
            let id = self.id(chord)?;
            Some(serde_json::json!({"id":id,"bounds":self.lifecycle_chord_bounds(chord).unwrap()}))
        }).collect::<Vec<_>>();
        if !observations.is_empty() {
            self.dot_callback_trace.last_mut().unwrap()["chord_observations"] = json!(observations);
        }
        if stage == "enter" {
            self.dot_callback_depth += 1;
        }
    }
    fn recount_augmentation_note(&mut self, target: usize) -> Result<(), String> {
        if self.vertices[target - 1].removed {
            return Ok(());
        }
        let note = &self.vertices[target - 1];
        let (phase, class) = match note.data["kind"].as_str() {
            Some("RestInter") => (Phase::Symbol, "RestChordInter"),
            Some("HeadInter") => (Phase::Head, "HeadChordInter"),
            _ => return Err("music_dot_typed_supported_note_required".into()),
        };
        let owners = self
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.kind == "Containment" && e.key.target == target)
            .map(|e| e.key.source)
            .collect::<Vec<_>>();
        if owners.len() != 1 {
            return Err("music_dot_owned_note_chord_required".into());
        }
        let owner = owners[0];
        if self.vertices[owner - 1].data["kind"] != class || note.key.phase != phase {
            return Err("music_dot_owned_note_chord_required".into());
        }
        self.count_augmentation_chord(owner)
    }
    fn count_augmentation_chord(&mut self, owner: usize) -> Result<(), String> {
        let chord = &self.vertices[owner - 1];
        let (phase, class, note_class) = match chord.data["kind"].as_str() {
            Some("HeadChordInter") => (Phase::Head, "HeadChordInter", "HeadInter"),
            Some("RestChordInter") => (Phase::Symbol, "RestChordInter", "RestInter"),
            _ => return Err("music_dot_owned_note_chord_required".into()),
        };
        let mut members = self
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.kind == "Containment" && e.key.source == owner)
            .map(|e| e.key.target)
            .collect::<Vec<_>>();
        if chord.removed || chord.key.phase != Phase::Chord || chord.data["kind"] != class {
            return Err("music_dot_owned_note_chord_required".into());
        }
        if members.is_empty() {
            return Ok(());
        }
        if class == "RestChordInter" && members.len() != 1 {
            return Err("music_dot_owned_note_chord_required".into());
        }
        let timed = !chord.data["duration"].is_null() || !chord.data["voice"].is_null();
        if timed {
            let data = &chord.data;
            let actual_measure = self
                .dot_ownership
                .as_ref()
                .is_some_and(|owners| owners.contains_chord(chord.key.system, data));
            if !actual_measure
                || data["duration"].as_str().is_none()
                || data["time_offset"].as_str().is_none()
                || data["voice"].as_u64().is_none()
                || data["slot_id"].as_u64().is_none()
            {
                return Err("music_dot_rhythm_reset_required".into());
            }
            if class != "HeadChordInter"
                || members.len() != 1
                || self.vertices[members[0] - 1].data["shape"] != "NOTEHEAD_BLACK"
            {
                return Err("music_dot_timed_special_or_multihead_callback_pending".into());
            }
            crate::music_duration::live(&self.snapshot(), owner)?;
        }
        let ordinate = |id: &usize| -> Result<i64, String> {
            let b = &self.vertices[id - 1].data["bounds"];
            Ok(b[1].as_i64().ok_or("music_dot_bounds_required")?
                + b[3].as_i64().ok_or("music_dot_bounds_required")? / 2)
        };
        let mut ordered = members
            .iter()
            .map(|id| Ok((ordinate(id)?, *id)))
            .collect::<Result<Vec<_>, String>>()?;
        ordered.sort_by_key(|(y, _)| std::cmp::Reverse(*y));
        members = ordered.into_iter().map(|(_, id)| id).collect();
        let snapshot = self.snapshot();
        let mut counts = Vec::new();
        for id in &members {
            let member = &self.vertices[id - 1];
            if member.removed
                || member.key.phase != phase
                || member.key.system != chord.key.system
                || member.data["kind"] != note_class
            {
                return Err("music_dot_owned_note_member_required".into());
            }
            if phase == Phase::Head
                && !matches!(
                    member.data["shape"].as_str(),
                    Some("NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                )
            {
                return Err("music_dot_special_head_callback_pending".into());
            }
            counts.push(crate::music_dots::note_count(&snapshot, &member.data));
        }
        let dots =
            (counts.iter().sum::<usize>() as f64 / counts.len() as f64).round_ties_even() as usize;
        self.vertices[owner - 1].data["dots_number"] = json!(dots);
        if timed {
            // Original getters derive duration from countDots' live inputs;
            // this projection must change without resetting Voice/slot/time.
            let duration = crate::music_duration::live(&self.snapshot(), owner)?;
            self.vertices[owner - 1].data["duration"] =
                json!(duration.intrinsic.map(crate::music_time::format_ticks));
        }
        // countDots can remove other automatic dots, which calls countDots
        // again. Each nested removal consumes a live vertex; manual dots stay.
        if dots == 0 && counts.iter().any(|&c| c != 0) {
            for id in members {
                let snapshot = self.snapshot();
                if crate::music_dots::note_count(&snapshot, &self.vertices[id - 1].data) == 0 {
                    continue;
                }
                let dot_ids = self
                    .live_edges()
                    .into_iter()
                    .filter(|e| {
                        !e.removed && e.key.kind == "AugmentationRelation" && e.key.target == id
                    })
                    .map(|e| e.key.source)
                    .collect::<Vec<_>>();
                for dot_id in dot_ids {
                    let dot = &self.vertices[dot_id - 1];
                    if !dot.removed && dot.data["manual"] != true {
                        self.remove_counted_dot(dot_id,json!({"chord_id":owner,"note_id":id,"member_dot_counts":counts,"rounded_chord_count":dots,"reason":"AbstractChord.countDots_inconsistent_nonmanual_zero"}))?;
                    }
                }
            }
        }
        Ok(())
    }
    fn refresh_beam_groups(&mut self, removed: &[u64]) -> Result<(), String> {
        let mut snap = self.snapshot();
        let dying = crate::beam_groups::remove_owned_members(&mut snap, removed)?;
        for v in self
            .vertices
            .iter_mut()
            .filter(|v| !v.removed && v.data["kind"] == "BeamGroupInter")
        {
            if dying.iter().any(|id| v.data["id"] == *id) {
                v.removed = true;
            } else if let Some(n) = snap["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == v.data["id"])
            {
                v.data = n.clone();
            }
        }
        self.prune();
        Ok(())
    }
    fn prune(&mut self) {
        let pedals = self
            .live_edges()
            .iter()
            .filter(|e| {
                e.key.kind == "ChordPedalRelation"
                    && (self.vertices[e.key.source - 1].removed
                        || self.vertices[e.key.target - 1].removed)
            })
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        self.remove_pedal_edges_ordered(&pedals);
        let articulations = self
            .live_edges()
            .iter()
            .filter(|e| {
                e.key.kind == "ChordArticulationRelation"
                    && (self.vertices[e.key.source - 1].removed
                        || self.vertices[e.key.target - 1].removed)
            })
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        self.remove_articulation_edges_ordered(&articulations);
        let alters = self
            .live_edges()
            .iter()
            .filter(|e| {
                e.key.kind == "AlterHeadRelation"
                    && (self.vertices[e.key.source - 1].removed
                        || self.vertices[e.key.target - 1].removed)
            })
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        self.remove_alter_edges_ordered(&alters);
        let mut dots = BTreeSet::new();
        for e in &mut self.edges {
            if self.vertices[e.key.source - 1].removed || self.vertices[e.key.target - 1].removed {
                if !e.removed
                    && matches!(
                        e.key.kind.as_str(),
                        "AugmentationRelation" | "DoubleDotRelation"
                    )
                {
                    dots.insert(e.key.source);
                }
                e.removed = true;
            }
        }
        self.refresh_augmentation_abnormal(&dots);
    }
    fn refresh_augmentation_abnormal(&mut self, affected: &BTreeSet<usize>) {
        // Both relation callbacks recheck their surviving source dot. A second
        // dot is normal through its outgoing DoubleDot even if the first dot
        // has just lost its note; target consistency is a separate LINK check.
        let linked = self
            .edges
            .iter()
            .filter(|e| {
                !e.removed
                    && matches!(
                        e.key.kind.as_str(),
                        "AugmentationRelation" | "DoubleDotRelation"
                    )
            })
            .map(|e| e.key.source)
            .collect::<BTreeSet<_>>();
        for &id in affected {
            let v = &self.vertices[id - 1];
            if !v.removed && !v.removing && v.data["kind"] == "AugmentationDotInter" {
                self.set_owned_abnormal(id, !linked.contains(&id));
            }
        }
    }
    /// Import GRID vertices/relations in existing system and registration order.
    /// IDs in payload are rewritten; source identities remain in origin. Glyph
    /// IDs retain the explicit GRID namespace, not a fabricated shared index.
    pub fn import_grid(&mut self, grid: &Value) -> Result<(), String> {
        let mut next = self.clone();
        let systems = grid["bar_semantics"]["systems"]
            .as_array()
            .ok_or("missing_shared_sig_grid_systems")?;
        let mut seen = BTreeSet::new();
        for s in systems {
            let system = s["id"]
                .as_u64()
                .filter(|id| *id > 0)
                .ok_or("invalid_shared_sig_system")?;
            if !seen.insert(system) {
                return Err("duplicate_shared_sig_system".into());
            }
            next.require_pre_heads(system)?;
            next.phase(system, Phase::Grid);
            let mut locals = BTreeSet::new();
            for v in s["inters"]
                .as_array()
                .ok_or("missing_shared_sig_grid_inters")?
            {
                let local = v["id"]
                    .as_u64()
                    .and_then(|id| usize::try_from(id).ok())
                    .filter(|id| *id > 0)
                    .ok_or("invalid_shared_sig_local_id")?;
                if !locals.insert(local) {
                    return Err("duplicate_shared_sig_local_id".into());
                }
                let mut data = v.clone();
                data["glyph_id_namespace"] = json!("GRID");
                next.vertex(
                    Key {
                        system,
                        phase: Phase::Grid,
                        local,
                    },
                    data,
                    false,
                )?;
            }
            let mut relations = Vec::new();
            for r in s["relations"]
                .as_array()
                .ok_or("missing_shared_sig_grid_relations")?
            {
                let key = |field| -> Result<Key, String> {
                    let local = r[field]
                        .as_u64()
                        .and_then(|id| usize::try_from(id).ok())
                        .ok_or("invalid_shared_sig_grid_endpoint")?;
                    Ok(Key {
                        system,
                        phase: Phase::Grid,
                        local,
                    })
                };
                relations.push((key("source")?, key("target")?, r.clone()));
            }
            next.relations(system, Phase::Grid, relations)?;
        }
        next.prune();
        *self = next;
        Ok(())
    }
    pub fn sync_candidates(&mut self, state: &SystemCandidates) -> Result<(), String> {
        if self.vertices.iter().any(|v| {
            v.key.system == state.system_id()
                && matches!(v.key.phase, Phase::Group | Phase::Rest | Phase::Head)
                && !v.removed
        }) {
            return Err("complete_shared_sig_phase_state_required".into());
        }
        let mut next = self.clone();
        next.beams(state)?;
        next.prune();
        *self = next;
        Ok(())
    }
    fn beams(&mut self, state: &SystemCandidates) -> Result<(), String> {
        let system = state.system_id();
        self.phase(system, Phase::Beam);
        for b in state.inters() {
            self.vertex(Key{system,phase:Phase::Beam,local:b.id()},json!({"kind":b.kind,"shape":b.shape,"bounds":b.bounds,"median":b.item.median,"height":b.item.height,"grade":b.impacts.grade,"glyph_id":b.glyph_id(),"glyph_id_namespace":"BEAMS","glyph":crate::beam_dispatch::glyph_json(&b.glyph)}),b.is_removed())?;
        }
        let key = |local| Key {
            system,
            phase: Phase::Beam,
            local,
        };
        self.relations(
            system,
            Phase::Beam,
            state
                .exclusions()
                .iter()
                .map(|&(a, b)| (key(a), key(b), json!({"kind":"Exclusion"})))
                .collect(),
        )
    }
    /// Synchronize all mutable BEAMS/rest phase snapshots transactionally. Phase
    /// logic is unchanged; absent vertices and incident edges become tombstones.
    pub fn sync_system(
        &mut self,
        state: &SystemCandidates,
        groups: &GroupPass,
        rests: &RestGraph,
    ) -> Result<(), String> {
        let mut next = self.clone();
        let system = state.system_id();
        next.require_pre_heads(system)?;
        next.beams(state)?;
        next.phase(system, Phase::Group);
        next.phase(system, Phase::Rest);
        let beam = |local| Key {
            system,
            phase: Phase::Beam,
            local,
        };
        let group = |local| Key {
            system,
            phase: Phase::Group,
            local,
        };
        let rest = |local| Key {
            system,
            phase: Phase::Rest,
            local,
        };
        let mut relations = Vec::new();
        let mut ids = BTreeSet::new();
        for g in &groups.groups {
            if !ids.insert(g.id) {
                return Err("duplicate_shared_sig_group".into());
            }
            let members = g
                .members
                .iter()
                .map(|&id| next.endpoint(beam(id)))
                .collect::<Result<Vec<_>, _>>()?;
            next.vertex(
                group(g.id),
                json!({"kind":"BeamGroupInter","members":members,"bounds":g.bounds}),
                false,
            )?;
            for &id in &g.members {
                relations.push((group(g.id), beam(id), json!({"kind":"Containment"})));
            }
        }
        for &(a, b) in &groups.supports {
            relations.push((beam(a), beam(b), json!({"kind":"BeamBeamRelation"})));
        }
        next.relations(system, Phase::Group, relations)?;
        let mut ids = BTreeSet::new();
        for r in &rests.rests {
            if !ids.insert(r.id) {
                return Err("duplicate_shared_sig_rest_id".into());
            }
            let source = next
                .id(beam(r.source_beam))
                .ok_or("missing_shared_sig_rest_source")?;
            next.vertex(rest(r.id),json!({"kind":"MultipleRestInter","staff_id":r.staff_id,"median":r.median,"height":r.height,"grade":r.grade,"source_beam_id":source}),false)?;
        }
        for s in &rests.serifs {
            if !ids.insert(s.id) {
                return Err("duplicate_shared_sig_rest_id".into());
            }
            next.vertex(rest(s.id),json!({"kind":"VerticalSerifInter","glyph_id":s.glyph_id,"glyph_id_namespace":"BEAMS","median":s.plan.median,"width":s.plan.width,"grade":s.plan.peak.grade,"impacts":s.plan.peak.impacts}),false)?;
        }
        for relation in &rests.relations {
            if !rests.rests.iter().any(|r| r.id == relation.rest_id)
                || !rests
                    .serifs
                    .iter()
                    .any(|s| s.id == relation.serif_id && s.plan.left == relation.left)
            {
                return Err("invalid_shared_sig_rest_relation_types".into());
            }
        }
        next.relations(system,Phase::Rest,rests.relations.iter().map(|r|(rest(r.rest_id),rest(r.serif_id),json!({"kind":"MultipleRestSerifRelation","rest_side":if r.left{"LEFT"}else{"RIGHT"}}))).collect())?;
        next.prune();
        *self = next;
        Ok(())
    }
    /// Fresh ledger phase registration order; reduced nodes remain tombstones.
    pub fn sync_ledgers(&mut self, system: u64, ledgers: &[Value]) -> Result<(), String> {
        let mut next = self.clone();
        next.require_pre_heads(system)?;
        next.phase(system, Phase::Ledger);
        for node in ledgers {
            let local = node["id"].as_u64().ok_or("invalid_shared_ledger_id")? as usize;
            let mut data = node.clone();
            data["kind"] = json!("LedgerInter");
            data["shape"] = json!("LEDGER");
            data["glyph_id_namespace"] = json!("BEAMS");
            next.vertex(
                Key {
                    system,
                    phase: Phase::Ledger,
                    local,
                },
                data,
                node["removed"] == true,
            )?;
        }
        next.prune();
        *self = next;
        Ok(())
    }
    /// HEADS continuation on the same graph. Earlier identities remain stable;
    /// removed heads/beams and incident relations become tombstones transactionally.
    pub fn sync_heads(
        &mut self,
        system: u64,
        heads: &[Value],
        exclusions: &[Value],
        beam_changes: &[Value],
    ) -> Result<(), String> {
        if system == 0 {
            return Err("invalid_shared_head_system".into());
        }
        if self
            .vertices
            .iter()
            .any(|v| v.key.system == system && v.key.phase == Phase::Stem && !v.removed)
        {
            return Err("complete_shared_sig_stems_state_required".into());
        }
        let mut next = self.clone();
        next.phase(system, Phase::Head);
        let mut seen = BTreeSet::new();
        for h in heads {
            let local = h["id"]
                .as_u64()
                .and_then(|i| usize::try_from(i).ok())
                .filter(|i| *i > 0)
                .ok_or("invalid_shared_head_id")?;
            if !seen.insert(local) || h["kind"] != "HeadInter" {
                return Err("invalid_shared_head_vertex".into());
            }
            next.vertex(
                Key {
                    system,
                    phase: Phase::Head,
                    local,
                },
                h.clone(),
                h["removed"] == true,
            )?;
        }
        let mut changed = BTreeSet::new();
        for b in beam_changes {
            let key: Key = serde_json::from_value(b["origin"].clone())
                .map_err(|_| "invalid_shared_head_beam_origin")?;
            if key.system != system
                || key.phase != Phase::Beam
                || !changed.insert(key)
                || next.id(key) != b["id"].as_u64().map(|x| x as usize)
            {
                return Err("invalid_shared_head_beam_update".into());
            }
            next.vertex(key, b.clone(), b["removed"] == true)?;
        }
        let mut relations = Vec::new();
        let mut unique = BTreeSet::new();
        for r in exclusions {
            let a = r["source"]
                .as_u64()
                .ok_or("invalid_shared_head_exclusion")? as usize;
            let b = r["target"]
                .as_u64()
                .ok_or("invalid_shared_head_exclusion")? as usize;
            if r["kind"] != "Exclusion" || a >= b || !unique.insert((a, b)) {
                return Err("invalid_shared_head_exclusion".into());
            }
            relations.push((
                Key {
                    system,
                    phase: Phase::Head,
                    local: a,
                },
                Key {
                    system,
                    phase: Phase::Head,
                    local: b,
                },
                r.clone(),
            ));
        }
        next.relations(system, Phase::Head, relations)?;
        next.prune();
        *self = next;
        Ok(())
    }
    /// Replace a system's explicitly selected nonmanual STEMS boundary in one
    /// transaction. Existing HEADS exclusions, glyphs, staff order and other
    /// phases are retained; added/removed head–stem edges refresh abnormal
    /// state exactly as the original relation callbacks. No automatic STEMS,
    /// chords, SIG contextual grades, cleaner or epilog is certified here.
    pub fn sync_stems(
        &mut self,
        system: u64,
        stems: &[Value],
        attachments: &[HeadStemAttachment],
    ) -> Result<(), String> {
        if system == 0 || stems.len() > 100_000 || attachments.len() > 1_000_000 {
            return Err("invalid_shared_stem_system_or_budget".into());
        }
        let retained_relations = self.snapshot()["relations"].clone();
        let mut next = self.clone();
        next.phase(system, Phase::Stem);
        let mut seen = BTreeSet::new();
        for stem in stems {
            let local = stem["id"]
                .as_u64()
                .and_then(|i| usize::try_from(i).ok())
                .filter(|i| *i > 0)
                .ok_or("invalid_shared_stem_id")?;
            if !seen.insert(local) || stem["manual"] == true || stem["is_manual"] == true {
                return Err("unsupported_shared_stem_vertex".into());
            }
            let glyph = crate::beam_dispatch::glyph_from_json(&stem["glyph"])?;
            let grade = stem["grade"].as_f64().ok_or("invalid_shared_stem_grade")?;
            let expected = crate::stem_glyph::record(local, &glyph, grade)?;
            let key = Key {
                system,
                phase: Phase::Stem,
                local,
            };
            let fields = ["kind", "shape", "bounds", "median", "width"];
            let constructor = fields.iter().all(|field| stem[*field] == expected[*field]);
            let retained = self.refined_stem_geometry.get(&key).is_some_and(|actual| {
                fields
                    .iter()
                    .chain(std::iter::once(&"glyph"))
                    .all(|field| stem[*field] == actual[*field])
            });
            if !constructor && !retained {
                return Err("inconsistent_shared_stem_geometry".into());
            }
            let mut data = stem.clone();
            data["abnormal"] = if stem["removed"] == true {
                next.id(key)
                    .map(|id| next.vertices[id - 1].data["abnormal"].clone())
                    .unwrap_or(json!(true))
            } else {
                json!(true)
            };
            next.vertex(key, data, stem["removed"] == true)?;
        }
        // ChordStemRelation.removed invalidates each surviving HeadChord before
        // incident edges are pruned. Stage all vertices first, so an unchanged
        // stem is live even if it was registered after a removed stem.
        let chord_edges = next
            .edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                (!e.removed
                    && e.key.system == system
                    && e.key.kind == "ChordStemRelation"
                    && next.vertices[e.key.target - 1].removed)
                    .then_some(i + 1)
            })
            .collect::<Vec<_>>();
        next.remove_chord_identity_edges(&chord_edges)?;
        let mut relation_ids = BTreeSet::new();
        let mut relations = Vec::new();
        for attachment in attachments {
            if attachment.local == 0
                || !relation_ids.insert(attachment.local)
                || attachment.head.system != system
                || attachment.head.phase != Phase::Head
            {
                return Err("invalid_shared_head_stem_attachment".into());
            }
            let source = next.endpoint(attachment.head)?;
            let target_key = Key {
                system,
                phase: Phase::Stem,
                local: attachment.stem_local,
            };
            let target = next.endpoint(target_key)?;
            let head = &next.vertices[source - 1].data;
            if head["kind"] != "HeadInter"
                || !head["shape"]
                    .as_str()
                    .is_some_and(|s| crate::head_seed_search::ALL_STEM_SHAPES.contains(&s))
                || head["manual"] == true
                || head["is_manual"] == true
            {
                return Err("unsupported_shared_stem_head".into());
            }
            let r = &attachment.relation;
            if r["kind"] != "HeadStemRelation"
                || r["manual"] == true
                || r["is_manual"] == true
                || !r["head_side"]
                    .as_str()
                    .is_some_and(|s| ["LEFT", "RIGHT"].contains(&s))
            {
                return Err("unsupported_shared_head_stem_relation".into());
            }
            let point: [f64; 2] = serde_json::from_value(r["extension_point"].clone())
                .map_err(|_| "invalid_shared_stem_extension_point")?;
            if point.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.) {
                return Err("unsupported_shared_stem_relation_context".into());
            }
            let profile = r["profile"]
                .as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or("invalid_shared_stem_profile")?;
            let dx = r["dx"].as_f64().ok_or("invalid_shared_stem_gap")?;
            let dy = r["dy"].as_f64().ok_or("invalid_shared_stem_gap")?;
            let mut expected = crate::head_stem_relation::gap_grade(dx, dy, profile)?;
            // Refinement changes the median, but never resets the existing
            // relation's constructor-time consistency or support ratios.
            let preserved = self.refined_stem_geometry.contains_key(&target_key)
                && retained_relations.as_array().unwrap().iter().any(|old| {
                    old == r
                        && old["source"] == source
                        && old["target"] == target
                        && old["local_relation_id"] == attachment.local
                });
            let consistency = if preserved {
                r["consistency"]
                    .as_f64()
                    .filter(|v| v.is_finite() && *v > 0.)
                    .ok_or("invalid_shared_retained_stem_consistency")?
            } else if let Some(main) = r.get("consistency_interline") {
                let interline = main
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or("invalid_shared_stem_consistency_interline")?;
                let median: [f64; 4] =
                    serde_json::from_value(next.vertices[target - 1].data["median"].clone())
                        .map_err(|_| "invalid_shared_stem_consistency_median")?;
                crate::head_stem_relation::measured_consistency(
                    head["shape"].as_str().unwrap(),
                    median,
                    interline,
                )?
            } else {
                1.
            };
            if !r["consistency"]
                .as_f64()
                .is_some_and(|v| v.is_finite() && (v - consistency).abs() < 1e-12)
            {
                return Err("inconsistent_shared_stem_consistency".into());
            }
            expected["source_ratio"] =
                json!(1. + (4. * consistency) * expected["grade"].as_f64().unwrap());
            if expected["eligible"] != true {
                return Err("ineligible_shared_stem_relation".into());
            }
            for field in ["grade", "source_ratio", "target_ratio"] {
                if !r[field].as_f64().is_some_and(|v| {
                    v.is_finite() && (v - expected[field].as_f64().unwrap()).abs() < 1e-12
                }) {
                    return Err("inconsistent_shared_stem_relation_grade".into());
                }
            }
            if r["invading"].as_bool() != Some(crate::head_stem_relation::is_invading(dx, dy)?) {
                return Err("inconsistent_shared_stem_invading".into());
            }
            if next.edges.iter().any(|e| {
                e.key.system == system
                    && e.key.phase == Phase::Stem
                    && e.data["local_relation_id"] == attachment.local
                    && (e.key.source != source || e.key.target != target)
            }) {
                return Err("changed_shared_stem_relation_endpoints".into());
            }
            let mut data = r.clone();
            data["local_relation_id"] = json!(attachment.local);
            relations.push((attachment.head, target_key, data));
        }
        next.relations(system, Phase::Stem, relations)?;
        next.prune();
        let linked = next
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.system == system && e.key.kind == "HeadStemRelation")
            .flat_map(|e| [e.key.source, e.key.target])
            .collect::<BTreeSet<_>>();
        for (i, v) in next.vertices.iter_mut().enumerate() {
            if v.removed || v.key.system != system {
                continue;
            }
            if v.key.phase == Phase::Stem
                || (v.key.phase == Phase::Head
                    && v.data["shape"]
                        .as_str()
                        .is_some_and(|s| crate::head_seed_search::ALL_STEM_SHAPES.contains(&s)))
            {
                v.data["abnormal"] = json!(!linked.contains(&(i + 1)));
            }
        }
        *self = next;
        Ok(())
    }
    // Bounded native head/stem support computation after image linking. Prior
    // GRID/ledger nodes are retained; no reduction or completion flag is implied.
    pub(crate) fn contextualize_head_stems(&mut self) -> Result<(), String> {
        self.contextualize_head_stems_in(None)
    }
    pub(crate) fn contextualize_system_head_stems(&mut self, system: u64) -> Result<(), String> {
        self.contextualize_head_stems_in(Some(system))
    }
    fn contextualize_head_stems_in(&mut self, system: Option<u64>) -> Result<(), String> {
        let mut nodes = self
            .vertices
            .iter()
            .filter(|v| {
                !v.removed
                    && system.is_none_or(|id| v.key.system == id)
                    && (matches!(v.key.phase, Phase::Head | Phase::Stem)
                        || matches!(
                            v.data["kind"].as_str(),
                            Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
                        ))
            })
            .map(|v| v.data.clone())
            .collect::<Vec<_>>();
        let ids = nodes
            .iter()
            .map(|v| v["id"].as_u64().unwrap() as usize)
            .collect::<BTreeSet<_>>();
        let mut relations = Vec::new();
        for e in self.edges.iter().filter(|e| !e.removed) {
            if matches!(
                e.data["kind"].as_str(),
                Some("Containment" | "ChordStemRelation")
            ) {
                continue;
            }
            let source = ids.contains(&e.key.source);
            let target = ids.contains(&e.key.target);
            if source != target {
                // Original computeContextualGrade obtains Support relations;
                // an exclusion to a retained GRID barline contributes nothing.
                // Keep that edge in the live SIG for reduceExclusions, while
                // still rejecting unsupported external supports/other classes.
                let external = if source { e.key.target } else { e.key.source };
                if e.key.kind == "Exclusion"
                    && matches!(
                        self.vertices[external - 1].data["kind"].as_str(),
                        Some("BarlineInter" | "BarConnectorInter" | "BraceInter")
                    )
                {
                    continue;
                }
                return Err("unsupported_head_stem_context_external_relation".into());
            }
            if source {
                relations.push(e.data.clone());
            }
        }
        crate::sig_context::contextualize_head_stems(&mut nodes, &relations)?;
        for node in nodes {
            let id = node["id"].as_u64().unwrap() as usize;
            self.vertices[id - 1].data["contextual_grade"] = node["contextual_grade"].clone();
        }
        Ok(())
    }

    pub(crate) fn contextualize_head_stem_flags(&mut self) -> Result<(), String> {
        let mut nodes = self
            .vertices
            .iter()
            .filter(|v| {
                !v.removed
                    && matches!(
                        v.data["kind"].as_str(),
                        Some(
                            "HeadInter"
                                | "StemInter"
                                | "BeamInter"
                                | "BeamHookInter"
                                | "SmallBeamInter"
                                | "FlagInter"
                        )
                    )
            })
            .map(|v| v.data.clone())
            .collect::<Vec<_>>();
        let ids = nodes
            .iter()
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect::<BTreeSet<_>>();
        let mut relations = Vec::new();
        for e in self.edges.iter().filter(|e| !e.removed) {
            // Original getSupports excludes non-Support identity relations.
            if matches!(
                e.data["kind"].as_str(),
                Some("Containment" | "ChordStemRelation")
            ) {
                continue;
            }
            // Accidental support has a zero target coefficient. Linking an
            // Alter cannot change existing Head/Stem contextual grades.
            if e.key.kind == "AlterHeadRelation" && e.data["target_ratio"] == 1.0 {
                self.validate_alter_head(e.key.source, e.key.target, &e.data)?;
                continue;
            }
            if e.key.kind == "ChordArticulationRelation" && e.data["source_ratio"] == 1.0 {
                self.validate_chord_articulation(e.key.source, e.key.target, &e.data)?;
                continue;
            }
            if e.key.kind == "ChordOrnamentRelation" && e.data["source_ratio"] == 1.0 {
                self.validate_chord_ornament(e.key.source, e.key.target, &e.data)?;
                continue;
            }
            // Augmentation supports the dot only (target coefficient zero).
            // Its separate contextualization cannot change the note/Stem grades.
            if e.key.kind == "AugmentationRelation" && e.data["target_ratio"] == 1.0 {
                if self.vertices[e.key.source - 1].data["kind"] != "AugmentationDotInter"
                    || !matches!(
                        self.vertices[e.key.target - 1].data["kind"].as_str(),
                        Some("HeadInter" | "RestInter")
                    )
                {
                    return Err("music_dot_typed_support_required".into());
                }
                continue;
            }
            let source = ids.contains(&e.key.source);
            let target = ids.contains(&e.key.target);
            // Exclusion/NoExclusion are not support. With an external endpoint
            // they cannot conflict between this projection's support partners.
            // Keep the real edge for subsequent recognition conflict reduction.
            if source != target && matches!(e.key.kind.as_str(), "Exclusion" | "NoExclusion") {
                continue;
            }
            if source != target {
                return Err("unsupported_head_stem_flag_external_relation".into());
            }
            if source {
                relations.push(e.data.clone())
            }
        }
        crate::sig_context::contextualize_head_stem_flags(&mut nodes, &relations)?;
        for n in nodes {
            let id = n["id"].as_u64().unwrap() as usize;
            self.vertices[id - 1].data["contextual_grade"] = n["contextual_grade"].clone();
        }
        Ok(())
    }

    /// Raw Graph callers cannot commit a cached StemInter removal. The image
    /// owner uses the staged pass below and updates the same cache objects.
    #[cfg(test)]
    pub(crate) fn purge_weak_foundations(&mut self) -> Result<Vec<usize>, String> {
        let (next, remove) = self.prepare_weak_foundations()?;
        if remove
            .iter()
            .any(|id| self.vertices[*id - 1].data["kind"] == "StemInter")
        {
            return Err("weak_stem_requires_owned_cache_removal_callback".into());
        }
        *self = next;
        Ok(remove)
    }

    /// Original SIGraph.deleteWeakInters selection/callbacks, staged without
    /// mutating live identities. Only the retained owner can commit this graph
    /// after resolving and locking every affected canonical cached stem.
    pub(crate) fn prepare_weak_foundations(&self) -> Result<(Self, Vec<usize>), String> {
        let mut remove = Vec::new();
        for (i, v) in self.vertices.iter().enumerate().filter(|(_, v)| !v.removed) {
            if matches!(v.data["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                && (v.key.phase != Phase::Beam
                    || !v.data["grade"]
                        .as_f64()
                        .is_some_and(|g| (0.0..=1.0).contains(&g)))
            {
                return Err("invalid_owned_weak_foundation_beam".into());
            }
            if v.data["kind"] == "BeamGroupInter"
                || v.data["frozen"] == true
                || v.data["is_frozen"] == true
                || v.data["shape"] == "LEDGER"
            {
                continue;
            }
            if !matches!(
                v.data["kind"].as_str(),
                Some(
                    "HeadInter"
                        | "StemInter"
                        | "BeamInter"
                        | "BeamHookInter"
                        | "BarlineInter"
                        | "BarConnectorInter"
                        | "BraceInter"
                )
            ) {
                return Err(format!(
                    "unsupported_weak_foundation_kind:{}",
                    v.data["kind"].as_str().unwrap_or("unknown")
                ));
            }
            let grade = v.data["contextual_grade"]
                .as_f64()
                .filter(|g| (0.0..=1.0).contains(g))
                .ok_or("missing_validated_foundation_contextual_grade")?;
            if grade < 0.5 {
                remove.push(i + 1);
            }
        }
        let mut next = self.clone();
        for id in &remove {
            let vertex = &next.vertices[*id - 1];
            if matches!(
                vertex.data["kind"].as_str(),
                Some("BarConnectorInter" | "BraceInter")
            ) {
                next.remove_foundation_grid_member(vertex.key)?;
            } else {
                next.vertices[*id - 1].removed = true;
            }
        }
        next.prune();
        next.refresh_beam_groups(&remove.iter().map(|id| *id as u64).collect::<Vec<_>>())?;
        // Original HeadStemRelation.removed callbacks refresh each live endpoint.
        let linked = next
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.kind == "HeadStemRelation")
            .flat_map(|e| [e.key.source, e.key.target])
            .collect::<BTreeSet<_>>();
        for (i, v) in next
            .vertices
            .iter_mut()
            .enumerate()
            .filter(|(_, v)| !v.removed)
        {
            if v.data["kind"] == "StemInter"
                || (v.data["kind"] == "HeadInter"
                    && v.data["shape"]
                        .as_str()
                        .is_some_and(|s| crate::head_seed_search::ALL_STEM_SHAPES.contains(&s)))
            {
                v.data["abnormal"] = json!(!linked.contains(&(i + 1)));
            }
        }
        Ok((next, remove))
    }

    /// Original canonical-ID insertSupport/insertExclusion for the recognized
    /// ordinary chord subset. Existing support/exclusion prevents its opposite.
    pub(crate) fn insert_foundation_contact(
        &mut self,
        a: Key,
        b: Key,
        kind: &str,
        cause: Option<&str>,
    ) -> Result<bool, String> {
        let ai = self.endpoint(a)?;
        let bi = self.endpoint(b)?;
        if a.system != b.system || ai == bi {
            return Err("invalid_foundation_contact_endpoints".into());
        }
        let ak = self.vertices[ai - 1].data["kind"].as_str();
        let bk = self.vertices[bi - 1].data["kind"].as_str();
        let valid = match kind {
            "HeadHeadRelation" => {
                ak == Some("HeadInter") && bk == Some("HeadInter") && cause.is_none()
            }
            "Exclusion" => {
                matches!(cause, Some("OVERLAP" | "INCOMPATIBLE"))
                    && [ak, bk].iter().all(|kind| {
                        matches!(
                            kind,
                            Some(
                                "HeadInter"
                                    | "HeadChordInter"
                                    | "StemInter"
                                    | "BeamInter"
                                    | "BeamHookInter"
                                    | "SmallBeamInter"
                                    | "BarlineInter"
                                    | "BarConnectorInter"
                                    | "BraceInter"
                            )
                        )
                    })
            }
            _ => false,
        };
        if !valid {
            return Err("unsupported_foundation_contact_kind".into());
        }
        for edge in self.edges.iter().filter(|e| {
            !e.removed
                && ((e.key.source == ai && e.key.target == bi)
                    || (e.key.source == bi && e.key.target == ai))
        }) {
            if edge.key.kind == kind
                || edge.key.kind == "Exclusion"
                || (kind == "Exclusion"
                    && matches!(
                        edge.key.kind.as_str(),
                        "HeadStemRelation"
                            | "HeadHeadRelation"
                            | "BeamStemRelation"
                            | "BeamHeadRelation"
                            | "BeamBeamRelation"
                            | "MirrorRelation"
                            | "NoExclusion"
                            | "BarConnectionRelation"
                    ))
            {
                return Ok(false);
            }
        }
        let (source, target) = if ai < bi { (a, b) } else { (b, a) };
        let mut record = json!({"kind":kind});
        if let Some(c) = cause {
            record["cause"] = json!(c);
        }
        self.relations(a.system, Phase::Head, vec![(source, target, record)])?;
        Ok(true)
    }

    // Apply an automatic plan to the private identity store, preserving all keys
    // and tombstones. The owner locks canonical cache states before this call.
    pub(crate) fn apply_head_foundation_plan(
        &mut self,
        plan: &crate::stem_head_foundation_plan::Plan,
    ) -> Result<(), String> {
        use crate::stem_head_foundation_plan::Event;
        let mut next = self.clone();
        for event in &plan.events {
            match *event {
                Event::RemoveVertex(id) => {
                    let candidate = next
                        .vertices
                        .get(id.checked_sub(1).ok_or("invalid_foundation_vertex_id")? as usize)
                        .ok_or("missing_foundation_vertex")?;
                    if matches!(
                        candidate.data["kind"].as_str(),
                        Some("BarlineInter" | "BarConnectorInter" | "BraceInter")
                    ) {
                        next.remove_foundation_grid_member(candidate.key)?;
                        continue;
                    }
                    let v = next
                        .vertices
                        .get_mut(id.checked_sub(1).ok_or("invalid_foundation_vertex_id")? as usize)
                        .ok_or("missing_foundation_vertex")?;
                    if v.removed
                        || !matches!(
                            v.data["kind"].as_str(),
                            Some(
                                "HeadInter"
                                    | "StemInter"
                                    | "BeamInter"
                                    | "BeamHookInter"
                                    | "BeamGroupInter"
                            )
                        )
                    {
                        return Err(format!(
                            "unsupported_foundation_vertex_removal:{}",
                            json!({"id":id,"kind":v.data["kind"],"bounds":v.data["bounds"],"origin":v.key,"staff_id":v.data["staff_id"],"removed":v.removed})
                        ));
                    }
                    v.removed = true;
                    next.prune();
                    next.refresh_beam_groups(&[id])?;
                }
                Event::RemoveRelation(id) => {
                    let e = next
                        .edges
                        .get_mut(id.checked_sub(1).ok_or("invalid_foundation_edge_id")? as usize)
                        .ok_or("missing_foundation_edge")?;
                    if e.removed || e.key.kind != "HeadStemRelation" {
                        return Err("unsupported_foundation_relation_removal".into());
                    }
                    e.removed = true;
                }
                Event::Exclusion(a, b) => {
                    let source = next
                        .vertices
                        .get(a.checked_sub(1).ok_or("invalid_foundation_exclusion_id")? as usize)
                        .ok_or("missing_foundation_exclusion_vertex")?;
                    let target = next
                        .vertices
                        .get(b.checked_sub(1).ok_or("invalid_foundation_exclusion_id")? as usize)
                        .ok_or("missing_foundation_exclusion_vertex")?;
                    if source.removed
                        || target.removed
                        || source.key.system != target.key.system
                        || source.key.phase != Phase::Head
                        || target.key.phase != Phase::Stem
                    {
                        return Err("invalid_foundation_exclusion_endpoints".into());
                    }
                    if next.edges.iter().any(|e| {
                        !e.removed
                            && e.key.kind == "Exclusion"
                            && ((e.key.source == a as usize && e.key.target == b as usize)
                                || (e.key.source == b as usize && e.key.target == a as usize))
                    }) {
                        continue;
                    }
                    let system = source.key.system;
                    let s = source.key;
                    let t = target.key;
                    next.relations(
                        system,
                        Phase::Head,
                        vec![(s, t, json!({"kind":"Exclusion","cause":"OVERLAP"}))],
                    )?;
                }
            }
        }
        let final_nodes = plan.final_state["inters"]
            .as_array()
            .ok_or("missing_foundation_final_nodes")?;
        if final_nodes.len() != next.vertices.iter().filter(|v| !v.removed).count() {
            return Err("inconsistent_foundation_final_vertex_count".into());
        }
        for n in final_nodes {
            let id = n["id"].as_u64().ok_or("invalid_foundation_final_id")? as usize;
            let v = next
                .vertices
                .get_mut(id.checked_sub(1).ok_or("invalid_foundation_final_id")?)
                .ok_or("missing_foundation_final_vertex")?;
            if v.removed {
                return Err("removed_foundation_final_vertex".into());
            }
            let mut expected = v.data.clone();
            if matches!(v.key.phase, Phase::Head | Phase::Stem) {
                expected["contextual_grade"] = n["contextual_grade"].clone();
                expected["abnormal"] = n["abnormal"].clone();
            }
            if matches!(v.data["kind"].as_str(), Some("BeamInter" | "BeamHookInter")) {
                expected["contextual_grade"] = n["contextual_grade"].clone();
            }
            if expected != *n {
                return Err("changed_foundation_intrinsic_vertex".into());
            }
            v.data = expected;
        }
        let expected_edges = plan.final_state["relations"]
            .as_array()
            .ok_or("missing_foundation_final_edges")?;
        if expected_edges.len() != next.edges.iter().filter(|e| !e.removed).count() {
            return Err("inconsistent_foundation_final_edge_count".into());
        }
        for e in expected_edges {
            let id = e["id"].as_u64().ok_or("invalid_foundation_final_edge_id")?;
            if e["planned_synthetic_id"] == true {
                if !next.edges.iter().any(|r| {
                    !r.removed
                        && r.data["kind"] == e["kind"]
                        && r.data["source"] == e["source"]
                        && r.data["target"] == e["target"]
                        && r.data["cause"] == e["cause"]
                }) {
                    return Err("missing_foundation_new_exclusion".into());
                }
            } else {
                let actual = next
                    .edges
                    .get(
                        id.checked_sub(1)
                            .ok_or("invalid_foundation_final_edge_id")?
                            as usize,
                    )
                    .ok_or("missing_foundation_final_edge")?;
                if actual.removed || actual.data != *e {
                    return Err("changed_foundation_retained_edge".into());
                }
            }
        }
        *self = next;
        Ok(())
    }

    fn live_edges(&self) -> Vec<&Edge> {
        let mut live = self.edges.iter().filter(|e| !e.removed).collect::<Vec<_>>();
        live.sort_by_key(|e| e.live_order);
        live
    }
    /// Voices.getTiedId: first-Chord notes in ordinate order, live incident
    /// SlurHead relations, RIGHT tie, LEFT head, then its first owned ensemble.
    pub(crate) fn incoming_tied_voice_id(
        &self,
        id: usize,
        mut voice_of: impl FnMut(usize) -> Result<Option<usize>, String>,
    ) -> Result<Option<usize>, String> {
        let chord = self
            .vertices
            .get(
                id.checked_sub(1)
                    .ok_or("music_refine_chord_identity_required")?,
            )
            .filter(|v| !v.removed)
            .ok_or("music_refine_live_chord_required")?;
        if !matches!(
            chord.data["kind"].as_str(),
            Some("HeadChordInter" | "RestChordInter")
        ) {
            return Err("music_refine_typed_chord_required".into());
        }
        let mut notes: Vec<_> = self
            .live_edges()
            .into_iter()
            .filter(|e| e.key.kind == "Containment" && e.key.source == id)
            .map(|e| e.key.target)
            .collect();
        notes.sort_by_key(|id| {
            let bounds = &self.vertices[id - 1].data["bounds"];
            std::cmp::Reverse(bounds[1].as_i64().unwrap() + bounds[3].as_i64().unwrap() / 2)
        });
        for note in notes {
            if self.vertices[note - 1].data["kind"] != "HeadInter" {
                continue;
            }
            for eid in self.incident_edges(note) {
                let e = &self.edges[eid - 1];
                if e.key.kind != "SlurHeadRelation" || e.data["side"] != "RIGHT" {
                    continue;
                }
                let slur = if e.key.source == note {
                    e.key.target
                } else {
                    e.key.source
                };
                let v = &self.vertices[slur - 1];
                if v.data["kind"] != "SlurInter" || v.key.system != chord.key.system {
                    return Err("music_refine_owned_slur_required".into());
                }
                if v.data["is_tie"] != true {
                    continue;
                }
                let left = self.incident_edges(slur).into_iter().find_map(|eid| {
                    let e = &self.edges[eid - 1];
                    (e.key.kind == "SlurHeadRelation" && e.data["side"] == "LEFT").then_some(
                        if e.key.source == slur {
                            e.key.target
                        } else {
                            e.key.source
                        },
                    )
                });
                let Some(left) = left else {
                    continue;
                };
                if self.vertices[left - 1].data["kind"] != "HeadInter" {
                    return Err("music_refine_typed_tied_head_required".into());
                }
                if let Some(parent) = self.incident_edges(left).into_iter().find_map(|eid| {
                    let e = &self.edges[eid - 1];
                    (e.key.kind == "Containment" && e.key.target == left).then_some(e.key.source)
                }) {
                    if self.vertices[parent - 1].data["kind"] != "HeadChordInter" {
                        return Err("music_refine_ordinary_tied_chord_required".into());
                    }
                    if let Some(id) = voice_of(parent)? {
                        return Ok(Some(id));
                    }
                }
            }
        }
        Ok(None)
    }
    pub(crate) fn next_in_voice_neighbors(&self, id: usize) -> Result<Vec<usize>, String> {
        let vertex = self
            .vertices
            .get(
                id.checked_sub(1)
                    .ok_or("music_refine_chord_identity_required")?,
            )
            .ok_or("music_refine_chord_identity_required")?;
        if vertex.removed {
            return Ok(Vec::new());
        }
        if !matches!(
            vertex.data["kind"].as_str(),
            Some("HeadChordInter" | "RestChordInter")
        ) {
            return Err("music_refine_typed_chord_required".into());
        }
        let mut others = Vec::new();
        for eid in self.incident_edges(id) {
            let edge = &self.edges[eid - 1];
            if edge.key.kind != "NextInVoiceRelation" {
                continue;
            }
            let other = if edge.key.source == id {
                edge.key.target
            } else {
                edge.key.source
            };
            if !matches!(
                self.vertices[other - 1].data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            ) {
                return Err("music_refine_next_typed_chord_required".into());
            }
            others.push(other);
        }
        Ok(others)
    }
    fn incident_edges(&self, id: usize) -> Vec<usize> {
        let mut incident = Vec::new();
        // JGraphT keeps live insertion order independently of stable relation IDs.
        // edgesOf copies incoming first, then outgoing, without duplicating loops.
        for incoming in [true, false] {
            let mut edges = self
                .edges
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    !e.removed
                        && if incoming {
                            e.key.target == id
                        } else {
                            e.key.source == id
                        }
                })
                .collect::<Vec<_>>();
            edges.sort_by_key(|(_, e)| e.live_order);
            for (i, _) in edges {
                if !incident.contains(&(i + 1)) {
                    incident.push(i + 1);
                }
            }
        }
        incident
    }

    pub fn snapshot(&self) -> Value {
        let mut snapshot = json!({"schema":"audiveris-native.shared-sig.v1","id_namespace":"native shared registration-order vertex and relation IDs; not JVM IDs","inters":self.vertices.iter().filter(|v|!v.removed).map(|v|{let mut data=v.data.clone();if let Some(cache)=&v.chord_cache{data["retained_chord_cache"]=cache.snapshot();}data}).collect::<Vec<_>>(),"relations":self.live_edges().into_iter().map(|e|e.data.clone()).collect::<Vec<_>>(),"tombstones":self.vertices.iter().enumerate().filter(|(_,v)|v.removed).map(|(i,v)|json!({"id":i+1,"origin":v.key})).collect::<Vec<_>>(),"musicxml_complete":false});
        if self.dot_ownership.is_some() {
            snapshot["augmentation_dot_ownership"] = self.dot_ownership_snapshot();
        }
        if let Some(book) = self.editing_book_state() {
            snapshot["editing_book"] = book;
        }
        if !self.edit_publications.is_empty() {
            snapshot["edit_publications"] = json!(self.edit_publications);
        }
        if !self.edit_impacts.is_empty() {
            snapshot["edit_impacts"] = json!(self.edit_impacts);
        }
        snapshot
    }
}

#[cfg(test)]
mod chord_identity_tests {
    use super::*;
    #[test]
    fn original_member_and_stem_identity_callbacks_keep_mean_grade_geometry_and_vertices() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-chord-containment-lifecycle/controls.json"
        ))
        .unwrap();
        let mut graph = Graph::default();
        let mut keys = BTreeMap::new();
        for (i, n) in oracle["nodes"].as_array().unwrap().iter().enumerate() {
            let key = Key {
                system: 1,
                phase: if n["kind"] == "HeadInter" {
                    Phase::Head
                } else {
                    Phase::Stem
                },
                local: i + 1,
            };
            graph.vertex(key, n.clone(), false).unwrap();
            keys.insert(n["label"].as_str().unwrap(), key);
        }
        let chord = Key {
            system: 1,
            phase: Phase::Chord,
            local: 1,
        };
        let cid = graph
            .vertex(chord, json!({"kind":"HeadChordInter","grade":0.8}), false)
            .unwrap();
        for expected in oracle["states"].as_array().unwrap() {
            let (kind, target, remove) = match expected["operation"].as_str().unwrap() {
                "add-first-member" => ("Containment", keys["head0"], false),
                "add-second-member" => ("Containment", keys["head1"], false),
                "add-stem" => ("ChordStemRelation", keys["stem"], false),
                "remove-second-member" => ("Containment", keys["head1"], true),
                "remove-stem" => ("ChordStemRelation", keys["stem"], true),
                "remove-last-member" => ("Containment", keys["head0"], true),
                _ => panic!("unknown original operation"),
            };
            if remove {
                let edge = graph.snapshot()["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| {
                        r["kind"] == kind
                            && r["source"] == cid
                            && r["target"] == graph.id(target).unwrap()
                    })
                    .unwrap()["id"]
                    .as_u64()
                    .unwrap() as usize;
                graph.remove_chord_identity_edges(&[edge]).unwrap();
                let after = graph.snapshot();
                graph.remove_chord_identity_edges(&[edge]).unwrap();
                assert_eq!(graph.snapshot(), after);
            } else {
                graph
                    .relations(1, Phase::Chord, vec![(chord, target, json!({"kind":kind}))])
                    .unwrap();
            }
            let snapshot = graph.snapshot();
            let n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == cid)
                .unwrap();
            let state = &expected["state"];
            for field in ["grade", "contextual_grade", "bounds"] {
                assert_eq!(n[field], state[field], "{}:{field}", expected["operation"]);
            }
            let member_labels = n["member_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    keys.iter()
                        .find(|(_, k)| graph.id(**k) == Some(id.as_u64().unwrap() as usize))
                        .unwrap()
                        .0
                })
                .collect::<Vec<_>>();
            assert_eq!(json!(member_labels), state["members"]);
            assert_eq!(
                n["leading_note_id"],
                state["leading"]
                    .as_str()
                    .map(|l| json!(graph.id(keys[l]).unwrap()))
                    .unwrap_or(Value::Null)
            );
            for field in ["head_location", "tail_location"] {
                let actual: Option<[i64; 2]> = serde_json::from_value(n[field].clone()).unwrap();
                let expected: Option<[f64; 2]> =
                    serde_json::from_value(state[field].clone()).unwrap();
                assert_eq!(actual.map(|p| p.map(|x| x as f64)), expected);
            }
            for original in oracle["nodes"].as_array().unwrap() {
                let id = graph.id(keys[original["label"].as_str().unwrap()]).unwrap();
                let actual = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap();
                for field in ["bounds", "grade", "contextual_grade", "glyph"] {
                    assert_eq!(actual[field], original[field]);
                }
            }
        }
        // A malformed later identity does not commit earlier valid edges.
        let before = graph.snapshot();
        assert!(
            graph
                .relations(
                    1,
                    Phase::Chord,
                    vec![
                        (chord, keys["head0"], json!({"kind":"Containment"})),
                        (chord, keys["stem"], json!({"kind":"Containment"}))
                    ]
                )
                .is_err()
        );
        assert_eq!(graph.snapshot(), before);
        // setGrade preserves an already computed contextual grade.
        let mut n = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == cid)
            .unwrap()
            .clone();
        n["contextual_grade"] = json!(0.95);
        graph.vertex(chord, n, false).unwrap();
        graph
            .relations(
                1,
                Phase::Chord,
                vec![(chord, keys["head0"], json!({"kind":"Containment"}))],
            )
            .unwrap();
        let snapshot = graph.snapshot();
        let n = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == cid)
            .unwrap();
        assert_eq!(n["grade"], 0.6);
        assert_eq!(n["contextual_grade"], 0.95);
    }
}

#[cfg(test)]
mod rest_chord_identity_tests {
    use super::*;
    fn observed(graph: &Graph, labels: &BTreeMap<usize, String>) -> Value {
        let snapshot = graph.snapshot();
        let live = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                let id = n["id"].as_u64().unwrap() as usize;
                let mut data = json!({"label":labels[&id],"kind":n["kind"],
                "bounds":n["bounds"],"grade":n["grade"],"contextual_grade":n["contextual_grade"]});
                if n["kind"] == "RestInter" {
                    data["shape"] = n["shape"].clone();
                    data["glyph"] = n["glyph"].clone();
                } else {
                    data["member_labels"] = json!(
                        n["member_ids"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|id| labels[&(id.as_u64().unwrap() as usize)].clone())
                            .collect::<Vec<_>>()
                    );
                    for field in ["head_location", "tail_location"] {
                        let point: Option<[i64; 2]> =
                            serde_json::from_value(n[field].clone()).unwrap();
                        data[field] = json!(point.map(|p| p.map(|v| v as f64)));
                    }
                }
                data
            })
            .collect::<Vec<_>>();
        let removed = snapshot["tombstones"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| labels[&(n["id"].as_u64().unwrap() as usize)].clone())
            .collect::<Vec<_>>();
        let relations = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                json!({
            "kind":e["kind"],"source":labels[&(e["source"].as_u64().unwrap() as usize)],
            "target":labels[&(e["target"].as_u64().unwrap() as usize)]})
            })
            .collect::<Vec<_>>();
        json!({"live":live,"removed":removed,"relations":relations})
    }
    fn container_labels(projection: &Value, labels: &BTreeMap<usize, String>) -> Value {
        let names = |ids: &Value| {
            ids.as_array()
                .unwrap()
                .iter()
                .map(|id| labels[&(id.as_u64().unwrap() as usize)].clone())
                .collect::<Vec<_>>()
        };
        json!({"staff_note_labels":names(&projection["staff_notes"][0]["note_ids"]),
            "measure_registration":projection["measure_chords"].as_array().unwrap().iter()
                .map(|m| json!({"rest_chord_labels":names(&m["rest_chord_ids"])})).collect::<Vec<_>>()})
    }
    #[test]
    fn original_rest_chord_partial_dying_extensive_and_shared_member_lifecycle() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rest-chord-lifecycle/controls.json"
        ))
        .unwrap();
        let containers: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rest-chord-lifecycle/containers.json"
        ))
        .unwrap();
        let measures = vec![crate::music_measures::Measure {
            geometry: None,
            system: 1,
            part: 1,
            staff: 1,
            index: 0,
            left: 200,
            right: 400,
            timing: Default::default(),
            tuplets: Vec::new(),
            stack_tuplets: Vec::new(),
        }];
        for case in oracle["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let mut keys = BTreeMap::new();
            let mut labels = BTreeMap::new();
            for (i, n) in case["initial"]["live"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let key = Key {
                    system: 1,
                    phase: if n["kind"] == "RestInter" {
                        Phase::Symbol
                    } else {
                        Phase::Chord
                    },
                    local: i + 1,
                };
                let mut data = n.clone();
                data["staff_id"] = json!(1);
                if n["kind"] == "RestChordInter" {
                    data["part_id"] = json!(1);
                    data["measure_index"] = json!(0);
                }
                for field in ["label", "member_labels", "head_location", "tail_location"] {
                    data.as_object_mut().unwrap().remove(field);
                }
                if n["kind"] == "RestChordInter" {
                    data.as_object_mut().unwrap().remove("bounds");
                    data.as_object_mut().unwrap().remove("grade");
                }
                let id = graph.vertex(key, data, false).unwrap();
                keys.insert(n["label"].as_str().unwrap().to_owned(), key);
                labels.insert(id, n["label"].as_str().unwrap().to_owned());
            }
            graph
                .relations(
                    1,
                    Phase::Chord,
                    case["initial"]["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|e| {
                            (
                                keys[e["source"].as_str().unwrap()],
                                keys[e["target"].as_str().unwrap()],
                                json!({"kind":e["kind"]}),
                            )
                        })
                        .collect(),
                )
                .unwrap();
            assert_eq!(
                observed(&graph, &labels),
                case["initial"],
                "{}:initial",
                case["name"]
            );
            let mut owned = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["kind"] == "RestChordInter")
                .map(|n| crate::music_measures::Chord {
                    kind: crate::music_measures::ChordKind::Rest,
                    key: serde_json::from_value(n["origin"].clone()).unwrap(),
                    id: n["id"].as_u64().unwrap() as usize,
                    stem: None,
                    members: Vec::new(),
                    measure: 0,
                    bounds: [0, 0, 0, 0],
                })
                .collect::<Vec<_>>();
            let expected_containers = containers["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == case["name"])
                .unwrap();
            let initial_refresh = crate::music_containers::refresh(
                &mut graph,
                &measures,
                &mut owned,
                &mut Vec::new(),
            )
            .unwrap();
            assert_eq!(
                container_labels(&initial_refresh.projection, &labels),
                expected_containers["initial"]
            );
            assert!(initial_refresh.removed_chords.is_empty());
            if owned.len() > 1 {
                // Reject a bad later owner without replacing earlier members.
                owned.last_mut().unwrap().id += 100;
                let describe = |cs: &[crate::music_measures::Chord]| {
                    json!(
                        cs.iter()
                            .map(|c| json!([c.id, c.bounds, c.members]))
                            .collect::<Vec<_>>()
                    )
                };
                let before_refresh = describe(&owned);
                assert!(
                    crate::music_containers::refresh(
                        &mut graph,
                        &measures,
                        &mut owned,
                        &mut Vec::new()
                    )
                    .is_err()
                );
                assert_eq!(describe(&owned), before_refresh);
                owned.last_mut().unwrap().id -= 100;
            }
            let target = match case["name"].as_str().unwrap() {
                "partial-member" | "last-member" => keys["rest0"],
                "extensive-chord" | "shared-member" => keys["chord"],
                _ => panic!("unknown original operation"),
            };
            let before = graph.snapshot();
            // A later invalid rest->stem identity never commits earlier valid members.
            assert!(
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![
                            (keys["chord"], keys["rest0"], json!({"kind":"Containment"})),
                            (
                                keys["chord"],
                                keys["rest0"],
                                json!({"kind":"ChordStemRelation"})
                            )
                        ]
                    )
                    .is_err()
            );
            assert_eq!(graph.snapshot(), before);
            // Refuse frozen ownership loss atomically, retaining existing guard.
            let mut frozen = graph.clone();
            let frozen_id = frozen.id(target).unwrap();
            frozen.vertices[frozen_id - 1].data["frozen"] = json!(true);
            let frozen_before = frozen.snapshot();
            assert!(frozen.remove_music_inter(target).is_err());
            assert_eq!(frozen.snapshot(), frozen_before);
            graph.remove_music_inter(target).unwrap();
            assert_eq!(
                observed(&graph, &labels),
                case["after"],
                "{}:after",
                case["name"]
            );
            let refresh = crate::music_containers::refresh(
                &mut graph,
                &measures,
                &mut owned,
                &mut Vec::new(),
            )
            .unwrap();
            assert_eq!(
                container_labels(&refresh.projection, &labels),
                expected_containers["after"]
            );
            for c in &owned {
                let node = &graph.vertices[c.id - 1].data;
                assert_eq!(json!(c.bounds), node["bounds"]);
                assert_eq!(
                    json!(
                        c.members
                            .iter()
                            .map(|n| n["id"].clone())
                            .collect::<Vec<_>>()
                    ),
                    node["member_ids"]
                );
                assert!(
                    c.members
                        .iter()
                        .all(|n| graph.vertices[n["id"].as_u64().unwrap() as usize - 1].data == *n)
                );
            }
            let after = graph.snapshot();
            graph.remove_music_inter(target).unwrap();
            assert_eq!(graph.snapshot(), after);
            for (label, key) in &keys {
                assert_eq!(labels[&graph.id(*key).unwrap()], *label);
            }
        }
    }
}

#[cfg(test)]
mod beam_ensemble_tests {
    use super::*;
    #[test]
    fn original_owned_group_partial_last_and_extensive_removal() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-epilog/groups-epilog.json"
        ))
        .unwrap();
        for c in oracle["cleanup_controls"].as_array().unwrap() {
            let mut graph = Graph::default();
            let mut beams = Vec::new();
            let initial = &c["before"][0];
            for (i, b) in initial["members"].as_array().unwrap().iter().enumerate() {
                let key = Key {
                    system: 1,
                    phase: Phase::Beam,
                    local: i + 1,
                };
                graph.vertex(key, b.clone(), false).unwrap();
                beams.push(key);
            }
            let group = Key {
                system: 1,
                phase: Phase::Group,
                local: 1,
            };
            graph.vertex(group,json!({"kind":"BeamGroupInter","bounds":initial["bounds"],"members":beams.iter().map(|k|graph.id(*k).unwrap()).collect::<Vec<_>>()}),false).unwrap();
            graph
                .relations(
                    1,
                    Phase::Group,
                    beams
                        .iter()
                        .map(|k| (group, *k, json!({"kind":"Containment"})))
                        .collect(),
                )
                .unwrap();
            match c["mode"].as_str().unwrap() {
                "partial" => graph.remove_music_inter(beams[0]).unwrap(),
                "last-member" => {
                    for key in &beams {
                        graph.remove_music_inter(*key).unwrap();
                    }
                }
                "extensive" => graph.remove_music_inter(group).unwrap(),
                _ => panic!("unexpected oracle control"),
            }
            let snap = graph.snapshot();
            let ns = snap["inters"].as_array().unwrap();
            let actual: Vec<Value>= ns.iter().filter(|n|n["kind"]=="BeamGroupInter").map(|g|json!({"bounds":g["bounds"],"members":g["members"].as_array().unwrap().iter().map(|id| {let n=ns.iter().find(|n|n["id"]==*id).unwrap(); json!({"kind":n["kind"],"bounds":n["bounds"],"glyph_id":n["glyph_id"]})}).collect::<Vec<_>>()})).collect();
            assert_eq!(json!(actual), c["after"], "{}", c["mode"]);
            assert_eq!(
                json!(
                    beams
                        .iter()
                        .map(|k| graph.vertices[graph.id(*k).unwrap() - 1].removed)
                        .collect::<Vec<_>>()
                ),
                c["members_removed"]
            );
            if actual.is_empty() {
                graph.remove_music_inter(group).unwrap();
                assert_eq!(snap, graph.snapshot());
            }
        }
    }
}

pub(crate) use chord_cache::ChordRead;

#[cfg(test)]
mod grid_foundation_tests;

impl Graph {
    pub(crate) fn refine_stem_median(
        &mut self,
        key: Key,
        median: [f64; 4],
    ) -> Result<Value, String> {
        let id = self.id(key).ok_or("stem_refinement_actual_Stem_required")?;
        let (n, removed) = self.retained_inter(key)?;
        if key.phase != Phase::Stem
            || n["kind"] != "StemInter"
            || removed
            || median
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
            || median[1] >= median[3]
        {
            return Err("stem_refinement_live_vertical_median_required".into());
        }
        let width = n["width"]
            .as_f64()
            .filter(|w| w.is_finite() && *w > 0.)
            .ok_or("stem_refinement_actual_width_required")?;
        let left = (median[0].min(median[2]) - width / 2.).floor();
        let right = (median[0].max(median[2]) + width / 2.).ceil();
        let top = median[1].floor();
        let bottom = median[3].ceil();
        self.vertices[id - 1].data["median"] = json!(median);
        self.vertices[id - 1].data["bounds"] = json!([
            left as i64,
            top as i64,
            (right - left) as i64,
            (bottom - top) as i64
        ]);
        let updated = self.vertices[id - 1].data.clone();
        self.refined_stem_geometry.insert(key, updated.clone());
        Ok(updated)
    }
}

#[cfg(test)]
mod flag_competition_image_tests;
