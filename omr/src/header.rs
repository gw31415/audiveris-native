//! Staff headers (Audiveris HEADERS step): clef, key signature and time
//! signature at the beginning of each staff, recognized by the glyph
//! classifier on the staff-free image.

use crate::binary::{Bitmap, Component, Rect, components};
use crate::classifier::classify;
use crate::scale::Scale;
use crate::staff::Staff;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ClefKind {
    G,
    G8va,
    G8vb,
    F,
    F8va,
    F8vb,
    /// C clef with the staff line number (1 = bottom) it is centered on.
    C(i32),
    Percussion,
}

impl ClefKind {
    pub fn from_shape(shape: &str, pitch_center: f64) -> Option<Self> {
        Some(match shape {
            "G_CLEF" | "G_CLEF_SMALL" => ClefKind::G,
            "G_CLEF_8VA" => ClefKind::G8va,
            "G_CLEF_8VB" => ClefKind::G8vb,
            "F_CLEF" | "F_CLEF_SMALL" => ClefKind::F,
            "F_CLEF_8VA" => ClefKind::F8va,
            "F_CLEF_8VB" => ClefKind::F8vb,
            "C_CLEF" => {
                // pitch 0 = middle line (3rd line), -2 = 4th line, +2 = 2nd line
                let line = 3 - (pitch_center / 2.0).round() as i32;
                ClefKind::C(line.clamp(1, 5))
            }
            "PERCUSSION_CLEF" => ClefKind::Percussion,
            _ => return None,
        })
    }

    /// Diatonic index (octave*7 + step, C=0) of the note on the middle line (pitch 0).
    pub fn middle_line(self) -> i32 {
        let idx = |step: i32, octave: i32| octave * 7 + step;
        match self {
            ClefKind::G | ClefKind::Percussion => idx(6, 4), // B4
            ClefKind::G8va => idx(6, 5),
            ClefKind::G8vb => idx(6, 3),
            ClefKind::F => idx(1, 3), // D3
            ClefKind::F8va => idx(1, 4),
            ClefKind::F8vb => idx(1, 2),
            // C4 sits on line `l`; middle line is line 3.
            ClefKind::C(l) => idx(0, 4) + (3 - l) * 2,
        }
    }

    pub fn musicxml(self) -> (&'static str, i32, i32) {
        match self {
            ClefKind::G => ("G", 2, 0),
            ClefKind::G8va => ("G", 2, 1),
            ClefKind::G8vb => ("G", 2, -1),
            ClefKind::F => ("F", 4, 0),
            ClefKind::F8va => ("F", 4, 1),
            ClefKind::F8vb => ("F", 4, -1),
            ClefKind::C(l) => ("C", l, 0),
            ClefKind::Percussion => ("percussion", 3, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct TimeSig {
    pub beats: u32,
    pub beat_type: u32,
    pub symbol: Option<&'static str>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Header {
    pub clef: Option<ClefKind>,
    pub fifths: i32,
    pub time: Option<TimeSig>,
    pub stop: i32,
    /// Bounds of recognized header items (for debugging / erasure).
    pub items: Vec<(Rect, String)>,
}

/// Group components into columns of horizontally overlapping parts.
pub fn columns(mut comps: Vec<Component>) -> Vec<Component> {
    comps.sort_by_key(|c| c.bounds.x);
    let mut cols: Vec<Component> = Vec::new();
    for c in comps {
        if let Some(last) = cols.last_mut() {
            let ov = last.bounds.right().min(c.bounds.right()) - last.bounds.x.max(c.bounds.x);
            if ov as f64 >= 0.3 * c.bounds.w.min(last.bounds.w) as f64 {
                *last = Component::merge(&[last, &c]);
                continue;
            }
        }
        cols.push(c);
    }
    cols
}

pub fn time_of(shape: &str) -> Option<TimeSig> {
    let t = |b, bt| Some(TimeSig { beats: b, beat_type: bt, symbol: None });
    match shape {
        "COMMON_TIME" => Some(TimeSig { beats: 4, beat_type: 4, symbol: Some("common") }),
        "CUT_TIME" => Some(TimeSig { beats: 2, beat_type: 2, symbol: Some("cut") }),
        "TIME_FOUR_FOUR" => t(4, 4),
        "TIME_TWO_TWO" => t(2, 2),
        "TIME_TWO_FOUR" => t(2, 4),
        "TIME_THREE_FOUR" => t(3, 4),
        "TIME_FIVE_FOUR" => t(5, 4),
        "TIME_SIX_FOUR" => t(6, 4),
        "TIME_THREE_EIGHT" => t(3, 8),
        "TIME_SIX_EIGHT" => t(6, 8),
        "TIME_TWELVE_EIGHT" => t(12, 8),
        _ => None,
    }
}

pub fn digit_of(shape: &str) -> Option<u32> {
    Some(match shape {
        "TIME_ZERO" => 0,
        "TIME_ONE" => 1,
        "TIME_TWO" => 2,
        "TIME_THREE" => 3,
        "TIME_FOUR" => 4,
        "TIME_FIVE" => 5,
        "TIME_SIX" => 6,
        "TIME_SEVEN" => 7,
        "TIME_EIGHT" => 8,
        "TIME_NINE" => 9,
        "TIME_TWELVE" => 12,
        "TIME_SIXTEEN" => 16,
        _ => return None,
    })
}

fn best_among<'a>(ev: &[(&'a str, f64)], pred: impl Fn(&str) -> bool) -> Option<(&'a str, f64)> {
    ev.iter().copied().find(|(s, _)| pred(s))
}

/// Recognize a time signature from a column (whole pair, or numerator over denominator).
pub fn time_from_column(col: &Component, parts: &[Component], staff: &Staff, il: i32) -> Option<TimeSig> {
    let ev = classify(col, il);
    if let Some((s, g)) = ev.first()
        && *g >= 0.1
        && let Some(t) = time_of(s)
    {
        return Some(t);
    }
    // Split into upper and lower halves around the middle line.
    let mid = staff.pitch_y(0.0, col.bounds.cx());
    let upper: Vec<&Component> = parts.iter().filter(|p| p.bounds.cy() < mid).collect();
    let lower: Vec<&Component> = parts.iter().filter(|p| p.bounds.cy() >= mid).collect();
    if upper.is_empty() || lower.is_empty() {
        return None;
    }
    let num = |v: &[&Component]| -> Option<u32> {
        let g = Component::merge(v);
        let ev = classify(&g, il);
        best_among(&ev, |s| digit_of(s).is_some()).filter(|e| e.1 >= 0.1).and_then(|e| digit_of(e.0))
    };
    let (n, d) = (num(&upper)?, num(&lower)?);
    (n > 0 && [1, 2, 4, 8, 16, 32].contains(&d)).then_some(TimeSig { beats: n, beat_type: d, symbol: None })
}

pub fn recognize(nostaff: &Bitmap, staff: &Staff, scale: &Scale) -> Header {
    let il = scale.interline;
    let x0 = staff.left;
    let top = staff.top(x0 as f64).round() as i32;
    let bot = staff.bottom(x0 as f64).round() as i32;
    let roi = Rect::new(x0, top - 2 * il, 16 * il, bot - top + 4 * il);
    let comps: Vec<Component> = components(nostaff, Some(roi))
        .into_iter()
        .filter(|c| c.weight() as f64 >= 0.05 * (il * il) as f64)
        .filter(|c| {
            // Must intersect the staff height band.
            c.bounds.bottom() > top - il / 2 && c.bounds.y < bot + il / 2
        })
        .collect();
    let mut header = Header { clef: None, fifths: 0, time: None, stop: x0, items: Vec::new() };
    let cols_parts: Vec<(Component, Vec<Component>)> = {
        let mut sorted = comps.clone();
        sorted.sort_by_key(|c| c.bounds.x);
        let cols = columns(sorted.clone());
        cols.into_iter()
            .map(|col| {
                let parts =
                    sorted.iter().filter(|c| col.bounds.intersects(&c.bounds) || col.bounds.union(&c.bounds) == col.bounds).cloned().collect();
                (col, parts)
            })
            .collect()
    };
    let mut i = 0;
    let max_gap = 3 * il;
    // Clef: the best clef shape among the first 1..4 parts merged.
    {
        let mut best: Option<(&str, f64, usize, Component)> = None;
        for k in 1..=4.min(cols_parts.len()) {
            let merged = Component::merge(&cols_parts[..k].iter().map(|c| &c.0).collect::<Vec<_>>());
            if merged.bounds.x - x0 > 2 * il || merged.bounds.w > 4 * il {
                break;
            }
            if merged.bounds.h < 2 * il {
                continue;
            }
            let ev = classify(&merged, il);
            if let Some((s, g)) = best_among(&ev, |s| ClefKind::from_shape(s, 0.0).is_some())
                && g >= 0.03
                && best.as_ref().is_none_or(|b| g > b.1)
            {
                best = Some((s, g, k, merged));
            }
        }
        if let Some((s, _, k, col)) = best {
            let pc = staff.pitch_of(col.bounds.cy(), col.bounds.cx());
            header.clef = ClefKind::from_shape(s, pc);
            header.stop = col.bounds.right();
            header.items.push((col.bounds, s.to_string()));
            i = k;
        }
    }
    // Key signature: consecutive sharps or flats.
    let mut key_kind: Option<&str> = None;
    let mut count = 0;
    while i < cols_parts.len() {
        let col = &cols_parts[i].0;
        let gap = if count == 0 { 2 * il } else { il };
        if col.bounds.x - header.stop > gap || col.bounds.h > 4 * il {
            if std::env::var("OMR_DEBUG").is_ok() { eprintln!("  stop at {:?}", col.bounds); }
            break;
        }
        let is_alter = |ev: &[(&'static str, f64)]| {
            ev.first().copied().filter(|(s, g)| (*s == "SHARP" || *s == "FLAT") && *g >= 0.1 && key_kind.is_none_or(|k| k == *s))
        };
        let mut ev = classify(col, il);
        if std::env::var("OMR_DEBUG").is_ok() { eprintln!("  col {:?} {:?}", col.bounds, &ev[..3]); }
        let mut used = 1;
        if is_alter(&ev).is_none()
            && let Some((c2, _)) = cols_parts.get(i + 1)
            && c2.bounds.x - col.bounds.right() <= il / 3
        {
            // Try merging with the next part (alteration broken by line removal).
            let merged = Component::merge(&[col, c2]);
            let ev2 = classify(&merged, il);
            if is_alter(&ev2).is_some() {
                ev = ev2;
                used = 2;
            }
        }
        let Some((s, _)) = is_alter(&ev) else { break };
        let col = &Component::merge(&cols_parts[i..i + used].iter().map(|c| &c.0).collect::<Vec<_>>());
        key_kind = Some(s);
        count += 1;
        header.stop = col.bounds.right();
        header.items.push((col.bounds, s.to_string()));
        i += used;
    }
    header.fifths = match key_kind {
        Some("SHARP") => count.min(7),
        Some("FLAT") => -count.min(7),
        _ => 0,
    };
    // Time signature
    if i < cols_parts.len() {
        let (col, parts) = &cols_parts[i];
        if col.bounds.x - header.stop <= max_gap
            && let Some(t) = time_from_column(col, parts, staff, il)
        {
            header.time = Some(t);
            header.stop = col.bounds.right();
            header.items.push((col.bounds, format!("TIME {}/{}", t.beats, t.beat_type)));
        }
    }
    header.stop += il / 3;
    header
}
