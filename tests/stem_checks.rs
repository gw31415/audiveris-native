// SPDX-License-Identifier: AGPL-3.0-or-later
// Native StemChecker controls and original-context comparisons.
use audiveris_native::stem_checks;
pub use audiveris_native::{GrayImage, MAX_PIXELS};
use stem_checks::{Candidate, Parameters, check};

fn plain(profile: u8) -> (GrayImage, Candidate, Parameters) {
    let mut image = GrayImage::new(80, 100, vec![255; 8000]).unwrap();
    for y in 10..66 {
        for x in 20..22 {
            image.pixels[y * 80 + x] = 0;
        }
    }
    (
        image,
        Candidate {
            bounds: [20, 10, 2, 56],
            start: [20.5, 10.0],
            stop: [20.5, 65.0],
            line: [20.5, 9.5, 20.5, 65.5],
            mean_distance: 0.0,
        },
        Parameters::new(16, 3, 0.0, profile).unwrap(),
    )
}
#[test]
fn clean_vertical_both_profiles_reach_intrinsic_ceiling() {
    for profile in [0, 1] {
        let (image, c, p) = plain(profile);
        let r = check(&image, &c, &p).unwrap();
        assert_eq!(
            (r.context_black, r.context_white, r.context_gap),
            (56, 0, 0)
        );
        assert_eq!(r.grade, 0.8);
        assert!(r.accepted);
        assert!(r.checks.iter().all(|c| c.impact == 1.0));
    }
}
#[test]
fn profile_gap_and_endpoint_sampling_follow_source() {
    let (mut image, c, p) = plain(1);
    for y in 20..23 {
        for x in 20..22 {
            image.pixels[y * 80 + x] = 255;
        }
    }
    // End rows deliberately count as black even if physically white.
    for y in [10, 65] {
        for x in 20..22 {
            image.pixels[y * 80 + x] = 255;
        }
    }
    let r = check(&image, &c, &p).unwrap();
    assert_eq!(
        (r.context_black, r.context_white, r.context_gap),
        (53, 3, 3)
    );
    assert!((r.checks[6].impact - 0.375).abs() < 1e-15);
    let p0 = Parameters::new(16, 3, 0.0, 0).unwrap();
    assert_eq!(check(&image, &c, &p0).unwrap().grade, 0.0);
}
#[test]
fn clinging_ink_reduces_clean_but_not_black_rows() {
    let (mut image, c, p) = plain(1);
    for y in 10..66 {
        for x in 18..24 {
            image.pixels[y * 80 + x] = 0;
        }
    }
    let r = check(&image, &c, &p).unwrap();
    assert_eq!(r.context_black, 56);
    assert_eq!(r.checks[3].raw_value, 0.0);
    assert_eq!(r.grade, 0.0);
    assert!(!r.accepted);
}
#[test]
fn invalid_geometry_profile_and_nonfinite_input_fail_explicitly() {
    assert!(Parameters::new(16, 3, 0.0, 4).is_err());
    assert!(Parameters::new(0, 3, 0.0, 1).is_err());
    let (image, mut c, p) = plain(1);
    c.mean_distance = f64::NAN;
    assert!(check(&image, &c, &p).is_err());
    c.mean_distance = 0.;
    c.bounds[0] = -1;
    assert!(check(&image, &c, &p).is_err());
}

#[test]
fn original_356_check_contexts_match_independent_pixels_and_geometry() {
    use serde_json::Value;
    use std::{collections::BTreeMap, fs, path::PathBuf};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let params: Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/official-stem-seeds-parameters.json")).unwrap(),
    )
    .unwrap();
    let slopes: BTreeMap<_, _> = params["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["name"].as_str().unwrap().to_owned(),
                v["global_slope"].as_f64().unwrap(),
            )
        })
        .collect();
    let mut files: Vec<_> = fs::read_dir(root.join("fixtures/official-stem-seeds"))
        .unwrap()
        .map(|v| v.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 12);
    let array2 = |v: &Value| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()];
    let mut count = 0;
    let mut max_error = 0.0_f64;
    for path in files {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let d: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let image = GrayImage::decode(
            &fs::read(root.join(format!("fixtures/official-stem-seeds-pixels/{name}.png")))
                .unwrap(),
        )
        .unwrap();
        for system in d["systems"].as_array().unwrap() {
            for input in system["candidates"].as_array().unwrap() {
                let expected = &input["replayed_original_check"];
                if expected.is_null() {
                    continue;
                }
                let c = Candidate {
                    bounds: std::array::from_fn(|i| input["bounds"][i].as_i64().unwrap()),
                    start: array2(&input["start"]),
                    stop: array2(&input["stop"]),
                    line: std::array::from_fn(|i| input["line"][i].as_f64().unwrap()),
                    mean_distance: input["mean_distance"].as_f64().unwrap(),
                };
                let p = Parameters::new(
                    d["interline"].as_u64().unwrap() as usize,
                    d["stem_scale"]["max"].as_u64().unwrap() as usize,
                    slopes[name],
                    expected["profile"].as_u64().unwrap() as u8,
                )
                .unwrap();
                let actual = check(&image, &c, &p).unwrap();
                assert_eq!(
                    actual.context_black,
                    expected["context_black"].as_u64().unwrap() as usize,
                    "{name} {:?}",
                    c.bounds
                );
                assert_eq!(
                    actual.context_white,
                    expected["context_white"].as_u64().unwrap() as usize,
                    "{name} {:?}",
                    c.bounds
                );
                assert_eq!(
                    actual.context_gap,
                    expected["context_gap"].as_u64().unwrap() as usize,
                    "{name} {:?}",
                    c.bounds
                );
                for (i, check) in actual.checks.iter().enumerate() {
                    let raw = &expected["raw_checks"][i];
                    let impact = &expected["impacts"]["checks"][i];
                    assert_eq!(check.name, raw["name"].as_str().unwrap());
                    assert_eq!(check.covariant, raw["covariant"].as_bool().unwrap());
                    for (got, want) in [
                        (check.raw_value, raw["raw_value"].as_f64().unwrap()),
                        (check.low, raw["low"].as_f64().unwrap()),
                        (check.high, raw["high"].as_f64().unwrap()),
                        (check.weight, impact["weight"].as_f64().unwrap()),
                        (check.impact, impact["impact"].as_f64().unwrap()),
                    ] {
                        let error = (got - want).abs();
                        max_error = max_error.max(error);
                        assert!(
                            error <= 1e-12,
                            "{name} {:?} {} got {got:?}, want {want:?}",
                            c.bounds,
                            check.name
                        );
                    }
                }
                let error = (actual.grade - expected["impacts"]["grade"].as_f64().unwrap()).abs();
                max_error = max_error.max(error);
                assert!(
                    error <= 1e-12,
                    "{name} {:?} grade {}",
                    c.bounds,
                    actual.grade
                );
                assert_eq!(
                    actual.accepted,
                    input["accepted_seed"].as_bool().unwrap(),
                    "{name} {:?}",
                    c.bounds
                );
                count += 1;
            }
        }
    }
    assert_eq!(count, 356);
    eprintln!("StemChecker: {count} contexts, maximum float delta {max_error:e}");
}

#[test]
fn valid_side_belt_linear_index_wrap_is_not_silently_clamped() {
    let mut image = GrayImage::new(80, 100, vec![255; 8000]).unwrap();
    for y in 10..66 {
        for x in 0..2 {
            image.pixels[y * 80 + x] = 0;
        }
    }
    let c = Candidate {
        bounds: [0, 10, 2, 56],
        start: [0.5, 10.0],
        stop: [0.5, 65.0],
        line: [0.5, 9.5, 0.5, 65.5],
        mean_distance: 0.0,
    };
    let p = Parameters::new(16, 3, 0.0, 1).unwrap();
    assert_eq!(check(&image, &c, &p).unwrap().grade, 0.8);
    // x=-1,-2 at row y address x=79,78 on row y-1 in ByteProcessor.get.
    for y in 9..65 {
        for x in 78..80 {
            image.pixels[y * 80 + x] = 0;
        }
    }
    let result = check(&image, &c, &p).unwrap();
    assert_eq!(result.context_black, 56);
    assert_eq!(result.checks[3].raw_value, 0.0);
    assert_eq!(result.grade, 0.0);
}
