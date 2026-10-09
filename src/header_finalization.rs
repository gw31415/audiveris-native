// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// ClefBuilder.selectClef / HeaderBuilder.purgeBarlines / StaffHeader.freeze.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::clef_competition::{ClefNode, ExclusionEdge, competing_clef_order};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn integer(v: &Value) -> Result<i64, String> {
    v.as_i64().ok_or("invalid_header_integer".into())
}
fn id(v: &Value) -> Result<u64, String> {
    v.as_u64().ok_or("invalid_header_id".into())
}
fn grade(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|x| (0.0..=1.0).contains(x))
        .ok_or("invalid_header_grade".into())
}
fn bounds(v: &Value) -> Result<[i64; 4], String> {
    let a = v
        .as_array()
        .filter(|a| a.len() == 4)
        .ok_or("invalid_header_bounds")?;
    let mut b = [0; 4];
    for (i, x) in a.iter().enumerate() {
        b[i] = integer(x)?;
    }
    if b.iter().any(|v| !(-4_000_000..=4_000_000).contains(v)) || b[2] <= 0 || b[3] <= 0 {
        return Err("invalid_header_bounds".into());
    }
    Ok(b)
}
fn remove(inters: &mut Vec<Value>, relations: &mut Vec<Value>, removed: &BTreeSet<u64>) {
    inters.retain(|n| !removed.contains(&n["id"].as_u64().unwrap()));
    relations.retain(|r| {
        !removed.contains(&r["source"].as_u64().unwrap())
            && !removed.contains(&r["target"].as_u64().unwrap())
    });
}
fn clef_context(clef: u64, inters: &[Value], relations: &[Value]) -> Result<f64, String> {
    let by_id = inters
        .iter()
        .map(|n| Ok((id(&n["id"])?, n)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let intrinsic = grade(&by_id.get(&clef).ok_or("missing_header_clef")?["grade"])?;
    let mut partners = Vec::new();
    let mut seen = BTreeSet::new();
    for r in relations
        .iter()
        .filter(|r| r["kind"] == "ClefKeyRelation" && r["source"] == clef)
    {
        let partner = id(&r["target"])?;
        if !seen.insert(partner) {
            return Err("unsupported_duplicate_header_support".into());
        }
        let key = by_id.get(&partner).ok_or("missing_header_key_partner")?;
        if key["kind"] != "KeyInter" {
            return Err("invalid_header_key_partner".into());
        }
        partners.push((partner, grade(&key["grade"])?));
    }
    if partners.len() > 16 {
        return Err("unsupported_header_support_budget".into());
    }
    partners.sort_by(|a, b| b.1.total_cmp(&a.1));
    let edges = relations
        .iter()
        .filter(|r| r["kind"] == "Exclusion")
        .map(|r| Ok((id(&r["source"])?, id(&r["target"])?)))
        .collect::<Result<Vec<_>, String>>()?;
    let mut conflicts = vec![Vec::new(); partners.len()];
    for (i, (partner, _)) in partners.iter().enumerate() {
        for (a, b) in &edges {
            let other = if a == partner {
                b
            } else if b == partner {
                a
            } else {
                continue;
            };
            if let Some(j) = partners.iter().position(|p| p.0 == *other)
                && j > i
                && !conflicts[i].contains(&j)
            {
                conflicts[i].push(j);
            }
        }
    }
    // Exact SIGraph.getPartitions sequence enumeration, not arbitrary subsets.
    let mut sequences = vec![vec![0i8; partners.len()]];
    for (i, concurrent) in conflicts.iter().enumerate() {
        for k in 0..sequences.len() {
            if sequences[k][i] != -1 {
                sequences[k][i] = 1;
                if !concurrent.is_empty() {
                    let mut duplicate = sequences[k].clone();
                    duplicate[i] = 0;
                    sequences.push(duplicate);
                    for &j in concurrent {
                        sequences[k][j] = -1;
                    }
                }
            }
        }
    }
    let mut best = 0.0f64;
    for sequence in sequences {
        let mut sum = 0.;
        for (i, partner) in partners.iter().enumerate() {
            if sequence[i] == 1 {
                sum += partner.1 * 5.;
            }
        }
        best = best.max(((1. + sum) * intrinsic) / (1. + sum * intrinsic));
    }
    Ok(best)
}
fn freeze(
    root: u64,
    inters: &mut [Value],
    relations: &[Value],
    seen: &mut BTreeSet<u64>,
) -> Result<(), String> {
    let mut pending = vec![root];
    while let Some(current) = pending.pop() {
        if !seen.insert(current) {
            continue;
        }
        if seen.len() > 32768 {
            return Err("unsupported_header_freeze_budget".into());
        }
        let index = inters
            .iter()
            .position(|n| n["id"] == current)
            .ok_or("missing_header_freeze_node")?;
        inters[index]["frozen"] = json!(true);
        for r in relations
            .iter()
            .rev()
            .filter(|r| r["kind"] == "Containment" && r["source"] == current)
        {
            pending.push(id(&r["target"])?);
        }
    }
    Ok(())
}

fn freeze_detached(
    value: &Value,
    inters: &mut [Value],
    relations: &[Value],
    seen: &mut BTreeSet<u64>,
    depth: usize,
) -> Result<Value, String> {
    if depth > 16 {
        return Err("unsupported_header_ensemble_depth".into());
    }
    let nid = id(&value["id"])?;
    if inters.iter().any(|n| n["id"] == nid) {
        freeze(nid, inters, relations, seen)?;
    }
    let mut result = value.clone();
    result["frozen"] = json!(true);
    if let Some(members) = value["members"].as_array() {
        if members.len() > 32768 {
            return Err("unsupported_header_member_budget".into());
        }
        result["members"] = json!(
            members
                .iter()
                .map(|m| freeze_detached(m, inters, relations, seen, depth + 1))
                .collect::<Result<Vec<_>, _>>()?
        );
    }
    Ok(result)
}

struct HeaderSelection {
    staff_id: u64,
    start: i32,
    stop: i32,
    clef_id: Option<u64>,
    key_id: Option<u64>,
    time_id: Option<u64>,
    clef_range: Value,
    key_range: Value,
    time_range: Value,
}
// These are live kernel results, consumed before any header report is emitted.
// The native carrier allocation starts at finalization; earlier candidate object
// identity is not reconstructed or claimed by this transport prerequisite.
struct RawFinalizedSystem {
    staffs: Vec<HeaderSelection>,
    inters: Vec<Value>,
    relations: Vec<Value>,
    detached: BTreeMap<u64, Value>,
    removed_clef_ids: BTreeSet<u64>,
    purged_barline_ids: BTreeSet<u64>,
    frozen_ids: BTreeSet<u64>,
}
impl RawFinalizedSystem {
    fn report(&self) -> Value {
        let get = |id: Option<u64>| {
            id.and_then(|id| {
                self.inters
                    .iter()
                    .find(|n| n["id"] == id)
                    .or_else(|| self.detached.get(&id))
            })
            .cloned()
            .unwrap_or(Value::Null)
        };
        let staffs = self.staffs.iter().map(|s| json!({"staff_id":s.staff_id,
            "start":s.start,"stop":s.stop,"clef":get(s.clef_id),"key":get(s.key_id),"time":get(s.time_id),
            "clef_range":s.clef_range,"key_range":s.key_range,"time_range":s.time_range})).collect::<Vec<_>>();
        json!({"staffs":staffs,"sig_inters":self.inters,"relations":self.relations,
            "removed_clef_ids":self.removed_clef_ids,"purged_barline_ids":self.purged_barline_ids,
            "frozen_ids":self.frozen_ids})
    }
    fn into_owned(
        self,
        system: std::sync::Arc<crate::grid_ir::SystemGeometry>,
    ) -> Result<crate::native_headers::NativeSystemHeaders, String> {
        let mut staff_inputs = Vec::new();
        for selected in self.staffs {
            let staff = system
                .staffs
                .iter()
                .find(|s| s.id == selected.staff_id)
                .ok_or("native_header_selected_actual_Staff_required")?
                .clone();
            staff_inputs.push(crate::native_headers::HeaderStaffInput {
                staff,
                start: selected.start,
                stop: selected.stop,
                clef_id: selected.clef_id,
                key_id: selected.key_id,
                time_id: selected.time_id,
                clef_range: selected.clef_range,
                key_range: selected.key_range,
                time_range: selected.time_range,
            });
        }
        crate::native_headers::NativeSystemHeaders::from_finalized(
            system,
            staff_inputs,
            self.inters,
            self.relations,
            self.detached,
            self.removed_clef_ids,
            self.purged_barline_ids,
            self.frozen_ids,
        )
    }
}

// Legacy isolated-kernel callers receive only the unchanged projection.
pub fn finalize_system(
    grid: &Value,
    post_time: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    Ok(finalize_system_raw(grid, post_time, sheet_interline)?.report())
}

pub(crate) struct OwnedHeaderFinalization {
    pub(crate) report: Value,
    pub(crate) owner: crate::native_headers::NativeHeaders,
}

pub(crate) fn analyze_owned(
    grid: &Value,
    time_column: &Value,
    sheet_interline: usize,
    actual_systems: &[std::sync::Arc<crate::grid_ir::SystemGeometry>],
) -> Result<OwnedHeaderFinalization, String> {
    if time_column["time_column_complete"] != true {
        return Err("incomplete_header_time_dependency".into());
    }
    let systems = time_column["systems"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("invalid_header_systems")?;
    let mut retained = Vec::new();
    let mut ids = BTreeSet::new();
    for system in systems {
        let id = id(&system["system_id"])?;
        if !ids.insert(id) {
            return Err("duplicate_header_system".into());
        }
        let actual = actual_systems
            .iter()
            .find(|s| s.id == id)
            .ok_or("native_header_actual_System_required")?
            .clone();
        let raw = finalize_system_raw(grid, &system["after_system_time_stop"], sheet_interline)?;
        #[cfg(test)]
        let legacy_projection = raw.report();
        let owned = raw.into_owned(actual)?;
        #[cfg(test)]
        {
            let mut projection = owned.report();
            projection.as_object_mut().unwrap().remove("system_id");
            assert_eq!(projection, legacy_projection);
        }
        retained.push(owned);
    }
    let owner = crate::native_headers::NativeHeaders::new(retained)?;
    let report = owner.report();
    Ok(OwnedHeaderFinalization { report, owner })
}

/// Input is the live post-time graph, not a serialized/reloaded .omr graph.
/// This stage mutates only its clone and retains explicit removed-node IDs.
fn finalize_system_raw(
    grid: &Value,
    post_time: &Value,
    sheet_interline: usize,
) -> Result<RawFinalizedSystem, String> {
    if !(1..=128).contains(&sheet_interline) {
        return Err("unsupported_header_interline".into());
    }
    let staffs = post_time["staffs"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 1024)
        .ok_or("invalid_header_staffs")?;
    let mut staff_ids = BTreeSet::new();
    for staff in staffs {
        if !staff_ids.insert(id(&staff["staff_id"])?) {
            return Err("duplicate_header_staff".into());
        }
        let start = integer(&staff["header_start"])?;
        let stop = integer(&staff["header_stop"])?;
        if !(0..=4_000_000).contains(&start) || !(start..=4_000_000).contains(&stop) {
            return Err("invalid_header_range".into());
        }
        if !staff["clef_stop"].is_null()
            && !(0..=4_000_000).contains(&integer(&staff["clef_stop"])?)
        {
            return Err("invalid_header_clef_stop".into());
        }
    }
    let mut inters = post_time["sig_inters"]
        .as_array()
        .filter(|v| v.len() <= 32768)
        .ok_or("invalid_header_inters")?
        .clone();
    let mut relations = post_time["relations"]
        .as_array()
        .filter(|v| v.len() <= 65536)
        .ok_or("invalid_header_relations")?
        .clone();
    let mut ids = BTreeSet::new();
    for node in &inters {
        if !ids.insert(id(&node["id"])?) {
            return Err("duplicate_header_node".into());
        }
    }
    for r in &relations {
        if !ids.contains(&id(&r["source"])?) || !ids.contains(&id(&r["target"])?) {
            return Err("missing_header_relation_endpoint".into());
        }
    }
    let mut selected = BTreeMap::new();
    let mut removed_clefs = BTreeSet::new();
    for staff in staffs {
        let sid = id(&staff["staff_id"])?;
        let stop = staff["clef_stop"].as_i64().unwrap_or(
            integer(&staff["header_start"])?
                + (4.5 * sheet_interline as f64).round_ties_even() as i64,
        );
        let clefs = inters
            .iter()
            .filter(|n| n["kind"] == "ClefInter")
            .map(|n| {
                Ok(ClefNode {
                    id: id(&n["id"])?,
                    staff_id: id(&n["staff_id"])?,
                    left: bounds(&n["bounds"])?[0],
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let exclusions = relations
            .iter()
            .filter(|r| r["kind"] == "Exclusion")
            .map(|r| {
                Ok(ExclusionEdge {
                    source: id(&r["source"])?,
                    target: id(&r["target"])?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut order = competing_clef_order(sid, stop, &clefs, &exclusions)?;
        for &cid in &order {
            let g = clef_context(cid, &inters, &relations)?;
            inters.iter_mut().find(|n| n["id"] == cid).unwrap()["contextual_grade"] = json!(g);
        }
        order.sort_by(|a, b| {
            let g = |id| {
                inters.iter().find(|n| n["id"] == id).unwrap()["contextual_grade"]
                    .as_f64()
                    .unwrap()
            };
            g(*b).total_cmp(&g(*a))
        });
        if let Some(&winner) = order.first() {
            selected.insert(sid, winner);
            let discarded = order.into_iter().skip(1).collect::<BTreeSet<_>>();
            remove(&mut inters, &mut relations, &discarded);
            removed_clefs.extend(discarded);
        }
    }
    let left_ends = grid["bar_semantics"]["left_end_peaks"]
        .as_array()
        .ok_or("missing_header_left_end_flags")?;
    let grid_systems = grid["bar_semantics"]["systems"]
        .as_array()
        .ok_or("missing_header_grid_systems")?;
    let mut removed_bars = BTreeSet::new();
    for staff in staffs {
        let sid = id(&staff["staff_id"])?;
        let start = integer(&staff["header_start"])?;
        let stop = integer(&staff["header_stop"])?;
        for bar in inters
            .iter()
            .filter(|n| n["kind"] == "BarlineInter" && n["staff_id"] == sid)
        {
            let b = bounds(&bar["bounds"])?;
            let center = b[0] + b[2] / 2;
            let origin = &bar["origin"];
            let source_peak = if origin["stage"] == "GRID" {
                grid_systems
                    .iter()
                    .flat_map(|s| s["inters"].as_array().into_iter().flatten())
                    .find(|n| n["id"] == origin["id"])
                    .map(|n| &n["source_peak"])
                    .unwrap_or(&Value::Null)
            } else {
                &bar["source_peak"]
            };
            if center > start && center < stop && !left_ends.contains(source_peak) {
                removed_bars.insert(id(&bar["id"])?);
            }
        }
    }
    remove(&mut inters, &mut relations, &removed_bars);
    let mut frozen = BTreeSet::new();
    let mut detached = BTreeMap::new();
    for staff in staffs {
        let sid = id(&staff["staff_id"])?;
        if let Some(&cid) = selected.get(&sid) {
            freeze(cid, &mut inters, &relations, &mut frozen)?;
        }
        for k in ["header_key", "header_time"] {
            if !staff[k].is_null() {
                let nid = id(&staff[k]["id"])?;
                if inters.iter().any(|n| n["id"] == nid) {
                    freeze(nid, &mut inters, &relations, &mut frozen)?;
                } else {
                    let snapshot =
                        freeze_detached(&staff[k], &mut inters, &relations, &mut frozen, 0)?;
                    detached.insert(nid, snapshot);
                }
            }
        }
    }
    // Rebuild serialized ensemble members from the live graph after recursive freezing.
    let snapshot = inters.clone();
    for node in &mut inters {
        if node["kind"] == "KeyInter" || node["kind"] == "TimePairInter" {
            let root = id(&node["id"])?;
            let mut members = relations
                .iter()
                .filter(|r| r["kind"] == "Containment" && r["source"] == root)
                .map(|r| {
                    let member = id(&r["target"])?;
                    snapshot
                        .iter()
                        .find(|n| n["id"] == member)
                        .cloned()
                        .ok_or("missing_header_member".into())
                })
                .collect::<Result<Vec<_>, String>>()?;
            let axis = if node["kind"] == "KeyInter" { 0 } else { 1 };
            let mut keyed = members
                .into_iter()
                .map(|m| {
                    let b = bounds(&m["bounds"])?;
                    Ok((b[axis] + b[axis + 2] / 2, m))
                })
                .collect::<Result<Vec<_>, String>>()?;
            keyed.sort_by_key(|(center, _)| *center);
            members = keyed.into_iter().map(|(_, m)| m).collect();
            node["members"] = json!(members);
        }
    }
    let mut selected_staffs = Vec::new();
    for staff in staffs {
        let sid = id(&staff["staff_id"])?;
        selected_staffs.push(HeaderSelection {
            staff_id: sid,
            start: i32::try_from(integer(&staff["header_start"])?)
                .map_err(|_| "invalid_header_range")?,
            stop: i32::try_from(integer(&staff["header_stop"])?)
                .map_err(|_| "invalid_header_range")?,
            clef_id: selected.get(&sid).copied(),
            key_id: staff["header_key"]["id"].as_u64(),
            time_id: staff["header_time"]["id"].as_u64(),
            clef_range: json!({"valid":!staff["clef_stop"].is_null(),
                "browse_start":staff["header_start"],
                "browse_stop":integer(&staff["header_start"])?+(4.5*sheet_interline as f64).round_ties_even()as i64,
                "start":null,"stop":staff["clef_stop"]}),
            key_range: staff["key_range"].clone(),
            time_range: staff["time_range"].clone(),
        });
    }
    Ok(RawFinalizedSystem {
        staffs: selected_staffs,
        inters,
        relations,
        detached,
        removed_clef_ids: removed_clefs,
        purged_barline_ids: removed_bars,
        frozen_ids: frozen,
    })
}

/// Final HEADERS state for the current explicitly bounded fresh five-line path.
/// General GRID, note recognition, rhythm and export are separate unfinished stages.
pub fn analyze(grid: &Value, time_column: &Value, sheet_interline: usize) -> Result<Value, String> {
    if time_column["time_column_complete"] != true {
        return Err("incomplete_header_time_dependency".into());
    }
    let systems = time_column["systems"]
        .as_array()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or("invalid_header_systems")?;
    let mut output = Vec::new();
    let mut staffs = Vec::new();
    let mut system_ids = BTreeSet::new();
    let mut staff_ids = BTreeSet::new();
    for system in systems {
        if !system_ids.insert(id(&system["system_id"])?) {
            return Err("duplicate_header_system".into());
        }
        let mut state = finalize_system(grid, &system["after_system_time_stop"], sheet_interline)?;
        state["system_id"] = system["system_id"].clone();
        for staff in state["staffs"].as_array().unwrap() {
            if !staff_ids.insert(id(&staff["staff_id"])?) {
                return Err("duplicate_header_staff_across_systems".into());
            }
            let mut row = staff.clone();
            row["system_id"] = system["system_id"].clone();
            staffs.push(row);
        }
        output.push(state);
    }
    Ok(
        json!({"stage":"HEADERS.native","headers_complete":true,"note_recognition_complete":false,"supported_scope":"fresh automatic five-line native GRID; existing raster/scale/part budgets apply","staffs":staffs,"systems":output,"pending":["STEM_SEEDS","BEAMS","LEDGERS","automatic HEADS and stem attachment","rhythm and MusicXML","bounded-memory 300 DPI PDF integration"]}),
    )
}
