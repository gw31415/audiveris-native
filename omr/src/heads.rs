//! Note heads (Audiveris NoteHeadsBuilder, range scanning).
//!
//! For every staff, each line / space / ledger position is scanned from the
//! end of the header to the end of the staff. At each abscissa, every head
//! template is evaluated (anchored MIDDLE_LEFT on the theoretical ordinate,
//! with a small vertical tolerance). Matches below the "low" distance become
//! head candidates; neighboring matches of the same scan are aggregated, and
//! overlapping candidates of the staff are resolved by grade.

use crate::beams::{Beam, Ledger};
use crate::binary::{Bitmap, Rect, chamfer_distance};
use crate::grid::System;
use crate::scale::Scale;
use crate::staff::Staff;
use crate::template::{self, BLACK, HeadShape, Template, UNKNOWN, WHITE};

const MAX_DISTANCE_HIGH: f64 = 0.5;
const MAX_DISTANCE_LOW: f64 = 0.40;
const INTRINSIC_RATIO: f64 = 0.8;
const MIN_HOLE_WHITE_RATIO: f64 = 0.2;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Head {
    pub staff: usize,
    pub shape: HeadShape,
    pub pitch: i32,
    pub bounds: Rect,
    pub grade: f64,
    /// Matching distance.
    pub dist: f64,
    /// Contextual grade (after stem support).
    pub ctx: f64,
}

pub fn grade_of(dist: f64) -> f64 {
    INTRINSIC_RATIO * (1.0 - dist / MAX_DISTANCE_HIGH)
}

pub struct Catalog {
    pub point_size: i32,
    pub templates: Vec<Template>,
    /// Small (cue) black and void templates, at 0.67 of the point size.
    pub small: Vec<Template>,
}

impl Catalog {
    pub fn new(point_size: i32) -> Self {
        let templates = HeadShape::ALL
            .iter()
            .filter_map(|&s| template::build(s, point_size))
            .collect();
        let sps = (point_size as f64 * 0.67).round() as i32;
        let small = [HeadShape::Black, HeadShape::Void]
            .iter()
            .filter_map(|&s| template::build(s, sps))
            .collect();
        Self {
            point_size,
            templates,
            small,
        }
    }
    pub fn get(&self, shape: HeadShape) -> &Template {
        self.templates.iter().find(|t| t.shape == shape).unwrap()
    }
    pub fn get_small(&self, shape: HeadShape) -> &Template {
        self.small.iter().find(|t| t.shape == shape).unwrap()
    }
    pub fn template(&self, h: &Head) -> &Template {
        self.get(h.shape)
    }
}

/// Measure the typical black head width and derive the head point size
/// (Audiveris BlackHeadSizer / HeadSeedScale).
pub fn head_point_size(nostaff: &Bitmap, scale: &Scale) -> i32 {
    let il = scale.interline as f64;
    let hr = nostaff.horizontal_run_lengths();
    let vr = nostaff.vertical_run_lengths();
    let mut hist = vec![0u64; (3.0 * il) as usize + 2];
    for i in 0..hr.len() {
        let (h, v) = (hr[i] as f64, vr[i] as f64);
        if h >= 1.0 * il && h <= 2.0 * il && v >= 0.75 * il && v <= 1.35 * il {
            hist[hr[i] as usize] += 1;
        }
    }
    let nominal = 4 * scale.interline;
    let Some(width) = (0..hist.len())
        .max_by_key(|&i| hist[i])
        .filter(|&i| hist[i] > 0)
    else {
        return nominal;
    };
    // Choose the point size whose black template slim width matches best.
    let mut best = (nominal, i32::MAX);
    for ps in (3 * scale.interline)..=(6 * scale.interline) {
        if let Some(t) = template::build(HeadShape::Black, ps) {
            let d = (t.slim.w - width as i32).abs();
            if d < best.1 {
                best = (ps, d);
            }
        }
    }
    best.0
}

pub struct Cells {
    pub data: Vec<u8>,
    pub w: i32,
    pub h: i32,
    pub dist: Vec<i32>,
}

/// Image used for template matching: staff lines, ledgers and stem seeds are UNKNOWN.
pub fn build_cells(
    bin: &Bitmap,
    removed: &Bitmap,
    nostaff: &Bitmap,
    ledgers: &[Ledger],
    scale: &Scale,
) -> Cells {
    let (w, h) = (bin.width, bin.height);
    let mut data: Vec<u8> = bin
        .data
        .iter()
        .map(|&b| if b { BLACK } else { WHITE })
        .collect();
    for i in 0..w * h {
        if removed.data[i] {
            data[i] = UNKNOWN;
        }
    }
    let thin = scale.max_line + 1;
    for l in ledgers {
        for x in l.x0..=l.x1 {
            let y = l.y.round() as i32;
            let (a, b) = nostaff.vrun_at(x, y);
            if b - a + 1 <= thin {
                for yy in a..=b {
                    data[yy as usize * w + x as usize] = UNKNOWN;
                }
            }
        }
    }
    // Stem seeds: thin vertical sticks long enough.
    let vr = nostaff.vertical_run_lengths();
    let hr = nostaff.horizontal_run_lengths();
    let min_len = (1.5 * scale.interline as f64) as u16;
    let max_w = (scale.max_stem + 1) as u16;
    for i in 0..w * h {
        if vr[i] >= min_len && hr[i] <= max_w && hr[i] > 0 {
            data[i] = UNKNOWN;
        }
    }
    let mut known_black = Bitmap::new(w, h);
    for i in 0..w * h {
        known_black.data[i] = data[i] == BLACK;
    }
    Cells {
        data,
        w: w as i32,
        h: h as i32,
        dist: chamfer_distance(&known_black),
    }
}

struct Scan {
    pitch: i32,
    x_from: i32,
    x_to: i32,
    open_dir: Option<i32>,
    y: Box<dyn Fn(i32) -> i32>,
}

pub fn detect(
    cells: &Cells,
    catalog: &Catalog,
    staves: &[Staff],
    si: usize,
    system: &System,
    ledgers: &[Ledger],
    beams: &[Beam],
    scale: &Scale,
) -> Vec<Head> {
    let s = &staves[si];
    let il = scale.interline;
    let min_template_width = il;
    let template_half = 3 * il / 2;
    let max_dx = (0.375 * il as f64).round() as i32;
    let max_open_dy = ((0.2 * il as f64).round() as i32).max(1);
    // Barline areas of the system (with vertical margin) and long beams are competitors.
    let sys_top = staves[system.staves[0]].top(s.mid_x()) as i32 - 2 * il;
    let sys_bot = staves[*system.staves.last().unwrap()].bottom(s.mid_x()) as i32 + 2 * il;
    let bars: Vec<Rect> = system
        .boundaries
        .iter()
        .map(|b| Rect::new(b.x0, sys_top, b.x1 - b.x0 + 1, sys_bot - sys_top))
        .collect();
    let long_beams: Vec<Rect> = beams
        .iter()
        .filter(|b| b.width() >= 2.5 * il as f64)
        .map(|b| b.bounds())
        .collect();

    let mut scans: Vec<Scan> = Vec::new();
    let left = s.header_stop.max(s.left);
    let right = s.right - min_template_width;
    for p in -5..=5 {
        let open = match p {
            -5 => Some(-1),
            5 => Some(1),
            _ => None,
        };
        let staff = s.clone();
        scans.push(Scan {
            pitch: p,
            x_from: left,
            x_to: right,
            open_dir: open,
            y: Box::new(move |x| staff.pitch_y(p as f64, x as f64).round() as i32),
        });
    }
    for l in ledgers.iter().filter(|l| l.staff == si) {
        let dir = l.index.signum();
        let (ly, x_from, x_to) = (l.y, (l.x0 - il / 2).max(left), l.x1 - il / 2);
        scans.push(Scan {
            pitch: l.pitch(),
            x_from,
            x_to,
            open_dir: None,
            y: Box::new(move |_| ly.round() as i32),
        });
        let half = il as f64 / 2.0;
        scans.push(Scan {
            pitch: l.pitch() + dir,
            x_from,
            x_to,
            open_dir: Some(dir),
            y: Box::new(move |_| (ly + dir as f64 * half).round() as i32),
        });
    }

    let mut heads: Vec<Head> = Vec::new();
    for scan in &scans {
        let y_offsets: Vec<i32> = match scan.open_dir {
            Some(dir) => {
                let mut v = vec![0, dir, -dir];
                for i in 3..=max_open_dy {
                    v.push(dir * (i - 1));
                }
                v.truncate((1 + max_open_dy) as usize);
                v
            }
            None => vec![0, -1],
        };
        let mut found: Vec<Head> = Vec::new();
        let mut x0 = scan.x_from;
        while x0 <= scan.x_to {
            let y0 = (scan.y)(x0);
            let probe = x0 + template_half;
            if probe >= cells.w
                || y0 < 0
                || y0 >= cells.h
                || cells.dist[(y0 * cells.w + probe) as usize] / 3 > template_half
            {
                x0 += 2 * template_half;
                continue;
            }
            for t in &catalog.templates {
                let mut best: Option<(f64, i32)> = None;
                for &dy in &y_offsets {
                    let y = y0 + dy;
                    let slim = t.slim_at(x0, y);
                    if bars.iter().any(|b| b.intersects(&slim))
                        || long_beams.iter().any(|b| b.intersects(&slim))
                    {
                        continue;
                    }
                    let d = t.evaluate(&cells.data, cells.w, cells.h, x0, y);
                    if d <= MAX_DISTANCE_LOW && best.is_none_or(|b| d < b.0) {
                        best = Some((d, y));
                    }
                }
                let Some((d, y)) = best else { continue };
                let mut shape = t.shape;
                if shape == HeadShape::Black {
                    let void = catalog.get(HeadShape::Void);
                    if void.hole_white_ratio(&cells.data, cells.w, cells.h, x0, y)
                        >= MIN_HOLE_WHITE_RATIO
                    {
                        shape = HeadShape::Void;
                    }
                }
                let grade = grade_of(d);
                // Weak stem-less heads are not created.
                if shape.stemless() && grade < 0.5 {
                    continue;
                }
                let bounds = catalog.get(shape).slim_at(x0, y);
                // Pitch from the actual matched position (scans may overlap near ledgers).
                let pitch = s.pitch_of(bounds.cy(), bounds.cx()).round() as i32;
                let _ = scan.pitch;
                let head = Head {
                    staff: si,
                    shape,
                    pitch,
                    bounds,
                    grade,
                    dist: d,
                    ctx: grade,
                };
                // A small (cue / grace) head matching better is not a standard head:
                // it is left to the symbol classifier (grace notes).
                if !shape.stemless()
                    && catalog
                        .get(shape)
                        .fore_black_ratio(&cells.data, cells.w, cells.h, x0, y)
                        < 0.8
                {
                    let mut best_small: Option<(f64, i32, i32)> = None;
                    for st in &catalog.small {
                        for sx in x0 - 2..=x0 + il / 2 {
                            for sy in y - 2..=y + 2 {
                                let ds = st.evaluate(&cells.data, cells.w, cells.h, sx, sy);
                                if best_small.is_none_or(|b| ds < b.0) {
                                    best_small = Some((ds, sx, sy));
                                }
                            }
                        }
                    }
                    if best_small.is_some_and(|b| b.0 < d) {
                        continue;
                    }
                }
                found.push(head);
            }
            x0 += 1;
        }
        // Aggregate matches of this scan that are horizontally close.
        found.sort_by(|a, b| b.grade.partial_cmp(&a.grade).unwrap());
        let mut kept: Vec<Head> = Vec::new();
        let mut anchors: Vec<f64> = Vec::new();
        for h in found {
            if anchors
                .iter()
                .any(|&ax| (h.bounds.cx() - ax).abs() <= max_dx as f64)
            {
                continue;
            }
            anchors.push(h.bounds.cx());
            kept.push(h);
        }
        heads.extend(kept);
    }
    heads
}

/// HeadInter.overlaps: same staff, close pitches and significant horizontal / area overlap.
pub fn overlaps(a: &Head, b: &Head) -> bool {
    let dp = if a.staff == b.staff {
        Some((a.pitch - b.pitch).abs())
    } else {
        None
    };
    if dp.is_some_and(|d| d > 1) {
        return false;
    }
    let Some(common) = a.bounds.intersection(&b.bounds) else {
        return false;
    };
    let min_w = a.bounds.w.min(b.bounds.w);
    let wr = common.w as f64 / min_w as f64;
    if wr <= 0.2 {
        return false;
    }
    if wr >= 0.8 && dp.is_some_and(|d| d <= 1) {
        return true;
    }
    let min_area = a.bounds.area().min(b.bounds.area());
    common.area() as f64 / min_area as f64 > 0.25
}
