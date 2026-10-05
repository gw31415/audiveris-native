// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, stem_scale};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[test]
fn original_stem_scaler_matches_complete_native_pipeline() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-stem-seeds"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 12);
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap()
            .to_owned();
        println!("checking {name}");
        let expected: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
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
        let complete = line_completion::analyze(
            &binary,
            &sc,
            &prep,
            &st,
            &projection,
            &sticks,
            &systems,
            slope,
        );
        let (residual, _) =
            page_glyphs::extract(&binary, &complete, sc.interline.main as u32, None).unwrap();
        let grid = grid_ir::build(&residual, &sc, &prep, &complete, &systems, &sticks).unwrap();
        let headers = header_preparation::from_grid(&grid).unwrap();
        let candidates =
            clef_candidates::analyze(&residual, &grid, &headers, sc.interline.main).unwrap();
        let registered =
            clef_registration::analyze(&candidates, &grid, &headers, sc.interline.main).unwrap();

        let key =
            audiveris_native::key_column::analyze(&residual, &grid, &registered, sc.interline.main)
                .unwrap();
        let time = audiveris_native::time_column::analyze(
            &residual,
            &grid,
            &registered,
            &key,
            sc.interline.main,
        )
        .unwrap();
        let headers =
            audiveris_native::header_finalization::analyze(&grid, &time, sc.interline.main)
                .unwrap();
        assert_eq!(headers["headers_complete"], true);
        let before = residual.pixels.clone();
        let actual = stem_scale::analyze(&residual, &sc).unwrap();
        assert_eq!(
            residual.pixels, before,
            "{name}: StemScaler must not erase source pixels"
        );
        let hash = format!("{:x}", Sha256::digest(&residual.pixels));
        for key in [
            "no_staff_before_scaler",
            "stem_focused_mask",
            "no_staff_after_step",
        ] {
            assert_eq!(expected[key]["width"], residual.width, "{name}/{key} width");
            assert_eq!(
                expected[key]["height"], residual.height,
                "{name}/{key} height"
            );
            assert_eq!(expected[key]["sha256"], hash, "{name}/{key} native pixels");
            assert_eq!(
                expected[key]["black_count"],
                residual.pixels.iter().filter(|p| **p == 0).count(),
                "{name}/{key} black pixels"
            );
        }
        assert_eq!(expected["interline"], sc.interline.main);
        assert_eq!(expected["foreground_main"], sc.line.main);
        assert_eq!(expected["foreground_max"], sc.line.max);
        assert_eq!(
            json!(actual.histogram),
            expected["histogram"],
            "{name} histogram"
        );
        assert_eq!(json!(actual.peak), expected["peak"], "{name} chosen peak");
        assert_eq!(
            json!(actual.stem_scale),
            expected["stem_scale"],
            "{name} stem scale"
        );
        assert_eq!(actual.area, actual.histogram.values.iter().sum::<i32>());
        assert!(actual.stem_scaling_complete);
        assert!(!actual.stem_seeds_complete);
    }
}
