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
fn completed_independent_cue_grid_matches_original_all_observed_physical_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["cue"] {
        let image = GrayImage::decode(
            &std::fs::read(root.join("fixtures/official-head-actual-pages/images/Bravura-cue.png"))
                .unwrap(),
        )
        .unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        let sc = scale::analyze(&binary).result.unwrap();
        let prep = sections::prepare(&binary, &sc).unwrap();
        let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
        let st = clusters::analyze_two_pass(image.width, &sc, &prep.horizontal, &factory);
        let projection = staff_projection::analyze(&binary, &sc, &prep.horizontal, &st);
        let expected_projection: Value = serde_json::from_slice(
            &std::fs::read(root.join("fixtures/official-grid-cue-completion/cue-projection.json"))
                .unwrap(),
        )
        .unwrap();
        let actual = projection["staff_projections"].as_array().unwrap();
        assert_eq!(actual.len(), expected_projection.as_array().unwrap().len());
        for (a, b) in actual.iter().zip(expected_projection.as_array().unwrap()) {
            for key in [
                "staff_id",
                "left",
                "right",
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
                if key == "peak_grades" {
                    near(&a[key], &b[key]);
                } else {
                    assert_eq!(a[key], b[key], "projection {key}");
                }
            }
        }
        assert_eq!(actual[0]["chunk_threshold"], 16);
        assert_eq!(actual[0]["peaks"].as_array().unwrap().len(), 3);
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
            &std::fs::read(root.join("fixtures/official-grid-cue-completion/cue-final.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(a["grid_complete"], false);
        assert_eq!(a["staffs"].as_array().unwrap().len(), 4);
        for (index, staff) in a["staffs"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                staff["specific_interline"],
                b["staff_states"][index]["specific_interline"]
            );
            assert_eq!(
                staff["small"].as_bool().unwrap_or(false),
                b["staff_states"][index]["small"].as_bool().unwrap()
            );
        }

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

fn cue_input() -> (GrayImage, scale::Scale, sections::GridPreparation, Value) {
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-head-actual-pages/images/Bravura-cue.png"
    ))
    .unwrap();
    let binary = vertical::binarize(&image, |_, _, _| {});
    let sc = scale::analyze(&binary).result.unwrap();
    let prep = sections::prepare(&binary, &sc).unwrap();
    let factory = filament_merge::analyze(binary.width, &sc, &prep.horizontal);
    let staff = clusters::analyze_two_pass(binary.width, &sc, &prep.horizontal, &factory);
    (binary, sc, prep, staff)
}

#[test]
fn projection_refuses_unknown_or_inconsistent_staff_scale_without_partial_candidates() {
    let (binary, sc, prep, staff) = cue_input();
    for invalid in [0, 8, 129] {
        let mut input = staff.clone();
        input["staffs"][0]["interline"] = json!(invalid);
        let result = staff_projection::analyze(&binary, &sc, &prep.horizontal, &input);
        assert_eq!(
            result["error"],
            "unsupported_staff_projection_specific_interline"
        );
        assert!(result.get("staff_projections").is_none());
    }
    let mut input = staff.clone();
    input["staffs"][0]["small"] = json!(false);
    assert_eq!(
        staff_projection::analyze(&binary, &sc, &prep.horizontal, &input)["error"],
        "unsupported_staff_projection_specific_interline"
    );
    let mut unknown = sc;
    unknown.small_interline = None;
    assert_eq!(
        staff_projection::analyze(&binary, &unknown, &prep.horizontal, &staff)["error"],
        "unsupported_staff_projection_specific_interline"
    );
}

#[test]
fn actual_cue_grid_cli_runs_second_pass_and_completed_line_state_with_global_flags_false() {
    use std::process::Command;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp =
        std::env::temp_dir().join(format!("audiveris-cue-grid-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&temp).unwrap();
    let report = temp.join("grid.json");
    let run = Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .arg("grid-candidates")
        .arg(root.join("fixtures/official-head-actual-pages/images/Bravura-cue.png"))
        .arg(temp.join("binary.pgm"))
        .arg(&report)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let r: Value = serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
    assert_eq!(r["staff_candidates"]["second_pass_ran"], true);
    assert_eq!(r["staff_candidates"]["staffs"].as_array().unwrap().len(), 4);
    assert_eq!(
        r["line_completion"]["endpoint_geometry"]["staffs"][0]["interline"],
        7
    );
    assert_eq!(
        r["line_completion"]["endpoint_geometry"]["staffs"][0]["small"],
        true
    );
    for stage in ["staff_candidates", "bar_candidates", "line_completion"] {
        assert_eq!(r[stage]["grid_complete"], false);
    }
    std::fs::remove_dir_all(temp).unwrap();
}
