// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Bounded GRID subset of SIGraph.computeContextualGrade/getPartitions.
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// GRID node types only. Unsupported relation kinds are errors, not ignored support.
/// A maximum of 16 partner entries bounds exclusion partition storage to 65536 rows.
pub fn contextualize(inters: &mut [Value], relations: &[Value]) -> Result<(), String> {
    let mut grades = BTreeMap::new();
    for v in inters.iter() {
        if !matches!(
            v["kind"].as_str(),
            Some(
                "BarlineInter"
                    | "BracketInter"
                    | "BarConnectorInter"
                    | "BracketConnectorInter"
                    | "BraceInter"
            )
        ) {
            return Err("unsupported_grid_context_inter".into());
        }
        let id = v["id"].as_u64().ok_or("invalid_grid_context_id")?;
        let grade = v["grade"]
            .as_f64()
            .filter(|g| (0.0..=1.0).contains(g))
            .ok_or("invalid_grid_context_grade")?;
        if grades.insert(id, grade).is_some() {
            return Err("duplicate_grid_context_id".into());
        }
    }
    for r in relations {
        if !matches!(
            r["kind"].as_str(),
            Some("NoExclusion" | "BarGroupRelation" | "BarConnectionRelation" | "Exclusion")
        ) {
            return Err("unsupported_grid_context_relation".into());
        }
        for side in ["source", "target"] {
            if !r[side].as_u64().is_some_and(|id| grades.contains_key(&id)) {
                return Err("invalid_grid_context_endpoint".into());
            }
        }
        if r["kind"] == "BarConnectionRelation"
            && !r["grade"]
                .as_f64()
                .is_some_and(|g| (0.0..=1.0).contains(&g))
        {
            return Err("invalid_grid_context_support_grade".into());
        }
    }
    let mut adapted = relations.to_vec();
    for r in &mut adapted {
        if r["kind"] == "BarConnectionRelation" {
            let ratio = json!(1.0 + 5.0 * r["grade"].as_f64().unwrap());
            r["source_ratio"] = ratio.clone();
            r["target_ratio"] = ratio;
        }
    }
    apply_context(
        inters,
        &grades,
        &adapted,
        &["BarConnectionRelation"],
        "unsupported_grid_context_partner_limit_16",
    )
}

// Original SIGraph.computeContextualGrade/getPartitions for the head-only
// image owner. This computes support, without purging or reducing candidates.
pub(crate) fn contextualize_head_stems(
    inters: &mut [Value],
    relations: &[Value],
) -> Result<(), String> {
    contextualize_head_stem_flags_impl(inters, relations, false)
}
pub(crate) fn contextualize_head_stem_flags(
    inters: &mut [Value],
    relations: &[Value],
) -> Result<(), String> {
    contextualize_head_stem_flags_impl(inters, relations, true)
}
fn contextualize_head_stem_flags_impl(
    inters: &mut [Value],
    relations: &[Value],
    flags: bool,
) -> Result<(), String> {
    let mut grades = BTreeMap::new();
    for v in inters.iter() {
        if !matches!(v["kind"].as_str(), Some("HeadInter" | "StemInter"))
            && !(flags && v["kind"] == "FlagInter")
        {
            return Err("unsupported_head_stem_context_inter".into());
        }
        let id = v["id"]
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or("invalid_head_stem_context_id")?;
        let grade = v["grade"]
            .as_f64()
            .filter(|n| (0.0..=1.0).contains(n))
            .ok_or("invalid_head_stem_context_grade")?;
        if grades.insert(id, grade).is_some() {
            return Err("duplicate_head_stem_context_id".into());
        }
    }
    for r in relations {
        if !matches!(
            r["kind"].as_str(),
            Some("HeadStemRelation" | "Exclusion" | "NoExclusion")
        ) && !(flags && r["kind"] == "FlagStemRelation")
        {
            return Err("unsupported_head_stem_context_relation".into());
        }
        for side in ["source", "target"] {
            if !r[side].as_u64().is_some_and(|id| grades.contains_key(&id)) {
                return Err("invalid_head_stem_context_endpoint".into());
            }
        }
        if r["kind"] == "HeadStemRelation" || (flags && r["kind"] == "FlagStemRelation") {
            for side in ["source_ratio", "target_ratio"] {
                if !r[side].as_f64().is_some_and(|v| v.is_finite() && v >= 1.) {
                    return Err("invalid_head_stem_context_support_ratio".into());
                }
            }
        }
    }
    apply_context(
        inters,
        &grades,
        relations,
        if flags {
            &["HeadStemRelation", "FlagStemRelation"]
        } else {
            &["HeadStemRelation"]
        },
        "unsupported_head_stem_context_partner_limit_16",
    )
}

fn apply_context(
    inters: &mut [Value],
    grades: &BTreeMap<u64, f64>,
    relations: &[Value],
    support_kinds: &[&str],
    partner_limit_error: &str,
) -> Result<(), String> {
    let stem_ids = inters
        .iter()
        .filter(|p| p["kind"] == "StemInter")
        .map(|p| p["id"].as_u64().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    let mut computed = Vec::new();
    for v in inters.iter() {
        let id = v["id"].as_u64().unwrap();
        let mut partners = Vec::new();
        let mut contributions = BTreeMap::new();
        // JGraphT directed edgesOf enumerates incoming before outgoing edges.
        // This matters when opposite directed supports share the same partner.
        let supports = relations.iter().filter(|r| {
            r["kind"]
                .as_str()
                .is_some_and(|k| support_kinds.contains(&k))
        });
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
            let ratio = r[if b == id {
                "target_ratio"
            } else {
                "source_ratio"
            }]
            .as_f64()
            .ok_or("invalid_context_support_ratio")?;
            if ratio > 1.0 {
                partners.push(partner);
                contributions.insert(partner, grades[&partner] * (ratio - 1.0));
            }
        }
        if partners.len() > 16 {
            return Err(partner_limit_error.into());
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
        // SIGraph.getPartitions treats every stem partner of a focused head as
        // concurrent, even without an explicit Exclusion edge. The focused-head
        // rule differs from getPartitions(null, ...) used by the stem cleaner.
        if v["kind"] == "HeadInter" {
            for (i, partner) in partners.iter().enumerate() {
                if stem_ids.contains(partner) {
                    for other in partners.iter().filter(|p| stem_ids.contains(p)) {
                        // Original List.indexOf preserves the first duplicate slot.
                        let j = partners.iter().position(|p| p == other).unwrap();
                        if j > i && !conflicts[i].contains(&j) {
                            conflicts[i].push(j);
                        }
                    }
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
        computed.push(best);
    }
    // No partial mutation if a later node exceeds the supported bound.
    for (v, grade) in inters.iter_mut().zip(computed) {
        v["contextual_grade"] = json!(grade);
    }
    Ok(())
}

#[cfg(test)]
mod head_stem_tests {
    use super::*;
    #[test]
    fn original_actual_linked_head_stem_contextual_grades_match() {
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-head-context/controls.json"
        ))
        .unwrap();
        let expected = controls["inters"].as_array().unwrap();
        let mut actual = expected.clone();
        for v in &mut actual {
            v["contextual_grade"] = Value::Null;
        }
        contextualize_head_stems(&mut actual, controls["relations"].as_array().unwrap()).unwrap();
        assert_eq!(actual.len(), 53);
        let controlled: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-head-context/focused-head-controls.json"
        ))
        .unwrap();
        assert_eq!(controlled["cases"].as_array().unwrap().len(), 40);
        for case in controlled["cases"].as_array().unwrap() {
            let mut nodes = case["inters"].as_array().unwrap().clone();
            for node in &mut nodes {
                node["contextual_grade"] = Value::Null;
            }
            contextualize_head_stems(&mut nodes, case["relations"].as_array().unwrap()).unwrap();
            for (a, b) in nodes.iter().zip(case["inters"].as_array().unwrap()) {
                assert!(
                    (a["contextual_grade"].as_f64().unwrap()
                        - b["contextual_grade"].as_f64().unwrap())
                    .abs()
                        < 1e-12,
                    "case {} node {}",
                    case,
                    a["id"]
                );
            }
        }

        for (a, b) in actual.iter().zip(expected) {
            assert!(
                (a["contextual_grade"].as_f64().unwrap() - b["contextual_grade"].as_f64().unwrap())
                    .abs()
                    < 1e-12,
                "{}",
                a["id"]
            );
        }
    }
    #[test]
    fn head_stem_context_partitions_limits_and_empty_input_are_transactional() {
        contextualize_head_stems(&mut [], &[]).unwrap();
        let cases: Value =
            serde_json::from_str(include_str!("../fixtures/official-grid/context-cases.json"))
                .unwrap();
        // These are existing original SIGraph partition controls, translated
        // only in node/relation representation; no new oracle claims for glyphs.
        for case in cases.as_array().unwrap() {
            let mut nodes = case["inters"].as_array().unwrap().clone();
            for n in &mut nodes {
                n["id"] = json!(n["id"].as_u64().unwrap() + 1);
                n["kind"] = json!("HeadInter");
                n["contextual_grade"] = Value::Null;
            }
            let mut edges = case["relations"].as_array().unwrap().clone();
            for e in &mut edges {
                for side in ["source", "target"] {
                    e[side] = json!(e[side].as_u64().unwrap() + 1);
                }
                if e["kind"] == "BarConnectionRelation" {
                    e["kind"] = json!("HeadStemRelation");
                    let ratio = json!(1. + 5. * e["grade"].as_f64().unwrap());
                    e["source_ratio"] = ratio.clone();
                    e["target_ratio"] = ratio;
                } else if e["kind"] == "BarGroupRelation" {
                    e["kind"] = json!("NoExclusion");
                }
            }
            contextualize_head_stems(&mut nodes, &edges).unwrap();
            for (n, o) in nodes.iter().zip(case["inters"].as_array().unwrap()) {
                assert_eq!(
                    n["contextual_grade"].as_f64().unwrap().to_bits(),
                    o["contextual_grade"].as_f64().unwrap().to_bits()
                );
            }
        }
        let mut nodes = (1..=18)
            .map(|id| json!({"id":id,"kind":"HeadInter","grade":0.4,"contextual_grade":null}))
            .collect::<Vec<_>>();
        let mut edges=(2..=18).map(|id|json!({"kind":"HeadStemRelation","source":1,"target":id,"source_ratio":4.,"target_ratio":1.5})).collect::<Vec<_>>();
        let before = nodes.clone();
        assert_eq!(
            contextualize_head_stems(&mut nodes, &edges).unwrap_err(),
            "unsupported_head_stem_context_partner_limit_16"
        );
        assert_eq!(nodes, before);
        edges.pop();
        contextualize_head_stems(&mut nodes, &edges).unwrap();
        edges[0]["source_ratio"] = json!(0.5);
        let before = nodes.clone();
        assert!(contextualize_head_stems(&mut nodes, &edges).is_err());
        assert_eq!(nodes, before);
        edges[0]["source_ratio"] = json!(4.);
        edges[0]["kind"] = json!("BeamStemRelation");
        assert!(contextualize_head_stems(&mut nodes, &edges).is_err());
        assert_eq!(nodes, before);
    }
}
