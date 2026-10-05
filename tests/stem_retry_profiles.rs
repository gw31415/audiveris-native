// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, beam_dispatch, stem_checks};
use audiveris_staff_removal::Orientation;
use serde_json::Value;
fn raster(value: &Value) -> GrayImage {
    let g = beam_dispatch::glyph_from_json(value).unwrap();
    let mut image = GrayImage::new(g.width, g.height, vec![255; g.width * g.height]).unwrap();
    for r in &g.runs {
        for k in r.start..r.start + r.length {
            let (x, y) = if g.orientation == Orientation::Vertical {
                (g.left as usize + r.position, g.top as usize + k)
            } else {
                (g.left as usize + k, g.top as usize + r.position)
            };
            image.pixels[y * image.width + x] = 0;
        }
    }
    image
}

#[test]
fn original_actual_image_stem_glyphs_match_all_checks_at_retry_profiles() {
    let o: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-retry-profiles/controls.json"
    ))
    .unwrap();
    assert_eq!(o["cases"].as_array().unwrap().len(), 84);
    for c in o["cases"].as_array().unwrap() {
        let image = raster(if c["no_staff_image"].is_null() {
            &o["no_staff_image"]
        } else {
            &c["no_staff_image"]
        });
        let g = beam_dispatch::glyph_from_json(&c["glyph"]).unwrap();
        let p = &c["parameters"];
        let params = stem_checks::Parameters::new(
            p["interline"].as_u64().unwrap() as usize,
            p["max_stem"].as_u64().unwrap() as usize,
            p["global_slope"].as_f64().unwrap(),
            p["profile"].as_u64().unwrap() as u8,
        )
        .unwrap();
        let n = stem_checks::check_glyph(&image, &g, &params).unwrap();
        let expected = &c["checked"];
        assert_eq!(
            (n.context_black, n.context_white, n.context_gap),
            (
                expected["context_black"].as_u64().unwrap() as usize,
                expected["context_white"].as_u64().unwrap() as usize,
                expected["context_gap"].as_u64().unwrap() as usize
            )
        );
        assert!((n.grade - expected["grade"].as_f64().unwrap()).abs() <= 1e-12);
        assert_eq!(n.min_threshold, expected["min_threshold"].as_f64().unwrap());
        for (a, b) in n.checks.iter().zip(expected["checks"].as_array().unwrap()) {
            assert_eq!(a.name, b["name"].as_str().unwrap());
            assert_eq!(a.covariant, b["covariant"].as_bool().unwrap());
            for (actual, key) in [
                (a.raw_value, "raw_value"),
                (a.low, "low"),
                (a.high, "high"),
                (a.weight, "weight"),
                (a.impact, "impact"),
            ] {
                assert!(
                    (actual - b[key].as_f64().unwrap()).abs() <= 1e-12,
                    "profile{} {} {}: {} != {}",
                    params.profile,
                    a.name,
                    key,
                    actual,
                    b[key]
                );
            }
        }
    }
    assert!(stem_checks::Parameters::new(11, 5, 0., 4).is_err());
}
