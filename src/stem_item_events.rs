// SPDX-License-Identifier: AGPL-3.0-or-later
//! Item construction and gap events from declared, already selected sources.
//! Candidate discovery/filter selection, final physical parts/cache and epilog
//! remain separate. Tokens are caller item identities, not registry/SIG IDs.
use crate::{
    beam_dispatch, beam_structure,
    stem_length_map::{self, Item},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cmp::Ordering, collections::BTreeMap};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Kind {
    HalfLinkerItem,
    LinkerItem,
    GlyphItem,
    GapItem,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub token: usize,
    pub kind: Kind,
    pub glyph: Option<Value>,
    pub reference: Option<[f64; 2]>,
    pub beam_height: Option<f64>,
    pub head_bounds: Option<[i32; 4]>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    pub token: Option<usize>,
    pub kind: Kind,
    pub reference: Option<[f64; 2]>,
    pub contrib: i32,
    pub line: [f64; 4],
    pub head_bounds: Option<[i32; 4]>,
}
impl Prepared {
    pub fn length_item(&self) -> Item {
        Item {
            gap: self.kind == Kind::GapItem,
            contrib: self.contrib,
            line: Some(self.line),
            head_bounds: self.head_bounds,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Events {
    pub sorted_tokens: Vec<usize>,
    pub items: Vec<Prepared>,
    pub truncated: bool,
}
fn rectangle(b: [i32; 4]) -> Result<(), String> {
    if b.iter().any(|v| v.unsigned_abs() > 1_000_000)
        || b[2] < 0
        || b[3] < 0
        || (b[0] as i64 + b[2] as i64).abs() > 1_000_000
        || (b[1] as i64 + b[3] as i64).abs() > 1_000_000
    {
        return Err("invalid item source bounds".into());
    }
    Ok(())
}
/// Glyph/HalfLinker contributions clip to the original theoretical yRange;
/// full LinkerItem uses the stump's entire height, as its original constructor.
pub fn construct(sources: &[Source], y_range: [i32; 4]) -> Result<Vec<Prepared>, String> {
    rectangle(y_range)?;
    if sources.len() > 32 {
        return Err("unsupported item suffix TimSort merge size".into());
    }
    if !sources.is_empty() && sources[0].kind != Kind::HalfLinkerItem {
        return Err("starting item must be a half linker".into());
    }
    let mut aliases = BTreeMap::new();
    let mut pixels = 0usize;
    let mut out = Vec::new();
    for s in sources {
        if s.token == 0 || s.kind == Kind::GapItem {
            return Err("invalid source item token/kind".into());
        }
        let identity = serde_json::to_value(s).map_err(|e| e.to_string())?;
        if let Some(old) = aliases.insert(s.token, identity.clone())
            && old != identity
        {
            return Err("conflicting item source identity".into());
        }
        if let Some(r) = s.reference
            && r.iter().any(|v| !v.is_finite() || v.abs() > 1_000_000.)
        {
            return Err("invalid item reference".into());
        }
        if let Some(b) = s.head_bounds {
            rectangle(b)?;
            if s.kind != Kind::HalfLinkerItem {
                return Err("head bounds require a corner half linker".into());
            }
        }
        if let Some(h) = s.beam_height
            && (s.kind != Kind::LinkerItem || !h.is_finite() || !(0.0..=1_000_000.).contains(&h))
        {
            return Err("invalid full beam height".into());
        }
        if (s.kind == Kind::GlyphItem && (s.reference.is_some() || s.beam_height.is_some()))
            || (s.kind != Kind::GlyphItem && s.reference.is_none())
        {
            return Err("invalid item source reference/kind".into());
        }
        let glyph = s
            .glyph
            .as_ref()
            .map(beam_dispatch::glyph_from_json)
            .transpose()?;
        let (line, contrib) = if let Some(g) = glyph {
            beam_dispatch::centroid(&g)?;
            for r in &g.runs {
                pixels = pixels
                    .checked_add(r.length)
                    .filter(|n| *n <= 8_000_000)
                    .ok_or("item source pixel budget")?;
            }
            let b = [g.left, g.top, g.width as i32, g.height as i32];
            rectangle(b)?;
            let line = beam_structure::glyph_geometry_signed(&g)?
                .center_line
                .ok_or("unsupported single-pixel item source")?;
            let c = if s.kind == Kind::LinkerItem {
                b[3]
            } else {
                ((b[1] + b[3]).min(y_range[1] + y_range[3]) - b[1].max(y_range[1])).max(0)
            };
            (line, c)
        } else {
            if s.kind == Kind::GlyphItem {
                return Err("glyph item needs a physical glyph".into());
            }
            let [x, y] = s.reference.unwrap();
            let h = s.beam_height.unwrap_or(0.) / 2.;
            ([x, y - h, x, y + h], 0)
        };
        stem_length_map::line_bounds(line)?;
        out.push(Prepared {
            token: Some(s.token),
            kind: s.kind,
            reference: if s.kind == Kind::HalfLinkerItem {
                s.reference
            } else {
                None
            },
            contrib,
            line,
            head_bounds: s.head_bounds,
        });
    }
    Ok(out)
}
fn compare(a: &Prepared, b: &Prepared, dir: i8) -> Ordering {
    if a.kind == Kind::HalfLinkerItem && b.kind == Kind::HalfLinkerItem {
        let a = a.reference.unwrap()[1];
        let b = b.reference.unwrap()[1];
        if dir > 0 {
            a.total_cmp(&b)
        } else {
            b.total_cmp(&a)
        }
    } else if dir > 0 {
        a.line[1].total_cmp(&b.line[1])
    } else {
        b.line[3].total_cmp(&a.line[3])
    }
}
// Java TimSort's run detection and binary insertion for suffixes shorter than32.
// A mixed comparator can be non-transitive; Rust's general sort is not used.
fn sort(items: &mut [Prepared], dir: i8) {
    if items.len() < 2 {
        return;
    }
    let descending = compare(&items[1], &items[0], dir) == Ordering::Less;
    let mut run = 2;
    while run < items.len()
        && if descending {
            compare(&items[run], &items[run - 1], dir) == Ordering::Less
        } else {
            compare(&items[run], &items[run - 1], dir) != Ordering::Less
        }
    {
        run += 1;
    }
    if descending {
        items[..run].reverse();
    }
    for i in run..items.len() {
        let pivot = items[i].clone();
        let (mut left, mut right) = (0, i);
        while left < right {
            let mid = (left + right) / 2;
            if compare(&pivot, &items[mid], dir) == Ordering::Less {
                right = mid
            } else {
                left = mid + 1
            }
        }
        items[left..=i].rotate_right(1);
    }
}
/// Sort the suffix, preserving entry zero. Insert gaps above0.01 and truncate
/// before an item whose directional distance exceeds the declared profile gap.
/// Prepared inputs permit explicit original boundary observations; construction
/// from source glyphs/linkers is a separate checked operation above.
pub fn events(input: &[Prepared], direction: i8, max_gap: i32) -> Result<Events, String> {
    if input.len() > 32 || !matches!(direction, -1 | 1) || !(0..=1_000_000).contains(&max_gap) {
        return Err("unsupported event size/direction/profile gap".into());
    }
    for i in input {
        if i.kind == Kind::GapItem || i.token.is_none_or(|t| t == 0) || i.line[1] > i.line[3] {
            return Err("invalid prepared source item".into());
        }
        if i.kind == Kind::HalfLinkerItem
            && i.reference
                .is_none_or(|r| r.iter().any(|v| !v.is_finite() || v.abs() > 1_000_000.))
        {
            return Err("invalid prepared half-linker reference".into());
        }
    }
    let native = input
        .iter()
        .map(|i| Some(i.length_item()))
        .collect::<Vec<_>>();
    stem_length_map::prefix_lengths(&native, direction, 0.)?;
    let mut sorted = input.to_vec();
    if sorted.len() > 1 {
        sort(&mut sorted[1..], direction);
    }
    let sorted_tokens = sorted.iter().map(|i| i.token.unwrap()).collect();
    let mut out = Vec::new();
    let mut furthest: Option<[f64; 2]> = None;
    for i in sorted {
        let start = if direction > 0 {
            [i.line[0], i.line[1]]
        } else {
            [i.line[2], i.line[3]]
        };
        let end = if direction > 0 {
            [i.line[2], i.line[3]]
        } else {
            [i.line[0], i.line[1]]
        };
        if let Some(previous) = furthest {
            let distance = direction as f64 * (start[1] - previous[1]);
            if distance > max_gap as f64 {
                return Ok(Events {
                    sorted_tokens,
                    items: out,
                    truncated: true,
                });
            }
            if distance > 0.01 {
                let (a, b) = if direction > 0 {
                    (previous, start)
                } else {
                    (start, previous)
                };
                let line = [a[0], a[1], b[0], b[1]];
                let contrib = stem_length_map::line_bounds(line)?[3];
                out.push(Prepared {
                    token: None,
                    kind: Kind::GapItem,
                    reference: None,
                    contrib,
                    line,
                    head_bounds: None,
                });
            }
            if direction as f64 * (end[1] - previous[1]) > 0.01 {
                furthest = Some(end);
            }
        } else {
            furthest = Some(end);
        }
        out.push(i);
    }
    Ok(Events {
        sorted_tokens,
        items: out,
        truncated: false,
    })
}
