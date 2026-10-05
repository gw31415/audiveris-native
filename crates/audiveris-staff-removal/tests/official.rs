use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation, Run, erase_staff};
fn decode(bytes: &[u8]) -> (usize, usize, Vec<u8>) {
    let mut d = png::Decoder::new(std::io::Cursor::new(bytes));
    d.set_transformations(png::Transformations::EXPAND);
    let mut reader = d.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(info.color_type, png::ColorType::Grayscale);
    assert_eq!(info.bit_depth, png::BitDepth::Eight);
    pixels.truncate(info.buffer_size());
    (info.width as usize, info.height as usize, pixels)
}
#[test]
fn thirty_official_staff_glyphs_match_every_java_residual_and_removed_pixel() {
    let (w, h, binary) = decode(include_bytes!("../fixtures/binary.png"));
    let (_, _, expected) = decode(include_bytes!("../fixtures/official-no-staff.png"));
    let (_, _, mask) = decode(include_bytes!("../fixtures/official-removed.png"));
    let json: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/official-staff-glyphs.json")).unwrap();
    let glyphs: Vec<_> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|g| GlyphPixels {
            left: g["left"].as_i64().unwrap() as i32,
            top: g["top"].as_i64().unwrap() as i32,
            width: g["width"].as_u64().unwrap() as usize,
            height: g["height"].as_u64().unwrap() as usize,
            orientation: match g["orientation"].as_str().unwrap() {
                "HORIZONTAL" => Orientation::Horizontal,
                "VERTICAL" => Orientation::Vertical,
                _ => panic!(),
            },
            runs: g["runs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| Run {
                    position: r["position"].as_u64().unwrap() as usize,
                    start: r["start"].as_u64().unwrap() as usize,
                    length: r["length"].as_u64().unwrap() as usize,
                })
                .collect(),
        })
        .collect();
    assert_eq!(glyphs.len(), 30);
    let out = erase_staff(
        BinaryPage::new(w, h, &binary).unwrap(),
        &glyphs,
        Limits::default(),
    )
    .unwrap();
    assert_eq!(out.residual, expected);
    assert_eq!(out.removed_mask, mask);
    assert_eq!(out.removed_pixels, 54_466);
    assert_eq!(out.residual.iter().filter(|&&x| x == 0).count(), 135_311);
}

#[test]
fn official_glyphfactory_matches_connectivity_order_runs_boxes_origins_and_weights() {
    use audiveris_staff_removal::extract_glyphs;
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../oracle/golden-manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    for case in manifest["cases"].as_array().unwrap() {
        let (w, h, pixels) =
            decode(&std::fs::read(root.join(case["input"].as_str().unwrap())).unwrap());
        let o = match case["orientation"].as_str().unwrap() {
            "HORIZONTAL" => Orientation::Horizontal,
            "VERTICAL" => Orientation::Vertical,
            _ => panic!(),
        };
        let actual = extract_glyphs(
            BinaryPage::new(w, h, &pixels).unwrap(),
            o,
            (17, -9),
            Limits::default(),
        )
        .unwrap();
        let got:Vec<_>=actual.iter().map(|g|serde_json::json!({"left":g.left,"top":g.top,"width":g.width,"height":g.height,"weight":g.runs.iter().map(|r|r.length).sum::<usize>(),"runs":g.runs.iter().map(|r|[r.position,r.start,r.length]).collect::<Vec<_>>()})).collect();
        let expected: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(case["output"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        assert_eq!(
            serde_json::Value::Array(got),
            expected,
            "{} {}",
            case["input"],
            case["orientation"]
        );
    }
}
