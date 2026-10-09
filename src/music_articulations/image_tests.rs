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

pub(crate) fn assert_real_reduction(music: &mut OwnedMusic) {
    let support: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-articulation-reduction/support-observed.json"
    ))
    .unwrap();
    let prior: Value = serde_json::from_str(include_str!(
        "../../evidence/real-symbol-ensemble/VALIDATION.json"
    ))
    .unwrap();
    let partner_bound = prior["numeric_grade_residuals"]["maximum_absolute_difference"]
        .as_f64()
        .unwrap();
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-articulation-reduction/stages-observed.json"
    ))
    .unwrap();
    let ordinary: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-articulation-reduction/image-observed.json"
    ))
    .unwrap();
    assert_eq!(source["stages"][4], ordinary["stages"][1]);
    let before = music.owner.graph.snapshot();
    let address = music.chords.as_ptr();
    let mut graph = music.owner.graph.clone();
    let mut rows = Vec::new();
    let observed = |graph: &Graph| {
        let snap = graph.snapshot();
        let nodes = snap["inters"].as_array().unwrap();
        music.owner.systems.iter().map(|sys| {
            let mut arts = nodes.iter().filter(|n| n["kind"]=="ArticulationInter" && n["origin"]["system"]==sys.id).map(|n| {
                let rels = snap["relations"].as_array().unwrap().iter().filter(|e|e["target"]==n["id"]).map(|e| {
                    let ch = nodes.iter().find(|c|c["id"]==e["source"]).unwrap();
                    json!({"kind":e["kind"],"opposite_kind":ch["kind"],"bounds":ch["bounds"]})
                }).collect::<Vec<_>>();
                json!({"shape":n["shape"],"bounds":n["bounds"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"relations":rels})
            }).collect::<Vec<_>>();
            arts.sort_by_key(|n|format!("{}{}",n["shape"],n["bounds"]));
            json!({"system":sys.id,"articulations":arts})
        }).collect::<Vec<_>>()
    };
    let expected = |stage: usize| {
        let mut systems = source["stages"][stage]["systems"]
            .as_array()
            .unwrap()
            .clone();
        for sys in &mut systems {
            sys["articulations"]
                .as_array_mut()
                .unwrap()
                .sort_by_key(|n| format!("{}{}", n["shape"], n["bounds"]));
        }
        systems
    };
    let initial = observed(&graph);
    compare(&json!(initial), &json!(expected(0)));
    let weak = contextualize_and_purge(&mut graph, &music.chords).unwrap();
    let mut contexts = initial.clone();
    let mut residuals = Vec::new();
    for sys in &mut contexts {
        for n in sys["articulations"].as_array_mut().unwrap() {
            let native = weak["contextualized"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["bounds"] == n["bounds"] && a["shape"] == n["shape"])
                .unwrap();
            let source = support["stages"][1]["systems"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|s| s["articulations"].as_array().unwrap())
                .find(|a| a["bounds"] == n["bounds"] && a["shape"] == n["shape"])
                .unwrap();
            let input = &native["sources"][0];
            let original = &source["relations"][0];
            let p = input["partner_grade"].as_f64().unwrap();
            let original_p = original["partner_grade"].as_f64().unwrap();
            let r = input["relation_grade"].as_f64().unwrap();
            let original_r = original["relation_grade"].as_f64().unwrap();
            let g = native["grade"].as_f64().unwrap();
            let original_g = source["grade"].as_f64().unwrap();
            let cg = native["contextual_grade"].as_f64().unwrap();
            let original_cg = source["contextual_grade"].as_f64().unwrap();
            let odds = |g: f64, r: f64, p: f64| g / (g + (1. - g) / (1. + 2.5 * r * p));
            assert!((odds(original_g, original_r, original_p) - original_cg).abs() < 1e-12);
            assert!((odds(g, r, p) - cg).abs() < 1e-12);
            assert!((p - original_p).abs() <= partner_bound + 1e-10);
            assert!((r - original_r).abs() < 1e-10);
            assert!((g - original_g).abs() < 1e-10);
            // Existing Head confidence residual propagates through support.
            // Its derivative in cached partner grade is at most2.5/4.
            assert!((cg - original_cg).abs() <= 0.625 * (p - original_p).abs() + 1e-9);
            residuals.push(json!({"bounds":n["bounds"],"shape":n["shape"],"native_partner_grade":p,"original_partner_grade":original_p,"native_contextual_grade":cg,"original_contextual_grade":original_cg,"partner_residual":(p-original_p).abs(),"context_residual":(cg-original_cg).abs()}));
            n["grade"] = native["grade"].clone();
            n["contextual_grade"] = native["contextual_grade"].clone();
        }
    }
    rows.push(json!({"stage":"CONTEXTUALIZED","actual":contexts,"expected":expected(1)}));
    let purged = observed(&graph);
    rows.push(json!({"stage":"WEAK_PURGED","actual":purged,"expected":expected(2)}));
    std::fs::write(
        "/tmp/real-articulation-reduction-native-current.json",
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    let without_context_grade = |mut systems: Vec<Value>| {
        for s in &mut systems {
            for n in s["articulations"].as_array_mut().unwrap() {
                n.as_object_mut().unwrap().remove("contextual_grade");
            }
        }
        systems
    };
    compare(
        &json!(without_context_grade(contexts)),
        &json!(without_context_grade(expected(1))),
    );
    compare(
        &json!(without_context_grade(purged)),
        &json!(without_context_grade(expected(2))),
    );
    let redundancy = check_redundancy(&mut graph, &music.chords).unwrap();
    let reduced = observed(&graph);
    rows.push(json!({"stage":"REDUNDANCY_CHECKED","actual":reduced,"expected":expected(3)}));
    std::fs::write(
        "/tmp/real-articulation-reduction-native-current.json",
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    compare(
        &json!(without_context_grade(reduced.clone())),
        &json!(without_context_grade(expected(3))),
    );
    compare(
        &json!(without_context_grade(reduced)),
        &json!(without_context_grade(expected(4))),
    );
    assert_eq!(
        weak["removed_weak_articulation_ids"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        redundancy["removed_redundant_articulation_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let after = graph.snapshot();
    for n in before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] != "ArticulationInter")
    {
        assert_eq!(
            after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["id"] == n["id"])
                .unwrap(),
            n
        );
    }
    for e in before["relations"].as_array().unwrap().iter() {
        if let Some(held) = after["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == e["id"])
        {
            assert_eq!(held, e);
        } else {
            assert_eq!(e["kind"], "ChordArticulationRelation");
        }
    }
    assert_eq!(music.chords.as_ptr(), address);
    music.owner.graph = graph;
    std::fs::write("/tmp/real-articulation-reduction-native-observed.json",serde_json::to_vec_pretty(&json!({"stages":rows,"weak":weak,"redundancy":redundancy,"numeric_cached_Chord_and_context_grade_residuals":residuals,"original16_10_8_membership_and_relations_match":true,"support_equation_matches_actual_cached_Chord_inputs":true,"ordinary_original_LINKS8_membership_match":true,"actual_grade_numeric_parity_claimed":false,"all_other_Inter_bodies_clocks_and_retained_relations_unchanged":true,"actual_Chord_vector_retained":true,"quality_accepted":false})).unwrap()).unwrap();
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
