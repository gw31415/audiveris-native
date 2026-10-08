// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame};
use super::beam_rest_linked_tests::{chord_key, declare_links};
use super::beam_rest_polygon_tests::{declare, prepare};
use super::*;
use crate::shared_sig::{Key, Phase, TupletRemovalObservation};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-preparation/observed.json"
    ))
    .unwrap();
    let frames = d["frame_sets"].as_array().unwrap().clone();
    let clocks = d["clock_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = frames[p["state"].as_u64().unwrap() as usize].clone();
        }
        for cb in c["callbacks"].as_array_mut().unwrap() {
            cb["clocks"] = clocks[cb["clocks"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
pub(super) struct Prepared {
    pub(super) c: Control,
    pub(super) tuplets: Vec<Key>,
    pub(super) relations: Vec<(usize, String)>,
}
pub(super) fn control(case: &Value) -> Prepared {
    let (mut c, _) = prepare(case);
    if case["geometry"].is_object() {
        declare(&mut c, case, &case["geometry"]);
    }
    declare_links(&mut c, case);
    let snapshot = c.owner.owner.graph.snapshot();
    let local = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
        .filter(|k| k.system == 1 && k.phase == Phase::Symbol)
        .map(|k| k.local)
        .max()
        .unwrap()
        + 1;
    let mut tuplets = Vec::new();
    for (i, spec) in case["declared_tuplets"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let key = Key {
            system: 1,
            phase: Phase::Symbol,
            local: local + i,
        };
        let id=c.owner.owner.graph.vertex(key,json!({"kind":"TupletInter","shape":spec["shape"],"implicit":spec["implicit"],"manual":spec["manual"],"bounds":spec["bounds"],"grade":0.8}),false).unwrap();
        c.labels.insert(id, spec["label"].as_str().unwrap().into());
        c.owner
            .register_rhythm_tuplet(1, key, spec["unassigned"] == true)
            .unwrap();
        tuplets.push(key);
    }
    let mut relations = Vec::new();
    for (i, spec) in case["declared_tuple_links"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let source = chord_key(&c, spec["source"].as_str().unwrap());
        let target = tuplets[spec["target"]
            .as_str()
            .unwrap()
            .strip_prefix("tuplet")
            .unwrap()
            .parse::<usize>()
            .unwrap()];
        c.owner
            .owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![(
                    source,
                    target,
                    json!({"kind":"ChordTupletRelation","manual":spec["manual"]}),
                )],
            )
            .unwrap();
        let s = c.owner.owner.graph.snapshot();
        let id = s["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["kind"] == "ChordTupletRelation"
                    && e["source"] == c.owner.owner.graph.id(source).unwrap()
                    && e["target"] == c.owner.owner.graph.id(target).unwrap()
            })
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        relations.push((id, format!("tuple_link{i}")));
    }
    if !c.rests.is_empty() {
        let snapshot = c.owner.owner.graph.snapshot();
        let beam: Key = serde_json::from_value(
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    c.labels
                        .get(&(n["id"].as_u64().unwrap() as usize))
                        .is_some_and(|l| l == "competition_beam0")
                })
                .unwrap()["origin"]
                .clone(),
        )
        .unwrap();
        for i in 0..2 {
            let ch = c
                .owner
                .chords
                .iter()
                .find(|ch| ch.key == c.rests[i])
                .unwrap();
            let note = serde_json::from_value(ch.members[0]["origin"].clone()).unwrap();
            c.owner
                .owner
                .graph
                .add_owned_beam_rest(
                    beam,
                    note,
                    i == 1,
                    c.owner.owner.sheet_skew,
                    c.owner.owner.interline,
                )
                .unwrap();
            c.owner
                .owner
                .graph
                .invalidate_owned_rest_cache(ch.key)
                .unwrap();
            c.baseline[i] = c.owner.owner.graph.rest_chord_beams(ch.key).unwrap();
        }
    }
    Prepared {
        c,
        tuplets,
        relations,
    }
}
fn names(c: &Control, keys: &[Key]) -> Value {
    let snapshot = c.owner.owner.graph.snapshot();
    json!(
        keys.iter()
            .map(|key| {
                let id = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .chain(snapshot["tombstones"].as_array().unwrap())
                    .find(|n| n["origin"] == json!(key))
                    .unwrap()["id"]
                    .as_u64()
                    .unwrap() as usize;
                c.labels[&id].clone()
            })
            .collect::<Vec<_>>()
    )
}
pub(super) fn state(p: &mut Prepared) -> Value {
    let mut all = p.c.owner.measures[0].tuplets.clone();
    for &key in &p.c.owner.measures[0].stack_tuplets {
        if !all.contains(&key) {
            all.push(key);
        }
    }
    let ts = p
        .tuplets
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let mut r = p.c.owner.owner.graph.tuplet_registration(*key);
            r["label"] = json!(format!("tuplet{i}"));
            r
        })
        .collect::<Vec<_>>();
    let rs = p
        .relations
        .iter()
        .map(|(id, label)| {
            let mut r = p.c.owner.owner.graph.tuplet_relation_registration(*id);
            r["label"] = json!(label);
            r
        })
        .collect::<Vec<_>>();
    json!({"implicit":names(&p.c,&p.c.owner.implicit_rhythm_tuplets(1).unwrap()),"measure_tuplets":names(&p.c,&p.c.owner.measures[0].tuplets),"stack_tuplets":names(&p.c,&all),"tuplets":ts,"relations":rs,"state":frame(&mut p.c)})
}
fn normalized(mut v: Value) -> Value {
    v.as_object_mut().unwrap().remove("phase");
    v["state"]["owned"]["relations"] = json!([]);
    v
}
pub(super) fn callback(
    p: &Prepared,
    cb: &TupletRemovalObservation,
    slots: &BTreeMap<Key, Option<Arc<()>>>,
) -> Value {
    let clocks=p.c.owner.chords.iter().filter(|ch|ch.measure==0).map(|ch| {
        let n=cb.snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==ch.id).unwrap();let (_,voice,slot)=cb.owners.iter().find(|(k,_,_)|*k==ch.key).unwrap();
        let voice=voice.as_ref().map(|r|format!("voice{}",p.c.refs.iter().position(|v|Arc::ptr_eq(&v.identity,r)).unwrap()));let same=match(slot,&slots[&ch.key]){(Some(a),Some(b))=>Arc::ptr_eq(a,b),(None,None)=>true,_=>false};
        json!({"label":p.c.labels[&ch.id],"manual":n["manual"]==true,"voice":voice,"slot_id":n["slot_id"],"same_baseline_slot":same,"time_offset":n["time_offset"]})
    }).collect::<Vec<_>>();
    let mut row = cb.state.clone();
    row["edge"] = json!(p.relations.iter().find(|(id, _)| *id == cb.edge).unwrap().1);
    row["tuplet"] = names(&p.c, &[cb.tuplet])[0].clone();
    row["measure_tuplets"] = names(&p.c, &cb.measure_tuplets);
    row["stack_tuplets"] = names(&p.c, &cb.stack_tuplets);
    row["clocks"] = json!(clocks);
    row
}
#[test]
fn original_implicit_tuplets_detach_before_callbacks_then_clear_detect_and_legitimate_reset() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slot_storage = p.c.owner.measures[0].timing.slots.as_ptr();
        let staffs = p.c.owner.owner.staffs.clone();
        let report = p.c.owner.owner.prepared.report.clone();
        let slots =
            p.c.owner
                .chords
                .iter()
                .map(|ch| {
                    (
                        ch.key,
                        p.c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
        p.c.owner.owner.graph.trace_tuplet_removals();
        for ph in case["phases"].as_array().unwrap() {
            match ph["phase"].as_str().unwrap() {
                "before_remove" => {}
                "removed_implicit" => p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap(),
                "cleared" => p.c.owner.clear_interleaved_rests(1).unwrap(),
                "detected" => {
                    p.c.owner.detect_interleaved_rests(1).unwrap();
                }
                "measure_reset" => reset_rhythm(
                    &mut p.c.owner.owner.graph,
                    &mut p.c.owner.voices,
                    &mut p.c.owner.measures[0],
                    &p.c.owner.chords,
                    0,
                )
                .unwrap(),
                _ => unreachable!(),
            }
            assert_eq!(
                state(&mut p),
                normalized(ph.clone()),
                "{} {}",
                case["name"],
                ph["phase"]
            );
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            if ph["phase"] != "measure_reset" {
                assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
            }
            for (a, b) in staffs.iter().zip(&p.c.owner.owner.staffs) {
                assert!(Arc::ptr_eq(a, b));
                for (a, b) in a.lines.iter().zip(&b.lines) {
                    assert!(Arc::ptr_eq(a, b));
                }
            }
        }
        let trace = p.c.owner.owner.graph.take_tuplet_removals();
        let observations = trace
            .iter()
            .filter(|cb| cb.phase == "finished")
            .map(|cb| callback(&p, cb, &slots))
            .collect::<Vec<_>>();
        assert_eq!(json!(observations), case["callbacks"]);
        for cb in &trace {
            let row = callback(&p, cb, &slots);
            assert_eq!(row["clocks"], case["callbacks"][0]["clocks"]);
            assert_eq!(row["removed"], true);
            assert_eq!(row["registered"], true);
        }
    }
}
#[test]
fn late_detect_failure_restores_implicit_manual_tuplets_containers_and_actual_clock_owners() {
    let d = original();
    let case = &d["cases"][0];
    let mut p = control(case);
    let key = chord_key(&p.c, "rest_chord1");
    let index =
        p.c.owner
            .chords
            .iter()
            .position(|ch| ch.key == key)
            .unwrap();
    let saved = p.c.owner.chords[index].members.clone();
    p.c.owner.chords[index].members[0]["origin"]["local"] = json!(999999);
    let before = state(&mut p);
    let graph = p.c.owner.owner.graph.snapshot();
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
    let voices = projection(&p.c.owner.voices, &p.c.owner.measures);
    p.c.owner.owner.graph.trace_tuplet_removals();
    p.c.owner.owner.graph.trace_beam_rest_callbacks();
    assert_eq!(
        p.c.owner.prepare_measure_rhythm(1).unwrap_err(),
        "music_rest_actual_note_required"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(state(&mut p), before);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
    assert_eq!(projection(&p.c.owner.voices, &p.c.owner.measures), voices);
    assert!(p.c.owner.owner.graph.take_tuplet_removals().is_empty());
    assert!(p.c.owner.owner.graph.take_beam_rest_callbacks().is_empty());
    p.c.owner.chords[index].members = saved;
    p.c.owner.prepare_measure_rhythm(1).unwrap();
    assert_eq!(state(&mut p), normalized(case["phases"][3].clone()));
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
}
