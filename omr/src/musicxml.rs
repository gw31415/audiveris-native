//! MusicXML 4.0 (score-partwise) export.

use std::fmt::Write;

use crate::grid::BarStyle;
use crate::rational::{Q, gcd, lcm};
use crate::score::{Event, Kind, Measure, Score};

fn type_name(base: Q) -> &'static str {
    match (base.n, base.d) {
        (4, 1) => "long",
        (2, 1) => "breve",
        (1, 1) => "whole",
        (1, 2) => "half",
        (1, 4) => "quarter",
        (1, 8) => "eighth",
        (1, 16) => "16th",
        (1, 32) => "32nd",
        (1, 64) => "64th",
        (1, 128) => "128th",
        _ => "256th",
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn divisions(score: &Score) -> i64 {
    let mut div = 1i64;
    let mut add = |q: Q| {
        if q.n != 0 {
            // quarters = 4q; need div * 4n/d integer
            div = lcm(div, q.d / gcd(4 * q.n, q.d));
        }
    };
    for p in &score.parts {
        for m in &p.measures {
            add(m.expected);
            for e in &m.events {
                add(e.duration);
                add(e.start);
            }
        }
    }
    div
}

fn dur(q: Q, div: i64) -> i64 {
    q.n * 4 * div / q.d
}

pub fn write(score: &Score, title: &str) -> String {
    let div = divisions(score);
    let mut x = String::new();
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n");
    x.push_str("<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 4.0 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\">\n");
    x.push_str("<score-partwise version=\"4.0\">\n");
    let _ = writeln!(x, "  <work><work-title>{}</work-title></work>", esc(title));
    x.push_str("  <identification><encoding><software>audiveris-native</software></encoding></identification>\n");
    x.push_str("  <part-list>\n");
    for (i, p) in score.parts.iter().enumerate() {
        let _ = writeln!(
            x,
            "    <score-part id=\"P{}\"><part-name>{}</part-name></score-part>",
            i + 1,
            esc(&p.name)
        );
    }
    x.push_str("  </part-list>\n");
    for (i, p) in score.parts.iter().enumerate() {
        let _ = writeln!(x, "  <part id=\"P{}\">", i + 1);
        for (mi, m) in p.measures.iter().enumerate() {
            let _ = writeln!(x, "    <measure number=\"{}\">", mi + 1);
            if m.new_system && mi > 0 {
                x.push_str("      <print new-system=\"yes\"/>\n");
            }
            write_attributes(&mut x, m, mi == 0, div, p.staves);
            write_measure_body(&mut x, m, div, p.staves);
            let style = match m.right_bar {
                BarStyle::Regular => None,
                BarStyle::LightLight => Some("light-light"),
                BarStyle::LightHeavy => Some("light-heavy"),
                BarStyle::HeavyLight => Some("heavy-light"),
                BarStyle::Heavy => Some("heavy"),
            };
            if let Some(s) = style {
                let _ = writeln!(
                    x,
                    "      <barline location=\"right\"><bar-style>{s}</bar-style></barline>"
                );
            }
            x.push_str("    </measure>\n");
        }
        x.push_str("  </part>\n");
    }
    x.push_str("</score-partwise>\n");
    x
}

fn write_attributes(x: &mut String, m: &Measure, first: bool, div: i64, staves: u32) {
    let Some(a) = &m.attributes else {
        if first {
            let _ = writeln!(
                x,
                "      <attributes><divisions>{div}</divisions></attributes>"
            );
        }
        return;
    };
    x.push_str("      <attributes>\n");
    if first {
        let _ = writeln!(x, "        <divisions>{div}</divisions>");
    }
    if let Some(f) = a.fifths {
        let _ = writeln!(x, "        <key><fifths>{f}</fifths></key>");
    }
    if let Some(t) = a.time {
        match t.symbol {
            Some(s) => {
                let _ = writeln!(
                    x,
                    "        <time symbol=\"{s}\"><beats>{}</beats><beat-type>{}</beat-type></time>",
                    t.beats, t.beat_type
                );
            }
            None => {
                let _ = writeln!(
                    x,
                    "        <time><beats>{}</beats><beat-type>{}</beat-type></time>",
                    t.beats, t.beat_type
                );
            }
        }
    }
    if first && staves > 1 {
        let _ = writeln!(x, "        <staves>{staves}</staves>");
    }
    for (n, c) in &a.clefs {
        write_clef(x, *n, *c, staves, "        ");
    }
    x.push_str("      </attributes>\n");
}

fn write_clef(x: &mut String, n: u32, c: crate::header::ClefKind, staves: u32, indent: &str) {
    let (sign, line, oct) = c.musicxml();
    let num = if staves > 1 {
        format!(" number=\"{n}\"")
    } else {
        String::new()
    };
    let _ = write!(
        x,
        "{indent}<clef{num}><sign>{sign}</sign><line>{line}</line>"
    );
    if oct != 0 {
        let _ = write!(x, "<clef-octave-change>{oct}</clef-octave-change>");
    }
    x.push_str("</clef>\n");
}

fn write_measure_body(x: &mut String, m: &Measure, div: i64, staves: u32) {
    let mut voices: Vec<u32> = m.events.iter().map(|e| e.voice).collect();
    voices.sort();
    voices.dedup();
    if voices.is_empty() {
        // Empty measure: one whole-measure rest per staff.
        for n in 1..=staves {
            if n > 1 {
                let _ = writeln!(
                    x,
                    "      <backup><duration>{}</duration></backup>",
                    dur(m.expected, div)
                );
            }
            let _ = write!(
                x,
                "      <note><rest measure=\"yes\"/><duration>{}</duration><voice>{}</voice>",
                dur(m.expected, div),
                1 + 4 * (n - 1)
            );
            if staves > 1 {
                let _ = write!(x, "<staff>{n}</staff>");
            }
            x.push_str("</note>\n");
        }
        return;
    }
    let mut done_dirs = vec![false; m.directions.len()];
    let mut done_clefs = vec![false; m.clef_changes.len()];
    for (vi, &v) in voices.iter().enumerate() {
        let mut evs: Vec<&Event> = m.events.iter().filter(|e| e.voice == v).collect();
        evs.sort_by(|a, b| a.start.cmp(&b.start).then(a.x.partial_cmp(&b.x).unwrap()));
        let staff = evs[0].staff;
        let mut cursor = Q::ZERO;
        for e in evs {
            if e.start > cursor {
                let _ = write!(
                    x,
                    "      <forward><duration>{}</duration><voice>{v}</voice>",
                    dur(e.start - cursor, div)
                );
                if staves > 1 {
                    let _ = write!(x, "<staff>{staff}</staff>");
                }
                x.push_str("</forward>\n");
            } else if e.start < cursor {
                // Overlap inside a voice (recognition inconsistency): back up.
                let _ = writeln!(
                    x,
                    "      <backup><duration>{}</duration></backup>",
                    dur(cursor - e.start, div)
                );
            }
            for (ci, (n, t, c)) in m.clef_changes.iter().enumerate() {
                if !done_clefs[ci] && *n == e.staff && *t <= e.start {
                    done_clefs[ci] = true;
                    x.push_str("      <attributes>\n");
                    write_clef(x, *n, *c, staves, "        ");
                    x.push_str("      </attributes>\n");
                }
            }
            for (di, d) in m.directions.iter().enumerate() {
                if !done_dirs[di] && d.staff == e.staff && d.start <= e.start {
                    done_dirs[di] = true;
                    let _ = write!(
                        x,
                        "      <direction placement=\"below\"><direction-type><dynamics><{}/></dynamics></direction-type>",
                        d.dynamics
                    );
                    if staves > 1 {
                        let _ = write!(x, "<staff>{}</staff>", d.staff);
                    }
                    x.push_str("</direction>\n");
                }
            }
            write_event(x, e, div, staves);
            cursor = e.start + e.duration;
        }
        if vi + 1 < voices.len() && cursor > Q::ZERO {
            let _ = writeln!(
                x,
                "      <backup><duration>{}</duration></backup>",
                dur(cursor, div)
            );
        }
    }
}

fn write_event(x: &mut String, e: &Event, div: i64, staves: u32) {
    let d = dur(e.duration, div);
    let common_tail = |x: &mut String, with_beams: bool| {
        if let Some((a, b)) = e.tuplet {
            let _ = write!(
                x,
                "<time-modification><actual-notes>{a}</actual-notes><normal-notes>{b}</normal-notes></time-modification>"
            );
        }
        if let Kind::Chord {
            stem_up: Some(up), ..
        } = e.kind
        {
            let _ = write!(x, "<stem>{}</stem>", if up { "up" } else { "down" });
        }
        if staves > 1 {
            let _ = write!(x, "<staff>{}</staff>", e.staff);
        }
        if with_beams {
            for (n, v) in &e.beams {
                let _ = write!(x, "<beam number=\"{n}\">{v}</beam>");
            }
        }
    };
    match &e.kind {
        Kind::Rest { whole_measure } => {
            if *whole_measure {
                let _ = write!(
                    x,
                    "      <note><rest measure=\"yes\"/><duration>{d}</duration><voice>{}</voice>",
                    e.voice
                );
            } else {
                let _ = write!(
                    x,
                    "      <note><rest/><duration>{d}</duration><voice>{}</voice><type>{}</type>",
                    e.voice,
                    type_name(e.base)
                );
                for _ in 0..e.dots {
                    x.push_str("<dot/>");
                }
            }
            common_tail(x, false);
            x.push_str("</note>\n");
        }
        Kind::Chord { pitches, .. } => {
            for (i, p) in pitches.iter().enumerate() {
                x.push_str("      <note>");
                if i > 0 {
                    x.push_str("<chord/>");
                }
                let _ = write!(x, "<pitch><step>{}</step>", p.step);
                if p.alter != 0 {
                    let _ = write!(x, "<alter>{}</alter>", p.alter);
                }
                let _ = write!(
                    x,
                    "<octave>{}</octave></pitch><duration>{d}</duration><voice>{}</voice><type>{}</type>",
                    p.octave,
                    e.voice,
                    type_name(e.base)
                );
                for _ in 0..e.dots {
                    x.push_str("<dot/>");
                }
                if let Some(a) = p.accidental {
                    let _ = write!(x, "<accidental>{a}</accidental>");
                }
                common_tail(x, i == 0);
                x.push_str("</note>\n");
            }
        }
    }
}
