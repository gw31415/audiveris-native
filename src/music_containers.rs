// SPDX-License-Identifier: AGPL-3.0-or-later
//! AbstractNote.added/remove and AbstractChord.invalidateCache/remove ->
//! Staff notes and Measure chord registration, before voices are assigned.
//! The retained SIG is authoritative; do not prune copied members by a caller's
//! deletion list or keep stale chord geometry after partial member removal.
use crate::{
    music_measures::{Chord, Measure},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Refresh {
    pub removed_chords: Vec<usize>,
    pub projection: Value,
}
pub(crate) fn refresh(
    graph: &Graph,
    measures: &[Measure],
    chords: &mut Vec<Chord>,
) -> Result<Refresh, String> {
    let snapshot = graph.snapshot();
    let nodes: BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let removed: BTreeSet<_> = snapshot["tombstones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_u64().unwrap() as usize)
        .collect();
    let mut retained = Vec::new();
    let mut removed_chords = Vec::new();
    let mut seen = BTreeSet::new();
    for chord in chords.iter() {
        if chord.key.phase != Phase::Chord
            || graph.id(chord.key) != Some(chord.id)
            || !seen.insert(chord.id)
        {
            return Err("music_container_chord_identity_required".into());
        }
        let Some(n) = nodes.get(&chord.id) else {
            if !removed.contains(&chord.id) {
                return Err("music_container_chord_tombstone_required".into());
            }
            removed_chords.push(chord.id);
            continue;
        };
        let (class, member_class, member_phase) = if chord.stem.is_some() {
            ("HeadChordInter", "HeadInter", Phase::Head)
        } else {
            ("RestChordInter", "RestInter", Phase::Symbol)
        };
        let measure = measures
            .get(chord.measure)
            .ok_or("music_container_measure_required")?;
        let origin: Key = serde_json::from_value(n["origin"].clone())
            .map_err(|_| "music_container_origin_required")?;
        if n["kind"] != class
            || origin != chord.key
            || measure.system != chord.key.system
            || n["part_id"] != measure.part
            || n["staff_id"] != measure.staff
            || n["measure_index"] != measure.index
        {
            return Err("music_container_owned_measure_required".into());
        }
        let stem_id = chord
            .stem
            .map(|key| graph.id(key).ok_or("music_container_stem_required"))
            .transpose()?;
        if n["stem_id"] != json!(stem_id) {
            return Err("music_container_stem_identity_required".into());
        }
        let actual_stem = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "ChordStemRelation" && e["source"] == chord.id)
            .map(|e| e["target"].as_u64().unwrap() as usize);
        if actual_stem != stem_id {
            return Err("music_container_stem_relation_required".into());
        }
        let stem_node = actual_stem
            .map(|id| {
                nodes
                    .get(&id)
                    .copied()
                    .ok_or("music_container_live_stem_required")
            })
            .transpose()?;
        let mut members = Vec::new();
        for id in snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == chord.id)
            .map(|e| e["target"].as_u64().unwrap() as usize)
        {
            let member = nodes
                .get(&id)
                .ok_or("music_container_live_member_required")?;
            let key: Key = serde_json::from_value(member["origin"].clone())
                .map_err(|_| "music_container_member_origin_required")?;
            if member["kind"] != member_class
                || key.phase != member_phase
                || key.system != chord.key.system
                || graph.id(key) != Some(id)
            {
                return Err("music_container_typed_member_required".into());
            }
            members.push((*member).clone());
        }
        // Removing the last vertex kills its ensemble. A live, empty chord
        // from direct relation editing needs a later original editing/cleaner
        // path; do not silently invent a vertex deletion here.
        if members.is_empty() {
            return Err("music_container_live_empty_chord_pending".into());
        }
        let member_refs = members.iter().collect::<Vec<_>>();
        let geometry = if chord.stem.is_some() {
            crate::head_chord_geometry::calculate(&member_refs, stem_node)?
        } else {
            crate::head_chord_geometry::calculate_rest(&member_refs)?
        };
        members.sort_by_key(|n| {
            std::cmp::Reverse(
                n["bounds"][1].as_i64().unwrap() + n["bounds"][3].as_i64().unwrap() / 2,
            )
        });
        if n["member_ids"] != json!(members.iter().map(|n| n["id"].clone()).collect::<Vec<_>>())
            || n["bounds"] != json!(geometry.bounds)
        {
            return Err("music_container_stale_chord_geometry_or_members".into());
        }
        let bounds = geometry
            .bounds
            .ok_or("music_container_chord_bounds_required")?;
        retained.push(Chord {
            key: chord.key,
            id: chord.id,
            stem: chord.stem,
            members,
            measure: chord.measure,
            bounds,
        });
    }
    let mut staff_notes = Vec::new();
    let mut staffs = BTreeSet::new();
    for m in measures {
        if !staffs.insert((m.system, m.staff)) {
            continue;
        }
        let ids = nodes
            .iter()
            .filter_map(|(id, n)| {
                if !matches!(n["kind"].as_str(), Some("HeadInter" | "RestInter"))
                    || n["staff_id"] != m.staff
                {
                    return None;
                }
                let key: Key = serde_json::from_value(n["origin"].clone()).ok()?;
                (key.system == m.system).then_some(*id)
            })
            .collect::<Vec<_>>();
        staff_notes.push(json!({"system_id":m.system,"staff_id":m.staff,"note_ids":ids}));
    }
    let measure_chords = measures.iter().enumerate().map(|(i,m)| json!({
        "system_id":m.system,"part_id":m.part,"staff_id":m.staff,"measure_index":m.index,
        "head_chord_ids":retained.iter().filter(|c|c.measure==i&&c.stem.is_some()).map(|c|c.id).collect::<Vec<_>>(),
        "rest_chord_ids":retained.iter().filter(|c|c.measure==i&&c.stem.is_none()).map(|c|c.id).collect::<Vec<_>>()
    })).collect::<Vec<_>>();
    *chords = retained;
    Ok(Refresh {
        removed_chords,
        projection: json!({"staff_notes":staff_notes,
        "measure_chords":measure_chords,"scope":"actual pre-rhythm note/chord containers; voices/lyrics/full Part cleanup pending"}),
    })
}
