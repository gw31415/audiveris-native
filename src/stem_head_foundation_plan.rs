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
    fn original_stem_checks_prune_tail_heads_by_all_connection_source_grades() {
        use crate::shared_sig::{Graph, Key, Phase};
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-consistency/controls.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let sk = Key {
                system: 1,
                phase: Phase::Stem,
                local: 1,
            };
            let mut data = case["stem"].clone();
            data["abnormal"] = json!(case["heads"].as_array().unwrap().is_empty());
            let sid = graph.vertex(sk, data, false).unwrap();
            let mut head_ids = Vec::new();
            let mut head_edges = Vec::new();
            for (i, (head, rel)) in case["heads"]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["head_relations"].as_array().unwrap())
                .enumerate()
            {
                let hk = Key {
                    system: 1,
                    phase: Phase::Head,
                    local: i + 1,
                };
                let mut data = head.clone();
                data["abnormal"] = json!(false);
                head_ids.push(graph.vertex(hk, data, false).unwrap());
                let mut relation = rel.clone();
                relation["local_relation_id"] = json!(i + 1);
                graph
                    .relations(1, Phase::Stem, vec![(hk, sk, relation)])
                    .unwrap();
                head_edges.push(
                    graph.snapshot()["relations"]
                        .as_array()
                        .unwrap()
                        .last()
                        .unwrap()["id"]
                        .as_u64()
                        .unwrap() as usize,
                );
            }
            if !case["beam"].is_null() {
                let bk = Key {
                    system: 1,
                    phase: Phase::Beam,
                    local: 1,
                };
                graph.vertex(bk, case["beam"].clone(), false).unwrap();
                graph
                    .relations(
                        1,
                        Phase::Beam,
                        vec![(bk, sk, case["beam_relation"].clone())],
                    )
                    .unwrap();
            }
            let before = graph.snapshot();
            assert_eq!(
                json!(direction(&before, &before["inters"][0])),
                case["direction"],
                "{}",
                case["name"]
            );
            let plan = prepare_stem_checks(&before).unwrap();
            assert_eq!(plan.stages["modified_stems"], case["modified"]);
            graph.apply_head_foundation_plan(&plan).unwrap();
            let after = graph.snapshot();
            assert_eq!(
                !after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|n| n["id"] == sid),
                case["stem_removed"].as_bool().unwrap()
            );
            for (i, (&hid, &eid)) in head_ids.iter().zip(&head_edges).enumerate() {
                assert_eq!(
                    after["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["id"] == eid),
                    case["retained_head_relations"][i].as_bool().unwrap()
                );
                assert_eq!(
                    after["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["kind"] == "Exclusion"
                            && ((r["source"] == hid && r["target"] == sid)
                                || (r["target"] == hid && r["source"] == sid))),
                    case["head_stem_exclusions"][i].as_bool().unwrap()
                );
            }
        }
    }
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
    fn orphan_stem_is_retained_until_original_stem_consistency_stage() {
        let input = json!({"inters":[{"id":1,"kind":"StemInter","grade":0.8,"contextual_grade":0.8,"abnormal":true,"median":[1.,0.,1.,10.]}],"relations":[]});
        let before = input.clone();
        assert_eq!(prepare(&input).unwrap().final_state, before);
        let checked = prepare_stem_checks(&input).unwrap();
        assert!(checked.final_state["inters"].as_array().unwrap().is_empty());
        assert!(matches!(
            checked.events.as_slice(),
            [Event::RemoveVertex(1)]
        ));
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
        if e["extension_point"].is_null() {
            continue;
        }
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
            let beam_links = edges
                .iter()
                .filter(|e| e["kind"] == "BeamStemRelation" && e["target"] == stem["id"])
                .cloned()
                .collect::<Vec<_>>();
            let mut extensions = links.clone();
            extensions.extend(beam_links);
            let line = extended(stem, &extensions);
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
    // Mirror the actual retained Graph's original ensemble callback.
    crate::beam_groups::remove_owned_members(g, &[id]).expect("validated owned beam group removal");
    refresh(g);
}
fn contextualize(g: &mut Value) {
    let mut nodes = g["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| {
            matches!(
                n["kind"].as_str(),
                Some("HeadInter" | "StemInter" | "BeamInter" | "BeamHookInter")
            )
        })
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
            n["kind"] != "BeamGroupInter"
                && n["frozen"] != true
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
pub(crate) fn direction(g: &Value, s: &Value) -> i32 {
    let nodes = g["inters"].as_array().unwrap();
    let mut links = g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            matches!(
                e["kind"].as_str(),
                Some(
                    "HeadStemRelation"
                        | "BeamStemRelation"
                        | "FlagStemRelation"
                        | "TremoloStemRelation"
                )
            ) && e["target"] == s["id"]
        })
        .cloned()
        .collect::<Vec<_>>();
    let line = extended(s, &links);
    links.sort_by(|a, b| {
        let grade = |e: &Value| {
            let n = nodes.iter().find(|n| n["id"] == e["source"]).unwrap();
            n["contextual_grade"]
                .as_f64()
                .or_else(|| n["grade"].as_f64())
                .unwrap()
        };
        grade(b).total_cmp(&grade(a))
    });
    for e in links {
        // StemInter.computeDirection sorts every stem connection by the
        // source's best grade. A stronger beam can decide ahead of a head.
        if e["kind"] == "BeamStemRelation" {
            return if e["extension_point"][1].as_f64().unwrap() < (line[1] + line[3]) / 2. {
                -1
            } else {
                1
            };
        }
        if e["kind"] == "FlagStemRelation" {
            let f = nodes.iter().find(|n| n["id"] == e["source"]).unwrap();
            let il = g["interline"].as_f64().unwrap();
            let y = e["extension_point"][1].as_f64().unwrap();
            if matches!(
                f["shape"].as_str(),
                Some("FLAG_1" | "FLAG_2" | "FLAG_3" | "FLAG_4" | "FLAG_5")
            ) {
                if y < line[1] + il {
                    return -1;
                }
            } else if y > line[3] - il {
                return 1;
            }
            continue;
        }
        if e["kind"] != "HeadStemRelation" {
            continue;
        }
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
fn check_heads(g: &mut Value, events: &mut Vec<Event>) -> usize {
    let mut modified = 0;
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
            modified += 1;
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
                modified += 1;
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
            modified += 1;
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
    modified
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
                matches!(
                    r["kind"].as_str(),
                    Some(
                        "HeadStemRelation"
                            | "BeamStemRelation"
                            | "BeamHeadRelation"
                            | "BeamBeamRelation"
                            | "HeadHeadRelation"
                    )
                ) && (r["source"] == weak || r["target"] == weak)
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
            if matches!(
                node["kind"].as_str(),
                Some("HeadInter" | "StemInter" | "BeamInter" | "BeamHookInter")
            ) && !partners.contains(&node["id"].as_u64().unwrap())
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

/// Single original check boundary, without an early weak/exclusion reduction.
pub(crate) fn prepare_ending_heads(input: &Value) -> Result<Plan, String> {
    validate(input)?;
    let p = plan(input);
    let mut g = input.clone();
    let mut events = Vec::new();
    let mut stems = BTreeSet::new();
    for id in p["removed_native_relation_ids"].as_array().unwrap() {
        let edge = input["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == *id)
            .unwrap();
        stems.insert(edge["target"].as_u64().unwrap());
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
    Ok(Plan {
        events,
        stages: json!({"modified":stems.len()}),
        final_state: g,
    })
}

pub(crate) fn prepare_heads(input: &Value) -> Result<Plan, String> {
    validate(input)?;
    let mut g = input.clone();
    let mut events = Vec::new();
    let modified = check_heads(&mut g, &mut events);
    Ok(Plan {
        events,
        stages: json!({"modified":modified}),
        final_state: g,
    })
}

/// Existing supported exclusion algorithm at the original post-consistency
/// boundary. Full overlap discovery/chord prolog/late checks remain pending.
pub(crate) fn prepare_exclusions(input: &Value) -> Result<Plan, String> {
    validate(input)?;
    let mut g = input.clone();
    let mut events = Vec::new();
    reduce(&mut g, &mut events);
    let modified = events
        .iter()
        .filter(|e| matches!(e, Event::RemoveVertex(_)))
        .count();
    Ok(Plan {
        events,
        stages: json!({"modified":modified}),
        final_state: g,
    })
}

fn count(g: &Value) -> Value {
    let nodes = g["inters"].as_array().unwrap();
    json!({"heads":nodes.iter().filter(|n|n["kind"]=="HeadInter").count(),"stems":nodes.iter().filter(|n|n["kind"]=="StemInter").count(),"head_stem_relations":g["relations"].as_array().unwrap().iter().filter(|r|r["kind"]=="HeadStemRelation").count()})
}
pub(crate) fn validate(input: &Value) -> Result<(), String> {
    validate_role(input, false)
}
/// analyzeChords has original intrinsic shape/size semantics for all retained
/// heads. This does not admit them into the still ordinary-only cleaner/checks.
pub(crate) fn validate_chord_inputs(input: &Value) -> Result<(), String> {
    validate_role(input, true)
}
fn validate_role(input: &Value, chords: bool) -> Result<(), String> {
    let nodes = input["inters"]
        .as_array()
        .ok_or("missing_native_foundation_nodes")?;
    for n in nodes {
        // SIGraph.deleteWeakInters explicitly exempts ledgers. They remain
        // owned geometry; head/stem support does not assign their ctx grade.
        if n["kind"] == "BeamGroupInter" || (n["kind"] == "LedgerInter" && n["shape"] == "LEDGER") {
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
            Some(
                "HeadInter"
                    | "StemInter"
                    | "BeamInter"
                    | "BeamHookInter"
                    | "BarlineInter"
                    | "BarConnectorInter"
                    | "BraceInter"
            )
        ) && !(chords && n["kind"] == "SmallBeamInter")
        {
            return Err(format!(
                "unsupported_head_foundation_kind:{}",
                n["kind"].as_str().unwrap_or("unknown")
            ));
        }
        if matches!(n["kind"].as_str(), Some("BarConnectorInter" | "BraceInter")) {
            let key: crate::shared_sig::Key = serde_json::from_value(n["origin"].clone())
                .map_err(|_| "foundation_actual_GRID_origin_required")?;
            if key.phase != crate::shared_sig::Phase::Grid || key.system == 0 || key.local == 0 {
                return Err("foundation_actual_GRID_origin_required".into());
            }
        }
        if matches!(n["kind"].as_str(), Some("HeadInter" | "StemInter"))
            && (n["frozen"] == true || n["is_frozen"] == true)
        {
            return Err("unsupported_frozen_foundation".into());
        }
        if n["kind"] == "HeadInter" {
            let supported_shape = n["shape"].as_str().is_some_and(|shape| {
                if chords {
                    crate::head_registration::intrinsic_duration_quarters(shape).is_some()
                } else {
                    matches!(shape, "NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                }
            });
            if !supported_shape || n["frozen"] == true || n["is_frozen"] == true {
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
        .filter(|n| {
            matches!(
                n["kind"].as_str(),
                Some("HeadInter" | "StemInter" | "BeamInter" | "BeamHookInter")
            ) || (chords && n["kind"] == "SmallBeamInter")
        })
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
    crate::beam_groups::validate_owned_groups(input)?;
    Ok(())
}
pub(crate) fn prepare(input: &Value) -> Result<Plan, String> {
    validate(input)?;
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
    Ok(Plan {
        events,
        stages: json!({"ending":ending,"ending_purge":ending_purge,"heads":heads,"heads_purge":head_purge,"reduced":count(&g)}),
        final_state: g,
    })
}

/// SigReducer.checkStems after beam endpoints/ledgers: delete stems without
/// heads, then stemHasSingleHeadEnd removes tail-side links and inserts an
/// overlap exclusion for invading links. This is not the full reduction loop.
pub(crate) fn prepare_stem_checks(input: &Value) -> Result<Plan, String> {
    validate(input)?;
    let mut g = input.clone();
    let mut events = Vec::new();
    let mut modified = 0;
    let stems = g["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "StemInter")
        .cloned()
        .collect::<Vec<_>>();
    for stem in stems {
        let links = g["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "HeadStemRelation" && r["target"] == stem["id"])
            .cloned()
            .collect::<Vec<_>>();
        if links.is_empty() {
            remove(&mut g, stem["id"].as_u64().unwrap(), &mut events);
            modified += 1;
            continue;
        }
        let dir = direction(&g, &stem);
        if dir == 0 {
            continue;
        }
        let connections = g["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                matches!(
                    r["kind"].as_str(),
                    Some("HeadStemRelation" | "BeamStemRelation")
                ) && r["target"] == stem["id"]
            })
            .cloned()
            .collect::<Vec<_>>();
        let line = extended(&stem, &connections);
        let forbidden = if dir > 0 {
            StemPortion::StemBottom
        } else {
            StemPortion::StemTop
        };
        let mut pruned = false;
        for rel in links {
            let head = g["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|h| h["id"] == rel["source"])
                .unwrap();
            let portion = stem_portion(
                serde_json::from_value(head["bounds"].clone()).unwrap(),
                line,
                rel["extension_point"][1].as_f64().unwrap(),
            )
            .unwrap();
            if portion != forbidden {
                continue;
            }
            events.push(Event::RemoveRelation(rel["id"].as_u64().unwrap()));
            g["relations"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["id"] != rel["id"]);
            if rel["invading"] == true {
                exclusion(
                    &mut g,
                    rel["source"].as_u64().unwrap(),
                    stem["id"].as_u64().unwrap(),
                    &mut events,
                );
            }
            pruned = true;
        }
        if pruned {
            modified += 1;
            refresh(&mut g);
        }
    }
    Ok(Plan {
        events,
        stages: json!({"modified_stems":modified,"remaining":count(&g)}),
        final_state: g,
    })
}
