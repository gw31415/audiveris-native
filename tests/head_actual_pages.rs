// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn scratch(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "audiveris-actual-pages-{tag}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&p).unwrap();
    p
}
#[test]
fn independent_three_font_score_pages_match_original_complete_heads_terminals() {
    let temp = scratch("fonts");
    for family in ["Bravura", "Leland", "FinaleJazz"] {
        let name = format!("{family}-five");
        let input = root().join(format!(
            "fixtures/official-head-actual-pages/images/{name}.png"
        ));
        let pgm = temp.join(format!("{name}.pgm"));
        let report = temp.join(format!("{name}.json"));
        let native = Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
            .arg("heads-catalog-control")
            .arg(&input)
            .arg(&pgm)
            .arg(&report)
            .args([family, "51"])
            .output()
            .unwrap();
        assert!(
            native.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&native.stderr)
        );
        let comparison = Command::new("python3")
            .arg(root().join("scripts/check_head_actual_pages.py"))
            .arg(&report)
            .arg(&name)
            .output()
            .unwrap();
        assert!(
            comparison.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&comparison.stderr)
        );
    }
    std::fs::remove_dir_all(temp).unwrap();
}
#[test]
fn actual_cue_page_cli_matches_original_terminal_heads_and_preserves_incomplete_flags() {
    use audiveris_native::{GrayImage, clusters, filament_merge, scale, sections, vertical};
    let input = root().join("fixtures/official-head-actual-pages/images/Bravura-cue.png");
    let image = GrayImage::decode(&std::fs::read(&input).unwrap()).unwrap();
    let binary = vertical::binarize(&image, |_, _, _| {});
    let sc = scale::analyze(&binary).result.unwrap();
    assert_eq!(sc.interline.main, 11);
    assert_eq!(sc.small_interline.as_ref().unwrap().main, 7);
    let prep = sections::prepare(&binary, &sc).unwrap();
    let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
    let staff = clusters::analyze(&sc, &prep.horizontal, &factory);
    assert_eq!(staff["status"], "unsupported");
    assert_eq!(staff["error"], "multiple_interline_requires_second_pass");
    let temp = scratch("cue");
    for command in ["page-glyphs", "heads-catalog-control"] {
        let pgm = temp.join(format!("{command}.pgm"));
        let report = temp.join(format!("{command}.json"));
        let mut cli = Command::new(env!("CARGO_BIN_EXE_audiveris-native"));
        cli.arg(command).arg(&input).arg(&pgm).arg(&report);
        if command == "heads-catalog-control" {
            cli.args(["Bravura", "51"]);
        }
        let result = cli.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(pgm.exists() && report.exists());
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
        if command == "heads-catalog-control" {
            assert_eq!(value["head_registration"]["retained_heads"], 168);
            assert_eq!(value["head_registration"]["overlap_pairs"], 198);
        } else {
            assert_eq!(value["head_registration"]["retained_heads"], 156);
            assert_eq!(value["head_registration"]["overlap_pairs"], 189);
            assert_eq!(
                value["head_registration"]["head_registration_complete"],
                true
            );
            assert!(value.get("head_catalog_control").is_none());
            let comparison = Command::new("python3")
                .arg(root().join("scripts/check_cue_default_heads.py"))
                .arg(&report)
                .arg("cue")
                .output()
                .unwrap();
            assert!(
                comparison.status.success(),
                "{}",
                String::from_utf8_lossy(&comparison.stderr)
            );
        }
        assert_eq!(value["head_registration"]["heads_complete"], false);
        assert_eq!(value["musicxml_complete"], false);
        assert_eq!(value["later_beams"]["epilog"]["complete"], true);
        if command == "heads-catalog-control" {
            let code = "import sys; from pathlib import Path; from check_head_page_catalog import check_control; check_control(Path(sys.argv[1]),Path(sys.argv[2]))";
            let comparison = Command::new("python3")
                .arg("-c")
                .arg(code)
                .arg(&report)
                .arg(root().join("fixtures/official-head-actual-pages/Bravura-cue.json"))
                .env("PYTHONPATH", root().join("scripts"))
                .output()
                .unwrap();
            assert!(
                comparison.status.success(),
                "{}",
                String::from_utf8_lossy(&comparison.stderr)
            );
        }
    }
    std::fs::remove_dir_all(temp).unwrap();
}
