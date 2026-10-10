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
    pub residual: Bitmap,
    pub scale: Scale,
    pub staves: Vec<Staff>,
    pub systems: Vec<System>,
    pub headers: Vec<Header>,
    pub beams: Vec<Beam>,
    pub ledgers: Vec<Ledger>,
    pub heads: Vec<Head>,
    /// Small black heads with a stem (up?), for grace notes.
    pub grace_heads: Vec<(Head, bool)>,
    pub stems: Vec<Stem>,
    pub symbols: Vec<Symbol>,
    pub wedges: Vec<crate::wedges::Wedge>,
    /// Key signature changes: (staff, barline right abscissa, fifths).
    pub key_changes: Vec<(usize, i32, i32)>,
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
    let mut smalls = Vec::new();
    for sys in &systems {
        for &si in &sys.staves {
            let (h, sm) = heads::detect(&cells, &catalog, &staves, si, sys, &ledgers, &beams, &scale);
            candidates.extend(h);
            smalls.extend(sm);
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
    let grace_heads = heads::grace_heads(&bin, smalls, &heads, &scale);
    log(
        verbose,
        &t,
        format!("STEMS {} heads, {} stems", heads.len(), stems.len()),
    );
    let mut items: Vec<Rect> = headers
        .iter()
        .flat_map(|h| h.items.iter().map(|i| i.0))
        .collect();
    // Key signature changes after double barlines.
    let mut key_changes = Vec::new();
    for sys in &systems {
        for &si in &sys.staves {
            let clef = headers[si].clef.unwrap_or(crate::header::ClefKind::G);
            let mut prev = headers[si].fifths;
            let n = sys.boundaries.len();
            for b in sys.boundaries.iter().take(n.saturating_sub(1)).skip(1) {
                if b.style == crate::grid::BarStyle::Regular {
                    continue;
                }
                if let Some((f, its)) = crate::header::key_change(&nostaff, &staves[si], clef, prev, b.x1 + 1, &scale) {
                    key_changes.push((si, b.x1, f));
                    items.extend(its);
                    prev = f;
                }
            }
        }
    }
    let residual = symbols::residual(
        &nostaff, &heads, &catalog, &stems, &beams, &ledgers, &systems, &staves, &items, &scale,
    );
    let symbols = symbols::recognize(&residual, &staves, &scale);
    stems::purge_by_symbols(&mut heads, &mut stems, &symbols);
    log(verbose, &t, format!("SYMBOLS {}", symbols.len()));
    let wedges = crate::wedges::detect(&residual, &staves, &scale);
    log(verbose, &t, format!("WEDGES {}", wedges.len()));
    Ok(Sheet {
        binary: bin,
        residual,
        scale,
        staves,
        systems,
        headers,
        beams,
        ledgers,
        heads,
        grace_heads,
        stems,
        symbols,
        wedges,
        key_changes,
    })
}
