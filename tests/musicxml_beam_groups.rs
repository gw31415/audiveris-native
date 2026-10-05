// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, beam_dispatch, music_xml};
use serde_json::Value;

fn linking(v: &Value) -> Option<&Value> {
    if v["schema"] == "audiveris-native.stem-image-head-linking.v1" {
        return Some(v);
    }
    if let Some(o) = v.as_object() {
        for child in o.values() {
            if let Some(v) = linking(child) {
                return Some(v);
            }
        }
    }
    if let Some(a) = v.as_array() {
        for child in a {
            if let Some(v) = linking(child) {
                return Some(v);
            }
        }
    }
    None
}

#[test]
fn separate_image_beam_groups_keep_owned_stems_and_independent_musicxml_begin_end() {
    let input =
        include_bytes!("../fixtures/official-image-musicxml-beam-groups/beam-two-groups.png");
    let o: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-groups/observed.json"
    ))
    .unwrap();
    let result = music_xml::recognize(&GrayImage::decode(input).unwrap()).unwrap();
    let report = &result.owner.owner.prepared.report;
    let events = report["events"].as_array().unwrap();
    assert_eq!(events.len(), o["events"].as_array().unwrap().len());
    for (e, c) in events.iter().zip(o["events"].as_array().unwrap()) {
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
            assert_eq!(e[field], c[field], "{field}");
        }
        if let Some(beams) = e["beams"].as_array() {
            assert_eq!(beams.len(), c["beams"].as_array().unwrap().len());
            for (a, b) in beams.iter().zip(c["beams"].as_array().unwrap()) {
                for field in ["number", "value"] {
                    assert_eq!(a[field], b[field]);
                }
            }
            assert_eq!(e["stem_direction"], c["stem"]);
        } else {
            assert!(c["beams"].as_array().unwrap().is_empty());
        }
    }
    let sig = &report["shared_sig"];
    let nodes = sig["inters"].as_array().unwrap();
    let edges = sig["relations"].as_array().unwrap();
    let oracle_builders: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-beam-groups/builders.json"
    ))
    .unwrap();
    let actual = linking(report).unwrap()["systems"][0]["beam_origin_builders"]
        .as_array()
        .unwrap();
    assert_eq!(
        actual.len(),
        oracle_builders["cases"].as_array().unwrap().len()
    );
    for c in actual {
        let b = &c["builder"];
        let expected = oracle_builders["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["beam_median"] == b["beam_median"] && e["horizontal"] == b["side"])
            .unwrap();
        for field in ["theoretical_line", "length_map"] {
            assert_eq!(b[field], expected[field]);
        }
        let items = b["items"].as_array().unwrap();
        let original = expected["items"].as_array().unwrap();
        assert_eq!(items.len(), original.len());
        for (a, b) in items.iter().zip(original) {
            for field in ["kind", "contrib", "line", "glyph"] {
                assert_eq!(a[field], b[field]);
            }
        }
        assert_eq!(c["stem"]["glyph"], expected["stem"]["glyph"]);
        // These separated groups do not intersect the theoretical lines.
        // Positive shrinking and its boundaries are the separate original
        // numeric controls, not an end-to-end positive-alien claim.
        assert!(b["closer_beam_id"].is_null());
        assert!(b["closer_limit"].is_null());
    }
    assert_eq!(
        nodes
            .iter()
            .filter(|n| n["kind"] == "BeamGroupInter")
            .count(),
        2
    );
    let beams = nodes
        .iter()
        .filter(|n| n["kind"] == "BeamInter")
        .collect::<Vec<_>>();
    assert_eq!(beams.len(), 2);
    assert!(!edges.iter().any(|e| e["kind"] == "BeamHeadRelation"));
    for beam in beams {
        assert_eq!(
            beam_dispatch::glyph_json(
                &result
                    .owner
                    .owner
                    .prepared
                    .registry
                    .get(beam["glyph_id"].as_u64().unwrap() as usize)
                    .unwrap()
                    .glyph
            ),
            beam["glyph"]
        );
        let endpoints = edges
            .iter()
            .filter(|e| e["kind"] == "BeamStemRelation" && e["source"] == beam["id"])
            .collect::<Vec<_>>();
        assert_eq!(endpoints.len(), 2);
        for side in ["LEFT", "RIGHT"] {
            let e = endpoints
                .iter()
                .find(|e| e["beam_portion"] == side)
                .unwrap();
            let stem = nodes
                .iter()
                .find(|n| n["id"] == e["target"] && n["kind"] == "StemInter")
                .unwrap();
            assert!(
                edges
                    .iter()
                    .any(|e| e["kind"] == "HeadStemRelation" && e["target"] == stem["id"])
            );
        }
    }
    assert!(result.xml.contains("<divisions>2</divisions>"));
    for value in ["begin", "end"] {
        assert_eq!(
            result
                .xml
                .matches(&format!("<beam number=\"1\">{value}</beam>"))
                .count(),
            2
        );
    }
    assert!(!result.xml.contains(">continue</beam>"));
    for k in [
        "heads_complete",
        "stems_complete",
        "musicxml_complete",
        "epilog_complete",
        "automatic_raster_pipeline_complete",
    ] {
        assert_eq!(report[k], false);
    }
}
