// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_time::{format_delta, format_fraction};

pub(crate) struct RecognitionFrame {
    pub graph: crate::shared_sig::Graph,
    pub measures: Vec<crate::music_measures::Measure>,
    pub voices: Vec<crate::music_voices::Voice>,
}
pub(crate) fn assert_real_measure_rhythm(music: &OwnedMusic) -> RecognitionFrame {
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-measure-rhythm/image-observed.json"
    ))
    .unwrap();
    assert_eq!(source["implicit_tuplets_enabled"], false);
    let mut graph = music.owner.graph.clone();
    let mut measures = music.measures.clone();
    let mut voices = music.voices.clone();
    let before = graph.snapshot();
    let mut rows = Vec::new();
    let mut mappings = Vec::new();
    let mut meter = None;
    for stage in source["stages"].as_array().unwrap() {
        let system = stage["system"].as_u64().unwrap();
        for stack_index in 1..=2 {
            let indices = measures
                .iter()
                .enumerate()
                .filter(|(_, m)| {
                    m.system == system && m.geometry.as_ref().unwrap().stack.index == stack_index
                })
                .map(|(mi, _)| mi)
                .collect::<Vec<_>>();
            let signature_values = before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| {
                    matches!(n["kind"].as_str(), Some("TimePairInter" | "TimeWholeInter"))
                        && n["origin"]["system"] == system
                        && indices.iter().any(|&mi| {
                            measures[mi].left < n["bounds"][0].as_i64().unwrap()
                                && n["bounds"][0].as_i64().unwrap() <= measures[mi].right
                        })
                })
                .map(|n| {
                    assert_eq!(n["frozen"], true);
                    Fraction::new(
                        n["numerator"].as_u64().unwrap() as u32,
                        n["denominator"].as_u64().unwrap() as u32,
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();
            if let Some(&duration) = signature_values.first() {
                assert!(signature_values.iter().all(|v| *v == duration));
                meter = Some(duration);
            }
            let expected = meter.expect("actual selected header time required");
            for mi in indices {
                crate::music_voices::reset_stack_rhythm(&mut measures[mi], Some(expected));
                let result = Context {
                    graph: &mut graph,
                    voices: &mut voices,
                    measures: &mut measures,
                    chords: &music.chords,
                    interline: music.owner.interline,
                    rest_geometry: super::super::beam_rest::RestGeometry {
                        staffs: &music.owner.staffs,
                        skew: music.owner.sheet_skew,
                        interline: music.owner.interline,
                    },
                }
                .process_recognition_measure(mi);
                if let Err(error) = &result {
                    std::fs::write("/tmp/real-measure-rhythm-native-current.json",serde_json::to_vec_pretty(&json!({"completed":rows,"system":system,"part":measures[mi].part,"measure_index":measures[mi].index,"error":error})).unwrap()).unwrap();
                }
                let (mapping, ok) = result.unwrap();
                mappings.push(json!({"system":system,"part":measures[mi].part,"measure_index":measures[mi].index,"mapping":mapping}));
                if !ok {
                    measures[mi].timing.abnormal = true;
                }
                let measure = &measures[mi];
                let snapshot = graph.snapshot();
                let nodes = snapshot["inters"].as_array().unwrap();
                let chord_ref = |id: usize| {
                    let c = music.chords.iter().find(|c| c.id == id).unwrap();
                    let m = &measures[c.measure];
                    json!({"bounds":c.bounds,"part":m.part,"measure_index":m.index})
                };
                let original = stage["stacks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|s| s["measures"].as_array().unwrap())
                    .find(|m| m["part"] == measure.part && m["measure_index"] == measure.index)
                    .unwrap();
                let original_chords = stage["chord_systems"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s["system"] == system)
                    .unwrap()["chords"]
                    .as_array()
                    .unwrap();
                let chords=music.chords.iter().filter(|c|c.measure==mi).map(|c| {
                    let n=nodes.iter().find(|n|n["id"]==c.id).unwrap();
                    json!({"bounds":c.bounds,"voice":n["voice"],"slot":n["slot_id"],"time_offset":n["time_offset"],"duration":n["duration"]})
                }).collect::<Vec<_>>();
                let mut original_chords=original_chords.iter().filter(|n|n["part"]==measure.part && n["measure_index"]==measure.index).map(|n|json!({"bounds":n["bounds"],"voice":n["voice"],"slot":n["slot"],"time_offset":if n["time_offset"]=="null"{Value::Null}else{n["time_offset"].clone()},"duration":n["duration"]})).collect::<Vec<_>>();
                let mut chords = chords;
                chords.sort_by_key(|n| n["bounds"].to_string());
                original_chords.sort_by_key(|n| n["bounds"].to_string());
                let actual_voices=voices.iter().filter(|v|v.measure==mi).map(|v|json!({"id":v.id,"kind":match v.kind{crate::music_voices::VoiceKind::High=>"HIGH",crate::music_voices::VoiceKind::Low=>"LOW",crate::music_voices::VoiceKind::Infra=>"INFRA",crate::music_voices::VoiceKind::HighAndLow=>"HIGH_AND_LOW"},"starting_staff":v.starting_staff,"termination":v.termination.map(format_delta),"excess":v.excess.map(format_fraction),"chords":v.chord_ids.iter().map(|&id|chord_ref(id)).collect::<Vec<_>>(),"slots":[]})).collect::<Vec<_>>();
                let result = stage["results"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["part"] == measure.part && r["measure_index"] == measure.index)
                    .unwrap();
                let actual = json!({"system":system,"part":measure.part,"measure_index":measure.index,"ok":ok,"abnormal":measure.timing.abnormal,"expected":measure.timing.expected.map(format_fraction),"voices":actual_voices,"chords":chords});
                let expected = json!({"system":system,"part":measure.part,"measure_index":measure.index,"ok":result["ok"],"abnormal":original["abnormal"],"expected":stage["stacks"][stack_index-1]["expected"],"voices":original["voices"],"chords":original_chords});
                rows.push(json!({"actual":actual,"expected":expected}));
                std::fs::write(
                    "/tmp/real-measure-rhythm-native-current.json",
                    serde_json::to_vec_pretty(&json!({"measures":rows,"mappings":mappings}))
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(
                    actual, expected,
                    "real MeasureRhythm system{system} Part{} Measure{}",
                    measure.part, measure.index
                );
            }
        }
    }
    assert_eq!(rows.len(), 8);
    let after = graph.snapshot();
    assert_eq!(after["relations"], before["relations"]);
    for (actual, original) in measures.iter().zip(&music.measures) {
        assert!(std::sync::Arc::ptr_eq(
            actual.geometry.as_ref().unwrap(),
            original.geometry.as_ref().unwrap()
        ));
    }
    for original in before["inters"].as_array().unwrap() {
        let n = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == original["id"])
            .unwrap();
        if original["kind"] != "HeadChordInter" {
            assert_eq!(n, original);
        } else {
            let mut actual = n.clone();
            let mut original = original.clone();
            for key in [
                "voice",
                "slot_id",
                "time_offset",
                "duration",
                "retained_chord_cache",
            ] {
                actual.as_object_mut().unwrap().remove(key);
                original.as_object_mut().unwrap().remove(key);
            }
            assert_eq!(actual, original);
        }
    }
    for c in &music.chords {
        let voice = graph.chord_voice_owner(c.key, &voices).unwrap().unwrap();
        assert!(
            voices
                .iter()
                .any(|v| std::sync::Arc::ptr_eq(&v.identity, &voice.identity))
        );
        assert_eq!(voice.measure, c.measure);
    }
    assert_eq!(music.owner.graph.snapshot(), before);
    std::fs::write("/tmp/real-measure-rhythm-native-observed.json",serde_json::to_vec_pretty(&json!({"measures":rows,"mappings":mappings,"all8_real_MeasureRhythm_results_and_states_match":true,"original_input_owner_unchanged":true})).unwrap()).unwrap();
    RecognitionFrame {
        graph,
        measures,
        voices,
    }
}
