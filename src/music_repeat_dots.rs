// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{grid_ir::StaffGeometry, music_measures::OwnedMusic, shared_sig::Key};
use serde_json::{Value, json};
use std::sync::Arc;
pub(crate) struct Lookup {
    pub(crate) staff: Option<Arc<StaffGeometry>>,
    pub(crate) pitch: Option<f64>,
    pub(crate) pitch_allowed: bool,
    pub(crate) bar: Option<(Key, Value)>,
}
pub(crate) fn lookup(
    music: &OwnedMusic,
    system: u64,
    b: [i64; 4],
    profile: u32,
) -> Result<Lookup, String> {
    let (xmax, ymax) = match profile {
        0 => (1.5, 0.5),
        1 => (2.2, 0.75),
        _ => return Err("music_repeat_profile_pending".into()),
    };
    let system = music
        .owner
        .systems
        .iter()
        .find(|s| s.id == system)
        .ok_or("music_repeat_actual_System_required")?;
    let p = [
        b[0] as f64 + b[2] as f64 / 2.,
        b[1] as f64 + b[3] as f64 / 2.,
    ];
    let staff = crate::music_symbols::geometry::closest(&music.owner, system, p)?;
    let pitch = staff
        .as_ref()
        .map(|s| music.owner.ledger_memberships.pitch_at(s, p))
        .transpose()?;
    let pitch_allowed = pitch.is_some_and(|pp| (pp.abs() - 1.).abs() <= 2. * ymax);
    let il = music.owner.interline as f64;
    let dx = (il * xmax).round_ties_even() as i64;
    let dy = (il * ymax).round_ties_even() as i64;
    let lu = [
        p[0].round_ties_even() as i64 - dx,
        p[1].round_ties_even() as i64 - dy,
        2 * dx,
        2 * dy,
    ];
    let mut bars = music.owner.graph.live_inter_keys(system.id, "BarlineInter");
    bars.sort_by_key(|k| music.owner.graph.retained_inter(*k).unwrap().0["bounds"][0].as_i64());
    let mut best = None;
    for key in bars {
        let (n, _) = music.owner.graph.retained_inter(key)?;
        let cb: [i64; 4] = serde_json::from_value(n["bounds"].clone())
            .map_err(|_| "music_repeat_actual_Barline_bounds_required")?;
        if cb[0] >= lu[0] + lu[2]
            || lu[0] >= cb[0] + cb[2]
            || cb[1] >= lu[1] + lu[3]
            || lu[1] >= cb[1] + cb[3]
        {
            continue;
        }
        let median: [f64; 4] = serde_json::from_value(n["median"].clone())
            .map_err(|_| "music_repeat_actual_Barline_median_required")?;
        let width = n["width"]
            .as_f64()
            .filter(|w| w.is_finite() && *w > 0.)
            .ok_or("music_repeat_actual_Barline_width_required")?;
        if median[1] == median[3] {
            return Err("music_repeat_vertical_Barline_required".into());
        }
        let center = [
            cb[0] as f64 + cb[2] as f64 / 2.,
            cb[1] as f64 + cb[3] as f64 / 2.,
        ];
        let y = center[1] + cb[3] as f64 / 8. * if p[1] > center[1] { 1. } else { -1. };
        let x = median[0]
            + (median[2] - median[0]) * (y - median[1]) / (median[3] - median[1])
            + width / 2. * if p[0] > center[0] { 1. } else { -1. };
        let xgap = (x - p[0]).abs() / il;
        let ygap = (y - p[1]).abs() / il;
        let grade = crate::music_dots::gap_grade(xgap, ygap, xmax, ymax, 3., 1.);
        if grade >= 0.1 && best.as_ref().is_none_or(|(_, _, old)| xgap < *old) {
            best = Some((
                key,
                json!({"kind":"RepeatDotBarRelation","grade":grade,"dx":xgap,"dy":ygap,"source_ratio":1.+5.*grade,"target_ratio":1.,"manual":false}),
                xgap,
            ));
        }
    }
    Ok(Lookup {
        staff,
        pitch,
        pitch_allowed,
        bar: best.map(|(k, r, _)| (k, r)),
    })
}
pub(crate) fn instant_check(music: &OwnedMusic, system: u64, b: [i64; 4]) -> Result<(), String> {
    let profile = music
        .owner
        .systems
        .iter()
        .find(|s| s.id == system)
        .ok_or("music_repeat_actual_System_required")?
        .profile;
    let found = lookup(music, system, b, profile)?;
    if found.staff.is_some() && found.pitch.is_some() && found.pitch_allowed && found.bar.is_some()
    {
        return Err("music_repeat_positive_factory_pending".into());
    }
    Ok(())
}

#[cfg(test)]
mod image_tests;
#[cfg(test)]
pub(crate) use image_tests::assert_actual_lookup;
