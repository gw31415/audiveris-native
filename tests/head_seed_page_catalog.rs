// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, beam_dispatch, head_glyph_retrieval, head_range_search, head_seed_search,
    head_seed_veto, head_staff_catalog::TemplateCatalog, template_probes, vertical,
};
use serde_json::{Value, json};
fn before() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-seed-page-catalog/native-before-seeds.json"
    ))
    .unwrap()
}
fn cases() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-seed-page-catalog/cases.json"
    ))
    .unwrap()
}
fn binary() -> GrayImage {
    vertical::binarize(
        &GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap(),
        |_, _, _| {},
    )
}
fn matches(a: &Value, e: &Value) {
    assert_eq!(
        a.as_array().unwrap().len(),
        e.as_array().unwrap().len(),
        "{a} != {e}"
    );
    for (a, e) in a.as_array().unwrap().iter().zip(e.as_array().unwrap()) {
        for k in ["shape", "source_shape", "anchor", "x", "y", "bounds"] {
            assert_eq!(a[k], e[k], "{k}: {a} != {e}");
        }
        for k in ["score", "grade"] {
            assert!((a[k].as_f64().unwrap() - e[k].as_f64().unwrap()).abs() < 1e-12);
        }
    }
}
#[test]
fn original_thirteen_controlled_page_seed_staff_ledger_and_veto_catalogs() {
    let input = before();
    let binary = binary();
    let cases = cases();
    assert_eq!(cases["native_outputs_used"], false);
    assert_eq!(cases["cases"].as_array().unwrap().len(), 13);
    let mut total_pairs = 0;
    for c in cases["cases"].as_array().unwrap() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/official-head-seed-page-catalog")
            .join(format!("{}.json", c["name"].as_str().unwrap()));
        let g: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(g["native_outputs_used"], false);
        let mut grid = input["grid"].clone();
        if c["cue_first_staff"] == true {
            grid["staffs"][0]["specific_interline"] = json!(7);
        }
        let scale = json!({"music_font":{"name":c["family"],"pointSize":c["stored_point_size"]}});
        let raw = head_seed_search::analyze_catalog(
            &binary,
            &grid,
            &input["seeds"],
            &input["ledgers"],
            &scale,
            5,
            11,
        )
        .unwrap();
        let veto = head_seed_veto::analyze(head_seed_veto::Input {
            binary: &binary,
            grid: &grid,
            seeds: &input["seeds"],
            ledgers: &input["ledgers"],
            shared_sig: &input["shared_sig"],
            raw_search: &raw,
            main_interline: 11,
            max_stem: 5,
        })
        .unwrap();
        assert_eq!(raw["template_catalog"], g["template_catalog"]);
        assert_eq!(veto["template_catalog"], g["template_catalog"]);
        assert_eq!(raw["heads_complete"], false);
        assert_eq!(veto["heads_complete"], false);
        assert_eq!(veto["seed_pitch_pairs"], c["seed_pitch_pairs"]);
        assert_eq!(veto["proposal_count"], c["veto_proposals"]);
        assert_eq!(raw["distance_preparation"], veto["distance_preparation"]);
        assert_eq!(raw["distance_preparation"]["mask_sha256"], g["mask_sha256"]);
        assert_eq!(
            raw["distance_preparation"]["distance_sha256_be_i16"],
            g["distance_sha256_be_i16"]
        );
        let mut staff_sizes = Vec::new();
        let mut raw_count = 0;
        for ((rs, vs), gs) in raw["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(veto["systems"].as_array().unwrap())
            .zip(g["systems"].as_array().unwrap())
        {
            assert_eq!(rs["system_id"], gs["system_id"]);
            assert_eq!(
                rs["staffs"].as_array().unwrap().len(),
                gs["staffs"].as_array().unwrap().len()
            );
            for ((rf, vf), gf) in rs["staffs"]
                .as_array()
                .unwrap()
                .iter()
                .zip(vs["staffs"].as_array().unwrap())
                .zip(gs["staffs"].as_array().unwrap())
            {
                assert_eq!(rf["staff_id"], gf["staff_id"]);
                assert_eq!(rf["point_size"], gf["point_size"]);
                staff_sizes.push(rf["point_size"].clone());
                for key in ["scanners", "ledger_scanners"] {
                    assert_eq!(
                        rf[key].as_array().unwrap().len(),
                        gf[key].as_array().unwrap().len()
                    );
                    for ((r, v), e) in rf[key]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(vf[key].as_array().unwrap())
                        .zip(gf[key].as_array().unwrap())
                    {
                        for k in [
                            "line_index",
                            "ledger_index",
                            "ledger_order",
                            "dir",
                            "pitch",
                            "x_offsets",
                            "y_offsets",
                        ] {
                            assert_eq!(r[k], e[k]);
                        }
                        let xo: Vec<i32> = serde_json::from_value(r["x_offsets"].clone()).unwrap();
                        let yo: Vec<i32> = serde_json::from_value(r["y_offsets"].clone()).unwrap();
                        assert_eq!(
                            r["seeds"].as_array().unwrap().len(),
                            e["seeds"].as_array().unwrap().len()
                        );
                        for ((rz, vz), ez) in r["seeds"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .zip(v["seeds"].as_array().unwrap())
                            .zip(e["seeds"].as_array().unwrap())
                        {
                            assert_eq!(rz["seed"]["bounds"], ez["seed"]["bounds"]);
                            assert_eq!(rz["x0"], ez["x0"]);
                            assert_eq!(rz["y0"], ez["y0"]);
                            let mut expected_raw = Vec::new();
                            for an in ["LEFT_STEM", "RIGHT_STEM"] {
                                for sh in ["NOTEHEAD_BLACK", "NOTEHEAD_VOID"] {
                                    let k = format!(
                                        "{}:{}:{}:{xo:?}:{yo:?}:{an}:{sh}",
                                        rf["point_size"], rz["x0"], rz["y0"]
                                    );
                                    let expected =
                                        g["raw_controls"].as_object().unwrap().get(&k).unwrap();
                                    if !expected.is_null() {
                                        expected_raw.push(expected.clone());
                                    }
                                }
                            }
                            matches(&rz["matches"], &json!(expected_raw));
                            matches(&vz["matches"], &ez["matches"]);
                            raw_count += expected_raw.len();
                        }
                    }
                }
            }
        }
        assert_eq!(json!(staff_sizes), c["point_sizes"]);
        assert_eq!(raw["proposal_count"].as_u64().unwrap(), raw_count as u64);
        if c["name"] == "Bravura-51" {
            assert_eq!(raw_count, 418);
            assert_eq!(veto["proposal_count"], 358);
        }
        total_pairs += c["seed_pitch_pairs"].as_u64().unwrap();
    }
    assert_eq!(total_pairs, 14764);
}
#[test]
fn declared_catalog_validation_missing_assets_and_legacy_scope_remain_explicit() {
    let input = before();
    let image = binary();
    for family in ["unknown", "bravura", ""] {
        assert!(
            TemplateCatalog {
                family: family.into(),
                stored_point_size: 51
            }
            .validate()
            .is_err()
        );
    }
    for size in [0, 31, 33, 50, 52, 65, u16::MAX] {
        assert!(
            TemplateCatalog {
                family: "Bravura".into(),
                stored_point_size: size
            }
            .validate()
            .is_err()
        );
    }
    let catalog = TemplateCatalog {
        family: "Bravura".into(),
        stored_point_size: 51,
    };
    assert_eq!(catalog.staff_point_size(7, 11).unwrap(), 32);
    assert!(catalog.staff_point_size(8, 11).is_err());
    assert!(catalog.staff_point_size(0, 11).is_err());
    assert!(
        TemplateCatalog::from_report(
            &json!({"template_catalog":{"family":"Leland","stored_point_size":40,"unknown":true}})
        )
        .is_err()
    );
    assert_eq!(
        TemplateCatalog::from_report(&json!({"template_catalog_required":true})).unwrap_err(),
        "missing_head_template_catalog"
    );
    let mut grid = input["grid"].clone();
    grid["staffs"][0]["specific_interline"] = json!(8);
    let scale = json!({"music_font":{"name":"Bravura","pointSize":51}});
    assert_eq!(
        head_seed_search::analyze_catalog(&image, &grid, &json!({}), &json!({}), &scale, 5, 11)
            .unwrap_err(),
        "unsupported_head_staff_template_catalog"
    );
    grid["staffs"][0]["specific_interline"] = json!(7);
    assert_eq!(
        head_seed_search::analyze(&image, &grid, &json!({}), &json!({}), &scale, 5, 11)
            .unwrap_err(),
        "unsupported_head_staff_template_catalog"
    );
    let wrong = json!({"music_font":{"name":"Leland","pointSize":51}});
    assert_eq!(
        head_seed_search::analyze(
            &image,
            &input["grid"],
            &json!({}),
            &json!({}),
            &wrong,
            5,
            11
        )
        .unwrap_err(),
        "unsupported_head_template_catalog"
    );
}
#[test]
fn stale_staff_size_and_inconsistent_range_glyph_contexts_fail_without_registry_mutation() {
    let input = before();
    let binary = binary();
    let mut raw = head_seed_search::analyze_catalog(
        &binary,
        &input["grid"],
        &input["seeds"],
        &input["ledgers"],
        &json!({"music_font":{"name":"Leland","pointSize":51}}),
        5,
        11,
    )
    .unwrap();
    raw["systems"][0]["staffs"][0]["point_size"] = json!(40);
    assert_eq!(
        head_seed_veto::analyze(head_seed_veto::Input {
            binary: &binary,
            grid: &input["grid"],
            seeds: &input["seeds"],
            ledgers: &input["ledgers"],
            shared_sig: &input["shared_sig"],
            raw_search: &raw,
            main_interline: 11,
            max_stem: 5
        })
        .unwrap_err(),
        "inconsistent_head_seed_catalog"
    );
    let veto = json!({"seed_veto_complete":true,"template_catalog":{"family":"Leland","stored_point_size":51}});
    let headers = json!({"headers_complete":true});
    let geometry =
        beam_dispatch::Geometry::from_grid(binary.width, binary.height, &input["grid"], 11)
            .unwrap();
    let invalid_veto = json!({"seed_veto_complete":true,"template_catalog":{"family":"unknown","stored_point_size":51}});
    assert_eq!(
        head_range_search::analyze_catalog(head_range_search::Input {
            binary: &binary,
            head_spots: &binary,
            grid: &input["grid"],
            headers: &headers,
            seeds: &input["seeds"],
            ledgers: &input["ledgers"],
            shared_sig: &input["shared_sig"],
            seed_veto: &invalid_veto,
            geometry: &geometry,
            main_interline: 11,
            max_stem: 5
        })
        .unwrap_err(),
        "unsupported_head_template_catalog"
    );
    let white = GrayImage {
        width: 16,
        height: 16,
        pixels: vec![255; 256],
    };
    let table = template_probes::distances(&white, None).unwrap();
    let mut registry = beam_dispatch::Registry::default();
    let before = registry.snapshot();
    assert_eq!(
        head_glyph_retrieval::analyze_for_registration(
            head_glyph_retrieval::Input {
                binary: &binary,
                grid: &input["grid"],
                seed_veto: &veto,
                catalog: &json!({"range_catalog_candidates_complete":true}),
                main_interline: 11,
                table: &table,
                shared_sig: &input["shared_sig"],
                max_stem: 5
            },
            &mut registry
        )
        .unwrap_err(),
        "inconsistent_head_template_catalog"
    );
    assert_eq!(registry.snapshot(), before);
}
