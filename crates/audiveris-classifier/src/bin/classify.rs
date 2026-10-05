// SPDX-License-Identifier: AGPL-3.0-or-later
// Audiveris native classifier adapter, 2026-10-02. See NOTICE and LICENSE.
use audiveris_classifier::{MAX_MODEL_BYTES, Model, SOURCE_COMMIT};
use serde::Serialize;
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process::ExitCode,
};

fn read_bounded(path: &str, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(format!("{path}: exceeds {limit} byte limit").into());
    }
    Ok(bytes)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 {
        match args[0].as_str() {
            "--help" | "-h" => {
                println!(
                    "Usage: audiveris-classify MODEL.json FEATURES.json\nFEATURES.json must be an array of 110 finite raw f64 features.\nOutputs 149 sigmoid scores in the original class order. No glyph extraction.\nOptions: --help, --version, --license"
                );
                return Ok(());
            }
            "--version" => {
                println!(
                    "audiveris-classifier {} (Audiveris 5.11.0 {SOURCE_COMMIT})",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(());
            }
            "--license" => {
                print!(
                    "{}\n{}",
                    include_str!("../../NOTICE"),
                    include_str!("../../LICENSE")
                );
                return Ok(());
            }
            _ => {}
        }
    }
    if args.len() != 2 {
        return Err("expected MODEL.json FEATURES.json; use --help".into());
    }
    let model = Model::from_json(&read_bounded(&args[0], MAX_MODEL_BYTES)?)?;
    let features: Vec<f64> = serde_json::from_slice(&read_bounded(&args[1], 65_536)?)?;
    let evaluation = model.evaluate(&features)?;
    #[derive(Serialize)]
    struct Output<'a> {
        class_labels: &'a [String],
        scores: &'a [f64],
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    serde_json::to_writer(
        &mut out,
        &Output {
            class_labels: model.output_labels(),
            scores: &evaluation.scores,
        },
    )?;
    writeln!(out)?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("audiveris-classify: {error}");
            ExitCode::from(2)
        }
    }
}
