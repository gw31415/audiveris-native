// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Registry},
    head_stem_relation,
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_attachment::{self, HeadRequest, SelectedStem},
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-stem-support/controls.json"
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
        profile: 0,
        learned_dx: [None, None],
    }
}
#[test]
fn original_1536_selected_physical_stem_length_support_and_sig_batches() {
    let g = controls();
    let cases = g["batches"].as_array().unwrap();
    assert_eq!(cases.len(), 1536);
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let mut accepted = 0;
    let mut small = 0;
    for c in cases {
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
        let unset = stem_attachment::register_selected(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[request(c)],
        )
        .unwrap();
        subset(&unset["head_requests"][0]["relation"], &c["before"]);
        let old = graph.snapshot();
        let ids = old["relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].clone())
            .collect::<Vec<_>>();
        let r = stem_attachment::register_selected_with_consistency(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[request(c)],
        )
        .unwrap();
        assert_eq!(r["length_consistency_populated"], true);
        subset(&r["head_requests"][0]["relation"], &c["relation"]);
        assert_eq!(r["physical_stems"][0]["glyph"], c["glyph"]);
        subset(&r["physical_stems"][0]["median"], &c["median"]);
        subset(&r["physical_stems"][0]["width"], &c["width"]);
        assert_eq!(r["physical_stems"][0]["bounds"], c["stem_bounds"]);
        if !c["relation"].is_null() {
            accepted += 1;
            if c["small_head"] == true {
                small += 1;
            }
            assert_eq!(
                r["head_requests"][0]["relation"]["consistency_interline"],
                11
            );
            assert_eq!(
                r["head_requests"][0]["relation"]["grade"],
                unset["head_requests"][0]["relation"]["grade"]
            );
            assert_eq!(
                r["head_requests"][0]["relation"]["target_ratio"],
                unset["head_requests"][0]["relation"]["target_ratio"]
            );
        }
        let now = graph.snapshot();
        assert_eq!(
            now["relations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["id"].clone())
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(now["relations"][0], before["relations"][0]);
        for v in before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["origin"]["phase"] == "GRID")
        {
            assert!(now["inters"].as_array().unwrap().contains(v));
        }
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
        for flag in [
            "automatic_stems_complete",
            "stems_complete",
            "heads_complete",
            "musicxml_complete",
        ] {
            assert_eq!(r[flag], false);
        }
        assert_eq!(c["after"]["prior_SIG_preserved"], true);
        assert_eq!(c["after"]["staff_notes_preserved"], true);
    }
    assert_eq!(accepted, 1152);
    assert_eq!(small, 192);
}
#[test]
fn measured_support_context_guards_roll_back_and_preserve_unset_api() {
    let data = controls();
    let negative = data["batches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["glyph"]["bounds"][1].as_i64().unwrap() < 0)
        .unwrap();
    let signed = beam_dispatch::glyph_from_json(&negative["glyph"]).unwrap();
    assert!(audiveris_native::beam_structure::glyph_geometry(&signed).is_err());
    let fitted = audiveris_native::stem_glyph::geometry(&signed).unwrap();
    subset(&json!(fitted.median), &negative["median"]);
    assert_eq!(json!(fitted.bounds), negative["stem_bounds"]);
    let mut outside = signed.clone();
    outside.top = -(audiveris_native::sections::MAX_DIMENSION as i32) - 1;
    assert!(audiveris_native::stem_glyph::geometry(&outside).is_err());
    let c = &data["batches"][0];
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
    let r = stem_attachment::register_selected_with_consistency(
        1,
        11,
        &bank,
        &mut registry,
        &mut graph,
        &stems,
        &[request(c)],
    )
    .unwrap();
    let before = graph.snapshot();
    let index = registry.snapshot();
    let stem = &r["physical_stems"][0];
    let good = &r["head_requests"][0]["relation"];
    for mode in 0..6 {
        let mut relation = good.clone();
        match mode {
            0 => {
                relation["consistency"] = json!(1.);
            }
            1 => {
                relation["consistency_interline"] = json!(0);
            }
            2 => {
                relation
                    .as_object_mut()
                    .unwrap()
                    .remove("consistency_interline");
            }
            3 => {
                relation["source_ratio"] = json!(999.);
            }
            4 => {
                relation["target_ratio"] = json!(999.);
            }
            _ => {
                relation["consistency_interline"] = json!("11");
            }
        }
        let attachment = HeadStemAttachment {
            local: 1,
            head: request(c).head,
            stem_local: 1,
            relation,
        };
        assert!(
            graph
                .sync_stems(1, std::slice::from_ref(stem), &[attachment])
                .is_err()
        );
        assert_eq!(graph.snapshot(), before);
        assert_eq!(registry.snapshot(), index);
    }
    let mut bad = request(c);
    bad.point_size = 50;
    assert!(
        stem_attachment::register_selected_with_consistency(
            1,
            11,
            &bank,
            &mut registry,
            &mut graph,
            &stems,
            &[bad]
        )
        .is_err()
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(registry.snapshot(), index);
    let old = stem_attachment::register_selected(
        1,
        11,
        &bank,
        &mut registry,
        &mut graph,
        &stems,
        &[request(c)],
    )
    .unwrap();
    subset(&old["head_requests"][0]["relation"], &c["before"]);
    assert!(old.get("length_consistency_populated").is_none());
    assert!(
        old["head_requests"][0]["relation"]
            .get("consistency_interline")
            .is_none()
    );
    for (line, il, shape) in [
        ([0., 1., 0., 1.], 11, "NOTEHEAD_BLACK_SMALL"),
        ([0., 2., 0., 1.], 11, "NOTEHEAD_BLACK"),
        ([0., 1., 0., 2.], 0, "NOTEHEAD_BLACK"),
        ([0., 1., 0., 2.], 129, "NOTEHEAD_BLACK"),
        ([0., 1., 0., f64::INFINITY], 11, "NOTEHEAD_BLACK"),
        ([0., 1., 0., 2.], 11, "WHOLE_NOTE"),
    ] {
        assert!(head_stem_relation::measured_consistency(shape, line, il).is_err());
    }
    assert_eq!(
        head_stem_relation::consistency(true, Some(0.)).unwrap(),
        f64::INFINITY
    );
}
