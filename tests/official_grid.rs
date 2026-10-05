// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, bar_sticks, clusters, filament_merge, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
use std::path::Path;
fn read(root: &Path, name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
}
fn compare(a: &Value, b: &Value) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                compare(&a[k], v)
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                compare(x, y);
            }
        }
        Value::Number(n) => {
            let y = n.as_f64().unwrap();
            let x = a.as_f64().unwrap();
            assert!((x - y).abs() <= 1e-9 + 1e-12 * y.abs(), "{x} != {y}");
        }
        _ => assert_eq!(a, b),
    }
}
#[test]
fn original_raster_to_initial_staff_peaks_sticks_and_systems_matches_official_jar() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-grid");
    let golden = read(&root, "golden.json");
    for name in [
        "laura-gray",
        "synthetic-system-bars",
        "synthetic-two-staves",
    ] {
        let image =
            GrayImage::decode(&std::fs::read(root.join(format!("{name}.png"))).unwrap()).unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        let expected =
            GrayImage::decode(&std::fs::read(root.join(format!("{name}-binary.png"))).unwrap())
                .unwrap();
        assert_eq!(binary.pixels, expected.pixels);
        let sc = scale::analyze(&binary).result.unwrap();
        let reference = golden["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["case"] == name && c["target"] == "SCALE")
            .unwrap();
        for (k, r) in [("line", sc.line), ("interline", sc.interline)] {
            for (field, v) in [("min", r.min), ("main", r.main), ("max", r.max)] {
                assert_eq!(
                    v,
                    reference["scale"][k][field]
                        .as_str()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap()
                );
            }
        }
        assert_eq!(
            sc.beam.thickness,
            reference["scale"]["beam"]["main-thickness"]
                .as_str()
                .unwrap()
                .parse::<usize>()
                .unwrap()
        );
        let prep = sections::prepare(&binary, &sc).unwrap();
        let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
        let staff = clusters::analyze(&sc, &prep.horizontal, &factory);
        let official = read(&root, &format!("{name}-projection.json"));
        assert_eq!(
            staff["staffs"].as_array().unwrap().len(),
            official.as_array().unwrap().len()
        );
        for (a, b) in staff["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(official.as_array().unwrap())
        {
            assert_eq!(a["id"], b["staff_id"]);
            compare(&a["left"], &b["left"]);
            compare(&a["right"], &b["right"]);
            for (x, y) in a["lines"]
                .as_array()
                .unwrap()
                .iter()
                .zip(b["lines"].as_array().unwrap())
            {
                compare(&x["points"], &y["points"]);
            }
        }
        let projections = staff_projection::analyze(&binary, &sc, &prep.horizontal, &staff);
        for (a, b) in projections["staff_projections"]
            .as_array()
            .unwrap()
            .iter()
            .zip(official.as_array().unwrap())
        {
            for k in [
                "projection",
                "derivative_threshold",
                "blank_threshold",
                "lines_threshold",
                "chunk_threshold",
                "blanks",
                "ending_blanks",
                "peaks",
                "peak_grades",
            ] {
                compare(&a[k], &b[k]);
            }
            for (x, y) in a["line_thicknesses"]
                .as_array()
                .unwrap()
                .iter()
                .zip(b["lines"].as_array().unwrap())
            {
                compare(x, &y["thickness"]);
            }
        }
        let sticks = bar_sticks::analyze(&binary, &sc, &prep, &projections);
        compare(
            &sticks["staffs"],
            &read(&root, &format!("{name}-sticks.json")),
        );
        let systems = system_graph::analyze(
            &binary,
            &sc,
            &staff,
            &sticks,
            factory["line_filtering"]["global_slope"]
                .as_f64()
                .unwrap_or(0.),
        );
        compare(&systems, &read(&root, &format!("{name}-graph.json")));
        assert_eq!(systems["grid_complete"], false);
        if name == "synthetic-two-staves" {
            assert_eq!(systems["status"], "failed");
            assert_eq!(systems["error"], "no_system_found");
        } else {
            assert_eq!(systems["status"], "initial_systems");
            let final_grid = golden["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["case"] == name && c["target"] == "GRID")
                .unwrap();
            assert_eq!(
                systems["systems"].as_array().unwrap().len(),
                final_grid["system_count"].as_u64().unwrap() as usize
            );
            for system in systems["systems"].as_array().unwrap() {
                for id in system["staff_ids"].as_array().unwrap() {
                    let r = &final_grid["staves"][id.as_u64().unwrap() as usize - 1];
                    assert_eq!(
                        system["id"].as_u64().unwrap(),
                        r["system_id"].as_str().unwrap().parse::<u64>().unwrap()
                    );
                }
            }
        }
    }
}
#[test]
fn no_system_cli_does_not_claim_grid_completion() {
    let dir = std::env::temp_dir().join(format!("audiveris-grid-failure-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-grid/synthetic-two-staves.png");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("grid-candidates")
        .arg(input)
        .arg(dir.join("binary.pgm"))
        .arg(dir.join("report.json"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&result.stdout).contains("INITIAL SYSTEMS"));
    let report = read(&dir, "report.json");
    assert_eq!(report["system_candidates"]["error"], "no_system_found");
    assert_eq!(report["system_candidates"]["grid_complete"], false);
    std::fs::remove_dir_all(dir).unwrap();
}
