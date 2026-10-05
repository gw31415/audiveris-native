// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch, head_stem_relation,
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_cleaner, stem_glyph,
};
use serde_json::{Value, json};
pub fn observe(f: &Value) -> Value {
    let mut cases = Vec::new();
    for c in f["cases"].as_array().unwrap() {
        let mut graph = Graph::default();
        graph.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"},{"id":2,"kind":"BarlineInter"}],"relations":[{"kind":"BarGroupRelation","source":1,"target":2}]},{"id":2,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}})).unwrap();
        let heads = vec![
            json!({"id":1,"kind":"HeadInter","shape":c["shape"],"bounds":c["head_bounds"],"grade":0.8,"pitch":0.,"staff_id":1,"abnormal":true,"removed":false,"glyph":null}),
            json!({"id":2,"kind":"HeadInter","shape":"WHOLE_NOTE","bounds":[100,100,20,12],"grade":0.8,"pitch":0.,"staff_id":1,"abnormal":false,"removed":false,"glyph":null}),
        ];
        graph
            .sync_heads(
                1,
                &heads,
                &[json!({"kind":"Exclusion","source":1,"target":2})],
                &[],
            )
            .unwrap();
        let mut records = Vec::new();
        let mut attachments = Vec::new();
        for (j, s) in c["stems"].as_array().unwrap().iter().enumerate() {
            let glyph = beam_dispatch::glyph_from_json(&s["parts"][0]).unwrap();
            let record = stem_glyph::record(j + 1, &glyph, s["grade"].as_f64().unwrap()).unwrap();
            assert_eq!(record["median"], s["median"]);
            assert_eq!(record["bounds"], s["bounds"]);
            assert_eq!(record["width"], s["width"]);
            records.push(record);
            if !s["relation"].is_null() {
                let mut r = s["relation"].clone();
                r["kind"] = json!("HeadStemRelation");
                r["profile"] = json!(0);
                r["manual"] = json!(false);
                let expected = head_stem_relation::gap_grade(
                    r["dx"].as_f64().unwrap(),
                    r["dy"].as_f64().unwrap(),
                    0,
                )
                .unwrap();
                assert!(
                    (expected["grade"].as_f64().unwrap() - r["grade"].as_f64().unwrap()).abs()
                        < 1e-12
                );
                attachments.push(HeadStemAttachment {
                    local: j + 1,
                    head: Key {
                        system: 1,
                        phase: Phase::Head,
                        local: 1,
                    },
                    stem_local: j + 1,
                    relation: r,
                });
            }
        }
        graph.sync_stems(1, &records, &attachments).unwrap();
        let before = graph.snapshot();
        let prior_vertices = before["inters"].as_array().unwrap().len();
        let prior_edges = before["relations"].as_array().unwrap().len();
        let order = c["order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| Key {
                system: 1,
                phase: Phase::Stem,
                local: i.as_u64().unwrap() as usize + 1,
            })
            .collect::<Vec<_>>();
        let outcome = stem_cleaner::clean_head(
            &mut graph,
            Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            },
            &order,
        )
        .unwrap();
        assert_eq!(outcome["stems_complete"], false);
        let after = graph.snapshot();
        let hid = graph
            .id(Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            })
            .unwrap();
        let by_id = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (v["id"].as_u64().unwrap() as usize, v))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut kept = Vec::new();
        let mut abnormal = Vec::new();
        let mut removed = Vec::new();
        for j in 0..records.len() {
            let id = graph
                .id(Key {
                    system: 1,
                    phase: Phase::Stem,
                    local: j + 1,
                })
                .unwrap();
            if after["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "HeadStemRelation" && e["source"] == hid && e["target"] == id)
            {
                kept.push(j)
            }
            abnormal.push(by_id[&id]["abnormal"].clone());
            removed.push(by_id[&id]["removed"].clone());
        }
        // All vertices except callback abnormal fields and all non-head-stem edges
        // retain their prior identities, data and order, including another system.
        for v in before["inters"].as_array().unwrap() {
            let mut a = v.clone();
            let mut b = by_id[&(v["id"].as_u64().unwrap() as usize)].clone();
            a.as_object_mut().unwrap().remove("abnormal");
            b.as_object_mut().unwrap().remove("abnormal");
            assert_eq!(a, b);
        }
        let other = |snapshot: &Value| {
            snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] != "HeadStemRelation")
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(other(&before), other(&after));
        let mut row = c.clone();
        row["kept"] = json!(kept);
        row["head_abnormal"] = by_id[&hid]["abnormal"].clone();
        row["stem_abnormal"] = json!(abnormal);
        row["stem_removed"] = json!(removed);
        row["vertex_delta"] =
            json!(after["inters"].as_array().unwrap().len() as i64 - prior_vertices as i64);
        row["edge_delta"] =
            json!(after["relations"].as_array().unwrap().len() as i64 - prior_edges as i64);
        cases.push(row);
    }
    json!({"cases":cases})
}
