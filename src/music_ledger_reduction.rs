// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original SigReducer.checkLedgers / ledgerHasHeadOrLedger, ordinary one-staff systems.
use crate::{
    shared_sig::{Key, Phase},
    stem_image_owned::OwnedItems,
};
use serde_json::{Value, json};
fn bounds(n: &Value) -> Result<[f64; 4], String> {
    let b: [f64; 4] =
        serde_json::from_value(n["bounds"].clone()).map_err(|_| "music_ledger_bounds_required")?;
    if b.iter().any(|v| !v.is_finite() || v.abs() > 1_000_000.) || b[2] <= 0. || b[3] <= 0. {
        return Err("music_ledger_bounds_invalid".into());
    }
    Ok(b)
}
fn supported(ledger: &Value, nodes: &[Value], interline: f64) -> Result<bool, String> {
    let b = bounds(ledger)?;
    let index = ledger["index"]
        .as_i64()
        .filter(|i| *i != 0 && i.unsigned_abs() <= 1024)
        .ok_or("music_ledger_index_required")?;
    let next = index + index.signum();
    let pitch = index.signum() * 4 + index * 2;
    for n in nodes
        .iter()
        .filter(|n| n["origin"]["system"] == ledger["origin"]["system"])
    {
        if n["kind"] == "LedgerInter" && n["staff_id"] == ledger["staff_id"] && n["index"] == next {
            let q = bounds(n)?;
            if b[0].max(q[0]) < (b[0] + b[2]).min(q[0] + q[2]) {
                return Ok(true);
            }
        } else if n["kind"] == "HeadInter" {
            let q = bounds(n)?;
            // java.awt.Rectangle.intersects excludes mere boundary contacts.
            if b[0] < q[0] + q[2]
                && q[0] < b[0] + b[2]
                && b[1] - interline < q[1] + q[3]
                && q[1] < b[1] + b[3] + interline
            {
                let p = n["pitch"]
                    .as_f64()
                    .filter(|p| p.is_finite() && p.abs() <= 32.)
                    .ok_or("music_ledger_head_pitch_required")?
                    .round_ties_even() as i64;
                if p == pitch || p == pitch + index.signum() {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
pub(crate) fn check(mut o: OwnedItems) -> Result<OwnedItems, String> {
    o.ledger_memberships.validate(&o.graph)?;
    let snapshot = o.graph.snapshot();
    let mut ledgers = snapshot["inters"]
        .as_array()
        .ok_or("music_ledger_SIG_required")?
        .iter()
        .filter(|n| n["kind"] == "LedgerInter")
        .cloned()
        .collect::<Vec<_>>();
    for n in &ledgers {
        let k: Key = serde_json::from_value(n["origin"].clone())
            .map_err(|_| "music_ledger_origin_required")?;
        if k.phase != Phase::Ledger {
            return Err("music_ledger_phase_required".into());
        }
        let staffs = o.upstream_report["grid_ir"]["staffs"]
            .as_array()
            .ok_or("music_ledger_staffs_required")?
            .iter()
            .filter(|s| s["system_id"] == k.system)
            .collect::<Vec<_>>();
        if staffs.len() != 1
            || staffs[0]["id"] != n["staff_id"]
            || staffs[0]["lines"].as_array().is_none_or(|l| l.len() != 5)
        {
            return Err("music_shared_or_special_ledger_reduction_pending".into());
        }
    }
    ledgers.sort_by_key(|n| {
        (
            n["origin"]["system"].as_u64(),
            n["staff_id"].as_u64(),
            n["index"].as_i64(),
            n["id"].as_u64(),
        )
    });
    // Validate every predicate before mutating the actual owner.
    for n in &ledgers {
        supported(
            n,
            snapshot["inters"].as_array().unwrap(),
            o.interline as f64,
        )?;
    }
    let mut removed = Vec::new();
    loop {
        let start = removed.len();
        for n in &ledgers {
            if removed.iter().any(|r: &Value| r["id"] == n["id"]) {
                continue;
            }
            let live = o.graph.snapshot();
            if !supported(n, live["inters"].as_array().unwrap(), o.interline as f64)? {
                let key: Key = serde_json::from_value(n["origin"].clone())
                    .map_err(|_| "music_ledger_origin_required")?;
                let (body, _) = o.graph.retained_inter(key)?;
                o.ledger_memberships
                    .remove_from_owner(key, body["staff_id"].as_u64());
                o.graph.remove_music_inter(key)?;
                removed.push(json!({"id":n["id"],"origin":key,"bounds":n["bounds"],"index":n["index"],"glyph_id":n["glyph_id"]}));
            }
        }
        if removed.len() == start {
            break;
        }
    }
    o.prepared.report["orphan_ledger_consistency"] = json!({"original":"SigReducer.checkLedgers/ledgerHasHeadOrLedger","removed":removed,"physical_registry_retained":true,"scope":"ordinary five-line one-staff systems; shared-ledger correction pending","full_REDUCTION_complete":false});
    o.prepared.report["shared_sig"] = o.graph.snapshot();
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concrete_head_and_next_ledger_support_exclude_boundary_contacts() {
        let ledger = json!({"kind":"LedgerInter","origin":{"system":1},"staff_id":1,"index":1,"bounds":[100,200,12,2]});
        let head =
            json!({"kind":"HeadInter","origin":{"system":1},"bounds":[111,205,8,8],"pitch":6.0});
        assert!(supported(&ledger, std::slice::from_ref(&head), 11.).unwrap());
        let mut boundary = head;
        boundary["bounds"][0] = json!(112);
        assert!(!supported(&ledger, &[boundary], 11.).unwrap());
        let mut next = ledger.clone();
        next["index"] = json!(2);
        next["bounds"][0] = json!(111);
        assert!(supported(&ledger, std::slice::from_ref(&next), 11.).unwrap());
        next["bounds"][0] = json!(112);
        assert!(!supported(&ledger, &[next], 11.).unwrap());
        let mut invalid = ledger;
        invalid["index"] = json!(i64::MIN);
        assert!(supported(&invalid, &[], 11.).is_err());
    }
}
