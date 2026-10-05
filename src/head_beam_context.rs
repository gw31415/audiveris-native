// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Bounded beam support subset of SIGraph.computeContextualGrade/getPartitions.
use serde_json::Value;
use std::collections::BTreeMap;

/// Beam node types only. Unsupported relation kinds are errors, not ignored support.
/// A maximum of 16 partner entries bounds exclusion partition storage to 65536 rows.
pub fn contextual_grade(target: u64, inters: &[Value], relations: &[Value]) -> Result<f64, String> {
    let mut grades = BTreeMap::new();
    for v in inters.iter() {
        if !matches!(
            v["kind"].as_str(),
            Some("BeamInter" | "BeamHookInter" | "SmallBeamInter" | "SmallBeamHookInter")
        ) {
            return Err("unsupported_head_beam_context_inter".into());
        }
        let id = v["id"].as_u64().ok_or("invalid_head_beam_context_id")?;
        let grade = v["grade"]
            .as_f64()
            .filter(|g| (0.0..=1.0).contains(g))
            .ok_or("invalid_head_beam_context_grade")?;
        if grades.insert(id, grade).is_some() {
            return Err("duplicate_head_beam_context_id".into());
        }
    }
    for r in relations {
        if !matches!(
            r["kind"].as_str(),
            Some("NoExclusion" | "BeamBeamRelation" | "Exclusion")
        ) {
            return Err("unsupported_head_beam_context_relation".into());
        }
        for side in ["source", "target"] {
            if !r[side].as_u64().is_some_and(|id| grades.contains_key(&id)) {
                return Err("invalid_head_beam_context_endpoint".into());
            }
        }
        if r["kind"] == "BeamBeamRelation"
            && !match r.get("grade") {
                None => Some(1.0),
                Some(v) => v.as_f64(),
            }
            .is_some_and(|g| (0.0..=1.0).contains(&g))
        {
            return Err("invalid_head_beam_context_support_grade".into());
        }
    }
    let mut computed = None;
    for v in inters.iter() {
        let id = v["id"].as_u64().unwrap();
        if id != target {
            continue;
        }
        let mut partners = Vec::new();
        let mut contributions = BTreeMap::new();
        // JGraphT directed edgesOf enumerates incoming before outgoing edges.
        // This matters when opposite directed supports share the same partner.
        let supports = relations.iter().filter(|r| r["kind"] == "BeamBeamRelation");
        for r in supports
            .clone()
            .filter(|r| r["target"] == id)
            .chain(supports.filter(|r| r["source"] == id && r["target"] != id))
        {
            let a = r["source"].as_u64().unwrap();
            let b = r["target"].as_u64().unwrap();
            let partner = if b == id {
                a
            } else if a == id {
                b
            } else {
                continue;
            };
            let ratio = 1.0 + 3.0 * r["grade"].as_f64().unwrap_or(1.0);
            if ratio > 1.0 {
                partners.push(partner);
                contributions.insert(partner, grades[&partner] * (ratio - 1.0));
            }
        }
        if partners.len() > 16 {
            return Err("unsupported_head_beam_context_partner_limit_16".into());
        }
        // Stable grade ordering, preserving duplicate entries and map overwrite semantics.
        partners.sort_by(|a, b| grades[b].total_cmp(&grades[a]));
        let mut conflicts = vec![Vec::new(); partners.len()];
        for (i, partner) in partners.iter().enumerate() {
            for r in relations.iter().filter(|r| r["kind"] == "Exclusion") {
                let a = r["source"].as_u64().unwrap();
                let b = r["target"].as_u64().unwrap();
                let other = if a == *partner {
                    b
                } else if b == *partner {
                    a
                } else {
                    continue;
                };
                if let Some(j) = partners.iter().position(|p| *p == other)
                    && j > i
                    && !conflicts[i].contains(&j)
                {
                    conflicts[i].push(j);
                }
            }
        }
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
            let mut sum = 0.0;
            for (i, p) in partners.iter().enumerate() {
                if sequence[i] == 1 {
                    sum += contributions[p];
                }
            }
            let grade = grades[&id];
            best = best.max(((1.0 + sum) * grade) / (1.0 + sum * grade));
        }
        computed = Some(best);
    }
    computed.ok_or("missing_head_beam_context_target".into())
}
