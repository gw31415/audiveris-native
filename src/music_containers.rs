// SPDX-License-Identifier: AGPL-3.0-or-later
//! AbstractNote.added/remove and AbstractChord.invalidateCache/remove ->
//! Staff notes and Measure chord registration, before voices are assigned.
//! The retained SIG is authoritative; do not prune copied members by a caller's
//! deletion list or keep stale chord geometry after partial member removal.
use crate::{
    music_measures::{Chord, ChordKind, Measure},
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
        let (class, member_class, member_phase) = if chord.kind == ChordKind::Head {
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
        // The actual typed relation, not a stale copied optional key, is the
        // original HeadChord.getStem authority after cache-owned removal.
        let stem_edges = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "ChordStemRelation" && e["source"] == chord.id)
            .collect::<Vec<_>>();
        if stem_edges.len() > 1 || (chord.kind == ChordKind::Rest && !stem_edges.is_empty()) {
            return Err("music_container_stem_relation_required".into());
        }
        let actual_stem = stem_edges
            .first()
            .map(|e| e["target"].as_u64().unwrap() as usize);
        if n["stem_id"] != json!(actual_stem) {
            return Err("music_container_stem_identity_required".into());
        }
        let stem = actual_stem
            .map(|id| {
                let node = nodes.get(&id).ok_or("music_container_live_stem_required")?;
                let key: Key = serde_json::from_value(node["origin"].clone())
                    .map_err(|_| "music_container_stem_origin_required")?;
                if node["kind"] != "StemInter"
                    || key.phase != Phase::Stem
                    || key.system != chord.key.system
                    || graph.id(key) != Some(id)
                {
                    return Err("music_container_typed_stem_required");
                }
                Ok(key)
            })
            .transpose()?;
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
        let geometry = if chord.kind == ChordKind::Head {
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
            kind: chord.kind,
            key: chord.key,
            id: chord.id,
            stem,
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
        "head_chord_ids":retained.iter().filter(|c|c.measure==i&&c.kind == ChordKind::Head).map(|c|c.id).collect::<Vec<_>>(),
        "rest_chord_ids":retained.iter().filter(|c|c.measure==i&&c.kind == ChordKind::Rest).map(|c|c.id).collect::<Vec<_>>()
    })).collect::<Vec<_>>();
    *chords = retained;
    Ok(Refresh {
        removed_chords,
        projection: json!({"staff_notes":staff_notes,
        "measure_chords":measure_chords,"scope":"actual pre-rhythm note/chord containers; voices/lyrics/full Part cleanup pending"}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GrayImage,
        beam_dispatch::{self, Registry},
        stem_cache_sig::Retriever,
        stem_checks,
    };
    use std::sync::Arc;

    #[test]
    fn original_post_chord_stem_removal_keeps_head_class_geometry_and_canonical_cache() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-chord-geometry/post-stem-removal.json"
        ))
        .unwrap();
        // Actual image-backed glyph registry and retriever, not cache hydration.
        // No HeadStem edge: the saved original control declares that boundary.
        let glyph = beam_dispatch::glyph_from_json(&json!({
            "bounds":oracle["stem_bounds"],"orientation":"VERTICAL",
            "runs":[[0,0,58],[1,0,58]]
        }))
        .unwrap();
        let mut image = GrayImage {
            width: 400,
            height: 400,
            pixels: vec![255; 400 * 400],
        };
        for run in &glyph.runs {
            for y in run.start..run.start + run.length {
                image.pixels[(glyph.top as usize + y) * 400 + glyph.left as usize + run.position] =
                    0;
            }
        }
        let mut registry = Registry::default();
        let mut cache = Retriever::new(1).unwrap();
        let params = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
        let stem = cache
            .create_selected(&mut registry, std::slice::from_ref(&glyph), &image, &params)
            .unwrap()
            .stem
            .unwrap();
        let mut graph = Graph::default();
        let stem_id = cache.register(&registry, &mut graph, &stem).unwrap();
        let head = Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        };
        let head_id = graph
            .vertex(
                head,
                json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
            "bounds":oracle["head_bounds"],"staff_id":1,"grade":0.6,"contextual_grade":0.75}),
                false,
            )
            .unwrap();
        let chord = Key {
            system: 1,
            phase: Phase::Chord,
            local: 1,
        };
        let chord_id = graph
            .vertex(
                chord,
                json!({"kind":"HeadChordInter","staff_id":1,
            "part_id":1,"measure_index":0}),
                false,
            )
            .unwrap();
        let stem_key = Key {
            system: 1,
            phase: Phase::Stem,
            local: stem.glyph_id(),
        };
        graph
            .relations(
                1,
                Phase::Chord,
                vec![
                    (chord, head, json!({"kind":"Containment"})),
                    (chord, stem_key, json!({"kind":"ChordStemRelation"})),
                ],
            )
            .unwrap();
        let measures = vec![Measure {
            system: 1,
            part: 1,
            staff: 1,
            index: 0,
            left: 100,
            right: 350,
        }];
        let mut chords = vec![Chord {
            kind: ChordKind::Head,
            key: chord,
            id: chord_id,
            stem: Some(stem_key),
            members: vec![],
            measure: 0,
            bounds: [0; 4],
        }];
        let compare = |graph: &Graph, stage: &str| {
            let snapshot = graph.snapshot();
            let nodes = snapshot["inters"].as_array().unwrap();
            let c = nodes.iter().find(|n| n["id"] == chord_id).unwrap();
            let expected = &oracle[stage];
            for (actual, original) in [
                ("kind", "chord_kind"),
                ("bounds", "bounds"),
                ("grade", "grade"),
                ("head_location", "head_location"),
                ("tail_location", "tail_location"),
            ] {
                if matches!(actual, "head_location" | "tail_location") {
                    let point: [f64; 2] = serde_json::from_value(c[actual].clone()).unwrap();
                    let expected_point: [f64; 2] =
                        serde_json::from_value(expected[original].clone()).unwrap();
                    assert_eq!(point, expected_point, "{stage}:{actual}");
                } else {
                    assert_eq!(c[actual], expected[original], "{stage}:{actual}");
                }
            }
            assert_eq!(c["member_ids"], json!([head_id]));
            assert_eq!(
                c["stem_id"],
                if stage == "before" {
                    json!(stem_id)
                } else {
                    Value::Null
                }
            );
            assert_eq!(
                nodes.len(),
                expected["live_vertices"].as_u64().unwrap() as usize
            );
            assert_eq!(
                snapshot["relations"].as_array().unwrap().len(),
                expected["live_relations"].as_u64().unwrap() as usize
            );
        };
        compare(&graph, "before");
        refresh(&graph, &measures, &mut chords).unwrap();
        let registry_before = registry.clone();
        let cache_before = stem.snapshot().unwrap();
        // A failing callback remains atomic across both Graph and canonical Arc.
        let mut invalid = graph.clone();
        let mut bad_head = invalid.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == head_id)
            .unwrap()
            .clone();
        bad_head["contextual_grade"] = json!("invalid");
        invalid.vertex(head, bad_head, false).unwrap();
        let invalid_before = invalid.snapshot();
        assert!(cache.remove(&mut invalid, &stem).is_err());
        assert_eq!(invalid.snapshot(), invalid_before);
        assert_eq!(stem.snapshot().unwrap(), cache_before);
        cache.remove(&mut graph, &stem).unwrap();
        compare(&graph, "after");
        let refreshed = refresh(&graph, &measures, &mut chords).unwrap();
        assert!(refreshed.removed_chords.is_empty());
        assert_eq!(chords[0].kind, ChordKind::Head);
        assert_eq!(chords[0].stem, None);
        assert_eq!(json!(chords[0].bounds), oracle["after"]["bounds"]);
        assert_eq!(
            refreshed.projection["measure_chords"][0]["head_chord_ids"],
            json!([chord_id])
        );
        assert_eq!(
            refreshed.projection["measure_chords"][0]["rest_chord_ids"],
            json!([])
        );
        let after = graph.snapshot();
        cache.remove(&mut graph, &stem).unwrap();
        assert_eq!(graph.snapshot(), after);
        image.pixels.fill(255);
        let cached = cache
            .create_selected(&mut registry, &[glyph], &image, &params)
            .unwrap();
        assert!(cached.cache_hit);
        let cached = cached.stem.unwrap();
        assert!(Arc::ptr_eq(&stem, &cached));
        assert_eq!(cached.snapshot().unwrap()["removed"], true);
        assert_eq!(cached.snapshot().unwrap()["id"], stem_id);
        // Physical registry unchanged: retain the original content registration.
        assert_eq!(format!("{registry:?}"), format!("{registry_before:?}"));
    }
}
