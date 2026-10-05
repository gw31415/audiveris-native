// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    head_beam_context, head_registration as heads,
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-registration/laura.json"
    ))
    .unwrap()
}
fn purge_controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-registration/purge-controls.json"
    ))
    .unwrap()
}
fn epilog() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-registration/epilog-controls.json"
    ))
    .unwrap()
}
fn declared(inputs: &Value) -> Vec<Value> {
    inputs
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut h = r.clone();
            h["id"] = json!(i + 1);
            h["kind"] = json!("HeadInter");
            h["staff_id"] = json!(r["staff"].as_u64().unwrap() + 1);
            if h["removed"].is_null() {
                h["removed"] = json!(false);
            }
            h
        })
        .collect()
}
fn record(a: &Value, e: &Value) {
    for k in ["shape", "bounds", "pitch", "glyph", "staff_id"] {
        assert_eq!(a[k], e[k], "{k}: {a} != {e}");
    }
    for k in ["grade", "left_dx", "right_dx"] {
        if e[k].is_null() {
            assert!(a[k].is_null(), "{k}");
        } else {
            assert!(
                (a[k].as_f64().unwrap() - e[k].as_f64().unwrap()).abs() < 1e-12,
                "{k}: {a} != {e}"
            );
        }
    }
}
fn scale(a: &Value, e: &Value) {
    assert_eq!(a.as_array().unwrap().len(), 4);
    for (a, e) in a.as_array().unwrap().iter().zip(e.as_array().unwrap()) {
        assert_eq!(a["shape"], e["shape"]);
        assert_eq!(a["side"], e["side"]);
        if e["dx"].is_null() {
            assert!(a["dx"].is_null());
        } else {
            assert!((a["dx"].as_f64().unwrap() - e["dx"].as_f64().unwrap()).abs() < 1e-14);
        }
    }
}
#[test]
fn original_40_duplicate_overlap_seed_transfer_epsilon_and_geometry_controls() {
    let g = purge_controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["controls"].as_array().unwrap().len(), 40);
    for c in g["controls"].as_array().unwrap() {
        let mut h = declared(&c["input"]);
        let (count, edges) = heads::purge(&mut h, c["do_remove"].as_bool().unwrap()).unwrap();
        heads::purge_removed_tally(&mut h);
        assert_eq!(json!(count), c["count"], "{}", c["name"]);
        assert_eq!(
            json!(
                edges
                    .iter()
                    .map(|&(a, b)| [a - 1, b - 1])
                    .collect::<Vec<_>>()
            ),
            c["exclusions"],
            "{}",
            c["name"]
        );
        assert_eq!(
            json!(
                h.iter()
                    .enumerate()
                    .filter(|(_, h)| h["removed"] != true)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>()
            ),
            c["kept_indices"]
        );
        for (a, e) in h.iter().zip(c["after"].as_array().unwrap()) {
            record(a, e);
            assert_eq!(a["removed"], e["removed"]);
        }
    }
}
#[test]
fn original_seven_beam_context_support_partition_and_parallel_relation_controls() {
    let g = epilog();
    assert_eq!(g["native_outputs_used"], false);
    for c in g["context"].as_array().unwrap() {
        let b = c["beams"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                let mut b = b.clone();
                b["id"] = b["local_id"].clone();
                b
            })
            .collect::<Vec<_>>();
        for beam in &b {
            let grade = head_beam_context::contextual_grade(
                beam["id"].as_u64().unwrap(),
                &b,
                c["relations"].as_array().unwrap(),
            )
            .unwrap();
            assert!(
                (grade - beam["contextual_grade"].as_f64().unwrap()).abs() < 1e-14,
                "mode {}",
                c["mode"]
            );
        }
    }
}
#[test]
fn original_nine_small_beam_area_grade_tie_width_and_removal_order_controls() {
    for c in epilog()["small_beams"].as_array().unwrap() {
        let mut h = declared(&c["head_inputs"]);
        let mut b = vec![c["beam_input"].clone()];
        heads::purge_small_beams(&mut h, &mut b, &[], 11).unwrap();
        assert_eq!(
            b[0]["removed"] == true,
            c["beam_after"]["removed"] == true,
            "mode {}",
            c["mode"]
        );
        for (a, e) in h.iter().zip(c["heads_after"].as_array().unwrap()) {
            record(a, e);
            assert_eq!(a["removed"], e["removed"]);
        }
    }
}
#[test]
fn original_empty_nine_ten_and_removed_seed_tally_quorum() {
    for c in epilog()["seed_scale"].as_array().unwrap() {
        scale(
            &heads::seed_scale(&declared(&c["input"])).unwrap(),
            &c["head_seed_scale"],
        );
    }
}
fn glyph_input() -> Value {
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-glyph-retrieval/laura.json"
    ))
    .unwrap();
    let systems=g["systems"].as_array().unwrap().iter().map(|s|json!({"system_id":s["system_id"],"staffs":s["staffs"].as_array().unwrap().iter().map(|f|json!({"staff_id":f["staff_id"],"seed_heads":f["scanners"][0]["glyph_retrieval"]["new_staff_seeds"].as_array().unwrap().iter().map(|h|{let mut h=h.clone();h["kind"]=json!("HeadInter");h}).collect::<Vec<_>>(),"scanners":f["scanners"].as_array().unwrap().iter().map(|c|json!({"range_glyphs":c["glyph_retrieval"]["range_glyphs"].as_array().unwrap().iter().map(|r|{let mut r=r.clone();if !r["result"].is_null(){r["result"]["kind"]=json!("HeadInter");}r}).collect::<Vec<_>>()})).collect::<Vec<_>>(),"ledger_scanners":f["ledger_scanners"].as_array().unwrap().iter().map(|c|json!({"range_glyphs":c["glyph_retrieval"]["range_glyphs"].as_array().unwrap().iter().map(|r|{let mut r=r.clone();if !r["result"].is_null(){r["result"]["kind"]=json!("HeadInter");}r}).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    json!({"head_glyph_retrieval_complete":true,"live_seed_conflicts_complete":true,"systems":systems})
}
#[test]
fn original_352_heads_513_exclusions_staff_note_order_and_epilog() {
    let mut graph = Graph::default();
    let input = glyph_input();
    let r = heads::analyze(&input, &mut graph, 11).unwrap();
    let g = original();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(r["heads_complete"], false);
    assert_eq!(r["retained_heads"], 352);
    assert_eq!(r["duplicates_removed"], 22);
    let mut exclusions = 0;
    for (s, e) in r["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(g["systems"].as_array().unwrap())
    {
        let h = s["heads"].as_array().unwrap();
        let eh = e["heads"].as_array().unwrap();
        assert_eq!(h.len(), eh.len());
        let mut ids = std::collections::BTreeMap::new();
        for (a, b) in h.iter().zip(eh) {
            record(a, b);
            assert_eq!(a["abnormal"], b["abnormal"]);
            ids.insert(b["id"].as_u64().unwrap(), a["id"].clone());
        }
        let ee = e["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| {
                ids.contains_key(&x["source"].as_u64().unwrap())
                    && ids.contains_key(&x["target"].as_u64().unwrap())
            })
            .map(|x| {
                json!([
                    ids[&x["source"].as_u64().unwrap()],
                    ids[&x["target"].as_u64().unwrap()]
                ])
            })
            .collect::<Vec<_>>();
        let ae = s["exclusions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| json!([x["source"], x["target"]]))
            .collect::<Vec<_>>();
        assert_eq!(ae, ee);
        exclusions += ae.len();
        for (a, b) in s["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(e["staffs"].as_array().unwrap())
        {
            assert_eq!(a["staff_id"], b["staff_id"]);
            assert_eq!(
                a["notes"],
                json!(
                    b["notes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|id| ids[&id.as_u64().unwrap()].clone())
                        .collect::<Vec<_>>()
                )
            );
        }
    }
    assert_eq!(exclusions, 513);
    scale(&r["head_seed_scale"], &g["head_seed_scale"]);
    let before = graph.snapshot();
    assert_eq!(
        heads::analyze(&input, &mut graph, 11).unwrap()["shared_sig"],
        before
    );
}
#[test]
fn shared_head_identity_tombstone_dangling_edge_and_transaction_contract() {
    let mut g = Graph::default();
    g.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}})).unwrap();
    let before = g.snapshot();
    let mut h = declared(
        &json!([{"bounds":[0,0,10,10],"grade":0.5,"shape":"NOTEHEAD_BLACK","pitch":0.,"staff":0,"left_dx":null,"right_dx":null},{"bounds":[1,0,10,10],"grade":0.6,"shape":"NOTEHEAD_VOID","pitch":0.,"staff":0,"left_dx":null,"right_dx":null}]),
    );
    let edge = json!({"kind":"Exclusion","source":1,"target":2});
    g.sync_heads(1, &h, std::slice::from_ref(&edge), &[])
        .unwrap();
    let first = g.snapshot();
    assert_eq!(first["inters"][0], before["inters"][0]);
    let id = g
        .id(Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        })
        .unwrap();
    assert_eq!(id, 2);
    g.sync_heads(1, &h, std::slice::from_ref(&edge), &[])
        .unwrap();
    assert_eq!(first, g.snapshot());
    let bad = json!({"kind":"Exclusion","source":1,"target":99});
    assert!(g.sync_heads(1, &h, &[bad], &[]).is_err());
    assert_eq!(first, g.snapshot());
    h[0]["removed"] = json!(true);
    g.sync_heads(1, &h, &[], &[]).unwrap();
    let end = g.snapshot();
    assert!(end["relations"].as_array().unwrap().is_empty());
    assert!(
        end["tombstones"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["id"] == id)
    );
    assert_eq!(
        g.id(Key {
            system: 1,
            phase: Phase::Head,
            local: 1
        }),
        Some(id)
    );
}
#[test]
fn empty_and_malformed_registration_do_not_partially_mutate_graph() {
    let mut g = Graph::default();
    let input = json!({"head_glyph_retrieval_complete":true,"live_seed_conflicts_complete":true,"systems":[]});
    let before = g.snapshot();
    assert_eq!(
        heads::analyze(&input, &mut g, 11).unwrap()["retained_heads"],
        0
    );
    assert_eq!(before, g.snapshot());
    let mut bad = glyph_input();
    bad["systems"][1]["staffs"][0]["seed_heads"][0]["grade"] = json!(-1.);
    assert!(heads::analyze(&bad, &mut g, 11).is_err());
    assert_eq!(before, g.snapshot());
    let mut h = declared(&purge_controls()["controls"][2]["input"]);
    if !h.is_empty() {
        h[0]["bounds"] = json!([0, 0, 0, 0]);
        let before = h.clone();
        assert!(heads::purge(&mut h, true).is_err());
        assert_eq!(before, h);
    }
}
