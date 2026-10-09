// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{
    grid_ir::StaffGeometry,
    music_measure_layout::MeasureLayout,
    shared_sig::{Graph, Key},
};
use std::sync::Arc;

type ChordOwner = (usize, Key, usize, usize, Option<Key>, Vec<Value>, [i64; 4]);
pub(super) struct Control {
    source: Value,
    geometry: Arc<MeasureLayout>,
    owners: Vec<ChordOwner>,
    slots: Vec<Option<Arc<()>>>,
    snapshot: Value,
    before: Value,
}
fn sorted(mut rows: Vec<Value>) -> Vec<Value> {
    rows.sort_by_key(Value::to_string);
    rows
}
fn node(snapshot: &Value, id: usize) -> &Value {
    snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
}
fn geometry(snapshot: &Value, chord: &Chord) -> Value {
    let n = node(snapshot, chord.id);
    let notes = sorted(
        chord
            .members
            .iter()
            .map(|member| {
                let n = node(snapshot, member["id"].as_u64().unwrap() as usize);
                json!({"kind":n["kind"],"shape":n["shape"],"bounds":n["bounds"],
            "manual":n["manual"]==true,"staff":n["staff_id"]})
            })
            .collect(),
    );
    json!({"kind":n["kind"],"bounds":n["bounds"],"staff":n["staff_id"],"notes":notes})
}
fn owners(chords: &[&Chord]) -> Vec<ChordOwner> {
    chords
        .iter()
        .map(|c| {
            (
                std::ptr::from_ref(*c) as usize,
                c.key,
                c.id,
                c.measure,
                c.stem,
                c.members.clone(),
                c.bounds,
            )
        })
        .collect()
}
fn state(
    graph: &Graph,
    voices: &[Voice],
    measures: &[Measure],
    mi: usize,
    chords: &[&Chord],
) -> Value {
    let snapshot = graph.snapshot();
    let rows = sorted(
        chords
            .iter()
            .map(|c| {
                let n = node(&snapshot, c.id);
                let mut row = geometry(&snapshot, c);
                row["manual"] = json!(n["manual"] == true);
                row["voice"] = n["voice"].clone();
                row["slot"] = n["slot_id"].clone();
                row["time_offset"] = json!(
                    graph
                        .actual_chord_time_offset(c.key)
                        .unwrap()
                        .map(crate::music_time::format_fraction)
                );
                row["duration"] = json!(graph.chord_duration(c.key).unwrap());
                row
            })
            .collect(),
    );
    let voices = voices
        .iter()
        .filter(|v| v.measure == mi)
        .map(|v| {
            let refs = v
                .chord_ids
                .iter()
                .map(|id| geometry(&snapshot, chords.iter().find(|c| c.id == *id).unwrap()))
                .collect::<Vec<_>>();
            json!({"id":v.id,"kind":v.kind.label(),"starting_staff":v.starting_staff,
            "is_measure_rest":v.measure_rest_chord.is_some(),"chords":refs,
            "termination":v.termination.map(crate::music_time::format_delta),
            "excess":v.excess.map(crate::music_time::format_fraction)})
        })
        .collect::<Vec<_>>();
    json!({"expected":measures[mi].timing.expected.map(crate::music_time::format_fraction),
        "abnormal":measures[mi].timing.abnormal,"chords":rows,"voices":voices})
}
fn assert_owners(
    graph: &Graph,
    measure: &Measure,
    mi: usize,
    chords: &[&Chord],
    staffs: &[Arc<StaffGeometry>],
) {
    let layout = measure.geometry.as_ref().unwrap();
    graph.require_actual_music_measure(layout).unwrap();
    assert_eq!(layout.part.id, measure.part);
    assert_eq!(layout.part.system, measure.system);
    assert_eq!(layout.stack.system, measure.system);
    assert_eq!(layout.stack.index, measure.index);
    let snapshot = graph.snapshot();
    for c in chords {
        assert_eq!(c.measure, mi);
        assert_eq!(graph.id(c.key), Some(c.id));
        let n = node(&snapshot, c.id);
        assert_eq!(n["origin"]["system"], measure.system);
        assert_eq!(n["part_id"], measure.part);
        assert_eq!(n["measure_index"], measure.index);
        let mut live_members = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == c.id)
            .map(|e| e["target"].as_u64().unwrap())
            .collect::<Vec<_>>();
        let mut retained_members = c
            .members
            .iter()
            .map(|n| n["id"].as_u64().unwrap())
            .collect::<Vec<_>>();
        live_members.sort_unstable();
        retained_members.sort_unstable();
        assert_eq!(
            live_members, retained_members,
            "actual Containment and retained Chord members"
        );
        for member in &c.members {
            let n = node(&snapshot, member["id"].as_u64().unwrap() as usize);
            let staff = layout
                .part
                .staffs
                .iter()
                .find(|s| n["staff_id"] == s.id)
                .unwrap();
            assert!(staffs.iter().any(|s| Arc::ptr_eq(s, staff)));
        }
    }
}
pub(super) fn before(
    graph: &Graph,
    voices: &[Voice],
    measures: &[Measure],
    mi: usize,
    chords: &[&Chord],
    selected: &[&Chord],
    staffs: &[Arc<StaffGeometry>],
) -> Option<Control> {
    let measure = &measures[mi];
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-mixed-rest-starting/observed.json"
    ))
    .unwrap();
    if source["system"] != measure.system
        || source["part"] != measure.part
        || source["physical_measure_index"] != measure.index
    {
        return None;
    }
    let snapshot = graph.snapshot();
    // Geometry selects a test observation, never a production recognition path.
    if json!(
        selected
            .iter()
            .map(|c| geometry(&snapshot, c))
            .collect::<Vec<_>>()
    ) != source["starting_chords"]
    {
        return None;
    }
    assert_eq!(source["implicit_tuplets"], false);
    for (_, check) in source["source_identity_checks"].as_object().unwrap() {
        assert_eq!(check.as_bool(), Some(true));
    }
    assert_owners(graph, measure, mi, chords, staffs);
    for c in chords {
        assert!(graph.chord_voice_owner(c.key, voices).unwrap().is_none());
    }
    let before = state(graph, voices, measures, mi, chords);
    assert_eq!(
        before, source["before"],
        "actual mixed-start pre-state, including unclocked later chords"
    );
    Some(Control {
        source,
        geometry: measure.geometry.as_ref().unwrap().clone(),
        owners: owners(chords),
        slots: chords
            .iter()
            .map(|c| graph.chord_slot_identity(c.key).cloned())
            .collect(),
        snapshot,
        before,
    })
}
impl Control {
    pub(super) fn after(
        self,
        graph: &Graph,
        voices: &[Voice],
        measures: &[Measure],
        mi: usize,
        chords: &[&Chord],
        staffs: &[Arc<StaffGeometry>],
    ) {
        assert!(Arc::ptr_eq(
            &self.geometry,
            measures[mi].geometry.as_ref().unwrap()
        ));
        assert_eq!(owners(chords), self.owners);
        assert_owners(graph, &measures[mi], mi, chords, staffs);
        let after_snapshot = graph.snapshot();
        let ids = |s: &Value| {
            s["inters"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| json!([n["id"], n["origin"], n["manual"] == true]))
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&after_snapshot), ids(&self.snapshot));
        assert_eq!(after_snapshot["relations"], self.snapshot["relations"]);
        for (c, old_slot) in chords.iter().zip(&self.slots) {
            match (old_slot, graph.chord_slot_identity(c.key)) {
                (Some(a), Some(b)) => assert!(Arc::ptr_eq(a, b)),
                (None, None) => {}
                _ => panic!("starting Voice creation changed an actual Slot reference"),
            }
            if let Some(owner) = graph.chord_voice_owner(c.key, voices).unwrap() {
                assert_eq!(owner.measure, mi);
                assert!(owner.chord_ids.contains(&c.id));
                assert!(
                    voices
                        .iter()
                        .any(|v| Arc::ptr_eq(&v.identity, &owner.identity))
                );
                assert_eq!(node(&after_snapshot, c.id)["voice"], owner.id);
            }
        }
        let after = state(graph, voices, measures, mi, chords);
        let proof = json!({"schema":"audiveris-native.mixed-rest-starting-comparison.v1",
            "system":measures[mi].system,"part":measures[mi].part,"physical_measure_index":measures[mi].index,
            "source_observation_sha256":self.source["raw_observation_sha256"],
            "before":self.before,"after":after,"expected_after":self.source["after"],
            "actual_Chord_Staff_Part_Measure_Slot_and_relation_identities_retained":true,
            "actual_Voice_owners_verified":true,"source_fixture_used_as_recognition_input":false,
            "states_match":after==self.source["after"]});
        std::fs::write(
            "/tmp/real-rest-mixed-starting-native.json",
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
        assert_eq!(
            after, self.source["after"],
            "mixed ordinary start and BeamGroup propagated clock"
        );
    }
}
