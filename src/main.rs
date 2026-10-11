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
mod report;
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
      --report FILE   write a recognition report: per-page notes, assumed or corrected
                      time signatures, and measures whose durations do not add up
  -v, --verbose       print step timings
  -h, --help          show this help";

struct Args {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
    debug: Option<PathBuf>,
    report: Option<PathBuf>,
    verbose: bool,
    lang: String,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        inputs: Vec::new(),
        output: None,
        debug: None,
        report: None,
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
            "--report" => a.report = Some(it.next().ok_or("missing value for --report")?.into()),
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
    let mut pages = Vec::new();
    for input in &args.inputs {
        let sheet = pipeline::process(input, args.verbose, &args.lang)?;
        let mut page = score::build(&sheet, &mut state);
        if args.verbose {
            for note in &page.notes {
                eprintln!("{}: {note}", input.display());
            }
        }
        pages.push(report::Page {
            name: input.file_name().map_or(String::new(), |s| s.to_string_lossy().into_owned()),
            width: sheet.binary.width,
            height: sheet.binary.height,
            interline: sheet.scale.interline,
            systems: sheet.systems.len(),
            notes: std::mem::take(&mut page.notes),
        });
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
    if let Some(path) = &args.report {
        let (text, flagged) = report::write(&pages, &score);
        std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        if args.verbose {
            eprintln!("wrote {} ({flagged} flagged measures)", path.display());
        }
    }
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

    /// Der Winter (2/4) with its time signatures erased, or replaced by An
    /// Laura's 3/4: the time is taken from the measure contents and the
    /// result is the same as with the 2/4 read.
    #[test]
    fn time_signature_from_contents() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/real");
        let winter = image::open(dir.join("der-winter.png")).unwrap().to_luma8();
        let laura = image::open(dir.join("an-laura.png")).unwrap().to_luma8();
        let three = image::imageops::crop_imm(&laura, 343, 388, 24, 51).to_image();
        // Time signature columns of the first system (x, y of the 2/4).
        let spots = [(329, 501), (330, 639), (330, 758)];
        let read = |img: &image::GrayImage, name: &str| {
            let path = std::env::temp_dir().join(format!("audiveris-native-{name}-{}.png", std::process::id()));
            img.save(&path).unwrap();
            let sheet = pipeline::process(&path, false, "eng").unwrap();
            std::fs::remove_file(&path).ok();
            score::build(&sheet, &mut score::State::default())
        };
        let reference = read(&winter, "winter");
        assert!(reference.notes.is_empty());
        let mut erased = winter.clone();
        for &(x, y) in &spots {
            for yy in y - 3..y + 52 {
                for xx in x - 3..x + 22 {
                    erased.put_pixel(xx, yy, image::Luma([255]));
                }
            }
        }
        let mut replaced = erased.clone();
        for &(x, y) in &spots {
            image::imageops::replace(&mut replaced, &three, x as i64 - 3, y as i64 - 2);
        }
        for (img, name, note) in [(&erased, "erased", "no time signature read"), (&replaced, "replaced", "read as 3/4")] {
            let s = read(img, name);
            assert!(s.notes.iter().any(|n| n.contains(note)), "{name}: {:?}", s.notes);
            assert_eq!(musicxml::write(&s, "t"), musicxml::write(&reference, "t"), "{name}");
        }
        // The correction stays on its page: a following page really in 3/4 keeps it.
        let path = std::env::temp_dir().join(format!("audiveris-native-next-{}.png", std::process::id()));
        replaced.save(&path).unwrap();
        let mut state = score::State::default();
        score::build(&pipeline::process(&path, false, "eng").unwrap(), &mut state);
        std::fs::remove_file(&path).ok();
        let next = score::build(&pipeline::process(&dir.join("an-laura.png"), false, "eng").unwrap(), &mut state);
        let t = next.parts[0].measures[0].attributes.as_ref().and_then(|a| a.time).unwrap();
        assert_eq!((t.beats, t.beat_type), (3, 4));
    }

    /// The report lists the overfull measures of Der Winter by number and system.
    #[test]
    fn report_flags_measures() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/real/der-winter.png");
        let sheet = pipeline::process(&path, false, "eng").unwrap();
        let score = score::build(&sheet, &mut score::State::default());
        let page = report::Page {
            name: "der-winter.png".into(),
            width: sheet.binary.width,
            height: sheet.binary.height,
            interline: sheet.scale.interline,
            systems: sheet.systems.len(),
            notes: Vec::new(),
        };
        let (text, flagged) = report::write(&[page], &score);
        assert_eq!(flagged, 2, "{text}");
        assert!(text.contains("measures: 9 (P1=9, P2=9), numbered 0..8"), "{text}");
        assert!(text.contains("m5 (page 1, S02): P1 v1 overfull (15/4 of 2 quarters)"), "{text}");
        assert!(text.contains("m7 (page 1, S02): P1 v1 overfull"), "{text}");
    }
}
