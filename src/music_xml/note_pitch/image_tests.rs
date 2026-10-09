// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

pub(crate) fn assert_real_pitch(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-head-pitch/image-observed.json"
    ))
    .unwrap();
    let before = music.owner.graph.snapshot();
    let chords = music.chords.as_ptr();
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    let mut owners = Vec::new();
    for system in source["systems"].as_array().unwrap() {
        for original in system["heads"].as_array().unwrap() {
            let chord = music
                .chords
                .iter()
                .find(|c| {
                    c.key.system == system["system"].as_u64().unwrap()
                        && c.members.iter().any(|h| h["bounds"] == original["bounds"])
                })
                .unwrap();
            let head = chord
                .members
                .iter()
                .find(|h| h["bounds"] == original["bounds"])
                .unwrap();
            let key: Key = serde_json::from_value(head["origin"].clone()).unwrap();
            let mut row = music.read_actual_note_pitch(key).unwrap();
            owners.push(json!({"head":key,"chord":chord.key,"effective_clef_id":row["effective_clef_id"],"effective_key_id":row["effective_key_id"]}));
            row.as_object_mut().unwrap().remove("effective_clef_id");
            row.as_object_mut().unwrap().remove("effective_key_id");
            actual.push(json!({"system":system["system"],"head":row}));
            expected.push(json!({"system":system["system"],"head":original}));
        }
    }
    std::fs::write("/tmp/real-head-pitch-native-observed.json", serde_json::to_vec_pretty(&json!({
        "actual":actual,"expected":expected,"owners":owners,
        "actual_Chord_vector_retained":chords==music.chords.as_ptr(),
        "graph_bodies_clocks_relations_and_manual_status_unchanged":before==music.owner.graph.snapshot()
    })).unwrap()).unwrap();
    assert_eq!(actual.len(), 94);
    assert_eq!(actual, expected);
    assert_eq!(
        actual
            .iter()
            .filter(|h| !h["head"]["local_accidental"].is_null())
            .count(),
        13
    );
    assert_eq!(before, music.owner.graph.snapshot());
    assert_eq!(chords, music.chords.as_ptr());
    assert_eq!(
        music
            .read_actual_note_pitch(music.chords[0].key)
            .unwrap_err(),
        "music_pitch_actual_Head_required"
    );
}
