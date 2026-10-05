// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, beam_dispatch, beam_structure, stem_checks};
use serde_json::Value;
fn cases() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-stem-glyph-grade/controls.json"
    ))
    .unwrap()
}
fn close(a: &Value, e: &Value) {
    match e {
        Value::Number(n) if n.is_f64() => assert!(
            (a.as_f64().unwrap() - n.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {e}"
        ),
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (a, e) in a.as_array().unwrap().iter().zip(v) {
                close(a, e)
            }
        }
        Value::Object(v) => {
            for (k, e) in v {
                close(&a[k], e)
            }
        }
        _ => assert_eq!(a, e),
    }
}
fn image(v: &Value) -> GrayImage {
    let g = beam_dispatch::glyph_from_json(v).unwrap();
    let mut p = vec![255; g.width * g.height];
    for r in g.runs {
        for x in r.start..r.start + r.length {
            p[r.position * g.width + x] = 0;
        }
    }
    GrayImage {
        width: g.width,
        height: g.height,
        pixels: p,
    }
}
#[test]
fn original_physical_glyph_nearline_and_grade_on_independent_rasters() {
    let g = cases();
    let cs = g["cases"].as_array().unwrap();
    assert_eq!(cs.len(), 1152);
    let (mut finite, mut rejected_nonfinite, mut accepted) = (0, 0, 0);
    for c in cs {
        let glyph = beam_dispatch::glyph_from_json(&c["glyph"]).unwrap();
        let candidate = stem_checks::candidate_from_glyph(&glyph).unwrap();
        close(&serde_json::to_value(&candidate).unwrap(), &c["candidate"]);
        let p = &c["parameters"];
        let params = stem_checks::Parameters::new(
            p["interline"].as_u64().unwrap() as usize,
            p["max_stem"].as_u64().unwrap() as usize,
            p["global_slope"].as_f64().unwrap(),
            p["profile"].as_u64().unwrap() as u8,
        )
        .unwrap();
        let result = stem_checks::check_glyph(&image(&c["image"]), &glyph, &params);
        if c["checked"]["impacts"]["grade"].is_string() {
            assert!(result.is_err());
            rejected_nonfinite += 1;
            continue;
        }
        let result = result.unwrap();
        let row = serde_json::to_value(&result).unwrap();
        let e = &c["checked"];
        for k in [
            "context_black",
            "context_white",
            "context_gap",
            "min_threshold",
        ] {
            close(&row[k], &e[k]);
        }
        close(&row["grade"], &e["impacts"]["grade"]);
        for (i, check) in result.checks.iter().enumerate() {
            let actual = serde_json::to_value(check).unwrap();
            close(&actual, &e["raw_checks"][i]);
            close(&actual, &e["impacts"]["checks"][i]);
        }
        assert_eq!(
            result.accepted,
            result.grade >= e["min_threshold"].as_f64().unwrap()
        );
        finite += 1;
        accepted += usize::from(result.accepted);
    }
    assert_eq!(finite, 996);
    assert_eq!(rejected_nonfinite, 156);
    assert!(accepted > 0);
}
#[test]
fn physical_check_boundary_and_pixel_center_are_distinct() {
    let g = cases();
    let c = &g["cases"][0];
    let mut glyph = beam_dispatch::glyph_from_json(&c["glyph"]).unwrap();
    let raw = stem_checks::candidate_from_glyph(&glyph).unwrap();
    let center = beam_structure::glyph_geometry(&glyph)
        .unwrap()
        .center_line
        .unwrap();
    assert_ne!(raw.line, center);
    assert_eq!(center[0], raw.line[0] + 0.5);
    assert_eq!(center[3], raw.line[3] + 1.);
    let img = image(&c["image"]);
    let p = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
    glyph.top = -1;
    assert!(stem_checks::check_glyph(&img, &glyph, &p).is_err());
    glyph.top = 160;
    assert!(stem_checks::check_glyph(&img, &glyph, &p).is_err());
    glyph.top = 40;
    glyph.runs[0].length = 1;
    glyph.height = 1;
    assert_eq!(
        stem_checks::candidate_from_glyph(&glyph).unwrap_err(),
        "unsupported_single_pixel_stem_check_glyph"
    );
    glyph.runs[0].length = 0;
    assert!(stem_checks::candidate_from_glyph(&glyph).is_err());
    assert!(stem_checks::Parameters::new(11, 5, 0., 4).is_err());
}
