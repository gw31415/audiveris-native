// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::{
    music_font_symbols::TupletFontSymbols,
    shared_sig::{Graph, Key},
};
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-merge/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    for f in d["clock_sets"].as_array_mut().unwrap() {
        f["relations"] = rs[f["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    let cs = d["clock_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
        for cb in c["callbacks"].as_array_mut().unwrap() {
            cb["clocks"] = cs[cb["clocks"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
pub(super) fn prepare(case: &Value) -> Prepared {
    let extra = case["declared_extra_groups"].as_array().unwrap();
    let mut p = if extra.is_empty() {
        control(case)
    } else {
        let mut declared = case.clone();
        declared["declared_extra_group"] = extra[0].clone();
        super::tuplet_reuse_tests::prepare(&declared)
    };
    p.c.owner.measures[0].stack_tuplets.insert(0, p.tuplets[0]);
    // added() leaves implicit and these incomplete explicit signs abnormal.
    for &key in &p.tuplets {
        p.c.owner
            .owner
            .graph
            .set_tuplet_abnormal(key, true)
            .unwrap();
    }
    for row in case["geometry"].as_array().unwrap() {
        let key = chord_key(&p.c, row["chord"].as_str().unwrap());
        p.c.owner
            .owner
            .graph
            .declare_chord_geometry(
                key,
                serde_json::from_value(row["bounds"].clone()).unwrap(),
                serde_json::from_value(row["head"].clone()).unwrap(),
                serde_json::from_value(row["tail"].clone()).unwrap(),
            )
            .unwrap();
    }
    p
}
pub(super) fn name(p: &Prepared, key: Key, new: &[Key]) -> String {
    p.tuplets
        .iter()
        .position(|&t| t == key)
        .map(|i| format!("tuplet{i}"))
        .unwrap_or_else(|| format!("generated{}", new.iter().position(|&t| t == key).unwrap()))
}
fn names(p: &Prepared, keys: &[Key], new: &[Key]) -> Value {
    json!(keys.iter().map(|&k| name(p, k, new)).collect::<Vec<_>>())
}
fn all(a: &[Key], b: &[Key]) -> Vec<Key> {
    let mut v = a.to_vec();
    for &key in b {
        if !v.contains(&key) {
            v.push(key);
        }
    }
    v
}
fn signs(p: &Prepared, g: &Graph, new: &[Key], details: bool) -> Value {
    json!(
        p.tuplets
            .iter()
            .chain(new)
            .map(|&key| {
                let mut row = g.tuplet_registration(key);
                row["label"] = json!(name(p, key, new));
                if details {
                    let node = g.retained_tuplet_body(key);
                    row["shape"] = node["shape"].clone();
                    row["bounds"] = node["bounds"].clone();
                    row["chords"] = if row["removed"] == true {
                        Value::Null
                    } else {
                        let mut group = g.tuplet_chords(key).unwrap();
                        group.sort_by_key(|key| g.retained_or_live_chord_bounds(*key).unwrap()[0]);
                        json!(
                            group
                                .iter()
                                .map(|k| p.c.labels[&g.id(*k).unwrap()].clone())
                                .collect::<Vec<_>>()
                        )
                    };
                }
                row
            })
            .collect::<Vec<_>>()
    )
}
pub(super) fn state(p: &mut Prepared, new: &[Key]) -> Value {
    let relations = p
        .relations
        .iter()
        .map(|(id, label)| {
            let mut r = p.c.owner.owner.graph.tuplet_relation_registration(*id);
            r["label"] = json!(label);
            r
        })
        .collect::<Vec<_>>();
    let m = &p.c.owner.measures;
    json!({"measure_tuplets":names(p,&m[0].tuplets,new),"stack_tuplets":names(p,&all(&m[0].tuplets,&m[0].stack_tuplets),new),"other_stack_tuplets":names(p,&all(&m[1].tuplets,&m[1].stack_tuplets),new),"root_unassigned":names(p,&m[0].stack_tuplets,new),"other_unassigned":names(p,&m[1].stack_tuplets,new),"tuplets":signs(p,&p.c.owner.owner.graph,new,true),"relations":relations,"state":frame(&mut p.c)})
}
pub(super) fn normalized(mut v: Value) -> Value {
    v.as_object_mut().unwrap().remove("phase");
    v["state"]["owned"]["relations"] = json!([]);
    v
}
pub(super) fn compare(actual: Value, expected: Value, label: &str) {
    if actual != expected {
        std::fs::write(
            format!("/tmp/tuplet-merge-{label}-native.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
        std::fs::write(
            format!("/tmp/tuplet-merge-{label}-source.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(actual, expected, "{label}");
}
#[test]
fn original_merge_keeps_stale_previous_groups_and_actual_beam_identity_with_ordered_remove_then_generate_callbacks()
 {
    for case in original()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["error"].is_null())
    {
        let mut p = prepare(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        assert_eq!(
            state(&mut p, &[]),
            normalized(case["phases"][0].clone()),
            "{} before",
            case["name"]
        );
        let font =
            TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
                .unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        p.c.owner.owner.graph.trace_tuplet_removals();
        let returned = p.c.owner.merge_rhythm_tuplets(1, &font).unwrap();
        let (new, callbacks) = observed_callbacks(&mut p);
        let mut source = case["callbacks"].clone();
        for cb in source.as_array_mut().unwrap() {
            cb["clocks"]["relations"] = json!([]);
        }
        compare(
            callbacks,
            source,
            &format!("{}-callbacks", case["name"].as_str().unwrap()),
        );
        compare(
            state(&mut p, &new),
            normalized(case["phases"][1].clone()),
            &format!("{}-after", case["name"].as_str().unwrap()),
        );
        assert!(
            returned
                .iter()
                .all(|key| p.tuplets.contains(key) || new.contains(key))
        );
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}
#[test]
fn merge_registration_failure_restores_actual_old_signs_manual_relations_containers_and_clocks() {
    let d = original();
    let case = d["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["error"] == "IllegalStateException")
        .unwrap();
    assert_eq!(case["callbacks"].as_array().unwrap().len(), 3);
    assert_ne!(case["phases"][0], case["phases"][1]);
    assert_eq!(case["phases"][1]["tuplets"][3]["registered"], true);
    assert!(case["phases"][1]["tuplets"][3]["chords"].is_null());
    let mut p = prepare(case);
    let before = state(&mut p, &[]);
    let graph = p.c.owner.owner.graph.snapshot();
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let containers =
        p.c.owner
            .measures
            .iter()
            .map(|m| (m.tuplets.as_ptr(), m.stack_tuplets.as_ptr()))
            .collect::<Vec<_>>();
    let font = TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
        .unwrap();
    p.c.owner.owner.graph.trace_tuplet_additions();
    p.c.owner.owner.graph.trace_tuplet_removals();
    p.c.owner
        .owner
        .graph
        .fail_tuplet_registration_after(Some(1));
    assert_eq!(
        p.c.owner.merge_rhythm_tuplets(1, &font).unwrap_err(),
        "music_tuplet_addition_observer_failure"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(state(&mut p, &[]), before);
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    assert!(p.c.owner.owner.graph.take_tuplet_removals().is_empty());
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    for (m, (tuplets, stack)) in p.c.owner.measures.iter().zip(containers) {
        assert_eq!(m.tuplets.as_ptr(), tuplets);
        assert_eq!(m.stack_tuplets.as_ptr(), stack);
    }
    p.c.owner.owner.graph.fail_tuplet_registration_after(None);
    p.c.owner.merge_rhythm_tuplets(1, &font).unwrap();
    let new =
        p.c.owner
            .owner
            .graph
            .take_tuplet_additions()
            .iter()
            .filter(|cb| cb.phase == "vertex_registered")
            .map(|cb| cb.tuplet)
            .collect::<Vec<_>>();
    let source = &d["cases"][0];
    compare(
        state(&mut p, &new),
        normalized(source["phases"][1].clone()),
        "retry",
    );
}

pub(super) fn observed_callbacks(p: &mut Prepared) -> (Vec<Key>, Value) {
    let additions = p.c.owner.owner.graph.take_tuplet_additions();
    let removals = p.c.owner.owner.graph.take_tuplet_removals();
    let mut new = Vec::new();
    for cb in &additions {
        if !new.contains(&cb.tuplet) {
            new.push(cb.tuplet);
        }
    }
    let mut callbacks = Vec::new();
    for cb in removals.iter().filter(|cb| cb.phase == "finished") {
        let containers = cb.containers.as_ref().unwrap();
        let held = std::mem::replace(&mut p.c.owner.owner.graph, cb.graph.clone());
        let clocks = frame(&mut p.c)["owned"].clone();
        p.c.owner.owner.graph = held;
        let snapshot = cb.graph.snapshot();
        let node = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == cb.graph.id(cb.tuplet).unwrap())
            .unwrap();
        callbacks.push((cb.order,json!({"phase":"chord_tuplet_removed","edge":p.relations.iter().find(|(id,_)|*id==cb.edge).unwrap().1,"tuplet":name(p,cb.tuplet,&new),"sig_attached":cb.sig_attached,"registered":cb.state["registered"],"removed":cb.state["removed"],"abnormal":node["abnormal"]==true,"manual":cb.state["manual"],"measure_tuplets":names(p,&cb.measure_tuplets,&new),"stack_tuplets":names(p,&cb.stack_tuplets,&new),"root_unassigned":names(p,&containers[0].1,&new),"old_tuplets":signs(p,&cb.graph,&[],false),"clocks":clocks})));
    }
    for cb in &additions {
        let held = std::mem::replace(&mut p.c.owner.owner.graph, cb.graph.clone());
        let clocks = frame(&mut p.c)["owned"].clone();
        p.c.owner.owner.graph = held;
        let snapshot = cb.graph.snapshot();
        let node = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == cb.graph.id(cb.tuplet).unwrap())
            .unwrap();
        callbacks.push((cb.order,json!({"phase":cb.phase,"edge":null,"tuplet":name(p,cb.tuplet,&new),"sig_attached":cb.sig_attached,"registered":cb.graph.tuplet_registration(cb.tuplet)["registered"],"removed":cb.graph.tuplet_registration(cb.tuplet)["removed"],"abnormal":node["abnormal"]==true,"manual":cb.edge.map(|id|cb.graph.tuplet_relation_registration(id)["manual"].clone()),"measure_tuplets":names(p,&cb.measure_tuplets,&new),"stack_tuplets":names(p,&cb.stack_tuplets,&new),"root_unassigned":names(p,&cb.container_members[0].1,&new),"old_tuplets":signs(p,&cb.graph,&[],false),"clocks":clocks})));
    }
    callbacks.sort_by_key(|(order, _)| *order);
    (
        new,
        json!(callbacks.into_iter().map(|(_, v)| v).collect::<Vec<_>>()),
    )
}
