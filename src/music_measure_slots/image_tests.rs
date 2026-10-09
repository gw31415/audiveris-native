// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measures::OwnedMusic;
use serde_json::json;
pub(crate) fn assert_real_measure_slots(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-measure-slots/image-observed.json"
    ))
    .unwrap();
    let before = music.owner.graph.snapshot();
    let mut rows = Vec::new();
    for original in source["measures"].as_array().unwrap() {
        let (mi, measure) = music
            .measures
            .iter()
            .enumerate()
            .find(|(_, m)| {
                m.system == original["system"].as_u64().unwrap()
                    && m.part == original["part"].as_u64().unwrap()
                    && m.index == original["measure_index"].as_u64().unwrap() as usize
            })
            .unwrap();
        let chords = music
            .chords
            .iter()
            .filter(|c| c.measure == mi)
            .collect::<Vec<_>>();
        let (narrow, wide) =
            from_owned(&music.owner.graph, measure, &chords, music.owner.interline).unwrap();
        let centers = chords
            .iter()
            .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
            .collect();
        let compounds = build_compounds(&narrow.slots, &wide.slots, &centers).unwrap();
        let chord_bounds = |id: &usize| json!(chords.iter().find(|c| c.id == *id).unwrap().bounds);
        let slots = |retrieval: &Retrieval| {
            retrieval.slots.iter().map(|s|json!({"id":s.id,"chord_bounds":s.chord_ids.iter().map(chord_bounds).collect::<Vec<_>>()})).collect::<Vec<_>>()
        };
        let row = json!({"system":measure.system,"part":measure.part,"measure_index":measure.index,"narrow":slots(&narrow),"wide":slots(&wide),"compounds":compounds.iter().map(|c|json!({"id":c.id,"narrow_slot_ids":c.members.iter().map(|s|s.id).collect::<Vec<_>>(),"chord_bounds":c.chord_ids.iter().map(chord_bounds).collect::<Vec<_>>()})).collect::<Vec<_>>()});
        let mut expected = original.clone();
        expected.as_object_mut().unwrap().remove("offsets");
        rows.push(json!({"actual":row,"expected":expected}));
        std::fs::write(
            "/tmp/real-measure-slots-native-current.json",
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
        assert_eq!(row, expected);
        for offset in original["offsets"].as_array().unwrap() {
            let c = chords
                .iter()
                .find(|c| json!(c.bounds) == offset["bounds"])
                .unwrap();
            let p = [
                (c.bounds[0] + c.bounds[2] / 2) as f64,
                (c.bounds[1] + c.bounds[3] / 2) as f64,
            ];
            let x = music
                .owner
                .graph
                .actual_music_stack_x_offset(measure.geometry.as_ref().unwrap(), p)
                .unwrap();
            assert!(
                (x - offset["x_offset"].as_f64().unwrap()).abs() < 1e-9,
                "actual Stack x-offset {:?}: {x} vs {}",
                c.bounds,
                offset["x_offset"]
            );
        }
    }
    assert_eq!(music.owner.graph.snapshot(), before);
    std::fs::write("/tmp/real-measure-slots-native-observed.json",serde_json::to_vec_pretty(&json!({"measures":rows,"actual_Stack_offsets_match_for_all67_Chords":true,"narrow_wide_compound_members_and_order_match_all8_Measures":true,"whole_graph_retained":true})).unwrap()).unwrap();
}
