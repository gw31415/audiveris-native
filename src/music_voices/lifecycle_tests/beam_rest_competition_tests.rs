// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame};
use super::beam_rest_linked_tests::chord_key;
use super::beam_rest_polygon_tests::{declare, prepare};
use super::*;
use crate::shared_sig::{CallbackObservation, Key, Phase};
use crate::sheet_skew::SheetSkew;
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-competition/observed.json"
    ))
    .unwrap();
    let frames = d["frame_sets"].as_array().unwrap().clone();
    let clocks = d["clock_sets"].as_array().unwrap().clone();
    for case in d["cases"].as_array_mut().unwrap() {
        for s in case["scenarios"].as_array_mut().unwrap() {
            for key in ["before", "after"] {
                s[key] = frames[s[key].as_u64().unwrap() as usize].clone();
            }
            for cb in s["callbacks"].as_array_mut().unwrap() {
                cb["clocks"] = clocks[cb["clocks"].as_u64().unwrap() as usize].clone();
            }
        }
    }
    d
}
fn beams(c: &Control) -> Vec<Key> {
    (0..4)
        .map(|i| {
            let id = *c
                .labels
                .iter()
                .find(|(_, s)| **s == format!("competition_beam{i}"))
                .unwrap()
                .0;
            serde_json::from_value(
                c.owner.owner.graph.snapshot()["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap()["origin"]
                    .clone(),
            )
            .unwrap()
        })
        .collect()
}
fn add_old(c: &mut Control, bs: &[Key], spec: &Value) -> Vec<(usize, String)> {
    let mut tracked = Vec::new();
    for (i, e) in spec["edges"].as_array().unwrap().iter().enumerate() {
        let note: Key = serde_json::from_value(
            c.owner
                .chords
                .iter()
                .find(|ch| ch.key == c.rests[e[1].as_u64().unwrap() as usize])
                .unwrap()
                .members[0]["origin"]
                .clone(),
        )
        .unwrap();
        let id = c
            .owner
            .owner
            .graph
            .add_owned_beam_rest(
                bs[e[0].as_u64().unwrap() as usize],
                note,
                e[2] == 1,
                c.owner.owner.sheet_skew,
                c.owner.owner.interline,
            )
            .unwrap();
        tracked.push((id, format!("old{i}")));
    }
    if spec["reinsert_first"] == true {
        let id = tracked[0].0;
        let snapshot = c.owner.owner.graph.snapshot();
        let mut edge = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .unwrap()
            .clone();
        let b = bs[spec["edges"][0][0].as_u64().unwrap() as usize];
        let ch = chord_key(c, &format!("rest_chord{}", spec["edges"][0][1]));
        let note: Key = serde_json::from_value(
            c.owner
                .chords
                .iter()
                .find(|chord| chord.key == ch)
                .unwrap()
                .members[0]["origin"]
                .clone(),
        )
        .unwrap();
        c.owner.owner.graph.remove_lookup_beam_rest(id).unwrap();
        edge["_editing_registration_ordinal"] =
            json!(c.owner.owner.graph.beam_rest_registration_ordinal(id));
        c.owner
            .owner
            .graph
            .relations(1, Phase::Symbol, vec![(b, note, edge)])
            .unwrap();
        assert!(c.owner.owner.graph.beam_rest_registration(id)["registered"] == true);
    }
    tracked
}
fn registrations(c: &Control, tracked: &[(usize, String)]) -> Value {
    json!(
        tracked
            .iter()
            .map(|(id, label)| {
                let mut r = c.owner.owner.graph.beam_rest_registration(*id);
                r["label"] = json!(label);
                r
            })
            .collect::<Vec<_>>()
    )
}
fn observed_callback(
    c: &Control,
    observation: &CallbackObservation,
    tracked: &[(usize, String)],
    slots: &BTreeMap<Key, Option<Arc<()>>>,
) -> Value {
    let (eid, label) = tracked
        .iter()
        .find(|(eid, _)| *eid == observation.edge)
        .unwrap();
    let edge = c.owner.owner.graph.snapshot()["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == *eid)
        .cloned();
    let beam = if let Some(edge) = edge {
        edge["source"].as_u64().unwrap() as usize
    } else {
        // Removed registration retains its endpoint identity even after liveness ends.
        c.owner.owner.graph.beam_rest_endpoints(*eid).unwrap().0
    };
    let clocks=c.owner.chords.iter().filter(|ch|ch.measure==0).map(|ch| {
        let n=observation.snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==ch.id).unwrap();
        let (_,voice,slot)=observation.owners.iter().find(|(k,_,_)|*k==ch.key).unwrap();
        let voice=voice.as_ref().map(|ref_|format!("voice{}",c.refs.iter().position(|v|Arc::ptr_eq(&v.identity,ref_)).unwrap()));
        let same=match(slot,&slots[&ch.key]){(Some(now),Some(old))=>Arc::ptr_eq(now,old),(None,None)=>true,_=>false};
        json!({"label":c.labels[&ch.id],"manual":n["manual"]==true,"voice":voice,"slot_id":n["slot_id"],"same_baseline_slot":same,"time_offset":n["time_offset"]})
    }).collect::<Vec<_>>();
    let mut r = observation.registration.clone();
    r["phase"] = json!(observation.phase);
    r["edge"] = json!(label);
    r["beam"] = json!(c.labels[&beam]);
    r["beam_abnormal"] = json!(
        observation.snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == beam)
            .unwrap()["abnormal"]
            == true
    );
    r["clocks"] = json!(clocks);
    r
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_competing_manual_relations_callbacks_and_partial_rejections_preserve_actual_owners() {
    for case in original()["cases"].as_array().unwrap() {
        let (mut c, group) = prepare(case);
        declare(&mut c, case, &case["geometry"]);
        let bs = beams(&c);
        for scenario in case["scenarios"].as_array().unwrap() {
            let spec = &scenario["spec"];
            declare(
                &mut c,
                case,
                if spec["geometry"].is_object() {
                    &spec["geometry"]
                } else {
                    &case["geometry"]
                },
            );
            let left = chord_key(&c, case["left"].as_str().unwrap());
            let right = chord_key(&c, case["right"].as_str().unwrap());
            let keys = c.rests.clone();
            assert_eq!(
                c.owner
                    .interleaved_rest_lookup_geometry(1, group, left, right, &keys)
                    .unwrap()["polygon"],
                spec["observed_polygon"]
            );
            c.owner.owner.sheet_skew = SheetSkew::new(spec["slope"].as_f64().unwrap()).unwrap();
            let mut tracked = add_old(&mut c, &bs, spec);
            assert_eq!(registrations(&c, &tracked), scenario["before_relations"]);
            assert_eq!(frame(&mut c), normalized(scenario["before"].clone()));
            let before = frame(&mut c);
            let storage = c.owner.chords.as_ptr();
            let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
            let report = c.owner.owner.prepared.report.clone();
            let staff_refs = c.owner.owner.staffs.clone();
            let slots = c
                .owner
                .chords
                .iter()
                .map(|ch| {
                    (
                        ch.key,
                        c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let old_relations = c.owner.owner.graph.snapshot()["relations"].clone();
            let candidates = spec["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| c.rests[i.as_u64().unwrap() as usize])
                .collect::<Vec<_>>();
            let white = spec["white"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| c.rests[i.as_u64().unwrap() as usize])
                .collect::<Vec<_>>();
            let left = chord_key(&c, case["left"].as_str().unwrap());
            let right = chord_key(&c, case["right"].as_str().unwrap());
            c.owner.owner.graph.trace_beam_rest_callbacks();
            let added = c
                .owner
                .lookup_interleaved_rests(1, group, left, right, &candidates, &white)
                .unwrap();
            tracked.extend(
                added
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (*id, format!("added{i}"))),
            );
            assert_eq!(
                registrations(&c, &tracked),
                scenario["after_relations"],
                "{} {}",
                case["name"],
                spec["name"]
            );
            let live = c.owner.owner.graph.snapshot()["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|e| {
                    tracked
                        .iter()
                        .find(|(id, _)| e["id"] == *id)
                        .map(|(_, label)| label.clone())
                })
                .collect::<Vec<_>>();
            assert_eq!(json!(live), scenario["live_order"]);
            let callbacks = c.owner.owner.graph.take_beam_rest_callbacks();
            let mut observed = Vec::new();
            for observation in &callbacks {
                let row = observed_callback(&c, observation, &tracked, &slots);
                assert_eq!(
                    row["clocks"], scenario["callbacks"][0]["clocks"],
                    "every callback entry/exit keeps original clocks"
                );
                assert!(
                    row["clocks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|r| r["same_baseline_slot"] == true)
                );
                if observation.phase == "entered" {
                    assert_eq!(row["registered"], true);
                    assert_eq!(row["manual"], false);
                    assert_eq!(row["dy"], 0);
                    assert_eq!(row["portion"], Value::Null);
                } else if observation.phase != "remove_entered"
                    || row["edge"].as_str().unwrap().starts_with("old")
                {
                    observed.push(row);
                }
            }
            assert_eq!(
                json!(observed),
                scenario["callbacks"],
                "{} {} callbacks",
                case["name"],
                spec["name"]
            );
            assert_eq!(frame(&mut c), normalized(scenario["after"].clone()));
            assert_eq!(frame(&mut c), before);
            assert_eq!(c.owner.chords.as_ptr(), storage);
            assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
            assert_eq!(c.owner.owner.prepared.report, report);
            for (old, now) in staff_refs.iter().zip(&c.owner.owner.staffs) {
                assert!(Arc::ptr_eq(old, now));
                for (old, now) in old.lines.iter().zip(&now.lines) {
                    assert!(Arc::ptr_eq(old, now));
                }
            }
            for (key, old) in &slots {
                match (old, c.owner.owner.graph.chord_slot_identity(*key)) {
                    (Some(old), Some(now)) => assert!(Arc::ptr_eq(old, now)),
                    (None, None) => {}
                    _ => panic!("lookup changed Slot"),
                }
            }
            for e in old_relations
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| !tracked.iter().any(|(id, _)| e["id"] == *id))
            {
                assert!(
                    c.owner.owner.graph.snapshot()["relations"]
                        .as_array()
                        .unwrap()
                        .contains(e)
                );
            }
            for (key, old) in c.rests.iter().zip(&c.baseline) {
                assert!(Arc::ptr_eq(
                    &c.owner.owner.graph.rest_chord_beams(*key).unwrap(),
                    old
                ));
            }
            for (id, _) in &tracked {
                if c.owner.owner.graph.beam_rest_registration(*id)["registered"] == true {
                    c.owner.owner.graph.remove_lookup_beam_rest(*id).unwrap();
                }
            }
        }
    }
}
#[test]
fn later_member_failure_after_staged_removal_and_addition_restores_manual_identity_clocks_and_unused_registration()
 {
    let d = original();
    let case = &d["cases"][0];
    let (mut c, group) = prepare(case);
    declare(&mut c, case, &case["geometry"]);
    let bs = beams(&c);
    let s = case["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["spec"]["name"] == "manual_loser")
        .unwrap();
    let tracked = add_old(&mut c, &bs, &s["spec"]);
    let index = c
        .owner
        .chords
        .iter()
        .position(|ch| ch.key == c.rests[1])
        .unwrap();
    let saved = c.owner.chords[index].members.clone();
    c.owner.chords[index].members[0]["origin"]["local"] = json!(999999);
    let left = chord_key(&c, case["left"].as_str().unwrap());
    let right = chord_key(&c, case["right"].as_str().unwrap());
    let keys = [c.rests[0], c.rests[1]];
    c.owner.owner.graph.trace_beam_rest_callbacks();
    let snapshot = c.owner.owner.graph.snapshot();
    let max_id = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_u64().unwrap() as usize)
        .max()
        .unwrap();
    let physical = c.owner.chords.as_ptr();
    let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
    let voices = projection(&c.owner.voices, &c.owner.measures);
    let layout = c.owner.layout();
    let old = registrations(&c, &tracked);
    let before = frame(&mut c);
    let report = c.owner.owner.prepared.report.clone();
    let staffs = c.owner.owner.staffs.clone();
    assert_eq!(
        c.owner
            .lookup_interleaved_rests(1, group, left, right, &keys, &[])
            .unwrap_err(),
        "music_rest_actual_note_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), snapshot);
    assert!(c.owner.owner.graph.take_beam_rest_callbacks().is_empty());
    assert_eq!(registrations(&c, &tracked), old);
    assert_eq!(frame(&mut c), before);
    assert_eq!(c.owner.chords.as_ptr(), physical);
    assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), voices);
    assert_eq!(c.owner.layout(), layout);
    assert_eq!(c.owner.owner.prepared.report, report);
    for (a, b) in staffs.iter().zip(&c.owner.owner.staffs) {
        assert!(Arc::ptr_eq(a, b));
    }
    c.owner.chords[index].members = saved;
    let added = c
        .owner
        .lookup_interleaved_rests(1, group, left, right, &keys, &[])
        .unwrap();
    assert_eq!(added, vec![max_id + 1, max_id + 2]);
    assert_eq!(
        c.owner.owner.graph.beam_rest_registration(tracked[0].0)["registered"],
        false
    );
    assert_eq!(
        c.owner.owner.graph.beam_rest_registration(tracked[0].0)["manual"],
        true
    );
    assert_eq!(
        c.owner.owner.graph.beam_rest_registration(added[0])["registered"],
        false
    );
    assert_eq!(
        c.owner.owner.graph.beam_rest_registration(added[1])["dy"],
        28
    );
    assert_eq!(frame(&mut c), before);
}
