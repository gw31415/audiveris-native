// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original SigReducer.checkLedgers / ledgerHasHeadOrLedger, actual ordinary Staff ledger maps.
use crate::{shared_sig::Key, stem_image_owned::OwnedItems};
use serde_json::{Value, json};
fn bounds(n: &Value) -> Result<[f64; 4], String> {
    let b: [f64; 4] =
        serde_json::from_value(n["bounds"].clone()).map_err(|_| "music_ledger_bounds_required")?;
    if b.iter().any(|v| !v.is_finite() || v.abs() > 1_000_000.) || b[2] <= 0. || b[3] <= 0. {
        return Err("music_ledger_bounds_invalid".into());
    }
    Ok(b)
}
fn supported(
    ledger: &Value,
    index: i32,
    next_ledgers: &[Key],
    graph: &crate::shared_sig::Graph,
    heads: &[Value],
    interline: f64,
) -> Result<bool, String> {
    let b = bounds(ledger)?;
    if index == 0 || index.unsigned_abs() > 1024 {
        return Err("music_ledger_index_required".into());
    }
    let pitch = index.signum() * 4 + index * 2;
    // Original next-line lists may still reference a removed Ledger body.
    for key in next_ledgers {
        let (next, _) = graph.retained_inter(*key)?;
        let q = bounds(next)?;
        if b[0].max(q[0]) < (b[0] + b[2]).min(q[0] + q[2]) {
            return Ok(true);
        }
    }
    for n in heads
        .iter()
        .filter(|n| n["origin"]["system"] == ledger["origin"]["system"])
    {
        let q = bounds(n)?;
        if b[0] < q[0] + q[2]
            && q[0] < b[0] + b[2]
            && b[1] - interline < q[1] + q[3]
            && q[1] < b[1] + b[3] + interline
        {
            let p = n["pitch"]
                .as_f64()
                .filter(|p| p.is_finite() && p.abs() <= 32.)
                .ok_or("music_ledger_head_pitch_required")?
                .round_ties_even() as i32;
            if p == pitch || p == pitch + index.signum() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn reduce(
    graph: &mut crate::shared_sig::Graph,
    owners: &mut crate::ledger_membership_owners::LedgerMemberships,
    staffs: &[std::sync::Arc<crate::grid_ir::StaffGeometry>],
    interline: f64,
) -> Result<Vec<Value>, String> {
    owners.validate(graph)?;
    let mut ordered = Vec::new();
    for staff in staffs {
        if staff.lines.len() != 5 {
            return Err("music_special_staff_ledger_reduction_pending".into());
        }
        let map = owners
            .maps
            .iter()
            .position(|m| std::sync::Arc::ptr_eq(&m.staff, staff))
            .ok_or("music_actual_ledger_staff_map_required")?;
        ordered.push(map);
    }
    for pair in ordered.windows(2) {
        let upper = &owners.maps[pair[0]];
        let lower = &owners.maps[pair[1]];
        if upper.staff.system != lower.staff.system {
            continue;
        }
        if let (Some(first), Some((_, last))) = (upper.lines.get(&1), lower.lines.first_key_value())
            && lower.lines.contains_key(&-1)
            && first.iter().any(|k| last.contains(k))
        {
            return Err("music_actual_shared_ledger_reassignment_pending".into());
        }
    }
    let heads = graph.snapshot()["inters"]
        .as_array()
        .ok_or("music_ledger_SIG_required")?
        .iter()
        .filter(|n| n["kind"] == "HeadInter")
        .cloned()
        .collect::<Vec<_>>();
    // A late unsupported removal must leave both actual graph and maps unchanged.
    let mut next_graph = graph.clone();
    let mut next_owners = owners.clone();
    let mut removed = Vec::new();
    loop {
        let start = removed.len();
        for map in &ordered {
            let entries = next_owners.maps[*map].lines.clone();
            for (index, members) in entries {
                for key in members {
                    let (body, retired) = next_graph.retained_inter(key)?;
                    if retired {
                        continue;
                    }
                    let next_index = index + index.signum();
                    let next_line = next_owners.maps[*map]
                        .lines
                        .get(&next_index)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);
                    if !supported(body, index, next_line, &next_graph, &heads, interline)? {
                        let observation = json!({"id":next_graph.id(key),"origin":key,"bounds":body["bounds"],"index":body["index"],"map_index":index,"map_staff":next_owners.maps[*map].staff.id,"glyph_id":body["glyph_id"]});
                        next_owners.remove_from_owner(key, body["staff_id"].as_u64());
                        next_graph.remove_music_inter(key)?;
                        removed.push(observation);
                    }
                }
            }
        }
        if removed.len() == start {
            break;
        }
    }
    *graph = next_graph;
    *owners = next_owners;
    Ok(removed)
}
pub(crate) fn check(mut o: OwnedItems) -> Result<OwnedItems, String> {
    let removed = reduce(
        &mut o.graph,
        &mut o.ledger_memberships,
        &o.staffs,
        o.interline as f64,
    )?;
    o.prepared.report["orphan_ledger_consistency"] = json!({"original":"SigReducer.checkLedgers/ledgerHasHeadOrLedger","removed":removed,"physical_registry_retained":true,"scope":"actual ordinary five-line Staff maps; genuine shared-ledger reassignment pending","full_REDUCTION_complete":false});
    o.prepared.report["shared_sig"] = o.graph.snapshot();
    Ok(o)
}

#[cfg(test)]
mod tests;
