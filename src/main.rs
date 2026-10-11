//! audiveris-native: score image(s) -> MusicXML, following the Audiveris
//! recognition pipeline.

use std::path::{Path, PathBuf};

mod beams;
mod binary;
mod classifier;
mod debug;
mod features;
mod grid;
mod header;
mod heads;
mod labels;
mod musicxml;
mod pipeline;
mod rational;
mod scale;
mod score;
mod staff;
mod stems;
mod symbols;
mod template;
mod text;
mod curves;

const USAGE: &str = "\
Usage: audiveris-native [OPTIONS] IMAGE...

Recognize score page image(s) (PNG, JPEG, TIFF, BMP) and write MusicXML.
Several images are processed as consecutive pages of one score.

Options:
  -o, --output FILE   MusicXML output (default: first image with .musicxml extension)
      --lang LANGS    OCR languages for lyrics (Tesseract names, default eng)
      --debug DIR     write per-page annotated images and recognition JSON into DIR
  -v, --verbose       print step timings
  -h, --help          show this help";

struct Args {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
    debug: Option<PathBuf>,
    verbose: bool,
    lang: String,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        inputs: Vec::new(),
        output: None,
        debug: None,
        verbose: false,
        lang: "eng".into(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(s) = it.next() {
        match s.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            "-o" | "--output" => {
                a.output = Some(it.next().ok_or("missing value for --output")?.into())
            }
            "--lang" => a.lang = it.next().ok_or("missing value for --lang")?,
            "--debug" => a.debug = Some(it.next().ok_or("missing value for --debug")?.into()),
            "-v" | "--verbose" => a.verbose = true,
            s if s.starts_with('-') => return Err(format!("unknown option {s}\n\n{USAGE}")),
            s => a.inputs.push(s.into()),
        }
    }
    if a.inputs.is_empty() {
        return Err(USAGE.into());
    }
    Ok(a)
}

fn write_debug(
    dir: &Path,
    stem: &str,
    sheet: &pipeline::Sheet,
    score: &score::Score,
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut ov = debug::Overlay::new(&sheet.binary);
    for s in &sheet.staves {
        for l in &s.lines {
            for x in s.left..=s.right {
                ov.dot(x, l.y_at(x as f64).round() as i32, [0, 0, 255]);
            }
        }
    }
    for sys in &sheet.systems {
        let top = sheet.staves[sys.staves[0]].top(sys.left as f64) as i32;
        let bot = sheet.staves[*sys.staves.last().unwrap()].bottom(sys.left as f64) as i32;
        for b in &sys.boundaries {
            ov.rect(
                binary::Rect::new(b.x0, top, b.x1 - b.x0 + 1, bot - top),
                [0, 200, 0],
            );
        }
    }
    for h in &sheet.headers {
        for (r, _) in &h.items {
            ov.rect(*r, [0, 150, 150]);
        }
    }
    for l in &sheet.ledgers {
        ov.line(l.x0 as f64, l.y, l.x1 as f64, l.y, [0, 160, 0]);
    }
    for st in &sheet.stems {
        let c = if st.up { [0, 180, 0] } else { [200, 0, 200] };
        ov.line(st.x as f64, st.top as f64, st.x as f64, st.bottom as f64, c);
        for &bi in &st.beams {
            let b = &sheet.beams[bi];
            ov.line(b.x0, b.y_at(b.x0), b.x1, b.y_at(b.x1), [255, 128, 0]);
        }
    }
    for h in &sheet.heads {
        let c = match h.shape {
            template::HeadShape::Black => [255, 0, 0],
            template::HeadShape::Void => [0, 0, 255],
            _ => [200, 0, 200],
        };
        ov.rect(h.bounds, c);
    }
    for s in &sheet.symbols {
        ov.rect(s.bounds.grow(1, 1), [0, 150, 255]);
    }
    for w in &sheet.wedges {
        ov.rect(w.bounds.grow(2, 2), [255, 0, 255]);
    }
    for s in &sheet.slurs {
        ov.line(s.left.0, s.left.1, s.right.0, s.right.1, [0, 200, 200]);
        ov.rect(s.bounds, [0, 200, 200]);
    }
    for e in &sheet.endings {
        ov.line(e.x0, e.y - 3.0, e.x1, e.y - 3.0, [255, 0, 255]);
    }
    ov.save(&dir.join(format!("{stem}.png")))?;
    debug::Overlay::new(&sheet.residual).save(&dir.join(format!("{stem}.residual.png")))?;
    let json = serde_json::json!({
        "scale": sheet.scale,
        "staves": sheet.staves,
        "systems": sheet.systems,
        "headers": sheet.headers,
        "heads": sheet.heads,
        "stems": sheet.stems,
        "beams": sheet.beams,
        "symbols": sheet.symbols,
        "wedges": sheet.wedges,
        "endings": sheet.endings,
        "slurs": sheet.slurs,
        "score": score,
    });
    std::fs::write(
        dir.join(format!("{stem}.json")),
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .map_err(|e| e.to_string())
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let mut book: Option<score::Score> = None;
    let mut state = score::State::default();
    for input in &args.inputs {
        let sheet = pipeline::process(input, args.verbose, &args.lang)?;
        let page = score::build(&sheet, &mut state);
        if let Some(dir) = &args.debug {
            let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
            write_debug(dir, stem, &sheet, &page)?;
        }
        match &mut book {
            None => book = Some(page),
            Some(b) => {
                for (i, p) in page.parts.into_iter().enumerate() {
                    match b.parts.get_mut(i) {
                        Some(bp) => {
                            bp.staves = bp.staves.max(p.staves);
                            bp.measures.extend(p.measures);
                        }
                        None => b.parts.push(p),
                    }
                }
            }
        }
    }
    let score = book.unwrap();
    let title = args.inputs[0]
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Score");
    let xml = musicxml::write(&score, title);
    let out = args
        .output
        .clone()
        .unwrap_or_else(|| args.inputs[0].with_extension("musicxml"));
    std::fs::write(&out, xml).map_err(|e| format!("{}: {e}", out.display()))?;
    if args.verbose {
        eprintln!("wrote {}", out.display());
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("audiveris-native: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end: the An Laura page gives 2 parts x 4 measures of valid
    /// music whose first voice notes are the expected ones.
    #[test]
    fn an_laura_end_to_end() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/real/an-laura.png");
        let sheet = pipeline::process(&path, false, "eng").unwrap();
        let score = score::build(&sheet, &mut score::State::default());
        assert_eq!(score.parts.len(), 2);
        assert!(score.parts.iter().all(|p| p.measures.len() == 4));
        assert_eq!(score.parts[1].staves, 2);
        let first: Vec<String> = score.parts[0].measures[0]
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                score::Kind::Chord { pitches, .. } => {
                    Some(format!("{}{}{}", pitches[0].step, pitches[0].alter, pitches[0].octave))
                }
                _ => None,
            })
            .collect();
        assert_eq!(first, ["A-14", "G04", "F04", "F04", "F04", "F04", "E04"]);
        let xml = musicxml::write(&score, "test");
        assert!(xml.contains("<fifths>-4</fifths>") && xml.contains("<beats>3</beats>"));
        // Every part opens with the 3/4 shown on all staves.
        for p in &score.parts {
            let t = p.measures[0].attributes.as_ref().and_then(|a| a.time).unwrap();
            assert_eq!((t.beats, t.beat_type), (3, 4));
        }
    }
}
