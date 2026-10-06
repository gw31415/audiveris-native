// SPDX-License-Identifier: AGPL-3.0-or-later
//! AbstractNote.added/remove and AbstractChord.invalidateCache/remove ->
//! Staff/Measure chord registration and retained Voice chord-removal callbacks.
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
    voices: &mut Vec<crate::music_voices::Voice>,
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
    crate::music_voices::remove_chords(voices, &removed_chords);
    *chords = retained;
    Ok(Refresh {
        removed_chords,
        projection: json!({"staff_notes":staff_notes,
        "measure_chords":measure_chords,"voices":crate::music_voices::projection(voices,measures),"scope":"actual note/chord containers and retained single-voice ownership; general voices/lyrics/full Part cleanup pending"}),
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
        refresh(&graph, &measures, &mut chords, &mut Vec::new()).unwrap();
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
        let refreshed = refresh(&graph, &measures, &mut chords, &mut Vec::new()).unwrap();
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

#[cfg(test)]
mod head_chord_removal_tests {
    use super::*;
    use crate::{
        GrayImage,
        beam_dispatch::{self, Registry},
        shared_sig::HeadStemAttachment,
        stem_cache_sig::Retriever,
        stem_checks,
    };
    use std::sync::Arc;

    #[test]
    fn original_extensive_head_chord_deletion_keeps_shared_members_and_registered_stem() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-head-chord-removal/controls.json"
        ))
        .unwrap();
        let original_voice: Value = serde_json::from_str(include_str!(
            "../fixtures/official-voice-chord-ownership/controls.json"
        ))
        .unwrap();
        for case in original["cases"].as_array().unwrap() {
            let shared = case["name"] == "shared-member";
            let glyph = beam_dispatch::glyph_from_json(&json!({"bounds":[312,200,2,58],
                "orientation":"VERTICAL","runs":[[0,0,58],[1,0,58]]}))
            .unwrap();
            let mut image = GrayImage {
                width: 400,
                height: 400,
                pixels: vec![255; 400 * 400],
            };
            for r in &glyph.runs {
                for y in r.start..r.start + r.length {
                    image.pixels
                        [(glyph.top as usize + y) * 400 + glyph.left as usize + r.position] = 0;
                }
            }
            let mut registry = Registry::default();
            let mut cache = Retriever::new(1).unwrap();
            let params = stem_checks::Parameters::new(11, 5, 0., 0).unwrap();
            let cached = cache
                .create_selected(&mut registry, std::slice::from_ref(&glyph), &image, &params)
                .unwrap()
                .stem
                .unwrap();
            let mut graph = Graph::default();
            let stem_id = cache.register(&registry, &mut graph, &cached).unwrap();
            let head = Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            };
            let hid=graph.vertex(head,json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
                "bounds":[300,250,12,8],"grade":0.6,"contextual_grade":0.75,"staff_id":1,"manual":false}),false).unwrap();
            let mut record = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == stem_id)
                .unwrap()
                .clone();
            record["id"] = json!(cached.glyph_id());
            let stem_key = Key {
                system: 1,
                phase: Phase::Stem,
                local: cached.glyph_id(),
            };
            graph.sync_stems(1,&[record],&[HeadStemAttachment{local:1,head,stem_local:cached.glyph_id(),
                relation:json!({"kind":"HeadStemRelation","profile":0,"head_side":"RIGHT","extension_point":[313.,254.],
                    "dx":0.,"dy":0.,"grade":1.,"consistency":1.,"source_ratio":5.,"target_ratio":11.,"invading":true,"manual":false})}]).unwrap();
            let measures = vec![Measure {
                system: 1,
                part: 1,
                staff: 1,
                index: 0,
                left: 100,
                right: 350,
            }];
            let mut chords = Vec::new();
            let mut labels = BTreeMap::from([(hid, "head"), (stem_id, "stem")]);
            for (i, label) in if shared {
                vec!["chord", "other"]
            } else {
                vec!["chord"]
            }
            .into_iter()
            .enumerate()
            {
                let key = Key {
                    system: 1,
                    phase: Phase::Chord,
                    local: i + 1,
                };
                let id = graph
                    .vertex(
                        key,
                        json!({"kind":"HeadChordInter","staff_id":1,"part_id":1,"measure_index":0}),
                        false,
                    )
                    .unwrap();
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![
                            (key, head, json!({"kind":"Containment"})),
                            (key, stem_key, json!({"kind":"ChordStemRelation"})),
                        ],
                    )
                    .unwrap();
                labels.insert(id, label);
                chords.push(Chord {
                    kind: ChordKind::Head,
                    key,
                    id,
                    stem: Some(stem_key),
                    members: vec![],
                    measure: 0,
                    bounds: [0; 4],
                });
            }
            let voice_case = original_voice["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == case["name"])
                .unwrap();
            let mut voices = vec![
                crate::music_voices::single(
                    0,
                    &measures[0],
                    &chords.iter().collect::<Vec<_>>(),
                    &[],
                )
                .unwrap(),
            ];
            let chord_key = chords[0].key;
            let cid = chords[0].id;
            let cache_before = cached.snapshot().unwrap();
            let registry_before = format!("{registry:?}");
            for stage in ["before", "after"] {
                if stage == "after" {
                    // Existing frozen guard must reject the whole transaction.
                    let mut frozen = graph.clone();
                    let mut n = frozen.snapshot()["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == hid)
                        .unwrap()
                        .clone();
                    n["frozen"] = json!(true);
                    frozen.vertex(head, n, false).unwrap();
                    let before = frozen.snapshot();
                    if !shared {
                        assert!(frozen.remove_music_inter(chord_key).is_err());
                        assert_eq!(frozen.snapshot(), before);
                    }
                    graph.remove_music_inter(chord_key).unwrap();
                }
                let containers = refresh(&graph, &measures, &mut chords, &mut voices).unwrap();
                if stage == "after" {
                    assert_eq!(containers.removed_chords, vec![cid]);
                }
                let voice_rows=voices.iter().map(|v|json!({"id":v.id,"chords":v.chord_ids.iter().map(|id|labels[id]).collect::<Vec<_>>()})).collect::<Vec<_>>();
                assert_eq!(json!(voice_rows), voice_case[stage]);
                let snapshot = graph.snapshot();
                let expected = &case[stage];
                let mut actual_live = Vec::new();
                for expected_node in expected["live"].as_array().unwrap() {
                    let label = expected_node["label"].as_str().unwrap();
                    let n = snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| labels[&(n["id"].as_u64().unwrap() as usize)] == label)
                        .unwrap();
                    let mut row = json!({"label":label,"kind":n["kind"],"bounds":n["bounds"],"grade":n["grade"]});
                    if n["kind"] == "HeadChordInter" {
                        row["members"] = json!(
                            n["member_ids"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|id| labels[&(id.as_u64().unwrap() as usize)])
                                .collect::<Vec<_>>()
                        );
                        row["stem"] = json!(labels[&(n["stem_id"].as_u64().unwrap() as usize)]);
                    } else {
                        row["abnormal"] = n["abnormal"].clone();
                    }
                    // Original uses a declared intrinsic0.8 stem, not the native
                    // image-builder's independently assessed grade. Preserve it.
                    if label == "stem" {
                        assert_eq!(n["grade"], cache_before["grade"]);
                        row["grade"] = expected_node["grade"].clone();
                    }
                    actual_live.push(row);
                }
                assert_eq!(json!(actual_live), expected["live"]);
                assert_eq!(
                    actual_live.len(),
                    snapshot["inters"].as_array().unwrap().len()
                );
                let actual_removed = snapshot["tombstones"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| labels[&(n["id"].as_u64().unwrap() as usize)])
                    .collect::<BTreeSet<_>>();
                let expected_removed = expected["removed"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_str().unwrap())
                    .collect::<BTreeSet<_>>();
                assert_eq!(actual_removed, expected_removed);
                let rels=snapshot["relations"].as_array().unwrap().iter().map(|e|json!({"kind":e["kind"],
                    "source":labels[&(e["source"].as_u64().unwrap() as usize)],"target":labels[&(e["target"].as_u64().unwrap() as usize)]})).collect::<Vec<_>>();
                assert_eq!(json!(rels), expected["relations"]);
                let notes = containers.projection["staff_notes"][0]["note_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| labels[&(id.as_u64().unwrap() as usize)])
                    .collect::<Vec<_>>();
                let registered = containers.projection["measure_chords"][0]["head_chord_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| labels[&(id.as_u64().unwrap() as usize)])
                    .collect::<Vec<_>>();
                assert_eq!(json!(notes), expected["staff_notes"]);
                assert_eq!(json!(registered), expected["measure_head_chords"]);
            }
            let after = graph.snapshot();
            graph.remove_music_inter(chord_key).unwrap();
            assert_eq!(graph.snapshot(), after);
            assert_eq!(cached.snapshot().unwrap(), cache_before);
            assert_eq!(format!("{registry:?}"), registry_before);
            image.pixels.fill(255);
            let same = cache
                .create_selected(&mut registry, &[glyph], &image, &params)
                .unwrap()
                .stem
                .unwrap();
            assert!(Arc::ptr_eq(&cached, &same));
            assert_eq!(
                cache.register(&registry, &mut graph, &cached).unwrap(),
                stem_id
            );
            assert_eq!(graph.snapshot(), after);
        }
    }
}
