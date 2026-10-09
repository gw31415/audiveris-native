// SPDX-License-Identifier: AGPL-3.0-or-later
//! Nullable Arpeggiato factory lookup against current same-System HeadChords.
use crate::{
    music_measures::{ChordKind, OwnedMusic},
    shared_sig::{Graph, Key},
};

#[derive(Clone, Debug)]
struct Candidate {
    key: Key,
    bounds: [i64; 4],
    heads: [i64; 4],
}
struct Lookup {
    link: Option<(Key, f64)>,
    #[cfg(test)]
    area: [i64; 4],
    #[cfg(test)]
    head_chord_count: usize,
    #[cfg(test)]
    nearby: Vec<Candidate>,
}
fn rectangle(bounds: [i64; 4]) -> Result<[i64; 4], String> {
    if bounds[2] <= 0 || bounds[3] <= 0 || bounds.iter().any(|v| v.unsigned_abs() > 4_000_000) {
        return Err("music_arpeggiato_positive_bounds_required".into());
    }
    Ok(bounds)
}
fn lookup_area(bounds: [i64; 4], interline: usize, profile: u32) -> Result<[i64; 4], String> {
    let bounds = rectangle(bounds)?;
    if interline == 0 || interline > 10000 {
        return Err("music_arpeggiato_scale_required".into());
    }
    // ConstantSet falls back through lower profiles; only p1 overrides p0.
    let maximum = if profile == 0 { 1.5 } else { 2.5 };
    Ok([
        bounds[0] + bounds[2],
        bounds[1],
        (maximum * interline as f64).round_ties_even() as i64,
        bounds[3],
    ])
}
fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[2] > 0
        && a[3] > 0
        && b[2] > 0
        && b[3] > 0
        && a[0] < b[0] + b[2]
        && b[0] < a[0] + a[2]
        && a[1] < b[1] + b[3]
        && b[1] < a[1] + a[3]
}
fn y_overlap(a: [i64; 4], b: [i64; 4]) -> i64 {
    (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1])
}
fn select(area: [i64; 4], candidates: &[Candidate]) -> Option<(Key, f64)> {
    let mut best = None;
    let mut best_overlap = 0;
    for candidate in candidates {
        if !intersects(candidate.bounds, area) || !intersects(candidate.heads, area) {
            continue;
        }
        let overlap = y_overlap(candidate.heads, area);
        // Source keeps the first Chord in stable abscissa order on ties.
        if best_overlap < overlap {
            best = Some(candidate.key);
            best_overlap = overlap;
        }
    }
    let grade = best_overlap as f64 / area[3] as f64;
    best.filter(|_| grade >= 0.1).map(|key| (key, grade))
}
fn lookup(
    graph: &Graph,
    music: &OwnedMusic,
    system: u64,
    bounds: [i64; 4],
) -> Result<Lookup, String> {
    let actual_system = music
        .owner
        .systems
        .iter()
        .find(|s| s.id == system)
        .ok_or("music_arpeggiato_actual_System_required")?;
    let area = lookup_area(bounds, music.owner.interline, actual_system.profile)?;
    let mut chords = graph
        .live_inter_keys(system, "HeadChordInter")
        .into_iter()
        .map(|key| {
            let chord = music
                .chords
                .iter()
                .find(|c| c.key == key && c.kind == ChordKind::Head)
                .ok_or("music_arpeggiato_actual_HeadChord_required")?;
            if graph.id(key) != Some(chord.id) {
                return Err("music_arpeggiato_actual_HeadChord_required".into());
            }
            Ok((key, rectangle(graph.retained_or_live_chord_bounds(key)?)?))
        })
        .collect::<Result<Vec<_>, String>>()?;
    chords.sort_by_key(|(_, b)| b[0]);
    #[cfg(test)]
    let head_chord_count = chords.len();
    let heads = graph.live_inter_keys(system, "HeadInter");
    let mut nearby = Vec::new();
    for (key, chord_bounds) in chords {
        if !intersects(chord_bounds, area) {
            if chord_bounds[0] > area[0] + area[2] - 1 {
                break;
            }
            continue;
        }
        let members = heads
            .iter()
            .filter(|head| graph.has_relation(key, **head, "Containment"))
            .map(|head| graph.retained_inter(*head).map(|(body, _)| body))
            .collect::<Result<Vec<_>, _>>()?;
        // The retained Chord's current Containment owns these heads. A cached
        // report/member copy, or its stem-inclusive box, changes this lookup.
        let head_bounds = crate::head_chord_geometry::calculate(&members, None)?
            .bounds
            .ok_or("music_arpeggiato_actual_head_members_required")?;
        nearby.push(Candidate {
            key,
            bounds: chord_bounds,
            heads: head_bounds,
        });
    }
    let link = select(area, &nearby);
    Ok(Lookup {
        link,
        #[cfg(test)]
        area,
        #[cfg(test)]
        head_chord_count,
        #[cfg(test)]
        nearby,
    })
}

pub(crate) fn require_null_factory(
    graph: &Graph,
    music: &OwnedMusic,
    system: u64,
    bounds: [i64; 4],
    intrinsic_grade: f64,
) -> Result<(), String> {
    if !intrinsic_grade.is_finite() || !(0.0..=1.0).contains(&intrinsic_grade) {
        return Err("music_arpeggiato_grade_required".into());
    }
    #[cfg(test)]
    let before = tests::before(graph);
    let found = lookup(graph, music, system, bounds)?;
    #[cfg(test)]
    tests::record(
        graph,
        music,
        system,
        bounds,
        intrinsic_grade,
        &found,
        before,
    );
    if found.link.is_some() {
        // A positive factory adds its Inter and relation before weak removal.
        // Later missing Chords cannot turn that unported callback into null.
        return Err("musicxml_arpeggiato_chord_link_pending".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
