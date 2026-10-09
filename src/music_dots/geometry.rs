// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    grid_ir::StaffGeometry, music_measures::OwnedMusic, shared_sig::Key,
    stem_image_owned::OwnedItems,
};
use std::sync::Arc;

pub(super) fn closest(
    owner: &OwnedItems,
    system: u64,
    bounds: [i64; 4],
) -> Result<Arc<StaffGeometry>, String> {
    let system = owner
        .systems
        .iter()
        .find(|s| s.id == system)
        .ok_or("music_dot_actual_System_required")?;
    crate::music_symbols::geometry::closest(owner, system, center(bounds))?
        .ok_or_else(|| "music_dot_actual_Staff_source_required".into())
}
fn center(b: [i64; 4]) -> [f64; 2] {
    [
        b[0] as f64 + b[2] as f64 / 2.,
        b[1] as f64 + b[3] as f64 / 2.,
    ]
}
pub(super) fn concrete_ledger(
    owner: &OwnedItems,
    staff: &Arc<StaffGeometry>,
    point: [f64; 2],
) -> Result<Option<Key>, String> {
    let pitch = owner.ledger_memberships.pitch_at(staff, point)?;
    let line = 2 * (pitch / 2.).round_ties_even() as i32;
    if line.unsigned_abs() <= staff.lines.len() as u32 {
        return Ok(None);
    }
    let index = (line - line.signum() * (staff.lines.len() as i32 - 1)) / 2;
    let map = owner
        .ledger_memberships
        .maps
        .iter()
        .find(|m| Arc::ptr_eq(&m.staff, staff))
        .ok_or("music_dot_actual_Ledger_map_required")?;
    let Some(ledgers) = map.lines.get(&index) else {
        return Ok(None);
    };
    let il = staff.specific_interline as f64;
    let mut best = None;
    for &key in ledgers {
        // Membership, rather than live-SIG filtering, is the lookup source. A
        // retained alias can still reference a removed body until its own reset.
        let (ledger, _) = owner.graph.retained_inter(key)?;
        let b: [i64; 4] = serde_json::from_value(ledger["bounds"].clone())
            .map_err(|_| "music_dot_actual_Ledger_bounds_required")?;
        if (b[0] as f64) < point[0] + il
            && point[0] - il < (b[0] + b[2]) as f64
            && (b[1] as f64) < point[1] + il
            && point[1] - il < (b[1] + b[3]) as f64
        {
            // Staff chooses with integer Inter center; DotFactory subsequently
            // measures against center2D of that selected concrete Ledger.
            let distance = ((b[1] + b[3] / 2) as f64 - point[1]).abs();
            if best.is_none_or(|(_, d)| distance < d) {
                best = Some((key, distance));
            }
        }
    }
    Ok(best.map(|(key, _)| key))
}
pub(super) fn distant(
    owner: &OwnedItems,
    staff: &Arc<StaffGeometry>,
    bounds: [i64; 4],
) -> Result<bool, String> {
    if staff.lines.len() != 5 || staff.specific_interline == 0 {
        return Err("music_dot_standard_actual_Staff_required".into());
    }
    let point = center(bounds);
    let pitch = owner.ledger_memberships.pitch_at(staff, point)?;
    let distance = if pitch.abs() <= staff.lines.len() as f64 {
        (pitch - 2. * (pitch / 2.).round_ties_even()).abs() / 2.
    } else if let Some(key) = concrete_ledger(owner, staff, point)? {
        let (ledger, _) = owner.graph.retained_inter(key)?;
        let b: [i64; 4] = serde_json::from_value(ledger["bounds"].clone())
            .map_err(|_| "music_dot_actual_Ledger_bounds_required")?;
        (point[1] - (b[1] as f64 + b[3] as f64 / 2.)).abs() / staff.specific_interline as f64
    } else {
        return Ok(true);
    };
    Ok(distance >= 0.3)
}
pub(super) fn measure_at(
    music: &OwnedMusic,
    staff: &Arc<StaffGeometry>,
    point: [i64; 2],
) -> Result<usize, String> {
    let part = music
        .measure_layout
        .parts
        .iter()
        .find(|p| p.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)))
        .ok_or("music_dot_actual_Part_required")?;
    let geometry = part
        .measure_at(
            &music.owner.staffs,
            &music.owner.staff_areas,
            point.map(|v| v as f64),
        )?
        .ok_or("music_dot_actual_Measure_source_required")?;
    music
        .measures
        .iter()
        .position(|m| {
            m.geometry
                .as_ref()
                .is_some_and(|g| Arc::ptr_eq(g, &geometry))
        })
        .ok_or_else(|| "music_dot_actual_Measure_required".into())
}
