// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected StemBuilder.filter boundary. Candidate discovery, item expansion,
//! gap/length maps, link creation and epilog are separate dependencies.
use crate::{
    beam_dispatch, beam_structure, stem_glyph,
    stem_parts_alignment::{self, Part},
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    HeadCorner,
    BeamHalf,
    Beam,
}
#[derive(Clone, Copy)]
pub struct Linker<'a> {
    pub id: usize,
    pub kind: Kind,
    pub stump: Option<Part<'a>>,
    pub reference: [f64; 2],
    pub beam_height: Option<f64>,
}
#[derive(Clone)]
pub struct Parameters {
    pub alignment: stem_parts_alignment::Parameters,
    pub y_range: [i32; 4],
    pub start_is_beam: bool,
    pub head_box: Option<[i32; 4]>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Item {
    pub kind: &'static str,
    pub contrib: i32,
    pub line: [f64; 4],
    pub glyph_id: Option<usize>,
    pub linker_id: Option<usize>,
}
#[derive(Debug, serde::Serialize)]
pub struct Filtered {
    pub retained_part_order: Vec<usize>,
    pub retained_linker_order: Vec<usize>,
    pub last_head_y: Option<f64>,
    pub items: Vec<Item>,
}
fn bounds(b: [i32; 4]) -> Result<(), String> {
    if b[2] < 0
        || b[3] < 0
        || b.iter().any(|x| x.unsigned_abs() > 4_000_000)
        || (b[0] as i64 + b[2] as i64).abs() > 4_000_000
        || (b[1] as i64 + b[3] as i64).abs() > 4_000_000
    {
        return Err("invalid_stem_filter_bounds".into());
    }
    Ok(())
}
fn overlap(a: [i32; 4], b: [i32; 4]) -> i32 {
    (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1])
}
fn physical_bounds(p: Part<'_>) -> [i32; 4] {
    [
        p.glyph.left,
        p.glyph.top,
        p.glyph.width as i32,
        p.glyph.height as i32,
    ]
}
fn line(p: Part<'_>) -> Result<[f64; 4], String> {
    let line = beam_structure::glyph_geometry_signed(p.glyph)?
        .center_line
        .ok_or("unsupported_single_pixel_filter_item")?;
    if line.iter().any(|v| !v.is_finite()) {
        return Err("unsupported_nonfinite_filter_item_line".into());
    }
    Ok(line)
}
fn cached_line(p: Part<'_>, cache: &mut BTreeMap<usize, [f64; 4]>) -> Result<[f64; 4], String> {
    if let Some(line) = cache.get(&p.id) {
        return Ok(*line);
    }
    let line = line(p)?;
    cache.insert(p.id, line);
    Ok(line)
}
fn same_linker(a: Linker<'_>, b: Linker<'_>) -> bool {
    a.kind == b.kind
        && a.reference == b.reference
        && a.beam_height == b.beam_height
        && a.stump.map(|x| x.id) == b.stump.map(|x| x.id)
}
// Java TimSort's small-list run detection and stable binary insertion. The
// mixed half/full linker comparator can depend on the pair's types. Larger
// lists need independently compared TimSort merge behavior and refuse.
fn sort(items: &mut [Item], links: &BTreeMap<usize, Linker<'_>>, direction: i8) {
    let compare = |a: &Item, b: &Item| {
        let la = links[&a.linker_id.unwrap()];
        let lb = links[&b.linker_id.unwrap()];
        if la.kind != Kind::Beam && lb.kind != Kind::Beam {
            if direction > 0 {
                la.reference[1].partial_cmp(&lb.reference[1]).unwrap()
            } else {
                lb.reference[1].partial_cmp(&la.reference[1]).unwrap()
            }
        } else if direction > 0 {
            a.line[1].partial_cmp(&b.line[1]).unwrap()
        } else {
            b.line[3].partial_cmp(&a.line[3]).unwrap()
        }
    };
    if items.len() < 2 {
        return;
    }
    let mut run = 2;
    let descending = compare(&items[1], &items[0]) == Ordering::Less;
    while run < items.len()
        && if descending {
            compare(&items[run], &items[run - 1]) == Ordering::Less
        } else {
            compare(&items[run], &items[run - 1]) != Ordering::Less
        }
    {
        run += 1;
    }
    if descending {
        items[..run].reverse()
    }
    for i in run..items.len() {
        let pivot = items[i].clone();
        let (mut left, mut right) = (0, i);
        while left < right {
            let mid = (left + right) / 2;
            if compare(&pivot, &items[mid]) == Ordering::Less {
                right = mid
            } else {
                left = mid + 1
            }
        }
        items[left..=i].rotate_right(1);
    }
}
/// Input IDs are caller object identity tags, independent of physical registry
/// content IDs. Results are a pure plan; no registry/cache/SIG mutation occurs.
pub fn filter(
    parts: &[Part<'_>],
    start_stump: Option<Part<'_>>,
    linkers: &[Linker<'_>],
    p: &Parameters,
) -> Result<Filtered, String> {
    bounds(p.y_range)?;
    if let Some(b) = p.head_box {
        bounds(b)?
    }
    if linkers.len() >= 32 {
        return Err("unsupported_stem_filter_linker_sort_size".into());
    }
    if parts.len() > stem_glyph::MAX_PARTS {
        return Err("stem_filter_part_budget".into());
    }
    let mut glyphs = BTreeMap::<usize, &audiveris_staff_removal::GlyphPixels>::new();
    let mut total = 0usize;
    for part in parts
        .iter()
        .copied()
        .chain(start_stump)
        .chain(linkers.iter().filter_map(|l| l.stump))
    {
        if part.id == 0 {
            return Err("invalid_stem_filter_glyph_id".into());
        }
        if let Some(g) = glyphs.get(&part.id) {
            if beam_dispatch::glyph_json(g) != beam_dispatch::glyph_json(part.glyph) {
                return Err("conflicting_stem_filter_glyph_identity".into());
            }
        } else {
            beam_dispatch::centroid(part.glyph)?;
            let weight = part
                .glyph
                .runs
                .iter()
                .try_fold(0usize, |n, r| n.checked_add(r.length))
                .ok_or("stem_filter_pixel_budget")?;
            total = total
                .checked_add(weight)
                .filter(|n| *n <= stem_glyph::MAX_PART_PIXELS)
                .ok_or("stem_filter_pixel_budget")?;
            glyphs.insert(part.id, part.glyph);
        }
    }
    let mut links = BTreeMap::new();
    for &l in linkers {
        if l.id == 0
            || l.reference
                .iter()
                .any(|x| !x.is_finite() || x.abs() > 4_000_000.)
            || (l.kind == Kind::Beam
                && l.beam_height
                    .is_none_or(|x| !x.is_finite() || !(0.0..=4_000_000.).contains(&x)))
            || (l.kind != Kind::Beam && l.beam_height.is_some())
        {
            return Err("invalid_stem_filter_linker".into());
        }
        if let Some(old) = links.insert(l.id, l)
            && !same_linker(old, l)
        {
            return Err("conflicting_stem_filter_linker_identity".into());
        }
    }
    let aligned = stem_parts_alignment::filter(parts, start_stump, &p.alignment)?;
    let mut dropped = aligned.removed_order.into_iter().collect::<BTreeSet<_>>();
    let retained = linkers
        .iter()
        .copied()
        .filter(|l| l.stump.is_none_or(|s| !dropped.contains(&s.id)))
        .collect::<Vec<_>>();
    let mut items = Vec::new();
    let mut lines = BTreeMap::new();
    for l in &retained {
        let (center, contrib) = if let Some(s) = l.stump {
            (
                cached_line(s, &mut lines)?,
                if l.kind == Kind::Beam {
                    s.glyph.height as i32
                } else {
                    overlap(p.y_range, physical_bounds(s)).max(0)
                },
            )
        } else {
            let [x, y] = l.reference;
            let center = if l.kind == Kind::Beam {
                let h = l.beam_height.unwrap() / 2.;
                [x, y - h, x, y + h]
            } else {
                [x, y, x, y]
            };
            (center, 0)
        };
        items.push(Item {
            kind: if l.kind == Kind::Beam {
                "LinkerItem"
            } else {
                "HalfLinkerItem"
            },
            contrib,
            line: center,
            glyph_id: l.stump.map(|s| s.id),
            linker_id: Some(l.id),
        });
    }
    sort(&mut items, &links, p.alignment.direction);
    let last_head_y = if p.start_is_beam {
        items.iter().rev().find_map(|i| {
            let l = links[&i.linker_id.unwrap()];
            (l.kind == Kind::HeadCorner).then_some(l.reference[1])
        })
    } else {
        None
    };
    if let Some(y) = last_head_y {
        for part in parts {
            if !dropped.contains(&part.id) {
                let b = physical_bounds(*part);
                let center = b[1] + b[3] / 2;
                if (center as f64 - y) * p.alignment.direction as f64 >= 0. {
                    dropped.insert(part.id);
                }
            }
        }
    }
    let retained_parts = parts
        .iter()
        .copied()
        .filter(|part| !dropped.contains(&part.id))
        .collect::<Vec<_>>();
    for part in &retained_parts {
        if retained
            .iter()
            .any(|l| l.stump.is_some_and(|s| s.id == part.id))
        {
            continue;
        }
        let b = physical_bounds(*part);
        if p.head_box.is_some_and(|head| overlap(head, b) > 0) {
            continue;
        }
        let contrib = overlap(p.y_range, b).max(0);
        if contrib > 0 {
            items.push(Item {
                kind: "GlyphItem",
                contrib,
                line: cached_line(*part, &mut lines)?,
                glyph_id: Some(part.id),
                linker_id: None,
            });
        }
    }
    Ok(Filtered {
        retained_part_order: retained_parts.iter().map(|p| p.id).collect(),
        retained_linker_order: retained.iter().map(|l| l.id).collect(),
        last_head_y,
        items,
    })
}
