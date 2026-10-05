// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::GrayImage;
use audiveris_native::beam_spots::*;

#[test]
fn length_filter_is_inclusive_and_preserves_original_input() {
    let input = GrayImage::new(
        8,
        2,
        vec![0, 0, 255, 0, 0, 0, 255, 0, 255, 0, 0, 0, 0, 255, 0, 0],
    )
    .unwrap();
    let output = filter_horizontal(&input, 3).unwrap();
    assert_eq!(
        output.pixels,
        vec![
            255, 255, 255, 0, 0, 0, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255
        ]
    );
    assert_eq!(input.pixels[0], 0);
}
#[test]
fn median_has_unmodified_boundary() {
    let input = GrayImage::new(3, 3, vec![0, 0, 0, 0, 255, 0, 255, 255, 255]).unwrap();
    let output = median_filter(&input).unwrap();
    assert_eq!(output.pixels, vec![0, 0, 0, 0, 0, 0, 255, 255, 255]);
    let line = GrayImage::new(1, 4, vec![0, 255, 0, 255]).unwrap();
    assert_eq!(median_filter(&line).unwrap().pixels, line.pixels);
}
#[test]
fn roi_is_inclusive_and_clipped() {
    let image = GrayImage::new(4, 3, vec![0; 12]).unwrap();
    let out = erase_headers(
        &image,
        &[HeaderRect {
            left: -1,
            top: 1,
            right: 1,
            bottom: 9,
        }],
    )
    .unwrap();
    assert_eq!(out.pixels, vec![0, 0, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0]);
    assert!(
        erase_headers(
            &image,
            &[HeaderRect {
                left: 2,
                top: 1,
                right: 1,
                bottom: 9
            }]
        )
        .is_err()
    );
}
#[test]
fn threshold_equality_and_four_connected_component_order() {
    let image = GrayImage::new(3, 3, vec![140, 255, 255, 255, 139, 255, 255, 255, 141]).unwrap();
    let binary = threshold(&image, 140).unwrap();
    let spots = extract_spots(&binary).unwrap();
    assert_eq!(spots.len(), 2);
    assert_eq!((spots[0].left, spots[0].top), (0, 0));
    assert_eq!((spots[1].left, spots[1].top), (1, 1));
}
#[test]
fn radius_disk_has_strict_plus_one_support() {
    assert_eq!(circular_offsets(0.).unwrap(), vec![[0, 0]]);
    assert_eq!(
        circular_offsets(1.).unwrap(),
        vec![[0, -1], [-1, 0], [0, 0], [1, 0], [0, 1]]
    );
    assert_eq!(circular_offsets(-0.1).unwrap().len(), 5);
    assert!(circular_offsets(f32::NAN).is_err());
}
#[test]
fn small_images_and_constant_images_are_stable() {
    for (w, h) in [(1, 1), (1, 5), (5, 1), (3, 4)] {
        for value in [0, 128, 255] {
            let image = GrayImage::new(w, h, vec![value; w * h]).unwrap();
            assert_eq!(gaussian_filter(&image).unwrap().pixels, image.pixels);
            for radius in [-0.1, 0., 1., 4.] {
                assert_eq!(
                    close_with_radius(&image, radius).unwrap().pixels,
                    image.pixels
                );
            }
        }
    }
}
#[test]
fn prepared_empty_image_remains_empty_without_complete_beams_claim() {
    let image = GrayImage::new(17, 13, vec![255; 17 * 13]).unwrap();
    let output = prepare(
        &image,
        Parameters {
            max_stem: 3,
            effective_beam: 6.,
        },
        &[],
    )
    .unwrap();
    assert!(output.glyphs.is_empty());
    for mask in [
        &output.filtered_horizontal,
        &output.median,
        &output.gaussian,
        &output.header_erased,
        &output.closed,
        &output.binarized,
        &output.head_spots,
    ] {
        assert_eq!(mask.pixels, image.pixels);
    }
    assert_eq!(output.parameters.max_stem, 3);
}
#[test]
fn invalid_images_and_unbounded_work_fail_explicitly() {
    let bad = GrayImage {
        width: 1,
        height: 1,
        pixels: vec![],
    };
    assert!(gaussian_filter(&bad).is_err());
    let gray = GrayImage::new(1, 1, vec![100]).unwrap();
    assert!(filter_horizontal(&gray, 1).is_err());
    assert!(extract_spots(&gray).is_err());
    assert!(close(&gray, f64::INFINITY).is_err());
    assert!(close_with_radius(&gray, 65.).is_err());
    let big = GrayImage::new(1000, 1000, vec![255; 1_000_000]).unwrap();
    assert_eq!(
        close_with_radius(&big, 64.).unwrap_err(),
        "unsupported_beam_morph_work_budget"
    );
}
