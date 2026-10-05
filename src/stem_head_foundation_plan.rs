// SPDX-License-Identifier: AGPL-3.0-or-later
//! Private original SigReducer head-only geometry/competition plan on native
//! metadata. Full overlap/chord/late consistency and epilog remain pending.
use crate::{
    head_stem_relation::{StemPortion, stem_portion},
    sig_context as context,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[cfg(test)]
mod boundaries {
    use super::*;
    #[test]
    fn empty_and_unsupported_scope_do_not_mutate_input() {
        let empty = json!({"inters":[],"relations":[]});
        let result = prepare(&empty).unwrap();
        assert!(result.events.is_empty());
        assert_eq!(empty, result.final_state);
        let ledger = json!({"inters":[{"id":1,"kind":"LedgerInter","shape":"LEDGER","grade":0.67}],"relations":[]});
        assert_eq!(prepare(&ledger).unwrap().final_state, ledger);
        let related = json!({"inters":ledger["inters"],"relations":[{"id":1,"kind":"Exclusion","source":1,"target":1}]});
        assert_eq!(
            prepare(&related).err().unwrap(),
            "ledger_foundation_relations_pending"
        );
        for n in [
            json!({"id":1,"kind":"HeadInter","shape":"WHOLE_NOTE","contextual_grade":0.8}),
            json!({"id":1,"kind":"HeadInter","shape":"NOTEHEAD_BLACK","contextual_grade":0.8,"frozen":true}),
            json!({"id":1,"kind":"BeamInter","contextual_grade":0.8}),
            json!({"id":1,"kind":"HeadInter","shape":"NOTEHEAD_BLACK"}),
        ] {
            let input = json!({"inters":[n],"relations":[]});
            let before = input.clone();
            assert!(prepare(&input).is_err());
            assert_eq!(input, before);
        }
    }
    #[test]
    fn remaining_orphan_stem_consistency_is_an_explicit_refusal() {
        let input = json!({"inters":[{"id":1,"kind":"StemInter","grade":0.8,"contextual_grade":0.8,"median":[1.,0.,1.,10.]}],"relations":[]});
        let before = input.clone();
        assert_eq!(
            prepare(&input).err().unwrap(),
            "orphan_stem_requires_remaining_original_consistency_checks"
        );
        assert_eq!(input, before);
    }
}
#[derive(Debug)]
pub(crate) enum Event {
    RemoveVertex(u64),
    RemoveRelation(u64),
    Exclusion(u64, u64),
}
pub(crate) struct Plan {
    pub events: Vec<Event>,
    pub final_state: Value,
    pub stages: Value,
}
fn extended(stem: &Value, links: &[Value]) -> [f64; 4] {
    let mut line: [f64; 4] = serde_json::from_value(stem["median"].clone()).unwrap();
    for e in links {
        let p: [f64; 2] = serde_json::from_value(e["extension_point"].clone()).unwrap();
        if p[1] < line[1] {
            line[0] = p[0];
            line[1] = p[1];
        }
        if p[1] > line[3] {
            line[2] = p[0];
            line[3] = p[1];
        }
    }
    line
}
fn plan(input: &Value) -> Value {
    let nodes = input["inters"].as_array().unwrap();
    let mut edges = input["relations"].as_array().unwrap().clone();
    let mut removed = Vec::new();
    let mut exclusions = Vec::new();
    for stem in nodes.iter().filter(|v| v["kind"] == "StemInter") {
        loop {
            let links = edges
                .iter()
                .filter(|e| e["kind"] == "HeadStemRelation" && e["target"] == stem["id"])
                .cloned()
                .collect::<Vec<_>>();
            let line = extended(stem, &links);
            let bad = links.iter().find(|e| {
                let h = nodes.iter().find(|h| h["id"] == e["source"]).unwrap();
                assert!(matches!(
                    h["shape"].as_str(),
                    Some("NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                )); // beam-free ordinary head prototype only
                let portion = stem_portion(
                    serde_json::from_value(h["bounds"].clone()).unwrap(),
                    line,
                    e["extension_point"][1].as_f64().unwrap(),
                )
                .unwrap();
                (portion == StemPortion::StemBottom && e["head_side"] != "RIGHT")
                    || (portion == StemPortion::StemTop && e["head_side"] != "LEFT")
            });
            let Some(bad) = bad else { break };
            if bad["invading"] == true {
                exclusions
                    .push(json!({"source":bad["source"],"target":bad["target"],"cause":"OVERLAP"}));
            }
            removed.push(bad["id"].clone());
            edges.retain(|e| e["id"] != bad["id"]);
        }
    }
    json!({"removed_native_relation_ids":removed,"new_exclusion_endpoints":exclusions,"projected_relations":edges,"prototype_only":true,"live_Graph_updated":false,"cache_updated":false})
}
fn refresh(g: &mut Value) {
    let linked = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "HeadStemRelation")
        .flat_map(|r| [r["source"].as_u64().unwrap(), r["target"].as_u64().unwrap()])
        .collect::<BTreeSet<_>>();
    for n in g["inters"].as_array_mut().unwrap() {
        if matches!(n["kind"].as_str(), Some("HeadInter" | "StemInter")) {
            n["abnormal"] = json!(!linked.contains(&n["id"].as_u64().unwrap()));
        }
    }
}
fn remove(g: &mut Value, id: u64, events: &mut Vec<Event>) {
    events.push(Event::RemoveVertex(id));
    g["inters"]
        .as_array_mut()
        .unwrap()
        .retain(|n| n["id"] != id);
    g["relations"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["source"] != id && e["target"] != id);
    refresh(g);
}
fn contextualize(g: &mut Value) {
    let mut nodes = g["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| matches!(n["kind"].as_str(), Some("HeadInter" | "StemInter")))
        .cloned()
        .collect::<Vec<_>>();
    let ids = nodes
        .iter()
        .map(|n| n["id"].as_u64().unwrap())
        .collect::<BTreeSet<_>>();
    let edges = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            ids.contains(&e["source"].as_u64().unwrap())
                && ids.contains(&e["target"].as_u64().unwrap())
        })
        .cloned()
        .collect::<Vec<_>>();
    context::contextualize_head_stems(&mut nodes, &edges).unwrap();
    for n in nodes {
        let old = g["inters"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["id"] == n["id"])
            .unwrap();
        old["contextual_grade"] = n["contextual_grade"].clone();
    }
}
fn purge(g: &mut Value, events: &mut Vec<Event>) {
    contextualize(g);
    let ids = g["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| {
            n["frozen"] != true
                && n["shape"] != "LEDGER"
                && n["contextual_grade"].as_f64().unwrap() < 0.5
        })
        .map(|n| n["id"].as_u64().unwrap())
        .collect::<Vec<_>>();
    for id in ids {
        remove(g, id, events)
    }
}
fn exclusion(g: &mut Value, a: u64, b: u64, events: &mut Vec<Event>) {
    let edges = g["relations"].as_array_mut().unwrap();
    if edges.iter().any(|e| {
        e["kind"] == "Exclusion"
            && ((e["source"] == a && e["target"] == b) || (e["source"] == b && e["target"] == a))
    }) {
        return;
    }
    events.push(Event::Exclusion(a, b));
    let id = u64::MAX - edges.len() as u64;
    edges.push(json!({"id":id,"kind":"Exclusion","source":a,"target":b,"cause":"OVERLAP","planned_synthetic_id":true}));
}
fn direction(g: &Value, s: &Value) -> i32 {
    let nodes = g["inters"].as_array().unwrap();
    let mut links = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "HeadStemRelation" && e["target"] == s["id"])
        .cloned()
        .collect::<Vec<_>>();
    let line = extended(s, &links);
    links.sort_by(|a, b| {
        let grade = |e: &Value| {
            nodes.iter().find(|n| n["id"] == e["source"]).unwrap()["contextual_grade"]
                .as_f64()
                .unwrap()
        };
        grade(b).total_cmp(&grade(a))
    });
    for e in links {
        let h = nodes.iter().find(|h| h["id"] == e["source"]).unwrap();
        let portion = stem_portion(
            serde_json::from_value(h["bounds"].clone()).unwrap(),
            line,
            e["extension_point"][1].as_f64().unwrap(),
        )
        .unwrap();
        if portion == StemPortion::StemBottom && e["head_side"] == "RIGHT" {
            return -1;
        }
        if portion == StemPortion::StemTop && e["head_side"] == "LEFT" {
            return 1;
        }
    }
    0
}
fn check_heads(g: &mut Value, events: &mut Vec<Event>) {
    let heads = g["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "HeadInter")
        .cloned()
        .collect::<Vec<_>>();
    for h in heads {
        let id = h["id"].as_u64().unwrap();
        let links = g["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "HeadStemRelation" && e["source"] == id)
            .cloned()
            .collect::<Vec<_>>();
        if links.is_empty() {
            remove(g, id, events);
            continue;
        }
        for e in links {
            let stem = g["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == e["target"])
                .cloned();
            let Some(s) = stem else { continue };
            let d = direction(g, &s);
            if d == 0 {
                remove(g, s["id"].as_u64().unwrap(), events);
                continue;
            }
            if (e["head_side"] == "LEFT" && d > 0) || (e["head_side"] == "RIGHT" && d < 0) {
                continue;
            }
            let pitch = h["pitch"].as_f64().unwrap().round_ties_even();
            let counterpart = g["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| {
                    r["kind"] == "HeadStemRelation"
                        && r["target"] == s["id"]
                        && r["head_side"] != e["head_side"]
                })
                .any(|r| {
                    let n = g["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == r["source"])
                        .unwrap();
                    n["staff_id"] == h["staff_id"]
                        && (n["pitch"].as_f64().unwrap().round_ties_even() - pitch).abs() <= 1.
                });
            if counterpart {
                continue;
            }
            events.push(Event::RemoveRelation(e["id"].as_u64().unwrap()));
            g["relations"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["id"] != e["id"]);
            if e["invading"] == true {
                exclusion(g, id, s["id"].as_u64().unwrap(), events);
            }
            refresh(g);
        }
    }
}
fn reduce(g: &mut Value, events: &mut Vec<Event>) {
    loop {
        let nodes = g["inters"].as_array().unwrap();
        let mut best: Option<(Value, f64)> = None;
        for e in g["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Exclusion")
        {
            let grade = |id: &Value| {
                nodes.iter().find(|n| n["id"] == *id).unwrap()["contextual_grade"]
                    .as_f64()
                    .unwrap()
            };
            let cp = grade(&e["source"]).max(grade(&e["target"]));
            if cp > best.as_ref().map_or(0., |b| b.1) {
                best = Some((e.clone(), cp));
            }
        }
        let Some((e, _)) = best else { break };
        let grade = |id: &Value| {
            nodes.iter().find(|n| n["id"] == *id).unwrap()["contextual_grade"]
                .as_f64()
                .unwrap()
        };
        let weak = if grade(&e["source"]) < grade(&e["target"]) {
            e["source"].as_u64().unwrap()
        } else {
            e["target"].as_u64().unwrap()
        };
        let partners = g["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["kind"] == "HeadStemRelation" && (r["source"] == weak || r["target"] == weak)
            })
            .map(|r| {
                if r["source"] == weak {
                    r["target"].as_u64().unwrap()
                } else {
                    r["source"].as_u64().unwrap()
                }
            })
            .collect::<BTreeSet<_>>();
        remove(g, weak, events);
        let before = g.clone();
        contextualize(g);
        for node in g["inters"].as_array_mut().unwrap() {
            if matches!(node["kind"].as_str(), Some("HeadInter" | "StemInter"))
                && !partners.contains(&node["id"].as_u64().unwrap())
            {
                node["contextual_grade"] = before["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == node["id"])
                    .unwrap()["contextual_grade"]
                    .clone();
            }
        }
    }
}

fn count(g: &Value) -> Value {
    let nodes = g["inters"].as_array().unwrap();
    json!({"heads":nodes.iter().filter(|n|n["kind"]=="HeadInter").count(),"stems":nodes.iter().filter(|n|n["kind"]=="StemInter").count(),"head_stem_relations":g["relations"].as_array().unwrap().iter().filter(|r|r["kind"]=="HeadStemRelation").count()})
}
pub(crate) fn prepare(input: &Value) -> Result<Plan, String> {
    let nodes = input["inters"]
        .as_array()
        .ok_or("missing_native_foundation_nodes")?;
    for n in nodes {
        // SIGraph.deleteWeakInters explicitly exempts ledgers. They remain
        // owned geometry; head/stem support does not assign their ctx grade.
        if n["kind"] == "LedgerInter" && n["shape"] == "LEDGER" {
            if n["id"].as_u64().is_none_or(|id| id == 0) {
                return Err("invalid_foundation_ledger_id".into());
            }
            continue;
        }
        if !n["contextual_grade"]
            .as_f64()
            .is_some_and(|g| (0.0..=1.0).contains(&g))
        {
            return Err("invalid_foundation_contextual_grade".into());
        }
        if !matches!(
            n["kind"].as_str(),
            Some("HeadInter" | "StemInter" | "BarlineInter")
        ) {
            return Err("unsupported_head_foundation_kind".into());
        }
        if matches!(n["kind"].as_str(), Some("HeadInter" | "StemInter"))
            && (n["frozen"] == true || n["is_frozen"] == true)
        {
            return Err("unsupported_frozen_foundation".into());
        }
        if n["kind"] == "HeadInter" {
            if !matches!(
                n["shape"].as_str(),
                Some("NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
            ) || n["frozen"] == true
                || n["is_frozen"] == true
            {
                return Err("unsupported_head_foundation_shape_or_frozen".into());
            }
            if !n["pitch"]
                .as_f64()
                .is_some_and(|p| p.is_finite() && p.abs() <= 64.)
                || n["staff_id"].as_u64().is_none()
            {
                return Err("missing_foundation_pitch_staff".into());
            }
        }
    }
    let edges = input["relations"]
        .as_array()
        .ok_or("missing_native_foundation_edges")?;
    if edges.iter().any(|e| {
        nodes.iter().any(|n| {
            n["kind"] == "LedgerInter" && (e["source"] == n["id"] || e["target"] == n["id"])
        })
    }) {
        return Err("ledger_foundation_relations_pending".into());
    }
    let mut verify = nodes
        .iter()
        .filter(|n| matches!(n["kind"].as_str(), Some("HeadInter" | "StemInter")))
        .cloned()
        .collect::<Vec<_>>();
    let ids = verify
        .iter()
        .map(|n| n["id"].as_u64().unwrap())
        .collect::<BTreeSet<_>>();
    let verify_edges = edges
        .iter()
        .filter(|e| {
            ids.contains(&e["source"].as_u64().unwrap())
                && ids.contains(&e["target"].as_u64().unwrap())
        })
        .cloned()
        .collect::<Vec<_>>();
    context::contextualize_head_stems(&mut verify, &verify_edges)?;
    let mut events = Vec::new();
    let mut g = input.clone();
    let p = plan(&g);
    for id in p["removed_native_relation_ids"].as_array().unwrap() {
        events.push(Event::RemoveRelation(id.as_u64().unwrap()));
    }
    g["relations"] = p["projected_relations"].clone();
    for e in p["new_exclusion_endpoints"].as_array().unwrap() {
        exclusion(
            &mut g,
            e["source"].as_u64().unwrap(),
            e["target"].as_u64().unwrap(),
            &mut events,
        );
    }
    refresh(&mut g);
    let ending = count(&g);
    purge(&mut g, &mut events);
    let ending_purge = count(&g);
    check_heads(&mut g, &mut events);
    let heads = count(&g);
    purge(&mut g, &mut events);
    let head_purge = count(&g);
    reduce(&mut g, &mut events);
    let linked = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "HeadStemRelation")
        .map(|e| e["target"].as_u64().unwrap())
        .collect::<BTreeSet<_>>();
    if g["inters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["kind"] == "StemInter" && !linked.contains(&n["id"].as_u64().unwrap()))
    {
        return Err("orphan_stem_requires_remaining_original_consistency_checks".into());
    }
    Ok(Plan {
        events,
        stages: json!({"ending":ending,"ending_purge":ending_purge,"heads":heads,"heads_purge":head_purge,"reduced":count(&g)}),
        final_state: g,
    })
}
