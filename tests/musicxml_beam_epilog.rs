// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, beam_dispatch, music_xml};
use serde_json::{Value, json};
fn linking(v: &Value) -> Option<&Value> {
    if v["schema"] == "audiveris-native.stem-image-head-linking.v1" {
        return Some(v);
    }
    if let Some(o) = v.as_object() {
        for c in o.values() {
            if let Some(v) = linking(c) {
                return Some(v);
            }
        }
    }
    if let Some(a) = v.as_array() {
        for c in a {
            if let Some(v) = linking(c) {
                return Some(v);
            }
        }
    }
    None
}
#[test]
fn same_glyph_competition_keeps_original_live_ensemble_and_eleven_xml_events() {
    let result = music_xml::recognize(
        &GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-beam-epilog/beam-competing-hooks.png"
        ))
        .unwrap(),
    )
    .unwrap();
    let report = &result.owner.owner.prepared.report;
    let original: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-epilog/observed.json"
    ))
    .unwrap();
    let events = report["events"].as_array().unwrap();
    assert_eq!(events.len(), 11);
    for (a, b) in events.iter().zip(original["events"].as_array().unwrap()) {
        for field in [
            "measure_number",
            "step",
            "octave",
            "alter",
            "duration_quarters",
            "offset_quarters",
            "type",
            "voice",
        ] {
            if ["duration_quarters", "offset_quarters"].contains(&field) {
                assert_eq!(a[field].as_f64(), b[field].as_f64(), "{field}");
            } else {
                assert_eq!(a[field], b[field], "{field}");
            }
        }
        if a["beams"].is_array() {
            assert_eq!(a["beams"], b["beams"]);
            assert_eq!(a["stem_direction"], b["stem"]);
        } else {
            assert!(b["beams"].as_array().unwrap().is_empty());
        }
    }
    let initial = &linking(report).unwrap()["shared_sig"];
    let initial_nodes = initial["inters"].as_array().unwrap();
    let initial_edges = initial["relations"].as_array().unwrap();
    let mut aliases = 0;
    for hook in initial_nodes
        .iter()
        .filter(|n| n["kind"] == "BeamHookInter")
    {
        let beam = initial_nodes
            .iter()
            .find(|n| n["kind"] == "BeamInter" && n["glyph_id"] == hook["glyph_id"])
            .unwrap();
        assert!(initial_edges.iter().any(|e| e["kind"] == "Exclusion"
            && ((e["source"] == beam["id"] && e["target"] == hook["id"])
                || (e["source"] == hook["id"] && e["target"] == beam["id"]))));
        aliases += 1;
    }
    assert_eq!(aliases, 4);
    let sig = &report["shared_sig"];
    let ns = sig["inters"].as_array().unwrap();
    let es = sig["relations"].as_array().unwrap();
    let groups: Vec<Value> = ns
        .iter()
        .filter(|n| n["kind"] == "BeamGroupInter")
        .map(|g| {
            let members = g["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    let n = ns
                        .iter()
                        .find(|n| n["id"] == *id)
                        .expect("no removed member in retained group");
                    assert!(es.iter().any(|e| e["kind"] == "Containment"
                        && e["source"] == g["id"]
                        && e["target"] == *id));
                    assert_eq!(
                        beam_dispatch::glyph_json(
                            &result
                                .owner
                                .owner
                                .prepared
                                .registry
                                .get(n["glyph_id"].as_u64().unwrap() as usize)
                                .unwrap()
                                .glyph
                        ),
                        n["glyph"]
                    );
                    json!({"kind":n["kind"],"bounds":n["bounds"]})
                })
                .collect::<Vec<_>>();
            json!({"bounds":g["bounds"],"members":members})
        })
        .collect();
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-epilog/groups-epilog.json"
    ))
    .unwrap();
    let expected:Vec<Value>=oracle["after_reduction_cleaner"].as_array().unwrap().iter().map(|g|json!({"bounds":g["bounds"],"members":g["members"].as_array().unwrap().iter().map(|n|json!({"kind":n["kind"],"bounds":n["bounds"]})).collect::<Vec<_>>()})).collect();
    assert_eq!(json!(groups), json!(expected));
    assert!(!es.iter().any(|e| e["kind"] == "BeamHeadRelation"));
    for flag in [
        "heads_complete",
        "stems_complete",
        "musicxml_complete",
        "epilog_complete",
        "automatic_raster_pipeline_complete",
    ] {
        assert_eq!(report[flag], false);
    }
}
