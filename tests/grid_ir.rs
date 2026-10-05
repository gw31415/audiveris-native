// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, bar_sticks, clusters, filament_merge, grid_ir, line_completion, page_glyphs, scale,
    sections, staff_projection, system_graph, vertical,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn near(a: &Value, b: &Value) {
    match b {
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                near(x, y)
            }
        }
        Value::Number(_) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-9,
            "{a} != {b}"
        ),
        _ => assert_eq!(a, b),
    }
}
fn check_glyph(a: &Value, b: &Value) {
    assert_eq!(
        json!([a["left"], a["top"], a["width"], a["height"]]),
        b["bounds"]
    );
    assert_eq!(a["orientation"], b["orientation"]);
    assert_eq!(
        json!(
            a["runs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| json!([r["position"], r["start"], r["length"]]))
                .collect::<Vec<_>>()
        ),
        b["runs"]
    );
}
#[test]
fn native_persistent_staff_sections_membership_and_ordinary_sig_match_final_official_grid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in [
        "laura-gray",
        "synthetic-system-bars",
        "cclef-pair",
        "cclef-tail",
        "cclef-too-wide",
        "cclef-distant",
        "right-extending",
        "line-root",
        "cclef-thick-second",
        "cclef-connected",
        "bracket-pair",
        "bracket-middle",
        "bracket-small-serif",
    ] {
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
        let a = grid_ir::build(&residual, &sc, &prep, &complete, &systems, &sticks).unwrap();
        let b: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-final.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(a["grid_complete"], false);
        let header = audiveris_native::header_preparation::from_grid(&a).unwrap();
        assert_eq!(
            header["staff_headers"], b["header_preparation"],
            "{name} header starts"
        );
        assert_eq!(header["headers_complete"], false);
        let (_, mask) = audiveris_native::note_mask::from_grid(&binary, &a).unwrap();
        for key in [
            "mask_sha256",
            "masked_pixel_count",
            "distance_sha256_be_i16",
        ] {
            assert_eq!(mask[key], b["note_preparation"][key], "{name} mask {key}");
        }
        if name == "bracket-pair" {
            let mut other = scale::analyze(&binary).result.unwrap();
            other.interline.main = 15;
            assert!(
                grid_ir::build(&residual, &other, &prep, &complete, &systems, &sticks)
                    .unwrap_err()
                    .contains("unsupported_bracket_font_metrics")
            );
        }
        assert_eq!(a["pages"], b["pages"]);
        if matches!(name, "laura-gray" | "synthetic-system-bars") {
            let brace_ref: Value = serde_json::from_slice(
                &std::fs::read(root.join(format!("fixtures/official-grid/{name}-brace.json")))
                    .unwrap(),
            )
            .unwrap();
            let expected_portions = brace_ref["phases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["phase"] == "detectBracePortions")
                .unwrap()["portions"]
                .as_array()
                .unwrap();
            let native_portions = complete["braces"]["portions"].as_array().unwrap();
            assert_eq!(native_portions.len(), expected_portions.len());
            for (p, q) in native_portions.iter().zip(expected_portions) {
                for k in ["staff_id", "peak", "sections"] {
                    assert_eq!(p[k], q[k]);
                }
                assert!(
                    q["attributes"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("BRACE_{}", p["kind"].as_str().unwrap()))
                );
            }
        } else {
            assert!(
                complete["braces"]["portions"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
        assert_eq!(a["hlag"], b["hlag"]);
        assert_eq!(a["vlag"], b["vlag"]);
        let native_glyphs = a["glyphs"].as_array().unwrap();
        let java_glyphs = b["glyphs"].as_array().unwrap();
        let mut gid_map = BTreeMap::new();
        for (s, t) in a["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(b["staffs"].as_array().unwrap())
        {
            assert_eq!(s["system_id"], t["system_id"]);
            assert_eq!(s["lines"].as_array().unwrap().len(), 5);
            for (l, m) in s["lines"]
                .as_array()
                .unwrap()
                .iter()
                .zip(t["lines"].as_array().unwrap())
            {
                near(&l["points"], &m["points"]);
                near(&l["thickness"], &m["thickness"]);
                let g = native_glyphs
                    .iter()
                    .find(|g| g["id"] == l["glyph_id"])
                    .unwrap();
                let h = java_glyphs
                    .iter()
                    .find(|g| g["id"] == m["glyph_id"])
                    .unwrap();
                check_glyph(g, h);
                gid_map.insert(g["id"].as_u64().unwrap(), h["id"].as_u64().unwrap());
            }
        }
        for (s, t) in a["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(b["systems"].as_array().unwrap())
        {
            if t.get("part_groups").is_some() {
                assert_eq!(s["part_groups"], t["part_groups"], "{name} part_groups");
            }
            for k in [
                "staff_ids",
                "coordinates",
                "horizontal_section_ids",
                "vertical_section_ids",
                "parts",
            ] {
                assert_eq!(s[k], t[k], "{name} {k}");
            }
        }
        let mut idmap = BTreeMap::new();
        let mut total = 0;
        for (s, t) in a["bar_semantics"]["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(b["systems"].as_array().unwrap())
        {
            let expected: Vec<_> = t["inters"].as_array().unwrap().iter().collect();
            assert_eq!(s["inters"].as_array().unwrap().len(), expected.len());
            total += expected.len();
            for inter in s["inters"].as_array().unwrap() {
                let matched: Vec<_> = expected
                    .iter()
                    .filter(|i| {
                        i["kind"] == inter["kind"]
                            && i["bounds"] == inter["bounds"]
                            && i["staff_id"] == inter["staff_id"]
                    })
                    .collect();
                assert_eq!(matched.len(), 1, "{inter}");
                let java = matched[0];
                idmap.insert(inter["id"].as_u64().unwrap(), java["id"].as_u64().unwrap());
                for k in ["shape", "grade", "contextual_grade"] {
                    near(&inter[k], &java[k]);
                }
                assert_eq!(inter["frozen"], java["frozen"], "freeze: {inter}");
                if !inter["glyph_id"].is_null() {
                    let g = native_glyphs
                        .iter()
                        .find(|g| g["id"] == inter["glyph_id"])
                        .unwrap();
                    let h = java_glyphs
                        .iter()
                        .find(|g| g["id"] == java["glyph_id"])
                        .unwrap();
                    check_glyph(g, h);
                    if let Some(old) =
                        gid_map.insert(g["id"].as_u64().unwrap(), h["id"].as_u64().unwrap())
                    {
                        assert_eq!(old, h["id"].as_u64().unwrap());
                    }
                }
            }
            let native: BTreeSet<_> = s["relations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r["kind"].as_str().unwrap(),
                        idmap[&r["source"].as_u64().unwrap()],
                        idmap[&r["target"].as_u64().unwrap()],
                    )
                })
                .collect();
            let java: BTreeSet<_> = t["relations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r["kind"].as_str().unwrap(),
                        r["source"].as_u64().unwrap(),
                        r["target"].as_u64().unwrap(),
                    )
                })
                .collect();
            assert_eq!(native, java);
        }
        assert_eq!(
            total,
            b["systems"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["inters"].as_array().unwrap().len())
                .sum::<usize>()
        );
        assert_eq!(gid_map.len(), native_glyphs.len());
        assert_eq!(
            gid_map.values().collect::<BTreeSet<_>>().len(),
            gid_map.len()
        );
    }
}
#[test]
fn exact_pixel_registry_reuses_content_and_keeps_distinct_origins_and_orientations() {
    let mut r = grid_ir::GlyphRegistry::default();
    let g = json!({"left":12,"top":24,"width":2,"height":2,"orientation":"HORIZONTAL","runs":[{"position":0,"start":0,"length":2}]});
    assert_eq!(r.register(&g), 1);
    let mut same = g.clone();
    same["source_id"] = json!(999);
    assert_eq!(r.register(&same), 1);
    let mut other = g.clone();
    other["left"] = json!(13);
    assert_eq!(r.register(&other), 2);
    let mut other = g;
    other["orientation"] = json!("VERTICAL");
    assert_eq!(r.register(&other), 3);
    assert_eq!(r.entries.len(), 3);
}

#[test]
fn incomplete_brace_raster_stops_at_explicit_fallback_boundary() {
    let mut image =
        GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap();
    // Remove the lower portion of the first piano brace in a test-local raster.
    // This tests refusal, not parity with an implemented brace fallback.
    for y in 590..700 {
        for x in 210..231 {
            image.pixels[y * image.width + x] = 255;
        }
    }
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
    assert_eq!(complete["status"], "failed");
    assert!(
        complete["error"]
            .as_str()
            .unwrap()
            .contains("unsupported_brace_fallback_alignment"),
        "{complete}"
    );
    assert_eq!(complete["grid_complete"], false);
}
