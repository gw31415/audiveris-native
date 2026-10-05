// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, bar_sticks, clusters, filament_merge, line_completion, page_glyphs, scale, sections,
    staff_projection, system_graph, vertical,
};
use serde_json::Value;
fn close(a: &Value, b: &Value) {
    match b {
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                close(x, y);
            }
        }
        Value::Number(_) => assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-9),
        _ => assert_eq!(a, b),
    }
}
fn png_pixels(path: &std::path::Path) -> Vec<u8> {
    let mut d = png::Decoder::new(std::io::Cursor::new(std::fs::read(path).unwrap()));
    d.set_transformations(png::Transformations::EXPAND);
    let mut r = d.read_info().unwrap();
    let mut p = vec![0; r.output_buffer_size().unwrap()];
    r.next_frame(&mut p).unwrap();
    p
}
#[test]
fn original_rasters_match_official_completion_phases_and_native_removal() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["laura-gray", "synthetic-system-bars"] {
        let image = GrayImage::decode(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}.png"))).unwrap(),
        )
        .unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        let sc = scale::analyze(&binary).result.unwrap();
        let prep = sections::prepare(&binary, &sc).unwrap();
        let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
        let st = clusters::analyze(&sc, &prep.horizontal, &factory);
        let projection = staff_projection::analyze(&binary, &sc, &prep.horizontal, &st);
        let sticks = bar_sticks::analyze(&binary, &sc, &prep, &projection);
        let slope = factory["line_filtering"]["global_slope"].as_f64().unwrap();
        let systems = system_graph::analyze(&binary, &sc, &st, &sticks, slope);
        let actual = line_completion::analyze(
            &binary,
            &sc,
            &prep,
            &st,
            &projection,
            &sticks,
            &systems,
            slope,
        );
        assert_eq!(actual["grid_complete"], false);
        let expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-completion.json")))
                .unwrap(),
        )
        .unwrap();
        let a = actual["line_phases"].as_array().unwrap();
        let b = expected["line_phases"].as_array().unwrap();
        assert_eq!(a.len(), 9);
        assert_eq!(a.len(), b.len());
        for (p, q) in a.iter().zip(b) {
            for (s, t) in p["staffs"]
                .as_array()
                .unwrap()
                .iter()
                .zip(q["staffs"].as_array().unwrap())
            {
                close(&s["left"], &t["left"]);
                close(&s["right"], &t["right"]);
                for (l, m) in s["lines"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(t["lines"].as_array().unwrap())
                {
                    assert_eq!(l["section_ids"], m["section_ids"]);
                    close(&l["points"], &m["points"]);
                }
            }
        }
        if name == "laura-gray" {
            let gold: Value = serde_json::from_slice(
                &std::fs::read(
                    root.join("crates/audiveris-staff-removal/fixtures/official-staff-glyphs.json"),
                )
                .unwrap(),
            )
            .unwrap();
            let glyphs = actual["staff_line_glyphs"].as_array().unwrap();
            assert_eq!(glyphs.len(), 30);
            for (g, h) in glyphs.iter().zip(gold.as_array().unwrap()) {
                for k in ["left", "top", "width", "height", "orientation", "runs"] {
                    assert_eq!(g[k], h[k], "glyph {} {k}", g["line_index"]);
                }
            }
            let model = audiveris_classifier::Model::from_json(include_bytes!(
                "../crates/audiveris-classifier/models/audiveris-5.11.0.json"
            ))
            .unwrap();
            let (residual, report) =
                page_glyphs::extract(&binary, &actual, sc.interline.main as u32, Some(&model))
                    .unwrap();
            assert_eq!(report["removed_pixels"], 54466);
            assert_eq!(report["component_count"], 604);
            assert_eq!(
                residual.pixels,
                png_pixels(
                    &root.join("crates/audiveris-staff-removal/fixtures/official-no-staff.png")
                )
            );
            let mask = png_pixels(
                &root.join("crates/audiveris-staff-removal/fixtures/official-removed.png"),
            );
            for ((a, b), m) in binary.pixels.iter().zip(&residual.pixels).zip(mask) {
                assert_eq!(if a != b { 0 } else { 255 }, m);
            }
            for c in report["components"].as_array().unwrap() {
                assert_eq!(c["features"].as_array().unwrap().len(), 110);
                assert_eq!(c["scores"].as_array().unwrap().len(), 149);
            }
        }
    }
}
