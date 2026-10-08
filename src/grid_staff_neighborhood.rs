// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    grid_ir::{PartGeometry, StaffGeometry},
    system_area_geometry::Band,
};
use std::sync::Arc;
pub(crate) struct StaffNeighborhood {
    pub(crate) staffs: Vec<Arc<StaffGeometry>>,
    pub(crate) parts: Vec<Arc<PartGeometry>>,
}
impl StaffNeighborhood {
    pub(crate) fn part_bounds(&self) -> Vec<[i32; 4]> {
        debug_assert_eq!(self.staffs.len(), self.parts.len());
        self.parts.iter().map(|p| p.bounds).collect()
    }
}
/// SystemInfo.getStavesAround uses containing Staff areas first, integer
/// distance ties in encounter order, then core first/last curves at this x.
pub(crate) fn around(
    staffs: &[Arc<StaffGeometry>],
    areas: &[Arc<Band>],
    parts: &[Arc<PartGeometry>],
    system: u64,
    point: [f64; 2],
) -> Result<StaffNeighborhood, String> {
    if staffs.len() != areas.len() || point.iter().any(|v| !v.is_finite()) {
        return Err("grid_actual_staff_area_required".into());
    }
    let current = staffs
        .iter()
        .enumerate()
        .filter(|(_, s)| s.system == system)
        .collect::<Vec<_>>();
    let mut closest = None;
    for (i, (index, staff)) in current.iter().enumerate() {
        if areas[*index].contains(point)? {
            let top = staff
                .lines
                .first()
                .ok_or("grid_actual_staff_lines_required")?
                .y(point[0]);
            let bottom = staff.lines.last().unwrap().y(point[0]);
            if !top.is_finite() || !bottom.is_finite() {
                return Err("grid_actual_staff_lines_required".into());
            }
            let distance = (top - point[1]).max(point[1] - bottom) as i32;
            if closest.is_none_or(|(_, best)| distance < best) {
                closest = Some((i, distance));
            }
        }
    }
    let Some((index, _)) = closest else {
        return Ok(StaffNeighborhood {
            staffs: Vec::new(),
            parts: Vec::new(),
        });
    };
    let staff = current[index].1;
    let to_top = staff.lines.first().unwrap().y(point[0]) - point[1];
    let to_bottom = staff.lines.last().unwrap().y(point[0]) - point[1];
    let mut first = index;
    let mut last = index;
    if to_top * to_bottom > 0. {
        if to_top > 0. {
            first = first.saturating_sub(1);
        } else if last + 1 < current.len() {
            last += 1;
        }
    }
    let mut found_staffs = Vec::new();
    let mut found_parts = Vec::new();
    for (_, staff) in &current[first..=last] {
        let matches = parts
            .iter()
            .filter(|p| p.system == system && p.staffs.iter().any(|s| Arc::ptr_eq(s, staff)))
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].id == 0 {
            return Err("grid_actual_staff_part_required".into());
        }
        found_staffs.push((*staff).clone());
        found_parts.push(matches[0].clone());
    }
    // Two surrounding staves may refer to the same actual Part; retain both.
    Ok(StaffNeighborhood {
        staffs: found_staffs,
        parts: found_parts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn real_image_original_staff_core_beam_centers_outside_areas_and_same_part_neighbors_match_retained_owners()
     {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-real-beam-staff-neighborhood/observed.json"
        ))
        .unwrap();
        let image =
            crate::GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png"))
                .unwrap();
        let page = crate::page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
        let graph = page.graph.snapshot();
        let staff_storage = page.staffs.as_ptr();
        let part_storage = page.parts.as_ptr();
        for part in &page.parts {
            assert!(
                part.staffs
                    .iter()
                    .all(|s| page.staffs.iter().any(|held| Arc::ptr_eq(s, held)))
            );
        }
        let mut duplicate_part_seen = false;
        for system in original["systems"].as_array().unwrap() {
            let sid = system["system"].as_u64().unwrap();
            let parts=page.parts.iter().filter(|p|p.system==sid).map(|p|json!({"id":p.id,"staff_ids":p.staffs.iter().map(|s|s.id).collect::<Vec<_>>(),"bounds":p.bounds})).collect::<Vec<_>>();
            assert_eq!(json!(parts), system["parts"]);
            for case in system["cases"].as_array().unwrap() {
                let point = serde_json::from_value(case["point"].clone()).unwrap();
                let selected =
                    around(&page.staffs, &page.staff_areas, &page.parts, sid, point).unwrap();
                assert_eq!(
                    json!(selected.staffs.iter().map(|s| s.id).collect::<Vec<_>>()),
                    case["staff_ids"],
                    "{} {:?}",
                    case["kind"],
                    point
                );
                assert_eq!(
                    json!(selected.parts.iter().map(|p| p.id).collect::<Vec<_>>()),
                    case["part_ids"]
                );
                assert_eq!(json!(selected.part_bounds()), case["part_bounds"]);
                assert!(
                    selected
                        .parts
                        .iter()
                        .all(|p| page.parts.iter().any(|held| Arc::ptr_eq(p, held)))
                );
                duplicate_part_seen |= selected.parts.windows(2).any(|p| Arc::ptr_eq(&p[0], &p[1]));
            }
        }
        assert!(duplicate_part_seen);
        assert_eq!(page.graph.snapshot(), graph);
        assert_eq!(page.staffs.as_ptr(), staff_storage);
        assert_eq!(page.parts.as_ptr(), part_storage);
    }
}
