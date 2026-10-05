use audiveris_native::GrayImage;
#[test]
fn grayscale_png_transparency_is_rejected() {
    for transparent in [false, true] {
        let mut bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut bytes, 32, 32);
            enc.set_color(png::ColorType::Grayscale);
            enc.set_depth(png::BitDepth::Eight);
            if transparent {
                enc.set_trns(vec![0, 0]);
            }
            enc.write_header()
                .unwrap()
                .write_image_data(&[0; 1024])
                .unwrap();
        }
        let result = GrayImage::decode(&bytes);
        if transparent {
            assert!(result.unwrap_err().contains("tRNS"));
        } else {
            assert_eq!(result.unwrap().pixels, vec![0; 1024]);
        }
    }
}
#[test]
fn blocked_filaments_does_not_claim_completion() {
    let dir = std::env::temp_dir().join(format!("audiveris-review-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("blank.pgm");
    let mut bytes = b"P5\n32 32\n255\n".to_vec();
    bytes.extend([255; 1024]);
    std::fs::write(&input, bytes).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
        .args([
            "filaments",
            input.to_str().unwrap(),
            dir.join("binary.pgm").to_str().unwrap(),
            dir.join("report.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&result.stdout).contains("FILAMENTS"));
    assert!(String::from_utf8_lossy(&result.stdout).contains("BINARY complete"));
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("report.json")).unwrap()).unwrap();
    assert!(report.get("filaments").is_none());
    std::fs::remove_dir_all(dir).unwrap();
}
