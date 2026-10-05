// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch::{self, Group, Registry},
    stem_chunks,
    stem_parts_alignment::Parameters,
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::json;
fn glyph(y: i32) -> GlyphPixels {
    beam_dispatch::glyph_from_json(
        &json!({"bounds":[40,y,1,12],"orientation":"VERTICAL","runs":[[0,0,12]]}),
    )
    .unwrap()
}
#[test]
fn chunk_success_retains_existing_physical_allocations_groups_and_aliases() {
    let a = glyph(40);
    let b = glyph(52);
    let mut r = Registry::default();
    let id = r.register(&a).unwrap();
    r.add_group(id, Group::VerticalSeed).unwrap();
    let old = r.snapshot();
    let allocation = r.get(id).unwrap().glyph.runs.as_ptr();
    let c = stem_chunks::select(
        &mut r,
        &[a, b.clone(), b],
        &[id],
        None,
        None,
        &Parameters::new(1, 2., 44., 0.).unwrap(),
    )
    .unwrap();
    assert_eq!(c.registered_ids, vec![1, 2, 2]);
    assert_eq!(c.new_count, 1);
    assert_eq!(r.get(id).unwrap().glyph.runs.as_ptr(), allocation);
    assert_eq!(r.snapshot()[0], old[0]);
    assert_eq!(r.len(), 2);
}
#[test]
fn late_chunk_refusal_preserves_allocations_and_complete_registry() {
    let a = glyph(40);
    let b = glyph(52);
    let mut r = Registry::default();
    let id = r.register(&a).unwrap();
    r.add_group(id, Group::VerticalSeed).unwrap();
    let old = r.snapshot();
    let allocation = r.get(id).unwrap().glyph.runs.as_ptr();
    let mut invalid = a.clone();
    invalid.runs.clear();
    assert!(
        stem_chunks::select(
            &mut r,
            &[b],
            &[],
            None,
            Some((&invalid, false)),
            &Parameters::new(1, 2., 44., 0.).unwrap()
        )
        .is_err()
    );
    assert_eq!(r.snapshot(), old);
    assert_eq!(r.get(id).unwrap().glyph.runs.as_ptr(), allocation);
}
