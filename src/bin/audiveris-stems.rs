// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image entry point retaining the unchanged native HEADS report as an output.
use audiveris_native::{
    GrayImage, stem_image_beam_generation, stem_image_beam_targets, stem_image_head_chunks,
    stem_image_head_parts, stem_image_head_plan, stem_image_linkers, stem_image_seeds,
    stem_image_sources, stem_image_stumps,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().collect();
    let music_symbols = args.get(1).is_some_and(|s| s == "--music-symbols");
    let music_measures = music_symbols || args.get(1).is_some_and(|s| s == "--music-measures");
    let head_foundations = music_measures || args.get(1).is_some_and(|s| s == "--head-foundations");
    let foundation_purge = args.get(1).is_some_and(|s| s == "--weak-foundations");
    let head_linking = args.get(1).is_some_and(|s| s == "--linked-head-stems");
    let head_expansion = args.get(1).is_some_and(|s| s == "--head-stems");
    let head_items = args.get(1).is_some_and(|s| s == "--head-items");
    let head_chunks = args.get(1).is_some_and(|s| s == "--head-chunks");
    let head_parts = args.get(1).is_some_and(|s| s == "--head-parts");
    let beam_generation = args.get(1).is_some_and(|s| s == "--beam-generation");
    let beam_targets = args.get(1).is_some_and(|s| s == "--beam-targets");
    let linkers = args.get(1).is_some_and(|s| s == "--head-linkers");
    let sources = args.get(1).is_some_and(|s| s == "--sources");
    let stumps = args.get(1).is_some_and(|s| s == "--stumps");
    let seeds = args.get(1).is_some_and(|s| s == "--seeds");
    if head_foundations
        || foundation_purge
        || head_linking
        || head_expansion
        || head_items
        || head_chunks
        || head_parts
        || beam_generation
        || beam_targets
        || linkers
        || sources
        || seeds
        || stumps
    {
        args.remove(1);
    }
    if args.len() == 2 && args[1] == "--help" {
        println!(
            "audiveris-stems [--music-symbols|--music-measures|--head-foundations|--weak-foundations|--linked-head-stems|--seeds|--stumps|--sources|--head-linkers|--beam-targets|--beam-generation|--head-parts|--head-chunks|--head-items|--head-stems] IMAGE RESIDUAL.pgm HEADS.json STEMS.json\n--music-symbols connects the strong SymbolsFilter/isolated-two-part CHECKED subset to the retained Graph and glyph registry; bounded Rest/TimeWhole glyph registration; broader compound symbols/flags/dots/rhythm/export pending.\n--music-measures connects single staff-barlines and single-stem head chords to the retained SIG/cache. Meter/duration/rests/rhythm/export pending.\n--head-foundations connects the bounded black/void head ending/side/competition pass to the actual live Graph and cache removals; quarter reduces to8/8/8. Complete overlap/chord/late consistency, cleaner/epilog/rhythm/export remain pending.\n--weak-foundations also applies original weak-candidate threshold to the retained live Graph; frozen/ledger retained, cached stems retained. Weak stem deletion refuses. Full overlap/chord reduction, candidate certainty, cleaner/epilog and MusicXML remain pending.\n--linked-head-stems runs original beam-first single-standard-beam builders and head linking phases1/2 using actual cache objects and the live SIG; siblings/hooks/alien limits refuse. HEADS raw grades are retained; XML-loaded comparison uses its three-decimal projection. Cleaner/reduction/rhythm/export and full STEMS remain pending.\n--head-stems expands native image head builders and creates selected stem glyphs in actual per-system caches at the inspection boundary; SIG scheduling/attachment/cleaner/epilog pending.\n--head-items also constructs final head builder items/gaps/lengths from image filter/chunks; final stem glyph/cache/SIG/epilog pending.\n--head-chunks also retrieves GRID sections and registers/filters physical chunks automatically; final items/glyph/cache/SIG/epilog pending.\n--head-parts also connects image seeds/head/beam targets to head builder filter inputs; chunks/final glyph/cache/SIG/epilog remain pending.\n--beam-generation also generates beam stumps/BLinkers and head-origin anchors; final stems/cache/SIG/epilog remain pending.\n--beam-targets also derives head-origin sibling beam members; BLinker seed/section stumps and nearest/anchor selection remain pending.\n--head-linkers also selects other head corner targets using image HEADS and current exclusions; beam targets/items/final glyph/cache/SIG/epilog remain pending.\n--sources also collects builder seeds and applies alignment automatically; other-head/beam linkers and final glyphs remain pending.\n--stumps also assembles corner stumps automatically; final stem glyphs/cache/SIG remain pending.\n--seeds also selects existing physical seeds automatically; section fallback remains pending.\nRuns native image HEADS, then derives head corners, beam neighborhoods and targets in explicit XML-restored context. Items/glyph selection, cache/SIG and cleaner/epilog remain unfinished. Ordinary uniform five-line Bravura44/51 and zero skew only. Existing outputs are refused."
        );
        return Ok(());
    }
    if args.len() != 5 {
        return Err("usage: audiveris-stems IMAGE RESIDUAL.pgm HEADS.json STEMS.json".into());
    }
    let outputs: Vec<PathBuf> = args[2..].iter().map(PathBuf::from).collect();
    for (i, p) in outputs.iter().enumerate() {
        if p.exists() || outputs[..i].contains(p) {
            return Err("output paths must be distinct and must not exist".into());
        }
    }
    let mut image_bytes = Vec::new();
    fs::File::open(&args[1])?
        .take(32_000_001)
        .read_to_end(&mut image_bytes)?;
    if image_bytes.len() > 32_000_000 {
        return Err("input exceeds 32 MB".into());
    }
    let image = GrayImage::decode(&image_bytes)?;
    let mut live_owner = None;
    let (mut report, data) =
        if head_items || head_expansion || head_linking || foundation_purge || head_foundations {
            let upstream =
                audiveris_native::page_runtime::analyze(&image, "page-glyphs", None, None)?;
            let mut pixels = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[2])?;
            pixels.write_all(&upstream.output_pixels)?;
            let mut data = serde_json::to_vec(&upstream.report)?;
            data.push(b'\n');
            let mut heads = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[3])?;
            heads.write_all(&data)?;
            let mut owned = audiveris_native::stem_image_owned::continue_items(&image, upstream)?;
            if head_linking || foundation_purge || head_foundations {
                owned = audiveris_native::stem_image_head_linking::continue_linking(owned)?;
                if foundation_purge || head_foundations {
                    owned = audiveris_native::stem_foundation_purge::continue_purge(owned)?;
                    if head_foundations {
                        owned = audiveris_native::stem_head_foundations::continue_reduction(owned)?;
                        if music_measures {
                            let music = audiveris_native::music_measures::continue_measures(owned)?;
                            owned = if music_symbols {
                                audiveris_native::music_symbols::continue_symbols(music)?.owner
                            } else {
                                music.owner
                            };
                        }
                    }
                }
            } else if head_expansion {
                owned = audiveris_native::stem_image_head_expansion::continue_expansion(owned)?;
            }
            let report = std::mem::take(&mut owned.prepared.report);
            live_owner = Some(owned);
            (report, data)
        } else {
            let upstream = std::env::current_exe()?
                .parent()
                .ok_or("missing binary directory")?
                .join("audiveris-native");
            if !upstream.is_file() {
                return Err("audiveris-native must be installed beside audiveris-stems".into());
            }
            let status = Command::new(upstream)
                .arg("page-glyphs")
                .args(&args[1..4])
                .status()?;
            if !status.success() {
                return Err("native image HEADS failed; inspect retained upstream outputs".into());
            }
            let mut data = Vec::new();
            fs::File::open(&args[3])?
                .take(32_000_001)
                .read_to_end(&mut data)?;
            if data.len() > 32_000_000 {
                return Err("native HEADS report exceeds 32 MB".into());
            }
            let page = serde_json::from_slice(&data)?;
            let report = if head_chunks {
                stem_image_head_chunks::analyze(&image, &page)?
            } else if head_parts {
                stem_image_head_parts::analyze(&image, &page)?
            } else if beam_generation {
                stem_image_beam_generation::analyze(&image, &page)?
            } else if beam_targets {
                stem_image_beam_targets::analyze(&image, &page)?
            } else if linkers {
                stem_image_linkers::analyze(&image, &page)?
            } else if sources {
                stem_image_sources::analyze(&image, &page)?
            } else if stumps {
                stem_image_stumps::analyze(&image, &page)?
            } else if seeds {
                stem_image_seeds::analyze(&image, &page)?
            } else {
                stem_image_head_plan::analyze(&image, &page)?
            };
            (report, data)
        };
    report["source_image_sha256"] =
        serde_json::json!(format!("{:x}", Sha256::digest(&image_bytes)));
    report["native_HEADS_report_sha256"] =
        serde_json::json!(format!("{:x}", Sha256::digest(&data)));
    let encoded = serde_json::to_vec(&report)?;
    if encoded.len() > 32_000_000 {
        return Err("native STEMS report exceeds 32 MB".into());
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(&args[4]))?;
    file.write_all(&encoded)?;
    file.write_all(b"\n")?;
    println!(
        "Image head corners/targets: {}; complete STEMS: false",
        report["corner_count"]
    );
    drop(live_owner);
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
