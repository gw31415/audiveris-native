// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::head_candidate_filter::{aggregate_indices, filter_seed_indices};
use serde_json::{Value, json};
#[test]
fn original_all_118_live_range_candidate_aggregates() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-candidate-filter/laura.json"
    ))
    .unwrap();
    let catalog: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-range-catalog/laura.json"
    ))
    .unwrap();
    assert_eq!(oracle["native_outputs_used"], false);
    let mut scans = 0;
    let mut total = 0;
    for (s, g) in oracle["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(catalog["systems"].as_array().unwrap())
    {
        for (f, ref_f) in s["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(g["staffs"].as_array().unwrap())
        {
            for key in ["scanners", "ledger_scanners"] {
                for (c, ref_c) in f[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(ref_f[key].as_array().unwrap())
                {
                    assert_eq!(c["matches"], ref_c["matches"]);
                    let indices = aggregate_indices(c["matches"].as_array().unwrap(), 4).unwrap();
                    assert_eq!(json!(indices), c["candidate_filter"]["aggregate_indices"]);
                    assert_eq!(
                        c["candidate_filter"]["aggregate_indices"],
                        c["candidate_filter"]["filtered_indices"]
                    );
                    total += indices.len();
                    scans += 1;
                }
            }
        }
    }
    assert_eq!(scans, 118);
    assert!(total > 0 && total < 2224);
    println!("{scans} original scanners, {total} aggregates");
}
fn input(v: &Value) -> Vec<Value> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let mut h = row.clone();
            h["grade"] = json!(f64::from_bits(h["grade_bits"].as_u64().unwrap()));
            h
        })
        .collect()
}
#[test]
fn original_stable_duplicates_parallel_candidates_iou_and_grade_boundaries() {
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-candidate-filter/controls.json"
    ))
    .unwrap();
    assert_eq!(g["native_outputs_used"], false);
    for c in g["cases"].as_array().unwrap() {
        let heads = input(&c["heads"]);
        let competitors = input(&c["competitors"]);
        let a = aggregate_indices(&heads, 4).unwrap();
        assert_eq!(json!(a), c["aggregate_indices"], "{}", c["name"]);
        let selected = a.iter().map(|&i| heads[i].clone()).collect::<Vec<_>>();
        let indices = filter_seed_indices(&selected, &competitors)
            .unwrap()
            .into_iter()
            .map(|i| a[i])
            .collect::<Vec<_>>();
        assert_eq!(json!(indices), c["filtered_indices"], "{}", c["name"]);
    }
}
#[test]
fn malformed_and_unsorted_inputs_fail_without_mutation() {
    let good = json!({"kind":"HeadInter","bounds":[0,0,11,1],"grade":0.7});
    let mut bad = good.clone();
    bad["bounds"] = json!([0, 0, 0, 1]);
    assert!(aggregate_indices(&[bad], 4).is_err());
    assert!(aggregate_indices(std::slice::from_ref(&good), -1).is_err());
    let mut bad = good.clone();
    bad["grade"] = json!(1.1);
    assert!(filter_seed_indices(&[bad], &[]).is_err());
    let mut far = good.clone();
    far["bounds"] = json!([20, 0, 11, 1]);
    assert!(filter_seed_indices(&[], &[far, good.clone()]).is_err());
    assert_eq!(aggregate_indices(&[], 4).unwrap(), Vec::<usize>::new());
    assert_eq!(
        filter_seed_indices(std::slice::from_ref(&good), &[json!({"kind":"BeamInter"})]).unwrap(),
        vec![0]
    );
    assert_eq!(
        good,
        json!({"kind":"HeadInter","bounds":[0,0,11,1],"grade":0.7})
    );
}
