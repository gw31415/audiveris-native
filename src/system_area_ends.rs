// SPDX-License-Identifier: AGPL-3.0-or-later
// SystemManager.horizontalNeighbor and computeSystemArea horizontal ends at
// 9e1e55cd2746037d059345881c53e6a6754bffbd. Encounter order is authoritative.
use serde_json::{Value, json};
use std::collections::BTreeSet;
/// Derive horizontal ends from native SystemInfo coordinates, not area bounds.
/// yOverlaps uses strict overlap; touching bottom/top does not count.
pub fn discover(systems: &[Value], width: usize) -> Result<Vec<Value>, String> {
    if systems.is_empty() || systems.len() > 1024 || width == 0 || width > 4_000_000 {
        return Err("invalid_horizontal_system_geometry".into());
    }
    let mut seen = BTreeSet::new();
    let mut boxes = Vec::new();
    for s in systems {
        let id = s["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or("invalid_horizontal_system_id")?;
        if !seen.insert(id) {
            return Err("duplicate_horizontal_system_id".into());
        }
        let c = s["coordinates"]
            .as_array()
            .filter(|c| c.len() == 5)
            .ok_or("invalid_horizontal_system_coordinates")?;
        let v = c
            .iter()
            .map(|n| n.as_i64().ok_or("invalid_horizontal_system_coordinates"))
            .collect::<Result<Vec<_>, _>>()?;
        let right = v[0]
            .checked_add(v[2])
            .ok_or("invalid_horizontal_system_coordinates")?;
        if v[0] < 0
            || v[2] <= 0
            || right > width as i64
            || v[1] < 0
            || v[3] <= v[1]
            || v.iter().any(|n| n.unsigned_abs() > 4_000_000)
        {
            return Err("invalid_horizontal_system_coordinates".into());
        }
        boxes.push((id, v[0], v[1], right, v[3]));
    }
    let mut result = Vec::new();
    for (i, a) in boxes.iter().enumerate() {
        let overlaps = |j: usize| a.2.max(boxes[j].2) < a.4.min(boxes[j].4);
        let left = (0..i).rev().find(|&j| overlaps(j));
        let right = (i + 1..boxes.len()).find(|&j| overlaps(j));
        let l = left.map(|j| (boxes[j].3 + a.1) / 2).unwrap_or(0);
        let r = right
            .map(|j| (a.3 + boxes[j].1) / 2)
            .unwrap_or(width as i64);
        if l >= r || l < 0 || r > width as i64 {
            return Err("unsupported_horizontal_system_area_ends".into());
        }
        result.push(json!({"id":a.0,"left_neighbor":left.map(|j|boxes[j].0),"right_neighbor":right.map(|j|boxes[j].0),"x_range":[l,r]}));
    }
    Ok(result)
}
/// Legacy single-column gate; general GRID geometry uses system_area_geometry.
pub fn native_grid_ends(systems: &[Value], width: usize) -> Result<Vec<Value>, String> {
    let ends = discover(systems, width)?;
    if ends
        .iter()
        .any(|e| !e["left_neighbor"].is_null() || !e["right_neighbor"].is_null())
    {
        return Err("unsupported_grid_horizontal_staff_chains".into());
    }
    Ok(ends)
}
