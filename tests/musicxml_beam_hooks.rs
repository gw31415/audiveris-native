// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, beam_dispatch, music_xml};
use serde_json::Value;
fn linking(v: &Value) -> Option<&Value> {
    if v["schema"] == "audiveris-native.stem-image-head-linking.v1" {
        return Some(v);
    }
    if let Some(o) = v.as_object() {
        for c in o.values() {
            if let Some(r) = linking(c) {
                return Some(r);
            }
        }
    }
    if let Some(a) = v.as_array() {
        for c in a {
            if let Some(r) = linking(c) {
                return Some(r);
            }
        }
    }
    None
}
#[test]
fn hooks_from_image_keep_actual_cache_sig_and_export_original_forward_backward_sixteenths() {
    let input =
        include_bytes!("../fixtures/official-image-musicxml-beam-hooks/beam-four-hooks.png");
    let expected: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-hooks/observed.json"
    ))
    .unwrap();
    let result = music_xml::recognize(&GrayImage::decode(input).unwrap()).unwrap();
    let report = &result.owner.owner.prepared.report;
    let events = report["events"].as_array().unwrap();
    assert_eq!(events.len(), expected["events"].as_array().unwrap().len());
    for (a, b) in events.iter().zip(expected["events"].as_array().unwrap()) {
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
            assert_eq!(a[field], b[field], "{field}");
        }
        if let Some(beams) = a["beams"].as_array() {
            assert_eq!(beams, b["beams"].as_array().unwrap());
            assert_eq!(a["stem_direction"], b["stem"]);
        } else {
            assert!(b["beams"].as_array().unwrap().is_empty());
        }
    }
    let sig = &report["shared_sig"];
    let nodes = sig["inters"].as_array().unwrap();
    let edges = sig["relations"].as_array().unwrap();
    let hooks = nodes
        .iter()
        .filter(|n| n["kind"] == "BeamHookInter")
        .collect::<Vec<_>>();
    assert_eq!(hooks.len(), 4);
    assert!(!edges.iter().any(|e| e["kind"] == "BeamHeadRelation"));
    let original: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-hooks/continuation.json"
    ))
    .unwrap();
    let original_edges = original["systems"][0]["after_BeamHeadCleaner"]["relations"]
        .as_array()
        .unwrap();
    for beam in nodes
        .iter()
        .filter(|n| matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")))
    {
        let gid = beam["glyph_id"].as_u64().unwrap() as usize;
        assert_eq!(
            beam_dispatch::glyph_json(
                &result.owner.owner.prepared.registry.get(gid).unwrap().glyph
            ),
            beam["glyph"]
        );
        for bs in edges
            .iter()
            .filter(|e| e["kind"] == "BeamStemRelation" && e["source"] == beam["id"])
        {
            let stem = nodes.iter().find(|n| n["id"] == bs["target"]).unwrap();
            let owner: audiveris_native::shared_sig::Key =
                serde_json::from_value(stem["origin"].clone()).unwrap();
            assert!(result.owner.owner.caches.contains_key(&owner.system));
            assert!(
                edges
                    .iter()
                    .any(|e| e["kind"] == "ChordStemRelation" && e["target"] == stem["id"])
            );
            let original = original_edges
                .iter()
                .find(|e| {
                    e["kind"] == "BeamStemRelation"
                        && e["source_bounds"] == beam["bounds"]
                        && e["target_bounds"] == stem["bounds"]
                })
                .unwrap();
            for field in ["beam_portion", "grade", "source_ratio", "target_ratio"] {
                assert_eq!(bs[field], original[field], "{field}");
            }
        }
        if beam["kind"] == "BeamHookInter" {
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| e["kind"] == "BeamStemRelation"
                        && e["source"] == beam["id"]
                        && e["beam_portion"] != "CENTER")
                    .count(),
                1
            );
        }
    }
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-hooks/builders.json"
    ))
    .unwrap();
    let observations = linking(report).unwrap()["systems"][0]["beam_origin_builders"]
        .as_array()
        .unwrap();
    assert_eq!(observations.len(), 16);
    for obs in observations {
        let b = &obs["builder"];
        let o = oracle["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| {
                o["beam_median"] == b["beam_median"]
                    && o["horizontal"] == b["side"]
                    && o["vertical"] == b["vertical"]
                    && o["reference"] == b["reference"]
            })
            .unwrap();
        for field in ["profile", "theoretical_line", "length_map"] {
            assert_eq!(b[field], o[field], "{field}");
        }
        let items = b["items"].as_array().unwrap();
        let original = o["items"].as_array().unwrap();
        assert_eq!(items.len(), original.len());
        for (a, b) in items.iter().zip(original) {
            for field in ["kind", "contrib", "line", "glyph"] {
                assert_eq!(a[field], b[field], "{field}");
            }
        }
    }
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
