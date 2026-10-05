// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_chunks_controls.rs"]
mod controls;
use audiveris_native::{
    beam_dispatch::{self, Group, Registry},
    stem_chunks,
    stem_parts_alignment::Parameters,
};
use serde_json::json;
#[test]
fn original_complete_lookup_chunks_controls() {
    let o = serde_json::from_slice(include_bytes!(
        "../fixtures/official-stem-chunks/controls.json"
    ))
    .unwrap();
    let _ = controls::observe(&o);
}
#[test]
fn atomic_refusal_alias_stump_and_stable_directional_order() {
    let a = beam_dispatch::glyph_from_json(
        &json!({"bounds":[4,10,1,5],"orientation":"VERTICAL","runs":[[0,0,5]]}),
    )
    .unwrap();
    let mut b = a.clone();
    b.left = 5;
    let mut r = Registry::default();
    let id = r.register(&a).unwrap();
    r.add_group(id, Group::VerticalSeed).unwrap();
    let p = Parameters::new(1, 100., 100., 0.).unwrap();
    let out = stem_chunks::select(
        &mut r,
        &[a.clone(), b.clone(), a.clone()],
        &[],
        Some(id),
        None,
        &p,
    )
    .unwrap();
    assert_eq!(out.registered_ids, vec![1, 2, 1]);
    assert_eq!(out.retained_ids, vec![2, 1]);
    assert!(r.get(id).unwrap().groups.contains(&Group::VerticalSeed));
    let before = format!("{r:?}");
    let mut bad = a.clone();
    bad.runs.clear();
    assert!(
        stem_chunks::select(
            &mut r,
            std::slice::from_ref(&b),
            &[],
            Some(id),
            Some((&bad, false)),
            &p
        )
        .is_err()
    );
    assert_eq!(format!("{r:?}"), before);
    assert!(stem_chunks::select(&mut r, &[], &[999], None, None, &p).is_err());
    assert!(stem_chunks::select(&mut r, &[], &[], Some(999), None, &p).is_err());
    assert_eq!(format!("{r:?}"), before);
    let out = stem_chunks::select(
        &mut r,
        &[a.clone(), a.clone()],
        &[id, id],
        Some(id),
        None,
        &p,
    )
    .unwrap();
    assert!(out.retained_ids.is_empty());
    assert_eq!(r.len(), 2);
    let mut c = a.clone();
    c.top = 20;
    let rev = Parameters::new(-1, 100., 100., 0.).unwrap();
    let out = stem_chunks::select(&mut r, &[b, c, a], &[], None, None, &rev).unwrap();
    assert_eq!(out.retained_ids, vec![3, 2, 1]);
    assert!(
        stem_chunks::select(
            &mut r,
            &[],
            &[],
            None,
            None,
            &Parameters {
                direction: 1,
                max_dx: 1.,
                max_dy: 1.,
                slope: 0.1
            }
        )
        .is_err()
    );
    assert!(
        stem_chunks::select(&mut r, &[], &[], None, None, &p)
            .unwrap()
            .retained_ids
            .is_empty()
    );
}
