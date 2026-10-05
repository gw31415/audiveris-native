use audiveris_template_bank::*;
use audiveris_template_match::{Limits, Storage, Target, Weights, compute};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
const MANIFEST: &[u8] = include_bytes!("../assets/manifest.json");
const PAYLOAD: &[u8] = include_bytes!("../assets/templates.json");
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn resigned(payload: &[u8], change: impl FnOnce(&mut Value)) -> Result<Bank, Error> {
    let mut m: Value = serde_json::from_slice(MANIFEST).unwrap();
    m["asset_sha256"] = hash(payload).into();
    change(&mut m);
    let m = serde_json::to_vec(&m).unwrap();
    let h = hash(&m);
    Bank::from_bytes(
        &m,
        payload,
        Trust {
            manifest_sha256: &h,
            ..BUILTIN_TRUST
        },
    )
}
fn mutated(change: impl FnOnce(&mut Value)) -> Result<Bank, Error> {
    let mut p: Value = serde_json::from_slice(PAYLOAD).unwrap();
    change(&mut p);
    resigned(&serde_json::to_vec(&p).unwrap(), |_| {})
}
#[test]
fn official_coverage_calibration_and_scores() {
    let bank = Bank::load_builtin().unwrap();
    assert_eq!(bank.manifest_info().keys.len(), 4);
    assert_eq!(bank.point_size_for_width(15.).unwrap(), 51);
    assert_eq!(Bank::staff_point_size(11., 11., 51).unwrap(), 51);
    let j: Value = serde_json::from_str(include_str!("../oracle/generated/assets.json")).unwrap();
    for a in j["assets"].as_array().unwrap() {
        let asset = bank.get(a["shape"].as_str().unwrap(), 51).unwrap();
        assert_eq!(
            asset.dimensions(),
            (
                a["width"].as_u64().unwrap() as u32,
                a["height"].as_u64().unwrap() as u32
            )
        );
        let pixels = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("oracle/generated")
                .join(a["file"].as_str().unwrap()),
        )
        .unwrap();
        let (w, h) = asset.dimensions();
        let t = compute(
            w as usize,
            h as usize,
            &pixels,
            Target::Foreground,
            Storage::JavaShort,
            Limits::default(),
        )
        .unwrap();
        let score = asset
            .template()
            .evaluate(&t, 0, 0, None, Weights::default())
            .unwrap()
            .score;
        assert_eq!(score, a["scores"][0]["score"].as_f64().unwrap());
        for anchor in a["anchors"].as_array().unwrap() {
            let got = asset.anchor(anchor["name"].as_str().unwrap()).unwrap();
            assert_eq!(got.x, anchor["x"].as_f64().unwrap());
            assert_eq!(got.y, anchor["y"].as_f64().unwrap());
        }
    }
}
#[test]
fn unsupported_does_not_interpolate() {
    let b = Bank::load_builtin().unwrap();
    for size in [0, 24, 44, 50, 52, 64, 65535] {
        assert!(matches!(
            b.get("NOTEHEAD_BLACK", size),
            Err(Error::UnsupportedTemplate)
        ));
    }
    for shape in [
        "notehead_black",
        "Notehead_Black",
        "NOTEHEAD_CROSS",
        "WHOLE_NOTE_SMALL",
    ] {
        assert!(matches!(b.get(shape, 51), Err(Error::UnsupportedTemplate)));
    }
    assert!(matches!(
        b.point_size_for_width(8.),
        Err(Error::UnsupportedCalibration)
    ));
    assert!(b.point_size_for_width(f64::NAN).is_err());
    assert!(b.point_size_for_width(f64::INFINITY).is_err());
}
#[test]
fn integrity_identity_and_raw_field_duplicates() {
    let mut bad = PAYLOAD.to_vec();
    bad[0] = b' ';
    assert!(matches!(
        Bank::from_bytes(MANIFEST, &bad, BUILTIN_TRUST),
        Err(Error::Integrity)
    ));
    assert!(matches!(
        Bank::from_bytes(
            MANIFEST,
            PAYLOAD,
            Trust {
                font_sha256: "wrong",
                ..BUILTIN_TRUST
            }
        ),
        Err(Error::Identity)
    ));
    assert!(matches!(
        Bank::from_bytes(
            MANIFEST,
            PAYLOAD,
            Trust {
                model_sha256: "wrong",
                ..BUILTIN_TRUST
            }
        ),
        Err(Error::Identity)
    ));
    assert!(matches!(
        Bank::from_bytes(
            MANIFEST,
            PAYLOAD,
            Trust {
                coverage_input_sha256: "wrong",
                ..BUILTIN_TRUST
            }
        ),
        Err(Error::Identity)
    ));
    let duplicate = String::from_utf8(PAYLOAD.to_vec()).unwrap().replacen(
        "\"schema\":1",
        "\"schema\":1,\"schema\":1",
        1,
    );
    assert!(matches!(
        resigned(duplicate.as_bytes(), |_| {}),
        Err(Error::Json(_))
    ));
    assert!(resigned(PAYLOAD, |m| m["Schema"] = json!(1)).is_err());
}
#[test]
fn duplicate_case_collisions_and_missing_keys() {
    assert!(mutated(|p| p["templates"][1] = p["templates"][0].clone()).is_err());
    assert!(mutated(|p| p["templates"][0]["key"]["shape"] = json!("breve")).is_err());
    assert!(
        resigned(PAYLOAD, |m| {
            let k = m["keys"][0].clone();
            m["keys"].as_array_mut().unwrap().push(k);
        })
        .is_err()
    );
    assert!(
        resigned(PAYLOAD, |m| {
            let mut k = m["keys"][0].clone();
            k["shape"] = json!("breve");
            m["keys"].as_array_mut().unwrap().push(k);
        })
        .is_err()
    );
    assert!(
        mutated(|p| {
            p["templates"].as_array_mut().unwrap().pop();
        })
        .is_err()
    );
    assert!(
        mutated(|p| {
            p["calibration"].as_array_mut().unwrap().pop();
        })
        .is_err()
    );
    assert!(mutated(|p| p["calibration"][1] = p["calibration"][0].clone()).is_err());
}
#[test]
fn invalid_dimensions_points_anchors_and_numbers() {
    for v in [0, 2049, u32::MAX] {
        assert!(mutated(|p| p["templates"][0]["width"] = json!(v)).is_err());
    }
    assert!(mutated(|p| p["templates"][0]["slim_bounds"] = json!([u32::MAX, 0, 10, 10])).is_err());
    assert!(mutated(|p| p["templates"][0]["points"][0]["x"] = json!(2048)).is_err());
    assert!(
        mutated(|p| p["templates"][0]["points"][1] = p["templates"][0]["points"][0].clone())
            .is_err()
    );
    assert!(mutated(|p| p["templates"][0]["anchors"][0]["x"] = json!(-1)).is_err());
    assert!(
        mutated(|p| p["templates"][0]["anchors"][1] = p["templates"][0]["anchors"][0].clone())
            .is_err()
    );
    assert!(
        mutated(|p| {
            p["templates"][0]["anchors"].as_array_mut().unwrap().pop();
        })
        .is_err()
    );
    for invalid in ["NaN", "Infinity", "1e400"] {
        let mut f = String::from_utf8(PAYLOAD.to_vec()).unwrap();
        let start = f.find("\"d\":").unwrap() + 4;
        let end = start + f[start..].find([',', '}']).unwrap();
        f.replace_range(start..end, invalid);
        assert!(matches!(
            resigned(f.as_bytes(), |_| {}),
            Err(Error::Json(_))
        ));
        let s = String::from_utf8(PAYLOAD.to_vec()).unwrap().replacen(
            "\"width\":35",
            &format!("\"width\":{invalid}"),
            1,
        );
        assert!(matches!(
            resigned(s.as_bytes(), |_| {}),
            Err(Error::Json(_))
        ));
    }
    assert!(mutated(|p| p["calibration"][0]["width"] = json!(0)).is_err());
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("audiveris-bank-{}-{n}", std::process::id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn bounded_files_missing_files_and_symlinks() {
    assert!(matches!(
        Bank::from_bytes(MANIFEST, &vec![0; MAX_ASSET_BYTES + 1], BUILTIN_TRUST),
        Err(Error::TooLarge)
    ));
    assert!(matches!(
        Bank::from_bytes(&vec![0; MAX_MANIFEST_BYTES + 1], PAYLOAD, BUILTIN_TRUST),
        Err(Error::TooLarge)
    ));
    let dir = Temp::new();
    fs::write(dir.0.join("manifest.json"), MANIFEST).unwrap();
    assert!(matches!(
        Bank::load_dir(&dir.0, BUILTIN_TRUST),
        Err(Error::Io(_))
    ));
    let file = fs::File::create(dir.0.join("templates.json")).unwrap();
    file.set_len(MAX_ASSET_BYTES as u64 + 1).unwrap();
    drop(file);
    assert!(matches!(
        Bank::load_dir(&dir.0, BUILTIN_TRUST),
        Err(Error::TooLarge)
    ));
    fs::write(dir.0.join("templates.json"), PAYLOAD).unwrap();
    assert!(Bank::load_dir(&dir.0, BUILTIN_TRUST).is_ok());
    #[cfg(unix)]
    {
        fs::remove_file(dir.0.join("templates.json")).unwrap();
        std::os::unix::fs::symlink("manifest.json", dir.0.join("templates.json")).unwrap();
        assert!(matches!(
            Bank::load_dir(&dir.0, BUILTIN_TRUST),
            Err(Error::NotRegularFile)
        ));
    }
}
