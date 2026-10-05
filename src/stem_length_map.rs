// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original StemBuilder length maps on declared items. Automatic item/target
//! selection, profile-4 artificial stem creation and page epilog are separate.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const LIMIT: i32 = 1_000_000;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub gap: bool,
    pub contrib: i32,
    pub line: Option<[f64; 4]>,
    /// Only a CLinker item's source HeadInter bounds belongs here.
    pub head_bounds: Option<[i32; 4]>,
}
fn rect(r: [i32; 4]) -> Result<[i32; 2], String> {
    if r.iter().any(|v| v.unsigned_abs() > LIMIT as u32)
        || r[2] < 0
        || r[3] < 0
        || (r[0] as i64 + r[2] as i64).abs() > LIMIT as i64
        || (r[1] as i64 + r[3] as i64).abs() > LIMIT as i64
    {
        return Err("invalid or unsupported item rectangle".into());
    }
    Ok([r[1], r[1] + r[3]])
}
/// Java Line2D.getBounds: floor the minima, ceil the maxima, including zero
/// width/height. This is an exclusive rectangle, not inclusive pixel bounds.
pub fn line_bounds(l: [f64; 4]) -> Result<[i32; 4], String> {
    if l.iter().any(|v| !v.is_finite() || v.abs() > LIMIT as f64) {
        return Err("invalid or unsupported item line".into());
    }
    let x = l[0].min(l[2]).floor() as i32;
    let y = l[1].min(l[3]).floor() as i32;
    Ok([
        x,
        y,
        l[0].max(l[2]).ceil() as i32 - x,
        l[1].max(l[3]).ceil() as i32 - y,
    ])
}
/// Entry zero corresponds to getLengthAt(-1), followed by each item prefix.
/// A null item is ignored here, as in the original getLengthAt method.
pub fn prefix_lengths(
    items: &[Option<Item>],
    direction: i8,
    start_y: f64,
) -> Result<Vec<i32>, String> {
    if items.len() > 4096
        || !matches!(direction, -1 | 1)
        || !start_y.is_finite()
        || start_y.abs() > LIMIT as f64
    {
        return Err("unsupported item count, direction or reference ordinate".into());
    }
    let start = start_y as i32; // Java d2i truncates toward zero in this bounded range.
    let mut union: Option<[i32; 2]> = None;
    let mut out = vec![0];
    for item in items {
        if let Some(i) = item {
            if i.contrib.unsigned_abs() > LIMIT as u32 {
                return Err("unsupported item contribution".into());
            }
            let line = i.line.map(line_bounds).transpose()?;
            let head = i.head_bounds.map(rect).transpose()?;
            if !i.gap
                && let Some(l) = line
            {
                let mut bounds = rect(l)?;
                if let Some(h) = head {
                    bounds = [bounds[0].min(h[0]), bounds[1].max(h[1])];
                }
                union = Some(match union {
                    Some(b) => [b[0].min(bounds[0]), b[1].max(bounds[1])],
                    None => bounds,
                });
            }
        }
        out.push(match union {
            None => 0,
            Some(b) => {
                if direction > 0 {
                    b[1] - start
                } else {
                    start - b[0]
                }
            }
        });
    }
    Ok(out)
}
/// First exceeded gaps keep the first stored length. A gap larger than the
/// selected maximum profile stops immediately; remaining profiles stay absent.
/// TreeMap key order is preserved even for non-monotonic supplied thresholds.
pub fn retrieve_lengths(
    items: &[Option<Item>],
    direction: i8,
    start_y: f64,
    gap_map: &BTreeMap<i32, i32>,
    max_profile: i32,
    initial: &BTreeMap<i32, i32>,
) -> Result<BTreeMap<i32, i32>, String> {
    if gap_map.is_empty()
        || gap_map.len() > 32
        || initial.len() > 32
        || gap_map
            .iter()
            .any(|(&k, &v)| !(0..32).contains(&k) || !(0..=LIMIT).contains(&v))
        || initial
            .iter()
            .any(|(&k, &v)| !(0..32).contains(&k) || v.unsigned_abs() > LIMIT as u32)
        || items.iter().any(Option::is_none)
    {
        return Err("unsupported gap map, initial lengths or null retrieveLengths item".into());
    }
    let max_gap = *gap_map
        .get(&max_profile)
        .ok_or("maximum profile absent from gap map")?;
    let prefixes = prefix_lengths(items, direction, start_y)?;
    let mut lengths = initial.clone();
    for (index, item) in items.iter().enumerate() {
        let item = item.as_ref().unwrap();
        if !item.gap {
            continue;
        }
        for (&profile, &gap) in gap_map {
            if item.contrib <= gap {
                break;
            }
            lengths.entry(profile).or_insert(prefixes[index]);
        }
        if item.contrib > max_gap {
            lengths.entry(max_profile).or_insert(prefixes[index]);
            return Ok(lengths);
        }
    }
    for &profile in gap_map.keys() {
        lengths.entry(profile).or_insert(*prefixes.last().unwrap());
    }
    Ok(lengths)
}
