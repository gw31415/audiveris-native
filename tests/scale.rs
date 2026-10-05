// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    scale::{self, Failure, Range},
    vertical,
};
#[test]
fn hilo_matches_128_unmodified_java_goldens() {
    let text = include_str!("../fixtures/scale/peak-goldens.txt");
    let mut count = 0;
    for line in text.lines().filter(|s| !s.starts_with('#')) {
        let fields: Vec<_> = line.split('|').collect();
        let threshold = fields[0].parse().unwrap();
        let values: Vec<i32> = fields[1].split(',').map(|v| v.parse().unwrap()).collect();
        let expected: Vec<Range> = fields[2]
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let r: Vec<usize> = s.split(',').map(|v| v.parse().unwrap()).collect();
                Range {
                    min: r[0],
                    main: r[1],
                    max: r[2],
                }
            })
            .collect();
        assert_eq!(
            scale::find_peaks(&values, threshold),
            expected,
            "golden {count}: {line}"
        );
        count += 1;
    }
    assert_eq!(count, 128);
}
fn analyze(bytes: &[u8]) -> scale::Analysis {
    let image = GrayImage::decode(bytes).unwrap();
    scale::analyze(&vertical::binarize(&image, |_, _, _| {}))
}
#[test]
fn clean_staff_and_two_population_results_match_java() {
    let clean = analyze(include_bytes!("../fixtures/scale/clean-16.pgm"))
        .result
        .unwrap();
    assert_eq!(
        clean.line,
        Range {
            min: 2,
            main: 2,
            max: 2
        }
    );
    assert_eq!(
        clean.interline,
        Range {
            min: 16,
            main: 16,
            max: 16
        }
    );
    assert!(clean.beam.extrapolated);
    assert_eq!(clean.beam.thickness, 9);
    let dual = analyze(include_bytes!("../fixtures/scale/two-populations.pgm"))
        .result
        .unwrap();
    assert_eq!(dual.interline.main, 24);
    assert_eq!(dual.small_interline.unwrap().main, 16);
    let near = analyze(include_bytes!("../fixtures/scale/nearby-merge.pgm"))
        .result
        .unwrap();
    assert_eq!(
        near.interline,
        Range {
            min: 16,
            main: 17,
            max: 18
        }
    );
}
#[test]
fn all_five_scale_failure_classes_match_java() {
    for (data, error) in [
        (
            include_bytes!("../fixtures/scale/blank.pgm").as_slice(),
            Failure::NoRegularLines,
        ),
        (
            include_bytes!("../fixtures/scale/sparse.pgm").as_slice(),
            Failure::TooFewBlackPixels,
        ),
        (
            include_bytes!("../fixtures/scale/no-black-peak.pgm").as_slice(),
            Failure::NoSignificantBlackLines,
        ),
        (
            include_bytes!("../fixtures/scale/too-low.pgm").as_slice(),
            Failure::InterlineTooLow,
        ),
        (
            include_bytes!("../fixtures/scale/too-high.pgm").as_slice(),
            Failure::InterlineTooHigh,
        ),
    ] {
        assert_eq!(analyze(data).result, Err(error));
    }
}
#[test]
fn histogram_limit_rounding_and_out_of_domain_runs_match_java() {
    let rounded = analyze(include_bytes!("../fixtures/scale/rounding-white.pgm"));
    assert_eq!(rounded.histograms.max_white, 80);
    let long = analyze(include_bytes!("../fixtures/scale/long-blacks.pgm"));
    assert_eq!(long.result.unwrap().line.main, 2);
    assert_eq!(long.histograms.black.len(), 49);
    assert_eq!(long.histograms.combo.len(), 199);
}
#[test]
fn beam_selection_and_inverted_signed_bounds_match_java() {
    let two = analyze(include_bytes!("../fixtures/scale/two-beams.pgm"))
        .result
        .unwrap();
    assert_eq!(two.beam.thickness, 16);
    assert!(!two.beam.extrapolated);
    assert_eq!(two.small_beam.unwrap().thickness, 12);
    // Preserve upstream's implausible-but-accepted outcome; do not turn unsigned underflow into panic.
    let inverted = analyze(include_bytes!(
        "../fixtures/scale/dominant-black-inverted-beam.pgm"
    ))
    .result
    .unwrap();
    assert_eq!(inverted.line.main, 70);
    assert_eq!(inverted.interline.main, 16);
    assert_eq!(inverted.beam.thickness, 48);
    assert!(inverted.beam.extrapolated);
}
