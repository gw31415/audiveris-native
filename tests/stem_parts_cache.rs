// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_parts_prefix_controls.rs"]
mod controls;
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    stem_cache_sig, stem_checks,
    stem_parts_cache::{self, Item},
};
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-prefix/controls.json"
    ))
    .unwrap()
}
#[test]
fn original_prefix_equality_gap_and_selected_cache_sig_plus_prior_head_composition() {
    let o = controls::observe(&fixture());
    assert_eq!(o["prefix_controls"], 4680);
    assert_eq!(o["selected_cache_controls"].as_array().unwrap().len(), 12);
    assert!(o["heads_complete"] == false && o["stems_complete"] == false);
}
#[test]
fn empty_bounds_and_invalid_late_owner_leave_registry_cache_unchanged() {
    let f = fixture();
    let g =
        beam_dispatch::glyph_from_json(&f["selected_cache_controls"][0]["selected"][0]).unwrap();
    let mut registry = Registry::default();
    let id = registry.register(&g).unwrap();
    let items = [
        Item {
            glyph_id: Some(id),
            gap: false,
        },
        Item {
            glyph_id: None,
            gap: true,
        },
        Item {
            glyph_id: Some(id),
            gap: false,
        },
    ];
    assert_eq!(
        stem_parts_cache::select_until(&registry, &items, -1).unwrap(),
        Vec::<usize>::new()
    );
    assert_eq!(
        stem_parts_cache::select_until(&registry, &items, 2).unwrap(),
        vec![id]
    );
    assert_eq!(stem_parts_cache::last_gap_before(&items, 1).unwrap(), None);
    assert_eq!(
        stem_parts_cache::last_gap_before(&items, 2).unwrap(),
        Some(1)
    );
    assert_eq!(
        stem_parts_cache::last_gap_before(&items, 3).unwrap(),
        Some(1)
    );
    assert!(stem_parts_cache::last_gap_before(&items, 4).is_err());
    assert!(stem_parts_cache::select_until(&registry, &items, 3).is_err());
    assert!(stem_parts_cache::select_until(&registry, &items, -2).is_err());
    let image = GrayImage {
        width: 896,
        height: 160,
        pixels: vec![255; 896 * 160],
    };
    let p = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
    let mut retriever = stem_cache_sig::Retriever::new(1).unwrap();
    let before = format!("{registry:?}");
    let bad = [
        items[0],
        Item {
            glyph_id: Some(0),
            gap: false,
        },
    ];
    assert!(
        stem_parts_cache::create_until(&mut retriever, &mut registry, &bad, 0, &image, &p).is_err()
    );
    assert_eq!(format!("{registry:?}"), before);
    assert!(
        stem_parts_cache::create_until(&mut retriever, &mut registry, &[], -1, &image, &p).is_err()
    );
    assert_eq!(format!("{registry:?}"), before);
    assert!(stem_parts_cache::select_until(&registry, &vec![items[0]; 4097], 0).is_err());
    let mut painted = GrayImage {
        width: image.width,
        height: image.height,
        pixels: image.pixels.clone(),
    };
    for r in &g.runs {
        for y in r.start..r.start + r.length {
            painted.pixels[(g.top as usize + y) * 896 + g.left as usize + r.position] = 0;
        }
    }
    let accepted =
        stem_parts_cache::create_until(&mut retriever, &mut registry, &items, 0, &painted, &p)
            .unwrap()
            .stem
            .unwrap();
    let saved = accepted.snapshot().unwrap();
    let mut foreign = Registry::default();
    let mut conflicting = g.clone();
    conflicting.left += 1;
    let foreign_id = foreign.register(&conflicting).unwrap();
    assert!(
        stem_parts_cache::create_until(
            &mut retriever,
            &mut foreign,
            &[Item {
                glyph_id: Some(foreign_id),
                gap: false
            }],
            0,
            &painted,
            &p
        )
        .is_err()
    );
    assert_eq!(accepted.snapshot().unwrap(), saved);
    let mut rejected_registry = Registry::default();
    let compound = f["selected_cache_controls"][1]["selected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| Item {
            glyph_id: Some(
                rejected_registry
                    .register(&beam_dispatch::glyph_from_json(v).unwrap())
                    .unwrap(),
            ),
            gap: false,
        })
        .collect::<Vec<_>>();
    let mut rejected_cache = stem_cache_sig::Retriever::new(1).unwrap();
    let rejected = stem_parts_cache::create_until(
        &mut rejected_cache,
        &mut rejected_registry,
        &compound,
        1,
        &image,
        &p,
    )
    .unwrap();
    assert!(rejected.stem.is_none());
    assert!(rejected_registry.get(rejected.glyph_id).is_ok());
    assert!(
        compound
            .iter()
            .all(|i| i.glyph_id != Some(rejected.glyph_id))
    );
}
#[test]
fn identical_pixels_different_run_orientation_preserve_first_physical_occurrence() {
    let f = fixture();
    let values = &f["orientation_groups"][0]["cases"][0]["items"];
    let mut registry = Registry::default();
    let items = values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| Item {
            glyph_id: Some(
                registry
                    .register(&beam_dispatch::glyph_from_json(&v["glyph"]).unwrap())
                    .unwrap(),
            ),
            gap: false,
        })
        .collect::<Vec<_>>();
    assert_eq!(items[0].glyph_id, items[1].glyph_id);
    assert_eq!(items[2].glyph_id, items[3].glyph_id);
    assert_ne!(items[0].glyph_id, items[2].glyph_id);
    assert_eq!(
        stem_parts_cache::select_until(&registry, &items, 3).unwrap(),
        vec![items[0].glyph_id.unwrap(), items[2].glyph_id.unwrap()]
    );
}
