use audiveris_glyph_features as features;
use audiveris_staff_removal::{
    BinaryPage, GlyphPixels, Limits, Orientation, Run, erase_staff, extract_glyphs,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join("../../fixtures/binary.png"))?;
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info()?;
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels)?;
    assert_eq!(info.color_type, png::ColorType::Grayscale);
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(
        root.join("../../fixtures/official-staff-glyphs.json"),
    )?)?;
    let staff_glyphs: Vec<_> = json
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
    let erased = erase_staff(
        BinaryPage::new(
            info.width as usize,
            info.height as usize,
            &pixels[..info.buffer_size()],
        )?,
        &staff_glyphs,
        Limits::default(),
    )?;
    assert_eq!(erased.removed_pixels, 54_466);
    let glyphs = extract_glyphs(
        BinaryPage::new(info.width as usize, info.height as usize, &erased.residual)?,
        Orientation::Vertical,
        (0, 0),
        Limits::default(),
    )?;
    let model = audiveris_classifier::Model::from_json(&std::fs::read(
        root.join("../../../audiveris-classifier/models/audiveris-5.11.0.json"),
    )?)?;
    let mut evaluated = 0;
    let mut rejected = Vec::new();
    for (i, g) in glyphs.iter().enumerate() {
        let geometry = features::Geometry {
            width: g.width,
            height: g.height,
            left: g.left,
            top: g.top,
            interline: 11,
            orientation: match g.orientation {
                Orientation::Horizontal => features::Orientation::Horizontal,
                Orientation::Vertical => features::Orientation::Vertical,
            },
        };
        let runs: Vec<_> = g
            .runs
            .iter()
            .map(|r| features::Run {
                position: r.position,
                start: r.start,
                length: r.length,
            })
            .collect();
        match features::from_runs(geometry, &runs) {
            Ok(raw) => {
                let evaluation = model.evaluate(&raw)?;
                assert_eq!(raw.len(), 110);
                assert_eq!(evaluation.scores.len(), 149);
                assert!(evaluation.scores.iter().all(|v| v.is_finite()));
                evaluated += 1;
            }
            Err(e) => rejected.push(serde_json::json!({"glyph_index":i,"error":e.to_string()})),
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"removed_pixels":erased.removed_pixels,"connected_glyphs":glyphs.len(),"evaluated_feature_and_classifier":evaluated,"rejected":rejected,"qualification":"Connected components are not guaranteed musical symbols; no page segmentation or musical correctness is claimed."})
        )?
    );
    Ok(())
}
