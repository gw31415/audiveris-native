// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(crate) mod refinement;
use crate::music_measures::OwnedMusic;
fn sorted(mut v: Vec<Value>) -> Vec<Value> {
    v.sort_by_key(Value::to_string);
    v
}
pub(crate) fn assert_real_voice_families(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-family/image-observed.json"
    ))
    .unwrap();
    let mut actual = Vec::new();
    for chord in &music.chords {
        let measure = &music.measures[chord.measure];
        let (staff, kind) = starting_family(measure, chord).unwrap();
        assert!(music.owner.staffs.iter().any(|s| {
            Arc::ptr_eq(
                s,
                measure
                    .geometry
                    .as_ref()
                    .unwrap()
                    .part
                    .staffs
                    .iter()
                    .find(|s| s.id == staff)
                    .unwrap(),
            )
        }));
        let voice = construct(chord.measure, measure, &[chord], &[]).unwrap();
        assert_eq!(voice.starting_staff, staff);
        assert_eq!(voice.kind, kind);
        actual.push(json!({"system":measure.system,"part":measure.part,"measure_index":measure.index,"bounds":chord.bounds,"merged_part":false,"top_staff":staff,"same_actual_Part_Staff":true,"kind":kind.label(),"kind_ordinal":kind.id_offset()/4,"id_offset":kind.id_offset(),"next_voice_id":voice.id,"existing_voices":0}));
    }
    assert_eq!(
        sorted(actual.clone()),
        sorted(source["chords"].as_array().unwrap().clone())
    );
    let construction: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-construction/image-observed.json"
    ))
    .unwrap();
    let mut graph = music.owner.graph.clone();
    let mut voices = Vec::new();
    let mut pending: Option<Voice> = None;
    let mut old: Option<Voice> = None;
    let mut retired: Option<Voice> = None;
    let mut states = Vec::new();
    let before = graph.snapshot();
    for expected in construction["states"].as_array().unwrap() {
        let seed = music
            .chords
            .iter()
            .find(|c| {
                c.key.system == expected["system"].as_u64().unwrap()
                    && music.measures[c.measure].part == expected["part"].as_u64().unwrap()
                    && music.measures[c.measure].index
                        == expected["measure_index"].as_u64().unwrap() as usize
                    && json!(c.bounds) == expected["seed_bounds"]
            })
            .unwrap();
        let voice = match expected["phase"].as_str().unwrap() {
            "CONSTRUCTED" | "LOW_RECONSTRUCTED" | "LOW_NUMBER_REUSED" => {
                let voice = construct(
                    seed.measure,
                    &music.measures[seed.measure],
                    &[seed],
                    &voices,
                )
                .unwrap();
                let mut transient = voices.clone();
                let index = transient.len();
                transient.push(voice);
                assign_chord_voice(&mut graph, &mut transient, &music.measures, seed.id, index)
                    .unwrap();
                let voice = transient.pop().unwrap();
                voices = transient;
                if expected["phase"] == "LOW_RECONSTRUCTED" && old.is_none() {
                    old = Some(voice.clone());
                }
                if expected["phase"] == "LOW_NUMBER_REUSED" {
                    assert!(!Arc::ptr_eq(
                        &voice.identity,
                        &retired.as_ref().unwrap().identity
                    ));
                }
                pending = Some(voice.clone());
                voice
            }
            "REGISTERED" | "LOW_REGISTERED" | "LOW_REPLACEMENT_REGISTERED" => {
                let voice = pending.take().unwrap();
                voices.push(voice.clone());
                voice
            }
            "OLD_LOW_EMPTIED" => {
                let reference = old.as_ref().unwrap();
                let voice = voices
                    .iter_mut()
                    .find(|v| Arc::ptr_eq(&v.identity, &reference.identity))
                    .unwrap();
                voice.reset_chords();
                voice.clone()
            }
            "OLD_LOW_RETIRED" => {
                let reference = old.as_ref().unwrap();
                purge_measure_voices(&mut graph, &mut voices, seed.measure);
                let voice = graph
                    .retained_voice_owner(&reference.identity)
                    .unwrap()
                    .clone();
                retired = Some(voice.clone());
                voice
            }
            _ => panic!("unexpected source phase"),
        };
        let measure = &music.measures[seed.measure];
        let snapshot = graph.snapshot();
        let nodes = snapshot["inters"].as_array().unwrap();
        let mut wanted = expected.clone();
        wanted["chords"] = json!(sorted(wanted["chords"].as_array().unwrap().clone()));
        wanted["all_measure_chords"] = json!(sorted(
            wanted["all_measure_chords"].as_array().unwrap().clone()
        ));
        let row = json!({"phase":expected["phase"],"system":measure.system,"part":measure.part,"measure_index":measure.index,"seed_bounds":seed.bounds,"kind":voice.kind.label(),"id":voice.id,"starting_staff":voice.starting_staff,"same_actual_Part_Staff":true,"same_actual_Measure":voice.measure == seed.measure,"registered":voices.iter().any(|v|Arc::ptr_eq(&v.identity,&voice.identity)),"termination":voice.termination.map(crate::music_time::format_delta),"chords":sorted(voice.chord_ids.iter().map(|id|json!(music.chords.iter().find(|c|c.id == *id).unwrap().bounds)).collect()),"all_measure_chords":sorted(music.chords.iter().filter(|c|c.measure == seed.measure).map(|c|{let n=nodes.iter().find(|n|n["id"] == c.id).unwrap();json!({"bounds":c.bounds,"assigned_to_this_actual_Voice":graph.chord_has_voice(c.key,&voice),"time":n["time_offset"],"slot":n["slot_id"]})}).collect())});
        assert_eq!(row, wanted);
        states.push(row);
    }
    let after = graph.snapshot();
    assert_eq!(before["relations"], after["relations"]);
    for node in before["inters"].as_array().unwrap() {
        let mut expected = node.clone();
        let mut actual = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == node["id"])
            .unwrap()
            .clone();
        expected.as_object_mut().unwrap().remove("voice");
        actual.as_object_mut().unwrap().remove("voice");
        assert_eq!(actual, expected);
    }
    std::fs::write("/tmp/real-voice-family-native-observed.json",serde_json::to_vec_pretty(&json!({"families":actual,"constructor_states":states,"actual_Part_Staff_Arcs_retained":true,"relations_and_every_nonvoice_Inter_field_retained":true,"old_LOW_object_retained_and_reused_ID_is_new_object":true})).unwrap()).unwrap();
}
