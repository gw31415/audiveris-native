// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::grid_ir::StaffGeometry;

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-staff-midline/observed.json"
    ))
    .unwrap()
}
#[test]
fn original_staff_midline_selection_interpolation_rounding_and_extrapolation_match_real_lines() {
    for case in original()["cases"].as_array().unwrap() {
        let lines = case["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|points| {
                let mut persistent: Vec<crate::curves::Point> =
                    serde_json::from_value(points.clone()).unwrap();
                for point in &mut persistent {
                    point[1] += 0.5;
                }
                Arc::new(crate::curves::Curve::interpolate(persistent).unwrap())
            })
            .collect::<Vec<_>>();
        let staff = StaffGeometry {
            specific_interline: 11,
            abscissae: [0, 500],
            id: 1,
            system: 1,
            lines,
        };
        assert_eq!(
            json!(
                staff
                    .lines
                    .iter()
                    .map(|line| &line.points)
                    .collect::<Vec<_>>()
            ),
            case["retained_lines"]
        );
        let middle = staff.lines[case["middle_index"].as_u64().unwrap() as usize].clone();
        for query in case["samples"].as_array().unwrap() {
            assert_eq!(
                staff
                    .middle_at(query["x"].as_i64().unwrap() as i32)
                    .unwrap(),
                query["y"].as_i64().unwrap() as i32
            );
            assert_eq!(query["same_middle_line"], true);
            assert!(Arc::ptr_eq(
                &middle,
                &staff.lines[(staff.lines.len() - 1) / 2]
            ));
        }
    }
}

#[test]
fn actual_measured_staff_lines_move_to_music_owner_and_ignore_report_replacement_and_missing_staff()
{
    let image = crate::GrayImage::decode(include_bytes!(
        "../../../fixtures/official-image-musicxml-parallel-voices/parallel-beam.png"
    ))
    .unwrap();
    let page = crate::page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    let staffs = page.staffs.clone();
    let owner = crate::stem_image_owned::continue_items(&image, page).unwrap();
    let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
    let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
    let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
    let owner = crate::music_links::finalize_stems(owner).unwrap();
    let owner = crate::music_measures::continue_measures(owner).unwrap();
    let mut export =
        crate::music_xml::continue_export(crate::music_symbols::continue_symbols(owner).unwrap())
            .unwrap();
    for (held, actual) in staffs.iter().zip(&export.owner.owner.staffs) {
        assert!(Arc::ptr_eq(held, actual));
        for (line, actual) in held.lines.iter().zip(&actual.lines) {
            assert!(Arc::ptr_eq(line, actual));
        }
    }
    let baseline = export.owner.owner.graph.snapshot();
    let layout = export.owner.layout();
    let voices = projection(&export.owner.voices, &export.owner.measures);
    let slots = export
        .owner
        .chords
        .iter()
        .map(|ch| {
            (
                ch.key,
                export
                    .owner
                    .owner
                    .graph
                    .chord_slot_identity(ch.key)
                    .cloned(),
            )
        })
        .collect::<Vec<_>>();
    let physical = export.owner.chords.as_ptr();
    let actual_points = export.owner.owner.staffs[0].lines[2].points.clone();
    export.owner.owner.upstream_report["grid_ir"]["staffs"][0]["lines"] = json!([]);
    export.owner.owner.upstream_report["grid_ir"]["staffs"][0]["id"] = json!(999);
    for query in original()["source"]["samples"].as_array().unwrap() {
        assert_eq!(
            export
                .owner
                .staff_midline_at(1, query["x"].as_i64().unwrap() as i32)
                .unwrap(),
            query["y"].as_i64().unwrap() as i32
        );
    }
    assert_eq!(export.owner.owner.staffs[0].lines[2].points, actual_points);
    let held = std::mem::take(&mut export.owner.owner.staffs);
    assert_eq!(
        export.owner.staff_midline_at(1, 360),
        Err("music_actual_staff_owner_required".into())
    );
    assert_eq!(export.owner.owner.graph.snapshot(), baseline);
    assert_eq!(export.owner.layout(), layout);
    assert_eq!(
        projection(&export.owner.voices, &export.owner.measures),
        voices
    );
    assert_eq!(export.owner.chords.as_ptr(), physical);
    for (key, old) in slots {
        match (old, export.owner.owner.graph.chord_slot_identity(key)) {
            (Some(old), Some(now)) => assert!(Arc::ptr_eq(&old, now)),
            (None, None) => {}
            _ => panic!("read-only middle line changed actual Slot"),
        }
    }
    export.owner.owner.staffs = held;
    assert_eq!(export.owner.staff_midline_at(1, 360).unwrap(), 243);
    for (actual, held) in export.owner.owner.staffs.iter().zip(staffs) {
        assert!(Arc::ptr_eq(actual, &held));
    }
}
