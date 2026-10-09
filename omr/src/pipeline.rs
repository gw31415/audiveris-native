//! The recognition pipeline of one sheet image, step by step as in Audiveris:
//! BINARY, SCALE, GRID, HEADERS, BEAMS, LEDGERS, HEADS, STEMS (+REDUCTION),
//! SYMBOLS. Musical assembly (MEASURES, CHORDS, RHYTHMS) is in `score`.

use std::path::Path;
use std::time::Instant;

use crate::beams::{self, Beam, Ledger};
use crate::binary::{self, Bitmap, Rect};
use crate::grid::{self, System};
use crate::header::{self, Header};
use crate::heads::{self, Catalog, Head};
use crate::scale::{self, Scale};
use crate::staff::{self, Staff};
use crate::stems::{self, Stem};
use crate::symbols::{self, Symbol};

pub struct Sheet {
    pub binary: Bitmap,
    pub nostaff: Bitmap,
    pub residual: Bitmap,
    pub scale: Scale,
    pub staves: Vec<Staff>,
    pub systems: Vec<System>,
    pub headers: Vec<Header>,
    pub beams: Vec<Beam>,
    pub ledgers: Vec<Ledger>,
    pub catalog: Catalog,
    pub heads: Vec<Head>,
    pub stems: Vec<Stem>,
    pub symbols: Vec<Symbol>,
}

fn log(verbose: bool, t: &Instant, msg: String) {
    if verbose {
        eprintln!("[{:>6.0} ms] {msg}", t.elapsed().as_secs_f64() * 1000.0);
    }
}

pub fn process(path: &Path, verbose: bool) -> Result<Sheet, String> {
    let t = Instant::now();
    let (w, h, gray) = binary::load_gray(path)?;
    let bin = binary::binarize(w, h, &gray);
    log(verbose, &t, format!("BINARY {w}x{h}"));
    let scale = scale::compute(&bin)?;
    log(verbose, &t, format!("SCALE {scale:?}"));
    let mut staves = staff::retrieve(&bin, &scale);
    if staves.is_empty() {
        return Err("no staff found".into());
    }
    let systems = grid::build_systems(&bin, &mut staves, &scale);
    let (nostaff, removed) = staff::remove_lines(&bin, &staves, &scale);
    log(
        verbose,
        &t,
        format!("GRID {} staves, {} systems", staves.len(), systems.len()),
    );
    let mut headers = Vec::new();
    for s in staves.iter_mut() {
        let hd = header::recognize(&nostaff, s, &scale);
        s.header_stop = hd.stop;
        headers.push(hd);
    }
    // Bars found inside the headers (e.g. time signature digits) are not measure boundaries.
    let mut systems = systems;
    for sys in systems.iter_mut() {
        let stop = sys
            .staves
            .iter()
            .map(|&s| staves[s].header_stop)
            .max()
            .unwrap();
        let first = sys.boundaries[0].clone();
        sys.boundaries.retain(|b| b.x0 >= stop || b.x0 == first.x0);
    }
    log(verbose, &t, "HEADERS".into());
    let beams = beams::detect(&nostaff, &scale);
    let mut ledgers = Vec::new();
    for si in 0..staves.len() {
        ledgers.extend(beams::ledgers(&nostaff, &staves, si, &scale));
    }
    log(
        verbose,
        &t,
        format!(
            "BEAMS {} candidates, LEDGERS {}",
            beams.len(),
            ledgers.len()
        ),
    );
    let catalog = Catalog::new(heads::head_point_size(&nostaff, &scale));
    let cells = heads::build_cells(&bin, &removed, &nostaff, &ledgers, &scale);
    let mut candidates = Vec::new();
    for sys in &systems {
        for &si in &sys.staves {
            candidates.extend(heads::detect(
                &cells, &catalog, &staves, si, sys, &ledgers, &beams, &scale,
            ));
        }
    }
    log(
        verbose,
        &t,
        format!(
            "HEADS {} candidates (point size {})",
            candidates.len(),
            catalog.point_size
        ),
    );
    if let Ok(v) = std::env::var("OMR_PROBE") {
        let v: Vec<i32> = v.split(',').map(|x| x.parse().unwrap()).collect();
        let r = Rect::new(v[0] - 15, v[1] - 15, 30, 30);
        for h in candidates.iter().filter(|h| h.bounds.intersects(&r)) {
            eprintln!("probe {h:?}");
        }
    }
    let (mut heads, mut stems) = stems::link(&nostaff, candidates, &scale);
    stems::attach_beams(&mut stems, &heads, &beams, &scale);
    log(
        verbose,
        &t,
        format!("STEMS {} heads, {} stems", heads.len(), stems.len()),
    );
    let items: Vec<Rect> = headers
        .iter()
        .flat_map(|h| h.items.iter().map(|i| i.0))
        .collect();
    let residual = symbols::residual(
        &nostaff, &heads, &catalog, &stems, &beams, &ledgers, &systems, &staves, &items, &scale,
    );
    let symbols = symbols::recognize(&residual, &staves, &scale);
    stems::purge_by_symbols(&mut heads, &mut stems, &symbols);
    log(verbose, &t, format!("SYMBOLS {}", symbols.len()));
    Ok(Sheet {
        binary: bin,
        nostaff,
        residual,
        scale,
        staves,
        systems,
        headers,
        beams,
        ledgers,
        catalog,
        heads,
        stems,
        symbols,
    })
}
