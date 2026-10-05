// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, template_probes, vertical};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn official_full_raster_distance_and_template_scores() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["laura-gray", "synthetic-system-bars"] {
        let image = GrayImage::decode(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}.png"))).unwrap(),
        )
        .unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        for mode in ["raw", "mask"] {
            let golden: Value = serde_json::from_slice(
                &std::fs::read(root.join(format!("fixtures/template-page/{name}-{mode}.json")))
                    .unwrap(),
            )
            .unwrap();
            let mask = (mode == "mask").then(|| {
                GrayImage::new(
                    image.width,
                    image.height,
                    (0..image.width * image.height)
                        .map(|i| {
                            if i % 997 == 0 || i / image.width == image.height / 2 {
                                0
                            } else {
                                255
                            }
                        })
                        .collect(),
                )
                .unwrap()
            });
            let actual = template_probes::evaluate(
                &binary,
                mask.as_ref(),
                &serde_json::to_vec(&golden["probes"]).unwrap(),
            )
            .unwrap();
            for key in ["width", "height", "binary_sha256", "distance_sha256_be_i16"] {
                assert_eq!(actual[key], golden["expected"][key], "{name} {mode} {key}");
            }
            let results = actual["results"].as_array().unwrap();
            let expected = golden["expected"]["results"].as_array().unwrap();
            assert_eq!(results.len(), expected.len());
            for (i, (a, b)) in results.iter().zip(expected).enumerate() {
                for key in ["score", "hole_white_ratio"] {
                    assert_eq!(a[key], b[key], "{name} {mode} {i} {key}");
                }
            }
            assert_eq!(actual["grid_complete"], false);
            assert_eq!(actual["note_recognition_complete"], false);
        }
    }
}

#[test]
fn input_contract_rejects_unsupported_and_malformed_requests() {
    let page = GrayImage::new(10, 10, vec![255; 100]).unwrap();
    let probe = json!({"shape":"NOTEHEAD_BLACK","point_size":51,"x":0,"y":0,"anchor":null});
    for request in [
        json!([]),
        json!([{"shape":"NOTEHEAD_BLACK","point_size":50,"x":0,"y":0}]),
        json!([{"shape":"NOTEHEAD_BLACK","point_size":51,"x":0,"y":0,"anchor":"wrong"}]),
        json!([{"shape":"OTHER","point_size":51,"x":0,"y":0}]),
        json!([{"shape":"NOTEHEAD_BLACK","point_size":51,"x":0,"y":0,"extra":true}]),
        json!(vec![probe.clone(); 4097]),
    ] {
        assert!(
            template_probes::evaluate(&page, None, &serde_json::to_vec(&request).unwrap()).is_err()
        );
    }
    assert!(
        template_probes::evaluate(
            &page,
            None,
            &vec![b' '; template_probes::MAX_REQUEST_BYTES + 1]
        )
        .is_err()
    );
    assert!(
        template_probes::distances(&page, Some(&GrayImage::new(1, 1, vec![255]).unwrap())).is_err()
    );
    assert!(
        template_probes::distances(&page, Some(&GrayImage::new(10, 10, vec![17; 100]).unwrap()))
            .is_err()
    );
    assert!(
        template_probes::distances(&GrayImage::new(10, 10, vec![17; 100]).unwrap(), None).is_err()
    );
    let mask = GrayImage::new(10, 10, vec![0; 100]).unwrap();
    let report = template_probes::evaluate(
        &page,
        Some(&mask),
        &serde_json::to_vec(&json!([probe])).unwrap(),
    )
    .unwrap();
    assert_eq!(report["results"][0]["score"].as_f64(), Some(f64::MAX));
    assert_eq!(report["results"][0]["included_points"], 0);
}

#[test]
fn cli_error_has_no_success_artifact_and_existing_output_is_preserved() {
    let dir = std::env::temp_dir().join(format!("audiveris-probes-test-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let image = dir.join("input.pgm");
    let request = dir.join("probes.json");
    let output = dir.join("output.json");
    std::fs::write(&image, GrayImage::new(2, 2, vec![255; 4]).unwrap().pgm()).unwrap();
    std::fs::write(
        &request,
        br#"[{"shape":"NOTEHEAD_BLACK","point_size":50,"x":0,"y":0}]"#,
    )
    .unwrap();
    let run = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
            .arg("template-probes")
            .arg(&image)
            .arg(&request)
            .arg(&output)
            .output()
            .unwrap()
    };
    let result = run();
    assert_eq!(result.status.code(), Some(1));
    assert!(!output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("UnsupportedTemplate"));
    std::fs::write(&output, b"preserve").unwrap();
    assert_eq!(run().status.code(), Some(1));
    assert_eq!(std::fs::read(&output).unwrap(), b"preserve");
    std::fs::remove_dir_all(dir).unwrap();
}
