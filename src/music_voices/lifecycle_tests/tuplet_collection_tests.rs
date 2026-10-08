// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::shared_sig::Key;
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-remove-collection/observed.json"
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
fn prepare(case: &Value) -> Prepared {
    let mut p = control(case);
    // Manual Measure.addInter does not undo the earlier bounded added() membership.
    p.c.owner.measures[0].stack_tuplets.insert(0, p.tuplets[0]);
    if let Some(name) = case["foreign_tuplet"].as_str() {
        let key = p.tuplets[name
            .strip_prefix("tuplet")
            .unwrap()
            .parse::<usize>()
            .unwrap()];
        p.c.owner.measures[1].stack_tuplets.push(key);
    }
    p
}
fn names(p: &Prepared, keys: &[Key]) -> Value {
    json!(
        keys.iter()
            .map(|key| format!("tuplet{}", p.tuplets.iter().position(|t| t == key).unwrap()))
            .collect::<Vec<_>>()
    )
}
fn all(measure: &[Key], unassigned: &[Key]) -> Vec<Key> {
    let mut v = measure.to_vec();
    for &key in unassigned {
        if !v.contains(&key) {
            v.push(key);
        }
    }
    v
}
fn state(p: &mut Prepared) -> Value {
    let tuplets = p
        .tuplets
        .iter()
        .enumerate()
        .map(|(i, &key)| {
            let mut r = p.c.owner.owner.graph.tuplet_registration(key);
            r["label"] = json!(format!("tuplet{i}"));
            r
        })
        .collect::<Vec<_>>();
    let relations = p
        .relations
        .iter()
        .map(|(id, label)| {
            let mut r = p.c.owner.owner.graph.tuplet_relation_registration(*id);
            r["label"] = json!(label);
            r
        })
        .collect::<Vec<_>>();
    json!({"measure_tuplets":names(p,&p.c.owner.measures[0].tuplets),"stack_tuplets":names(p,&all(&p.c.owner.measures[0].tuplets,&p.c.owner.measures[0].stack_tuplets)),"other_stack_tuplets":names(p,&all(&p.c.owner.measures[1].tuplets,&p.c.owner.measures[1].stack_tuplets)),"root_unassigned":names(p,&p.c.owner.measures[0].stack_tuplets),"other_unassigned":names(p,&p.c.owner.measures[1].stack_tuplets),"tuplets":tuplets,"relations":relations,"state":frame(&mut p.c)})
}
fn normalized(mut v: Value) -> Value {
    v.as_object_mut().unwrap().remove("phase");
    v["state"]["owned"]["relations"] = json!([]);
    v
}
fn requested(p: &Prepared, case: &Value) -> Vec<Option<Key>> {
    case["requested"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| {
            name.as_str()
                .map(|s| p.tuplets[s.strip_prefix("tuplet").unwrap().parse::<usize>().unwrap()])
        })
        .collect()
}
#[test]
fn original_collection_removes_actual_explicit_implicit_duplicate_refs_in_order_with_other_stack_and_old_clocks()
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
            state(&mut p),
            normalized(case["phases"][0].clone()),
            "{} before",
            case["name"]
        );
        let request = requested(&p, case);
        p.c.owner.owner.graph.trace_tuplet_removals();
        p.c.owner.remove_rhythm_tuplets(1, &request).unwrap();
        let observations = p.c.owner.owner.graph.take_tuplet_removals();
        let observed=observations.iter().filter(|cb|cb.phase=="finished").map(|cb|{
            let held=std::mem::replace(&mut p.c.owner.owner.graph,cb.graph.clone());let clocks=frame(&mut p.c)["owned"].clone();p.c.owner.owner.graph=held;
            let containers=cb.containers.as_ref().unwrap();
            json!({"edge":p.relations.iter().find(|(id,_)|*id==cb.edge).unwrap().1,"tuplet":names(&p,&[cb.tuplet])[0],"registered":cb.state["registered"],"removed":cb.state["removed"],"manual":cb.state["manual"],"measure_tuplets":names(&p,&cb.measure_tuplets),"stack_tuplets":names(&p,&cb.stack_tuplets),"other_stack_tuplets":names(&p,&all(&containers[1].0,&containers[1].1)),"root_unassigned":names(&p,&containers[0].1),"other_unassigned":names(&p,&containers[1].1),"clocks":clocks})
        }).collect::<Vec<_>>();
        let mut expected_callbacks = case["callbacks"].clone();
        for cb in expected_callbacks.as_array_mut().unwrap() {
            cb["clocks"]["relations"] = json!([]);
        }
        assert_eq!(
            json!(observed),
            expected_callbacks,
            "{} callbacks",
            case["name"]
        );
        assert_eq!(
            state(&mut p),
            normalized(case["phases"][1].clone()),
            "{} after",
            case["name"]
        );
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}
#[test]
fn null_collection_member_restores_earlier_removed_actual_tuplet_containers_relations_and_clock_owners()
 {
    let d = original();
    let case = d["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["error"] == "NullPointerException")
        .unwrap();
    assert_ne!(case["phases"][0], case["phases"][1]);
    assert_eq!(case["callbacks"].as_array().unwrap().len(), 2);
    let mut p = prepare(case);
    let before = state(&mut p);
    let graph = p.c.owner.owner.graph.snapshot();
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let containers =
        p.c.owner
            .measures
            .iter()
            .map(|m| (m.tuplets.as_ptr(), m.stack_tuplets.as_ptr()))
            .collect::<Vec<_>>();
    let request = requested(&p, case);
    p.c.owner.owner.graph.trace_tuplet_removals();
    assert_eq!(
        p.c.owner.remove_rhythm_tuplets(1, &request).unwrap_err(),
        "music_tuplet_null_collection_member"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(state(&mut p), before);
    assert!(p.c.owner.owner.graph.take_tuplet_removals().is_empty());
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    for (m, (tuples, stack)) in p.c.owner.measures.iter().zip(containers) {
        assert_eq!(m.tuplets.as_ptr(), tuples);
        assert_eq!(m.stack_tuplets.as_ptr(), stack);
    }
    p.c.owner.remove_rhythm_tuplets(1, &request[..1]).unwrap();
    assert_eq!(state(&mut p), normalized(case["phases"][1].clone()));
    assert_eq!(p.c.owner.owner.graph.take_tuplet_removals().len(), 4);
}
