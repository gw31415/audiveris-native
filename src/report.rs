//! Recognition report (`--report FILE`): what was recognized on each page,
//! decisions taken without evidence (assumed or corrected time signatures),
//! and the measures whose contents do not add up to the time signature, so
//! that the MusicXML can be used as reference material with known weak spots.

use std::fmt::Write;

use crate::musicxml::measure_labels;
use crate::rational::Q;
use crate::score::{Measure, Score};

/// Below this interline (pixels between staff lines) Audiveris refuses a
/// sheet as too low in resolution (ScaleBuilder minInterline).
const MIN_INTERLINE: i32 = 11;

pub struct Page {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub interline: i32,
    pub systems: usize,
    pub notes: Vec<String>,
}

fn quarters(q: Q) -> String {
    let q = q * Q::new(4, 1);
    if q.d == 1 { format!("{}", q.n) } else { format!("{}/{}", q.n, q.d) }
}

/// End of each voice in a measure: (voice, end).
fn voice_ends(m: &Measure) -> Vec<(u32, Q)> {
    let mut out: Vec<(u32, Q)> = Vec::new();
    for e in &m.events {
        let end = e.start + e.duration;
        match out.iter_mut().find(|v| v.0 == e.voice) {
            Some(v) => v.1 = v.1.max(end),
            None => out.push((e.voice, end)),
        }
    }
    out.sort_by_key(|v| v.0);
    out
}

/// Longest voice of measures (None when empty).
fn length(ms: &[&Measure]) -> Option<Q> {
    ms.iter().flat_map(|m| voice_ends(m)).map(|v| v.1).max()
}

/// The report text and the number of flagged measures.
pub fn write(pages: &[Page], score: &Score) -> (String, usize) {
    let mut x = String::new();
    let parts = &score.parts;
    let n = parts.iter().map(|p| p.measures.len()).min().unwrap_or(0);
    let labels = parts.first().map(measure_labels).unwrap_or_default();
    // System number (1-based, as scoretool crops S01, S02, ...) of each measure.
    let mut system = Vec::with_capacity(n);
    let mut s = 0;
    for (mi, m) in parts.first().map_or(&[][..], |p| &p.measures[..]).iter().enumerate() {
        if m.new_system || mi == 0 {
            s += 1;
        }
        system.push(s);
    }
    let page_of = |sys: usize| {
        let mut acc = 0;
        for (k, p) in pages.iter().enumerate() {
            acc += p.systems;
            if sys <= acc {
                return k + 1;
            }
        }
        pages.len()
    };

    let _ = writeln!(x, "# audiveris-native report");
    let _ = writeln!(x);
    let _ = writeln!(x, "## pages");
    let mut first_sys = 1;
    for (k, p) in pages.iter().enumerate() {
        let systems = if p.systems == 0 {
            "no system".to_string()
        } else {
            format!("systems S{:02}-S{:02}", first_sys, first_sys + p.systems - 1)
        };
        let _ = writeln!(
            x,
            "page {} {}: {}x{} px, interline {} px, {}",
            k + 1,
            p.name,
            p.width,
            p.height,
            p.interline,
            systems
        );
        if p.interline < MIN_INTERLINE {
            let _ = writeln!(
                x,
                "  warning: interline below {MIN_INTERLINE} px, resolution too low for reliable recognition (scan at 300 dpi or more)"
            );
        }
        for note in &p.notes {
            let _ = writeln!(x, "  {note}");
        }
        first_sys += p.systems;
    }

    let _ = writeln!(x);
    let _ = writeln!(x, "## structure");
    let counts: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| format!("P{}={}", i + 1, p.measures.len()))
        .collect();
    let _ = writeln!(
        x,
        "measures: {} ({}), numbered {}..{}",
        n,
        counts.join(", "),
        labels.first().map_or("-", |l| l.as_str()),
        labels.last().map_or("-", |l| l.as_str()),
    );
    if parts.iter().any(|p| p.measures.len() != n) {
        let _ = writeln!(x, "  warning: parts have different measure counts");
    }
    // Time and key changes as written.
    let mut changes = Vec::new();
    for mi in 0..n {
        let Some(a) = &parts[0].measures[mi].attributes else { continue };
        let mut items = Vec::new();
        if let Some(t) = a.time {
            items.push(format!("time {}/{}", t.beats, t.beat_type));
        }
        if let Some(f) = a.fifths {
            items.push(format!("key {f}"));
        }
        if !items.is_empty() {
            changes.push(format!("m{} (S{:02}) {}", labels[mi], system[mi], items.join(" ")));
        }
    }
    let _ = writeln!(x, "time/key: {}", changes.join("; "));

    // Measures still covered by a multiple rest, per part.
    let mut rest_left = vec![0u32; parts.len()];
    let mut flagged = Vec::new();
    for mi in 0..n {
        let ms: Vec<&Measure> = parts.iter().map(|p| &p.measures[mi]).collect();
        let expected = ms[0].expected;
        let mut issues = Vec::new();
        for (pi, m) in ms.iter().enumerate() {
            if let Some(r) = m.attributes.as_ref().and_then(|a| a.multiple_rest) {
                rest_left[pi] = r - 1;
                issues.push(format!("P{} multi-measure rest of {r} (check the number against the image)", pi + 1));
                continue;
            }
            if rest_left[pi] > 0 {
                rest_left[pi] -= 1;
                continue;
            }
            // Nothing recognized in a part that has music elsewhere in the
            // system (a part absent from the system is empty throughout).
            if m.events.is_empty() && !m.implicit {
                let same_system = |k: usize| system[k] == system[mi];
                if (0..n).any(|k| same_system(k) && !parts[pi].measures[k].events.is_empty()) {
                    issues.push(format!("P{} empty (no note or rest recognized)", pi + 1));
                }
                continue;
            }
            for (v, end) in voice_ends(m) {
                if end == m.expected || (m.implicit && end < m.expected) {
                    continue;
                }
                let what = if end > m.expected { "overfull" } else { "underfull" };
                issues.push(format!(
                    "P{} v{v} {what} ({} of {} quarters)",
                    pi + 1,
                    quarters(end),
                    quarters(m.expected)
                ));
            }
        }
        if issues.is_empty() {
            continue;
        }
        // A missed or extra barline spans the system: every part shows it.
        let lengths = |k: usize| -> Vec<Q> {
            parts.iter().filter_map(|p| length(&[&p.measures[k]])).collect()
        };
        let all = |k: usize, f: &dyn Fn(Q) -> bool| {
            let l = lengths(k);
            !l.is_empty() && l.into_iter().all(f)
        };
        let double = |l: Q| l >= expected * Q::new(7, 4);
        let half = |l: Q| l * Q::new(2, 1) <= expected;
        let mut hints = Vec::new();
        if all(mi, &double) {
            hints.push("about two measures of music in every part: a barline may have been missed");
        }
        if all(mi, &half) && ((mi > 0 && all(mi - 1, &half)) || (mi + 1 < n && all(mi + 1, &half))) {
            hints.push("half-length measures side by side in every part: a barline may have been added");
        }
        if mi + 1 == n && all(mi, &|l| l < expected) {
            hints.push("last measure: fine if it completes the pickup");
        }
        if ms.iter().any(|m| m.events.iter().any(|e| e.tuplet.is_some())) {
            hints.push("tuplets involved");
        }
        if ms.iter().any(|m| m.events.iter().any(|e| (e.voice - 1) % 4 != 0)) {
            hints.push("several voices on a staff: voice assignment may be wrong");
        }
        let mut line = format!(
            "m{} (page {}, S{:02}): {}",
            labels[mi],
            page_of(system[mi]),
            system[mi],
            issues.join("; ")
        );
        if !hints.is_empty() {
            let _ = write!(line, " [{}]", hints.join("; "));
        }
        flagged.push(line);
    }
    let _ = writeln!(x);
    let _ = writeln!(x, "## flagged measures: {} of {}", flagged.len(), n);
    let _ = writeln!(
        x,
        "Durations that do not add up to the time signature, or values to check against the image: treat their rhythm as unreliable."
    );
    for f in &flagged {
        let _ = writeln!(x, "{f}");
    }
    (x, flagged.len())
}
