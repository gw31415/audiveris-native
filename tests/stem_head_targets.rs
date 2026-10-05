// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_head_targets_controls.rs"]
mod controls;
use audiveris_native::stem_head_targets::{Beam, Catalog, Group};
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-head-targets/controls.json"
    ))
    .unwrap()
}
#[test]
fn original_actual_neighborhood_targets_small_heads_and_ulp_distances() {
    let f = fixture();
    let o = controls::observe(&f);
    assert_eq!(
        o["cases"].as_array().unwrap().len(),
        f["head_builders"].as_u64().unwrap() as usize * 5
    );
    assert!(o["heads_complete"] == false && o["stems_complete"] == false);
}
#[test]
fn catalog_and_late_candidate_refusals_are_read_only() {
    let f = fixture();
    let mut beams: Vec<Beam> = serde_json::from_value(f["beams"].clone()).unwrap();
    let mut groups: Vec<Group> = serde_json::from_value(f["groups"].clone()).unwrap();
    let c = Catalog::new(&beams, &groups).unwrap();
    let mut r = controls::request(&f["cases"][0]);
    let before = serde_json::to_value(c.select(&r).unwrap()).unwrap();
    r.candidates.push(usize::MAX);
    assert!(c.select(&r).is_err());
    r.candidates.pop();
    r.head.slope = 0.01;
    assert!(c.select(&r).is_err());
    r.head.slope = 0.;
    assert_eq!(serde_json::to_value(c.select(&r).unwrap()).unwrap(), before);
    assert!(c.lookup([0., 0.], 1, 0, &vec![1; 4097], 0.).is_err());
    assert!(c.lookup([f64::NAN, 0.], 1, 0, &[], 0.).is_err());
    assert!(c.lookup([0., 0.], 0, 0, &[], 0.).is_err());
    assert!(c.lookup([0., 0.], 1, -1, &[], 0.).is_err());
    drop(c);
    let saved = groups[0].members.clone();
    groups[0].members.clear();
    assert!(Catalog::new(&beams, &groups).is_err());
    groups[0].members = saved;
    beams[0].top[2] = beams[0].top[0];
    assert!(Catalog::new(&beams, &groups).is_err());
}
