// SPDX-License-Identifier: AGPL-3.0-or-later
//! HeadStemsCleaner.check and bounded automatic checkHeadStems ensembles.
//! Parts selection and complete STEMS/epilog remain separate dependencies.
use crate::{
    head_stem_relation::{self, HeadSide, StemPortion},
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug)]
pub struct Link {
    pub relation_id: usize,
    pub median: [f64; 4],
    pub stem_grade: f64,
    pub target_ratio: f64,
    pub head_side: HeadSide,
    pub dy: f64,
}
fn canonical(bounds: [i32; 4], links: &[&Link]) -> Result<bool, String> {
    let (mut left, mut right) = (None, None);
    for l in links {
        if l.dy > 0.2 {
            return Ok(false);
        }
        match l.head_side {
            HeadSide::Left => left = Some(*l),
            HeadSide::Right => right = Some(*l),
        }
    }
    let (Some(left), Some(right)) = (left, right) else {
        return Ok(false);
    };
    let center = (bounds[1] + bounds[3] / 2) as f64;
    if center >= (left.median[1] + left.median[3]) / 2.
        || center <= (right.median[1] + right.median[3]) / 2.
    {
        return Ok(false);
    }
    Ok(
        head_stem_relation::stem_portion(bounds, left.median, bounds[1] as f64)?
            == StemPortion::StemTop
            && head_stem_relation::stem_portion(
                bounds,
                right.median,
                (bounds[1] + bounds[3] - 1) as f64,
            )? == StemPortion::StemBottom,
    )
}
/// Caller supplies the relation list in original check order. Strict minimum
/// preserves the first contribution tie, using intrinsic stem grade only.
pub fn select_removals(bounds: [i32; 4], ordered: &[Link]) -> Result<Vec<usize>, String> {
    if bounds[2] <= 0
        || bounds[3] <= 0
        || bounds.iter().any(|v| v.unsigned_abs() > 4_000_000)
        || ordered.len() > 100_000
    {
        return Err("invalid_stem_cleaner_input".into());
    }
    let mut ids = BTreeSet::new();
    for l in ordered {
        if l.relation_id == 0
            || !ids.insert(l.relation_id)
            || !l.stem_grade.is_finite()
            || !(0.0..=1.0).contains(&l.stem_grade)
            || !l.target_ratio.is_finite()
            || !(1.0..=11.0).contains(&l.target_ratio)
            || !l.dy.is_finite()
            || l.dy.abs() > 4_000_000.
            || l.median
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
            || l.median[1] >= l.median[3]
        {
            return Err("unsupported_stem_cleaner_link".into());
        }
    }
    let mut links = ordered.iter().collect::<Vec<_>>();
    let mut removed = Vec::new();
    loop {
        if links.len() < 2 || (links.len() == 2 && canonical(bounds, &links)?) {
            break;
        }
        let mut lowest = f64::MAX;
        let mut worst = None;
        for (i, l) in links.iter().enumerate() {
            let contribution = l.stem_grade * (l.target_ratio - 1.);
            if lowest > contribution {
                lowest = contribution;
                worst = Some(i)
            }
        }
        let i = worst.ok_or("unsupported_nonfinite_stem_cleaner_contribution")?;
        removed.push(links.remove(i).relation_id);
    }
    Ok(removed)
}
/// Selected nonmanual live head/stem keys; unrelated edges and vertices are
/// retained transactionally. Missing relations are skipped as in the original.
/// Parallel relations to one supplied stem need a separate ordering oracle.
pub fn clean_head(graph: &mut Graph, head: Key, ordered: &[Key]) -> Result<Value, String> {
    if head.system == 0 || head.phase != Phase::Head || ordered.len() > 100_000 {
        return Err("invalid_stem_cleaner_head".into());
    }
    let snapshot = graph.snapshot();
    let vs = snapshot["inters"]
        .as_array()
        .ok_or("invalid_stem_cleaner_sig")?;
    let by_id = vs
        .iter()
        .map(|v| (v["id"].as_u64().unwrap() as usize, v))
        .collect::<BTreeMap<_, _>>();
    let hid = graph.id(head).ok_or("missing_stem_cleaner_head")?;
    let h = by_id.get(&hid).ok_or("removed_stem_cleaner_head")?;
    if h["kind"] != "HeadInter" || h["manual"] == true || h["is_manual"] == true {
        return Err("unsupported_stem_cleaner_head".into());
    }
    let bounds: [i32; 4] = serde_json::from_value(h["bounds"].clone())
        .map_err(|_| "invalid_stem_cleaner_head_bounds")?;
    let edges = snapshot["relations"]
        .as_array()
        .ok_or("invalid_stem_cleaner_edges")?;
    let mut links = Vec::new();
    let mut seen = BTreeSet::new();
    for k in ordered {
        if k.system != head.system || k.phase != Phase::Stem || !seen.insert(*k) {
            return Err("invalid_stem_cleaner_order".into());
        }
        let id = graph.id(*k).ok_or("missing_stem_cleaner_stem")?;
        let s = by_id.get(&id).ok_or("removed_stem_cleaner_stem")?;
        if s["kind"] != "StemInter" || s["manual"] == true || s["is_manual"] == true {
            return Err("unsupported_stem_cleaner_stem".into());
        }
        let matches = edges
            .iter()
            .filter(|e| e["kind"] == "HeadStemRelation" && e["source"] == hid && e["target"] == id)
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err("unsupported_parallel_stem_cleaner_relations".into());
        }
        let Some(e) = matches.first() else { continue };
        if e["manual"] == true || e["is_manual"] == true {
            return Err("unsupported_manual_stem_cleaner_relation".into());
        }
        links.push(Link {
            relation_id: e["id"].as_u64().ok_or("invalid_stem_cleaner_relation_id")? as usize,
            median: serde_json::from_value(s["median"].clone())
                .map_err(|_| "invalid_stem_cleaner_median")?,
            stem_grade: s["grade"].as_f64().ok_or("invalid_stem_cleaner_grade")?,
            target_ratio: e["target_ratio"]
                .as_f64()
                .ok_or("invalid_stem_cleaner_ratio")?,
            head_side: serde_json::from_value(e["head_side"].clone())
                .map_err(|_| "invalid_stem_cleaner_side")?,
            dy: e["dy"].as_f64().ok_or("invalid_stem_cleaner_gap")?,
        });
    }
    let removed = select_removals(bounds, &links)?;
    if removed.is_empty() {
        return Ok(
            json!({"removed_relation_ids":removed,"selected_cleaner_complete":true,"automatic_parts_selection_complete":false,"stems_complete":false,"heads_complete":false,"epilog_complete":false,"shared_sig":snapshot}),
        );
    }
    let removed_ids = removed.iter().copied().collect::<BTreeSet<_>>();
    let mut records = Vec::new();
    for v in vs {
        let key: Key = serde_json::from_value(v["origin"].clone())
            .map_err(|_| "invalid_stem_cleaner_origin")?;
        if key.system == head.system && key.phase == Phase::Stem {
            let mut r = v.clone();
            r["id"] = json!(key.local);
            records.push(r)
        }
    }
    let mut attachments = Vec::new();
    for e in edges {
        if e["kind"] != "HeadStemRelation"
            || e["system_id"] != head.system
            || removed_ids.contains(&(e["id"].as_u64().unwrap() as usize))
        {
            continue;
        }
        let source = by_id
            .get(&(e["source"].as_u64().unwrap() as usize))
            .ok_or("missing_stem_cleaner_endpoint")?;
        let target = by_id
            .get(&(e["target"].as_u64().unwrap() as usize))
            .ok_or("missing_stem_cleaner_endpoint")?;
        let source: Key = serde_json::from_value(source["origin"].clone())
            .map_err(|_| "invalid_stem_cleaner_origin")?;
        let target: Key = serde_json::from_value(target["origin"].clone())
            .map_err(|_| "invalid_stem_cleaner_origin")?;
        attachments.push(HeadStemAttachment {
            local: e["local_relation_id"]
                .as_u64()
                .ok_or("invalid_stem_cleaner_relation_id")? as usize,
            head: source,
            stem_local: target.local,
            relation: e.clone(),
        });
    }
    let mut next = graph.clone();
    next.sync_stems(head.system, &records, &attachments)?;
    *graph = next;
    Ok(
        json!({"removed_relation_ids":removed,"selected_cleaner_complete":true,"automatic_parts_selection_complete":false,"stems_complete":false,"heads_complete":false,"epilog_complete":false,"shared_sig":graph.snapshot()}),
    )
}

/// Original StemsRetriever.checkHeadStems: null-focus partitions, not the
/// focused-head contextual-grade hack. Uses live relation order/intrinsic grade.
pub(crate) fn check_head_stems(graph: &mut Graph, head: Key) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let hid = graph.id(head).ok_or("automatic cleaner head required")?;
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("automatic cleaner SIG required")?;
    let h = nodes
        .iter()
        .find(|n| n["id"] == hid)
        .ok_or("automatic cleaner live head required")?;
    if head.phase != Phase::Head || h["kind"] != "HeadInter" {
        return Err("automatic cleaner typed head required".into());
    }
    let edges = snapshot["relations"]
        .as_array()
        .ok_or("automatic cleaner edges required")?;
    let mut stems = Vec::new();
    for e in edges
        .iter()
        .filter(|e| e["kind"] == "HeadStemRelation" && e["source"] == hid)
    {
        let stem = nodes
            .iter()
            .find(|n| n["id"] == e["target"])
            .ok_or("automatic cleaner live stem required")?;
        let key: Key = serde_json::from_value(stem["origin"].clone())
            .map_err(|_| "automatic cleaner stem origin required")?;
        if key.phase != Phase::Stem
            || key.system != head.system
            || stems.iter().any(|(k, _, _)| *k == key)
        {
            return Err("automatic cleaner parallel or foreign stem pending".into());
        }
        let grade = stem["grade"]
            .as_f64()
            .filter(|v| (0.0..=1.).contains(v))
            .ok_or("automatic cleaner intrinsic grade required")?;
        stems.push((key, stem["id"].as_u64().unwrap(), grade));
    }
    if stems.len() > 16 {
        return Err("automatic cleaner partition limit16".into());
    }
    stems.sort_by(|a, b| b.2.total_cmp(&a.2));
    let mut conflicts = vec![Vec::new(); stems.len()];
    for (i, (_, id, _)) in stems.iter().enumerate() {
        for e in edges.iter().filter(|e| e["kind"] == "Exclusion") {
            let other = if e["source"] == *id {
                e["target"].as_u64()
            } else if e["target"] == *id {
                e["source"].as_u64()
            } else {
                None
            };
            if let Some(j) = stems.iter().position(|(_, id, _)| Some(*id) == other)
                && j > i
                && !conflicts[i].contains(&j)
            {
                conflicts[i].push(j);
            }
        }
    }
    let partitions = crate::sig_context::compatible_partitions(&conflicts);
    let mut next = graph.clone();
    let mut removed = Vec::new();
    for ensemble in &partitions {
        if ensemble.len() > 1 {
            let keys = ensemble.iter().map(|i| stems[*i].0).collect::<Vec<_>>();
            let check = clean_head(&mut next, head, &keys)?;
            removed.extend(
                check["removed_relation_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .cloned(),
            );
        }
    }
    *graph = next;
    Ok(
        json!({"head_id":hid,"partition_stem_ids":partitions.iter().map(|p|p.iter().map(|i|stems[*i].1).collect::<Vec<_>>()).collect::<Vec<_>>(),"removed_relation_ids":removed}),
    )
}

/// System-head order is retained. Staging prevents a later refusal from
/// publishing a partly cleaned system. Existing cached stem objects survive.
pub(crate) fn check_system_heads(graph: &mut Graph, system: u64) -> Result<Value, String> {
    let snap = graph.snapshot();
    let mut next = graph.clone();
    let mut checks = Vec::new();
    let mut heads = snap["inters"]
        .as_array()
        .ok_or("automatic cleaner SIG required")?
        .iter()
        .filter(|n| {
            n["kind"] == "HeadInter"
                && n["origin"]["system"] == system
                && n["shape"]
                    .as_str()
                    .is_some_and(|s| crate::head_seed_search::ALL_STEM_SHAPES.contains(&s))
        })
        .collect::<Vec<_>>();
    for head in &heads {
        head["bounds"][0]
            .as_i64()
            .ok_or("automatic cleaner head abscissa required")?;
    }
    // Original inspectStems sorts systemHeads stably by Inters.byAbscissa.
    heads.sort_by_key(|n| n["bounds"][0].as_i64().unwrap());
    for n in heads {
        let key = serde_json::from_value(n["origin"].clone())
            .map_err(|_| "automatic cleaner head origin required")?;
        checks.push(check_head_stems(&mut next, key)?);
    }
    *graph = next;
    Ok(
        json!({"head_checks":checks,"scope":"original null-focus exclusion partitions and HeadStemsCleaner at end of system linking; complete STEMS/epilog still pending"}),
    )
}

#[cfg(test)]
mod automatic_tests {
    use super::*;
    #[test]
    fn original_null_focus_partitions_preserve_excluded_alternatives_and_owned_stems() {
        let c: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-epilog/head-partitions.json"
        ))
        .unwrap();
        let mut graph = Graph::default();
        graph.sync_heads(1,&[json!({"id":1,"kind":"HeadInter","shape":"NOTEHEAD_BLACK","bounds":c["head_bounds"],"grade":0.8,"abnormal":true,"removed":false})],&[],&[]).unwrap();
        let head = Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        };
        let mut records = Vec::new();
        let mut attachments = Vec::new();
        let mut keys = Vec::new();
        for (i, s) in c["stems"].as_array().unwrap().iter().enumerate() {
            let glyph = crate::beam_dispatch::glyph_from_json(&s["parts"][0]).unwrap();
            records.push(
                crate::stem_glyph::record(i + 1, &glyph, s["grade"].as_f64().unwrap()).unwrap(),
            );
            let mut relation = s["relation"].clone();
            relation["kind"] = json!("HeadStemRelation");
            relation["profile"] = json!(0);
            // Original unset consistency defaults to 1, reflected by its source ratio5.
            relation["consistency"] = json!(1.);
            attachments.push(HeadStemAttachment {
                local: i + 1,
                head,
                stem_local: i + 1,
                relation,
            });
            keys.push(Key {
                system: 1,
                phase: Phase::Stem,
                local: i + 1,
            });
        }
        graph.sync_stems(1, &records, &attachments).unwrap();
        let endpoints: Vec<usize> = c["exclusion"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.as_u64().unwrap() as usize)
            .collect();
        graph
            .relations(
                1,
                Phase::Head,
                vec![(
                    keys[endpoints[0]],
                    keys[endpoints[1]],
                    json!({"kind":"Exclusion","cause":"OVERLAP"}),
                )],
            )
            .unwrap();
        let ids = keys
            .iter()
            .map(|k| graph.id(*k).unwrap())
            .collect::<Vec<_>>();
        let before = graph.snapshot();
        let result = check_head_stems(&mut graph, head).unwrap();
        let partitions = result["partition_stem_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                p.as_array()
                    .unwrap()
                    .iter()
                    .map(|id| ids.iter().position(|i| json!(i) == *id).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(json!(partitions), c["partitions"]);
        let after = graph.snapshot();
        let ns = after["inters"].as_array().unwrap();
        let es = after["relations"].as_array().unwrap();
        let kept = keys
            .iter()
            .enumerate()
            .filter_map(|(i, k)| {
                es.iter()
                    .any(|e| {
                        e["kind"] == "HeadStemRelation" && e["target"] == graph.id(*k).unwrap()
                    })
                    .then_some(i)
            })
            .collect::<Vec<_>>();
        assert_eq!(json!(kept), c["kept"]);
        for (i, k) in keys.iter().enumerate() {
            let id = graph.id(*k).unwrap();
            let n = ns.iter().find(|n| n["id"] == id).unwrap();
            let prior = before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .unwrap();
            for field in ["glyph", "grade", "median", "bounds", "width", "origin"] {
                assert_eq!(n[field], prior[field]);
            }
            assert_eq!(n["abnormal"], c["stem_abnormal"][i]);
            assert_eq!(c["stem_removed"][i], false);
        }
        assert_eq!(
            ns.iter().find(|n| n["kind"] == "HeadInter").unwrap()["abnormal"],
            c["head_abnormal"]
        );
        assert!(after["tombstones"].as_array().unwrap().is_empty());
        // A fresh pass must not resurrect the relation removed in an earlier ensemble.
        check_head_stems(&mut graph, head).unwrap();
        assert_eq!(graph.snapshot(), after);
    }
}
