// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, time_candidates, time_preparation};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::{Value, json};
#[test]
fn time_preparation_and_provisional_candidates_match_original() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-grid"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with("-time-column.json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 71);
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix("-time-column.json")
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

        let column =
            audiveris_native::key_column::analyze(&residual, &grid, &registered, sc.interline.main)
                .unwrap();
        let inputs =
            time_preparation::after_key_staffs(&grid, &registered, &column, sc.interline.main)
                .unwrap();
        for system in expected["systems"].as_array().unwrap() {
            let initial = &system["chronology"][0]["state"]["staffs"];
            for before in initial.as_array().unwrap() {
                let sid = before["staff_id"].as_u64().unwrap();
                let input = inputs.iter().find(|s| s.staff_id == sid).unwrap().clone();
                assert_eq!(
                    input.header_stop,
                    before["header_stop"].as_i64().unwrap(),
                    "{name}/{sid} native post-key stop"
                );
                let p =
                    time_preparation::prepare_staff(&residual, sc.interline.main, &input).unwrap();
                let b = &before["builder"];
                assert_eq!(json!(p.roi), b["roi"], "{name}/{sid} roi");
                assert_eq!(
                    json!(p.projection),
                    b["projection"]["values"],
                    "{name}/{sid} projection"
                );
                assert_eq!(
                    json!(
                        p.spaces
                            .iter()
                            .map(|s| [s.start, s.stop])
                            .collect::<Vec<_>>()
                    ),
                    b["spaces"],
                    "{name}/{sid} spaces"
                );
                let Some(stage) =
                    system["chronology"].as_array().unwrap().iter().find(|s| {
                        s["operation"] == "before_filterCandidates" && s["staff_id"] == sid
                    })
                else {
                    continue;
                };
                let built = stage["state"]["staffs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s["staff_id"] == sid)
                    .unwrap();
                let builder = &built["builder"];
                assert_eq!(
                    json!(p.start),
                    builder["range"]["start"],
                    "{name}/{sid} start"
                );
                assert_eq!(json!(p.stop), builder["range"]["stop"], "{name}/{sid} stop");
                let actual =
                    time_candidates::find_candidates(&residual, sc.interline.main, &input).unwrap();
                assert_eq!(
                    actual["adapters"].as_array().unwrap().len(),
                    builder["adapters"].as_array().unwrap().len()
                );
                for (a, b) in actual["adapters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(builder["adapters"].as_array().unwrap())
                {
                    assert_eq!(a["kind"], b["kind"]);
                    assert_eq!(a["rectangle"], b["search_rect"]);
                    assert_eq!(a["parts"], b["parts"], "{name}/{sid}/{} parts", a["kind"]);
                    assert_eq!(
                        a["trials"], b["trials"],
                        "{name}/{sid}/{} trials",
                        a["kind"]
                    );
                    assert_eq!(
                        a["best"].as_array().unwrap().len(),
                        b["best_per_shape"].as_array().unwrap().len()
                    );
                    for (x, y) in a["best"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(b["best_per_shape"].as_array().unwrap())
                    {
                        assert_eq!(x["shape"], y["shape"]);
                        assert_eq!(x["glyph"], y["inter"]["glyph"]);
                        assert_eq!(x["bounds"], y["inter"]["bounds"]);
                        assert!(
                            (x["grade"].as_f64().unwrap() - y["inter"]["grade"].as_f64().unwrap())
                                .abs()
                                < 1e-12
                        );
                    }
                }
            }
        }
    }
}
