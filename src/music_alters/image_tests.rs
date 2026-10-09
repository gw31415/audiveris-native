// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{beam_dispatch, music_measures::OwnedMusic, music_symbols};
use sha2::{Digest, Sha256};
fn fingerprint(g: &audiveris_staff_removal::GlyphPixels) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&beam_dispatch::glyph_json(g)).unwrap())
    )
}
fn state(graph: &Graph, key: Key) -> Value {
    let (alter, removed) = graph.retained_inter(key).unwrap();
    assert!(!removed);
    let snapshot = graph.snapshot();
    let id = graph.id(key).unwrap();
    let relations=snapshot["relations"].as_array().unwrap().iter().filter(|e|e["kind"]=="AlterHeadRelation" && e["source"]==id).map(|e| {
        let head=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==e["target"]).unwrap();
        json!({"head_bounds":head["bounds"],"head_staff":head["staff_id"],"grade":e["grade"],"dx":e["dx"],"dy":e["dy"],"manual":e["manual"],"source_ratio":e["source_ratio"],"target_ratio":e["target_ratio"]})
    }).collect::<Vec<_>>();
    json!({"registered":true,"manual":alter["manual"],"abnormal":alter["abnormal"],"relations":relations})
}
fn assert_observed(actual: &Value, original: &Value) {
    match (actual, original) {
        (Value::Number(a), Value::Number(b)) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-12,
            "{a} vs {b}"
        ),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                assert_observed(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, v) in a {
                assert_observed(v, &b[k]);
            }
        }
        _ => assert_eq!(actual, original),
    }
}
pub(crate) fn assert_actual_image_factory(music: &OwnedMusic, cleaned: &crate::GrayImage) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-alters/image-observed.json"
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
    let mut count = 0;
    let mut linked = 0;
    for group in music_symbols::geometry::dispatch(&music.owner, &glyphs).unwrap() {
        let original = source["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| Some(group.system.id) == s["system"].as_u64())
            .unwrap();
        assert_eq!(json!(group.system.profile), original["profile"]);
        let parts = group.glyphs.into_iter().cloned().collect::<Vec<_>>();
        let candidates =
            music_symbols::symbol_candidates(&parts, music.owner.interline, group.system.left, &[])
                .unwrap();
        for (row, control) in original["before_late_checks"]
            .as_array()
            .unwrap()
            .iter()
            .zip(original["controls"].as_array().unwrap())
        {
            let digest = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&row["glyph"]).unwrap())
            );
            let (glyph, _) = candidates
                .iter()
                .find(|(g, _)| fingerprint(g) == digest)
                .unwrap();
            let staff = music
                .owner
                .staffs
                .iter()
                .find(|s| Some(s.id) == row["staff"].as_u64())
                .unwrap();
            let mut registry = music.owner.prepared.registry.clone();
            let gid = registry.register(glyph).unwrap();
            let data = create(
                &music.owner,
                glyph,
                row["shape"].as_str().unwrap(),
                row["grade"].as_f64().unwrap(),
                staff,
                gid,
            )
            .unwrap();
            assert_observed(&data["pitch"], &row["pitch"]);
            assert_observed(&data["measured_pitch"], &row["measured_pitch"]);
            assert_observed(
                &json!({"registered":false,"manual":data["manual"],"abnormal":data["abnormal"],"relations":[]}),
                &control["plain"],
            );
            let mut graph = music.owner.graph.clone();
            let before = graph.snapshot();
            let key = Key {
                system: group.system.id,
                phase: Phase::Symbol,
                local: gid * 2,
            };
            graph.vertex(key, data, false).unwrap();
            assert_observed(&state(&graph, key), &control["added"]);
            set_links(&mut graph, key, music.owner.interline, group.system.profile).unwrap();
            assert_observed(&state(&graph, key), &control["linked"]);
            let snapshot = graph.snapshot();
            for name in ["inters", "relations"] {
                for old in before[name].as_array().unwrap() {
                    assert_eq!(
                        snapshot[name]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|n| n["id"] == old["id"])
                            .unwrap(),
                        old
                    );
                }
            }
            let old = graph.snapshot();
            let origin = graph.live_inter_keys(group.system.id, "StemInter")[0];
            assert!(graph.relations(group.system.id,Phase::Symbol,vec![(key,origin,json!({"kind":"AlterHeadRelation","grade":0.5,"source_ratio":2.5,"target_ratio":1.0}))]).is_err());
            assert_eq!(graph.snapshot(), old);
            let removed = graph.remove_alter_links_observed(key);
            assert_eq!(
                removed.len(),
                control["removed_states"].as_array().unwrap().len()
            );
            for (node, expected) in removed
                .iter()
                .zip(control["removed_states"].as_array().unwrap())
            {
                assert_observed(
                    &json!({"registered":true,"manual":node["manual"],"abnormal":node["abnormal"],"relations":[]}),
                    expected,
                );
            }
            linked += control["linked"]["relations"].as_array().unwrap().len();
            count += 1;
        }
    }
    assert_eq!((count, linked), (17, 13));
}

pub(crate) fn assert_supported_context(music: &OwnedMusic, cleaned: &crate::GrayImage) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-supported-alters/image-observed.json"
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
    let mut cases = 0;
    let mut actual_removed = 0;
    let mut max_actual_grade_residual = 0.0_f64;
    for group in music_symbols::geometry::dispatch(&music.owner, &glyphs).unwrap() {
        let original = source["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system"] == group.system.id)
            .unwrap();
        let parts = group.glyphs.into_iter().cloned().collect::<Vec<_>>();
        let candidates =
            music_symbols::symbol_candidates(&parts, music.owner.interline, group.system.left, &[])
                .unwrap();
        let mut actual_graph = music.owner.graph.clone();
        let mut actual_keys = Vec::new();
        for (row, observed) in original["trials"].as_array().unwrap().iter().zip(
            original["before_contextualize_and_purge"]
                .as_array()
                .unwrap(),
        ) {
            let digest = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&row["glyph"]).unwrap())
            );
            let (glyph, _) = candidates
                .iter()
                .find(|(g, _)| fingerprint(g) == digest)
                .unwrap();
            let staff = music
                .owner
                .staffs
                .iter()
                .find(|s| row["staff"] == s.id)
                .unwrap();
            let mut registry = music.owner.prepared.registry.clone();
            let gid = registry.register(glyph).unwrap();
            for control in row["controls"].as_array().unwrap() {
                let mut graph = music.owner.graph.clone();
                let key = graph.next_symbol_key(group.system.id).unwrap();
                let mut data = create(
                    &music.owner,
                    glyph,
                    row["shape"].as_str().unwrap(),
                    control["grade"].as_f64().unwrap(),
                    staff,
                    gid,
                )
                .unwrap();
                data["manual"] = control["manual"].clone();
                data["frozen"] = control["frozen"].clone();
                graph.vertex(key, data, false).unwrap();
                set_links(&mut graph, key, music.owner.interline, group.system.profile).unwrap();
                // Declared partner grades isolate the support contract from
                // whole classifier/context parity.
                for support in control["supports"].as_array().unwrap() {
                    let head = graph
                        .live_inter_keys(group.system.id, "HeadInter")
                        .into_iter()
                        .find(|k| {
                            let (n, _) = graph.retained_inter(*k).unwrap();
                            n["bounds"] == support["head_bounds"]
                                && n["staff_id"] == support["head_staff"]
                        })
                        .unwrap();
                    let (mut data, _) = graph
                        .retained_inter(head)
                        .map(|(n, r)| (n.clone(), r))
                        .unwrap();
                    data["grade"] = support["head_grade"].clone();
                    graph.vertex(head, data, false).unwrap();
                }
                let before = graph.snapshot();
                let contextual =
                    contextual_grade(&before, graph.retained_inter(key).unwrap().0).unwrap();
                assert_observed(&json!(contextual), &control["contextual_grade"]);
                let report = contextualize_and_purge(&mut graph).unwrap();
                let (node, removed) = graph.retained_inter(key).unwrap();
                assert_eq!(removed, control["weak"]);
                assert_observed(&node["contextual_grade"], &control["contextual_grade"]);
                assert_eq!(node["manual"], control["manual"]);
                assert_eq!(node["frozen"], control["frozen"]);
                assert_eq!(node["abnormal"], control["abnormal"]);
                assert_eq!(
                    report["removed_weak_alters"].as_array().unwrap().len(),
                    usize::from(removed)
                );
                let after = graph.snapshot();
                let actual = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| (n["id"].as_u64().unwrap(), n))
                    .collect::<std::collections::BTreeMap<_, _>>();
                for old in before["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| n["id"] != graph.id(key).unwrap())
                {
                    assert_eq!(actual[&old["id"].as_u64().unwrap()], old);
                }
                cases += 1;
            }
            let key = actual_graph.next_symbol_key(group.system.id).unwrap();
            actual_graph
                .vertex(
                    key,
                    create(
                        &music.owner,
                        glyph,
                        row["shape"].as_str().unwrap(),
                        observed["grade"].as_f64().unwrap(),
                        staff,
                        gid,
                    )
                    .unwrap(),
                    false,
                )
                .unwrap();
            set_links(
                &mut actual_graph,
                key,
                music.owner.interline,
                group.system.profile,
            )
            .unwrap();
            actual_keys.push(key);
        }
        let before = actual_graph.snapshot();
        let rollback_key = actual_keys
            .iter()
            .copied()
            .find(|k| {
                before["relations"].as_array().unwrap().iter().any(|e| {
                    e["kind"] == "AlterHeadRelation" && e["source"] == actual_graph.id(*k).unwrap()
                })
            })
            .unwrap_or(actual_keys[0]);
        // A late invalid partner grade must not commit earlier grade writes or
        // weak removals. Keep the real Chord and relation identities in place.
        if let Some(edge) = before["relations"].as_array().unwrap().iter().find(|e| {
            e["kind"] == "AlterHeadRelation"
                && e["source"] == actual_graph.id(rollback_key).unwrap()
        }) {
            let mut invalid = actual_graph.clone();
            let mut head = before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == edge["target"])
                .unwrap()
                .clone();
            let key: Key = serde_json::from_value(head["origin"].clone()).unwrap();
            head["grade"] = json!(1.01);
            invalid.vertex(key, head, false).unwrap();
            let before = invalid.snapshot();
            assert!(contextualize_and_purge(&mut invalid).is_err());
            assert_eq!(invalid.snapshot(), before);
        }
        contextualize_and_purge(&mut actual_graph).unwrap();
        for (key, expected) in actual_keys.into_iter().zip(
            original["after_contextualize_and_purge"]
                .as_array()
                .unwrap(),
        ) {
            let (node, removed) = actual_graph.retained_inter(key).unwrap();
            assert_eq!(removed, expected["removed"]);
            let residual = (node["contextual_grade"].as_f64().unwrap()
                - expected["contextual_grade"].as_f64().unwrap())
            .abs();
            max_actual_grade_residual = max_actual_grade_residual.max(residual);
            assert!(residual < 0.0004);
            assert_eq!(node["manual"], expected["manual"]);
            actual_removed += usize::from(removed);
        }
    }
    assert_eq!((cases, actual_removed), (204, 2));
    std::fs::write(
        "/tmp/real-supported-alter-native-residual.json",
        serde_json::to_vec(&json!({"max_actual_grade_residual":max_actual_grade_residual}))
            .unwrap(),
    )
    .unwrap();
}
