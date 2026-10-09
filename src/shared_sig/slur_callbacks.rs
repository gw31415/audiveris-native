// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! Fresh automatic Slur callbacks with retained selected relation ownership.
use super::*;
use crate::music_slurs::{Relation, lookup::Side};
use std::sync::Arc;
impl Graph {
    pub(crate) fn next_curve_key(&self, system: u64) -> Result<Key, String> {
        let local = self
            .keys
            .keys()
            .filter(|k| k.system == system && k.phase == Phase::Curve)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("slur_identity_budget")?;
        Ok(Key {
            system,
            phase: Phase::Curve,
            local,
        })
    }
    pub(super) fn validate_fresh_slur_head(
        &self,
        source: usize,
        target: usize,
        data: &Value,
    ) -> Result<(), String> {
        let s = &self.vertices[source - 1];
        let h = &self.vertices[target - 1];
        let (pending_source, pending_target, owner) = self
            .pending_slur_relation
            .as_ref()
            .ok_or("slur_typed_pending_relation_required")?;
        if *pending_source != s.key || *pending_target != h.key || owner.projection() != *data {
            return Err("slur_pending_owner_endpoints_and_body_required".into());
        }
        if s.key.phase != Phase::Curve
            || s.data["kind"] != "SlurInter"
            || s.key.system != h.key.system
            || h.key.phase != Phase::Head
            || h.data["kind"] != "HeadInter"
            || s.removed
            || h.removed
            || s.data["manual"] == true
            || data["manual"] == true
            || s.data["is_tie"] == true
            || !matches!(data["side"].as_str(), Some("LEFT" | "RIGHT"))
            || !data["euclidean"]
                .as_f64()
                .is_some_and(|d| d.is_finite() && d >= 0.)
            || data["grade"] != 1.0
            || data["source_ratio"] != 6.0
            || data["target_ratio"] != 1.0
        {
            return Err("slur_fresh_automatic_typed_relation_required".into());
        }
        if self.edges.iter().any(|e| {
            !e.removed
                && e.key.source == source
                && e.key.kind == "SlurHeadRelation"
                && e.data["side"] == data["side"]
        }) {
            return Err("slur_fresh_unique_side_required".into());
        }
        Ok(())
    }
    pub(super) fn refresh_fresh_slur_abnormal(&mut self, source: usize) {
        let complete = ["LEFT", "RIGHT"].iter().all(|side| {
            self.edges.iter().any(|e| {
                !e.removed
                    && e.key.source == source
                    && e.key.kind == "SlurHeadRelation"
                    && e.data["side"] == *side
            })
        });
        self.vertices[source - 1].data["abnormal"] = json!(!complete);
    }
    pub(crate) fn fresh_slur_relation_id(
        &self,
        source: Key,
        target: Key,
        side: Side,
    ) -> Result<usize, String> {
        let sid = self.endpoint(source)?;
        let tid = self.endpoint(target)?;
        let side = if side == Side::Left { "LEFT" } else { "RIGHT" };
        let ids = self
            .edges
            .iter()
            .filter(|e| {
                !e.removed
                    && e.key.phase == Phase::Curve
                    && e.key.source == sid
                    && e.key.target == tid
                    && e.key.kind == "SlurHeadRelation"
                    && e.data["side"] == side
            })
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        if ids.len() != 1 {
            return Err("slur_actual_relation_identity_required".into());
        }
        Ok(ids[0])
    }
    pub(crate) fn set_fresh_slur_part(&mut self, key: Key, part: u64) -> Result<(), String> {
        let id = self.endpoint(key)?;
        let s = &self.vertices[id - 1];
        if key.phase != Phase::Curve
            || s.data["kind"] != "SlurInter"
            || s.data["abnormal"] != false
            || part == 0
            || s.cached_part.is_some()
        {
            return Err("slur_fresh_dispatch_part_required".into());
        }
        // The caller proves this ID through the actual left Head's Measure/Part Arc.
        self.vertices[id - 1].cached_part = Some(part);
        self.vertices[id - 1].data["part_id"] = json!(part);
        Ok(())
    }
}

impl Graph {
    /// Pending ownership is bound before the original-style added callback runs.
    pub(crate) fn register_fresh_slur_relation(
        &mut self,
        source: Key,
        target: Key,
        relation: Arc<Relation>,
    ) -> Result<usize, String> {
        if self.pending_slur_relation.is_some() {
            return Err("slur_nested_pending_relation_forbidden".into());
        }
        let mut next = self.clone();
        next.pending_slur_relation = Some((source, target, Arc::clone(&relation)));
        next.relations(
            source.system,
            Phase::Curve,
            vec![(source, target, relation.projection())],
        )?;
        let id = next.fresh_slur_relation_id(source, target, relation.side)?;
        if next.pending_slur_relation.is_some()
            || !Arc::ptr_eq(next.fresh_slur_relation_owner(id)?, &relation)
        {
            return Err("slur_same_pending_relation_owner_required".into());
        }
        *self = next;
        Ok(id)
    }
    pub(crate) fn fresh_slur_relation_owner(&self, id: usize) -> Result<&Arc<Relation>, String> {
        let edge = self
            .edges
            .get(id.checked_sub(1).ok_or("slur_edge_identity_required")?)
            .ok_or("slur_edge_identity_required")?;
        let owner = self
            .slur_relation_bindings
            .get(&id)
            .ok_or("slur_edge_owner_required")?;
        if edge.removed || edge.key.phase != Phase::Curve || edge.key.kind != "SlurHeadRelation" {
            return Err("slur_live_automatic_relation_required".into());
        }
        for (name, value) in owner.projection().as_object().unwrap() {
            if edge.data[name] != *value {
                return Err("slur_relation_body_owner_mismatch".into());
            }
        }
        Ok(owner)
    }
    pub(super) fn require_fresh_slur_zero_head_support(&self, id: usize) -> Result<(), String> {
        let owner = self.fresh_slur_relation_owner(id)?;
        let edge = &self.edges[id - 1];
        let source = &self.vertices[edge.key.source - 1];
        let target = &self.vertices[edge.key.target - 1];
        let projection = owner.projection();
        if edge.data["id"] != json!(id)
            || source.removed
            || target.removed
            || source.key.phase != Phase::Curve
            || source.data["kind"] != "SlurInter"
            || source.key.system != target.key.system
            || edge.key.system != source.key.system
            || source.data["manual"] == true
            || source.data["is_tie"] == true
            || target.key.phase != Phase::Head
            || target.data["kind"] != "HeadInter"
            || !owner.euclidean.is_finite()
            || owner.euclidean < 0.
            || projection["manual"] != false
            || projection["grade"] != 1.0
            || projection["source_ratio"] != 6.0
            || projection["target_ratio"] != 1.0
        {
            return Err("slur_zero_head_support_requires_live_typed_automatic_owner".into());
        }
        Ok(())
    }
    pub(super) fn bind_pending_slur_relation(&mut self, id: usize) -> Result<(), String> {
        let (source, target, relation) = self
            .pending_slur_relation
            .take()
            .ok_or("slur_typed_pending_relation_required")?;
        let edge = &self.edges[id - 1];
        if self.id(source) != Some(edge.key.source)
            || self.id(target) != Some(edge.key.target)
            || self.slur_relation_bindings.contains_key(&id)
        {
            return Err("slur_fresh_pending_edge_endpoints_required".into());
        }
        self.slur_relation_bindings.insert(id, relation);
        self.fresh_slur_relation_owner(id)?;
        Ok(())
    }
}
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct SlurCallbackRecord {
    pub(crate) hook: &'static str,
    pub(crate) slur: Key,
    pub(crate) relation: Option<std::sync::Weak<Relation>>,
    pub(crate) state: Value,
}
#[cfg(test)]
impl Graph {
    pub(crate) fn enable_fresh_slur_trace(&mut self) {
        self.fresh_slur_trace = Some(Vec::new());
    }
    pub(crate) fn fresh_slur_trace(&self) -> &[SlurCallbackRecord] {
        self.fresh_slur_trace.as_deref().unwrap_or(&[])
    }
    pub(crate) fn record_fresh_slur(
        &mut self,
        hook: &'static str,
        slur: Key,
        relation_id: Option<usize>,
    ) {
        if self.fresh_slur_trace.is_none() {
            return;
        }
        let node = self.id(slur).map(|id| &self.vertices[id - 1]);
        let edges = node
            .map(|_| {
                self.edges
                    .iter()
                    .filter(|e| {
                        !e.removed
                            && Some(e.key.source) == self.id(slur)
                            && e.key.kind == "SlurHeadRelation"
                    })
                    .map(|e| json!({"id":e.data["id"],"side":e.data["side"]}))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let state = json!({"present":node.is_some(),"abnormal":node.is_some_and(|v|v.data["abnormal"]==true),
            "part_cached":node.and_then(|v|v.cached_part),"manual":node.is_some_and(|v|v.data["manual"]==true),
            "tie":node.is_some_and(|v|v.data["is_tie"]==true),"head_relations":edges,"part_memberships":0,"chord_clocks":self.slur_chord_clock_observation()});
        let relation =
            relation_id.map(|id| Arc::downgrade(self.fresh_slur_relation_owner(id).unwrap()));
        self.fresh_slur_trace
            .as_mut()
            .unwrap()
            .push(SlurCallbackRecord {
                hook,
                slur,
                relation,
                state,
            });
    }
    pub(crate) fn slur_preservation_observation(&self) -> Value {
        json!({"sig":self.snapshot(),"vertices":self.vertices.iter().map(|v|json!({
            "key":v.key,"removed":v.removed,"removing":v.removing,"cached_part":v.cached_part,
            "retained_voice":v.retained_voice,"voice_owner":v.voice_owner.as_ref().map(|x|Arc::as_ptr(x)as usize),
            "slot_owner":v.slot_owner.as_ref().map(|x|Arc::as_ptr(x)as usize),
            "cache":format!("{:?}",v.chord_cache),"rest_beams":v.rest_beams.as_ref().map(|x|Arc::as_ptr(x)as usize)
        })).collect::<Vec<_>>(),"relation_bindings":self.slur_relation_bindings.iter().map(|(id,r)|json!([id,Arc::as_ptr(r)as usize])).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
impl Graph {
    pub(crate) fn record_fresh_slur_membership(
        &mut self,
        hook: &'static str,
        key: Key,
        count: usize,
    ) {
        self.record_fresh_slur(hook, key, None);
        if let Some(row) = self.fresh_slur_trace.as_mut().and_then(|v| v.last_mut()) {
            row.state["part_memberships"] = json!(count);
        }
    }
    pub(crate) fn slur_chord_clock_observation(&self) -> Value {
        json!(self.vertices.iter().filter(|v|matches!(v.data["kind"].as_str(),Some("HeadChordInter"|"RestChordInter"))).map(|v|json!({
            "key":v.key,"removed":v.removed,"cached_part":v.cached_part,"manual":v.data["manual"],
            "retained_voice":v.retained_voice,"voice_owner":v.voice_owner.as_ref().map(|a|Arc::as_ptr(a)as usize),
            "slot_owner":v.slot_owner.as_ref().map(|a|Arc::as_ptr(a)as usize),
            "voice":v.data["voice"],"slot_id":v.data["slot_id"],"time_offset":v.data["time_offset"],"duration":v.data["duration"]
        })).collect::<Vec<_>>())
    }
}

impl Graph {
    pub(crate) fn validate_fresh_slur_part(&self, key: Key, part: u64) -> Result<(), String> {
        let id = self.endpoint(key)?;
        let v = &self.vertices[id - 1];
        if v.data["kind"] != "SlurInter"
            || key.phase != Phase::Curve
            || v.cached_part != Some(part)
            || v.data["part_id"] != part
        {
            return Err("slur_registered_cached_Part_binding_required".into());
        }
        Ok(())
    }
}
#[cfg(test)]
mod zero_head_support_tests {
    use super::*;

    fn declared_relation() -> (Graph, usize, Arc<Relation>) {
        let mut graph = Graph::default();
        let slur = Key {
            system: 1,
            phase: Phase::Curve,
            local: 1,
        };
        let head = Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        };
        graph
            .vertex(
                slur,
                json!({"kind":"SlurInter","shape":"SLUR_ABOVE",
            "grade":0.7,"manual":false,"is_tie":false}),
                false,
            )
            .unwrap();
        graph
            .vertex(
                head,
                json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
            "grade":0.8,"contextual_grade":0.9,"manual":true}),
                false,
            )
            .unwrap();
        let owner = Arc::new(Relation {
            side: Side::Left,
            euclidean: 1.,
        });
        let id = graph
            .register_fresh_slur_relation(slur, head, owner.clone())
            .unwrap();
        (graph, id, owner)
    }

    #[test]
    fn invalid_slur_support_cannot_publish_head_context() {
        for flags in [false, true] {
            for invalid in [
                "unbound",
                "target_ratio",
                "removed_source",
                "foreign_target",
            ] {
                let (mut graph, id, owner) = declared_relation();
                match invalid {
                    "unbound" => {
                        graph.slur_relation_bindings.remove(&id);
                    }
                    "target_ratio" => {
                        graph.edges[id - 1].data["target_ratio"] = json!(2.);
                    }
                    "removed_source" => {
                        let source = graph.edges[id - 1].key.source;
                        graph.vertices[source - 1].removed = true;
                    }
                    "foreign_target" => {
                        let target = graph.edges[id - 1].key.target;
                        graph.vertices[target - 1].key.system = 2;
                    }
                    _ => unreachable!(),
                }
                let before = graph.snapshot();
                let error = if flags {
                    graph.contextualize_head_stem_flags()
                } else {
                    graph.contextualize_head_stems()
                }
                .unwrap_err();
                let expected = match invalid {
                    "unbound" => "slur_edge_owner_required",
                    "target_ratio" => "slur_relation_body_owner_mismatch",
                    _ => "slur_zero_head_support_requires_live_typed_automatic_owner",
                };
                assert_eq!(error, expected, "{invalid}, flags={flags}");
                assert_eq!(graph.snapshot(), before, "{invalid}, flags={flags}");
                if invalid != "unbound" {
                    assert!(Arc::ptr_eq(&graph.slur_relation_bindings[&id], &owner));
                }
            }
        }
    }
}
