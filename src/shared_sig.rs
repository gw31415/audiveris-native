// SPDX-License-Identifier: AGPL-3.0-or-later
// Deterministic native identities and relation store across existing phases.
use crate::{
    beam_candidates::SystemCandidates, beam_groups::GroupPass,
    multiple_rest_registration::RestGraph,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
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
    key: EdgeKey,
    data: Value,
    removed: bool,
}
#[derive(Clone, Default, Debug)]
pub struct Graph {
    vertices: Vec<Vertex>,
    keys: BTreeMap<Key, usize>,
    edges: Vec<Edge>,
    edge_keys: BTreeMap<EdgeKey, usize>,
}
impl Graph {
    fn require_pre_heads(&self, system: u64) -> Result<(), String> {
        if self.vertices.iter().any(|v| {
            v.key.system == system && matches!(v.key.phase, Phase::Head | Phase::Stem) && !v.removed
        }) {
            return Err("complete_shared_sig_heads_state_required".into());
        }
        Ok(())
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
            });
            id
        };
        data["id"] = json!(id);
        data["origin"] = json!(key);
        // Context lives on the registered Inter, not the glyph cache. Re-registering
        // the same intrinsic stem/head must retain it, as original addVertex does.
        let old = &self.vertices[id - 1];
        if matches!(key.phase, Phase::Head | Phase::Stem)
            && !removed
            && old.data["grade"] == data["grade"]
            && data["contextual_grade"].is_null()
            && !old.data["contextual_grade"].is_null()
        {
            data["contextual_grade"] = old.data["contextual_grade"].clone();
        }
        self.vertices[id - 1] = Vertex { key, data, removed };
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
        if self.vertices[id - 1].removed {
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
        } else if kind == Some("RestChordInter") {
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
        let mut affected = BTreeSet::new();
        let mut unique = BTreeSet::new();
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
            unique.insert(*id);
            affected.insert(e.key.source);
            affected.insert(e.key.target);
        }
        // All fallible ownership checks precede mutation. Original callbacks
        // update abnormal only on the endpoints of removed live relations.
        for id in unique {
            self.edges[id - 1].removed = true;
        }
        let linked = self
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.kind == "HeadStemRelation")
            .flat_map(|e| [e.key.source, e.key.target])
            .collect::<BTreeSet<_>>();
        for id in affected {
            let v = &mut self.vertices[id - 1];
            if v.data["kind"] == "StemInter"
                || (v.data["kind"] == "HeadInter"
                    && v.data["shape"]
                        .as_str()
                        .is_some_and(|s| crate::head_seed_search::ALL_STEM_SHAPES.contains(&s)))
            {
                v.data["abnormal"] = json!(!linked.contains(&id));
            }
        }
        Ok(())
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
        if chords {
            // Original identity callbacks operate on the actual registered
            // chord. Validate/update a stage before changing the retained SIG.
            let mut next = self.clone();
            next.relations_impl(system, phase, relations)?;
            *self = next;
            Ok(())
        } else {
            self.relations_impl(system, phase, relations)
        }
    }
    fn relations_impl(
        &mut self,
        system: u64,
        phase: Phase,
        relations: Vec<(Key, Key, Value)>,
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
                    *count
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
            if chord_identity && (self.edges[id - 1].removed || self.edges[id - 1].data.is_null()) {
                affected_chords.insert(source);
            }
            self.edges[id - 1] = Edge {
                key,
                data,
                removed: false,
            };
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
            .filter(|id| !self.vertices[*id - 1].removed)
            .collect::<Vec<_>>()
        {
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
            next.edges[id - 1].removed = true;
        }
        next.invalidate_note_chords(&affected)?;
        *self = next;
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
        for e in &mut self.edges {
            if self.vertices[e.key.source - 1].removed || self.vertices[e.key.target - 1].removed {
                e.removed = true;
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
            for field in ["kind", "shape", "bounds", "median", "width"] {
                if stem[field] != expected[field] {
                    return Err("inconsistent_shared_stem_geometry".into());
                }
            }
            let key = Key {
                system,
                phase: Phase::Stem,
                local,
            };
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
            let consistency = if let Some(main) = r.get("consistency_interline") {
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
                    && self.vertices[external - 1].data["kind"] == "BarlineInter"
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
            let source = ids.contains(&e.key.source);
            let target = ids.contains(&e.key.target);
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
                Some("HeadInter" | "StemInter" | "BeamInter" | "BeamHookInter" | "BarlineInter")
            ) {
                return Err("unsupported_weak_foundation_kind".into());
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
            next.vertices[*id - 1].removed = true;
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
                        return Err("unsupported_foundation_vertex_removal".into());
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

    pub fn snapshot(&self) -> Value {
        json!({"schema":"audiveris-native.shared-sig.v1","id_namespace":"native shared registration-order vertex and relation IDs; not JVM IDs","inters":self.vertices.iter().filter(|v|!v.removed).map(|v|v.data.clone()).collect::<Vec<_>>(),"relations":self.edges.iter().filter(|e|!e.removed).map(|e|e.data.clone()).collect::<Vec<_>>(),"tombstones":self.vertices.iter().enumerate().filter(|(_,v)|v.removed).map(|(i,v)|json!({"id":i+1,"origin":v.key})).collect::<Vec<_>>(),"musicxml_complete":false})
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
            system: 1,
            part: 1,
            staff: 1,
            index: 0,
            left: 200,
            right: 400,
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
            let initial_refresh =
                crate::music_containers::refresh(&graph, &measures, &mut owned).unwrap();
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
                assert!(crate::music_containers::refresh(&graph, &measures, &mut owned).is_err());
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
            let refresh = crate::music_containers::refresh(&graph, &measures, &mut owned).unwrap();
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
