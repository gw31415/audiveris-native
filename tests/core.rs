// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{AdaptiveFilter, GrayImage, runs};
#[test]
fn integral_matches_brute_force_at_every_pixel() {
    for (w, h) in [(1, 1), (1, 41), (41, 1), (37, 37), (57, 43)] {
        let pixels = (0..w * h)
            .map(|i| ((i * 73 + i / 7 * 17) % 256) as u8)
            .collect();
        let image = GrayImage::new(w, h, pixels).unwrap();
        let filter = AdaptiveFilter::new(&image);
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0_u64;
                let mut sq = 0_u64;
                let mut n = 0;
                for yy in y.saturating_sub(18)..(y + 19).min(h) {
                    for xx in x.saturating_sub(18)..(x + 19).min(w) {
                        let v = u64::from(image.pixels[yy * w + xx]);
                        sum += v;
                        sq += v * v;
                        n += 1;
                    }
                }
                let mean = sum as f64 / n as f64;
                let std = (sq as f64 / n as f64 - mean * mean).abs().sqrt();
                let c = filter.context(x, y);
                assert_eq!(c.mean, mean);
                assert_eq!(c.std_dev, std);
                assert_eq!(c.threshold, 0.7 * mean + 0.9 * std);
            }
        }
    }
}
#[test]
fn constant_black_white_and_inclusive_zero_threshold() {
    for (input, expected) in [(0, 0), (128, 255), (255, 255)] {
        let image = GrayImage::new(2, 2, vec![input; 4]).unwrap();
        assert_eq!(
            AdaptiveFilter::new(&image).binary().pixels,
            vec![expected; 4]
        );
    }
}
#[test]
fn run_tables_round_trip_in_both_orientations() {
    let image = GrayImage::new(4, 3, vec![0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 0, 0]).unwrap();
    for vertical in [false, true] {
        let mut restored = vec![255; 12];
        for (pos, row) in runs(&image, vertical).iter().enumerate() {
            for r in row {
                for c in r.start..r.start + r.length {
                    restored[if vertical { c * 4 + pos } else { pos * 4 + c }] = 0;
                }
            }
        }
        assert_eq!(restored, image.pixels);
    }
    assert_eq!(
        runs(&image, false)[0]
            .iter()
            .map(|r| (r.start, r.length))
            .collect::<Vec<_>>(),
        vec![(0, 2), (3, 1)]
    );
}
#[test]
fn pgm_preserves_binary_whitespace_pixels_and_rejects_bad_input() {
    let image = GrayImage::new(4, 1, vec![10, 32, 35, 255]).unwrap();
    assert_eq!(
        GrayImage::from_pgm(&image.pgm()).unwrap().pixels,
        image.pixels
    );
    assert_eq!(
        GrayImage::from_pgm(b"P2\n# c\n2 1\n255\n0 255")
            .unwrap()
            .pixels,
        vec![0, 255]
    );
    for bad in [
        b"P2 0 1 255".as_slice(),
        b"P2 1 1 255 256",
        b"P5 2 2 255\nabc",
        b"P2 1 1 65535 12",
        b"P2 1 1 255 0 1",
    ] {
        assert!(GrayImage::from_pgm(bad).is_err());
    }
}
#[test]
fn png_matches_pgm_without_color_conversion() {
    let png = GrayImage::decode(include_bytes!("../fixtures/original-exercise.png")).unwrap();
    let pgm = GrayImage::decode(include_bytes!("../fixtures/original-exercise.pgm")).unwrap();
    assert_eq!(png.width, pgm.width);
    assert_eq!(png.height, pgm.height);
    assert_eq!(png.pixels, pgm.pixels);
    assert!(GrayImage::decode(b"\x89PNG\r\n\x1a\ntruncated").is_err());
}
#[test]
fn vertical_ring_matches_full_integrals_across_wrap_and_degenerate_images() {
    for (w, h) in [
        (1, 1),
        (1, 79),
        (79, 1),
        (18, 37),
        (37, 18),
        (38, 43),
        (121, 87),
    ] {
        let image = GrayImage::new(
            w,
            h,
            (0..w * h)
                .map(|i| ((i * 73 + i / 7 * 17) % 256) as u8)
                .collect(),
        )
        .unwrap();
        let reference = AdaptiveFilter::new(&image);
        let output = audiveris_native::vertical::binarize(&image, |x, y, c| {
            let expected = reference.context(x, y);
            assert_eq!(c.mean, expected.mean);
            assert_eq!(c.std_dev, expected.std_dev);
            assert_eq!(c.threshold, expected.threshold);
        });
        assert_eq!(output.pixels, reference.binary().pixels);
        for v in [false, true] {
            for (p, r) in runs(&output, v).into_iter().enumerate() {
                assert_eq!(
                    audiveris_native::position_runs(&output, v, p).collect::<Vec<_>>(),
                    r
                );
            }
        }
        assert!(audiveris_native::vertical::scratch_bytes(w, h) <= 16 * w * h);
    }
    assert_eq!(
        audiveris_native::vertical::scratch_bytes(2000, 2000),
        1_216_000
    );
    assert_eq!(
        audiveris_native::vertical::scratch_bytes(1, 4_000_000),
        64_000_000
    );
}
