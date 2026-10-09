//! Musical assembly (Audiveris MEASURES, CHORDS, LINKS and RHYTHMS steps):
//! chords and rests per measure, their durations (head shape, flags, beams,
//! augmentation dots, tuplets), pitches (clef, key, accidentals), voices and
//! time slots.

use std::collections::HashMap;

use crate::grid::BarStyle;
use crate::header::{ClefKind, TimeSig, digit_of, time_of};
use crate::pipeline::Sheet;
use crate::rational::Q;
use crate::template::HeadShape;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Pitch {
    pub step: char,
    pub octave: i32,
    pub alter: i32,
    /// Accidental printed in front of the note, if any.
    pub accidental: Option<&'static str>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum Kind {
    Chord { pitches: Vec<Pitch>, stem_up: Option<bool> },
    Rest { whole_measure: bool },
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Event {
    /// Staff number within the part (1-based).
    pub staff: u32,
    pub voice: u32,
    pub x: f64,
    pub kind: Kind,
    /// Notated type as fraction of a whole (undotted, without tuplet).
    pub base: Q,
    pub dots: u32,
    pub tuplet: Option<(u32, u32)>,
    pub start: Q,
    pub duration: Q,
    /// (beam number, begin|continue|end|forward hook|backward hook)
    pub beams: Vec<(u32, &'static str)>,
    #[serde(skip)]
    heads: Vec<usize>,
    #[serde(skip)]
    y_mid: f64,
    #[serde(skip)]
    stem: Option<usize>,
    #[serde(skip)]
    right: f64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Direction {
    pub staff: u32,
    pub start: Q,
    pub dynamics: &'static str,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Attributes {
    pub fifths: Option<i32>,
    pub time: Option<TimeSig>,
    /// (staff number, clef)
    pub clefs: Vec<(u32, ClefKind)>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Measure {
    pub new_system: bool,
    pub attributes: Option<Attributes>,
    pub events: Vec<Event>,
    pub directions: Vec<Direction>,
    /// Mid-measure clef changes: (staff, time, clef)
    pub clef_changes: Vec<(u32, Q, ClefKind)>,
    pub expected: Q,
    pub right_bar: BarStyle,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Part {
    pub name: String,
    pub staves: u32,
    pub measures: Vec<Measure>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Score {
    pub parts: Vec<Part>,
}

const SHARP_ORDER: [usize; 7] = [3, 0, 4, 1, 5, 2, 6]; // F C G D A E B
const FLAT_ORDER: [usize; 7] = [6, 2, 5, 1, 4, 0, 3]; // B E A D G C F
const STEPS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];

fn key_alter(fifths: i32, step: usize) -> i32 {
    if fifths > 0 {
        if SHARP_ORDER[..fifths as usize].contains(&step) { 1 } else { 0 }
    } else if fifths < 0 {
        if FLAT_ORDER[..(-fifths) as usize].contains(&step) { -1 } else { 0 }
    } else {
        0
    }
}

fn alter_of(shape: &str) -> Option<i32> {
    Some(match shape {
        "SHARP" => 1,
        "FLAT" => -1,
        "NATURAL" => 0,
        "DOUBLE_SHARP" => 2,
        "DOUBLE_FLAT" => -2,
        _ => return None,
    })
}

fn accidental_name(shape: &str) -> &'static str {
    match shape {
        "SHARP" => "sharp",
        "FLAT" => "flat",
        "NATURAL" => "natural",
        "DOUBLE_SHARP" => "double-sharp",
        _ => "flat-flat",
    }
}

fn rest_base(shape: &str) -> Option<Q> {
    Some(match shape {
        "LONG_REST" => Q::new(4, 1),
        "BREVE_REST" => Q::new(2, 1),
        "QUARTER_REST" => Q::new(1, 4),
        "EIGHTH_REST" => Q::new(1, 8),
        "ONE_16TH_REST" => Q::new(1, 16),
        "ONE_32ND_REST" => Q::new(1, 32),
        "ONE_64TH_REST" => Q::new(1, 64),
        "ONE_128TH_REST" => Q::new(1, 128),
        _ => return None,
    })
}

fn flag_count(shape: &str) -> u32 {
    let s = shape.trim_start_matches("FLAG_").trim_end_matches("_DOWN");
    s.parse().unwrap_or(0)
}

fn mode(values: impl Iterator<Item = i32>) -> Option<i32> {
    let mut m: HashMap<i32, usize> = HashMap::new();
    for v in values {
        *m.entry(v).or_default() += 1;
    }
    m.into_iter().max_by_key(|(v, c)| (*c, -v.abs())).map(|(v, _)| v)
}

pub fn build(sheet: &Sheet) -> Score {
    let il = sheet.scale.interline as f64;
    let mut parts: Vec<Part> = Vec::new();
    let mut clef_state: HashMap<(usize, u32), ClefKind> = HashMap::new();
    let mut fifths_state: Option<i32> = None;
    let mut time_state: Option<TimeSig> = None;

    for sys in &sheet.systems {
        // System-wide key and time from the staff headers.
        let fifths = mode(sys.staves.iter().map(|&s| sheet.headers[s].fifths)).unwrap_or(0);
        let time = sys.staves.iter().find_map(|&s| sheet.headers[s].time);
        for (pi, part_staves) in sys.parts.iter().enumerate() {
            if parts.len() <= pi {
                parts.push(Part { name: format!("Part {}", pi + 1), staves: part_staves.len() as u32, measures: Vec::new() });
            }
            let part = &mut parts[pi];
            part.staves = part.staves.max(part_staves.len() as u32);
            // Clefs at system start.
            let mut clefs_now: HashMap<u32, ClefKind> = HashMap::new();
            let mut changed_clefs = Vec::new();
            for (k, &si) in part_staves.iter().enumerate() {
                let n = k as u32 + 1;
                let prev = clef_state.get(&(pi, n)).copied();
                let c = sheet.headers[si].clef.or(prev).unwrap_or(if k == 0 { ClefKind::G } else { ClefKind::F });
                if prev != Some(c) {
                    changed_clefs.push((n, c));
                }
                clefs_now.insert(n, c);
            }
            for mi in 0..sys.boundaries.len().saturating_sub(1) {
                let x_lo = sys.boundaries[mi].x1 as f64;
                let x_hi = sys.boundaries[mi + 1].x0 as f64;
                let mut attrs = Attributes { fifths: None, time: None, clefs: Vec::new() };
                if mi == 0 {
                    if fifths_state != Some(fifths) {
                        attrs.fifths = Some(fifths);
                        fifths_state = Some(fifths);
                    }
                    if let Some(t) = time
                        && time_state != Some(t)
                    {
                        attrs.time = Some(t);
                        time_state = Some(t);
                    }
                    if time_state.is_none() {
                        let t = TimeSig { beats: 4, beat_type: 4, symbol: None };
                        attrs.time = Some(t);
                        time_state = Some(t);
                    }
                    attrs.clefs = changed_clefs.clone();
                }
                // Mid-system time signature (from symbols) at the start of this measure.
                if mi > 0
                    && let Some(t) = measure_time_change(sheet, part_staves, x_lo, x_lo + 4.0 * il)
                    && time_state != Some(t)
                {
                    attrs.time = Some(t);
                    time_state = Some(t);
                }
                let ts = time_state.unwrap();
                let expected = Q::new(ts.beats as i64, ts.beat_type as i64);
                let mut m = Measure {
                    new_system: mi == 0,
                    attributes: None,
                    events: Vec::new(),
                    directions: Vec::new(),
                    clef_changes: Vec::new(),
                    expected,
                    right_bar: sys.boundaries[mi + 1].style,
                };
                build_measure(sheet, part_staves, x_lo, x_hi, fifths, &mut clefs_now, &mut m);
                for (n, _, c) in &m.clef_changes {
                    clefs_now.insert(*n, *c);
                }
                if attrs.fifths.is_some() || attrs.time.is_some() || !attrs.clefs.is_empty() {
                    m.attributes = Some(attrs);
                }
                part.measures.push(m);
            }
            for (n, c) in clefs_now {
                clef_state.insert((pi, n), c);
            }
        }
    }
    Score { parts }
}

fn measure_time_change(sheet: &Sheet, part_staves: &[usize], x0: f64, x1: f64) -> Option<TimeSig> {
    let syms: Vec<_> =
        sheet.symbols.iter().filter(|s| part_staves.contains(&s.staff) && s.bounds.cx() > x0 && s.bounds.cx() < x1).collect();
    for s in &syms {
        if let Some(t) = time_of(s.shape) {
            return Some(t);
        }
    }
    let digits: Vec<_> = syms.iter().filter_map(|s| digit_of(s.shape).map(|d| (d, s))).collect();
    if digits.len() >= 2 {
        let (a, b) = (digits[0], digits[1]);
        let (num, den) = if a.1.bounds.cy() < b.1.bounds.cy() { (a.0, b.0) } else { (b.0, a.0) };
        if [2, 4, 8, 16].contains(&den) && num > 0 {
            return Some(TimeSig { beats: num, beat_type: den, symbol: None });
        }
    }
    None
}

fn build_measure(
    sheet: &Sheet,
    part_staves: &[usize],
    x_lo: f64,
    x_hi: f64,
    fifths: i32,
    clefs: &mut HashMap<u32, ClefKind>,
    m: &mut Measure,
) {
    let il = sheet.scale.interline as f64;
    let staff_no = |si: usize| part_staves.iter().position(|&s| s == si).map(|p| p as u32 + 1);
    let in_measure = |x: f64| x > x_lo && x < x_hi;
    let syms: Vec<&crate::symbols::Symbol> =
        sheet.symbols.iter().filter(|s| part_staves.contains(&s.staff) && in_measure(s.bounds.cx())).collect();

    // Clef changes inside the measure.
    let mut clef_changes: Vec<(u32, f64, ClefKind)> = Vec::new();
    for s in &syms {
        if s.shape.contains("CLEF") {
            let st = &sheet.staves[s.staff];
            let pc = st.pitch_of(s.bounds.cy(), s.bounds.cx());
            if let Some(c) = ClefKind::from_shape(s.shape, pc) {
                clef_changes.push((staff_no(s.staff).unwrap(), s.bounds.x as f64, c));
            }
        }
    }
    let clef_at = |n: u32, x: f64, clefs: &HashMap<u32, ClefKind>| -> ClefKind {
        clef_changes.iter().filter(|c| c.0 == n && c.1 < x).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|c| c.2).unwrap_or(clefs[&n])
    };

    // Accidentals: (staff, pitch, x right edge, shape)
    let accidentals: Vec<(usize, i32, f64, &'static str)> = syms
        .iter()
        .filter(|s| alter_of(s.shape).is_some())
        .map(|s| {
            let st = &sheet.staves[s.staff];
            let yref = if s.shape.contains("FLAT") { s.bounds.bottom() as f64 - 0.45 * il } else { s.bounds.cy() };
            (s.staff, st.pitch_of(yref, s.bounds.cx()).round() as i32, s.bounds.right() as f64, s.shape)
        })
        .collect();

    let mut events: Vec<Event> = Vec::new();
    // Chords from stems.
    let mut used_heads = vec![false; sheet.heads.len()];
    for (sti, stem) in sheet.stems.iter().enumerate() {
        if !in_measure(stem.x as f64) {
            continue;
        }
        let staff_votes = mode(stem.heads.iter().map(|&h| sheet.heads[h].staff as i32));
        let Some(si) = staff_votes.map(|s| s as usize) else { continue };
        let Some(n) = staff_no(si) else { continue };
        let heads: Vec<usize> = stem.heads.iter().copied().filter(|&h| sheet.heads[h].staff == si).collect();
        for &h in &heads {
            used_heads[h] = true;
        }
        let shape = sheet.heads[heads[0]].shape;
        let flags: u32 = if stem.beams.is_empty() {
            syms.iter()
                .filter(|s| s.shape.starts_with("FLAG_"))
                .filter(|s| {
                    let dx = s.bounds.x as f64 - stem.x as f64;
                    let tip_zone = if stem.up {
                        s.bounds.y as f64 <= stem.top as f64 + 1.0 * il && s.bounds.bottom() as f64 >= stem.top as f64
                    } else {
                        s.bounds.bottom() as f64 >= stem.bottom as f64 - 1.0 * il && (s.bounds.y as f64) <= stem.bottom as f64
                    };
                    (-0.4 * il..=0.6 * il).contains(&dx) && tip_zone
                })
                .map(|s| flag_count(s.shape))
                .sum()
        } else {
            0
        };
        let level = (stem.beams.len() as u32).max(flags);
        let base = match shape {
            HeadShape::Black => Q::new(1, 4 * (1 << level)),
            HeadShape::Void => Q::new(1, 2),
            HeadShape::Whole => Q::new(1, 1),
            HeadShape::Breve => Q::new(2, 1),
        };
        let x = heads.iter().map(|&h| sheet.heads[h].bounds.cx()).sum::<f64>() / heads.len() as f64;
        let right = heads.iter().map(|&h| sheet.heads[h].bounds.right()).max().unwrap() as f64;
        let y_mid = heads.iter().map(|&h| sheet.heads[h].bounds.cy()).sum::<f64>() / heads.len() as f64;
        let mut hs = heads.clone();
        hs.sort_by_key(|&h| -sheet.heads[h].pitch);
        events.push(Event {
            staff: n,
            voice: 0,
            x,
            kind: Kind::Chord { pitches: Vec::new(), stem_up: Some(stem.up) },
            heads: hs,
            base,
            dots: 0,
            tuplet: None,
            start: Q::ZERO,
            duration: Q::ZERO,
            beams: Vec::new(),
            y_mid,
            stem: Some(sti),
            right,
        });
    }
    // Stem-less heads (whole notes, breves), grouped by abscissa.
    let mut loose: Vec<usize> = (0..sheet.heads.len())
        .filter(|&h| !used_heads[h] && sheet.heads[h].shape.stemless())
        .filter(|&h| part_staves.contains(&sheet.heads[h].staff) && in_measure(sheet.heads[h].bounds.cx()))
        .collect();
    loose.sort_by(|&a, &b| sheet.heads[a].bounds.x.cmp(&sheet.heads[b].bounds.x));
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for h in loose {
        let hb = sheet.heads[h].bounds;
        match groups.iter_mut().find(|g| {
            let f = &sheet.heads[g[0]];
            f.staff == sheet.heads[h].staff && (f.bounds.cx() - hb.cx()).abs() < 0.8 * il
        }) {
            Some(g) => g.push(h),
            None => groups.push(vec![h]),
        }
    }
    for g in groups {
        let si = sheet.heads[g[0]].staff;
        let base = if sheet.heads[g[0]].shape == HeadShape::Breve { Q::new(2, 1) } else { Q::new(1, 1) };
        let mut hs = g.clone();
        hs.sort_by_key(|&h| -sheet.heads[h].pitch);
        events.push(Event {
            staff: staff_no(si).unwrap(),
            voice: 0,
            x: g.iter().map(|&h| sheet.heads[h].bounds.cx()).sum::<f64>() / g.len() as f64,
            kind: Kind::Chord { pitches: Vec::new(), stem_up: None },
            heads: hs,
            base,
            dots: 0,
            tuplet: None,
            start: Q::ZERO,
            duration: Q::ZERO,
            beams: Vec::new(),
            y_mid: g.iter().map(|&h| sheet.heads[h].bounds.cy()).sum::<f64>() / g.len() as f64,
            stem: None,
            right: g.iter().map(|&h| sheet.heads[h].bounds.right()).max().unwrap() as f64,
        });
    }
    // Rests.
    for s in &syms {
        let st = &sheet.staves[s.staff];
        let (cx, cy) = (s.bounds.cx(), s.bounds.cy());
        if cy < st.top(cx) - 1.5 * il || cy > st.bottom(cx) + 1.5 * il {
            continue;
        }
        let base = if s.shape == "HW_REST_set" {
            // Whole rest hangs from the 4th line, half rest sits on the middle line.
            let top = s.bounds.y as f64;
            let bot = s.bounds.bottom() as f64;
            if (top - st.pitch_y(-2.0, cx)).abs() <= (bot - st.pitch_y(0.0, cx)).abs() { Q::new(1, 1) } else { Q::new(1, 2) }
        } else if let Some(b) = rest_base(s.shape) {
            b
        } else {
            continue;
        };
        events.push(Event {
            staff: staff_no(s.staff).unwrap(),
            voice: 0,
            x: cx,
            kind: Kind::Rest { whole_measure: false },
            heads: Vec::new(),
            base,
            dots: 0,
            tuplet: None,
            start: Q::ZERO,
            duration: Q::ZERO,
            beams: Vec::new(),
            y_mid: cy,
            stem: None,
            right: s.bounds.right() as f64,
        });
    }

    // Augmentation dots.
    let dots: Vec<&crate::symbols::Symbol> = syms.iter().copied().filter(|s| s.shape == "DOT_set").collect();
    for ev in events.iter_mut() {
        let (top, bot) = match &ev.kind {
            Kind::Chord { .. } => {
                let hs = &ev.heads;
                (
                    hs.iter().map(|&h| sheet.heads[h].bounds.y).min().unwrap_or(0) as f64 - 0.5 * il,
                    hs.iter().map(|&h| sheet.heads[h].bounds.bottom()).max().unwrap_or(0) as f64 + 0.5 * il,
                )
            }
            Kind::Rest { .. } => (ev.y_mid - 1.0 * il, ev.y_mid + 1.0 * il),
        };
        let mut row: Vec<f64> = dots
            .iter()
            .filter(|d| d.bounds.cy() >= top && d.bounds.cy() <= bot && d.bounds.w as f64 <= 0.8 * il)
            .map(|d| d.bounds.x as f64)
            .filter(|&x| x >= ev.right - 1.0 && x <= ev.right + 1.5 * il)
            .collect();
        row.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if let Some(&first) = row.first() {
            ev.dots = 1;
            if row.iter().any(|&x| x > first + 0.3 * il && x <= first + 1.2 * il) {
                ev.dots = 2;
            }
        }
    }

    // Tuplets attached to beam groups.
    for s in syms.iter().filter(|s| s.shape == "TUPLET_THREE" || s.shape == "TUPLET_SIX") {
        let (cx, cy) = (s.bounds.cx(), s.bounds.cy());
        let beam = sheet
            .beams
            .iter()
            .enumerate()
            .filter(|(_, b)| cx >= b.x0 - il && cx <= b.x1 + il && (b.y_at(cx.clamp(b.x0, b.x1)) - cy).abs() <= 2.5 * il)
            .min_by(|a, b| (a.1.y_at(cx) - cy).abs().partial_cmp(&(b.1.y_at(cx) - cy).abs()).unwrap());
        if let Some((bi, _)) = beam {
            for ev in events.iter_mut() {
                if let Some(st) = ev.stem
                    && sheet.stems[st].beams.contains(&bi)
                {
                    ev.tuplet = Some((3, 2));
                }
            }
        }
    }

    // Pitches.
    let mut measure_alters: HashMap<(u32, i32), i32> = HashMap::new();
    events.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    for ev in events.iter_mut() {
        let Kind::Chord { pitches, .. } = &mut ev.kind else { continue };
        let mut out = Vec::new();
        for &h in &ev.heads {
            let head = &sheet.heads[h];
            let clef = clef_at(ev.staff, head.bounds.x as f64, clefs);
            let idx = clef.middle_line() - head.pitch;
            let step = idx.rem_euclid(7) as usize;
            let octave = idx.div_euclid(7);
            // Explicit accidental just left of the head.
            let acc = accidentals
                .iter()
                .filter(|a| a.0 == head.staff && a.1 == head.pitch)
                .filter(|a| a.2 <= head.bounds.x as f64 + 2.0 && a.2 >= head.bounds.x as f64 - 3.5 * il)
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            let alter = if let Some(a) = acc {
                let v = alter_of(a.3).unwrap();
                measure_alters.insert((ev.staff, idx), v);
                v
            } else {
                measure_alters.get(&(ev.staff, idx)).copied().unwrap_or_else(|| key_alter(fifths, step))
            };
            out.push(Pitch { step: STEPS[step], octave, alter, accidental: acc.map(|a| accidental_name(a.3)) });
        }
        *pitches = out;
    }

    // Durations.
    for ev in events.iter_mut() {
        let dot_factor = match ev.dots {
            0 => Q::new(1, 1),
            1 => Q::new(3, 2),
            _ => Q::new(7, 4),
        };
        ev.duration = ev.base * dot_factor;
        if let Some((a, b)) = ev.tuplet {
            ev.duration = ev.duration * Q::new(b as i64, a as i64);
        }
    }
    // A whole rest alone in its staff fills the measure.
    let staff_counts = |n: u32, evs: &[Event]| evs.iter().filter(|e| e.staff == n).count();
    let counts: Vec<usize> = events.iter().map(|e| staff_counts(e.staff, &events)).collect();
    for (ev, c) in events.iter_mut().zip(counts) {
        if matches!(ev.kind, Kind::Rest { .. }) && ev.base == Q::new(1, 1) && c == 1 {
            ev.kind = Kind::Rest { whole_measure: true };
            ev.duration = m.expected;
        }
    }

    assign_voices(&mut events, il);
    assign_times(&mut events, il);
    assign_beams(sheet, &mut events, il);

    // Dynamics directions, attached to the next event of the staff.
    for s in syms.iter().filter(|s| s.shape.starts_with("DYNAMICS_")) {
        let n = staff_no(s.staff).unwrap();
        let next = events.iter().filter(|e| e.staff == n).min_by(|a, b| (a.x - s.bounds.cx()).abs().partial_cmp(&(b.x - s.bounds.cx()).abs()).unwrap());
        let start = next.map(|e| e.start).unwrap_or(Q::ZERO);
        let name = match s.shape {
            "DYNAMICS_P" => "p",
            "DYNAMICS_PP" => "pp",
            "DYNAMICS_PPP" => "ppp",
            "DYNAMICS_MP" => "mp",
            "DYNAMICS_F" => "f",
            "DYNAMICS_FF" => "ff",
            "DYNAMICS_FFF" => "fff",
            "DYNAMICS_MF" => "mf",
            "DYNAMICS_FP" => "fp",
            "DYNAMICS_FZ" => "fz",
            "DYNAMICS_SF" => "sf",
            _ => "sfz",
        };
        m.directions.push(Direction { staff: n, start, dynamics: name });
    }
    for (n, x, c) in clef_changes {
        let t = events.iter().filter(|e| e.staff == n && e.x > x).map(|e| e.start).min().unwrap_or(Q::ZERO);
        m.clef_changes.push((n, t, c));
    }
    m.events = events;
}

/// Voices: one per staff unless chords overlap in time slots, in which case
/// up-stems form the upper voice and down-stems the lower one.
fn assign_voices(events: &mut [Event], il: f64) {
    let staves: Vec<u32> = {
        let mut v: Vec<u32> = events.iter().map(|e| e.staff).collect();
        v.sort();
        v.dedup();
        v
    };
    for n in staves {
        let idx: Vec<usize> = (0..events.len()).filter(|&i| events[i].staff == n).collect();
        let tol = 0.8 * il;
        let poly = idx.iter().any(|&i| idx.iter().any(|&j| j != i && (events[i].x - events[j].x).abs() < tol));
        let upper = 1 + 4 * (n - 1);
        for &i in &idx {
            events[i].voice = upper;
        }
        if !poly {
            continue;
        }
        let mid_y = {
            let ys: Vec<f64> = idx.iter().map(|&i| events[i].y_mid).collect();
            ys.iter().sum::<f64>() / ys.len() as f64
        };
        for &i in &idx {
            let lower = match &events[i].kind {
                Kind::Chord { stem_up: Some(up), .. } => !up,
                _ => events[i].y_mid > mid_y,
            };
            if lower {
                events[i].voice = upper + 1;
            }
        }
    }
}

/// Time slots: events aligned in abscissa share the same start time. Each
/// event starts when its voice is available and its slot begins.
fn assign_times(events: &mut [Event], il: f64) {
    let mut order: Vec<usize> = (0..events.len()).collect();
    order.sort_by(|&a, &b| events[a].x.partial_cmp(&events[b].x).unwrap());
    let mut slots: Vec<Vec<usize>> = Vec::new();
    for i in order {
        match slots.last_mut() {
            Some(s) if (events[i].x - events[s[0]].x).abs() <= 0.75 * il && s.iter().all(|&j| events[j].voice != events[i].voice) => s.push(i),
            _ => slots.push(vec![i]),
        }
    }
    let mut voice_time: HashMap<u32, Q> = HashMap::new();
    let mut last = Q::ZERO;
    for s in slots {
        let mut t = s.iter().map(|&i| voice_time.get(&events[i].voice).copied().unwrap_or(Q::ZERO)).max().unwrap();
        if t < last {
            t = last;
        }
        for &i in &s {
            events[i].start = t;
            voice_time.insert(events[i].voice, t + events[i].duration);
        }
        last = t;
    }
}

/// MusicXML beam values from the stems' beams.
fn assign_beams(sheet: &Sheet, events: &mut [Event], il: f64) {
    for ev in events.iter_mut() {
        let Some(si) = ev.stem else { continue };
        let stem = &sheet.stems[si];
        for (level, &bi) in stem.beams.iter().enumerate() {
            let mut xs: Vec<i32> = sheet.stems.iter().filter(|s| s.beams.contains(&bi)).map(|s| s.x).collect();
            xs.sort();
            let value = if xs.len() == 1 {
                let b = &sheet.beams[bi];
                if b.x1 - stem.x as f64 > 0.5 * il { "forward hook" } else { "backward hook" }
            } else if stem.x == xs[0] {
                "begin"
            } else if stem.x == *xs.last().unwrap() {
                "end"
            } else {
                "continue"
            };
            ev.beams.push((level as u32 + 1, value));
        }
    }
}
