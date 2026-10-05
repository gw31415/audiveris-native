// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Registry},
    head_stem_relation,
    shared_sig::{Graph, Key, Phase},
    stem_attachment::{self, HeadRequest, SelectedStem},
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-stem-stump/controls.json"
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
fn base(c: &Value) -> Graph {
    let mut g = Graph::default();
    g.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"},{"id":2,"kind":"BarlineInter"}],"relations":[{"kind":"BarGroupRelation","source":1,"target":2}]}]}})).unwrap();
    g.sync_heads(1,&[json!({"id":1,"kind":"HeadInter","shape":c["shape"],"bounds":c["head_bounds"],"staff_id":c["staff_id"],"grade":0.8,"abnormal":true,"removed":false,"manual":false})],&[],&[]).unwrap();
    g
}
fn request(c: &Value) -> HeadRequest {
    HeadRequest {
        local_relation: 1,
        head: Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        },
        stem_local: 1,
        point_size: 51,
        vertical: serde_json::from_value(c["vertical"].clone()).unwrap(),
        profile: c["profile"].as_u64().unwrap() as u8,
        learned_dx: [None, None],
    }
}
#[test]
fn original_6048_explicit_corner_stumps_on_two_part_stems_match_gaps_support_and_sig() {
    let data = controls();
    let cases = data["batches"].as_array().unwrap();
    assert_eq!(cases.len(), 6048);
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let mut accepted = 0;
    let mut missing = 0;
    let mut wrong_union_accepts = 0;
    for c in cases {
        let parts = c["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
            .collect::<Vec<_>>();
        let stump =
            (!c["stump"].is_null()).then(|| beam_dispatch::glyph_from_json(&c["stump"]).unwrap());
        if stump.is_none() {
            missing += 1;
        }
        let stems = [SelectedStem {
            local: 1,
            parts: &parts,
            grade: 0.8,
        }];
        let mut registry = Registry::default();
        let mut graph = base(c);
        let before = graph.snapshot();
        let contact = stem_attachment::StumpRequest {
            request: request(c),
            stump: stump.as_ref(),
        };
        let report = stem_attachment::register_selected_with_stumps(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[contact],
        )
        .unwrap();
        assert_eq!(report["corner_stumps_explicit"], true);
        assert_eq!(report["length_consistency_populated"], true);
        assert_eq!(report["physical_stems"][0]["glyph"], c["glyph"]);
        subset(&report["physical_stems"][0]["median"], &c["median"]);
        subset(&report["physical_stems"][0]["width"], &c["width"]);
        assert_eq!(report["physical_stems"][0]["bounds"], c["stem_bounds"]);
        assert_eq!(
            report["head_requests"][0]["stump_bounds"],
            c["stump"].get("bounds").cloned().unwrap_or(Value::Null)
        );
        subset(&report["head_requests"][0]["relation"], &c["relation"]);
        if !c["relation"].is_null() {
            accepted += 1;
        }
        if c["relation"].is_null() && !c["union_contact_relation"].is_null() {
            wrong_union_accepts += 1;
        }
        let input = head_stem_relation::ConnectionInput {
            shape: c["shape"].as_str().unwrap(),
            bounds: serde_json::from_value(c["head_bounds"].clone()).unwrap(),
            stem: serde_json::from_value(c["median"].clone()).unwrap(),
            glyph_bounds: Some(serde_json::from_value(c["glyph"]["bounds"].clone()).unwrap()),
            vertical: request(c).vertical,
            interline: 11,
            profile: request(c).profile,
            learned_dx: [None, None],
        };
        subset(
            &json!(head_stem_relation::check_relation(&bank, 51, input).unwrap()),
            &c["union_contact_relation"],
        );
        let now = graph.snapshot();
        assert_eq!(now["relations"][0], before["relations"][0]);
        for (phase, states) in [
            ("HEAD", &c["after"]["heads"]),
            ("STEM", &c["after"]["stems"]),
        ] {
            let v = now["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["origin"]["phase"] == phase)
                .unwrap();
            assert_eq!(v["abnormal"], states[0]["abnormal"]);
        }
        assert_eq!(
            now["relations"].as_array().unwrap().len(),
            1 + c["after"]["edges"].as_array().unwrap().len()
        );
        assert_eq!(c["after"]["prior_SIG_preserved"], true);
        assert_eq!(c["after"]["staff_notes_preserved"], true);
        for flag in [
            "automatic_stems_complete",
            "stems_complete",
            "heads_complete",
            "musicxml_complete",
        ] {
            assert_eq!(report[flag], false);
        }
    }
    assert_eq!(accepted, 4272);
    assert_eq!(missing, 3024);
    assert!(wrong_union_accepts > 0);
    println!("wrong union contact would accept {wrong_union_accepts} rejected requests");
}
#[test]
fn multiple_parts_require_stump_contract_and_invalid_stumps_roll_back() {
    let data = controls();
    let c = data["batches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| !c["stump"].is_null() && !c["relation"].is_null())
        .unwrap();
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let parts = c["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
        .collect::<Vec<_>>();
    let stems = [SelectedStem {
        local: 1,
        parts: &parts,
        grade: 0.8,
    }];
    let mut graph = base(c);
    let mut registry = Registry::default();
    let before = graph.snapshot();
    let index = registry.snapshot();
    assert_eq!(
        stem_attachment::register_selected(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[request(c)]
        )
        .unwrap_err(),
        "multiple_stem_parts_require_explicit_corner_stump"
    );
    assert_eq!(
        stem_attachment::register_selected_with_consistency(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[request(c)]
        )
        .unwrap_err(),
        "multiple_stem_parts_require_explicit_corner_stump"
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(registry.snapshot(), index);
    let stump = beam_dispatch::glyph_from_json(&c["stump"]).unwrap();
    let mut invalid = stump.clone();
    invalid.runs[0].length = 0;
    let invalid_contact = stem_attachment::StumpRequest {
        request: request(c),
        stump: Some(&invalid),
    };
    assert!(
        stem_attachment::register_selected_with_stumps(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[invalid_contact]
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(registry.snapshot(), index);
    let contact = || stem_attachment::StumpRequest {
        request: request(c),
        stump: Some(&stump),
    };
    assert!(
        stem_attachment::register_selected_with_stumps(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[contact(), contact()]
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(registry.snapshot(), index);
    let mut stale = request(c);
    stale.head.local = 99;
    assert!(
        stem_attachment::register_selected_with_stumps(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[stem_attachment::StumpRequest {
                request: stale,
                stump: Some(&stump)
            }]
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(registry.snapshot(), index);
    let a = stem_attachment::register_selected_with_stumps(
        1,
        11,
        &bank,
        &mut registry,
        &mut graph,
        &stems,
        &[contact()],
    )
    .unwrap();
    let snap = graph.snapshot();
    let ids = snap["relations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].clone())
        .collect::<Vec<_>>();
    let b = stem_attachment::register_selected_with_stumps(
        1,
        11,
        &bank,
        &mut registry,
        &mut graph,
        &stems,
        &[contact()],
    )
    .unwrap();
    assert_eq!(a, b);
    assert_eq!(
        graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].clone())
            .collect::<Vec<_>>(),
        ids
    );
    let no_contacts = stem_attachment::register_selected_with_stumps(
        1,
        11,
        &bank,
        &mut registry,
        &mut graph,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(no_contacts["physical_stems"], json!([]));
    assert_eq!(no_contacts["stems_complete"], false);
    assert_eq!(no_contacts["corner_stumps_explicit"], true);
}
