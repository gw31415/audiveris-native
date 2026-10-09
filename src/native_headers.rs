// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! Native HEADERS output retained at finalization, before report projection.
//! Ownership begins at the native freeze result; earlier JSON-kernel candidate
//! identity is not reconstructed or claimed by this carrier.
use crate::grid_ir::{StaffGeometry, SystemGeometry};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct NativeHeaderInter {
    id: u64,
    body: Value,
    in_final_sig: bool,
    members: Vec<Arc<NativeHeaderInter>>,
}
impl NativeHeaderInter {
    pub(crate) fn id(&self) -> u64 {
        self.id
    }
    pub(crate) fn body(&self) -> &Value {
        &self.body
    }
    pub(crate) fn members(&self) -> &[Arc<NativeHeaderInter>] {
        &self.members
    }
    pub(crate) fn in_final_sig(&self) -> bool {
        self.in_final_sig
    }
}

#[derive(Debug)]
pub(crate) struct NativeHeaderRelation {
    body: Value,
    source: Arc<NativeHeaderInter>,
    target: Arc<NativeHeaderInter>,
}
impl NativeHeaderRelation {
    pub(crate) fn body(&self) -> &Value {
        &self.body
    }
    pub(crate) fn source(&self) -> &Arc<NativeHeaderInter> {
        &self.source
    }
    pub(crate) fn target(&self) -> &Arc<NativeHeaderInter> {
        &self.target
    }
}

fn native_id(body: &Value) -> Result<u64, String> {
    body["id"]
        .as_u64()
        .filter(|id| *id > 0)
        .ok_or("native_header_inter_id_required".into())
}
fn collect_members(
    body: &Value,
    payloads: &mut BTreeMap<u64, Value>,
    depth: usize,
) -> Result<(), String> {
    if depth > 16 {
        return Err("native_header_member_depth".into());
    }
    if let Some(members) = body["members"].as_array() {
        for member in members {
            let id = native_id(member)?;
            if let Some(existing) = payloads.get(&id) {
                if existing != member {
                    return Err("native_header_member_payload_conflict".into());
                }
            } else {
                payloads.insert(id, member.clone());
            }
            collect_members(member, payloads, depth + 1)?;
        }
    }
    Ok(())
}
fn intern(
    id: u64,
    payloads: &BTreeMap<u64, Value>,
    live_ids: &BTreeSet<u64>,
    relations: &[Value],
    memo: &mut BTreeMap<u64, Arc<NativeHeaderInter>>,
    visiting: &mut BTreeSet<u64>,
) -> Result<Arc<NativeHeaderInter>, String> {
    if let Some(node) = memo.get(&id) {
        return Ok(Arc::clone(node));
    }
    if !visiting.insert(id) {
        return Err("native_header_member_cycle".into());
    }
    let body = payloads.get(&id).ok_or("native_header_member_missing")?;
    let mut members = Vec::new();
    if let Some(rows) = body["members"].as_array() {
        let mut member_ids = BTreeSet::new();
        for row in rows {
            let mid = native_id(row)?;
            if !member_ids.insert(mid) {
                return Err("native_header_duplicate_member".into());
            }
            if live_ids.contains(&id)
                && !relations.iter().any(|r| {
                    r["kind"] == "Containment"
                        && r["source"].as_u64() == Some(id)
                        && r["target"].as_u64() == Some(mid)
                })
            {
                return Err("native_header_live_member_relation_required".into());
            }
            members.push(intern(mid, payloads, live_ids, relations, memo, visiting)?);
        }
    }
    let node = Arc::new(NativeHeaderInter {
        id,
        body: body.clone(),
        in_final_sig: live_ids.contains(&id),
        members,
    });
    visiting.remove(&id);
    memo.insert(id, Arc::clone(&node));
    Ok(node)
}

/// Produced directly from the finalizer's staff variables; never from its report.
pub(crate) struct HeaderStaffInput {
    pub(crate) staff: Arc<StaffGeometry>,
    pub(crate) start: i32,
    pub(crate) stop: i32,
    pub(crate) clef_id: Option<u64>,
    pub(crate) key_id: Option<u64>,
    pub(crate) time_id: Option<u64>,
    pub(crate) clef_range: Value,
    pub(crate) key_range: Value,
    pub(crate) time_range: Value,
}
#[derive(Debug)]
pub(crate) struct NativeStaffHeader {
    staff: Arc<StaffGeometry>,
    start: i32,
    stop: i32,
    clef: Option<Arc<NativeHeaderInter>>,
    key: Option<Arc<NativeHeaderInter>>,
    time: Option<Arc<NativeHeaderInter>>,
    clef_range: Value,
    key_range: Value,
    time_range: Value,
}
impl NativeStaffHeader {
    pub(crate) fn staff(&self) -> &Arc<StaffGeometry> {
        &self.staff
    }
    #[cfg(test)]
    pub(crate) fn start(&self) -> i32 {
        self.start
    }
    pub(crate) fn stop(&self) -> i32 {
        self.stop
    }
    pub(crate) fn clef(&self) -> Option<&Arc<NativeHeaderInter>> {
        self.clef.as_ref()
    }
    pub(crate) fn key(&self) -> Option<&Arc<NativeHeaderInter>> {
        self.key.as_ref()
    }
    pub(crate) fn time(&self) -> Option<&Arc<NativeHeaderInter>> {
        self.time.as_ref()
    }
    pub(crate) fn report(&self) -> Value {
        json!({"staff_id":self.staff.id,"start":self.start,"stop":self.stop,
            "clef":self.clef().map(|n| n.body()),"key":self.key().map(|n| n.body()),
            "time":self.time().map(|n| n.body()),"clef_range":self.clef_range,
            "key_range":self.key_range,"time_range":self.time_range})
    }
}

#[derive(Debug)]
pub(crate) struct NativeSystemHeaders {
    system: Arc<SystemGeometry>,
    staffs: Vec<NativeStaffHeader>,
    inters: Vec<Arc<NativeHeaderInter>>,
    detached: BTreeMap<u64, Arc<NativeHeaderInter>>,
    relations: Vec<Arc<NativeHeaderRelation>>,
    removed_clef_ids: BTreeSet<u64>,
    purged_barline_ids: BTreeSet<u64>,
    frozen_ids: BTreeSet<u64>,
}
impl NativeSystemHeaders {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_finalized(
        system: Arc<SystemGeometry>,
        staff_inputs: Vec<HeaderStaffInput>,
        inters: Vec<Value>,
        relations: Vec<Value>,
        detached: BTreeMap<u64, Value>,
        removed_clef_ids: BTreeSet<u64>,
        purged_barline_ids: BTreeSet<u64>,
        frozen_ids: BTreeSet<u64>,
    ) -> Result<Self, String> {
        let mut payloads = BTreeMap::new();
        let mut live_ids = BTreeSet::new();
        let mut live_order = Vec::new();
        for body in inters {
            let id = native_id(&body)?;
            if !live_ids.insert(id) {
                return Err("native_header_duplicate_inter".into());
            }
            live_order.push(id);
            payloads.insert(id, body);
        }
        let detached_ids = detached.keys().copied().collect::<Vec<_>>();
        for (id, body) in detached {
            if native_id(&body)? != id || payloads.insert(id, body).is_some() {
                return Err("native_header_detached_identity_conflict".into());
            }
        }
        for body in payloads.values().cloned().collect::<Vec<_>>() {
            collect_members(&body, &mut payloads, 0)?;
        }
        let mut by_id = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        for id in payloads.keys() {
            intern(
                *id,
                &payloads,
                &live_ids,
                &relations,
                &mut by_id,
                &mut visiting,
            )?;
        }
        let canonical = live_order.iter().map(|id| Arc::clone(&by_id[id])).collect();
        let canonical_detached = detached_ids
            .into_iter()
            .map(|id| (id, Arc::clone(&by_id[&id])))
            .collect();
        let mut owned_relations = Vec::with_capacity(relations.len());
        for body in relations {
            let source = body["source"]
                .as_u64()
                .filter(|id| live_ids.contains(id))
                .ok_or("native_header_relation_source_required")?;
            let target = body["target"]
                .as_u64()
                .filter(|id| live_ids.contains(id))
                .ok_or("native_header_relation_target_required")?;
            owned_relations.push(Arc::new(NativeHeaderRelation {
                body,
                source: Arc::clone(&by_id[&source]),
                target: Arc::clone(&by_id[&target]),
            }));
        }
        let mut seen = BTreeSet::new();
        let mut staffs = Vec::with_capacity(staff_inputs.len());
        for input in staff_inputs {
            if input.staff.system != system.id
                || !system.staffs.iter().any(|s| Arc::ptr_eq(s, &input.staff))
                || !seen.insert(input.staff.id)
                || input.start < 0
                || input.stop < input.start
            {
                return Err("native_header_actual_staff_range_required".into());
            }
            let get = |id: Option<u64>,
                       kinds: &[&str]|
             -> Result<Option<Arc<NativeHeaderInter>>, String> {
                id.map(|id| {
                    let node = by_id.get(&id).ok_or("native_header_reference_missing")?;
                    if !kinds.iter().any(|kind| node.body["kind"] == *kind)
                        || node.body["frozen"] != true
                    {
                        return Err("native_header_frozen_kind_required".into());
                    }
                    if let Some(sid) = node.body["staff_id"].as_u64()
                        && sid != input.staff.id
                    {
                        return Err("native_header_inter_staff_conflict".into());
                    }
                    Ok(Arc::clone(node))
                })
                .transpose()
            };
            let clef = get(input.clef_id, &["ClefInter"])?;
            let key = get(input.key_id, &["KeyInter"])?;
            let time = get(input.time_id, &["TimePairInter", "TimeWholeInter"])?;
            staffs.push(NativeStaffHeader {
                staff: input.staff,
                start: input.start,
                stop: input.stop,
                clef,
                key,
                time,
                clef_range: input.clef_range,
                key_range: input.key_range,
                time_range: input.time_range,
            });
        }
        if seen.len() != system.staffs.len() {
            return Err("native_header_staff_coverage_required".into());
        }
        Ok(Self {
            system,
            staffs,
            inters: canonical,
            detached: canonical_detached,
            relations: owned_relations,
            removed_clef_ids,
            purged_barline_ids,
            frozen_ids,
        })
    }
    pub(crate) fn system(&self) -> &Arc<SystemGeometry> {
        &self.system
    }
    pub(crate) fn staffs(&self) -> &[NativeStaffHeader] {
        &self.staffs
    }
    #[cfg(test)]
    pub(crate) fn inters(&self) -> &[Arc<NativeHeaderInter>] {
        &self.inters
    }
    #[cfg(test)]
    pub(crate) fn detached(&self) -> &BTreeMap<u64, Arc<NativeHeaderInter>> {
        &self.detached
    }
    pub(crate) fn relations(&self) -> &[Arc<NativeHeaderRelation>] {
        &self.relations
    }
    pub(crate) fn report(&self) -> Value {
        json!({"system_id":self.system.id,"staffs":self.staffs.iter().map(NativeStaffHeader::report).collect::<Vec<_>>(),
            "sig_inters":self.inters.iter().map(|n| n.body()).collect::<Vec<_>>(),
            "relations":self.relations.iter().map(|r| r.body()).collect::<Vec<_>>(),"removed_clef_ids":self.removed_clef_ids,
            "purged_barline_ids":self.purged_barline_ids,"frozen_ids":self.frozen_ids})
    }
}

#[derive(Debug)]
pub(crate) struct NativeHeaders {
    systems: Vec<NativeSystemHeaders>,
}
impl NativeHeaders {
    pub(crate) fn new(systems: Vec<NativeSystemHeaders>) -> Result<Self, String> {
        let mut system_ids = BTreeSet::new();
        let mut staff_ids = BTreeSet::new();
        for system in &systems {
            if !system_ids.insert(system.system.id)
                || system.staffs.iter().any(|s| !staff_ids.insert(s.staff.id))
            {
                return Err("native_headers_duplicate_owner".into());
            }
        }
        Ok(Self { systems })
    }
    pub(crate) fn systems(&self) -> &[NativeSystemHeaders] {
        &self.systems
    }
    pub(crate) fn staff_header(
        &self,
        staff: &Arc<StaffGeometry>,
    ) -> Result<&NativeStaffHeader, String> {
        let (system, header) = self
            .systems
            .iter()
            .find_map(|system| {
                system
                    .staffs
                    .iter()
                    .find(|header| Arc::ptr_eq(&header.staff, staff))
                    .map(|header| (system, header))
            })
            .ok_or("native_header_actual_staff_owner_required")?;
        for selected in [header.clef(), header.key(), header.time()]
            .into_iter()
            .flatten()
        {
            let live = system.inters.iter().find(|node| node.id == selected.id);
            let detached = system.detached.get(&selected.id);
            let owned = match (live, detached) {
                (Some(node), None) => selected.in_final_sig && Arc::ptr_eq(node, selected),
                (None, Some(node)) => !selected.in_final_sig && Arc::ptr_eq(node, selected),
                _ => false,
            };
            if !owned {
                return Err("native_header_selected_registry_owner_required".into());
            }
        }
        Ok(header)
    }
    #[cfg(test)]
    pub(crate) fn header_stop(&self, staff: &Arc<StaffGeometry>) -> Result<i32, String> {
        Ok(self.staff_header(staff)?.stop())
    }
    pub(crate) fn report(&self) -> Value {
        let staffs = self
            .systems
            .iter()
            .flat_map(|system| {
                system.staffs.iter().map(|staff| {
                    let mut row = staff.report();
                    row["system_id"] = json!(system.system.id);
                    row
                })
            })
            .collect::<Vec<_>>();
        json!({"stage":"HEADERS.native","headers_complete":true,"note_recognition_complete":false,
            "supported_scope":"fresh automatic five-line native GRID; existing raster/scale/part budgets apply",
            "staffs":staffs,"systems":self.systems.iter().map(NativeSystemHeaders::report).collect::<Vec<_>>(),
            "pending":["STEM_SEEDS","BEAMS","LEDGERS","automatic HEADS and stem attachment","rhythm and MusicXML","bounded-memory 300 DPI PDF integration"]})
    }
}
