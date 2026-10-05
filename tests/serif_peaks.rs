// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, serif_raster};
use serde_json::Value;
#[test]
fn synthetic_original_accepted_and_rejected_serif_peaks_match() {
    let v: Value =
        serde_json::from_str(include_str!("../fixtures/official-serif-raster.json")).unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 75);
    let g = &v["geometry"];
    let w = g["width"].as_u64().unwrap() as usize;
    let h = g["height"].as_u64().unwrap() as usize;
    let rows = g["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| [r[0].as_i64().unwrap() as i32, r[1].as_i64().unwrap() as i32])
        .collect::<Vec<_>>();
    let thickness = g["line_thickness"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_f64().unwrap())
        .collect::<Vec<_>>();
    let mut positive = 0;
    for (i, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let mut pixels = vec![255; w * h];
        for r in c["rectangles"].as_array().unwrap() {
            let b: [i32; 4] = std::array::from_fn(|i| r[i].as_i64().unwrap() as i32);
            for y in b[1].max(0)..(b[1] + b[3]).min(h as i32) {
                for x in b[0].max(0)..(b[0] + b[2]).min(w as i32) {
                    pixels[y as usize * w + x as usize] = 0;
                }
            }
        }
        let image = GrayImage::new(w, h, pixels).unwrap();
        let projection = serif_raster::project(
            &image,
            &rows,
            std::array::from_fn(|n| g["abscissas"][n].as_i64().unwrap() as i32),
            g["interline"].as_u64().unwrap() as usize,
            g["fore"].as_u64().unwrap() as usize,
            &thickness,
        )
        .unwrap();
        let expected = c["projection_rle"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|r| {
                std::iter::repeat_n(
                    r[1].as_i64().unwrap() as i32,
                    r[0].as_u64().unwrap() as usize,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(projection.values, expected, "case {i} pixels");
        let t = &c["thresholds"];
        for (field, got) in [
            ("derivativeThreshold", projection.derivative_threshold),
            ("linesThreshold", projection.lines_threshold),
            ("blankThreshold", projection.blank_threshold),
            ("chunkThreshold", projection.chunk_threshold),
            ("barThreshold", projection.bar_threshold),
            ("verticalSerifWidth", projection.serif_width),
        ] {
            assert_eq!(got as i64, t[field].as_i64().unwrap(), "case {i} {field}");
        }
        let input = &c["input"];
        let search = serif_raster::search(
            &projection,
            input["x"].as_i64().unwrap() as i32,
            input["left"].as_bool().unwrap_or(true),
            input["added_chunk"].as_i64().unwrap_or(6) as i32,
        )
        .unwrap();
        if input["rectangles"].as_array().unwrap().is_empty() {
            assert!(
                search.bands.is_empty(),
                "blank synthetic input has seed bands"
            );
        }
        let middle = rows.iter().map(|r| (r[0] + r[1]) / 2).collect::<Vec<_>>();
        let mut detector = audiveris_native::serif_peaks::Detector::new(
            &image,
            audiveris_native::serif_peaks::StaffGeometry {
                rows: &rows,
                middle: &middle,
                abscissas: std::array::from_fn(|n| g["abscissas"][n].as_i64().unwrap() as i32),
                interline: g["interline"].as_u64().unwrap() as usize,
                fore: g["fore"].as_u64().unwrap() as usize,
                line_thickness: &thickness,
            },
        )
        .unwrap();
        let actual = detector
            .detect(
                input["x"].as_i64().unwrap() as i32,
                input["left"].as_bool().unwrap_or(true),
                input["added_chunk"].as_i64().unwrap_or(6) as i32,
            )
            .unwrap();
        // Feed the same original side control through native admission and
        // the integrated discovery boundary. The opposite side is an additional
        // native search, not an independently witnessed complete-rest control.
        let input_x = input["x"].as_i64().unwrap() as i32;
        let input_left = input["left"].as_bool().unwrap_or(true);
        let bx = if input_left { input_x } else { input_x - 100 };
        let beam = audiveris_native::multiple_rest_detection::BeamInput {
            id: i + 1,
            kind: "BeamInter",
            removed: false,
            bounds: [bx, 149, 101, 8],
            median: [bx as f64, 153., (bx + 100) as f64, 153.],
            height: 8.,
            grade: 0.72,
            staff: Some(audiveris_native::multiple_rest_admission::StaffPosition {
                id: 1,
                tablature: false,
                endpoint_pitches: [0., 0.],
                mid_line_thickness: 2.,
            }),
        };
        let mut witnessed = None;
        let _plan = audiveris_native::multiple_rest_detection::discover(&beam, 16, 2, |_, q| {
            let found = detector.detect(q.x, q.left, q.added_chunk)?;
            if q.left == input_left {
                witnessed = Some(found.clone());
            }
            Ok(found)
        })
        .unwrap();
        assert_eq!(
            witnessed.unwrap(),
            actual,
            "case {i} admitted original side"
        );
        let original = c["accepted_peaks"].as_array().unwrap();
        assert_eq!(
            actual.len(),
            original.len(),
            "synthetic case {i} acceptance"
        );
        for (got, expected) in actual.iter().zip(original) {
            for (name, value) in [
                ("start", got.start),
                ("stop", got.stop),
                ("top", got.top),
                ("bottom", got.bottom),
            ] {
                assert_eq!(
                    value as i64,
                    expected[name].as_i64().unwrap(),
                    "synthetic case {i} {name}"
                );
            }
            assert!(
                (got.grade - expected["grade"].as_f64().unwrap()).abs() < 1e-10,
                "synthetic case {i} grade {} vs {}",
                got.grade,
                expected["grade"]
            );
        }
        for peak in c["accepted_peaks"].as_array().unwrap() {
            positive += 1;
            let a = peak["start"].as_i64().unwrap() as i32;
            let b = peak["stop"].as_i64().unwrap() as i32;
            assert!(
                search.bands.iter().any(|r| r[0] <= b && r[1] >= a),
                "case {i} original accepted peak has no candidate seed"
            );
            assert!(peak["grade"].as_f64().unwrap() > 0.);
        }
    }
    assert_eq!(positive, 42);
}
#[test]
fn malformed_rows_and_search_ranges_reject() {
    let image = GrayImage::new(8, 8, vec![255; 64]).unwrap();
    assert!(serif_raster::project(&image, &[], [0, 7], 16, 2, &[2.; 5]).is_err());
    let p = serif_raster::project(&image, &[[1, 7]; 8], [0, 7], 16, 2, &[2.; 5]).unwrap();
    assert!(serif_raster::search(&p, 0, true, 6).is_err());
    assert!(serif_raster::search(&p, i32::MAX, true, i32::MIN).is_err());
}
