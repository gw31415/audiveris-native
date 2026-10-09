// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measures::OwnedMusic;
use crate::music_time::{format_delta, format_fraction};

pub(crate) fn assert_real_shared_slots(music: &mut OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-stack-slots/image-observed.json"
    ))
    .unwrap();
    let before = music.owner.graph.snapshot();
    let candidates = music
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
    let voices = music
        .voices
        .iter()
        .map(|v| v.identity.clone())
        .collect::<Vec<_>>();
    let chords_address = music.chords.as_ptr();
    let mut rows = Vec::new();
    let mut chord_rows = Vec::new();
    let mut nulls = 0;
    for si in 0..music.measure_layout.stacks.len() {
        music.gather_recognized_stack_rhythm(si + 1).unwrap();
        let owner = &music.measure_layout.stacks[si];
        let timing = owner.timing.as_ref().unwrap();
        let system = owner.stack.system;
        let stack_index = owner.stack.index;
        let stage = source["stages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["stage"] == "BEFORE_SYSTEM_REFINEMENT" && s["system"] == system)
            .unwrap();
        let expected = stage["stacks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["stack_index"] == stack_index)
            .unwrap();
        let indices = owner
            .measures
            .iter()
            .map(|g| {
                music
                    .measures
                    .iter()
                    .position(|m| m.geometry.as_ref().is_some_and(|held| Arc::ptr_eq(held, g)))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let chord_ref = |id: usize| {
            let c = music.chords.iter().find(|c| c.id == id).unwrap();
            let m = &music.measures[c.measure];
            json!({"bounds":c.bounds,"part":m.part,"measure_index":m.index})
        };
        let measure_rows=indices.iter().map(|&mi| {
            let m=&music.measures[mi];
            assert_eq!(m.timing.slots.len(),timing.slots.len());
            for (held,canonical) in m.timing.slots.iter().zip(&timing.slots) {assert!(Arc::ptr_eq(&held.identity,&canonical.identity));}
            assert_eq!(m.timing.expected,timing.expected);assert_eq!(m.timing.actual,timing.actual);assert_eq!(m.timing.excess,timing.excess);
            let voice_rows=music.voices.iter().filter(|v|v.measure==mi).map(|v| {
                let kind=match v.kind{VoiceKind::High=>"HIGH",VoiceKind::Low=>"LOW",VoiceKind::Infra=>"INFRA",VoiceKind::HighAndLow=>"HIGH_AND_LOW"};
                json!({"id":v.id,"kind":kind,"starting_staff":v.starting_staff,"termination":v.termination.map(format_delta),"excess":v.excess.map(format_fraction),"chords":v.chord_ids.iter().map(|&id|chord_ref(id)).collect::<Vec<_>>(),"slots":timing.slots.iter().filter_map(|s|v.slots.get(&s.id).map(|info|json!({"slot":s.id,"status":if info.continuing{"CONTINUE"}else{"BEGIN"},"chord":chord_ref(info.chord_id)}))).collect::<Vec<_>>()})
            }).collect::<Vec<_>>();
            json!({"part":m.part,"measure_index":m.index,"abnormal":m.timing.abnormal,"voices":voice_rows})
        }).collect::<Vec<_>>();
        let actual = json!({"stack_index":stack_index,"expected":timing.expected.map(format_fraction),"actual":timing.actual.map(format_fraction),"excess":timing.excess.map(format_fraction),"slots":timing.slots.iter().map(|s|json!({"id":s.id,"time_offset":s.time_offset.map(format_fraction),"x_offset":s.x_offset,"incoming_chords":s.incoming_ids.iter().map(|&id|chord_ref(id)).collect::<Vec<_>>()})).collect::<Vec<_>>(),"measures":measure_rows});
        rows.push(json!({"system":system,"actual":actual,"expected":expected}));
        std::fs::write(
            "/tmp/real-stack-slots-native-current.json",
            serde_json::to_vec_pretty(&json!({"stacks":rows,"chords":chord_rows})).unwrap(),
        )
        .unwrap();
        assert_eq!(
            &actual, expected,
            "actual shared Stack system{system} index{stack_index}"
        );
        let snapshot = music.owner.graph.snapshot();
        for c in music.chords.iter().filter(|c| indices.contains(&c.measure)) {
            let n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == c.id)
                .unwrap();
            let original = stage["chord_systems"]
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
            let held = music.owner.graph.chord_slot_identity(c.key).unwrap();
            let same = timing.slots.iter().any(|s| Arc::ptr_eq(&s.identity, held));
            let actual = json!({"bounds":c.bounds,"voice":n["voice"],"slot":n["slot_id"],"time_offset":n["time_offset"],"same_actual_stack_slot":same});
            let expected = json!({"bounds":original["bounds"],"voice":original["voice"],"slot":original["slot"],"time_offset":if original["time_offset"]=="null"{Value::Null}else{original["time_offset"].clone()},"same_actual_stack_slot":original["same_actual_stack_slot"]});
            if n["time_offset"].is_null() {
                nulls += 1;
                assert!(Arc::ptr_eq(held, &candidates[&c.id]));
                assert!(!same);
            } else {
                assert!(!Arc::ptr_eq(held, &candidates[&c.id]));
                assert!(same);
            }
            chord_rows.push(json!({"system":system,"actual":actual,"expected":expected}));
            std::fs::write(
                "/tmp/real-stack-slots-native-current.json",
                serde_json::to_vec_pretty(&json!({"stacks":rows,"chords":chord_rows})).unwrap(),
            )
            .unwrap();
            assert_eq!(actual, expected);
        }
    }
    assert_eq!(rows.len(), 4);
    assert_eq!(chord_rows.len(), 67);
    assert_eq!(nulls, 2);
    assert_eq!(music.chords.as_ptr(), chords_address);
    for (old, voice) in voices.iter().zip(&music.voices) {
        assert!(Arc::ptr_eq(old, &voice.identity));
    }
    let after = music.owner.graph.snapshot();
    assert_eq!(before["relations"], after["relations"]);
    for old in before["inters"].as_array().unwrap() {
        let n = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == old["id"])
            .unwrap();
        let mut old = old.clone();
        let mut n = n.clone();
        old.as_object_mut().unwrap().remove("slot_id");
        n.as_object_mut().unwrap().remove("slot_id");
        assert_eq!(old, n);
    }
    let baseline = music.owner.graph.snapshot();
    let clock = slots_projection(
        &music.measure_layout.stacks[0]
            .timing
            .as_ref()
            .unwrap()
            .slots,
    );
    assert!(music.gather_recognized_stack_rhythm(0).is_err());
    assert_eq!(music.owner.graph.snapshot(), baseline);
    assert_eq!(
        slots_projection(
            &music.measure_layout.stacks[0]
                .timing
                .as_ref()
                .unwrap()
                .slots
        ),
        clock
    );
    std::fs::write("/tmp/real-stack-slots-native-observed.json",serde_json::to_vec_pretty(&json!({"stacks":rows,"chords":chord_rows,"all4_actual_Stack_slots_and_termination_tables_match":true,"all67_actual_slot_bindings_match":true,"null_candidate_slot_objects_retained":2,"actual_Chord_Voice_and_relation_identity_retained":true,"invalid_selection_leaves_canonical_clock_and_graph_unchanged":true})).unwrap()).unwrap();
}
