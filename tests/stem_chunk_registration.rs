// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_chunk_registration_controls.rs"]
mod controls;
use audiveris_native::{
    beam_dispatch::{self, Group, Registry},
    stem_chunk_registration,
};
use serde_json::json;
#[test]
fn original_actual_raw_chunk_registration_alias_and_existing_removal_prefix_matches() {
    let original = serde_json::from_str(include_str!(
        "../fixtures/official-stem-chunk-registration/controls.json"
    ))
    .unwrap();
    let result = controls::observe(&original);
    let groups = result["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 612);
    assert_eq!(
        groups
            .iter()
            .map(|g| g["cases"].as_array().unwrap().len())
            .sum::<usize>(),
        2448
    );
    assert!(
        groups
            .iter()
            .all(|g| g["cases"][2]["retained_ids"].as_array().unwrap().is_empty())
    );
}
#[test]
fn register_before_removal_keeps_groups_aliases_and_refuses_invalid_late_input() {
    let v = beam_dispatch::glyph_from_json(
        &json!({"bounds":[10,10,2,5],"orientation":"VERTICAL","runs":[[0,0,5],[1,0,5]]}),
    )
    .unwrap();
    let h=beam_dispatch::glyph_from_json(&json!({"bounds":[10,10,2,5],"orientation":"HORIZONTAL","runs":[[0,0,2],[1,0,2],[2,0,2],[3,0,2],[4,0,2]]})).unwrap();
    let mut r = Registry::default();
    let id = r.register(&v).unwrap();
    r.add_group(id, Group::VerticalSeed).unwrap();
    let result = stem_chunk_registration::register_and_exclude(
        &mut r,
        &[v.clone(), v.clone(), h],
        &[id, id],
    )
    .unwrap();
    assert_eq!(result.registered_ids, vec![1, 1, 2]);
    assert_eq!(result.retained_ids, vec![2]);
    assert_eq!(result.new_count, 1);
    assert_eq!(r.len(), 2);
    assert!(r.get(id).unwrap().groups.contains(&Group::VerticalSeed));
    assert_eq!(
        beam_dispatch::glyph_json(&r.get(id).unwrap().glyph),
        beam_dispatch::glyph_json(&v)
    );
    let before = format!("{r:?}");
    let mut bad = v.clone();
    bad.runs[0].length = 0;
    assert!(stem_chunk_registration::register_and_exclude(&mut r, &[v.clone(), bad], &[]).is_err());
    assert!(
        stem_chunk_registration::register_and_exclude(&mut r, std::slice::from_ref(&v), &[999])
            .is_err()
    );
    assert!(
        stem_chunk_registration::register_and_exclude(&mut r, &vec![v.clone(); 4097], &[]).is_err()
    );
    let large = beam_dispatch::glyph_from_json(
        &json!({"bounds":[0,0,1,1_000_000],"orientation":"VERTICAL","runs":[[0,0,1_000_000]]}),
    )
    .unwrap();
    assert!(stem_chunk_registration::register_and_exclude(&mut r, &vec![large; 9], &[]).is_err());
    assert_eq!(format!("{r:?}"), before);
    let empty = stem_chunk_registration::register_and_exclude(&mut r, &[], &[id]).unwrap();
    assert!(empty.registered_ids.is_empty());
    assert!(empty.retained_ids.is_empty());
    assert_eq!(empty.registry_count, 2);
    assert_eq!(empty.new_count, 0);
}
