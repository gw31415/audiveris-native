// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, position_runs, scale, sections, vertical};
use std::{
    env, fs,
    io::{BufWriter, Read, Write},
    path::Path,
    process::ExitCode,
};
fn write_runs(out: &mut impl Write, image: &GrayImage, vertical: bool) -> std::io::Result<()> {
    write!(out, "[")?;
    let positions = if vertical { image.width } else { image.height };
    for pos in 0..positions {
        if pos != 0 {
            write!(out, ",")?;
        }
        write!(out, "[")?;
        for (i, r) in position_runs(image, vertical, pos).enumerate() {
            if i != 0 {
                write!(out, ",")?;
            }
            write!(out, "[{},{}]", r.start, r.length)?;
        }
        write!(out, "]")?;
    }
    write!(out, "]")
}
fn classify_features(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if args.len() != 4 {
        return Err("usage: audiveris-native classify-features MODEL.json FEATURES.json".into());
    }
    fn read(path: &str, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take((limit + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            return Err(format!("input exceeds {limit} byte limit").into());
        }
        Ok(bytes)
    }
    let model = audiveris_classifier::Model::from_json(&read(
        &args[2],
        audiveris_classifier::MAX_MODEL_BYTES,
    )?)?;
    let features: Vec<f64> = serde_json::from_slice(&read(&args[3], 65_536)?)?;
    let evaluation = model.evaluate(&features)?;
    let result =
        serde_json::json!({"class_labels":model.output_labels(),"scores":evaluation.scores});
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, &result)?;
    writeln!(out)?;
    Ok(true)
}
fn classify_glyph(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if ![5, 6, 8].contains(&args.len()) {
        return Err("usage: audiveris-native classify-glyph MODEL.json BINARY_IMAGE INTERLINE [horizontal|vertical [LEFT TOP]]".into());
    }
    let interline: u32 = args[4].parse()?;
    let orientation = match args.get(5).map(String::as_str).unwrap_or("vertical") {
        "horizontal" => audiveris_native::glyph::Orientation::Horizontal,
        "vertical" => audiveris_native::glyph::Orientation::Vertical,
        _ => return Err("glyph orientation must be horizontal or vertical".into()),
    };
    let (left, top) = if args.len() == 8 {
        (args[6].parse::<i32>()?, args[7].parse::<i32>()?)
    } else {
        (0, 0)
    };
    fn read(path: &str, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut b = Vec::new();
        fs::File::open(path)?
            .take((limit + 1) as u64)
            .read_to_end(&mut b)?;
        if b.len() > limit {
            return Err(format!("input exceeds {limit} byte limit").into());
        }
        Ok(b)
    }
    let model = audiveris_classifier::Model::from_json(&read(
        &args[2],
        audiveris_classifier::MAX_MODEL_BYTES,
    )?)?;
    let image = GrayImage::decode(&read(&args[3], 32_000_000)?)?;
    let raw = audiveris_native::glyph::from_image(&image, interline, orientation, left, top)?;
    let evaluation = model.evaluate(&raw)?;
    let output = serde_json::json!({"schema":"audiveris-native.isolated-glyph.v1","interline":interline,"origin":[left,top],"orientation":args.get(5).map(String::as_str).unwrap_or("vertical"),"input_labels":model.input_labels(),"raw_features":raw.as_slice(),"normalized":evaluation.normalized,"hidden":evaluation.hidden,"class_labels":model.output_labels(),"scores":evaluation.scores});
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, &output)?;
    writeln!(out)?;
    Ok(true)
}
fn page_glyphs(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    page_glyphs_with_catalog(args, None)
}
/// Development control: configure HEADS after the ordinary native upstream
/// phases, matching the original oracle's controlled catalog experiment.
fn head_catalog_control(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if ![7, 8].contains(&args.len()) || args.get(7).is_some_and(|s| s != "cue") {
        return Err("usage: audiveris-native heads-catalog-control INPUT OUTPUT.pgm REPORT.json FAMILY POINT_SIZE [cue]".into());
    }
    let catalog = audiveris_native::head_staff_catalog::TemplateCatalog {
        family: args[5].clone(),
        stored_point_size: args[6].parse()?,
    };
    catalog.validate()?;
    let mut standard = args[..5].to_vec();
    standard[1] = "page-glyphs".into();
    page_glyphs_with_catalog(&standard, Some((catalog, args.len() == 8)))
}
fn page_glyphs_with_catalog(
    args: &[String],
    declared: Option<(audiveris_native::head_staff_catalog::TemplateCatalog, bool)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if args.len() != 5 && args.len() != 6 {
        return Err(
            "usage: audiveris-native <page-glyphs|staff-mask|beam-spots> INPUT OUTPUT.pgm REPORT.json [MODEL.json]"
                .into(),
        );
    }
    if args[3] == args[4] || Path::new(&args[3]).exists() || Path::new(&args[4]).exists() {
        return Err("output paths must be distinct and must not exist".into());
    }
    let mut bytes = vec![];
    fs::File::open(&args[2])?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("input exceeds 32 MB".into());
    }
    let image = GrayImage::decode(&bytes)?;
    let model = if let Some(path) = args.get(5) {
        let mut b = vec![];
        fs::File::open(path)?
            .take((audiveris_classifier::MAX_MODEL_BYTES + 1) as u64)
            .read_to_end(&mut b)?;
        Some(audiveris_classifier::Model::from_json(&b)?)
    } else {
        None
    };
    let owned =
        audiveris_native::page_runtime::analyze(&image, &args[1], model.as_ref(), declared)?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    out.write_all(&owned.output_pixels)?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[4])?;
    serde_json::to_writer(&mut out, &owned.report)?;
    writeln!(out)?;
    if args[1] == "staff-mask" {
        println!(
            "Automatic GRID staff mask complete; HEADS mask and note recognition remain incomplete"
        );
    } else if args[1] == "beam-spots" {
        println!(
            "Native beam spots: {}; prolog complete: {}; initial candidates complete: {}; beam interpretation and MusicXML remain incomplete",
            owned.report["beam_preparation"]["spot_count"],
            owned.report["beam_prolog"]["beam_prolog_complete"],
            owned.report["beam_candidates"]["initial_beam_candidates_complete"]
        );
    } else {
        println!(
            "Experimental native page glyphs: {}; full GRID and musical recognition remain incomplete",
            owned.report["page_glyphs"]["component_count"]
        );
    }
    Ok(owned.success)
}

fn template_probes(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if args.len() != 5 && args.len() != 6 {
        return Err(
            "usage: audiveris-native template-probes INPUT PROBES.json REPORT.json [IGNORE_MASK]"
                .into(),
        );
    }
    if Path::new(&args[4]).exists() {
        return Err("output must not exist".into());
    }
    fn read(path: &str, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take((limit + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            return Err("input byte budget exceeded".into());
        }
        Ok(bytes)
    }
    let request = read(
        &args[3],
        audiveris_native::template_probes::MAX_REQUEST_BYTES,
    )?;
    let image = GrayImage::decode(&read(&args[2], 32_000_000)?)?;
    let binary = vertical::binarize(&image, |_, _, _| {});
    drop(image);
    let mask = args
        .get(5)
        .map(|path| -> Result<GrayImage, Box<dyn std::error::Error>> {
            Ok(GrayImage::decode(&read(path, 32_000_000)?)?)
        })
        .transpose()?;
    let result = audiveris_native::template_probes::evaluate(&binary, mask.as_ref(), &request)?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[4])?;
    serde_json::to_writer(&mut out, &result)?;
    writeln!(out)?;
    println!("Explicit template probes complete; automatic note recognition remains incomplete");
    Ok(true)
}

fn stem_parts_lookup(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    use audiveris_native::stem_parts_cli;
    if args.len() != 4 {
        return Err("usage: audiveris-native stem-parts-lookup REQUEST.json REPORT.json".into());
    }
    if Path::new(&args[3]).exists() {
        return Err("stem-parts-lookup report already exists".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&args[2])?
        .take((stem_parts_cli::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > stem_parts_cli::MAX_REQUEST_BYTES {
        return Err("stem_lookup_request_byte_budget".into());
    }
    let request: stem_parts_cli::Request = serde_json::from_slice(&bytes)?;
    let report = stem_parts_cli::evaluate(&request)?;
    let encoded = stem_parts_cli::encode_report(&report)?;
    let mut destination = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    destination.write_all(&encoded)?;
    println!(
        "Stem parts collected from supplied lookup geometry; complete page STEMS remains unfinished"
    );
    Ok(true)
}

fn stem_chunks_lookup(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    use audiveris_native::stem_chunks_cli;
    if args.len() != 4 {
        return Err("usage: audiveris-native stem-chunks-lookup REQUEST.json REPORT.json".into());
    }
    if Path::new(&args[3]).exists() {
        return Err("stem-chunks-lookup report already exists".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&args[2])?
        .take((stem_chunks_cli::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > stem_chunks_cli::MAX_REQUEST_BYTES {
        return Err("stem_chunks_request_byte_budget".into());
    }
    let request: stem_chunks_cli::Request = serde_json::from_slice(&bytes)?;
    let report = stem_chunks_cli::evaluate(&request)?;
    let encoded = stem_chunks_cli::encode_report(&report)?;
    let mut destination = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    destination.write_all(&encoded)?;
    println!(
        "Registered chunks selected from supplied lookup geometry; complete page STEMS remains unfinished"
    );
    Ok(true)
}

fn stem_items_plan(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    use audiveris_native::stem_items_cli;
    if args.len() != 4 {
        return Err("usage: audiveris-native stem-items-plan REQUEST.json REPORT.json".into());
    }
    if Path::new(&args[3]).exists() {
        return Err("stem-items-plan report already exists".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&args[2])?
        .take((stem_items_cli::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > stem_items_cli::MAX_REQUEST_BYTES {
        return Err("stem_items_request_byte_budget".into());
    }
    let request: stem_items_cli::Request = serde_json::from_slice(&bytes)?;
    let report = stem_items_cli::evaluate(&request)?;
    let encoded = stem_items_cli::encode_report(&report)?;
    let mut destination = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    destination.write_all(&encoded)?;
    println!(
        "Items and gaps selected from supplied glyph/linker sources; complete page STEMS remains unfinished"
    );
    Ok(true)
}

fn stem_head_plan(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    use audiveris_native::stem_head_neighborhood;
    if args.len() != 4 {
        return Err("usage: audiveris-native stem-head-plan REQUEST.json REPORT.json".into());
    }
    if Path::new(&args[3]).exists() {
        return Err("stem-head-plan report already exists".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&args[2])?
        .take((stem_head_neighborhood::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > stem_head_neighborhood::MAX_REQUEST_BYTES {
        return Err("stem_items_request_byte_budget".into());
    }
    let request: stem_head_neighborhood::Request = serde_json::from_slice(&bytes)?;
    let report = stem_head_neighborhood::evaluate(&request)?;
    let encoded = stem_head_neighborhood::encode_report(&report)?;
    let mut destination = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    destination.write_all(&encoded)?;
    println!(
        "Head candidates and targets selected from supplied page state; raster STEMS and cache/SIG remain unfinished"
    );
    Ok(true)
}

fn execute() -> Result<bool, Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.get(1).is_some_and(|s| s == "stem-head-plan") {
        return stem_head_plan(&args);
    }
    if args.get(1).is_some_and(|s| s == "stem-items-plan") {
        return stem_items_plan(&args);
    }
    if args.get(1).is_some_and(|s| s == "stem-chunks-lookup") {
        return stem_chunks_lookup(&args);
    }
    if args.get(1).is_some_and(|s| s == "stem-parts-lookup") {
        return stem_parts_lookup(&args);
    }
    if args.get(1).is_some_and(|s| s == "template-probes") {
        return template_probes(&args);
    }
    if args.get(1).is_some_and(|s| s == "heads-catalog-control") {
        return head_catalog_control(&args);
    }
    if args
        .get(1)
        .is_some_and(|s| s == "page-glyphs" || s == "staff-mask" || s == "beam-spots")
    {
        return page_glyphs(&args);
    }
    if args.get(1).is_some_and(|s| s == "classify-glyph") {
        return classify_glyph(&args);
    }
    if args.get(1).is_some_and(|s| s == "classify-features") {
        return classify_features(&args);
    }
    if args.len() == 2 && args[1] == "--license" {
        print!(
            "{}\n{}\n{}\n{}",
            include_str!("../NOTICE"),
            include_str!("../crates/audiveris-classifier/NOTICE"),
            concat!(
                include_str!("../crates/audiveris-glyph-features/NOTICE"),
                "\n",
                include_str!("../crates/audiveris-staff-removal/NOTICE"),
                "\n",
                include_str!("../crates/audiveris-template-bank/NOTICE"),
                "\n",
                include_str!("../crates/audiveris-template-match/NOTICE")
            ),
            include_str!("../LICENSE")
        );
        return Ok(true);
    }
    if args.len() == 2 && args[1] == "--version" {
        println!(
            "audiveris-native {} (Audiveris 5.11.0 9e1e55cd)",
            env!("CARGO_PKG_VERSION")
        );
        println!(
            "{}\n{}",
            include_str!("../crates/audiveris-area/NOTICE"),
            include_str!("../crates/audiveris-area/LICENSE")
        );
        return Ok(true);
    }
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        println!(
            "audiveris-native <binarize|scale|grid-prep|filament-seeds|filaments|staff-candidates|grid-candidates> INPUT BINARY.pgm REPORT.json\nEarly Audiveris 5.11.0 native preprocessing port. Grayscale PNG (8-bit) or PGM P2/P5 (maxval 255), <=4M pixels.\nOutputs binary PGM and horizontal/vertical [start,length] runs with threshold probes.\nscale adds staff line/interline/beam estimates and histogram diagnostics (exit 2 on failure).\ngrid-prep outputs LAG sections; staff detection is not implemented.\nfilament-seeds preserves the initial rectangular-only checkpoint.\nfilaments adds official curve interpolation, merge/expand, curvature/slope filtering and unrestricted curved combs; Staff clustering is not implemented.\nstaff-candidates adds default five-line, single-interline clusters and initial Staff geometry; later GRID stages remain incomplete.\ngrid-candidates adds bar projection/sticks and initial systems; unsupported layouts or no systems exit 2. Full GRID completion remains pending.\naudiveris-native classify-features MODEL.json FEATURES.json -> 149 labeled sigmoid scores on stdout; requires 110 pre-extracted features, NOT an image.\naudiveris-native classify-glyph MODEL.json BINARY_IMAGE INTERLINE [horizontal|vertical [LEFT TOP]] -> isolated binary glyph features and scores (no page segmentation).\npage-glyphs INPUT RESIDUAL.pgm REPORT.json [MODEL.json] -> experimental raw page components, optional isolated scores; GRID remains incomplete.\ntemplate-probes INPUT PROBES.json REPORT.json [IGNORE_MASK] -> fixed 51pt explicit-position scores; optional binary mask uses 0=ignored. No automatic notehead detection.\nstaff-mask INPUT MASK.pgm REPORT.json -> automatic GRID staff glyph/curve mask, before ledgers and stem seeds.\nbeam-spots INPUT SPOTS.pgm REPORT.json -> native beam spots, head sizing and system dispatch; BeamInter, rhythm and MusicXML remain incomplete.\nstem-parts-lookup REQUEST.json REPORT.json -> section selection and raw glyphs from supplied lookup polygons; automatic lookup geometry and complete STEMS/SIG remain unfinished.\nstem-chunks-lookup REQUEST.json REPORT.json -> section lookup, raw assembly, canonical registration and filtered/sorted chunks from supplied geometry and context; cache/head/SIG and epilog remain unfinished.\nstem-items-plan REQUEST.json REPORT.json -> construct items, suffix-sort, insert/truncate gaps and select profile lengths from supplied physical glyph/linker sources; automatic image target selection, cache/SIG and epilog remain unfinished.\nstem-head-plan REQUEST.json REPORT.json -> automatic neighborhood, initial-area candidate and beam target selection from supplied page state; raster discovery, cache/SIG and epilog remain unfinished.\nPDF, page-level note recognition, and MusicXML export are not implemented. Existing outputs are refused.\nOptions: --help, --version, --license"
        );
        return Ok(true);
    }
    if args.len() != 5
        || !matches!(
            args[1].as_str(),
            "binarize"
                | "scale"
                | "grid-prep"
                | "filament-seeds"
                | "filaments"
                | "staff-candidates"
                | "grid-candidates"
        )
    {
        return Err(
            "usage: audiveris-native <binarize|scale|grid-prep|filament-seeds|filaments|staff-candidates|grid-candidates> INPUT BINARY.pgm REPORT.json"
                .into(),
        );
    }
    if args[3] == args[4] || Path::new(&args[3]).exists() || Path::new(&args[4]).exists() {
        return Err("output paths must be distinct and must not exist".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&args[2])?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("input exceeds 32 MB".into());
    }
    let image = GrayImage::decode(&bytes)?;
    drop(bytes);
    let points = [
        (0, 0),
        (image.width / 2, image.height / 2),
        (image.width - 1, image.height - 1),
    ];
    let mut probes = [String::new(), String::new(), String::new()];
    let binary = vertical::binarize(&image, |x, y, c| {
        for (i, p) in points.iter().enumerate() {
            if *p == (x, y) {
                probes[i] = format!(
                    "{{\"x\":{x},\"y\":{y},\"mean\":{},\"std_dev\":{},\"threshold\":{}}}",
                    c.mean, c.std_dev, c.threshold
                );
            }
        }
    });
    let mut binary_out = BufWriter::new(
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[3])?,
    );
    write!(binary_out, "P5\n{} {}\n255\n", binary.width, binary.height)?;
    binary_out.write_all(&binary.pixels)?;
    binary_out.flush()?;
    let mut out = BufWriter::new(
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[4])?,
    );
    write!(
        out,
        "{{\"schema\":\"audiveris-native.preprocessing.v1\",\"upstream\":\"9e1e55cd2746037d059345881c53e6a6754bffbd\",\"width\":{},\"height\":{},\"foreground_pixels\":{},\"probes\":[{}],\"horizontal_runs\":",
        image.width,
        image.height,
        binary.pixels.iter().filter(|p| **p == 0).count(),
        probes.join(",")
    )?;
    write_runs(&mut out, &binary, false)?;
    write!(out, ",\"vertical_runs\":")?;
    write_runs(&mut out, &binary, true)?;
    let mut scale_ok = true;
    if args[1] != "binarize" {
        let analysis = scale::analyze(&binary);
        write!(out, ",\"scale\":")?;
        analysis.write_json(&mut out)?;
        if args[1] == "grid-prep"
            || args[1] == "filament-seeds"
            || args[1] == "filaments"
            || matches!(args[1].as_str(), "staff-candidates" | "grid-candidates")
        {
            write!(out, ",\"grid_preparation\":")?;
            if let Ok(s) = &analysis.result {
                match sections::prepare(&binary, s) {
                    Ok(prepared) => {
                        prepared.write_json(&mut out)?;
                        if args[1] == "filaments"
                            || matches!(args[1].as_str(), "staff-candidates" | "grid-candidates")
                        {
                            write!(out, ",\"filaments\":")?;
                            let result = audiveris_native::filament_merge::analyze(
                                image.width,
                                s,
                                &prepared.horizontal,
                            );
                            if result["status"] == "failed"
                                || result["line_filtering"]["too_few_after_curvature"] == true
                            {
                                scale_ok = false;
                                eprintln!(
                                    "Filament stage limited or too few lines; diagnostics retained"
                                );
                            }
                            serde_json::to_writer(&mut out, &result)?;
                            if matches!(args[1].as_str(), "staff-candidates" | "grid-candidates") {
                                let clusters = audiveris_native::clusters::analyze_two_pass(
                                    image.width,
                                    s,
                                    &prepared.horizontal,
                                    &result,
                                );
                                write!(out, ",\"staff_candidates\":")?;
                                serde_json::to_writer(&mut out, &clusters)?;
                                let projection = audiveris_native::staff_projection::analyze(
                                    &binary,
                                    s,
                                    &prepared.horizontal,
                                    &clusters,
                                );
                                write!(out, ",\"bar_candidates\":")?;
                                serde_json::to_writer(&mut out, &projection)?;
                                let sticks = audiveris_native::bar_sticks::analyze(
                                    &binary,
                                    s,
                                    &prepared,
                                    &projection,
                                );
                                write!(out, ",\"bar_sticks\":")?;
                                serde_json::to_writer(&mut out, &sticks)?;
                                let systems = audiveris_native::system_graph::analyze(
                                    &binary,
                                    s,
                                    &clusters,
                                    &sticks,
                                    result["line_filtering"]["global_slope"]
                                        .as_f64()
                                        .unwrap_or(0.),
                                );
                                write!(out, ",\"system_candidates\":")?;
                                serde_json::to_writer(&mut out, &systems)?;
                                let completion = audiveris_native::line_completion::analyze(
                                    &binary,
                                    s,
                                    &prepared,
                                    &clusters,
                                    &projection,
                                    &sticks,
                                    &systems,
                                    result["line_filtering"]["global_slope"]
                                        .as_f64()
                                        .unwrap_or(0.),
                                );
                                if completion["status"] != "partial_line_completion" {
                                    scale_ok = false;
                                    eprintln!("Line completion failed; diagnostics retained");
                                }
                                write!(out, ",\"line_completion\":")?;
                                serde_json::to_writer(&mut out, &completion)?;
                                if args[1] == "grid-candidates"
                                    && systems["status"] != "initial_systems"
                                {
                                    scale_ok = false;
                                    eprintln!(
                                        "Initial system detection incomplete or failed; diagnostics retained"
                                    );
                                }
                                if projection["status"] != "bar_candidates"
                                    || sticks["status"] != "bar_sticks"
                                {
                                    scale_ok = false;
                                }
                                if clusters["status"] != "initial_staffs" {
                                    scale_ok = false;
                                }
                            }
                        }
                        if args[1] == "filament-seeds" {
                            write!(out, ",\"filament_seeds\":")?;
                            serde_json::to_writer(
                                &mut out,
                                &audiveris_native::filaments::analyze(
                                    image.width,
                                    s,
                                    &prepared.horizontal,
                                ),
                            )?;
                        }
                    }
                    Err(code) => {
                        write!(out, "{{\"status\":\"failed\",\"error\":\"{code}\"}}")?;
                        scale_ok = false;
                        eprintln!("GRID preparation failed: {code}");
                    }
                }
            } else {
                write!(out, "{{\"status\":\"blocked\",\"error\":\"scale_failed\"}}")?;
            }
        }
        if let Err(e) = analysis.result {
            eprintln!("SCALE failed: {} (diagnostics retained)", e.code());
            scale_ok = false;
        }
    }
    writeln!(out, "}}")?;
    out.flush()?;
    let completed = if args[1] == "grid-candidates" && scale_ok {
        "INITIAL SYSTEMS (full GRID incomplete)"
    } else if matches!(args[1].as_str(), "staff-candidates" | "grid-candidates") && scale_ok {
        "INITIAL STAFF descriptors (five-line; bounded main/small interlines)"
    } else if args[1] == "filaments" && scale_ok {
        "FILAMENTS (Staff clustering incomplete)"
    } else if args[1] == "filament-seeds" && scale_ok {
        "FILAMENT SEEDS (restricted combs; staff detection incomplete)"
    } else if args[1] == "grid-prep" && scale_ok {
        "GRID preparation"
    } else if args[1] == "scale" && scale_ok {
        "SCALE"
    } else {
        "BINARY"
    };
    println!(
        "{completed} complete: {}x{}; {} + {}",
        image.width, image.height, args[3], args[4]
    );
    Ok(scale_ok)
}
fn main() -> ExitCode {
    match execute() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
