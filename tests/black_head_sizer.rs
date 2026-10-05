// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::black_head_sizer::*;
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};
fn controls() -> Value {
    serde_json::from_str(include_str!("../fixtures/black-head-sizer-controls.json")).unwrap()
}
fn glyph(value: &Value) -> GlyphPixels {
    let b = &value["bounds"];
    GlyphPixels {
        left: b[0].as_i64().unwrap() as i32,
        top: b[1].as_i64().unwrap() as i32,
        width: b[2].as_u64().unwrap() as usize,
        height: b[3].as_u64().unwrap() as usize,
        orientation: if value["orientation"] == "VERTICAL" {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        },
        runs: value["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| Run {
                position: r[0].as_u64().unwrap() as usize,
                start: r[1].as_u64().unwrap() as usize,
                length: r[2].as_u64().unwrap() as usize,
            })
            .collect(),
    }
}
fn canonical(g: &GlyphPixels) -> Value {
    json!({"bounds":[g.left,g.top,g.width,g.height],"orientation":if g.orientation==Orientation::Vertical {"VERTICAL"}else{"HORIZONTAL"},"runs":g.runs.iter().map(|r|[r.position,r.start,r.length]).collect::<Vec<_>>()})
}
fn rect(w: usize, h: usize) -> GlyphPixels {
    GlyphPixels {
        left: 20,
        top: 30,
        width: w,
        height: h,
        orientation: Orientation::Vertical,
        runs: (0..w)
            .map(|position| Run {
                position,
                start: 0,
                length: h,
            })
            .collect(),
    }
}
fn scale_json(head: Option<BlackHeadScale>, font: Option<&MusicFontScale>) -> Value {
    json!({"black_head":head.map(|s|json!({"widthMean":s.width_mean,"widthStd":s.width_std,"heightMean":s.height_mean,"heightStd":s.height_std})),"music_font":font.map(|s|json!({"name":s.name,"pointSize":s.point_size}))})
}
#[test]
fn all_128_parameter_sets_match_original_including_half_even_boundaries() {
    let fixture = controls();
    let rows = fixture["parameters"].as_array().unwrap();
    assert_eq!(rows.len(), 128);
    for row in rows {
        let il = row["interline"].as_u64().unwrap() as usize;
        let p = Parameters::from_interline(il).unwrap();
        let got = json!({"minWidth":p.min_width,"maxWidth":p.max_width,"minHeight":p.min_height,"maxHeight":p.max_height,"minWeight":p.min_weight,"maxWeight":p.max_weight,"minStackHeight":p.min_stack_height,"maxStackHeight":p.max_stack_height,"minStackWeight":p.min_stack_weight,"maxStackWeight":p.max_stack_weight,"diameter":p.diameter,"binarizationThreshold":p.binarization_threshold,"singlesQuorum":p.singles_quorum});
        assert_eq!(got, row["parameters"], "interline {il}");
    }
}
#[test]
fn original_728_check_close_and_actual_process_controls_match_exactly() {
    let fixture = controls();
    assert_eq!(fixture["native_outputs_used"], false);
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 728);
    let (mut singles, mut stacks, mut rejected_closed, mut disappeared) = (0, 0, 0, 0);
    for (i, row) in rows.iter().enumerate() {
        let g = glyph(&row["input"]);
        let p = Parameters::from_interline(row["interline"].as_u64().unwrap() as usize).unwrap();
        assert_eq!(
            check_spot(&g, &p).unwrap(),
            row["before"].as_bool().unwrap(),
            "precheck {i}"
        );
        let closed = close_black_head(&g, &p).unwrap();
        assert_eq!(
            closed.as_ref().map(canonical),
            if row["closed"].is_null() {
                None
            } else {
                Some(row["closed"].clone())
            },
            "closing {i}"
        );
        if let Some(g) = &closed {
            assert_eq!(
                check_spot(g, &p).unwrap(),
                row["after"].as_bool().unwrap(),
                "postcheck {i}"
            );
        }
        let mut registrations = 0;
        let out = process(std::slice::from_ref(&g), p, "Bravura", |_| {
            registrations += 1;
            Ok(Some(42))
        })
        .unwrap();
        let expected_registration = usize::from(row["before"] == true && closed.is_some());
        assert_eq!(registrations, expected_registration, "registration {i}");
        assert_eq!(out.closed_candidates.len(), expected_registration);
        let actual: Vec<Value> = out
            .closed_candidates
            .iter()
            .filter_map(|c| c.group)
            .map(|g| serde_json::to_value(g).unwrap())
            .collect();
        assert_eq!(
            actual,
            row["groups_after_process"].as_array().unwrap().clone(),
            "tags {i}"
        );
        assert_eq!(row["input_groups_after"], json!([]));
        assert_eq!(canonical(&g), row["input"]);
        singles += out.singles.len();
        stacks += out.stacks.len();
        rejected_closed += out
            .closed_candidates
            .iter()
            .filter(|g| !g.accepted_after_close)
            .count();
        disappeared += usize::from(closed.is_none());
    }
    assert!(singles > 0 && stacks > 0 && rejected_closed > 0 && disappeared > 0);
}
#[test]
fn original_11_measurement_lists_match_stable_width_sort_core_and_sample_statistics() {
    let fixture = controls();
    let rows = fixture["measurements"].as_array().unwrap();
    assert_eq!(rows.len(), 11);
    for row in rows {
        let input: Vec<_> = row["input"].as_array().unwrap().iter().map(glyph).collect();
        let refs: Vec<_> = input.iter().collect();
        let out = measure_singles(&refs, 20, "Bravura").unwrap();
        assert_eq!(
            json!(
                out.sorted_indices
                    .iter()
                    .map(|&i| canonical(&input[i]))
                    .collect::<Vec<_>>()
            ),
            row["sorted"]
        );
        assert_eq!(
            scale_json(out.black_head_scale, out.music_font_scale.as_ref()),
            row["scale"],
            "count {}",
            input.len()
        );
        if input.len() < 20 {
            assert!(out.core_indices.is_empty());
        } else {
            assert_eq!(
                out.core_indices.len(),
                3 * input.len() / 4 - input.len() / 4
            );
        }
    }
}
#[test]
fn original_2048_music_font_interpolations_match_pinned_generic_font_metrics() {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/black-head-font-controls.json")).unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 2048);
    for row in rows {
        let width = row[0].as_f64().unwrap();
        assert_eq!(
            music_font_scale(width, "Bravura").unwrap().point_size,
            row[1].as_i64().unwrap() as i32,
            "width {width}"
        );
    }
}
#[test]
fn all_13_official_sheet_spot_lists_reproduce_black_head_and_music_font_scales() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/official-beams");
    let mut count = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "json")
            || path.file_name().unwrap() == "provenance.json"
        {
            continue;
        }
        let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let Some(rows) = v["sheet_spots"].as_array() else {
            continue;
        };
        let input: Vec<_> = rows.iter().map(|r| glyph(&r["glyph"])).collect();
        let p =
            Parameters::from_interline(v["scale"]["interline"].as_u64().unwrap() as usize).unwrap();
        let out = process(&input, p, "Bravura", |_| Ok(None)).unwrap();
        assert_eq!(
            scale_json(out.black_head_scale, out.music_font_scale.as_ref()),
            v["final"]["head_scale"],
            "{}",
            path.display()
        );
        count += 1;
    }
    assert_eq!(count, 13);
}
#[test]
fn duplicate_unregistered_singles_count_and_unknown_font_keeps_measured_heads() {
    let g = rect(20, 16);
    let input = vec![g; 20];
    let out = process(
        &input,
        Parameters::from_interline(16).unwrap(),
        "unported family",
        |_| Ok(None),
    )
    .unwrap();
    assert_eq!(out.singles.len(), 20);
    assert_eq!(out.measured_core.len(), 10);
    assert_eq!(
        out.black_head_scale.unwrap(),
        BlackHeadScale {
            width_mean: 20.,
            width_std: 0.,
            height_mean: 16.,
            height_std: 0.
        }
    );
    assert!(out.music_font_scale.is_none());
    assert_eq!(
        out.music_font_unsupported.as_deref(),
        Some("unsupported_black_head_music_family")
    );
    assert!(
        out.closed_candidates
            .iter()
            .all(|c| c.registered_id.is_none())
    );
}
#[test]
fn below_quorum_is_no_scale_update_and_group_gap_is_untagged() {
    let p = Parameters::from_interline(16).unwrap();
    let out = process(&vec![rect(20, 16); 19], p, "Bravura", |_| Ok(Some(1))).unwrap();
    assert!(out.black_head_scale.is_none());
    assert!(out.music_font_scale.is_none());
    assert_eq!(out.singles.len(), 19);
    let out = process(&[rect(16, 28)], p, "Bravura", |_| Ok(Some(2))).unwrap();
    assert_eq!(out.closed_candidates.len(), 1);
    assert!(out.closed_candidates[0].accepted_after_close);
    assert_eq!(out.closed_candidates[0].group, None);
}
#[test]
fn invalid_inputs_and_aggregate_work_fail_before_any_registration() {
    assert!(Parameters::from_interline(0).is_err());
    assert!(Parameters::from_interline(129).is_err());
    let p = Parameters::from_interline(16).unwrap();
    let mut invalid = rect(16, 16);
    invalid.runs[0].length = 100;
    let mut called = 0;
    assert!(
        process(&[rect(16, 16), invalid], p, "Bravura", |_| {
            called += 1;
            Ok(None)
        })
        .is_err()
    );
    assert_eq!(called, 0);
    let large = vec![rect(128, 128); 20];
    let error = process(
        &large,
        Parameters::from_interline(128).unwrap(),
        "Bravura",
        |_| {
            called += 1;
            Ok(None)
        },
    )
    .unwrap_err();
    assert_eq!(error, "unsupported_black_head_total_morph_work");
    assert_eq!(called, 0);
    assert!(music_font_scale(f64::NAN, "Bravura").is_err());
    assert!(music_font_scale(257., "Bravura").is_err());
}

#[test]
fn control_and_font_artifacts_match_retained_original_provenance_hashes() {
    use sha2::{Digest, Sha256};
    let p: Value =
        serde_json::from_str(include_str!("../fixtures/black-head-provenance.json")).unwrap();
    assert_eq!(p["native_outputs_used"], false);
    assert_eq!(
        p["upstream_commit"],
        "9e1e55cd2746037d059345881c53e6a6754bffbd"
    );
    for (name, digest) in p["retained_artifacts"].as_object().unwrap() {
        let bytes =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            digest.as_str().unwrap(),
            "{name}"
        );
    }
    for run in p["runs"].as_array().unwrap() {
        assert_eq!(
            run["jar_sha256"],
            "9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5"
        );
        assert_eq!(
            run["resource_sha256"]["res/Bravura.otf"],
            "dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2"
        );
        assert!(
            run["fixtures"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| f["ok"] == true)
        );
    }
}

#[test]
fn horizontal_and_vertical_input_buffers_close_identically_and_coordinates_are_checked() {
    let v = rect(20, 16);
    let h = GlyphPixels {
        orientation: Orientation::Horizontal,
        runs: (0..16)
            .map(|position| Run {
                position,
                start: 0,
                length: 20,
            })
            .collect(),
        ..v.clone()
    };
    let p = Parameters::from_interline(16).unwrap();
    assert_eq!(
        close_black_head(&v, &p).unwrap(),
        close_black_head(&h, &p).unwrap()
    );
    let invalid = GlyphPixels {
        left: i32::MAX,
        ..v
    };
    let mut called = 0;
    assert!(
        process(&[invalid], p, "Bravura", |_| {
            called += 1;
            Ok(None)
        })
        .is_err()
    );
    assert_eq!(called, 0);
}
#[test]
fn repeated_measurement_references_hit_aggregate_run_bound() {
    let glyph = GlyphPixels {
        left: 0,
        top: 0,
        width: 11,
        height: 1,
        orientation: Orientation::Vertical,
        runs: (0..11)
            .map(|position| Run {
                position,
                start: 0,
                length: 1,
            })
            .collect(),
    };
    let repeated = vec![&glyph; 100_000];
    assert_eq!(
        measure_singles(&repeated, 20, "Bravura").unwrap_err(),
        "unsupported_black_head_measurement_runs"
    );
}
