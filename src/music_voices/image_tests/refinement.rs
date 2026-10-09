// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_time::{format_delta, format_fraction};

fn measure_rows(music: &OwnedMusic, system: u64) -> Vec<Value> {
    let chord_ref = |id: usize| {
        let c = music.chords.iter().find(|c| c.id == id).unwrap();
        let m = &music.measures[c.measure];
        json!({"bounds":c.bounds,"part":m.part,"measure_index":m.index})
    };
    let mut rows = Vec::new();
    for stack in music
        .measure_layout
        .stacks
        .iter()
        .filter(|s| s.stack.system == system)
    {
        let timing = stack.timing.as_ref().unwrap();
        for g in &stack.measures {
            let mi = music
                .measures
                .iter()
                .position(|m| m.geometry.as_ref().is_some_and(|held| Arc::ptr_eq(held, g)))
                .unwrap();
            let m = &music.measures[mi];
            rows.push(json!({"part":m.part,"measure_index":m.index,"abnormal":m.timing.abnormal,"voices":music.voices.iter().filter(|v|v.measure==mi).map(|v|json!({"id":v.id,"kind":v.kind.label(),"starting_staff":v.starting_staff,"termination":v.termination.map(format_delta),"excess":v.excess.map(format_fraction),"chords":v.chord_ids.iter().map(|&id|chord_ref(id)).collect::<Vec<_>>(),"slots":timing.slots.iter().filter_map(|s|v.slots.get(&s.id).map(|info|json!({"slot":s.id,"status":if info.continuing{"CONTINUE"}else{"BEGIN"},"chord":chord_ref(info.chord_id)}))).collect::<Vec<_>>()})).collect::<Vec<_>>()}));
        }
    }
    rows
}
pub(crate) fn assert_real_refinement(music: &mut OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-stack-slots/image-observed.json"
    ))
    .unwrap();
    let initial = music.owner.graph.snapshot();
    let address = music.chords.as_ptr();
    let voices = music.voices.clone();
    let slot_refs = music
        .chords
        .iter()
        .map(|c| {
            (
                c.id,
                music
                    .owner
                    .graph
                    .chord_slot_identity(c.key)
                    .unwrap()
                    .clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let clocks=music.measure_layout.stacks.iter().map(|s| {let t=s.timing.as_ref().unwrap();json!({"expected":t.expected,"actual":t.actual,"excess":t.excess,"slots":slots_projection(&t.slots)})}).collect::<Vec<_>>();
    let mut frames = Vec::new();
    for system in [1, 2] {
        for phase in ["BEFORE_SYSTEM_REFINEMENT", "AFTER_SYSTEM_REFINEMENT"] {
            if phase == "AFTER_SYSTEM_REFINEMENT" {
                refine_ordinary_system(
                    &mut music.owner.graph,
                    &mut music.voices,
                    &music.measures,
                    system,
                )
                .unwrap();
            }
            let original = source["stages"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["system"] == system && s["stage"] == phase)
                .unwrap();
            let actual = measure_rows(music, system);
            let expected = original["stacks"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|s| s["measures"].as_array().unwrap().iter().cloned())
                .collect::<Vec<_>>();
            let graph = music.owner.graph.snapshot();
            let mut chord_rows = Vec::new();
            for c in music.chords.iter().filter(|c| c.key.system == system) {
                let n = graph["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == c.id)
                    .unwrap();
                let src = original["chord_systems"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s["system"] == system)
                    .unwrap()["chords"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|ch| ch["bounds"] == json!(c.bounds))
                    .unwrap();
                let row = json!({"bounds":c.bounds,"voice":n["voice"],"slot":n["slot_id"],"time_offset":n["time_offset"]});
                let expected = json!({"bounds":src["bounds"],"voice":src["voice"],"slot":src["slot"],"time_offset":if src["time_offset"]=="null"{Value::Null}else{src["time_offset"].clone()}});
                chord_rows.push(json!({"actual":row,"expected":expected}));
            }
            frames.push(json!({"system":system,"phase":phase,"actual":actual,"expected":expected,"chords":chord_rows}));
            std::fs::write(
                "/tmp/real-voice-refinement-native-current.json",
                serde_json::to_vec_pretty(&frames).unwrap(),
            )
            .unwrap();
            assert_eq!(actual, expected, "real refinement system{system} {phase}");
            for row in chord_rows {
                assert_eq!(row["actual"], row["expected"]);
            }
        }
    }
    assert_eq!(music.chords.as_ptr(), address);
    assert_eq!(music.voices.len(), voices.len());
    for old in voices {
        let current = music
            .voices
            .iter()
            .find(|v| Arc::ptr_eq(&old.identity, &v.identity))
            .unwrap();
        assert_eq!(current.chord_ids, old.chord_ids);
        assert_eq!(current.kind, old.kind);
        assert_eq!(current.starting_staff, old.starting_staff);
        assert_eq!(
            current.termination.map(format_delta),
            old.termination.map(format_delta)
        );
        assert_eq!(current.excess, old.excess);
    }
    for c in &music.chords {
        assert!(Arc::ptr_eq(
            music.owner.graph.chord_slot_identity(c.key).unwrap(),
            &slot_refs[&c.id]
        ));
        let v = music
            .owner
            .graph
            .chord_voice_owner(c.key, &music.voices)
            .unwrap()
            .unwrap();
        assert_eq!(v.measure, c.measure);
    }
    let after = music.owner.graph.snapshot();
    assert_eq!(initial["relations"], after["relations"]);
    for old in initial["inters"].as_array().unwrap() {
        let mut old = old.clone();
        let mut now = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == old["id"])
            .unwrap()
            .clone();
        old.as_object_mut().unwrap().remove("voice");
        now.as_object_mut().unwrap().remove("voice");
        assert_eq!(old, now);
    }
    let after_clocks=music.measure_layout.stacks.iter().map(|s| {let t=s.timing.as_ref().unwrap();json!({"expected":t.expected,"actual":t.actual,"excess":t.excess,"slots":slots_projection(&t.slots)})}).collect::<Vec<_>>();
    assert_eq!(after_clocks, clocks);
    std::fs::write("/tmp/real-voice-refinement-native-observed.json",serde_json::to_vec_pretty(&json!({"frames":frames,"all4_before_after_source_frames_match":true,"actual_Chord_Voice_and_Slot_objects_retained":true,"all_clocks_termination_excess_and_relations_retained":true,"quality_accepted":false})).unwrap()).unwrap();
}
