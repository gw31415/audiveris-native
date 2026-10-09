// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hooks in existing real-image targets; no additional recognition run.
use super::*;
use crate::{
    grid_ir::StaffGeometry,
    music_measures::OwnedMusic,
    shared_sig::{RestRemovalEvent, RestRemovalStage},
};
use std::{cell::RefCell, sync::Arc};

struct Held {
    rest: Key,
    staff: Arc<StaffGeometry>,
    chord: Chord,
    chord_body: Value,
    relation: Value,
}
struct Capture {
    held: Vec<Held>,
    measures: Vec<Measure>,
    events: Vec<Value>,
}
thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}
pub(crate) fn begin(music: &OwnedMusic) {
    let graph = &music.owner.graph;
    let snapshot = graph.snapshot();
    let mut held = Vec::new();
    for chord in &music.chords {
        if chord.kind != crate::music_measures::ChordKind::Rest {
            continue;
        }
        assert_eq!(chord.members.len(), 1);
        let rest: Key = serde_json::from_value(chord.members[0]["origin"].clone()).unwrap();
        let staff = graph.retained_rest_staff(rest).unwrap().clone();
        assert!(music.owner.staffs.iter().any(|s| Arc::ptr_eq(s, &staff)));
        assert!(graph.staff_rest_members(&staff).unwrap().contains(&rest));
        let relation = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| {
                r["kind"] == "Containment"
                    && r["source"] == chord.id
                    && r["target"] == graph.id(rest).unwrap()
            })
            .unwrap()
            .clone();
        held.push(Held {
            rest,
            staff,
            chord: chord.clone(),
            chord_body: graph.retained_inter(chord.key).unwrap().0.clone(),
            relation,
        });
    }
    assert!(!held.is_empty());
    CAPTURE.with(|capture| {
        assert!(capture.borrow().is_none());
        *capture.borrow_mut() = Some(Capture {
            held,
            measures: music.measures.clone(),
            events: Vec::new(),
        });
    });
}
pub(super) fn record(
    graph: &Graph,
    measures: &[Measure],
    chords: &[Chord],
    voices: &[Voice],
    event: RestRemovalEvent,
) {
    CAPTURE.with(|capture| {
        let mut capture = capture.borrow_mut();
        let Some(capture) = capture.as_mut() else { return };
        let held = capture.held.iter().find(|held| held.rest == event.inter || held.chord.key == event.inter).unwrap();
        let same_staff = Arc::ptr_eq(graph.retained_rest_staff(held.rest).unwrap(), &held.staff);
        assert!(same_staff);
        assert!(!graph.staff_rest_members(&held.staff).unwrap().contains(&held.rest));
        let chord_body = graph.retained_inter(held.chord.key).unwrap().0;
        for field in ["time_offset", "voice", "slot_id", "manual"] {
            assert_eq!(chord_body[field], held.chord_body[field], "callback {field}");
        }
        assert_eq!(measures.len(), capture.measures.len());
        for (actual, old) in measures.iter().zip(&capture.measures) {
            assert!(Arc::ptr_eq(actual.geometry.as_ref().unwrap(), old.geometry.as_ref().unwrap()));
            assert_eq!(actual.timing.expected, old.timing.expected);
            assert_eq!(actual.timing.actual, old.timing.actual);
            assert_eq!(actual.timing.excess, old.timing.excess);
        }
        let membership = chords.iter().any(|c| c.key == held.chord.key);
        let retired = graph.retained_removed_rest_chord(held.chord.key);
        if event.stage == RestRemovalStage::Staff {
            assert!(membership);
            assert!(retired.is_none());
            assert_eq!(graph.weak_rest_registration(held.rest).unwrap(), (false, true));
        } else {
            assert!(!membership);
            let retired = retired.unwrap();
            assert_eq!((retired.key, retired.id, retired.kind, retired.stem, retired.measure, retired.bounds),
                (held.chord.key, held.chord.id, held.chord.kind, held.chord.stem, held.chord.measure, held.chord.bounds));
        }
        let lifecycle = graph.weak_rest_registration(event.inter).unwrap();
        let expected = match event.stage {
            RestRemovalStage::Staff | RestRemovalStage::Measure => (false, true),
            RestRemovalStage::Edge => (true, true),
            RestRemovalStage::Vertex => (true, false),
        };
        assert_eq!(lifecycle, expected);
        let relation = event.relation_id.map(|id| {
            let (body, removed) = graph.rest_removal_relation(id).unwrap();
            assert!(removed);
            body.clone()
        });
        if let Some(relation) = &relation
            && relation["kind"] == "Containment"
        {
            assert_eq!(relation, &held.relation);
        }
        let body = graph.retained_inter(event.inter).unwrap().0;
        let bounds = if event.inter == held.chord.key { json!(held.chord.bounds) } else { body["bounds"].clone() };
        let retired = retired.map(|c| json!({"key":c.key,"id":c.id,"bounds":c.bounds,
            "measure":c.measure,"stem":c.stem,"member_ids":c.members.iter().map(|n|n["id"].clone()).collect::<Vec<_>>()}));
        capture.events.push(json!({"stage":event.stage.as_str(),"system":event.inter.system,
            "key":event.inter,"kind":body["kind"],"shape":body["shape"],"bounds":bounds,
            "raw_staff":body["staff_id"],"manual":body["manual"] == true,"removed":lifecycle.0,"registered":lifecycle.1,
            "actual_Chord_in_Measure":membership,"same_actual_Staff":same_staff,"retired_actual_Chord":retired,
            "relation":relation,"chord_raw_time_offset":chord_body["time_offset"],"chord_raw_voice":chord_body["voice"],
            "chord_raw_slot":chord_body["slot_id"],"actual_Voices":crate::music_voices::projection(voices, measures)}));
    });
}
pub(crate) fn finish(music: &OwnedMusic, page: &str) {
    let capture = CAPTURE.with(|capture| capture.borrow_mut().take().unwrap());
    assert!(!capture.events.is_empty());
    let mut final_rows = Vec::new();
    for held in &capture.held {
        let removed = music.owner.graph.retained_inter(held.rest).unwrap().1;
        assert_eq!(
            music
                .owner
                .graph
                .staff_rest_members(&held.staff)
                .unwrap()
                .contains(&held.rest),
            !removed
        );
        if removed {
            assert!(!music.chords.iter().any(|c| c.key == held.chord.key));
            let retired = music
                .owner
                .graph
                .retained_removed_rest_chord(held.chord.key)
                .unwrap();
            assert!(retired.members.is_empty());
            final_rows.push(json!({"rest":held.rest,"retired_actual_Chord":{"key":retired.key,"id":retired.id,
                "measure":retired.measure,"bounds":retired.bounds,"stem":retired.stem,"members":retired.members}}));
        }
    }
    // All native work above completes before source fixture parsing. This saved
    // observer covered Rest vertices only; no missing edge state is synthesized.
    let mut source_compared = false;
    if page == "an_laura" {
        let source: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-owned-rest-removal/an-laura-vertex-subsequence.json"
        ))
        .unwrap();
        let expected = source["events"].as_array().unwrap().iter().map(|e| {
            let n = &e["inter"];
            if n["kind"] == "RestChordInter" {
                assert!(n["raw_voice"].is_null() && n["raw_slot"].is_null() && n["raw_timeOffset"].is_null());
            }
            json!({"kind":n["kind"],"shape":n["shape"],"bounds":n["bounds"],"raw_staff":n["raw_staff"],
                "manual":n["manual"],"removed":n["removed"],"registered":n["registered"]})
        }).collect::<Vec<_>>();
        let actual = capture.events.iter().filter(|e| e["system"] == 2 && e["stage"] == "VertexRemoved")
            .map(|e| {
                assert!(e["chord_raw_time_offset"].is_null() && e["chord_raw_voice"].is_null() && e["chord_raw_slot"].is_null());
                json!({"kind":e["kind"],"shape":e["shape"],"bounds":e["bounds"],"raw_staff":e["raw_staff"],
                    "manual":e["manual"],"removed":e["removed"],"registered":e["registered"]})
            }).collect::<Vec<_>>();
        assert_eq!(expected.len(), 8);
        assert_eq!(actual, expected);
        source_compared = true;
    }
    let path = match page {
        "an_laura" => "/tmp/real-an-laura-owned-rest-removal.json",
        "der_winter" => "/tmp/real-der-winter-owned-rest-removal.json",
        _ => panic!("unsupported observation target"),
    };
    std::fs::write(path, serde_json::to_vec_pretty(&json!({"page":page,"events":capture.events,
        "retired_actual_chords":final_rows,"source_AnLaura_System2_vertex_subsequence_compared":source_compared,
        "source_callback_clock_scope":"NULL clocks only; exact 7/64 Voice/Slot underfill/excess preservation is a separate declared scalar control",
        "source_Rest_edge_callback_observation":false,"common_mixed_first_purge_admitted":false,
        "scope":"Existing bounded family reducer calls actual Staff/Measure/Voice removal; original whole common LINKS order remains pending"})).unwrap()).unwrap();
}
