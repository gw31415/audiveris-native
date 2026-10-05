// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    key_preparation::StaffInput,
    time_preparation::{Parameters, Space, find_spaces, prepare_staff},
};
fn staff() -> StaffInput {
    StaffInput {
        staff_id: 1,
        header_start: 10,
        header_stop: 40,
        clef_stop: Some(30),
        specific_interline: 10,
        lines: (0..5)
            .map(|i| vec![[0., (40 + 10 * i) as f64], [199., (40 + 10 * i) as f64]])
            .collect(),
        barlines: vec![],
    }
}
#[test]
fn whitespace_includes_single_columns_threshold_and_terminal_space() {
    assert_eq!(
        find_spaces(40, &[4, 5, 0, 4, 8, 0], 4).unwrap(),
        vec![
            Space {
                start: 40,
                stop: 40
            },
            Space {
                start: 42,
                stop: 43
            },
            Space {
                start: 45,
                stop: 45
            }
        ]
    );
    assert!(find_spaces(0, &[], 4).unwrap().is_empty());
}
#[test]
fn blank_roi_is_not_a_signature_and_starts_at_header_stop() {
    let image = GrayImage::new(200, 120, vec![255; 24000]).unwrap();
    let p = prepare_staff(&image, 10, &staff()).unwrap();
    assert_eq!(p.raw_roi, [40, 40, 40, 41]);
    assert_eq!(
        p.spaces,
        vec![Space {
            start: 40,
            stop: 79
        }]
    );
    assert!(p.start.is_none());
    assert!(p.candidate_rectangles.is_none());
}
#[test]
fn range_retains_edge_space_and_half_height_uses_integer_division() {
    let mut image = GrayImage::new(200, 120, vec![255; 24000]).unwrap();
    for y in 45..75 {
        for x in 45..60 {
            image.pixels[y * 200 + x] = 0;
        }
    }
    let p = prepare_staff(&image, 10, &staff()).unwrap();
    assert_eq!((p.start, p.stop), (Some(44), Some(60)));
    assert_eq!(
        p.candidate_rectangles,
        Some([[44, 41, 17, 39], [44, 41, 17, 18], [44, 62, 17, 18]])
    );
}
#[test]
fn scale_rounding_uses_ties_even_and_sheet_staff_scales_separately() {
    let p = Parameters::new(11, 5).unwrap();
    assert_eq!(
        (p.roi_width, p.max_first_space_width, p.max_inner_space),
        (44, 28, 6)
    );
    assert_eq!(
        (
            p.y_margin,
            p.min_time_width,
            p.min_part_weight,
            p.min_half_time_weight
        ),
        (0, 5, 0, 19)
    );
}
#[test]
fn sloped_bottom_uses_stop_only_and_image_edges_are_clamped() {
    let image = GrayImage::new(70, 90, vec![255; 6300]).unwrap();
    let mut s = staff();
    s.lines = (0..5)
        .map(|i| vec![[0., (40 + 10 * i) as f64], [199., (20 + 10 * i) as f64]])
        .collect();
    let p = prepare_staff(&image, 10, &s).unwrap();
    assert_eq!(p.raw_roi, [40, 32, 40, 41]);
    assert_eq!(p.roi, [40, 32, 30, 41]);
    assert_eq!(p.projection.len(), 30);
}
#[test]
fn rejects_nonbinary_and_unbounded_inputs() {
    let mut image = GrayImage::new(200, 120, vec![255; 24000]).unwrap();
    image.pixels[0] = 42;
    assert!(prepare_staff(&image, 10, &staff()).is_err());
    assert!(Parameters::new(129, 10).is_err());
    assert!(find_spaces(0, &[-1], 4).is_err());
}
