// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(super) fn assert_real_chords(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-head-chords/image-observed.json"
    ))
    .unwrap();
    let mut expected = Vec::new();
    for system in source["systems"].as_array().unwrap() {
        for row in system["chords"].as_array().unwrap() {
            let mut row = row.clone();
            row["system"] = system["system"].clone();
            assert_eq!(row["same_actual_Part_Measure_object"], true);
            row.as_object_mut()
                .unwrap()
                .remove("same_actual_Part_Measure_object");
            for field in ["head_location", "tail_location"] {
                if let Some(p) = row[field].as_array() {
                    row[field] = json!(
                        p.iter()
                            .map(|v| v.as_f64().unwrap() as i64)
                            .collect::<Vec<_>>()
                    );
                }
            }
            expected.push(row);
        }
    }
    let mut actual = Vec::new();
    for chord in &music.chords {
        let (n, _) = music.owner.graph.retained_inter(chord.key).unwrap();
        let measure = &music.measures[chord.measure];
        let geometry = measure.geometry.as_ref().unwrap();
        assert!(music.measure_layout.parts.iter().any(|p| {
            std::sync::Arc::ptr_eq(&p.part, &geometry.part)
                && p.measures
                    .iter()
                    .any(|m| std::sync::Arc::ptr_eq(m, geometry))
        }));
        let members=chord.members.iter().map(|h|json!({"bounds":h["bounds"],"shape":h["shape"],"staff":h["staff_id"],"pitch":h["pitch"].as_f64().unwrap().round_ties_even() as i64})).collect::<Vec<_>>();
        let stem = chord
            .stem
            .map(|k| music.owner.graph.retained_inter(k).unwrap().0["bounds"].clone());
        actual.push(json!({"system":chord.key.system,"bounds":n["bounds"],"staff":n["staff_id"],"part":n["part_id"],"measure_index":n["measure_index"],"stack_index":geometry.stack.index,"stem_bounds":stem,"head_location":n["head_location"],"tail_location":n["tail_location"],"members":members}));
    }
    let stems = music.chords.iter().map(|c| {
        let (stem,_) = music.owner.graph.retained_inter(c.stem.unwrap()).unwrap();
        json!({"system":c.key.system,"heads":c.members.iter().map(|h|json!({"bounds":h["bounds"],"staff":h["staff_id"]})).collect::<Vec<_>>(),"bounds":stem["bounds"],"median":stem["median"],"width":stem["width"],"glyph":stem["glyph"]})
    }).collect::<Vec<_>>();
    std::fs::write(
        "/tmp/real-head-chords-native-stems.json",
        serde_json::to_vec_pretty(&stems).unwrap(),
    )
    .unwrap();
    std::fs::write(
        "/tmp/real-head-chords-native-observed.json",
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
    actual.sort_by_key(Value::to_string);
    expected.sort_by_key(Value::to_string);
    assert_eq!(actual.len(), expected.len(), "full image chord count");
    assert_eq!(
        actual, expected,
        "actual image Chord/Head/stem/Part/Measure geometry"
    );
}
