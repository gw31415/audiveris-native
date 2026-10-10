//! Musical assembly (Audiveris MEASURES, CHORDS, LINKS and RHYTHMS steps):
//! chords and rests per measure, their durations (head shape, flags, beams,
//! augmentation dots, tuplets), pitches (clef, key, accidentals), voices and
//! time slots.

use std::collections::HashMap;

use crate::binary::Rect;
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
    Chord {
        pitches: Vec<Pitch>,
        stem_up: Option<bool>,
    },
    Rest {
        whole_measure: bool,
    },
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
    /// MusicXML articulation names (staccato, accent, ...).
    pub articulations: Vec<&'static str>,
    /// Fermata type (upright / inverted).
    pub fermata: Option<&'static str>,
    /// Ornaments (MusicXML element names).
    pub ornaments: Vec<&'static str>,
    /// Arpeggiato sign on the left of the chord.
    pub arpeggiate: bool,
    /// Single tremolo strokes.
    pub tremolo: u32,
    /// Grace notes before the chord.
    pub graces: Vec<Grace>,
    /// Ties: (index of the note in the chord, start?).
    pub ties: Vec<(usize, bool)>,
    /// Slurs: (number, start?).
    pub slurs: Vec<(u32, bool)>,
    #[serde(skip)]
    heads: Vec<usize>,
    /// Bounds of the chord (heads and stem) or rest.
    #[serde(skip)]
    bbox: Rect,
    #[serde(skip)]
    y_mid: f64,
    #[serde(skip)]
    stem: Option<usize>,
    #[serde(skip)]
    right: f64,
}

/// Grace chord (Audiveris GraceChordInter / SmallChordInter).
#[derive(Clone, Debug, serde::Serialize)]
pub struct Grace {
    pub pitches: Vec<Pitch>,
    pub slash: bool,
    pub stem_up: bool,
    /// Notated type (eighth, 16th, 32nd).
    pub base: Q,
    /// (staff, pitch position, abscissa) of each head.
    #[serde(skip)]
    heads: Vec<(usize, i32, f64)>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Direction {
    pub staff: u32,
    pub start: Q,
    /// MusicXML direction type, e.g. `<dynamics><p/></dynamics>` or `<wedge type="stop"/>`.
    pub kind: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Attributes {
    pub fifths: Option<i32>,
    pub time: Option<TimeSig>,
    /// (staff number, clef)
    pub clefs: Vec<(u32, ClefKind)>,
    /// Measure count of a multiple rest starting here.
    pub multiple_rest: Option<u32>,
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
    /// Repeat signs: (forward at the left bar, backward at the right bar).
    pub repeat: (bool, bool),
    /// Ending (volta) number starting at the left bar, and (number, with a
    /// right leg) stopping at the right bar.
    pub ending: (Option<u32>, Option<(u32, bool)>),
    /// Pickup or second half of a measure split by a repeat (not counted).
    pub implicit: bool,
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
/// Pitch offset from area center to focus line of the Bravura flat glyph
/// (bounding box from -0.7 to +1.756 staff spaces around the baseline).
const FLAT_AREA_PITCH_OFFSET: f64 = 1.056;
const STEPS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];

fn key_alter(fifths: i32, step: usize) -> i32 {
    if fifths > 0 {
        if SHARP_ORDER[..fifths as usize].contains(&step) {
            1
        } else {
            0
        }
    } else if fifths < 0 {
        if FLAT_ORDER[..(-fifths) as usize].contains(&step) {
            -1
        } else {
            0
        }
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
    m.into_iter()
        .max_by_key(|(v, c)| (*c, -v.abs()))
        .map(|(v, _)| v)
}

/// Musical state carried from one page to the next.
#[derive(Default)]
pub struct State {
    /// Staff counts of the parts of the fullest system seen so far.
    template: Vec<usize>,
    clefs: HashMap<(usize, u32), ClefKind>,
    fifths: Option<i32>,
    time: Option<TimeSig>,
}

/// Align system parts to template parts by staff count, from the bottom
/// (a missing part is typically a hidden empty vocal staff above the piano).
fn align(counts: &[usize], template: &[usize]) -> Vec<Option<usize>> {
    let mut map = vec![None; counts.len()];
    let (mut i, mut j) = (counts.len(), template.len());
    while i > 0 && j > 0 {
        if counts[i - 1] == template[j - 1] {
            map[i - 1] = Some(j - 1);
            i -= 1;
        }
        j -= 1;
    }
    map
}

fn filler(m: &Measure) -> Measure {
    Measure {
        new_system: m.new_system,
        attributes: None,
        events: Vec::new(),
        directions: Vec::new(),
        clef_changes: Vec::new(),
        expected: m.expected,
        right_bar: m.right_bar,
        repeat: (false, false),
        ending: (None, None),
        implicit: false,
    }
}

pub fn build(sheet: &Sheet, state: &mut State) -> Score {
    let il = sheet.scale.interline as f64;
    let mut parts: Vec<Part> = Vec::new();
    let mut clef_state = std::mem::take(&mut state.clefs);
    let mut fifths_state = state.fifths;
    let mut time_state = state.time;

    // Parts already built on previous pages are carried by the book; here we
    // only need as many as the template.
    for _ in 0..state.template.len() {
        parts.push(Part {
            name: format!("Part {}", parts.len() + 1),
            staves: 1,
            measures: Vec::new(),
        });
    }
    for sys in &sheet.systems {
        // System-wide key and time from the staff headers.
        let fifths = mode(sys.staves.iter().map(|&s| sheet.headers[s].fifths)).unwrap_or(0);
        let time = column_time(sys.staves.iter().map(|&s| sheet.headers[s].time));
        let counts: Vec<usize> = sys.parts.iter().map(|p| p.len()).collect();
        if counts.len() > state.template.len() {
            // Grow the template: existing parts keep their place.
            let map = align(&state.template, &counts);
            let len = parts.iter().map(|p| p.measures.len()).max().unwrap_or(0);
            let model: Vec<Measure> = parts
                .iter()
                .find(|p| p.measures.len() == len)
                .map(|p| p.measures.iter().map(filler).collect())
                .unwrap_or_default();
            let mut grown: Vec<Option<Part>> = (0..counts.len()).map(|_| None).collect();
            for (old, part) in parts.drain(..).enumerate() {
                if let Some(Some(new)) = map.get(old) {
                    grown[*new] = Some(part);
                }
            }
            parts = grown
                .into_iter()
                .enumerate()
                .map(|(k, p)| {
                    p.unwrap_or(Part {
                        name: String::new(),
                        staves: counts[k] as u32,
                        measures: model.clone(),
                    })
                })
                .collect();
            for (k, p) in parts.iter_mut().enumerate() {
                p.name = format!("Part {}", k + 1);
            }
            state.template = counts.clone();
        }
        let map = align(&counts, &state.template);
        let n_measures = sys.boundaries.len().saturating_sub(1);
        for ti in 0..state.template.len() {
            if map.contains(&Some(ti)) {
                continue;
            }
            // Absent (hidden) part in this system: empty measures.
            let model = parts.iter().find(|p| !p.measures.is_empty());
            let expected = time_state
                .map(|t| Q::new(t.beats as i64, t.beat_type as i64))
                .unwrap_or(Q::new(1, 1));
            let _ = model;
            for mi in 0..n_measures {
                parts[ti].measures.push(Measure {
                    new_system: mi == 0,
                    attributes: None,
                    events: Vec::new(),
                    directions: Vec::new(),
                    clef_changes: Vec::new(),
                    expected,
                    right_bar: sys.boundaries[mi + 1].style,
                    repeat: (false, false),
                    ending: (None, None),
                    implicit: false,
                });
            }
        }
        let sys_fifths = fifths_state;
        let (repeats, repeat_dots) = repeat_signs(sheet, sys);
        let endings = system_endings(sheet, sys);
        // Multiple rests decided for the whole system (all parts expand alike).
        let multi: Vec<Option<u32>> = (0..sys.boundaries.len().saturating_sub(1))
            .map(|mi| multiple_rest(sheet, &sys.staves, sys.boundaries[mi].x1 as f64, sys.boundaries[mi + 1].x0 as f64))
            .collect();
        for (spi, part_staves) in sys.parts.iter().enumerate() {
            let Some(pi) = map[spi] else { continue };
            let part = &mut parts[pi];
            let mut part_fifths = sys_fifths;
            let mut fifths = fifths;
            part.staves = part.staves.max(part_staves.len() as u32);
            // Clefs at system start.
            let mut clefs_now: HashMap<u32, ClefKind> = HashMap::new();
            let mut changed_clefs = Vec::new();
            for (k, &si) in part_staves.iter().enumerate() {
                let n = k as u32 + 1;
                let prev = clef_state.get(&(pi, n)).copied();
                let c = sheet.headers[si].clef.or(prev).unwrap_or(if k == 0 {
                    ClefKind::G
                } else {
                    ClefKind::F
                });
                if prev != Some(c) {
                    changed_clefs.push((n, c));
                }
                clefs_now.insert(n, c);
            }
            for mi in 0..sys.boundaries.len().saturating_sub(1) {
                let x_lo = sys.boundaries[mi].x1 as f64;
                let x_hi = sys.boundaries[mi + 1].x0 as f64;
                let mut attrs = Attributes {
                    fifths: None,
                    time: None,
                    clefs: Vec::new(),
                    multiple_rest: None,
                };
                if mi == 0 {
                    if part_fifths != Some(fifths) {
                        attrs.fifths = Some(fifths);
                        part_fifths = Some(fifths);
                    }
                    if let Some(t) = time
                        && time_state != Some(t)
                    {
                        attrs.time = Some(t);
                        time_state = Some(t);
                    }
                    if time_state.is_none() {
                        let t = TimeSig {
                            beats: 4,
                            beat_type: 4,
                            symbol: None,
                        };
                        attrs.time = Some(t);
                        time_state = Some(t);
                    }
                    attrs.clefs = changed_clefs.clone();
                }
                // Mid-system time signature (from symbols) at the start of this measure.
                if mi > 0
                    && let Some(t) = measure_time_change(sheet, &sys.staves, x_lo, x_lo + 4.0 * il)
                    && time_state != Some(t)
                {
                    attrs.time = Some(t);
                    time_state = Some(t);
                }
                // Key change after the opening barline of this measure (majority of staves).
                if mi > 0
                    && let Some(f) = mode(sheet.key_changes.iter().filter(|k| {
                        part_staves.contains(&k.0) && k.1 == sys.boundaries[mi].x1
                    }).map(|k| k.2))
                {
                    fifths = f;
                    if part_fifths != Some(f) {
                        attrs.fifths = Some(f);
                        part_fifths = Some(f);
                    }
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
                    repeat: (false, false),
                    ending: (None, None),
                    implicit: false,
                };
                m.repeat = (repeats[mi].1, repeats[mi + 1].0);
                m.ending = endings[mi];
                build_measure(
                    sheet,
                    part_staves,
                    x_lo,
                    x_hi,
                    fifths,
                    &mut clefs_now,
                    &repeat_dots,
                    &mut m,
                );
                for (n, _, c) in &m.clef_changes {
                    clefs_now.insert(*n, *c);
                }
                if let Some(n) = multi[mi] {
                    m.events.clear();
                    m.directions.clear();
                    attrs.multiple_rest = Some(n);
                }
                if attrs.fifths.is_some() || attrs.time.is_some() || !attrs.clefs.is_empty() || attrs.multiple_rest.is_some() {
                    m.attributes = Some(attrs);
                }
                let extra = multi[mi].map_or(0, |n| n - 1);
                let mut fill = filler(&m);
                fill.new_system = false;
                fill.right_bar = crate::grid::BarStyle::Regular;
                let last_bar = m.right_bar;
                if extra > 0 {
                    m.right_bar = crate::grid::BarStyle::Regular;
                }
                part.measures.push(m);
                for k in 0..extra {
                    let mut f = filler(&fill);
                    if k + 1 == extra {
                        f.right_bar = last_bar;
                    }
                    part.measures.push(f);
                }
            }
            fifths_state = part_fifths;
            for (n, c) in clefs_now {
                clef_state.insert((pi, n), c);
            }
        }
    }
    state.clefs = clef_state;
    state.fifths = fifths_state;
    state.time = time_state;
    link_slurs(sheet, &mut parts);
    fix_measures(&mut parts);
    Score { parts }
}

/// Slurs and ties (Audiveris SlurLinker, ClumpPruner and SlurInter): each
/// end links to the closest head in a lookup area extending 2 il outwards,
/// 0.5 il inwards and 3 il on the concave side. Both ends on heads of the
/// same pitch and staff, with no chord in between and at most one barline
/// crossed, make a tie. Orphan ends near the staff ends are paired across
/// systems (checkCrossTie).
fn link_slurs(sheet: &Sheet, parts: &mut [Part]) {
    let il = sheet.scale.interline as f64;
    // (part, measure, event) of every chord, with its sheet staff.
    let mut chords: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (pi, p) in parts.iter().enumerate() {
        for (mi, m) in p.measures.iter().enumerate() {
            for (ei, e) in m.events.iter().enumerate() {
                if matches!(e.kind, Kind::Chord { .. }) && !e.heads.is_empty() {
                    chords.push((pi, mi, ei, sheet.heads[e.heads[0]].staff));
                }
            }
        }
    }
    let ev = |parts: &[Part], c: (usize, usize, usize, usize)| -> Event { parts[c.0].measures[c.1].events[c.2].clone() };
    // Head (chord index, note index) closest to an end, inside its area.
    let lookup = |parts: &[Part], end: (f64, f64), left: bool, above: bool| -> Option<(usize, usize, f64)> {
        let (x0, x1) = if left { (end.0 - 2.0 * il, end.0 + 0.5 * il) } else { (end.0 - 0.5 * il, end.0 + 2.0 * il) };
        let (y0, y1) = if above { (end.1 - 0.5 * il, end.1 + 3.0 * il) } else { (end.1 - 3.0 * il, end.1 + 0.5 * il) };
        // Chords (heads and stem) intersecting the area; the closest one, and
        // its head closest to the end.
        let mut best: Option<(usize, usize, f64)> = None;
        for (ci, &c) in chords.iter().enumerate() {
            let e = &parts[c.0].measures[c.1].events[c.2];
            let b = e.bbox;
            if (b.right() as f64) < x0 || (b.x as f64) > x1 || (b.bottom() as f64) < y0 || (b.y as f64) > y1 {
                continue;
            }
            let dx = (b.x as f64 - end.0).max(end.0 - b.right() as f64).max(0.0);
            let dy = (b.y as f64 - end.1).max(end.1 - b.bottom() as f64).max(0.0);
            let d = dx.hypot(dy);
            if best.is_none_or(|(_, _, bd)| d < bd) {
                let k = (0..e.heads.len())
                    .min_by(|&a, &b| {
                        let dist = |k: usize| {
                            let h = sheet.heads[e.heads[k]].bounds;
                            (h.cx() - end.0).hypot(h.cy() - end.1)
                        };
                        dist(a).partial_cmp(&dist(b)).unwrap()
                    })
                    .unwrap();
                best = Some((ci, k, d));
            }
        }
        best
    };
    let pitch_of = |e: &Event, k: usize| -> Option<(char, i32, i32)> {
        match &e.kind {
            Kind::Chord { pitches, .. } => pitches.get(k).map(|p| (p.step, p.octave, p.alter)),
            _ => None,
        }
    };
    let staff_index = |si: usize| -> usize {
        sheet.systems.iter().find_map(|y| y.parts.iter().find_map(|p| p.iter().position(|&s| s == si))).unwrap_or(0)
    };
    // Links: (left (chord, note) or None, right (chord, note) or None).
    let mut links: Vec<(Option<(usize, usize)>, Option<(usize, usize)>, (f64, f64), (f64, f64))> = Vec::new();
    for s in &sheet.slurs {
        let l = lookup(parts, s.left, true, s.above).map(|(c, k, _)| (c, k));
        let r = lookup(parts, s.right, false, s.above).map(|(c, k, _)| (c, k));
        if let (Some(a), Some(b)) = (l, r)
            && a.0 == b.0
        {
            continue;
        }
        links.push((l, r, s.left, s.right));
    }
    // Orphans near staff ends are halves of a slur or tie across systems.
    let staff_of = |pt: (f64, f64)| -> Option<usize> {
        (0..sheet.staves.len()).min_by(|&a, &b| {
            let d = |i: usize| ((sheet.staves[i].top(pt.0) + sheet.staves[i].bottom(pt.0)) / 2.0 - pt.1).abs();
            d(a).partial_cmp(&d(b)).unwrap()
        })
    };
    let mut pending: Vec<(usize, usize)> = Vec::new(); // left-linked halves at a system end
    let mut pairs: Vec<((usize, usize), (usize, usize))> = Vec::new();
    let mut order: Vec<usize> = (0..links.len()).collect();
    order.sort_by(|&a, &b| {
        let key = |i: usize| {
            let p = links[i].2;
            let sys = staff_of(p).and_then(|si| sheet.systems.iter().position(|y| y.staves.contains(&si))).unwrap_or(0);
            (sys, p.0 as i64)
        };
        key(a).cmp(&key(b))
    });
    for i in order {
        let (l, r, lp, rp) = links[i];
        match (l, r) {
            (Some(a), Some(b)) => pairs.push((a, b)),
            (Some(a), None) => {
                let si = chords[a.0].3;
                if sheet.staves[si].right as f64 - rp.0 <= 6.0 * il {
                    pending.push(a);
                }
            }
            (None, Some(b)) => {
                let si = chords[b.0].3;
                let start = sheet.staves[si].header_stop.max(sheet.staves[si].left) as f64;
                if lp.0 - start <= 6.0 * il
                    && let Some(k) = pending.iter().position(|a| {
                        chords[a.0].0 == chords[b.0].0 && staff_index(chords[a.0].3) == staff_index(si)
                    })
                {
                    pairs.push((pending.remove(k), b));
                }
            }
            _ => {}
        }
    }
    // Ties and slurs.
    let mut slur_list: Vec<(usize, usize)> = Vec::new();
    for (a, b) in pairs {
        let (ca, cb) = (chords[a.0], chords[b.0]);
        let (ea, eb) = (ev(parts, ca), ev(parts, cb));
        let same_pitch = pitch_of(&ea, a.1).is_some() && pitch_of(&ea, a.1) == pitch_of(&eb, b.1);
        let cross = ca.3 != cb.3;
        let same_staff = if cross { staff_index(ca.3) == staff_index(cb.3) } else { true };
        let ha = sheet.heads[ea.heads[a.1]].bounds;
        let hb = sheet.heads[eb.heads[b.1]].bounds;
        let clear = cross || {
            // No other chord invading the box between the two heads (beyond 25%).
            let bx = ha.union(&hb).union(&ea.bbox).union(&eb.bbox);
            let bx = Rect::new(ha.right(), bx.y, (hb.x - ha.right()).max(1), bx.h);
            cb.1 <= ca.1 + 1
                && !chords.iter().any(|&c| {
                    c != ca && c != cb && c.3 == ca.3 && {
                        let b = parts[c.0].measures[c.1].events[c.2].bbox;
                        b.intersection(&bx).is_some_and(|i| i.h as f64 > 0.25 * bx.h as f64)
                    }
                })
        };
        let dup = parts[ca.0].measures[ca.1].events[ca.2].ties.contains(&(a.1, true))
            || parts[cb.0].measures[cb.1].events[cb.2].ties.contains(&(b.1, false));
        if same_pitch && same_staff && clear && ca.0 == cb.0 {
            if dup {
                continue;
            }
            parts[ca.0].measures[ca.1].events[ca.2].ties.push((a.1, true));
            parts[cb.0].measures[cb.1].events[cb.2].ties.push((b.1, false));
        } else if ca.0 == cb.0 {
            slur_list.push((a.0, b.0));
        }
    }
    // Slur numbers: smallest free number per part.
    slur_list.sort_by_key(|&(a, _)| (chords[a].0, chords[a].1, parts[chords[a].0].measures[chords[a].1].events[chords[a].2].x as i64));
    let pos = |c: (usize, usize, usize, usize), parts: &[Part]| (c.0, c.1, parts[c.0].measures[c.1].events[c.2].x as i64);
    let mut active: Vec<((usize, usize, i64), u32)> = Vec::new();
    for (a, b) in slur_list {
        let (ca, cb) = (chords[a], chords[b]);
        let start = pos(ca, parts);
        active.retain(|(end, _)| *end >= start);
        let Some(n) = (1..=6).find(|n| !active.iter().any(|(_, m)| m == n)) else { continue };
        active.push((pos(cb, parts), n));
        parts[ca.0].measures[ca.1].events[ca.2].slurs.push((n, true));
        parts[cb.0].measures[cb.1].events[cb.2].slurs.push((n, false));
    }
}

/// Measure duration checks of a page (Audiveris MeasureFixer): a short first
/// measure of the page is a pickup, two short measures around a backward
/// repeat that make one full measure are halves, and an empty first measure
/// of a system is merged into the next one.
fn fix_measures(parts: &mut [Part]) {
    let n = parts.iter().map(|p| p.measures.len()).min().unwrap_or(0);
    // Duration missing (negative) or exceeding at the end of the measure,
    // when all voices agree; None for an empty measure (Some(None) when
    // voices disagree).
    let termination = |parts: &[Part], mi: usize| -> Option<Option<Q>> {
        let mut term: Option<Q> = None;
        let mut empty = true;
        for p in parts {
            let m = &p.measures[mi];
            if m.attributes.as_ref().is_some_and(|a| a.multiple_rest.is_some()) {
                return Some(Some(Q::ZERO));
            }
            let mut voices: Vec<u32> = m.events.iter().map(|e| e.voice).collect();
            voices.sort();
            voices.dedup();
            for v in voices {
                empty = false;
                let end = m.events.iter().filter(|e| e.voice == v).map(|e| e.start + e.duration).max().unwrap();
                let t = end - m.expected;
                if term.is_some_and(|x| x != t) {
                    return Some(None);
                }
                term = Some(t);
            }
        }
        (!empty).then_some(term)
    };
    let mut mi = 0;
    let mut prev: Option<Option<Q>> = None;
    let mut removed = 0;
    while mi < n - removed {
        let term = termination(parts, mi);
        let first_of_system = parts[0].measures[mi].new_system || mi == 0;
        let next_same_system = mi + 1 < n - removed && !parts[0].measures[mi + 1].new_system;
        if term.is_none() && first_of_system && next_same_system {
            // Empty first measure: merge into the next one.
            for p in parts.iter_mut() {
                let m = p.measures.remove(mi);
                let next = &mut p.measures[mi];
                next.new_system |= m.new_system;
                next.repeat.0 |= m.repeat.0;
                next.ending.0 = next.ending.0.or(m.ending.0);
                if next.attributes.is_none() {
                    next.attributes = m.attributes;
                } else if let (Some(a), Some(b)) = (next.attributes.as_mut(), m.attributes) {
                    a.fifths = a.fifths.or(b.fifths);
                    a.time = a.time.or(b.time);
                    if a.clefs.is_empty() {
                        a.clefs = b.clefs;
                    }
                }
            }
            removed += 1;
            continue;
        }
        let short = |t: Option<Option<Q>>| t.flatten().is_some_and(|t| t < Q::ZERO);
        if mi == 0 && short(term) {
            for p in parts.iter_mut() {
                p.measures[0].implicit = true;
            }
        } else if mi > 0 && short(prev) && short(term) && parts[0].measures[mi - 1].repeat.1 {
            let (a, b) = (prev.flatten().unwrap(), term.flatten().unwrap());
            if Q::ZERO - (a + b) == parts[0].measures[mi].expected {
                for p in parts.iter_mut() {
                    p.measures[mi].implicit = true;
                }
            }
        }
        prev = term;
        mi += 1;
    }
}

/// Endings of a system, per measure: (number started at the left bar,
/// (number, right leg) stopped at the right bar). The number text is not
/// read (no OCR): consecutive endings are numbered 1, 2, ...
fn system_endings(sheet: &Sheet, sys: &crate::grid::System) -> Vec<(Option<u32>, Option<(u32, bool)>)> {
    let il = sheet.scale.interline as f64;
    let n = sys.boundaries.len().saturating_sub(1);
    let mut out = vec![(None, None); n];
    let measure_at = |x: f64| sys.boundaries.windows(2).position(|w| x > w[0].x1 as f64 && x < w[1].x0 as f64);
    let mut list: Vec<&crate::curves::Ending> = sheet.endings.iter().filter(|e| sys.staves.contains(&e.staff)).collect();
    list.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap());
    let mut last: Option<(usize, u32)> = None;
    for e in list {
        let (Some(a), Some(b)) = (measure_at(e.x0 + il), measure_at(e.x1 - il)) else { continue };
        if out[a].0.is_some() || last.is_some_and(|(stop, _)| a <= stop) {
            continue;
        }
        let number = match last {
            Some((stop, k)) if stop + 1 == a => k + 1,
            _ => 1,
        };
        out[a].0 = Some(number);
        out[b].1 = Some((number, e.right_leg));
        last = Some((b, number));
    }
    out
}

/// Repeat signs (Audiveris DotFactory / RepeatDotInter): on each side of
/// every system barline, staves having a dot pair (pitch about -1 and +1)
/// within 1.5 il of the bar; the side repeats when most staves agree.
/// Returns (backward, forward) flags per boundary and the dots used.
fn repeat_signs(sheet: &Sheet, sys: &crate::grid::System) -> (Vec<(bool, bool)>, Vec<Rect>) {
    let il = sheet.scale.interline as f64;
    let mut flags = Vec::new();
    let mut used = Vec::new();
    for b in &sys.boundaries {
        let side = |left: bool| {
            let mut dots = Vec::new();
            let mut votes = 0;
            for &si in &sys.staves {
                let st = &sheet.staves[si];
                let near: Vec<&crate::symbols::Symbol> = sheet
                    .symbols
                    .iter()
                    .filter(|s| s.staff == si && s.shape == "DOT_set")
                    .filter(|s| {
                        let x = s.bounds.cx();
                        if left { x < b.x0 as f64 && b.x0 as f64 - x <= 1.5 * il } else { x > b.x1 as f64 && x - b.x1 as f64 <= 1.5 * il }
                    })
                    .filter(|s| (st.pitch_of(s.bounds.cy(), s.bounds.cx()).abs() - 1.0).abs() <= 1.0)
                    .collect();
                let up = near.iter().any(|s| st.pitch_of(s.bounds.cy(), s.bounds.cx()) < 0.0);
                let down = near.iter().any(|s| st.pitch_of(s.bounds.cy(), s.bounds.cx()) > 0.0);
                if up && down {
                    votes += 1;
                    dots.extend(near.iter().map(|s| s.bounds));
                }
            }
            (2 * votes > sys.staves.len()).then_some(dots)
        };
        let back = side(true);
        let fwd = side(false);
        flags.push((back.is_some(), fwd.is_some()));
        used.extend(back.into_iter().flatten());
        used.extend(fwd.into_iter().flatten());
    }
    (flags, used)
}

/// Multiple rest (Audiveris MultipleRestsBuilder): a beam-like bar of at
/// least 4 il stuck on the staff middle line (pitch within 0.2), with
/// vertical serifs at both ends; the measure count is the number above.
fn multiple_rest(sheet: &Sheet, staves: &[usize], x0: f64, x1: f64) -> Option<u32> {
    let il = sheet.scale.interline as f64;
    for &si in staves {
        let st = &sheet.staves[si];
        for b in &sheet.beams {
            if b.x0 < x0 || b.x1 > x1 || b.x1 - b.x0 < 4.0 * il {
                continue;
            }
            if [b.x0, b.x1].iter().any(|&x| st.pitch_of(b.y_at(x), x).abs() > 0.2) {
                continue;
            }
            let mid = b.y_at((b.x0 + b.x1) / 2.0);
            let serif = |x: f64| {
                (x as i32 - 2..=x as i32 + 2).any(|cx| {
                    let (a, e) = sheet.binary.vrun_at(cx, mid as i32);
                    (a as f64) <= mid - 0.75 * il && (e as f64) >= mid + 0.75 * il
                })
            };
            if !serif(b.x0) || !serif(b.x1) {
                continue;
            }
            let mut digits: Vec<(f64, u32)> = sheet
                .symbols
                .iter()
                .filter(|s| s.staff == si && s.bounds.cx() >= b.x0 && s.bounds.cx() <= b.x1 && s.bounds.cy() < st.top(s.bounds.cx()))
                .filter_map(|s| digit_of(s.shape).map(|d| (s.bounds.x as f64, d)))
                .collect();
            digits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let n = digits.iter().fold(0u32, |n, d| n * 10 + d.1);
            if n >= 2 {
                return Some(n);
            }
        }
    }
    None
}

/// Time signature of a column (Audiveris TimeColumn): every staff of the
/// system must show one; the most common value is taken.
fn column_time(times: impl Iterator<Item = Option<TimeSig>>) -> Option<TimeSig> {
    let times: Vec<TimeSig> = times.collect::<Option<Vec<_>>>()?;
    times
        .iter()
        .max_by_key(|t| times.iter().filter(|u| u == t).count())
        .copied()
}

/// Time signature change at the start of a measure, on all staves.
fn measure_time_change(sheet: &Sheet, staves: &[usize], x0: f64, x1: f64) -> Option<TimeSig> {
    column_time(staves.iter().map(|&si| staff_time(sheet, si, x0, x1)))
}

fn staff_time(sheet: &Sheet, si: usize, x0: f64, x1: f64) -> Option<TimeSig> {
    let syms: Vec<_> = sheet
        .symbols
        .iter()
        .filter(|s| s.staff == si && s.bounds.cx() > x0 && s.bounds.cx() < x1)
        .filter(|s| {
            let st = &sheet.staves[s.staff];
            s.bounds.cy() > st.top(s.bounds.cx()) && s.bounds.cy() < st.bottom(s.bounds.cx())
        })
        .collect();
    for s in &syms {
        if let Some(t) = time_of(s.shape) {
            return Some(t);
        }
    }
    let digits: Vec<_> = syms
        .iter()
        .filter_map(|s| digit_of(s.shape).map(|d| (d, s)))
        .collect();
    if digits.len() >= 2 {
        let (a, b) = (digits[0], digits[1]);
        let (num, den) = if a.1.bounds.cy() < b.1.bounds.cy() {
            (a.0, b.0)
        } else {
            (b.0, a.0)
        };
        if [2, 4, 8, 16].contains(&den) && num > 0 {
            return Some(TimeSig {
                beats: num,
                beat_type: den,
                symbol: None,
            });
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
    skip: &[Rect],
    m: &mut Measure,
) {
    let il = sheet.scale.interline as f64;
    let staff_no = |si: usize| {
        part_staves
            .iter()
            .position(|&s| s == si)
            .map(|p| p as u32 + 1)
    };
    let in_measure = |x: f64| x > x_lo && x < x_hi;
    let syms: Vec<&crate::symbols::Symbol> = sheet
        .symbols
        .iter()
        .filter(|s| part_staves.contains(&s.staff) && in_measure(s.bounds.cx()) && !skip.contains(&s.bounds))
        .collect();

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
        clef_changes
            .iter()
            .filter(|c| c.0 == n && c.1 < x)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|c| c.2)
            .unwrap_or(clefs[&n])
    };

    // Accidentals: (staff, pitch, x right edge, shape)
    // AlterInter.computePitch: mass center pitch; flats mix it (+0.65) with the
    // area center pitch shifted by the Bravura flat offset.
    let accidentals: Vec<(usize, i32, f64, &'static str)> = syms
        .iter()
        .filter(|s| alter_of(s.shape).is_some())
        .map(|s| {
            let st = &sheet.staves[s.staff];
            let mass = st.pitch_of(s.centroid.1, s.centroid.0);
            let p = if s.shape.contains("FLAT") {
                let geo = st.pitch_of(s.bounds.cy(), s.bounds.cx()) + FLAT_AREA_PITCH_OFFSET;
                0.5 * (mass + 0.65 + geo)
            } else {
                mass
            };
            (s.staff, p.round() as i32, s.bounds.right() as f64, s.shape)
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
        let Some(si) = staff_votes.map(|s| s as usize) else {
            continue;
        };
        let Some(n) = staff_no(si) else { continue };
        let heads: Vec<usize> = stem
            .heads
            .iter()
            .copied()
            .filter(|&h| sheet.heads[h].staff == si)
            .collect();
        for &h in &heads {
            used_heads[h] = true;
        }
        let shape = sheet.heads[heads[0]].shape;
        let flags: u32 = if stem.beams.is_empty() {
            let from_symbols: u32 = syms
                .iter()
                .filter(|s| s.shape.starts_with("FLAG_"))
                .filter(|s| {
                    let dx = s.bounds.x as f64 - stem.x as f64;
                    let mid = (stem.top + stem.bottom) as f64 / 2.0;
                    let tip_zone = if stem.up {
                        (s.bounds.y as f64) < mid && s.bounds.bottom() as f64 >= stem.top as f64
                    } else {
                        s.bounds.bottom() as f64 > mid && (s.bounds.y as f64) <= stem.bottom as f64
                    };
                    (-0.4 * il..=0.6 * il).contains(&dx) && tip_zone
                })
                .map(|s| flag_count(s.shape))
                .sum();
            if from_symbols > 0 {
                from_symbols
            } else {
                crate::symbols::stem_flags(&sheet.residual, stem, &sheet.scale)
            }
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
        let x = heads
            .iter()
            .map(|&h| sheet.heads[h].bounds.cx())
            .sum::<f64>()
            / heads.len() as f64;
        let right = heads
            .iter()
            .map(|&h| sheet.heads[h].bounds.right())
            .max()
            .unwrap() as f64;
        let y_mid = heads
            .iter()
            .map(|&h| sheet.heads[h].bounds.cy())
            .sum::<f64>()
            / heads.len() as f64;
        let mut hs = heads.clone();
        hs.sort_by_key(|&h| -sheet.heads[h].pitch);
        events.push(Event {
            staff: n,
            voice: 0,
            x,
            kind: Kind::Chord {
                pitches: Vec::new(),
                stem_up: Some(stem.up),
            },
            heads: hs,
            base,
            dots: 0,
            tuplet: None,
            start: Q::ZERO,
            duration: Q::ZERO,
            beams: Vec::new(),
            articulations: Vec::new(),
            fermata: None,
            ornaments: Vec::new(),
            arpeggiate: false,
            tremolo: 0,
            graces: Vec::new(),
            ties: Vec::new(),
            slurs: Vec::new(),
            bbox: Rect::default(),
            y_mid,
            stem: Some(sti),
            right,
        });
    }
    // Stem-less heads (whole notes, breves), grouped by abscissa.
    let mut loose: Vec<usize> = (0..sheet.heads.len())
        .filter(|&h| !used_heads[h] && sheet.heads[h].shape.stemless())
        .filter(|&h| {
            part_staves.contains(&sheet.heads[h].staff) && in_measure(sheet.heads[h].bounds.cx())
        })
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
        let base = if sheet.heads[g[0]].shape == HeadShape::Breve {
            Q::new(2, 1)
        } else {
            Q::new(1, 1)
        };
        let mut hs = g.clone();
        hs.sort_by_key(|&h| -sheet.heads[h].pitch);
        events.push(Event {
            staff: staff_no(si).unwrap(),
            voice: 0,
            x: g.iter().map(|&h| sheet.heads[h].bounds.cx()).sum::<f64>() / g.len() as f64,
            kind: Kind::Chord {
                pitches: Vec::new(),
                stem_up: None,
            },
            heads: hs,
            base,
            dots: 0,
            tuplet: None,
            start: Q::ZERO,
            duration: Q::ZERO,
            beams: Vec::new(),
            articulations: Vec::new(),
            fermata: None,
            ornaments: Vec::new(),
            arpeggiate: false,
            tremolo: 0,
            graces: Vec::new(),
            ties: Vec::new(),
            slurs: Vec::new(),
            bbox: Rect::default(),
            y_mid: g.iter().map(|&h| sheet.heads[h].bounds.cy()).sum::<f64>() / g.len() as f64,
            stem: None,
            right: g
                .iter()
                .map(|&h| sheet.heads[h].bounds.right())
                .max()
                .unwrap() as f64,
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
            if (top - st.pitch_y(-2.0, cx)).abs() <= (bot - st.pitch_y(0.0, cx)).abs() {
                Q::new(1, 1)
            } else {
                Q::new(1, 2)
            }
        } else if let Some(b) = rest_base(s.shape) {
            b
        } else {
            continue;
        };
        events.push(Event {
            staff: staff_no(s.staff).unwrap(),
            voice: 0,
            x: cx,
            kind: Kind::Rest {
                whole_measure: false,
            },
            heads: Vec::new(),
            base,
            dots: 0,
            tuplet: None,
            start: Q::ZERO,
            duration: Q::ZERO,
            beams: Vec::new(),
            articulations: Vec::new(),
            fermata: None,
            ornaments: Vec::new(),
            arpeggiate: false,
            tremolo: 0,
            graces: Vec::new(),
            ties: Vec::new(),
            slurs: Vec::new(),
            bbox: Rect::default(),
            y_mid: cy,
            stem: None,
            right: s.bounds.right() as f64,
        });
        events.last_mut().unwrap().bbox = s.bounds;
    }

    for e in events.iter_mut().filter(|e| !e.heads.is_empty()) {
        let mut b = e.heads.iter().map(|&h| sheet.heads[h].bounds).reduce(|a, b| a.union(&b)).unwrap();
        if let Some(st) = e.stem {
            let s = &sheet.stems[st];
            b = b.union(&Rect::new(s.x, s.top, 1, s.bottom - s.top + 1));
        }
        e.bbox = b;
    }

    // Augmentation dots: first dot column right of the chord / rest, second
    // dot column right of the first. For a chord, every head must find a dot
    // in its own space (the space above for a head on a line), otherwise the
    // dots are inconsistent and ignored.
    let dots: Vec<&crate::symbols::Symbol> = syms
        .iter()
        .copied()
        .filter(|s| s.shape == "DOT_set")
        .collect();
    let mut aug_dots: Vec<Rect> = Vec::new();
    for ev in events.iter_mut() {
        let st = &sheet.staves[part_staves[(ev.staff - 1) as usize]];
        let near = |d: &&&crate::symbols::Symbol| {
            let x = d.bounds.x as f64;
            x >= ev.right - 1.0 && x <= ev.right + 1.6 * il
        };
        let candidates: Vec<&&crate::symbols::Symbol> = dots.iter().filter(near).collect();
        let first_col: Vec<&&crate::symbols::Symbol> = match &ev.kind {
            Kind::Chord { .. } => {
                let mut slots: Vec<i32> = ev
                    .heads
                    .iter()
                    .map(|&h| sheet.heads[h].pitch)
                    .map(|p| if p % 2 == 0 { p - 1 } else { p })
                    .collect();
                slots.sort();
                slots.dedup();
                let found: Vec<&&crate::symbols::Symbol> = candidates
                    .iter()
                    .copied()
                    .filter(|d| {
                        let p = st.pitch_of(d.bounds.cy(), d.bounds.cx()).round() as i32;
                        slots.contains(&p) || slots.contains(&(p - 1)) || slots.contains(&(p + 1))
                    })
                    .collect();
                let covered = slots.iter().all(|&sl| {
                    found.iter().any(|d| {
                        (st.pitch_of(d.bounds.cy(), d.bounds.cx()) - sl as f64).abs() <= 1.0
                    })
                });
                if covered { found } else { Vec::new() }
            }
            Kind::Rest { .. } => candidates
                .into_iter()
                .filter(|d| (d.bounds.cy() - ev.y_mid).abs() <= 1.0 * il)
                .collect(),
        };
        let Some(first) = first_col.iter().map(|d| d.bounds.x as f64).reduce(f64::min) else {
            continue;
        };
        ev.dots = 1;
        aug_dots.extend(first_col.iter().map(|d| d.bounds));
        let seconds: Vec<Rect> = dots
            .iter()
            .filter(|d| {
                let x = d.bounds.x as f64;
                x > first + 0.3 * il
                    && x <= first + 1.6 * il
                    && first_col
                        .iter()
                        .any(|f| (f.bounds.cy() - d.bounds.cy()).abs() <= 0.3 * il)
            })
            .map(|d| d.bounds)
            .collect();
        if !seconds.is_empty() {
            ev.dots = 2;
            aug_dots.extend(seconds);
        }
    }

    attach_articulations(sheet, part_staves, &syms, &mut events, &aug_dots);
    attach_fermatas(sheet, part_staves, &syms, &mut events);
    attach_ornaments(sheet, &syms, &mut events);
    attach_arpeggios(sheet, &syms, &mut events);
    attach_pauses(part_staves, &syms, &mut events);
    attach_tremolos(sheet, &syms, &mut events);
    let small_heads: Vec<&crate::heads::GraceHead> = sheet
        .grace_heads
        .iter()
        .filter(|g| part_staves.contains(&g.head.staff) && in_measure(g.head.bounds.cx()))
        .collect();
    attach_graces(sheet, &syms, &small_heads, &mut events);

    // Tuplets attached to beam groups.
    for s in syms
        .iter()
        .filter(|s| s.shape == "TUPLET_THREE" || s.shape == "TUPLET_SIX")
    {
        let (cx, cy) = (s.bounds.cx(), s.bounds.cy());
        let beam = sheet
            .beams
            .iter()
            .enumerate()
            .filter(|(_, b)| {
                cx >= b.x0 - il
                    && cx <= b.x1 + il
                    && (b.y_at(cx.clamp(b.x0, b.x1)) - cy).abs() <= 2.5 * il
            })
            .min_by(|a, b| {
                (a.1.y_at(cx) - cy)
                    .abs()
                    .partial_cmp(&(b.1.y_at(cx) - cy).abs())
                    .unwrap()
            });
        let beamed = beam.is_some_and(|(bi, _)| {
            events.iter().any(|ev| {
                ev.stem
                    .is_some_and(|st| sheet.stems[st].beams.contains(&bi))
            })
        });
        if let Some((bi, _)) = beam.filter(|_| beamed) {
            for ev in events.iter_mut() {
                if let Some(st) = ev.stem
                    && sheet.stems[st].beams.contains(&bi)
                {
                    ev.tuplet = Some((3, 2));
                }
            }
        } else if let Some(n) = staff_no(s.staff) {
            // Bracketed (or plain) triplet over unbeamed notes: the three
            // chords of the staff centered below / above the number.
            let mut idx: Vec<usize> = (0..events.len())
                .filter(|&i| events[i].staff == n && matches!(events[i].kind, Kind::Chord { .. }))
                .collect();
            idx.sort_by(|&a, &b| events[a].x.partial_cmp(&events[b].x).unwrap());
            if let Some(c) = (0..idx.len()).min_by(|&a, &b| {
                (events[idx[a]].x - cx)
                    .abs()
                    .partial_cmp(&(events[idx[b]].x - cx).abs())
                    .unwrap()
            }) && c >= 1
                && c + 1 < idx.len()
                && (events[idx[c]].x - cx).abs() <= 1.5 * il
            {
                let group = [idx[c - 1], idx[c], idx[c + 1]];
                if group.iter().all(|&i| events[i].tuplet.is_none())
                    && events[group[2]].x - events[group[0]].x <= 12.0 * il
                {
                    for i in group {
                        events[i].tuplet = Some((3, 2));
                    }
                }
            }
        }
    }

    // Pitches.
    let mut measure_alters: HashMap<(u32, i32), i32> = HashMap::new();
    events.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    for ev in events.iter_mut() {
        let Kind::Chord { pitches, .. } = &mut ev.kind else {
            continue;
        };
        let mut out = Vec::new();
        let grace_heads: Vec<(usize, i32, f64)> = ev.graces.iter().flat_map(|g| g.heads.iter().copied()).collect();
        let grace_of: Vec<usize> = ev.graces.iter().enumerate().flat_map(|(k, g)| std::iter::repeat_n(k, g.heads.len())).collect();
        let chord_heads: Vec<(usize, i32, f64)> = ev
            .heads
            .iter()
            .map(|&h| (sheet.heads[h].staff, sheet.heads[h].pitch, sheet.heads[h].bounds.x as f64))
            .collect();
        for (k, &(head_staff, head_pitch, head_x)) in grace_heads.iter().chain(&chord_heads).enumerate() {
            let clef = clef_at(ev.staff, head_x, clefs);
            let idx = clef.middle_line() - head_pitch;
            let step = idx.rem_euclid(7) as usize;
            // Octave shift lines (8va above: sounding higher).
            let shift: i32 = sheet
                .octave_shifts
                .iter()
                .filter(|o| o.staff == head_staff && head_x >= o.x0 && head_x <= o.x1)
                .map(|o| (o.size as i32 - 1) / 7 * if o.above { 1 } else { -1 })
                .sum();
            let octave = idx.div_euclid(7) + shift;
            // Explicit accidental just left of the head.
            let acc = accidentals
                .iter()
                .filter(|a| a.0 == head_staff && a.1 == head_pitch)
                .filter(|a| a.2 <= head_x + 2.0 && a.2 >= head_x - 3.5 * il)
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            let alter = if let Some(a) = acc {
                let v = alter_of(a.3).unwrap();
                measure_alters.insert((ev.staff, idx), v);
                v
            } else {
                measure_alters
                    .get(&(ev.staff, idx))
                    .copied()
                    .unwrap_or_else(|| key_alter(fifths, step))
            };
            let p = Pitch {
                step: STEPS[step],
                octave,
                alter,
                accidental: acc.map(|a| accidental_name(a.3)),
            };
            if k < grace_heads.len() {
                ev.graces[grace_of[k]].pitches.push(p);
            } else {
                out.push(p);
            }
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
    let counts: Vec<usize> = events
        .iter()
        .map(|e| staff_counts(e.staff, &events))
        .collect();
    for (ev, c) in events.iter_mut().zip(counts) {
        if matches!(ev.kind, Kind::Rest { .. }) && ev.base == Q::new(1, 1) && c == 1 {
            ev.kind = Kind::Rest {
                whole_measure: true,
            };
            ev.duration = m.expected;
        }
    }

    assign_voices(&mut events, il);
    implicit_tuplets(sheet, &mut events, m.expected);
    assign_times(&mut events, il);
    assign_beams(sheet, &mut events, il);

    // Dynamics directions, attached to the next event of the staff.
    for s in syms.iter().filter(|s| s.shape.starts_with("DYNAMICS_")) {
        let n = staff_no(s.staff).unwrap();
        let next = events.iter().filter(|e| e.staff == n).min_by(|a, b| {
            (a.x - s.bounds.cx())
                .abs()
                .partial_cmp(&(b.x - s.bounds.cx()).abs())
                .unwrap()
        });
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
        m.directions.push(Direction {
            staff: n,
            start,
            kind: format!("<dynamics><{name}/></dynamics>"),
        });
    }
    // Markers (Audiveris MarkerInter): at the barline they overlap, else at
    // the closest event.
    for s in &syms {
        let kind = match s.shape {
            "SEGNO" => "<segno/>",
            "CODA" => "<coda/>",
            "DAL_SEGNO" => "<words>D.S.</words>",
            "DA_CAPO" => "<words>D.C.</words>",
            _ => continue,
        };
        let n = staff_no(s.staff).unwrap();
        let start = if (s.bounds.x as f64) <= x_lo {
            Q::ZERO
        } else if s.bounds.right() as f64 >= x_hi {
            events.iter().filter(|e| e.staff == n).map(|e| e.start).max().unwrap_or(Q::ZERO)
        } else {
            events
                .iter()
                .filter(|e| e.staff == n)
                .min_by(|a, b| (a.x - s.bounds.cx()).abs().partial_cmp(&(b.x - s.bounds.cx()).abs()).unwrap())
                .map_or(Q::ZERO, |e| e.start)
        };
        m.directions.push(Direction { staff: n, start, kind: kind.into() });
    }
    // Pedal marks (Audiveris PedalInter): the chord above overlapping the
    // sign (for a release, the box extends to the measure start).
    // Improvement: only below the last staff of a multi-staff part, since
    // without OCR, lyrics and words are often read as pedal marks.
    for s in syms.iter().filter(|s| s.shape == "PEDAL_MARK" || s.shape == "PEDAL_UP_MARK") {
        let (Some(&last), true) = (part_staves.last(), part_staves.len() > 1) else { continue };
        let st = &sheet.staves[last];
        if s.staff != last || s.bounds.cy() <= st.bottom(s.bounds.cx()) {
            continue;
        }
        let n = part_staves.len() as u32;
        let up = s.shape == "PEDAL_UP_MARK";
        let lo = if up { x_lo } else { s.bounds.x as f64 };
        let above = events
            .iter()
            .filter(|e| e.staff == n && matches!(e.kind, Kind::Chord { .. }))
            .filter(|e| e.bbox.right() as f64 > lo && (e.bbox.x as f64) < s.bounds.right() as f64)
            .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        if let Some(e) = above {
            let t = if up { "stop" } else { "start" };
            m.directions.push(Direction { staff: n, start: e.start, kind: format!("<pedal type=\"{t}\" line=\"no\"/>") });
        }
    }
    // Octave shifts: start at the first chord under the line, stop at the last.
    for o in sheet.octave_shifts.iter().filter(|o| part_staves.contains(&o.staff)) {
        let n = staff_no(o.staff).unwrap();
        let under: Vec<&Event> = events
            .iter()
            .filter(|e| e.staff == n && matches!(e.kind, Kind::Chord { .. }) && e.x >= o.x0 && e.x <= o.x1)
            .collect();
        let (Some(first), Some(last)) = (
            under.iter().min_by(|a, b| a.x.partial_cmp(&b.x).unwrap()),
            under.iter().max_by(|a, b| a.x.partial_cmp(&b.x).unwrap()),
        ) else {
            continue;
        };
        let size = o.size;
        if in_measure(o.x0) {
            let t = if o.above { "down" } else { "up" };
            m.directions.push(Direction { staff: n, start: first.start, kind: format!("<octave-shift type=\"{t}\" size=\"{size}\"/>") });
        }
        if in_measure(o.x1) {
            m.directions.push(Direction { staff: n, start: last.start, kind: format!("<octave-shift type=\"stop\" size=\"{size}\"/>") });
        }
    }
    // Hairpins: start at the left end, stop at the right end (Audiveris WedgeInter).
    for w in sheet.wedges.iter().filter(|w| part_staves.contains(&w.staff)) {
        let n = staff_no(w.staff).unwrap();
        let staff_events = || events.iter().filter(|e| e.staff == n);
        if in_measure(w.x0) {
            let start = staff_events()
                .min_by(|a, b| (a.x - w.x0).abs().partial_cmp(&(b.x - w.x0).abs()).unwrap())
                .map_or(Q::ZERO, |e| e.start);
            let t = if w.crescendo { "crescendo" } else { "diminuendo" };
            m.directions.push(Direction { staff: n, start, kind: format!("<wedge type=\"{t}\"/>") });
        }
        if in_measure(w.x1) {
            let start = staff_events()
                .filter(|e| e.x <= w.x1)
                .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
                .map_or(Q::ZERO, |e| e.start);
            m.directions.push(Direction { staff: n, start, kind: "<wedge type=\"stop\"/>".into() });
        }
    }
    for (n, x, c) in clef_changes {
        let t = events
            .iter()
            .filter(|e| e.staff == n && e.x > x)
            .map(|e| e.start)
            .min()
            .unwrap_or(Q::ZERO);
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
        let idx: Vec<usize> = (0..events.len())
            .filter(|&i| events[i].staff == n)
            .collect();
        let tol = 0.8 * il;
        let poly = idx.iter().any(|&i| {
            idx.iter()
                .any(|&j| j != i && (events[i].x - events[j].x).abs() < tol)
        });
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
                Kind::Chord {
                    stem_up: Some(up), ..
                } => !up,
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
/// Implicit tuplets (as Audiveris rhythm checks): when a voice overflows the
/// measure, beam groups of 3 (or 6) chords of equal value become triplets,
/// first the groups already marked nearby, then the others, until it fits.
fn implicit_tuplets(sheet: &Sheet, events: &mut [Event], expected: Q) {
    let mut voices: Vec<u32> = events.iter().map(|e| e.voice).collect();
    voices.sort();
    voices.dedup();
    for v in voices {
        let total = |evs: &[Event]| {
            evs.iter()
                .filter(|e| e.voice == v)
                .fold(Q::ZERO, |a, e| a + e.duration)
        };
        if total(events) <= expected {
            continue;
        }
        // Beam groups: events of this voice whose stems share a beam (transitively).
        let idx: Vec<usize> = (0..events.len())
            .filter(|&i| events[i].voice == v && events[i].stem.is_some())
            .collect();
        let mut group = vec![usize::MAX; events.len()];
        let mut next = 0;
        for (a, &i) in idx.iter().enumerate() {
            if group[i] == usize::MAX {
                group[i] = next;
                next += 1;
            }
            for &j in &idx[a + 1..] {
                let (bi, bj) = (
                    &sheet.stems[events[i].stem.unwrap()].beams,
                    &sheet.stems[events[j].stem.unwrap()].beams,
                );
                if bi.iter().any(|b| bj.contains(b)) {
                    let (gi, gj) = (group[i], group[j]);
                    if gj == usize::MAX {
                        group[j] = gi;
                    } else if gj != gi {
                        for g in group.iter_mut() {
                            if *g == gj {
                                *g = gi;
                            }
                        }
                    }
                }
            }
        }
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for g in 0..next {
            let mut members: Vec<usize> = idx.iter().copied().filter(|&i| group[i] == g).collect();
            members.sort_by(|&a, &b| events[a].x.partial_cmp(&events[b].x).unwrap());
            let n = members.len();
            if (n == 3 || n == 6)
                && events[members[0]].tuplet.is_none()
                && members.iter().all(|&i| events[i].dots == 0)
            {
                let sum = members.iter().fold(Q::ZERO, |a, &i| a + events[i].base);
                // Triplet of equal notes, or 3 units mixing e.g. quarter + eighth.
                let unit = members.iter().map(|&i| events[i].base).min().unwrap();
                if sum == unit * Q::new(n as i64, 1) || sum == unit * Q::new(3, 1) {
                    groups.push(members);
                }
            }
        }
        // Convert the first k groups only if that makes the voice fit exactly.
        let before = total(events);
        let mut acc = Q::ZERO;
        let mut k = 0;
        for g in &groups {
            acc = acc + g.iter().fold(Q::ZERO, |a, &i| a + events[i].duration) * Q::new(1, 3);
            k += 1;
            if before - acc <= expected {
                break;
            }
        }
        if k == 0 || before - acc != expected {
            continue;
        }
        for g in &groups[..k] {
            for &i in g {
                events[i].tuplet = Some((3, 2));
                events[i].duration = events[i].duration * Q::new(2, 3);
            }
        }
    }
}

fn assign_times(events: &mut [Event], il: f64) {
    let mut order: Vec<usize> = (0..events.len()).collect();
    order.sort_by(|&a, &b| events[a].x.partial_cmp(&events[b].x).unwrap());
    let mut slots: Vec<Vec<usize>> = Vec::new();
    for i in order {
        match slots.last_mut() {
            Some(s)
                if (events[i].x - events[s[0]].x).abs() <= 0.75 * il
                    && s.iter().all(|&j| events[j].voice != events[i].voice) =>
            {
                s.push(i)
            }
            _ => slots.push(vec![i]),
        }
    }
    let mut voice_time: HashMap<u32, Q> = HashMap::new();
    let mut last = Q::ZERO;
    for s in slots {
        let mut t = s
            .iter()
            .map(|&i| voice_time.get(&events[i].voice).copied().unwrap_or(Q::ZERO))
            .max()
            .unwrap();
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
            let mut xs: Vec<i32> = sheet
                .stems
                .iter()
                .filter(|s| s.beams.contains(&bi))
                .map(|s| s.x)
                .collect();
            xs.sort();
            let value = if xs.len() == 1 {
                let b = &sheet.beams[bi];
                if b.x1 - stem.x as f64 > 0.5 * il {
                    "forward hook"
                } else {
                    "backward hook"
                }
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

/// Grade of a connection from its gaps (Audiveris AbstractConnection with
/// GradeImpacts: weighted geometric mean of impacts, times intrinsic ratio 0.8).
pub fn connection_grade(dx: f64, dy: f64, x_max: f64, y_max: f64, wx: f64, wy: f64) -> f64 {
    let xi = ((x_max - dx.abs()) / x_max).max(0.0);
    let yi = ((y_max - dy.abs()) / y_max).max(0.0);
    if xi == 0.0 || yi == 0.0 {
        return 0.0;
    }
    0.8 * (xi.powf(wx) * yi.powf(wy)).powf(1.0 / (wx + wy))
}

/// Articulations (Audiveris ArticulationInter.lookupLink and
/// ChordArticulationRelation): the sign links to the head chord whose
/// top / bottom is vertically closest, within a lookup box of +/-0.8
/// interline around its center, up to 3 interlines away (at least 0.1).
/// Staccato signs come from dots that are neither augmentation dots nor
/// too close to a staff line (DotFactory, minDyFromLine 0.3).
fn attach_articulations(
    sheet: &Sheet,
    part_staves: &[usize],
    syms: &[&crate::symbols::Symbol],
    events: &mut [Event],
    aug_dots: &[Rect],
) {
    let il = sheet.scale.interline as f64;
    let (x_max, y_max, y_min) = (0.8 * il, 3.0 * il, 0.1 * il);
    let boxes: Vec<Option<Rect>> =
        events.iter().map(|e| matches!(e.kind, Kind::Chord { .. }).then_some(e.bbox)).collect();
    for s in syms {
        let (name, above_only, below_only) = match s.shape {
            "ACCENT" => ("accent", false, false),
            "TENUTO" => ("tenuto", false, false),
            "STACCATISSIMO" => ("staccatissimo", true, false),
            "STACCATISSIMO_BELOW" => ("staccatissimo", false, true),
            "MARCATO" => ("strong-accent", true, false),
            "MARCATO_BELOW" => ("strong-accent", false, true),
            "DOT_set" => ("staccato", false, false),
            _ => continue,
        };
        let st = &sheet.staves[s.staff];
        let (px, py) = (s.bounds.cx(), s.bounds.cy());
        if name == "staccato" {
            if aug_dots.contains(&s.bounds) {
                continue;
            }
            let pitch = st.pitch_of(py, px);
            if pitch.abs() <= 4.0 && (pitch - 2.0 * (pitch / 2.0).round()).abs() / 2.0 < 0.3 {
                continue;
            }
        }
        // Vertical lookup range (staff relative position of the sign).
        let k = part_staves.iter().position(|&x| x == s.staff).unwrap_or(0);
        let (top, bot) = (st.top(px), st.bottom(px));
        let (min_y, max_y) = if py < top {
            if below_only {
                continue;
            }
            let lower = if above_only { py } else { py - y_max };
            // Between two staves of the part: the upper staff counts too.
            let upper_bottom = k.checked_sub(1).map(|j| sheet.staves[part_staves[j]].top(px));
            (upper_bottom.map_or(lower, |t| t.min(lower)), bot)
        } else if py > bot {
            if above_only {
                continue;
            }
            let upper = if below_only { py } else { py + y_max };
            let next_bottom = part_staves.get(k + 1).map(|&j| sheet.staves[j].bottom(px));
            (top, next_bottom.map_or(upper, |b| b.max(upper)))
        } else {
            (if above_only { py } else { top }, if below_only { py } else { bot })
        };
        let mut best: Option<(usize, f64)> = None;
        for (i, b) in boxes.iter().enumerate() {
            let Some(b) = b else { continue };
            if b.right() as f64 <= px - x_max || b.x as f64 > px + x_max || b.bottom() as f64 <= min_y || b.y as f64 > max_y {
                continue;
            }
            if b.intersects(&s.bounds) {
                continue;
            }
            let center = (b.cx(), b.cy());
            let y_ref = if py > center.1 { b.bottom() as f64 } else { b.y as f64 };
            let y_gap = if py > center.1 { py - y_ref } else { y_ref - py };
            if y_gap < y_min {
                continue;
            }
            let g = connection_grade(center.0 - px, y_gap, x_max, y_max, 3.0, 1.0);
            if g >= 0.1 && best.is_none_or(|(_, bg)| y_gap.abs() < bg) {
                best = Some((i, y_gap.abs()));
            }
        }
        if let Some((i, _)) = best
            && !events[i].articulations.contains(&name)
        {
            events[i].articulations.push(name);
        }
    }
}

/// Fermatas (Audiveris FermataInter.lookupChordLink): the closest standard
/// chord (or rest) below a FERMATA (above a FERMATA_BELOW) overlapping it
/// horizontally, within 2.5 interlines of the chord or of its staff.
fn attach_fermatas(sheet: &Sheet, part_staves: &[usize], syms: &[&crate::symbols::Symbol], events: &mut [Event]) {
    let il = sheet.scale.interline as f64;
    let max_dy = 2.5 * il;
    for s in syms.iter().filter(|s| s.shape == "FERMATA" || s.shape == "FERMATA_BELOW") {
        let below = s.shape == "FERMATA_BELOW";
        let (cx, cy) = (s.bounds.cx(), s.bounds.cy());
        let dist = |b: &Rect| {
            let dx = (b.x as f64 - cx).max(cx - b.right() as f64).max(0.0);
            let dy = (b.y as f64 - cy).max(cy - b.bottom() as f64).max(0.0);
            dx.hypot(dy)
        };
        let best = (0..events.len())
            .filter(|&i| {
                let b = events[i].bbox;
                b.x < s.bounds.right() && s.bounds.x < b.right() && if below { b.cy() < cy } else { b.cy() > cy }
            })
            .min_by(|&a, &b| dist(&events[a].bbox).partial_cmp(&dist(&events[b].bbox)).unwrap());
        let Some(i) = best else { continue };
        if dist(&events[i].bbox) > max_dy {
            let st = &sheet.staves[part_staves[(events[i].staff - 1) as usize]];
            let d = if below { cy - st.bottom(cx) } else { st.top(cx) - cy };
            if d > max_dy {
                continue;
            }
        }
        events[i].fermata = Some(if below { "inverted" } else { "upright" });
    }
}

/// Ornaments (Audiveris OrnamentInter.lookupLink): the head chord whose
/// reference side (top or bottom) is vertically closest to the ornament
/// center, within 0.75 il horizontally and 3 il vertically.
fn attach_ornaments(sheet: &Sheet, syms: &[&crate::symbols::Symbol], events: &mut [Event]) {
    let il = sheet.scale.interline as f64;
    let (x_max, y_max) = (0.75 * il, 3.0 * il);
    for s in syms {
        // MusicXML names per Audiveris MusicXML.java (MORDENT is inverted-mordent).
        let name = match s.shape {
            "TR" => "trill-mark",
            "TURN" => "turn",
            "TURN_INVERTED" => "inverted-turn",
            "TURN_UP" => "vertical-turn",
            "TURN_SLASH" => "turn slash=\"yes\"",
            "MORDENT" => "inverted-mordent",
            "MORDENT_INVERTED" => "mordent",
            _ => continue,
        };
        let (px, py) = (s.bounds.cx(), s.bounds.cy());
        let mut best: Option<(usize, f64)> = None;
        for (i, e) in events.iter().enumerate() {
            let b = e.bbox;
            if !matches!(e.kind, Kind::Chord { .. }) || b.intersects(&s.bounds) {
                continue;
            }
            if b.right() as f64 <= px - x_max || b.x as f64 > px + x_max || b.bottom() as f64 <= py - y_max || b.y as f64 > py + y_max {
                continue;
            }
            let y_ref = if py > b.cy() { b.bottom() as f64 } else { b.y as f64 };
            let y_gap = (y_ref - py).abs();
            let g = connection_grade((b.cx() - px).abs(), y_gap, x_max, y_max, 2.0, 1.0);
            if g >= 0.1 && best.is_none_or(|(_, bg)| y_gap < bg) {
                best = Some((i, y_gap));
            }
        }
        if let Some((i, _)) = best {
            events[i].ornaments.push(name);
        }
    }
}

/// Arpeggios (Audiveris ArpeggiatoInter.lookupLink): the head chord on the
/// right, within 1.5 il, whose heads overlap the sign most vertically
/// (overlap ratio at least 0.1).
fn attach_arpeggios(sheet: &Sheet, syms: &[&crate::symbols::Symbol], events: &mut [Event]) {
    let il = sheet.scale.interline;
    for s in syms.iter().filter(|s| s.shape == "ARPEGGIATO") {
        let lu = Rect::new(s.bounds.right(), s.bounds.y, (1.5 * il as f64).round() as i32, s.bounds.h);
        let mut best: Option<(usize, i32)> = None;
        for (i, e) in events.iter().enumerate() {
            let Some(hb) = e.heads.iter().map(|&h| sheet.heads[h].bounds).reduce(|a, b| a.union(&b)) else { continue };
            if !hb.intersects(&lu) {
                continue;
            }
            let overlap = hb.bottom().min(lu.bottom()) - hb.y.max(lu.y);
            if best.is_none_or(|(_, o)| overlap > o) {
                best = Some((i, overlap));
            }
        }
        if let Some((i, o)) = best
            && o as f64 / lu.h as f64 >= 0.1
        {
            events[i].arpeggiate = true;
        }
    }
}

/// Breath marks and caesuras (Audiveris AbstractPauseInter.lookupLink): the
/// right-most head chord of the same staff and measure, left of the sign.
fn attach_pauses(part_staves: &[usize], syms: &[&crate::symbols::Symbol], events: &mut [Event]) {
    for s in syms {
        let name = match s.shape {
            "BREATH_MARK" => "breath-mark",
            "CAESURA" => "caesura",
            _ => continue,
        };
        let x_max = s.bounds.right() as f64;
        let best = (0..events.len())
            .filter(|&i| {
                let e = &events[i];
                matches!(e.kind, Kind::Chord { .. })
                    && part_staves.get((e.staff - 1) as usize) == Some(&s.staff)
                    && e.bbox.cx() <= x_max
            })
            .max_by(|&a, &b| events[a].bbox.cx().partial_cmp(&events[b].bbox.cx()).unwrap());
        if let Some(i) = best {
            events[i].articulations.push(name);
        }
    }
}

/// Single tremolos (Audiveris TremoloInter.searchLinks): the stem crossing
/// the sign center (within 0.3 il), else a stemless head chord within
/// 0.3 il horizontally and 2 il vertically.
fn attach_tremolos(sheet: &Sheet, syms: &[&crate::symbols::Symbol], events: &mut [Event]) {
    let il = sheet.scale.interline as f64;
    for s in syms {
        let n = match s.shape {
            "TREMOLO_1" => 1,
            "TREMOLO_2" => 2,
            "TREMOLO_3" => 3,
            _ => continue,
        };
        let (px, py) = (s.bounds.cx(), s.bounds.cy());
        let on_stem = events.iter().position(|e| {
            e.stem.is_some_and(|st| {
                let st = &sheet.stems[st];
                (st.x as f64 - px).abs() <= 0.3 * il && py >= st.top as f64 && py <= st.bottom as f64
            })
        });
        let whole = || {
            events.iter().position(|e| {
                let b = e.bbox;
                let dy = (b.y as f64 - py).max(py - b.bottom() as f64).max(0.0);
                matches!(e.kind, Kind::Chord { .. }) && e.stem.is_none() && (b.cx() - px).abs() <= 0.3 * il && dy <= 2.0 * il
            })
        };
        if let Some(i) = on_stem.or_else(whole) {
            events[i].tremolo = n;
        }
    }
}

/// Grace notes (Audiveris GraceChordInter.createValidAdded and
/// SmallChordInter.lookupLink): the small head lies at the bottom-left (stem
/// up) or top-right (stem down) corner of the glyph; it links to the head
/// chord on its right (within 2.5 il, 1.5 il vertically) whose leading head
/// is closest.
fn attach_graces(
    sheet: &Sheet,
    syms: &[&crate::symbols::Symbol],
    small_heads: &[&crate::heads::GraceHead],
    events: &mut [Event],
) {
    let il = sheet.scale.interline as f64;
    // Small head: 0.67 of the typical standard black head.
    let blacks: Vec<&Rect> = sheet.heads.iter().filter(|h| !h.shape.stemless()).map(|h| &h.bounds).collect();
    let (hw, hh) = if blacks.is_empty() {
        (1.2 * il, il)
    } else {
        let n = blacks.len() as f64;
        (blacks.iter().map(|b| b.w as f64).sum::<f64>() / n, blacks.iter().map(|b| b.h as f64).sum::<f64>() / n)
    };
    let (sw, sh) = (0.67 * hw, 0.67 * hh);
    let x_max = 2.5 * il;
    // (glyph bounds, head center, staff, stem up, slash, type)
    let mut graces: Vec<(Rect, (f64, f64), usize, bool, bool, Q)> = Vec::new();
    for s in syms {
        let (up, slash) = match s.shape {
            "GRACE_NOTE" => (true, false),
            "GRACE_NOTE_DOWN" => (false, false),
            "GRACE_NOTE_SLASH" => (true, true),
            "GRACE_NOTE_SLASH_DOWN" => (false, true),
            _ => continue,
        };
        let b = s.bounds;
        let c = if up {
            (b.x as f64 + sw / 2.0, b.bottom() as f64 - sh / 2.0)
        } else {
            (b.right() as f64 - sw / 2.0, b.y as f64 + sh / 2.0)
        };
        graces.push((b, c, s.staff, up, slash, Q::new(1, 8)));
    }
    // Small heads with a stem (improvement: Audiveris needs a whole-glyph
    // grace shape, absent for 16th / beamed graces and up appoggiaturas).
    for g in small_heads {
        let (h, up) = (&g.head, g.up);
        let c = (h.bounds.cx(), h.bounds.cy());
        if graces.iter().any(|g| g.0.intersects(&h.bounds))
            || syms.iter().any(|s| alter_of(s.shape).is_some() && s.bounds.intersects(&h.bounds))
            || sheet.stems.iter().flat_map(|st| &st.beams).map(|&bi| &sheet.beams[bi]).any(|b| {
                let x = h.bounds.cx().clamp(b.x0, b.x1);
                (x - h.bounds.cx()).abs() <= il / 2.0 && (b.y_at(x) - h.bounds.cy()).abs() <= b.thickness / 2.0 + il / 4.0
            })
        {
            continue;
        }
        // Type from the flags or cue beams (Audiveris CUE_BEAMS for grace chords).
        let base = Q::new(1, 8 << g.strokes.clamp(1, 3).saturating_sub(1));
        graces.push((h.bounds, c, h.staff, up, false, base));
    }
    // Right to left, so that a grace may also lead to the grace on its right
    // (grace groups); the vertical gap is measured to the closest head, up
    // to 3 il (improvement over the leading-head gap of 1.5 il).
    let y_max = 3.0 * il;
    graces.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap());
    let mut linked: Vec<((f64, f64), usize)> = Vec::new();
    for (b, (hx, hy), staff, up, slash, base) in graces {
        let mut best: Option<(usize, f64)> = None;
        let mut consider = |i: usize, (cx, cy): (f64, f64)| {
            let (dx, dy) = (cx - hx, cy - hy);
            if dx <= 0.0 || connection_grade(dx, dy.abs(), x_max, y_max, 2.0, 1.0) < 0.1 {
                return;
            }
            let d = dx.hypot(dy);
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        };
        for (i, e) in events.iter().enumerate() {
            if !matches!(e.kind, Kind::Chord { .. }) || e.bbox.intersects(&b) {
                continue;
            }
            for &h in &e.heads {
                consider(i, (sheet.heads[h].bounds.cx(), sheet.heads[h].bounds.cy()));
            }
        }
        for &(c, i) in &linked {
            consider(i, c);
        }
        if let Some((i, _)) = best {
            let pitch = sheet.staves[staff].pitch_of(hy, hx).round() as i32;
            let head = (staff, pitch, hx - sw / 2.0);
            // Heads on the same stem make one grace chord.
            match events[i].graces.iter_mut().find(|g| g.heads.iter().any(|h| (h.2 - head.2).abs() <= 0.3 * il)) {
                Some(g) => {
                    g.heads.push(head);
                    g.slash |= slash;
                    g.base = g.base.min(base);
                }
                None => events[i].graces.push(Grace { pitches: Vec::new(), slash, stem_up: up, base, heads: vec![head] }),
            }
            linked.push(((hx, hy), i));
        }
    }
    for e in events.iter_mut() {
        e.graces.sort_by(|a, b| a.heads[0].2.partial_cmp(&b.heads[0].2).unwrap());
        for g in e.graces.iter_mut() {
            g.heads.sort_by_key(|h| -h.1);
        }
    }
}
