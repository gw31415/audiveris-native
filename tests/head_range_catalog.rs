// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::Geometry,
    curves::Curve,
    head_ledger_search,
    head_range_search::{self, Scan},
    head_seed_veto::Context,
    template_probes, vertical,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn sorted(v: &Value) -> Vec<String> {
    let mut out = v
        .as_array()
        .unwrap()
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>();
    out.sort();
    out
}
#[test]
fn original_laura_all_118_default_catalog_scanners() {
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-range-catalog/laura.json"
    ))
    .unwrap();
    let veto: Value =
        serde_json::from_str(include_str!("../fixtures/official-head-veto.json")).unwrap();
    let grid: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/laura-gray-final.json"
    ))
    .unwrap();
    assert_eq!(g["native_outputs_used"], false);
    let spots_image =
        GrayImage::decode(include_bytes!("../fixtures/official-head-range-spots.png")).unwrap();
    // Original GRID snapshot has no native area contract. Use separately
    // certified original Laura system-neighbor geometry for dispatch.
    let neighbors: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-dispatch-neighbors.json"
    ))
    .unwrap();
    let config = neighbors["configurations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "original-laura-stacked")
        .unwrap();
    let mut dispatch_grid = config["input"]["grid"].clone();
    dispatch_grid["schema"] = json!("audiveris-native.grid-ir.v1");
    for (system, oracle) in dispatch_grid["systems"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(config["systems"].as_array().unwrap())
    {
        system["area"] = json!({"x_range":oracle["x_range"],"vertical_margin":10.});
        for (field, key) in [
            ("north_neighbor_last_line", "north_staff_ids"),
            ("south_neighbor_first_line", "south_staff_ids"),
        ] {
            let ids = oracle[key].as_array().unwrap();
            system["area"][field] = if ids.len() == 1 {
                ids[0].clone()
            } else {
                Value::Null
            };
            if ids.len() > 1 {
                system["area"][key] = oracle[key].clone();
            }
        }
    }
    let geometry =
        Geometry::from_grid(spots_image.width, spots_image.height, &dispatch_grid, 11).unwrap();
    let dispatch = head_range_search::dispatch(&spots_image, &geometry).unwrap();
    assert_eq!(dispatch["head_spots_sha256"], g["head_spots_sha256"]);
    assert_eq!(
        dispatch["system_spot_references"],
        g["system_spot_references"]
    );
    assert_eq!(dispatch["system_spot_references"], 402);
    let page =
        GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap();
    let binary = vertical::binarize(&page, |_, _, _| {});
    let mask = GrayImage::decode(include_bytes!("../fixtures/official-heads-mask.png")).unwrap();
    let table = template_probes::distances(&binary, Some(&mask)).unwrap();
    let bank = Bank::load_builtin().unwrap();
    let (mut scanners, mut visited, mut skipped, mut proposals, mut rejected) = (0, 0, 0, 0, 0);
    for sys in g["systems"].as_array().unwrap() {
        let ds = dispatch["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system_id"] == sys["system_id"])
            .unwrap();
        let mut a = ds["spots"].clone();
        for s in a.as_array_mut().unwrap() {
            s.as_object_mut().unwrap().remove("sheet_spot_index");
        }
        assert_eq!(sorted(&a), sorted(&sys["spots"]));
        let spots = ds["spots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| audiveris_native::beam_dispatch::glyph_from_json(&s["glyph"]).unwrap())
            .collect::<Vec<_>>();
        let vs = veto["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system_id"] == sys["system_id"])
            .unwrap();
        let context =
            Context::from_inters(vs["prolog"]["all_inters"].as_array().unwrap(), 11, 5).unwrap();
        for staff in sys["staffs"].as_array().unwrap() {
            let gs = grid["staffs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == staff["staff_id"])
                .unwrap();
            let vf = vs["staffs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["staff_id"] == staff["staff_id"])
                .unwrap();
            let lines = gs["lines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| {
                    Curve::interpolate(serde_json::from_value(l["points"].clone()).unwrap())
                        .unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(staff["point_size"], 51);
            for key in ["scanners", "ledger_scanners"] {
                assert_eq!(
                    staff[key].as_array().unwrap().len(),
                    vf[key].as_array().unwrap().len()
                );
                for (c, v) in staff[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(vf[key].as_array().unwrap())
                {
                    let dir = c["dir"].as_i64().unwrap() as i32;
                    let pitch = c["pitch"].as_i64().unwrap() as i32;
                    let index = c["line_index"].as_u64().unwrap_or(0) as usize;
                    let ledger: Option<[f64; 4]> = if key == "ledger_scanners" {
                        Some(serde_json::from_value(v["main_line"].clone()).unwrap())
                    } else {
                        None
                    };
                    let further: Vec<[f64; 4]> = if ledger.is_some() {
                        serde_json::from_value(v["further_lines"].clone()).unwrap()
                    } else {
                        Vec::new()
                    };
                    let line = if let Some(l) = ledger {
                        Curve::interpolate(vec![[l[0], l[1]], [l[2], l[3]]]).unwrap()
                    } else {
                        lines[index].clone()
                    };
                    let slice = context.slice(&line, 11, dir).unwrap();
                    let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone()).unwrap();
                    let left = if ledger.is_some() {
                        line.points[0][0].ceil()
                    } else {
                        line.points[0][0].floor()
                    } as i32;
                    let right = line.points.last().unwrap()[0].floor() as i32 - 11;
                    let range = [
                        left.max(staff["header_stop"].as_i64().unwrap() as i32),
                        right,
                    ];
                    let report = head_range_search::scan_catalog(
                        Scan {
                            table: &table,
                            bank: &bank,
                            line: &line,
                            interline: 11,
                            dir,
                            range,
                            main_interline: 11,
                            y_offsets: &yo,
                            spots: &spots,
                        },
                        &mut |x| {
                            if let Some(l) = ledger {
                                head_ledger_search::ordinate(l, &further, pitch, 11, x)
                            } else {
                                Ok((if dir == -1 && index > 0 {
                                    (line.y(x as f64) + lines[index - 1].y(x as f64)) / 2.
                                } else {
                                    line.y(x as f64) + dir as f64 * 11. / 2.
                                })
                                .round_ties_even() as i32)
                            }
                        },
                        &mut |b| slice.veto(b),
                    )
                    .unwrap();
                    for k in [
                        "scan_range",
                        "template_half",
                        "black_relevant_spans",
                        "visited",
                        "skipped",
                    ] {
                        assert_eq!(
                            report[k], c[k],
                            "staff {} pitch {pitch} {k}",
                            staff["staff_id"]
                        );
                    }
                    assert_eq!(
                        sorted(&report["selected_spots"]),
                        sorted(&c["selected_spots"])
                    );
                    assert_eq!(report["all_shapes"], c["all_shapes"]);
                    assert_eq!(report["hollow_shapes"], c["hollow_shapes"]);
                    for key in ["matches", "rejected"] {
                        compare_candidates(&report[key], &c[key]);
                    }
                    scanners += 1;
                    visited += report["visited"].as_array().unwrap().len();
                    skipped += report["skipped"].as_array().unwrap().len();
                    proposals += report["matches"].as_array().unwrap().len();
                    rejected += report["rejected"].as_array().unwrap().len();
                }
            }
        }
    }
    assert_eq!(
        (scanners, visited, skipped, proposals, rejected),
        (118, 47790, 869, 2224, 407)
    );
}

fn compare_candidates(actual: &Value, expected: &Value) {
    let a = actual.as_array().unwrap();
    let e = expected.as_array().unwrap();
    assert_eq!(a.len(), e.len(), "{actual} != {expected}");
    for (a, e) in a.iter().zip(e) {
        for k in ["shape", "source_shape", "anchor", "x", "y", "bounds"] {
            assert_eq!(a[k], e[k], "{k}: {a} != {e}");
        }
        for k in ["score", "grade"] {
            assert!(
                (a[k].as_f64().unwrap() - e[k].as_f64().unwrap()).abs() < 1e-12,
                "{k}: {a} != {e}"
            );
        }
        compare_decision(&a["eligibility"], &e["eligibility"]);
    }
}
fn compare_decision(a: &Value, e: &Value) {
    for k in [
        "stemless",
        "weak_stemless",
        "create_inter_eligible",
        "range_eligible",
    ] {
        assert_eq!(a[k], e[k], "{k}: {a} != {e}");
    }
    for k in [
        "intrinsic_grade",
        "future_staff_grade",
        "pending_stemless_boost",
    ] {
        assert!((a[k].as_f64().unwrap() - e[k].as_f64().unwrap()).abs() < 1e-12);
    }
}
#[test]
fn original_whole_breve_renders_with_and_without_black_spots() {
    use sha2::{Digest, Sha256};
    let controls: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-range-catalog/controls.json"
    ))
    .unwrap();
    assert_eq!(controls["native_outputs_used"], false);
    let bank = Bank::load_builtin().unwrap();
    let mut clean_stemless = 0;
    let mut weak_stemless = 0;
    for c in controls["images"].as_array().unwrap() {
        let image = GrayImage::decode(
            &std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("fixtures/official-head-range-catalog")
                    .join(c["image"].as_str().unwrap()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&image.pixels)),
            c["binary_sha256"]
        );
        let table = template_probes::distances(&image, None).unwrap();
        let result = head_range_search::candidates_at(
            &table,
            &bank,
            serde_json::from_value(c["pivot"].clone()).unwrap(),
            &[0, -1, 1],
            c["black_relevant"].as_bool().unwrap(),
            &mut |_| Ok(false),
        )
        .unwrap();
        for key in ["matches", "rejected"] {
            compare_candidates(&result[key], &c[key]);
        }
        if c["variant"] == "clean"
            && c["black_relevant"] == false
            && matches!(
                c["rendered_shape"].as_str().unwrap(),
                "BREVE" | "WHOLE_NOTE"
            )
        {
            assert!(
                result["matches"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|m| m["shape"] == c["rendered_shape"])
            );
            clean_stemless += 1;
        }
        weak_stemless += result["rejected"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["eligibility"]["weak_stemless"] == true)
            .count();
    }
    assert_eq!(clean_stemless, 2);
    assert!(weak_stemless > 0);
}
#[test]
fn original_stemless_and_constructor_grade_ulp_boundaries() {
    let controls: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-range-catalog/controls.json"
    ))
    .unwrap();
    for c in controls["boundaries"].as_array().unwrap() {
        let score = f64::from_bits(c["score_bits"].as_u64().unwrap());
        let a = head_range_search::candidate_decision(c["shape"].as_str().unwrap(), score).unwrap();
        compare_decision(&a, &c["eligibility"]);
    }
    for score in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert!(head_range_search::candidate_decision("BREVE", score).is_err());
    }
    assert!(head_range_search::candidate_decision("NOTEHEAD_CROSS", 0.).is_err());
}
