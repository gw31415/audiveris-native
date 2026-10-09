// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_time::{format_delta, format_fraction};

pub(crate) fn assert_real_page_rhythm(music: &mut OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/official-stack-slots/image-observed.json"
    ))
    .unwrap();
    let time_source: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/official-page-time-ranges/image-observed.json"
    ))
    .unwrap();
    let ranges = music.recognized_time_ranges().unwrap();
    let graph = music.owner.graph.snapshot();
    let nodes = graph["inters"].as_array().unwrap();
    let range_rows = ranges
        .iter()
        .map(|r| {
            let signature = r.signature.map(|k| {
                let n = nodes
                    .iter()
                    .find(|n| n["id"] == music.owner.graph.id(k).unwrap())
                    .unwrap();
                json!({"bounds":n["bounds"],"system":k.system,"staff":n["staff_id"]})
            });
            json!({"start":r.start,"stop":r.stop,"duration":r.duration,"signature":signature})
        })
        .collect::<Vec<_>>();
    assert_eq!(json!(range_rows), time_source["ranges"]);
    let address = music.chords.as_ptr();
    let geometry = music
        .measures
        .iter()
        .map(|m| m.geometry.clone().unwrap())
        .collect::<Vec<_>>();
    let old_voices = music.voices.clone();
    let old_slots = music
        .measure_layout
        .stacks
        .iter()
        .map(|s| s.timing.as_ref().unwrap().slots.clone())
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    let result = music.process_actual_page_rhythm(|phase, system, graph, voices, measures, stacks| {
        let snapshot = graph.snapshot();
        let nodes = snapshot["inters"].as_array().unwrap();
        let chord_ref = |id: usize| {
            let n = nodes.iter().find(|n| n["id"] == id).unwrap();
            json!({"bounds":n["bounds"],"part":n["part_id"],"measure_index":n["measure_index"]})
        };
        let actual = stacks.iter().filter(|s| s.stack.system == system).map(|s| {
            let t = s.timing.as_ref().unwrap();
            let rows = s.measures.iter().map(|g| {
                let mi = measures.iter().position(|m| m.geometry.as_ref().is_some_and(|held| Arc::ptr_eq(held,g))).unwrap();
                let m = &measures[mi];
                for (a,b) in m.timing.slots.iter().zip(&t.slots) { assert!(Arc::ptr_eq(&a.identity,&b.identity)); }
                json!({"part":m.part,"measure_index":m.index,"abnormal":m.timing.abnormal,"voices":voices.iter().filter(|v| v.measure == mi).map(|v| json!({"id":v.id,"kind":v.kind.label(),"starting_staff":v.starting_staff,"termination":v.termination.map(format_delta),"excess":v.excess.map(format_fraction),"chords":v.chord_ids.iter().map(|&id| chord_ref(id)).collect::<Vec<_>>(),"slots":t.slots.iter().filter_map(|slot| v.slots.get(&slot.id).map(|info| json!({"slot":slot.id,"status":if info.continuing{"CONTINUE"}else{"BEGIN"},"chord":chord_ref(info.chord_id)}))).collect::<Vec<_>>()})).collect::<Vec<_>>()})
            }).collect::<Vec<_>>();
            json!({"stack_index":s.stack.index,"expected":t.expected,"actual":t.actual,"excess":t.excess,"slots":t.slots.iter().map(|slot|json!({"id":slot.id,"time_offset":slot.time_offset,"x_offset":slot.x_offset,"incoming_chords":slot.incoming_ids.iter().map(|&id|chord_ref(id)).collect::<Vec<_>>()})).collect::<Vec<_>>(),"measures":rows})
        }).collect::<Vec<_>>();
        let expected = source["stages"].as_array().unwrap().iter().find(|s| s["system"] == system && s["stage"] == phase).unwrap();
        let mut chords = Vec::new();
        for n in nodes.iter().filter(|n| n["origin"]["system"] == system && n["kind"] == "HeadChordInter") {
            let old = expected["chord_systems"].as_array().unwrap().iter().find(|s| s["system"] == system).unwrap()["chords"].as_array().unwrap().iter().find(|c| c["bounds"] == n["bounds"]).unwrap();
            let key: Key = serde_json::from_value(n["origin"].clone()).unwrap();
            let held = graph.chord_slot_identity(key).unwrap();
            let same = stacks.iter().filter(|s| s.stack.system == system).flat_map(|s| &s.timing.as_ref().unwrap().slots).any(|slot| Arc::ptr_eq(&slot.identity, held));
            let row = json!({"bounds":n["bounds"],"voice":n["voice"],"slot":n["slot_id"],"time_offset":n["time_offset"],"same_actual_stack_slot":same});
            let original = json!({"bounds":old["bounds"],"voice":old["voice"],"slot":old["slot"],"time_offset":if old["time_offset"]=="null"{Value::Null}else{old["time_offset"].clone()},"same_actual_stack_slot":old["same_actual_stack_slot"]});
            chords.push(json!({"actual":row,"expected":original}));
        }
        frames.push(json!({"system":system,"phase":phase,"actual":actual,"expected":expected["stacks"],"chords":chords}));
        std::fs::write("/tmp/real-page-rhythm-native-current.json",serde_json::to_vec_pretty(&frames).unwrap()).unwrap();
        assert_eq!(json!(actual), expected["stacks"], "ordinary page system{system} {phase}");
        for row in chords { assert_eq!(row["actual"],row["expected"]); }
        Ok(())
    }).unwrap();
    assert_eq!(result["measure_checks"].as_array().unwrap().len(), 8);
    assert_eq!(
        result["measure_checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["ok"] == false)
            .count(),
        2
    );
    assert_eq!(frames.len(), 4);
    assert_eq!(music.chords.as_ptr(), address);
    for (g, m) in geometry.iter().zip(&music.measures) {
        assert!(Arc::ptr_eq(g, m.geometry.as_ref().unwrap()));
    }
    for old in old_voices {
        assert!(
            !music
                .voices
                .iter()
                .any(|v| Arc::ptr_eq(&old.identity, &v.identity))
        );
    }
    for slots in old_slots {
        for old in slots {
            assert!(
                !music
                    .measure_layout
                    .stacks
                    .iter()
                    .flat_map(|s| &s.timing.as_ref().unwrap().slots)
                    .any(|slot| Arc::ptr_eq(&old.identity, &slot.identity))
            );
        }
    }
    assert_eq!(
        graph["relations"],
        music.owner.graph.snapshot()["relations"]
    );
    let baseline = music.owner.graph.snapshot();
    let voice_refs = music.voices.clone();
    let clock_refs = music
        .measure_layout
        .stacks
        .iter()
        .map(|s| s.timing.as_ref().unwrap().slots.clone())
        .collect::<Vec<_>>();
    let original = music.measures[2].geometry.take();
    assert!(music.process_recognized_page_rhythm().is_err());
    music.measures[2].geometry = original;
    assert_eq!(music.owner.graph.snapshot(), baseline);
    for (old, current) in voice_refs.iter().zip(&music.voices) {
        assert!(Arc::ptr_eq(&old.identity, &current.identity));
    }
    for (old, stack) in clock_refs.iter().zip(&music.measure_layout.stacks) {
        for (a, b) in old.iter().zip(&stack.timing.as_ref().unwrap().slots) {
            assert!(Arc::ptr_eq(&a.identity, &b.identity));
        }
    }
    std::fs::write("/tmp/real-page-rhythm-native-observed.json",serde_json::to_vec_pretty(&json!({"time_ranges":range_rows,"frames":frames,"result":result,"all4_ordinary_page_source_frames_match":true,"legitimate_reset_replaces_old_Voice_and_canonical_Slot_objects":true,"actual_Chords_geometry_and_relations_retained":true,"missing_actual_Measure_preserves_graph_Voices_and_canonical_clocks":true,"quality_accepted":false})).unwrap()).unwrap();
}
