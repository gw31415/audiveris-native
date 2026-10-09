// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::shared_sig::Phase;

pub(crate) fn detect_overlaps(graph: &mut Graph) -> Result<Value, String> {
    let mut next = graph.clone();
    let initial = next.snapshot();
    let mut flags = initial["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "FlagInter")
        .collect::<Vec<_>>();
    flags.sort_by_key(|n| (n["origin"]["system"].as_u64(), n["bounds"][0].as_i64()));
    let mut inserted = Vec::new();
    for (i, a) in flags.iter().enumerate() {
        for b in &flags[i + 1..] {
            if a["origin"]["system"] != b["origin"]["system"]
                || !crate::music_symbol_reduction::overlap(a, b)?
            {
                continue;
            }
            let snapshot = next.snapshot();
            let direct = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| {
                    (e["source"] == a["id"] && e["target"] == b["id"])
                        || (e["target"] == a["id"] && e["source"] == b["id"])
                })
                .collect::<Vec<_>>();
            if direct.iter().any(|e| e["kind"] == "NoExclusion") {
                continue;
            }
            if a["frozen"] == true || b["frozen"] == true {
                return Err("flag_frozen_overlap_pending".into());
            }
            if direct.iter().any(|e| e["kind"] == "Exclusion") {
                continue;
            }
            if !direct.is_empty() {
                return Err("flag_pair_support_overlap_pending".into());
            }
            let ak: Key = serde_json::from_value(a["origin"].clone())
                .map_err(|_| "flag_owned_origin_required")?;
            let bk: Key = serde_json::from_value(b["origin"].clone())
                .map_err(|_| "flag_owned_origin_required")?;
            next.relations(
                ak.system,
                Phase::Symbol,
                vec![(ak, bk, json!({"kind":"Exclusion","cause":"OVERLAP"}))],
            )?;
            inserted.push(json!([a["id"], b["id"]]));
        }
    }
    *graph = next;
    Ok(json!({"inserted_flag_overlap_pairs":inserted}))
}
pub(crate) fn reduce_exclusions(graph: &mut Graph) -> Result<Value, String> {
    let mut next = graph.clone();
    let initial = next.snapshot();
    let systems = initial["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "FlagInter")
        .map(|n| n["origin"]["system"].as_u64().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    next.contextualize_head_stem_flags()?;
    let mut removed = Vec::new();
    let mut lifecycle = Vec::new();
    for system in systems {
        loop {
            let snapshot = next.snapshot();
            let nodes = snapshot["inters"].as_array().unwrap();
            let mut best = None;
            let mut best_grade = 0.;
            for edge in snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "Exclusion")
            {
                let a = nodes.iter().find(|n| n["id"] == edge["source"]).unwrap();
                let b = nodes.iter().find(|n| n["id"] == edge["target"]).unwrap();
                if a["kind"] != "FlagInter"
                    || b["kind"] != "FlagInter"
                    || a["origin"]["system"] != system
                {
                    continue;
                }
                if a["frozen"] == true || b["frozen"] == true {
                    return Err("flag_frozen_overlap_pending".into());
                }
                let grade = |n: &Value| {
                    n["contextual_grade"]
                        .as_f64()
                        .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
                        .ok_or("flag_context_grade_required")
                };
                let (ga, gb) = (grade(a)?, grade(b)?);
                if ga.max(gb) > best_grade {
                    best = Some((a, b, ga, gb));
                    best_grade = ga.max(gb);
                }
            }
            let Some((a, b, ga, gb)) = best else {
                break;
            };
            let weaker = if ga < gb { a } else { b };
            let key: Key = serde_json::from_value(weaker["origin"].clone())
                .map_err(|_| "flag_owned_origin_required")?;
            removed.push(weaker["id"].clone());
            remove_flag(&mut next, key, &mut lifecycle)?;
            next.contextualize_head_stem_flags()?;
        }
    }
    *graph = next;
    Ok(
        json!({"removed_conflicting_flags":removed,"lifecycle":lifecycle,
        "scope":"actual Flag glyph overlaps and strict contextual best-edge exclusion selection; other cross-kind/frozen conflicts remain explicit"}),
    )
}

#[cfg(test)]
pub(crate) fn reduce_overlaps(graph: &mut Graph) -> Result<Value, String> {
    let mut next = graph.clone();
    let mut result = detect_overlaps(&mut next)?;
    let reduced = reduce_exclusions(&mut next)?;
    result
        .as_object_mut()
        .unwrap()
        .extend(reduced.as_object().unwrap().clone());
    *graph = next;
    Ok(result)
}
