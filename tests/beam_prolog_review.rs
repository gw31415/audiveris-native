// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent public-boundary review; original geometry controls are retained
// with Java source/provenance in evidence/beam-prolog-review.
use audiveris_native::beam_dispatch::Geometry;
use serde_json::{Value, json};

fn geometry_input(points: &Value) -> Value {
    let points = points.as_array().unwrap();
    let lines = |offset: f64| {
        (0..5)
            .map(|line| {
                json!({"points": points.iter().map(|p| json!([p[0], p[1].as_f64().unwrap() + offset + 5.0 * line as f64])).collect::<Vec<_>>()})
            })
            .collect::<Vec<_>>()
    };
    json!({
        "staffs":[
            {"id":1,"system_id":1,"lines":lines(-20.0)},
            {"id":2,"system_id":2,"lines":lines(40.0)}
        ],
        "systems":[
            {"id":1,"staff_ids":[1],"coordinates":[10,10,81,50,0],"area":{"x_range":[0,100],"vertical_margin":4.0,"north_neighbor_last_line":null,"south_neighbor_first_line":2}},
            {"id":2,"staff_ids":[2],"coordinates":[10,70,81,110,0],"area":{"x_range":[0,100],"vertical_margin":4.0,"north_neighbor_last_line":1,"south_neighbor_first_line":null}}
        ]
    })
}

#[test]
fn original_area_boundary_controls_match_lines_quadratics_cubics_and_knots() {
    let controls: Value = serde_json::from_str(include_str!(
        "../evidence/beam-prolog-review/area-boundary-controls.json"
    ))
    .unwrap();
    assert_eq!(controls["native_outputs_used"], false);
    let mut discrepancies = Vec::new();
    for case in controls["cases"].as_array().unwrap() {
        let geometry = Geometry::from_grid(100, 140, &geometry_input(&case["points"]), 5).unwrap();
        let mut count = 0;
        let mut first = Vec::new();
        for query in case["queries"].as_array().unwrap() {
            let point = [
                query[0].as_i64().unwrap() as i32,
                query[1].as_i64().unwrap() as i32,
            ];
            let expected = (0..2)
                .filter(|&i| query[i + 2] == true)
                .map(|i| i as u64 + 1)
                .collect::<Vec<_>>();
            let actual = geometry.eligible(point).unwrap();
            if actual != expected {
                count += 1;
                if first.len() < 4 {
                    first.push(format!(
                        "{point:?}: expected {expected:?}, actual {actual:?}"
                    ));
                }
            }
        }
        if count > 0 {
            discrepancies.push(format!(
                "{}: {count} mismatches; {}",
                case["name"],
                first.join("; ")
            ));
        }
    }
    assert!(discrepancies.is_empty(), "{}", discrepancies.join("\n"));
}

#[test]
fn malformed_geometry_is_rejected_without_panicking() {
    let valid = geometry_input(&json!([[10, 30], [90, 50]]));
    let mut cases = Vec::new();
    let mut v = valid.clone();
    v["systems"][1]["staff_ids"] = json!([]);
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][1]["staff_ids"] = Value::Null;
    cases.push(v);
    let mut v = valid.clone();
    v["staffs"][1]["lines"][4]["points"] = json!([[50, 80], [50, 90]]);
    cases.push(v);
    let mut v = valid.clone();
    v["staffs"][1]["id"] = json!(1);
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][0]["coordinates"][2] = json!(0);
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][1]["area"]["x_range"] = json!([100, 50]);
    cases.push(v);
    let mut v = valid.clone();
    v["staffs"][0]["system_id"] = json!(2);
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][0]["staff_ids"] = json!([1, 1]);
    cases.push(v);
    for (index, v) in cases.iter().enumerate() {
        let result = std::panic::catch_unwind(|| Geometry::from_grid(100, 140, v, 5));
        assert!(result.is_ok(), "panic for malformed geometry {index}");
        assert!(
            result.unwrap().is_err(),
            "accepted malformed geometry {index}"
        );
    }
}

fn empty_prolog_inputs() -> (
    audiveris_native::beam_spots::PreparedSpots,
    Value,
    Value,
    audiveris_native::scale::Scale,
) {
    use audiveris_native::{
        GrayImage, beam_spots,
        scale::{Beam, Range, Scale},
    };
    let image = GrayImage::new(100, 140, vec![255; 14_000]).unwrap();
    let prepared = beam_spots::prepare(
        &image,
        beam_spots::Parameters {
            max_stem: 1,
            effective_beam: 2.0,
        },
        &[],
    )
    .unwrap();
    let grid = geometry_input(&json!([[10, 30], [90, 50]]));
    let seeds = json!({
        "stem_seeds_complete":true,
        "stem_scaling":{"stem_scale":{"max":1}},
        "systems":[{"system_id":1,"accepted_seeds":[]},{"system_id":2,"accepted_seeds":[]}]
    });
    let scale = Scale {
        line: Range {
            min: 1,
            main: 1,
            max: 1,
        },
        interline: Range {
            min: 5,
            main: 5,
            max: 5,
        },
        small_interline: None,
        beam: Beam {
            thickness: 2,
            extrapolated: false,
        },
        small_beam: None,
    };
    (prepared, grid, seeds, scale)
}

#[test]
fn malformed_seed_dependencies_are_rejected_without_panicking() {
    let (prepared, grid, valid, scale) = empty_prolog_inputs();
    let mut cases = Vec::new();
    let mut v = valid.clone();
    v["stem_seeds_complete"] = json!(false);
    cases.push(v);
    let mut v = valid.clone();
    v["stem_scaling"]["stem_scale"]["max"] = json!(2);
    cases.push(v);
    let mut v = valid.clone();
    v["systems"] = Value::Null;
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][0]["system_id"] = json!(2);
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][1]["accepted_seeds"] = Value::Null;
    cases.push(v);
    let mut v = valid.clone();
    v["systems"][1]["accepted_seeds"] = json!([{}]);
    cases.push(v);
    let seed = json!({"bounds":[20,20,1,1],"orientation":"VERTICAL","runs":[[0,0,1]]});
    let mut v = valid.clone();
    v["systems"][0]["accepted_seeds"] = json!([seed.clone(), seed]);
    cases.push(v);
    for (index, seeds) in cases.iter().enumerate() {
        let result = std::panic::catch_unwind(|| {
            audiveris_native::beam_prolog::build(&prepared, &grid, seeds, &scale)
        });
        assert!(result.is_ok(), "panic for malformed seed input {index}");
        assert!(
            result.unwrap().is_err(),
            "accepted malformed seed input {index}"
        );
    }
}

#[test]
fn valid_empty_prolog_reports_only_its_completed_scope() {
    let (prepared, grid, seeds, scale) = empty_prolog_inputs();
    let prolog = audiveris_native::beam_prolog::build(&prepared, &grid, &seeds, &scale).unwrap();
    let report = prolog.report().unwrap();
    assert_eq!(report["beam_prolog_complete"], true);
    assert_eq!(report["beams_complete"], false);
    assert_eq!(report["musicxml_complete"], false);
    assert_eq!(report["head_scale"]["black_head"], Value::Null);
    assert_eq!(report["head_scale"]["music_font"], Value::Null);
    assert_eq!(report["registry"], json!([]));
    assert_eq!(
        report["systems"],
        json!([
            {"system_id":1,"grouped_spots":[]},
            {"system_id":2,"grouped_spots":[]}
        ])
    );
}

#[test]
fn original_boundary_control_hashes_match_retained_provenance() {
    use sha2::{Digest, Sha256};
    let provenance: Value = serde_json::from_str(include_str!(
        "../evidence/beam-prolog-review/provenance.json"
    ))
    .unwrap();
    assert_eq!(provenance["native_outputs_used"], false);
    assert_eq!(provenance["query_count"], 49_248);
    assert_eq!(
        provenance["files"]["jar"]["sha256"],
        "9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5"
    );
    for key in ["source", "controls"] {
        let row = &provenance["files"][key];
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(row["path"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            row["sha256"].as_str().unwrap()
        );
    }
}

#[test]
fn aggregate_geometry_work_is_rejected_before_registry_or_system_mutation() {
    use audiveris_native::beam_dispatch::{self, Registry, SystemSpots};
    use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
    let points = (0..128)
        .map(|i| json!([10.0 + 80.0 * i as f64 / 127.0, 40 + (i % 2) * 5]))
        .collect::<Vec<_>>();
    let geometry = Geometry::from_grid(100, 140, &geometry_input(&json!(points)), 5).unwrap();
    assert!(geometry.validate_point_work(0).is_ok());
    assert!(geometry.validate_point_work(usize::MAX).is_err());
    let glyph = GlyphPixels {
        left: 20,
        top: 20,
        width: 1,
        height: 1,
        orientation: Orientation::Vertical,
        runs: vec![Run {
            position: 0,
            start: 0,
            length: 1,
        }],
    };
    let mut registry = Registry::default();
    let mut systems = vec![
        SystemSpots {
            system_id: 1,
            glyph_ids: vec![],
        },
        SystemSpots {
            system_id: 2,
            glyph_ids: vec![],
        },
    ];
    let error =
        beam_dispatch::dispatch_into(&vec![glyph; 10_000], &geometry, &mut registry, &mut systems)
            .unwrap_err();
    assert_eq!(error, "unsupported_beam_dispatch_query_work");
    assert!(registry.is_empty());
    assert!(systems.iter().all(|s| s.glyph_ids.is_empty()));
}

#[test]
fn supported_straight_clip_keeps_half_open_edges_and_dispatch_span() {
    let mut grid = geometry_input(&json!([[10, 30], [90, 50]]));
    grid["systems"][1]["area"]["x_range"] = json!([50, 100]);
    let geometry = Geometry::from_grid(100, 140, &grid, 5).unwrap();
    assert_eq!(geometry.eligible([49, 100]).unwrap(), Vec::<u64>::new());
    assert_eq!(geometry.eligible([50, 100]).unwrap(), vec![2]);
    assert_eq!(geometry.eligible([99, 100]).unwrap(), Vec::<u64>::new());
    assert!(!geometry.is_beam_side_in_system(2, [49., 100.], 0.).unwrap());
    assert!(geometry.is_beam_side_in_system(2, [50., 100.], 0.).unwrap());
    assert!(geometry.is_beam_side_in_system(2, [99., 100.], 0.).unwrap());
    assert!(
        !geometry
            .is_beam_side_in_system(2, [100., 100.], 0.)
            .unwrap()
    );
    assert_eq!(geometry.eligible([100, 100]).unwrap(), Vec::<u64>::new());
}
