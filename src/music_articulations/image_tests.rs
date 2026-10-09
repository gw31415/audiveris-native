// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{beam_dispatch, music_symbols};
use sha2::{Digest, Sha256};
fn state(graph: &Graph, key: Key) -> Value {
    let (node, removed) = graph.retained_inter(key).unwrap();
    assert!(!removed);
    let id = graph.id(key).unwrap();
    let snapshot = graph.snapshot();
    let relations=snapshot["relations"].as_array().unwrap().iter().filter(|e|e["kind"]=="ChordArticulationRelation"&&e["target"]==id).map(|e|{
        let chord=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==e["source"]).unwrap();
        json!({"chord_bounds":chord["bounds"],"grade":e["grade"],"dx":e["dx"],"dy":e["dy"],"source_ratio":e["source_ratio"],"target_ratio":e["target_ratio"],"manual":e["manual"]})
    }).collect::<Vec<_>>();
    json!({"registered":true,"manual":node["manual"],"abnormal":node["abnormal"],"relations":relations})
}
fn compare(a: &Value, b: &Value) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10)
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                compare(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, v) in a {
                compare(v, &b[k]);
            }
        }
        _ => assert_eq!(a, b),
    }
}
pub(crate) fn assert_actual_factory(music: &OwnedMusic, cleaned: &crate::GrayImage) {
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-dot-articulations/image-observed.json"
    ))
    .unwrap();
    let glyphs = audiveris_staff_removal::extract_glyphs(
        audiveris_staff_removal::BinaryPage::new(cleaned.width, cleaned.height, &cleaned.pixels)
            .unwrap(),
        audiveris_staff_removal::Orientation::Vertical,
        (0, 0),
        audiveris_staff_removal::Limits::default(),
    )
    .unwrap();
    let mut candidate_checks = 0;
    let mut positive = 0;
    let mut controlled = 0;
    for group in music_symbols::geometry::dispatch(&music.owner, &glyphs).unwrap() {
        let expected = original["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system"] == group.system.id)
            .unwrap();
        let fine = music_symbols::actual_fine_boxes(
            &music.owner.graph,
            group.system.id,
            music.owner.interline,
        )
        .unwrap();
        let parts = group.glyphs.into_iter().cloned().collect::<Vec<_>>();
        let candidates = music_symbols::symbol_candidates(
            &parts,
            music.owner.interline,
            group.system.left,
            &fine,
        )
        .unwrap();
        let actual = candidates
            .iter()
            .map(|(g, _)| {
                (
                    format!(
                        "{:x}",
                        Sha256::digest(serde_json::to_vec(&beam_dispatch::glyph_json(g)).unwrap())
                    ),
                    g,
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let resolve = |row: &Value| {
            *actual
                .get(&format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&row["glyph"]).unwrap())
                ))
                .unwrap()
        };
        for row in expected["controls"].as_array().unwrap() {
            let glyph = resolve(row);
            let b: [i64; 4] =
                serde_json::from_value(beam_dispatch::glyph_json(glyph)["bounds"].clone()).unwrap();
            let found = lookup(music, group.system.id, b, "STACCATO").unwrap();
            let actual=found.as_ref().map(|(key,r)|json!({"chord_bounds":music.owner.graph.retained_inter(*key).unwrap().0["bounds"],"grade":r["grade"],"dx":r["dx"],"dy":r["dy"],"source_ratio":r["source_ratio"],"target_ratio":r["target_ratio"]})).unwrap_or(Value::Null);
            compare(&actual, &row["articulation_lookup"]);
            candidate_checks += 1;
            positive += usize::from(found.is_some());
        }
        for row in expected["articulation_controls"].as_array().unwrap() {
            let glyph = resolve(row);
            let mut registry = music.owner.prepared.registry.clone();
            let gid = registry.register(glyph).unwrap();
            let shape = row["shape"].as_str().unwrap();
            let grade = row["grade"].as_f64().unwrap();
            let data = create(gid, beam_dispatch::glyph_json(glyph), shape, grade).unwrap();
            compare(
                &json!({"registered":false,"manual":data["manual"],"abnormal":data["abnormal"],"relations":[]}),
                &row["plain"],
            );
            let mut graph = music.owner.graph.clone();
            let before = graph.snapshot();
            let key = graph.next_symbol_key(group.system.id).unwrap();
            graph.vertex(key, data, false).unwrap();
            compare(&state(&graph, key), &row["added"]);
            let bounds: [i64; 4] =
                serde_json::from_value(beam_dispatch::glyph_json(glyph)["bounds"].clone()).unwrap();
            let (chord, r) = lookup(music, group.system.id, bounds, shape)
                .unwrap()
                .unwrap();
            graph
                .relations(group.system.id, Phase::Symbol, vec![(chord, key, r)])
                .unwrap();
            compare(&state(&graph, key), &row["linked"]);
            let linked = graph.snapshot();
            for name in ["inters", "relations"] {
                let actual = linked[name]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| (n["id"].as_u64().unwrap(), n))
                    .collect::<std::collections::BTreeMap<_, _>>();
                for old in before[name].as_array().unwrap() {
                    assert_eq!(actual[&old["id"].as_u64().unwrap()], old);
                }
            }
            let head = music
                .owner
                .graph
                .live_inter_keys(group.system.id, "HeadInter")[0];
            assert!(graph.relations(group.system.id,Phase::Symbol,vec![(head,key,json!({"kind":"ChordArticulationRelation","grade":0.5,"source_ratio":1.,"target_ratio":2.25}))]).is_err());
            assert_eq!(graph.snapshot(), linked);
            let states = graph.remove_articulation_links_observed(key);
            assert_eq!(
                states.len(),
                row["removed_states"].as_array().unwrap().len()
            );
            for (node, expected) in states.iter().zip(row["removed_states"].as_array().unwrap()) {
                compare(
                    &json!({"registered":true,"manual":node["manual"],"abnormal":node["abnormal"],"relations":[]}),
                    expected,
                );
            }
            graph.remove_music_inter(key).unwrap();
            assert_ne!(graph.next_symbol_key(group.system.id).unwrap(), key);
            let mut normal = music.owner.graph.clone();
            let created = create_added(
                &mut normal,
                music,
                group.system.id,
                gid,
                beam_dispatch::glyph_json(glyph),
                shape,
                grade,
            )
            .unwrap()
            .unwrap();
            compare(&state(&normal, created), &row["linked"]);
            controlled += 1;
        }
    }
    assert_eq!((candidate_checks, positive, controlled), (97, 15, 16));
}
