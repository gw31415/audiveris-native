// SPDX-License-Identifier: AGPL-3.0-or-later
//! One-time native-finalized-header to actual shared-SIG identity binding.
use super::{Graph, Key, Phase};
use crate::{
    grid_ir::StaffGeometry,
    native_headers::{NativeHeaderInter, NativeHeaderRelation, NativeStaffHeader},
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(super) struct HeaderBinding {
    source: Arc<NativeHeaderInter>,
    staff: Arc<StaffGeometry>,
}
impl Graph {
    pub(crate) fn bind_native_header_inter(
        &mut self,
        key: Key,
        header: &NativeStaffHeader,
        source: Arc<NativeHeaderInter>,
    ) -> Result<(), String> {
        let roots = [header.clef(), header.key(), header.time()];
        if !roots.into_iter().flatten().any(|node| {
            Arc::ptr_eq(node, &source) || node.members().iter().any(|m| Arc::ptr_eq(m, &source))
        }) {
            return Err("music_header_selected_actual_source_required".into());
        }
        let id = self.endpoint(key)?;
        let data = &self.vertices[id - 1].data;
        let expected_phase = match source.body()["kind"].as_str() {
            Some("ClefInter") => Phase::MusicHeader,
            Some("KeyInter" | "KeyAlterInter") => Phase::MusicKey,
            Some("TimeWholeInter" | "TimePairInter" | "TimeNumberInter") => Phase::MusicTime,
            _ => return Err("music_header_bound_kind_required".into()),
        };
        if key.phase != expected_phase
            || key.system != header.staff().system
            || data["staff_id"] != header.staff().id
            || data["source_header_id"] != source.id()
            || data["kind"] != source.body()["kind"]
            || data["bounds"] != source.body()["bounds"]
            || data["frozen"] != true
            || source.body()["frozen"] != true
            || self.native_header_bindings.contains_key(&key)
        {
            return Err("music_header_fresh_native_binding_required".into());
        }
        self.native_header_bindings.insert(
            key,
            HeaderBinding {
                source,
                staff: header.staff().clone(),
            },
        );
        Ok(())
    }
    pub(crate) fn native_header_source(&self, key: Key) -> Result<&Arc<NativeHeaderInter>, String> {
        let binding = self
            .native_header_bindings
            .get(&key)
            .ok_or("music_header_actual_source_binding_required")?;
        let id = self.endpoint(key)?;
        if key.system != binding.staff.system
            || self.vertices[id - 1].data["staff_id"] != binding.staff.id
        {
            return Err("music_header_actual_binding_Staff_changed".into());
        }
        Ok(&binding.source)
    }
    pub(crate) fn register_native_header_relation(
        &mut self,
        source: Key,
        target: Key,
        relation: Arc<NativeHeaderRelation>,
        phase: Phase,
    ) -> Result<(), String> {
        if !Arc::ptr_eq(self.native_header_source(source)?, relation.source())
            || !Arc::ptr_eq(self.native_header_source(target)?, relation.target())
            || source.system != target.system
        {
            return Err("music_header_actual_relation_endpoints_required".into());
        }
        let kind = relation.body()["kind"]
            .as_str()
            .ok_or("music_header_relation_kind_required")?;
        if self.has_relation(source, target, kind) {
            return Err("music_header_relation_fresh_registration_required".into());
        }
        let sid = self.id(source).unwrap();
        let tid = self.id(target).unwrap();
        let mut next = self.clone();
        next.relations(
            source.system,
            phase,
            vec![(source, target, relation.body().clone())],
        )?;
        let matches = next
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| {
                !edge.removed
                    && edge.key.source == sid
                    && edge.key.target == tid
                    && edge.key.kind == kind
            })
            .map(|(i, _)| i + 1)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err("music_header_single_retained_relation_required".into());
        }
        next.native_header_relation_bindings
            .insert(matches[0], relation);
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
impl Graph {
    pub(crate) fn assert_native_header_binding(
        &self,
        key: Key,
        source: &Arc<NativeHeaderInter>,
        staff: &Arc<StaffGeometry>,
    ) -> serde_json::Value {
        let binding = self.native_header_bindings.get(&key).unwrap();
        assert!(Arc::ptr_eq(&binding.source, source));
        assert!(Arc::ptr_eq(&binding.staff, staff));
        assert!(Arc::ptr_eq(self.native_header_source(key).unwrap(), source));
        let (body, removed) = self.retained_inter(key).unwrap();
        assert!(!removed && source.in_final_sig());
        for field in ["kind", "bounds", "shape", "staff_id", "frozen"] {
            assert_eq!(body[field], source.body()[field]);
        }
        assert_eq!(body["manual"] == true, source.body()["manual"] == true);
        assert_eq!(body["source_header_id"], source.id());
        serde_json::json!({"key":key,"id":self.id(key),"source_header_id":source.id(),"staff":staff.id,"kind":body["kind"],"manual":body["manual"]==true,"frozen":body["frozen"],"same_source_and_Staff_Arcs":true})
    }
    pub(crate) fn assert_native_header_relation(
        &self,
        source: Key,
        target: Key,
        relation: &Arc<NativeHeaderRelation>,
    ) -> serde_json::Value {
        let found = self
            .native_header_relation_bindings
            .iter()
            .filter(|(_, r)| Arc::ptr_eq(r, relation))
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1);
        let id = *found[0].0;
        let edge = &self.edges[id - 1];
        assert!(!edge.removed);
        assert_eq!(self.id(source), Some(edge.key.source));
        assert_eq!(self.id(target), Some(edge.key.target));
        assert_eq!(edge.key.kind, relation.body()["kind"].as_str().unwrap());
        assert!(Arc::ptr_eq(
            self.native_header_source(source).unwrap(),
            relation.source()
        ));
        assert!(Arc::ptr_eq(
            self.native_header_source(target).unwrap(),
            relation.target()
        ));
        assert_eq!(
            edge.data["manual"] == true,
            relation.body()["manual"] == true
        );
        serde_json::json!({"id":id,"kind":edge.key.kind,"source":edge.key.source,"target":edge.key.target,"manual":edge.data["manual"]==true,"same_relation_and_endpoint_Arcs":true})
    }
}
