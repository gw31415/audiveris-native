// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measures::{Chord, ChordKind};
use crate::shared_sig::{Key, Phase};

fn original() -> Value {
    let mut observed: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-clear/observed.json"
    ))
    .unwrap();
    let relations = observed["relation_sets"].as_array().unwrap().clone();
    for case in observed["cases"].as_array_mut().unwrap() {
        for step in case["states"].as_array_mut().unwrap() {
            let index = step["state"]["owned"]["relations"].as_u64().unwrap() as usize;
            step["state"]["owned"]["relations"] = relations[index].clone();
        }
    }
    observed
}
pub(super) struct Control {
    pub(super) owner: OwnedMusic,
    pub(super) labels: BTreeMap<usize, String>,
    pub(super) refs: Vec<Voice>,
    pub(super) rests: Vec<Key>,
    pub(super) baseline: Vec<Arc<Vec<usize>>>,
    pub(super) beam: usize,
    pub(super) relations: Vec<usize>,
}
pub(super) fn prepare_control(case: &Value) -> Control {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-parallel-voices/parallel-beam.png"
    ))
    .unwrap();
    let mut owner = crate::music_xml::recognize(&image).unwrap().owner;
    let mut heads = owner
        .chords
        .iter()
        .filter(|c| c.measure == 0)
        .collect::<Vec<_>>();
    heads.sort_by_key(|c| (c.bounds[0], c.bounds[1]));
    let mut labels = BTreeMap::new();
    for (i, ch) in heads.iter().enumerate() {
        labels.insert(ch.id, format!("chord{i}"));
        for (j, head) in ch.members.iter().enumerate() {
            labels.insert(
                head["id"].as_u64().unwrap() as usize,
                format!("head{i}_{j}"),
            );
        }
    }
    let vi = owner
        .voices
        .iter()
        .position(|v| v.measure == 0 && v.id == 1)
        .unwrap();
    let seed = *owner.voices[vi].chord_ids.first().unwrap();
    let last = *owner.voices[vi].chord_ids.last().unwrap();
    for (id, field, value) in [
        (seed, "manual", json!(true)),
        (last, "time_offset", case["declared_last_offset"].clone()),
    ] {
        let snapshot = owner.owner.graph.snapshot();
        let mut node = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap()
            .clone();
        node[field] = value;
        let key = serde_json::from_value(node["origin"].clone()).unwrap();
        owner.owner.graph.vertex(key, node, false).unwrap();
    }
    check_duration(
        &owner.owner.graph,
        &mut owner.voices[vi],
        &mut owner.measures[0],
    )
    .unwrap();
    let refs = owner
        .voices
        .iter()
        .filter(|v| v.measure == 0)
        .cloned()
        .collect::<Vec<_>>();
    let snapshot = owner.owner.graph.snapshot();
    let seed_key = owner.chords.iter().find(|c| c.id == seed).unwrap().key;
    let beam = owner.owner.graph.actual_chord_beams(seed_key).unwrap()[0];
    let group = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "Containment" && e["target"] == beam)
        .unwrap()["source"]
        .as_u64()
        .unwrap() as usize;
    labels.insert(beam, "source_beam".into());
    labels.insert(group, "source_group".into());
    let beam_key: Key = serde_json::from_value(
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == beam)
            .unwrap()["origin"]
            .clone(),
    )
    .unwrap();
    let local = |phase| {
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| serde_json::from_value::<Key>(n["origin"].clone()).unwrap())
            .filter(|k| k.system == 1 && k.phase == phase)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            + 1
    };
    let mut rests = Vec::new();
    let mut relations = Vec::new();
    for (i, declared) in case["declared_rests"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mi = if declared["source_measure"] == true {
            0
        } else {
            1
        };
        let rest = Key {
            system: 1,
            phase: Phase::Symbol,
            local: local(Phase::Symbol) + i,
        };
        let chord = Key {
            system: 1,
            phase: Phase::Chord,
            local: local(Phase::Chord) + i,
        };
        let bounds: [i64; 4] = serde_json::from_value(declared["bounds"].clone()).unwrap();
        let id = owner.owner.graph.vertex(rest,json!({"kind":"RestInter","shape":"QUARTER_REST","bounds":bounds,"grade":0.8,"staff_id":1}),false).unwrap();
        let cid = owner.owner.graph.vertex(chord,json!({"kind":"RestChordInter","staff_id":1,"part_id":1,"measure_index":owner.measures[mi].index,"bounds":bounds,"voice":null,"member_ids":[id]}),false).unwrap();
        owner
            .owner
            .graph
            .relations(
                1,
                Phase::Chord,
                vec![(chord, rest, json!({"kind":"Containment"}))],
            )
            .unwrap();
        let snapshot = owner.owner.graph.snapshot();
        let member = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap()
            .clone();
        owner.chords.push(Chord {
            kind: ChordKind::Rest,
            key: chord,
            id: cid,
            stem: None,
            members: vec![member],
            measure: mi,
            bounds,
        });
        labels.insert(id, format!("rest{i}"));
        labels.insert(cid, format!("rest_chord{i}"));
        rests.push(chord);
        if mi == 0 && declared["beam_link"] != false {
            owner.owner.graph.relations(1,Phase::Symbol,vec![(beam_key,rest,json!({"kind":"BeamRestRelation","manual":declared["manual"],"beam_portion":"CENTER"}))]).unwrap();
            let edge = owner.owner.graph.snapshot()["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["kind"] == "BeamRestRelation" && e["target"] == id)
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize;
            relations.push(edge);
        }
    }
    let baseline = rests
        .iter()
        .map(|r| owner.owner.graph.rest_chord_beams(*r).unwrap())
        .collect();
    Control {
        owner,
        labels,
        refs,
        rests,
        baseline,
        beam,
        relations,
    }
}
pub(super) fn frame(c: &mut Control) -> Value {
    let groups = c.owner.actual_measure_beam_groups(1).unwrap();
    let snapshot = c.owner.owner.graph.snapshot();
    let names = (0..c.refs.len())
        .map(|i| format!("voice{i}"))
        .collect::<Vec<_>>();
    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
    let mut owned =
        super::retired_beam_tests::measure_state(&c.owner, &c.labels, &c.refs, &names, 0);
    let order = c
        .owner
        .chords
        .iter()
        .filter(|ch| ch.measure == 0)
        .map(|ch| c.labels[&ch.id].clone())
        .collect::<Vec<_>>();
    owned["chords"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|ch| order.iter().position(|label| json!(label) == ch["label"]));
    owned["measure_chords"] = json!(
        c.owner
            .chords
            .iter()
            .filter(|ch| ch.measure == 0 && ch.kind == ChordKind::Head)
            .map(|ch| c.labels[&ch.id].clone())
            .collect::<Vec<_>>()
    );
    owned["relations"] = json!(
        c.relations
            .iter()
            .enumerate()
            .map(|(i, id)| json!({"label":format!("beam_rest{i}"),
        "registered":snapshot["relations"].as_array().unwrap().iter().any(|e|e["id"]==*id),
        "manual":original_manual(c,i)}))
            .collect::<Vec<_>>()
    );
    let rows = c.rests.iter().enumerate().map(|(i,key)| {
        let beams = c.owner.owner.graph.rest_chord_beams(*key).unwrap();
        let chord = c.owner.chords.iter().find(|ch|ch.key==*key).unwrap();
        let group = beams.first().and_then(|beam|snapshot["relations"].as_array().unwrap().iter().find(|e|e["kind"]=="Containment"&&e["target"]==*beam)).map(|e|c.labels[&(e["source"].as_u64().unwrap() as usize)].clone());
        json!({"label":c.labels[&chord.id],"beams":beams.iter().map(|id|c.labels[id].clone()).collect::<Vec<_>>(),
            "same_baseline_beams":Arc::ptr_eq(&beams,&c.baseline[i]),"group":group,"same_measure":chord.measure==0,
            "voice":c.owner.owner.graph.chord_voice_owner(*key,&c.owner.voices).unwrap().map(|v|v.id),
            "members":chord.members.iter().map(|n|c.labels[&(n["id"].as_u64().unwrap() as usize)].clone()).collect::<Vec<_>>()})
    }).collect::<Vec<_>>();
    json!({"owned":owned,"rests":rows,"beams":[{"label":"source_beam","abnormal":snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==c.beam).unwrap()["abnormal"]==true}],
        "measure_groups":groups.iter().map(|id|c.labels[id].clone()).collect::<Vec<_>>(),"measure_rests":c.owner.chords.iter().filter(|ch|ch.measure==0&&ch.kind==ChordKind::Rest).map(|ch|c.labels[&ch.id].clone()).collect::<Vec<_>>()})
}
fn original_manual(c: &Control, i: usize) -> bool {
    c.owner
        .owner
        .graph
        .edge_manual_status(c.relations[i])
        .unwrap()
}

#[test]
fn original_beam_rest_clear_preserves_manual_edges_cached_beams_and_real_clock_owners() {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        let chord_storage = c.owner.chords.as_ptr();
        let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
        let slot_refs = c.owner.measures[0]
            .timing
            .slots
            .iter()
            .map(|s| s.identity.clone())
            .collect::<Vec<_>>();
        let chord_slots = c
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
        let mut untouched = c.owner.owner.graph.snapshot()["relations"].clone();
        untouched
            .as_array_mut()
            .unwrap()
            .retain(|e| !c.relations.iter().any(|id| e["id"] == *id));
        for step in case["states"].as_array().unwrap() {
            match step["phase"].as_str().unwrap() {
                "before_clear" => {}
                "cleared" | "cleared_again" => c.owner.clear_interleaved_rests(1).unwrap(),
                "measure_reset" => reset_rhythm(
                    &mut c.owner.owner.graph,
                    &mut c.owner.voices,
                    &mut c.owner.measures[0],
                    &c.owner.chords,
                    0,
                )
                .unwrap(),
                "rest_invalidate" => {
                    for key in c.rests.iter().take(2) {
                        c.owner
                            .owner
                            .graph
                            .invalidate_owned_rest_cache(*key)
                            .unwrap();
                    }
                }
                _ => unreachable!(),
            }
            let mut expected = step["state"].clone();
            let original_edges = expected["owned"]["relations"].as_array_mut().unwrap();
            for row in original_edges
                .iter()
                .filter(|e| !e["label"].as_str().unwrap().starts_with("beam_rest"))
            {
                assert_eq!(
                    row,
                    case["states"][0]["state"]["owned"]["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["label"] == row["label"])
                        .unwrap()
                );
            }
            original_edges.retain(|e| e["label"].as_str().unwrap().starts_with("beam_rest"));
            assert_eq!(
                frame(&mut c),
                expected,
                "{} {}",
                case["name"],
                step["phase"]
            );
            let mut edges = c.owner.owner.graph.snapshot()["relations"].clone();
            edges
                .as_array_mut()
                .unwrap()
                .retain(|e| !c.relations.iter().any(|id| e["id"] == *id));
            assert_eq!(edges, untouched);
            assert_eq!(c.owner.chords.as_ptr(), chord_storage);
            assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
            for (slot, reference) in c.owner.measures[0].timing.slots.iter().zip(&slot_refs) {
                assert!(Arc::ptr_eq(&slot.identity, reference));
            }
            for chord in &c.owner.chords {
                let actual = c.owner.owner.graph.chord_slot_identity(chord.key);
                if chord.measure == 0
                    && matches!(
                        step["phase"].as_str(),
                        Some("measure_reset" | "rest_invalidate")
                    )
                {
                    assert!(actual.is_none());
                } else {
                    match (actual, &chord_slots[&chord.key]) {
                        (Some(actual), Some(old)) => assert!(Arc::ptr_eq(actual, old)),
                        (None, None) => {}
                        _ => panic!("actual Chord Slot changed outside reset"),
                    }
                }
            }
        }
    }
}

#[test]
fn beam_rest_clear_late_typed_endpoint_failure_keeps_actual_chords_slots_and_voice_clocks() {
    let observed = original();
    let case = &observed["cases"][0];
    let mut c = prepare_control(case);
    let state = frame(&mut c);
    let rest_id = c
        .owner
        .chords
        .iter()
        .find(|ch| ch.key == c.rests[1])
        .unwrap()
        .members[0]["id"]
        .as_u64()
        .unwrap() as usize;
    let snapshot = c.owner.owner.graph.snapshot();
    let mut node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == rest_id)
        .unwrap()
        .clone();
    let key = serde_json::from_value(node["origin"].clone()).unwrap();
    let repaired = node.clone();
    node["kind"] = json!("HeadInter");
    c.owner.owner.graph.vertex(key, node, false).unwrap();
    let graph = c.owner.owner.graph.snapshot();
    let report = c.owner.owner.prepared.report.clone();
    let layout = c.owner.layout();
    let chords = c.owner.chords.clone();
    let chord_storage = c.owner.chords.as_ptr();
    let slot_storage = c
        .owner
        .measures
        .iter()
        .map(|m| m.timing.slots.as_ptr())
        .collect::<Vec<_>>();
    let slot_refs = c
        .owner
        .measures
        .iter()
        .map(|m| {
            m.timing
                .slots
                .iter()
                .map(|s| s.identity.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let voices = c.owner.voices.clone();
    assert_eq!(
        c.owner.clear_interleaved_rests(1).unwrap_err(),
        "music_rest_automatic_typed_relation_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), graph);
    assert_eq!(c.owner.owner.prepared.report, report);
    assert_eq!(c.owner.layout(), layout);
    assert_eq!(c.owner.chords.as_ptr(), chord_storage);
    for (ch, old) in c.owner.chords.iter().zip(chords) {
        assert_eq!(ch.members, old.members);
        assert_eq!(ch.stem, old.stem);
        assert_eq!(ch.bounds, old.bounds);
        assert_eq!(ch.key, old.key);
        assert_eq!(ch.measure, old.measure);
    }
    for (mi, m) in c.owner.measures.iter().enumerate() {
        assert_eq!(m.timing.slots.as_ptr(), slot_storage[mi]);
        for (s, r) in m.timing.slots.iter().zip(&slot_refs[mi]) {
            assert!(Arc::ptr_eq(&s.identity, r));
        }
    }
    for (v, old) in c.owner.voices.iter().zip(&voices) {
        assert!(Arc::ptr_eq(&v.identity, &old.identity));
    }
    for (key, old) in c.rests.iter().zip(&c.baseline) {
        assert!(Arc::ptr_eq(
            &c.owner.owner.graph.rest_chord_beams(*key).unwrap(),
            old
        ));
    }
    for mi in [0, 3] {
        assert_eq!(
            c.owner.clear_interleaved_rests(mi).unwrap_err(),
            "music_rest_actual_measure_required"
        );
        assert_eq!(c.owner.owner.graph.snapshot(), graph);
    }
    c.owner.owner.graph.vertex(key, repaired, false).unwrap();
    assert_eq!(frame(&mut c), state);
    c.owner.clear_interleaved_rests(1).unwrap();
    for (i, id) in c.relations.iter().enumerate() {
        assert!(
            !c.owner.owner.graph.snapshot()["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["id"] == *id)
        );
        assert_eq!(
            c.owner.owner.graph.edge_manual_status(*id).unwrap(),
            case["declared_rests"][i]["manual"]
        );
    }
}
