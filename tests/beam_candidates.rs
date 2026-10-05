// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{beam_dispatch, beam_structure};
use serde_json::Value;
#[test]
fn original_glyph_metrics_match_all_thirteen_raster_diagnostics() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut count = 0;
    for entry in std::fs::read_dir(root.join("fixtures/official-beam-prolog")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let original: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        for row in original["replayed_beam_structure_diagnostics"]
            .as_array()
            .unwrap()
        {
            let glyph = beam_dispatch::glyph_from_json(&row["glyph"]).unwrap();
            let got = beam_structure::glyph_geometry(&glyph).unwrap();
            count += 1;
            assert_eq!(got.mean_thickness, row["mean_thickness"].as_f64().unwrap());
            let actual = got.center_line.unwrap();
            for (a, b) in actual
                .into_iter()
                .zip(row["center_line"].as_array().unwrap())
            {
                assert!(
                    (a - b.as_f64().unwrap()).abs() <= 1e-8,
                    "{} center line {a} != {b}",
                    path.display()
                );
            }
        }
    }
    assert_eq!(count, 483);
}
#[test]
fn singleton_geometry_is_explicitly_undefined_and_bounds_are_checked() {
    use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
    let mut g = GlyphPixels {
        left: 10,
        top: 10,
        width: 1,
        height: 1,
        orientation: Orientation::Vertical,
        runs: vec![Run {
            position: 0,
            start: 0,
            length: 1,
        }],
    };
    let got = beam_structure::glyph_geometry(&g).unwrap();
    assert_eq!(got.mean_thickness, 1.);
    assert!(got.center_line.is_none());
    g.left = 16384;
    assert!(beam_structure::glyph_geometry(&g).is_err());
}
