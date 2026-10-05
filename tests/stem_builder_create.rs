// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    stem_builder, stem_checks, stem_glyph,
};
use serde_json::{Value, json};
use std::sync::Arc;
fn observe(f: &Value) -> Value {
    let mut registry = Registry::default();
    let mut cache = stem_builder::Cache::new(1).unwrap();
    let mut cases = Vec::new();
    for c in f["cases"].as_array().unwrap() {
        let parts = c["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
            .collect::<Vec<_>>();
        let painted = stem_glyph::assemble(&parts).unwrap();
        let mut image = GrayImage {
            width: f["image_width"].as_u64().unwrap() as usize,
            height: f["image_height"].as_u64().unwrap() as usize,
            pixels: vec![
                255;
                (f["image_width"].as_u64().unwrap() * f["image_height"].as_u64().unwrap())
                    as usize
            ],
        };
        for r in &painted.runs {
            for y in r.start..r.start + r.length {
                image.pixels[(painted.top as usize + y) * image.width
                    + painted.left as usize
                    + r.position] = 0;
            }
        }
        let profile = c["profile"].as_u64().unwrap() as u8;
        let params = stem_checks::Parameters::new(
            f["interline"].as_u64().unwrap() as usize,
            f["max_stem"].as_u64().unwrap() as usize,
            f["global_slope"].as_f64().unwrap(),
            profile,
        )
        .unwrap();
        let before = registry.len();
        let cache_before = cache.len();
        let first = cache
            .create_selected(&mut registry, &parts, &image, &params)
            .unwrap();
        let glyph = registry.get(first.glyph_id).unwrap().glyph.clone();
        let first_json = first.stem.as_ref().map(|s| s.record.clone());
        let delta = registry.len() - before;
        let cache_delta = cache.len() - cache_before;
        image.pixels.fill(255);
        let mut other = params.clone();
        other.profile = 1 - profile;
        let second = cache
            .create_selected(&mut registry, &parts, &image, &other)
            .unwrap();
        assert_eq!(first.glyph_id, second.glyph_id);
        let same = match (&first.stem, &second.stem) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        let regrade = stem_checks::check_glyph(&image, &glyph, &other)
            .unwrap()
            .grade;
        cases.push(json!({"profile":profile,"parts":c["parts"],"registered":beam_dispatch::glyph_json(&glyph),"registered_delta":delta,"cache_delta":cache_delta,"first":first_json,"second":second.stem.as_ref().map(|s|&s.record),"cache_reused_same_stem":same,"second_white_image_if_regraded":regrade}));
    }
    json!({"builder_min_grade":stem_builder::MIN_GRADE,"cases":cases})
}
fn near(a: &Value, b: &Value) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                near(&a[k], v)
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                near(x, y)
            }
        }
        Value::Number(_) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {b}"
        ),
        _ => assert_eq!(a, b),
    }
}
#[test]
fn original_selected_stem_builder_144_grade_registration_orientation_and_cache_controls() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-builder-create/controls.json"
    ))
    .unwrap();
    let actual = observe(&f);
    assert_eq!(actual["builder_min_grade"], f["builder_min_grade"]);
    let mut comparable = f["cases"].clone();
    for c in comparable.as_array_mut().unwrap() {
        c.as_object_mut()
            .unwrap()
            .remove("SIG_unchanged_by_createStem");
    }
    near(&actual["cases"], &comparable);
    assert_eq!(f["cases"].as_array().unwrap().len(), 144);
    assert_eq!(
        f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| !c["first"].is_null())
            .count(),
        96
    );
    assert!(
        f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["SIG_unchanged_by_createStem"] == true
                && c["second_white_image_if_regraded"].as_f64() == Some(0.0))
    );
}
#[test]
fn selected_builder_refuses_invalid_inputs_preserves_groups_and_keeps_cache_per_system() {
    use audiveris_native::beam_dispatch::Group;
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-builder-create/controls.json"
    ))
    .unwrap();
    let c = f["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            !c["first"].is_null()
                && c["parts"].as_array().unwrap().len() == 1
                && c["parts"][0]["orientation"] == "HORIZONTAL"
        })
        .unwrap();
    let glyph = beam_dispatch::glyph_from_json(&c["parts"][0]).unwrap();
    let parts = vec![glyph.clone()];
    let mut image = GrayImage {
        width: 896,
        height: 160,
        pixels: vec![255; 896 * 160],
    };
    let drawn = stem_glyph::assemble(&parts).unwrap();
    for r in &drawn.runs {
        for y in r.start..r.start + r.length {
            image.pixels[(drawn.top as usize + y) * 896 + drawn.left as usize + r.position] = 0;
        }
    }
    let params = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
    let mut registry = Registry::default();
    let mut cache = stem_builder::Cache::new(1).unwrap();
    assert!(stem_builder::Cache::new(0).is_err());
    let before = registry.snapshot();
    assert!(
        cache
            .create_selected(&mut registry, &[], &image, &params)
            .is_err()
    );
    let mut malformed = glyph.clone();
    malformed.width = 0;
    assert!(
        cache
            .create_selected(&mut registry, &[malformed], &image, &params)
            .is_err()
    );
    let mut invalid = params.clone();
    invalid.profile = 4;
    assert!(
        cache
            .create_selected(&mut registry, &parts, &image, &invalid)
            .is_err()
    );
    let invalid_image = GrayImage {
        width: image.width,
        height: image.height,
        pixels: Vec::new(),
    };
    assert!(
        cache
            .create_selected(&mut registry, &parts, &invalid_image, &params)
            .is_err()
    );
    assert_eq!(registry.snapshot(), before);
    assert!(cache.is_empty());
    let id = registry.register(&glyph).unwrap();
    registry.add_group(id, Group::VerticalSeed).unwrap();
    let a = cache
        .create_selected(&mut registry, &parts, &image, &params)
        .unwrap();
    assert_eq!(a.glyph_id, id);
    assert_eq!(
        registry.get(id).unwrap().glyph.orientation,
        glyph.orientation
    );
    assert!(
        registry
            .get(id)
            .unwrap()
            .groups
            .contains(&Group::VerticalSeed)
    );
    image.pixels.fill(255);
    let b = cache
        .create_selected(&mut registry, &parts, &image, &params)
        .unwrap();
    assert!(b.cache_hit);
    assert!(Arc::ptr_eq(
        a.stem.as_ref().unwrap(),
        b.stem.as_ref().unwrap()
    ));
    let mut alien_registry = Registry::default();
    let mut alien = glyph.clone();
    alien.left += 1;
    assert_eq!(
        cache
            .create_selected(&mut alien_registry, &[alien], &image, &params)
            .unwrap_err(),
        "stem_builder_cache_registry_mismatch"
    );
    assert_eq!(cache.len(), 1);
    let mut other = stem_builder::Cache::new(2).unwrap();
    let rejected = other
        .create_selected(&mut registry, &parts, &image, &params)
        .unwrap();
    assert!(rejected.stem.is_none() && !rejected.cache_hit);
    assert!(other.is_empty());
    assert_eq!(other.system_id(), 2);
    assert_eq!(cache.len(), 1);
    assert_eq!(registry.len(), 1);
}
