// SPDX-License-Identifier: AGPL-3.0-or-later
use sha2::{Digest, Sha256};
#[test]
fn automatic_mask_cli_needs_no_probe_coordinates_and_preserves_outputs() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("audiveris-mask-test-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mask = dir.join("mask.pgm");
    let report = dir.join("report.json");
    let run = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
            .arg("staff-mask")
            .arg(root.join("fixtures/official-grid/bracket-pair.png"))
            .arg(&mask)
            .arg(&report)
            .output()
            .unwrap()
    };
    let result = run();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = std::fs::read(&mask).unwrap();
    let image = audiveris_native::GrayImage::decode(&bytes).unwrap();
    let r: serde_json::Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert_eq!(r["schema"], "audiveris-native.experimental-staff-mask.v1");
    assert_eq!(
        r["note_preparation"]["mask_sha256"],
        format!("{:x}", Sha256::digest(&image.pixels))
    );
    assert_eq!(r["note_preparation"]["automatic_staff_mask"], true);
    assert_eq!(r["note_preparation"]["heads_mask_complete"], false);
    assert_eq!(r["grid_ir"]["grid_complete"], false);
    assert_eq!(run().status.code(), Some(1));
    assert_eq!(std::fs::read(&mask).unwrap(), bytes);
    std::fs::remove_dir_all(dir).unwrap();
}
