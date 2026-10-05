// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Group, Registry},
    head_stem_relation::{self, ConnectionInput},
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_glyph,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-stem-connection/controls.json"
    ))
    .unwrap()
}
fn subset(a: &Value, e: &Value) {
    match e {
        Value::Number(n) if n.is_f64() => assert!(
            (a.as_f64().unwrap() - n.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {e}"
        ),
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (a, e) in a.as_array().unwrap().iter().zip(v) {
                subset(a, e);
            }
        }
        Value::Object(v) => {
            for (k, e) in v {
                subset(&a[k], e);
            }
        }
        _ => assert_eq!(a, e),
    }
}
#[test]
fn original_2112_template_and_learned_scale_reference_points() {
    let g = controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["references"].as_array().unwrap().len(), 2112);
    let banks =
        ["Bravura", "Leland", "FinaleJazz"].map(|n| (n, Bank::load_all_head_coverage(n).unwrap()));
    for c in g["references"].as_array().unwrap() {
        let bank = &banks
            .iter()
            .find(|(n, _)| *n == c["family"].as_str().unwrap())
            .unwrap()
            .1;
        let actual = head_stem_relation::stem_reference_point(
            bank,
            c["point_size"].as_u64().unwrap() as u16,
            c["shape"].as_str().unwrap(),
            serde_json::from_value(c["bounds"].clone()).unwrap(),
            serde_json::from_value(c["side"].clone()).unwrap(),
            serde_json::from_value(c["vertical"].clone()).unwrap(),
            serde_json::from_value(c["learned_dx"].clone()).unwrap(),
        )
        .unwrap();
        subset(&json!(actual), &c["point"]);
    }
}
#[test]
fn original_288_gap_grades_and_5760_checked_physical_and_median_connections() {
    let g = controls();
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    assert_eq!(g["minimum_grade"], 0.1);
    assert_eq!(g["gaps"].as_array().unwrap().len(), 288);
    assert_eq!(g["connections"].as_array().unwrap().len(), 5760);
    for c in g["gaps"].as_array().unwrap() {
        let a = head_stem_relation::gap_grade(
            c["dx"].as_f64().unwrap(),
            c["dy"].as_f64().unwrap(),
            c["profile"].as_u64().unwrap() as u8,
        )
        .unwrap();
        for k in [
            "grade",
            "x_impact",
            "y_impact",
            "source_ratio",
            "target_ratio",
            "eligible",
        ] {
            subset(&a[k], &c[k]);
        }
    }
    for c in g["connections"].as_array().unwrap() {
        let a = head_stem_relation::check_relation(
            &bank,
            51,
            ConnectionInput {
                shape: c["shape"].as_str().unwrap(),
                bounds: serde_json::from_value(c["bounds"].clone()).unwrap(),
                stem: serde_json::from_value(c["stem"].clone()).unwrap(),
                glyph_bounds: serde_json::from_value(c["glyph_bounds"].clone()).unwrap(),
                vertical: serde_json::from_value(c["vertical"].clone()).unwrap(),
                interline: 11,
                profile: c["profile"].as_u64().unwrap() as u8,
                learned_dx: [None, None],
            },
        )
        .unwrap();
        subset(&a.unwrap_or(Value::Null), &c["relation"]);
    }
}
#[test]
fn original_40_physical_glyph_part_unions_geometry_and_registry_acquisition() {
    let g = controls();
    assert_eq!(g["glyphs"].as_array().unwrap().len(), 40);
    for c in g["glyphs"].as_array().unwrap() {
        let parts = c["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| beam_dispatch::glyph_from_json(p).unwrap())
            .collect::<Vec<_>>();
        let actual = stem_glyph::assemble(&parts).unwrap();
        assert_eq!(beam_dispatch::glyph_json(&actual), c["glyph"]);
        let geo = stem_glyph::geometry(&actual).unwrap();
        subset(
            &serde_json::to_value(&geo).unwrap(),
            &json!({"median":c["median"],"width":c["width"],"bounds":c["bounds"]}),
        );
        let mut registry = Registry::default();
        let old = registry.register(&parts[0]).unwrap();
        registry.add_group(old, Group::VerticalSeed).unwrap();
        let id = stem_glyph::acquire(&mut registry, &parts).unwrap();
        assert_eq!(id, stem_glyph::acquire(&mut registry, &parts).unwrap());
        assert_eq!(
            beam_dispatch::glyph_json(&registry.get(id).unwrap().glyph),
            c["glyph"]
        );
        assert!(
            registry
                .get(old)
                .unwrap()
                .groups
                .contains(&Group::VerticalSeed)
        );
        if parts.len() == 1
            && parts[0].orientation == audiveris_staff_removal::Orientation::Vertical
        {
            assert_eq!(id, old);
        }
        if geo.bounds == [0, 0, 0, 0] {
            assert_eq!(
                stem_glyph::record(1, &actual, 0.8).unwrap_err(),
                "unsupported_nonvertical_stem_geometry"
            );
        } else {
            let record = stem_glyph::record(1, &actual, 0.8).unwrap();
            assert_eq!(record["kind"], "StemInter");
            subset(&record["median"], &c["median"]);
            assert_eq!(record["bounds"], c["bounds"]);
        }
    }
}
fn base(shape: &str) -> (Graph, Vec<Value>, Value) {
    let mut graph = Graph::default();
    graph.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"},{"id":2,"kind":"BarlineInter"}],"relations":[{"kind":"BarGroupRelation","source":1,"target":2}]}]}})).unwrap();
    let heads=(1..=2).map(|id|json!({"id":id,"kind":"HeadInter","shape":shape,"bounds":[40,50+20*(id-1),15,10],"grade":0.8,"pitch":0.,"staff_id":1,"glyph":null,"abnormal":true,"removed":false})).collect::<Vec<_>>();
    graph
        .sync_heads(
            1,
            &heads,
            &[json!({"kind":"Exclusion","source":1,"target":2})],
            &[],
        )
        .unwrap();
    let glyph = beam_dispatch::glyph_from_json(
        &json!({"bounds":[53,20,2,65],"orientation":"VERTICAL","runs":[[0,0,65],[1,0,65]]}),
    )
    .unwrap();
    let stem = stem_glyph::record(1, &glyph, 0.8).unwrap();
    (graph, heads, stem)
}
fn edge(local: usize, head: usize) -> HeadStemAttachment {
    HeadStemAttachment {
        local,
        head: Key {
            system: 1,
            phase: Phase::Head,
            local: head,
        },
        stem_local: 1,
        relation: json!({"kind":"HeadStemRelation","profile":0,"head_side":"RIGHT","extension_point":[54.,50.],"dx":0.,"dy":0.,"grade":1.,"consistency":1.,"source_ratio":5.,"target_ratio":11.,"invading":true,"manual":false}),
    }
}
#[test]
fn original_twelve_nonmanual_shared_sig_parallel_remove_and_retirement_lifecycles() {
    for c in controls()["lifecycle"].as_array().unwrap() {
        assert_eq!(c["parallel_relation_accepted"], true);
        let (mut graph, mut heads, stem) = base(c["shape"].as_str().unwrap());
        for (h, bounds) in heads.iter_mut().zip(c["head_bounds"].as_array().unwrap()) {
            h["bounds"] = bounds.clone();
            h["staff_id"] = c["staff_id"].clone();
        }
        graph
            .sync_heads(
                1,
                &heads,
                &[json!({"kind":"Exclusion","source":1,"target":2})],
                &[],
            )
            .unwrap();
        let prior = graph.snapshot();
        let ids = [1, 2].map(|local| {
            graph
                .id(Key {
                    system: 1,
                    phase: Phase::Head,
                    local,
                })
                .unwrap()
        });
        for (i, links) in [
            vec![],
            vec![(1, 1)],
            vec![(1, 1), (2, 2)],
            vec![(1, 1), (2, 2), (3, 1)],
            vec![(2, 2), (3, 1)],
            vec![(2, 2)],
            vec![],
            vec![],
        ]
        .iter()
        .enumerate()
        {
            let attachments = links
                .iter()
                .map(|(local, head)| edge(*local, *head))
                .collect::<Vec<_>>();
            graph
                .sync_stems(
                    1,
                    if i < 6 {
                        std::slice::from_ref(&stem)
                    } else {
                        &[]
                    },
                    &attachments,
                )
                .unwrap();
            if i == 7 {
                for h in &mut heads {
                    h["removed"] = json!(true);
                }
                graph.sync_heads(1, &heads, &[], &[]).unwrap();
            }
            let snapshot = graph.snapshot();
            let expected = &c["states"][i];
            assert_eq!(expected["prior_SIG_preserved"], true);
            assert_eq!(expected["staff_notes_preserved"], true);
            for (j, id) in ids.iter().enumerate() {
                let h = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["id"] == *id);
                assert_eq!(h.is_none(), expected["heads"][j]["removed"] == true);
                if let Some(h) = h {
                    assert_eq!(h["abnormal"], expected["heads"][j]["abnormal"]);
                    assert_eq!(h["staff_id"], c["staff_id"]);
                }
            }
            let live_stems = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["origin"]["phase"] == "STEM")
                .collect::<Vec<_>>();
            assert_eq!(
                live_stems.is_empty(),
                expected["stems"][0]["removed"] == true
            );
            if let Some(s) = live_stems.first() {
                assert_eq!(s["abnormal"], expected["stems"][0]["abnormal"]);
            }
            let edges = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["kind"] == "HeadStemRelation")
                .map(|r| json!([ids.iter().position(|id| r["source"] == *id).unwrap() + 1, 1]))
                .collect::<Vec<_>>();
            assert_eq!(json!(edges), expected["edges"]);
            for v in prior["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["origin"]["phase"] == "GRID")
            {
                assert!(snapshot["inters"].as_array().unwrap().contains(v));
            }
            assert_eq!(snapshot["relations"][0], prior["relations"][0]);
        }
    }
}
#[test]
fn original_144_selected_physical_stem_and_head_batches_commit_both_indexes() {
    use audiveris_native::{
        head_stem_relation::VerticalSide,
        stem_attachment::{self, HeadRequest, SelectedStem},
    };
    let g = controls();
    assert_eq!(g["batches"].as_array().unwrap().len(), 144);
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    for c in g["batches"].as_array().unwrap() {
        let (mut graph, mut heads, _) = base(c["shape"].as_str().unwrap());
        heads.truncate(1);
        heads[0]["bounds"] = c["head_bounds"].clone();
        heads[0]["staff_id"] = c["staff_id"].clone();
        graph.sync_heads(1, &heads, &[], &[]).unwrap();
        let before = graph.snapshot();
        let parts = c["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| beam_dispatch::glyph_from_json(p).unwrap())
            .collect::<Vec<_>>();
        let mut registry = Registry::default();
        let existing = registry.register(&parts[0]).unwrap();
        registry.add_group(existing, Group::VerticalSeed).unwrap();
        let stems = [SelectedStem {
            local: 1,
            parts: &parts,
            grade: 0.8,
        }];
        let requests = [HeadRequest {
            local_relation: 1,
            head: Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            },
            stem_local: 1,
            point_size: 51,
            vertical: serde_json::from_value::<VerticalSide>(c["vertical"].clone()).unwrap(),
            profile: 0,
            learned_dx: [None, None],
        }];
        let result = stem_attachment::register_selected(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &requests,
        )
        .unwrap();
        assert_eq!(result["selected_stem_boundary_complete"], true);
        for flag in [
            "automatic_stems_complete",
            "stems_complete",
            "heads_complete",
            "musicxml_complete",
        ] {
            assert_eq!(result[flag], false);
        }
        let actual = &result["physical_stems"][0];
        assert_eq!(actual["glyph"], c["glyph"]);
        assert_eq!(actual["bounds"], c["stem_bounds"]);
        subset(&actual["median"], &c["median"]);
        subset(&actual["width"], &c["width"]);
        subset(&result["head_requests"][0]["relation"], &c["relation"]);
        assert_eq!(actual["glyph_id"], existing);
        assert!(
            registry
                .get(existing)
                .unwrap()
                .groups
                .contains(&Group::VerticalSeed)
        );
        let snapshot = graph.snapshot();
        let h = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["origin"]["phase"] == "HEAD")
            .unwrap();
        let s = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["origin"]["phase"] == "STEM")
            .unwrap();
        assert_eq!(h["abnormal"], c["after"]["heads"][0]["abnormal"]);
        assert_eq!(s["abnormal"], c["after"]["stems"][0]["abnormal"]);
        assert_eq!(
            snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["kind"] == "HeadStemRelation")
                .count(),
            c["after"]["edges"].as_array().unwrap().len()
        );
        assert_eq!(c["after"]["prior_SIG_preserved"], true);
        assert_eq!(c["after"]["staff_notes_preserved"], true);
        for v in before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["origin"]["phase"] == "GRID")
        {
            assert!(snapshot["inters"].as_array().unwrap().contains(v));
        }
        assert_eq!(snapshot["relations"][0], before["relations"][0]);
    }
}
#[test]
fn malformed_stale_duplicate_and_manual_inputs_roll_back_both_indexes_and_prior_phases() {
    use audiveris_native::{
        head_stem_relation::VerticalSide,
        stem_attachment::{self, HeadRequest, SelectedStem},
    };
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let (mut graph, heads, stem) = base("NOTEHEAD_BLACK");
    let part = beam_dispatch::glyph_from_json(&stem["glyph"]).unwrap();
    let parts = vec![part.clone()];
    let mut bad = part.clone();
    bad.runs[0].length = 0;
    let mut registry = Registry::default();
    let old = registry.register(&part).unwrap();
    registry.add_group(old, Group::VerticalSeed).unwrap();
    let before = graph.snapshot();
    let index = registry.snapshot();
    let request = || HeadRequest {
        local_relation: 1,
        head: Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        },
        stem_local: 1,
        point_size: 51,
        vertical: VerticalSide::Top,
        profile: 0,
        learned_dx: [None, None],
    };
    let valid = [SelectedStem {
        local: 1,
        parts: &parts,
        grade: 0.8,
    }];
    let bad_parts = [bad];
    let invalid = [
        SelectedStem {
            local: 1,
            parts: &parts,
            grade: 0.8,
        },
        SelectedStem {
            local: 2,
            parts: &bad_parts,
            grade: 0.8,
        },
    ];
    assert!(
        stem_attachment::register_selected(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &invalid,
            &[request()]
        )
        .is_err()
    );
    for mode in 0..7 {
        let mut r = request();
        match mode {
            0 => r.stem_local = 99,
            1 => r.head.local = 99,
            2 => r.head.system = 2,
            3 => r.point_size = 50,
            4 => r.profile = 3,
            5 => r.learned_dx = [Some(f64::NAN), None],
            _ => r.local_relation = 0,
        }
        assert!(
            stem_attachment::register_selected(
                1,
                11,
                &bank,
                &mut registry,
                &mut graph,
                &valid,
                &[r]
            )
            .is_err()
        );
        assert_eq!(graph.snapshot(), before);
        assert_eq!(registry.snapshot(), index);
    }
    assert!(
        stem_attachment::register_selected(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &valid,
            &[request(), request()]
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(registry.snapshot(), index);
    graph
        .sync_stems(1, std::slice::from_ref(&stem), &[edge(1, 1), edge(2, 1)])
        .unwrap();
    let linked = graph.snapshot();
    let relation_ids = linked["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "HeadStemRelation")
        .map(|r| r["id"].clone())
        .collect::<Vec<_>>();
    graph
        .sync_stems(1, std::slice::from_ref(&stem), &[edge(2, 1)])
        .unwrap();
    let after = graph.snapshot();
    assert_eq!(
        after["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"] == "HeadStemRelation")
            .unwrap()["id"],
        relation_ids[1]
    );
    assert_eq!(
        graph.sync_heads(1, &heads, &[], &[]).unwrap_err(),
        "complete_shared_sig_stems_state_required"
    );
    assert_eq!(graph.snapshot(), after);
    assert_eq!(
        graph
            .sync_stems(1, std::slice::from_ref(&stem), &[edge(2, 2)])
            .unwrap_err(),
        "changed_shared_stem_relation_endpoints"
    );
    assert_eq!(graph.snapshot(), after);
    let mut manual = edge(2, 1);
    manual.relation["manual"] = json!(true);
    assert!(
        graph
            .sync_stems(1, std::slice::from_ref(&stem), &[manual])
            .is_err()
    );
    assert_eq!(graph.snapshot(), after);
    let result =
        stem_attachment::register_selected(1, 11, &bank, &mut registry, &mut graph, &[], &[])
            .unwrap();
    assert_eq!(result["stems_complete"], false);
    assert_eq!(result["physical_stems"], json!([]));
    assert_eq!(registry.snapshot(), index);
    assert!(
        graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["origin"]["phase"] == "HEAD")
            .all(|h| h["abnormal"] == true)
    );
    assert!(stem_glyph::assemble(&[]).is_err());
    assert!(head_stem_relation::gap_grade(f64::NAN, 0., 0).is_err());
    assert!(head_stem_relation::gap_grade(0., 0., 3).is_err());
}
