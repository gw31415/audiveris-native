// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::multiple_rest_admission::{self, StaffPosition};
use serde_json::Value;
#[test]
fn original_process_admission_and_serif_queries() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-multiple-rest-admission.json"
    ))
    .unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 73);
    let mut admitted = 0;
    for (i, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let staff = if c["staff"].is_null() {
            None
        } else {
            Some(StaffPosition {
                id: c["staff"]["id"].as_u64().unwrap(),
                tablature: c["staff"]["tablature"].as_bool().unwrap(),
                endpoint_pitches: std::array::from_fn(|n| {
                    c["staff"]["endpoint_pitches"][n].as_f64().unwrap()
                }),
                mid_line_thickness: c["staff"]["mid_line_thickness"].as_f64().unwrap(),
            })
        };
        let got = multiple_rest_admission::assess(
            c["input"]["kind"].as_str().unwrap_or("BeamInter"),
            false,
            std::array::from_fn(|n| c["bounds"][n].as_i64().unwrap() as i32),
            c["height"].as_f64().unwrap(),
            c["input"]["interline"].as_u64().unwrap_or(16) as usize,
            staff,
        )
        .unwrap();
        assert_eq!(
            got.rejection.is_none(),
            c["admitted_to_projector"].as_bool().unwrap(),
            "case {i}"
        );
        if got.rejection.is_none() {
            admitted += 1;
            let queries = got
                .queries
                .iter()
                .map(|q| serde_json::json!({"x":q.x,"left":q.left,"added_chunk":q.added_chunk}))
                .collect::<Vec<_>>();
            assert_eq!(serde_json::json!(queries), c["queries"], "case {i} queries");
            let arrays = c["peak_lists"].as_array().unwrap();
            let selected = multiple_rest_admission::select_serif_peaks(
                arrays[0].as_array().unwrap(),
                arrays[1].as_array().unwrap(),
            );
            assert_eq!(
                serde_json::json!(selected),
                c["selected_peaks"],
                "case {i} peaks"
            );
        }
    }
    assert!(admitted > 0);
}
#[test]
fn missing_staff_tablature_removed_and_invalid_measurements() {
    let staff = StaffPosition {
        id: 1,
        tablature: false,
        endpoint_pitches: [0., 0.],
        mid_line_thickness: 2.,
    };
    let assess = |s, removed| {
        multiple_rest_admission::assess("BeamInter", removed, [20, 10, 64, 8], 8., 16, s).unwrap()
    };
    assert_eq!(assess(None, false).rejection, Some("no_staff"));
    assert_eq!(
        assess(
            Some(StaffPosition {
                tablature: true,
                ..staff
            }),
            false
        )
        .rejection,
        Some("tablature")
    );
    assert_eq!(
        assess(Some(staff), true).rejection,
        Some("not_active_normal_beam")
    );
    assert!(
        multiple_rest_admission::assess(
            "BeamInter",
            false,
            [0, 0, 64, 8],
            8.,
            16,
            Some(StaffPosition {
                endpoint_pitches: [f64::NAN, 0.],
                ..staff
            })
        )
        .is_err()
    );
    let left = [2, 1, 4];
    let right = [8, 7, 9];
    assert_eq!(
        multiple_rest_admission::select_serif_peaks(&left, &right),
        [Some(&4), Some(&8)]
    );
}
