// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
#[test]
fn late_head_anchor_failure_keeps_actual_cache_geometry_and_manual_relation_identity() {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-image-musicxml-parallel-voices/parallel-quarters.png"
    ))
    .unwrap();
    let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
    let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
    let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
    let mut owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
    let heads = owner.graph.live_inter_keys(1, "HeadInter");
    let bad = *heads.last().unwrap();
    let id = owner.graph.id(bad).unwrap();
    owner.graph.vertices[id - 1].data["shape"] = json!("UNKNOWN_HEAD_ANCHOR");
    for relation in &mut owner.graph.edges {
        if relation.data["kind"] == "HeadStemRelation" {
            relation.data["manual"] = json!(true);
        }
    }
    let before = owner.graph.snapshot();
    let physical = owner.prepared.registry.snapshot();
    let held = owner
        .graph
        .live_inter_keys(1, "StemInter")
        .into_iter()
        .map(|key| {
            let object = owner.caches[&1]
                .registered_stem(&owner.graph, key.local)
                .unwrap();
            let state = object.snapshot().unwrap();
            (key, object, state)
        })
        .collect::<Vec<_>>();
    assert!(held.len() > 1);
    assert_eq!(
        crate::stem_head_refinement::apply(&mut owner).unwrap_err(),
        "unsupported_head_stem_shape"
    );
    assert_eq!(owner.graph.snapshot(), before);
    assert_eq!(owner.prepared.registry.snapshot(), physical);
    for (key, object, state) in held {
        assert_eq!(object.snapshot().unwrap(), state);
        assert!(std::sync::Arc::ptr_eq(
            &object,
            &owner.caches[&1]
                .registered_stem(&owner.graph, key.local)
                .unwrap()
        ));
    }
}
