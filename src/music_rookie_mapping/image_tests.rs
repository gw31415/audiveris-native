// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measures::OwnedMusic;
fn sorted(mut rows: Vec<Value>) -> Vec<Value> {
    rows.sort_by_key(Value::to_string);
    rows
}
pub(crate) fn assert_real_voice_distances(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-voice-distance/image-observed.json"
    ))
    .unwrap();
    let mut graph = music.owner.graph.clone();
    let mut voices = music.voices.clone();
    let mut measures = music.measures.clone();
    let mut observed = Vec::new();
    for stage in source["stages"].as_array().unwrap() {
        if stage["stage"] == "FIRST_ACTUAL_STAFF_VOICES" {
            for mi in 0..measures.len() {
                let measure = measures[mi].clone();
                let mut chords = music
                    .chords
                    .iter()
                    .filter(|c| c.measure == mi)
                    .collect::<Vec<_>>();
                chords.sort_by_key(|c| c.bounds[0]);
                for staff in &measure.geometry.as_ref().unwrap().part.staffs {
                    if let Some(seed) = chords
                        .iter()
                        .find(|c| c.members.last().unwrap()["staff_id"] == staff.id)
                    {
                        crate::music_voices::create_voice(&mut graph, &mut voices, &measures, seed)
                            .unwrap();
                    }
                }
            }
        }
        let mut actual = Vec::new();
        for mi in 0..measures.len() {
            let measure = measures[mi].clone();
            let chords = music
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            let (narrow, wide) = crate::music_measure_slots::from_owned(
                &graph,
                &measure,
                &chords,
                music.owner.interline,
            )
            .unwrap();
            let centers = chords
                .iter()
                .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
                .collect();
            let compounds =
                crate::music_measure_slots::build_compounds(&narrow.slots, &wide.slots, &centers)
                    .unwrap();
            let candidates =
                crate::music_measure_slots::CandidateReferences::new(&graph, &measure, &compounds)
                    .unwrap();
            let mut order =
                crate::music_measure_slots::CandidateOrder::new(&compounds, &candidates).unwrap();
            let mut clocks = crate::music_measure_slots::ClockState::new(&compounds).unwrap();
            let lifetime = crate::music_voice_lifetime::Lifetime::default();
            let snapshot = graph.snapshot();
            let clef = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "ClefInter" && n["staff_id"] == measure.staff)
                .unwrap();
            let context = Context {
                graph: &mut graph,
                voices: &mut voices,
                measures: &mut measures,
                chords: &chords,
                narrow: &narrow,
                measure_index: mi,
                candidates: &candidates,
                order: &mut order,
                expected: Fraction::new(3, 4).unwrap(),
                clef,
                interline: music.owner.interline,
                lifetime: &lifetime,
                slot_times: &mut clocks,
            };
            for left in &chords {
                for right in &chords {
                    let a = context.fact(&snapshot, left.id).unwrap();
                    let b = context.fact(&snapshot, right.id).unwrap();
                    let next = snapshot["relations"].as_array().unwrap().iter().any(|e| {
                        e["kind"] == "NextInVoiceRelation"
                            && e["source"] == left.id
                            && e["target"] == right.id
                    });
                    let distance = separated(&a, &b, music.owner.interline, next).unwrap();
                    actual.push(json!({"system":left.key.system,"part":snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"] == left.id).unwrap()["part_id"],"measure_index":snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"] == left.id).unwrap()["measure_index"],"left_bounds":left.bounds,"right_bounds":right.bounds,"left_staff":a.staff,"right_staff":b.staff,"distance":distance}));
                }
            }
            assert_eq!(context.graph.snapshot(), snapshot);
        }
        let expected = stage["pairs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                let mut p = p.clone();
                p.as_object_mut().unwrap().remove("details");
                p
            })
            .collect();
        std::fs::write(
            "/tmp/real-voice-distance-current.json",
            serde_json::to_vec_pretty(
                &json!({"stage":stage["stage"],"actual":actual,"expected":expected}),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(actual.len(), 613);
        assert_eq!(sorted(actual.clone()), sorted(expected));
        observed.push(json!({"stage":stage["stage"],"pairs":actual}));
    }
    std::fs::write("/tmp/real-voice-distance-native-observed.json",serde_json::to_vec_pretty(&json!({"stages":observed,"actual_Staff_clef_and_highest_member_used":true,"fact_distance_reads_leave_whole_graph_unchanged":true})).unwrap()).unwrap();
}
