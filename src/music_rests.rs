// SPDX-License-Identifier: AGPL-3.0-or-later
//! RestInter.createValid and the subsequent per-system ChordsBuilder pass.
use crate::{
    beam_dispatch,
    grid_ir::{StaffGeometry, SystemGeometry},
    music_measure_layout::MeasureLayout,
    music_measures::{Chord, ChordKind, OwnedMusic},
    shared_sig::{Graph, Key, Phase},
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) struct CreatedRest {
    pub(crate) staff: Arc<StaffGeometry>,
    pub(crate) measure: Arc<MeasureLayout>,
    pub(crate) measure_index: usize,
    pub(crate) body: Value,
}

pub(crate) fn create(
    music: &OwnedMusic,
    system: &SystemGeometry,
    glyph: &GlyphPixels,
    shape: &str,
    grade: f64,
    glyph_id: usize,
) -> Result<Option<CreatedRest>, String> {
    let centroid = beam_dispatch::centroid(glyph)?.map(|v| v as f64);
    let Some(staff) = crate::music_symbols::geometry::closest(&music.owner, system, centroid)?
    else {
        return Ok(None);
    };
    if staff.lines.len() != 5 {
        return Err("music_rest_special_staff_pending".into());
    }
    let part = music
        .measure_layout
        .parts
        .iter()
        .find(|p| p.part.staffs.iter().any(|s| Arc::ptr_eq(s, &staff)))
        .ok_or("music_rest_actual_Part_required")?;
    let Some(measure) = part.measure_at(&music.owner.staffs, &music.owner.staff_areas, centroid)?
    else {
        return Ok(None);
    };
    let measure_index = music
        .measures
        .iter()
        .position(|m| {
            m.geometry
                .as_ref()
                .is_some_and(|g| Arc::ptr_eq(g, &measure))
        })
        .ok_or("music_rest_actual_Measure_required")?;
    let [left, right] = measure.abscissae(&staff)?;
    let bounds = [
        glyph.left as i64,
        glyph.top as i64,
        glyph.width as i64,
        glyph.height as i64,
    ];
    let min_dx = (music.owner.interline as f64 * 0.5).round_ties_even() as i64;
    for chord in music
        .chords
        .iter()
        .filter(|c| c.kind == ChordKind::Head && c.key.system == system.id)
    {
        let (node, removed) = music.owner.graph.retained_inter(chord.key)?;
        // The original predicate uses the Chord's Staff and its center, not
        // the Part's top Staff or a previously assigned Measure index.
        if removed || node["staff_id"] != staff.id {
            continue;
        }
        let x = chord.bounds[0] + chord.bounds[2] / 2;
        if x > left
            && x <= right
            && crate::music_symbols::rest_too_close_to_chord(bounds, min_dx, chord)
        {
            return Ok(None);
        }
    }
    let pitch = music.owner.ledger_memberships.pitch_at(&staff, centroid)?;
    Ok(Some(CreatedRest {
        body: json!({"kind":"RestInter","shape":shape,"bounds":bounds,
            "glyph":beam_dispatch::glyph_json(glyph),"glyph_id":glyph_id,
            "staff_id":staff.id,"grade":grade,"pitch":pitch,"manual":false,"frozen":false,"abnormal":false}),
        staff,
        measure,
        measure_index,
    }))
}

fn chord_measure(
    music: &OwnedMusic,
    system: u64,
    rest_staff: &Arc<StaffGeometry>,
    point: [f64; 2],
) -> Result<usize, String> {
    let around = crate::grid_staff_neighborhood::around(
        &music.owner.staffs,
        &music.owner.staff_areas,
        &music.owner.parts,
        system,
        point,
    )?;
    let Some(above) = around.staffs.first() else {
        return Err("music_rest_nullable_Chord_Measure_pending".into());
    };
    for stack in music
        .measure_layout
        .stacks
        .iter()
        .filter(|s| s.stack.system == system)
    {
        let Some(reference) = stack
            .measures
            .iter()
            .find(|g| g.part.staffs.iter().any(|s| Arc::ptr_eq(s, above)))
        else {
            continue;
        };
        let [left, right] = reference.abscissae(above)?;
        if !(point[0] > left as f64 && point[0] <= right as f64) {
            continue;
        }
        let member = stack
            .measures
            .iter()
            .find(|g| g.part.staffs.iter().any(|s| Arc::ptr_eq(s, rest_staff)))
            .ok_or("music_rest_actual_Stack_Part_Measure_required")?;
        return music
            .measures
            .iter()
            .position(|m| m.geometry.as_ref().is_some_and(|g| Arc::ptr_eq(g, member)))
            .ok_or("music_rest_actual_Chord_Measure_required".into());
    }
    Err("music_rest_nullable_Chord_Measure_pending".into())
}

pub(crate) fn add_rest_chords(
    graph: &mut Graph,
    music: &OwnedMusic,
    system_id: u64,
) -> Result<Vec<Chord>, String> {
    let system = music
        .owner
        .systems
        .iter()
        .find(|s| s.id == system_id)
        .ok_or("music_rest_actual_System_required")?;
    let mut next = graph.clone();
    let rest_keys = graph.live_inter_keys(system_id, "RestInter");
    let initial = graph.snapshot();
    let mut local = initial["inters"]
        .as_array()
        .unwrap()
        .iter()
        .chain(initial["tombstones"].as_array().unwrap())
        .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
        .filter(|k| k.system == system_id && k.phase == Phase::Chord)
        .map(|k| k.local)
        .max()
        .unwrap_or(0);
    let mut chords = Vec::new();
    for rest in rest_keys {
        let body = next.retained_inter(rest)?.0.clone();
        let id = next.id(rest).ok_or("music_rest_actual_Rest_required")?;
        if initial["relations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["kind"] == "Containment" && r["target"] == id)
        {
            return Err("music_rest_chords_already_built".into());
        }
        let bounds: [i64; 4] = serde_json::from_value(body["bounds"].clone())
            .map_err(|_| "music_rest_bounds_required")?;
        let point = [
            (bounds[0] + bounds[2] / 2) as f64,
            (bounds[1] + bounds[3] / 2) as f64,
        ];
        let staff = music
            .owner
            .staffs
            .iter()
            .find(|s| s.system == system_id && body["staff_id"] == s.id)
            .ok_or("music_rest_actual_Staff_required")?;
        let center_staff = crate::music_symbols::geometry::closest(&music.owner, system, point)?;
        // RestChord.setStaff uses the integer center independently of the
        // factory's centroid. A null source must remain null.
        let chord_staff = center_staff.as_ref().map(|s| s.id);
        let measure = chord_measure(music, system_id, staff, point)?;
        local = local
            .checked_add(1)
            .ok_or("music_rest_chord_identity_budget")?;
        let key = Key {
            system: system_id,
            phase: Phase::Chord,
            local,
        };
        let chord_id = next.vertex(key, json!({"kind":"RestChordInter","bounds":null,"staff_id":chord_staff,
            "part_id":null,"measure_index":null,"member_ids":[],"grade":null,"duration":null,"voice":null,"manual":false,"frozen":false,"abnormal":false}), false)?;
        next.relations(
            system_id,
            Phase::Chord,
            vec![(key, rest, json!({"kind":"Containment"}))],
        )?;
        let mut chord_body = next.retained_inter(key)?.0.clone();
        chord_body["staff_id"] = json!(chord_staff);
        chord_body["part_id"] = json!(music.measures[measure].part);
        chord_body["measure_index"] = json!(music.measures[measure].index);
        next.vertex(key, chord_body, false)?;
        chords.push(Chord {
            kind: ChordKind::Rest,
            key,
            id: chord_id,
            stem: None,
            members: vec![body],
            measure,
            bounds,
        });
    }
    #[cfg(test)]
    tests::record(system_id, initial, next.snapshot(), graph);
    *graph = next;
    Ok(chords)
}

#[cfg(test)]
mod tests;
