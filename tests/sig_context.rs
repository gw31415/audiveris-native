// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::sig_context::contextualize;
use serde_json::{Value, json};
#[test]
fn original_jar_controlled_exclusion_cases() {
    let cases: Value =
        serde_json::from_str(include_str!("../fixtures/official-grid/context-cases.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let mut actual = case["inters"].as_array().unwrap().clone();
        for v in &mut actual {
            v["contextual_grade"] = Value::Null;
        }
        contextualize(&mut actual, case["relations"].as_array().unwrap()).unwrap();
        for (a, b) in actual.iter().zip(case["inters"].as_array().unwrap()) {
            assert_eq!(
                a["contextual_grade"].as_f64().unwrap().to_bits(),
                b["contextual_grade"].as_f64().unwrap().to_bits(),
                "{} inter {}",
                case["name"],
                a["id"]
            );
        }
    }
}
#[test]
fn unsupported_context_is_rejected_without_partial_writes() {
    let mut nodes: Vec<_> = (0..18)
        .map(|id| json!({"id":id,"kind":"BarlineInter","grade":0.4,"contextual_grade":null}))
        .collect();
    let mut relations: Vec<_> = (1..18)
        .map(|id| json!({"kind":"BarConnectionRelation","source":0,"target":id,"grade":0.8}))
        .collect();
    let original = nodes.clone();
    assert_eq!(
        contextualize(&mut nodes, &relations).unwrap_err(),
        "unsupported_grid_context_partner_limit_16"
    );
    assert_eq!(nodes, original);
    relations.pop();
    contextualize(&mut nodes, &relations).unwrap();
    nodes[0]["kind"] = json!("HeadInter");
    assert_eq!(
        contextualize(&mut nodes, &relations).unwrap_err(),
        "unsupported_grid_context_inter"
    );
    nodes[0]["kind"] = json!("BarlineInter");
    relations[0]["kind"] = json!("HeadStemRelation");
    assert_eq!(
        contextualize(&mut nodes, &relations).unwrap_err(),
        "unsupported_grid_context_relation"
    );
}
