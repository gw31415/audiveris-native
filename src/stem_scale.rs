// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// StemScaler/HistoKeeper and HiLoPeakFinder at
// 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Runtime contract verified against the pinned official JAR: getBuffer paints a
// detached BufferedImage; the ByteProcessor passed to RunTableFactory is unchanged.
use crate::{GrayImage, MAX_PIXELS, key_peaks::Peak, position_runs, scale::Scale};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct StemScale {
    pub main: usize,
    pub max: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Histogram {
    pub x_min: usize,
    pub x_max: usize,
    pub values: Vec<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Parameters {
    pub min_value: i32,
    pub min_derivative: i32,
    pub min_gain_ratio: f64,
    pub quorum: i32,
    pub stem_as_fore_ratio: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Analysis {
    pub stage: &'static str,
    pub stem_scale: StemScale,
    pub histogram: Histogram,
    /// IntegerFunction.getArea: count of accepted runs, not count of ink pixels.
    pub area: i32,
    pub peak: Option<Peak>,
    /// Original descending main-count order, with stable first ties.
    pub peaks: Vec<Peak>,
    pub parameters: Parameters,
    pub fallback_used: bool,
    pub stem_scaling_complete: bool,
    pub stem_seeds_complete: bool,
}
fn validate_interline(interline: usize) -> Result<(), String> {
    if !(1..=128).contains(&interline) {
        return Err("unsupported_stem_scale_interline".into());
    }
    Ok(())
}
/// Horizontal foreground run counts over the unchanged NO_STAFF byte buffer.
/// A whole run longer than interline is discarded; it is never split into bins.
pub fn histogram(no_staff: &GrayImage, interline: usize) -> Result<Histogram, String> {
    validate_interline(interline)?;
    if no_staff.width == 0
        || no_staff.height == 0
        || no_staff
            .width
            .checked_mul(no_staff.height)
            .is_none_or(|n| n > MAX_PIXELS || n != no_staff.pixels.len())
        || no_staff.pixels.iter().any(|&p| p != 0 && p != 255)
    {
        return Err("invalid_stem_scale_binary_image".into());
    }
    let mut values = vec![0i32; interline + 1];
    for y in 0..no_staff.height {
        for run in position_runs(no_staff, false, y) {
            if run.length <= interline {
                values[run.length] = values[run.length]
                    .checked_add(1)
                    .ok_or("unsupported_stem_histogram_overflow")?;
            }
        }
    }
    Ok(Histogram {
        x_min: 0,
        x_max: interline,
        values,
    })
}

/// Stem histogram bins can exceed the signed-short projection bound in key_peaks.
/// This local source translation accepts nonnegative Java-int counts after checking
/// their total area fits a Java int, so derivative and peak-area arithmetic is safe.
fn peaks(values: &[i32], parameters: Parameters) -> Vec<Peak> {
    let mut hilos = Vec::new();
    // Hi minimum and finished flag; Lo maximum. Other original DerPeak fields
    // affect debugging only and are never consulted by the algorithm.
    let mut hi: Option<(usize, bool)> = None;
    let mut lo: Option<usize> = None;
    let make = |min: usize, max: usize| {
        let mut main = min;
        for x in min..=max {
            if values[x] > values[main] {
                main = x;
            }
        }
        Peak { min, main, max }
    };
    for x in 1..values.len() {
        let derivative = values[x] - values[x - 1];
        if derivative >= parameters.min_derivative {
            if let Some(max) = lo.take() {
                hilos.push(make(hi.take().unwrap().0, max));
            }
            if hi.is_none_or(|(_, finished)| finished) {
                hi = Some((x, false));
            }
        } else if derivative <= -parameters.min_derivative {
            if hi.is_some() {
                lo = Some(x);
            }
        } else if let Some(max) = lo.take() {
            hilos.push(make(hi.take().unwrap().0, max));
        } else if let Some((_, finished)) = hi.as_mut() {
            if values[x] < parameters.min_value {
                hi = None;
            } else {
                *finished = true;
            }
        }
    }
    // Original intentionally does not flush an unfinished terminal HiLo.
    let mut order = (0..hilos.len()).collect::<Vec<_>>();
    order.sort_by_key(|&i| std::cmp::Reverse(values[hilos[i].main]));
    let mut by_hilo = vec![None::<Peak>; hilos.len()];
    let mut found = Vec::new();
    for i in order {
        let hilo = hilos[i];
        let mut minimum = (hilo.min - 1).max(1);
        if i > 0 {
            minimum = minimum.max(by_hilo[i - 1].map_or(hilos[i - 1].max, |p| p.max) + 1);
        }
        let (mut lower, mut upper, mut total) = (hilo.main, hilo.main, values[hilo.main]);
        loop {
            let before = if lower == minimum {
                0
            } else {
                values[lower - 1]
            };
            let after = if upper == hilo.max {
                0
            } else {
                values[upper + 1]
            };
            let gain = before.max(after);
            // Match Java NaN >= threshold == false when total + gain is zero.
            if total + gain == 0
                || gain as f64 / ((total + gain) as f64) < parameters.min_gain_ratio
            {
                break;
            }
            if before > after {
                lower -= 1;
            } else {
                upper += 1;
            }
            total += gain;
        }
        let peak = Peak {
            min: lower,
            main: hilo.main,
            max: upper,
        };
        by_hilo[i] = Some(peak);
        found.push(peak);
    }
    found.sort_by_key(|p| std::cmp::Reverse(values[p.main]));
    found.retain(|p| values[p.main] >= parameters.quorum);
    found
}
fn compute(histogram: Histogram, foreground: StemScale) -> Result<Analysis, String> {
    validate_interline(histogram.x_max)?;
    if histogram.x_min != 0
        || histogram.values.len() != histogram.x_max + 1
        || histogram.values[0] != 0
        || histogram.values.iter().any(|&v| v < 0)
    {
        return Err("invalid_stem_histogram".into());
    }
    if foreground.main == 0 || foreground.main > foreground.max || foreground.max > 4_000_000 {
        return Err("invalid_stem_foreground_scale".into());
    }
    let area = histogram
        .values
        .iter()
        .try_fold(0i32, |sum, &v| sum.checked_add(v))
        .ok_or("unsupported_stem_histogram_area_overflow")?;
    let parameters = Parameters {
        min_value: 1,
        min_derivative: (area as f64 * 0.05).round_ties_even() as i32,
        min_gain_ratio: 0.1,
        quorum: (area as f64 * 0.1).round_ties_even() as i32,
        stem_as_fore_ratio: 1.0,
    };
    let peaks = peaks(&histogram.values, parameters);
    let peak = peaks.first().copied();
    let stem_scale = peak
        .map(|p| StemScale {
            main: p.main,
            max: p.max,
        })
        .unwrap_or(foreground);
    Ok(Analysis {
        stage: "STEM_SEEDS.stem_scale",
        stem_scale,
        histogram,
        area,
        peak,
        peaks,
        parameters,
        fallback_used: peak.is_none(),
        stem_scaling_complete: true,
        stem_seeds_complete: false,
    })
}
/// Native StemScaler on a fresh default-scale NO_STAFF buffer. No rendering,
/// header masking, bar erasure, Java execution, or oracle data lookup occurs here.
pub fn analyze(no_staff: &GrayImage, scale: &Scale) -> Result<Analysis, String> {
    if scale.line.min == 0
        || scale.line.min > scale.line.main
        || scale.interline.min == 0
        || scale.interline.min > scale.interline.main
        || scale.interline.main > scale.interline.max
        || scale.interline.max > 4_000_000
    {
        return Err("invalid_stem_input_scale".into());
    }
    compute(
        histogram(no_staff, scale.interline.main)?,
        StemScale {
            main: scale.line.main,
            max: scale.line.max,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scale(interline: usize) -> Scale {
        Scale {
            line: crate::scale::Range {
                min: 2,
                main: 2,
                max: 3,
            },
            interline: crate::scale::Range {
                min: interline,
                main: interline,
                max: interline,
            },
            small_interline: None,
            beam: crate::scale::Beam {
                thickness: 6,
                extrapolated: true,
            },
            small_beam: None,
        }
    }
    fn from_values(values: &[i32]) -> Analysis {
        compute(
            Histogram {
                x_min: 0,
                x_max: values.len() - 1,
                values: values.to_vec(),
            },
            StemScale { main: 2, max: 3 },
        )
        .unwrap()
    }
    #[test]
    fn independent_original_histogram_controls_are_exact() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../fixtures/official-grid/stem-scale-histogram-controls.json"
        ))
        .unwrap();
        let foreground = StemScale {
            main: oracle["foreground_main"].as_u64().unwrap() as usize,
            max: oracle["foreground_max"].as_u64().unwrap() as usize,
        };
        let cases = oracle["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        for case in cases {
            let values: Vec<i32> = serde_json::from_value(case["histogram"].clone()).unwrap();
            let actual = compute(
                Histogram {
                    x_min: 0,
                    x_max: values.len() - 1,
                    values,
                },
                foreground,
            )
            .unwrap();
            assert_eq!(
                serde_json::json!(actual.area),
                case["observed_area"],
                "{} area",
                case["name"]
            );
            assert_eq!(
                serde_json::json!(actual.peaks),
                case["original_peaks"],
                "{} peak order",
                case["name"]
            );
            assert_eq!(
                serde_json::json!(actual.peak),
                case["selected_peak"],
                "{} selected",
                case["name"]
            );
            assert_eq!(
                serde_json::json!(actual.stem_scale),
                case["stem_scale"],
                "{} scale",
                case["name"]
            );
            assert_eq!(actual.fallback_used, case["selected_peak"].is_null());
        }
    }
    #[test]
    fn horizontal_runs_use_whole_image_and_exclude_overlong_runs() {
        let image = GrayImage::new(
            13,
            2,
            vec![
                0, 0, 255, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255,
                255, 255, 255, 255,
            ],
        )
        .unwrap();
        assert_eq!(histogram(&image, 4).unwrap().values, [0, 1, 1, 1, 1]);
    }
    #[test]
    fn empty_and_terminal_peak_use_foreground_fallback() {
        for values in [vec![0; 5], vec![0, 0, 0, 0, 100], vec![0, 100, 0]] {
            let a = from_values(&values);
            assert!(a.fallback_used);
            assert_eq!(a.stem_scale, StemScale { main: 2, max: 3 });
        }
    }
    #[test]
    fn thresholds_use_run_count_and_half_even_rounding() {
        let a = from_values(&[0, 5, 0, 0, 0]);
        assert_eq!(a.area, 5);
        assert_eq!(a.parameters.quorum, 0);
        assert_eq!(a.parameters.min_derivative, 0);
        assert_eq!(a.stem_scale.main, 1);
        let b = from_values(&[0, 15, 0, 0, 0]);
        assert_eq!(b.parameters.quorum, 2);
        assert_eq!(b.parameters.min_derivative, 1);
        let c = from_values(&[0, 10, 0, 0, 0]);
        assert_eq!(c.parameters.min_derivative, 0);
    }
    #[test]
    fn equal_peaks_keep_first_hilo_and_large_counts_are_supported() {
        let a = from_values(&[0, 40000, 0, 0, 40000, 0, 0]);
        assert_eq!(a.peaks.iter().map(|p| p.main).collect::<Vec<_>>(), [1, 4]);
        assert_eq!(a.stem_scale.main, 1);
    }
    #[test]
    fn unchanged_no_staff_runtime_contract_has_visible_effect() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../fixtures/official-grid/stem-buffer-image-control.json"
        ))
        .unwrap();
        assert_eq!(oracle["before_base64"], "AP//AP//AP//AP//AP//AP//AP8=");
        assert_eq!(oracle["before_base64"], oracle["after_buffer_base64"]);
        assert_eq!(oracle["buffer_unchanged"], true);
        assert_eq!(
            oracle["painted_image_samples"],
            serde_json::json!(vec![255; 20])
        );
        // Original independent 5×4 control retains these seven black pixels even
        // though Graphics2D paints all twenty detached image samples white.
        let image = GrayImage::new(
            5,
            4,
            vec![
                0, 255, 255, 0, 255, 255, 0, 255, 255, 0, 255, 255, 0, 255, 255, 0, 255, 255, 0,
                255,
            ],
        )
        .unwrap();
        let before = image.pixels.clone();
        let a = analyze(&image, &scale(3)).unwrap();
        assert_eq!(image.pixels, before);
        assert_eq!(a.histogram.values, [0, 7, 0, 0]);
        assert_eq!(a.stem_scale.main, 1);
    }
    #[test]
    fn malformed_buffers_scales_and_overflow_fail_without_mutation() {
        let mut image = GrayImage::new(4, 4, vec![255; 16]).unwrap();
        image.pixels[0] = 128;
        assert!(analyze(&image, &scale(4)).is_err());
        image.pixels[0] = 255;
        image.width = usize::MAX;
        assert!(analyze(&image, &scale(4)).is_err());
        assert!(histogram(&GrayImage::new(1, 1, vec![255]).unwrap(), 129).is_err());
        let bad = Histogram {
            x_min: 0,
            x_max: 3,
            values: vec![0, i32::MAX, 1, 0],
        };
        assert!(compute(bad, StemScale { main: 2, max: 3 }).is_err());
        let bad = Histogram {
            x_min: 0,
            x_max: 3,
            values: vec![0, 1, -1, 0],
        };
        assert!(compute(bad, StemScale { main: 2, max: 3 }).is_err());
        assert!(
            compute(
                Histogram {
                    x_min: 0,
                    x_max: 3,
                    values: vec![0; 4]
                },
                StemScale { main: 3, max: 2 }
            )
            .is_err()
        );
    }
}
