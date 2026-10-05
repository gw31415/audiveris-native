// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native image recognition to bounded MusicXML. No Java runtime or XML import.
use audiveris_native::{GrayImage, music_xml};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
const USAGE: &str = "audiveris-musicxml IMAGE OUTPUT.musicxml REPORT.json\naudiveris-musicxml --output-dir EXISTING_DIR [--part-name NAME] IMAGE...";
struct Job {
    image: PathBuf,
    xml: PathBuf,
    report: PathBuf,
}
fn destination(p: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let parent = p
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(fs::canonicalize(parent)?.join(p.file_name().ok_or("output filename required")?))
}
fn write_pair(
    xml: &Path,
    report: &Path,
    xml_data: &[u8],
    report_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut a = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(xml)?;
    let mut b = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(report)
    {
        Ok(b) => b,
        Err(e) => {
            drop(a);
            fs::remove_file(xml)?;
            return Err(e.into());
        }
    };
    let writes = a
        .write_all(xml_data)
        .and_then(|()| b.write_all(report_data))
        .and_then(|()| a.sync_all())
        .and_then(|()| b.sync_all());
    drop(a);
    drop(b);
    if let Err(e) = writes {
        // Remove only this job's create_new files; retain earlier batch outputs.
        let x = fs::remove_file(xml);
        let r = fs::remove_file(report);
        if x.is_err() || r.is_err() {
            return Err(format!(
                "output write failed: {e}; cleanup incomplete: XML={x:?}, report={r:?}"
            )
            .into());
        }
        return Err(e.into());
    }
    Ok(())
}
fn options(args: &[OsString]) -> Result<(Vec<Job>, String), Box<dyn std::error::Error>> {
    if args.first().is_none_or(|a| a != "--output-dir") {
        if args.len() != 3 {
            return Err(USAGE.into());
        }
        return Ok((
            vec![Job {
                image: args[0].clone().into(),
                xml: destination(Path::new(&args[1]))?,
                report: destination(Path::new(&args[2]))?,
            }],
            "Music".into(),
        ));
    }
    let directory = fs::canonicalize(args.get(1).ok_or(USAGE)?)?;
    if !directory.is_dir() {
        return Err("output-dir must be an existing directory".into());
    }
    let mut pos = 2;
    let mut name = String::from("Music");
    if args.get(pos).is_some_and(|a| a == "--part-name") {
        name = args
            .get(pos + 1)
            .and_then(|s| s.to_str())
            .ok_or("UTF8 part-name required")?
            .to_owned();
        pos += 2;
    }
    if pos == args.len() {
        return Err("at least one image required".into());
    }
    let mut jobs = Vec::new();
    for input in &args[pos..] {
        if input.to_str().is_some_and(|s| s.starts_with("--")) {
            return Err(
                "unknown option; image paths beginning -- require an explicit directory prefix"
                    .into(),
            );
        }
        let image = PathBuf::from(input);
        let stem = image.file_stem().ok_or("image filename required")?;
        let mut xml = stem.to_os_string();
        xml.push(".musicxml");
        let mut report = stem.to_os_string();
        report.push(".report.json");
        jobs.push(Job {
            image,
            xml: directory.join(xml),
            report: directory.join(report),
        });
    }
    Ok((jobs, name))
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() == 1 && args[0] == "--help" {
        println!(
            "{USAGE}\nClean opaque PNG/PGM, <=4M pixels, zero skew, ordinary five-line Bravura/interline11, native pitched header clef/C-major-or-header-key/recognized constant integer-quarter meter single voice, complete quarter/half notes and quarter/half rests; first-lower-ledger C4 only with retained concrete ledger; isolated/two-part quarter-half rest registration. Two/four-measure and system-break fixture validation; full HEADS/STEMS/OMR flags false. Ambiguous, incomplete or unsupported music refuses. Part name is metadata for a nontransposing part. Outputs must not exist. Batch preflights all paths; on recognition failure it stops, retaining earlier reported successful pairs."
        );
        return Ok(());
    }
    let (jobs, name) = options(&args)?;
    if name.trim().is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
        return Err("part-name requires a nonempty single line up to256 UTF8 bytes".into());
    }
    let mut outputs = BTreeSet::new();
    let inputs = jobs
        .iter()
        .map(|j| fs::canonicalize(&j.image))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for j in &jobs {
        for p in [&j.xml, &j.report] {
            if p.exists() || inputs.contains(p) || !outputs.insert(p.clone()) {
                return Err(
                    "all output paths must be distinct, not inputs, and must not exist".into(),
                );
            }
        }
    }
    for j in jobs {
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let mut bytes = Vec::new();
            fs::File::open(&j.image)?
                .take(32_000_001)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 32_000_000 {
                return Err("input exceeds32 MB".into());
            }
            let image = GrayImage::decode(&bytes)?;
            let result = music_xml::recognize_named(&image, &name)?;
            let mut data = serde_json::to_vec(&result.owner.owner.prepared.report)?;
            data.push(b'\n');
            write_pair(&j.xml, &j.report, result.xml.as_bytes(), &data)?;
            println!(
                "{}",
                serde_json::json!({"status":"written","input":j.image,"MusicXML":j.xml,"report":j.report})
            );
            Ok(())
        })();
        if let Err(e) = result {
            return Err(format!("{}: {e}", j.image.display()).into());
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("audiveris-musicxml: {e}");
        std::process::exit(1);
    }
}
