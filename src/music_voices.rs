// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained Voice chord ownership in the admitted ordinary single-staff rhythm
//! path. VoiceKind HIGH is justified by the existing first-staff-only guard;
//! general voice matching/merged parts/slots/tie propagation remain pending.
use crate::music_measures::{Chord, Measure};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone)]
pub(crate) struct Voice {
    pub id: usize,
    pub measure: usize,
    pub starting_staff: u64,
    pub chord_ids: Vec<usize>,
}
/// Voice constructor -> Measure.generateVoiceId(HIGH), Voice.addChord. The
/// caller has already admitted a single voice in this nonmerged one-staff part.
pub(crate) fn single(
    measure_index: usize,
    measure: &Measure,
    chords: &[&Chord],
    existing: &[Voice],
) -> Result<Voice, String> {
    if chords.is_empty() || existing.iter().any(|v| v.measure == measure_index) {
        return Err("music_voice_first_assignment_required".into());
    }
    let mut members = Vec::new();
    let mut seen = BTreeSet::new();
    for chord in chords {
        if chord.measure != measure_index || chord.key.system != measure.system {
            return Err("music_voice_owned_measure_chord_required".into());
        }
        // Original addChord ignores an existing actual chord and stably sorts
        // by integer center abscissa; it does not order by registration ID.
        if seen.insert(chord.id) {
            members.push(*chord);
        }
    }
    members.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
    let id = (1..)
        .find(|id| {
            !existing
                .iter()
                .any(|v| v.measure == measure_index && v.id == *id)
        })
        .unwrap();
    Ok(Voice {
        id,
        measure: measure_index,
        starting_staff: measure.staff,
        chord_ids: members.iter().map(|c| c.id).collect(),
    })
}
/// Measure.removeVoiceChord -> Voice.removeChord. Remove an empty Voice only
/// when this removal actually removed its chord, matching the original loop.
pub(crate) fn remove_chords(voices: &mut Vec<Voice>, removed: &[usize]) {
    for id in removed {
        let mut index = 0;
        while index < voices.len() {
            if let Some(member) = voices[index].chord_ids.iter().position(|c| c == id) {
                voices[index].chord_ids.remove(member);
                if voices[index].chord_ids.is_empty() {
                    voices.remove(index);
                    break;
                }
            }
            index += 1;
        }
    }
}
pub(crate) fn projection(voices: &[Voice], measures: &[Measure]) -> Value {
    json!(
        voices
            .iter()
            .map(|v| {
                let m = &measures[v.measure];
                json!({"id":v.id,"kind":"HIGH","system_id":m.system,"part_id":m.part,
            "measure_index":m.index,"starting_staff_id":v.starting_staff,"chord_ids":v.chord_ids})
            })
            .collect::<Vec<_>>()
    )
}

// BeamGroupInter.getChords/getFirstChord/getVoice, from actual retained SIG.
// The caller is the native owner after voice assignment, never a serialized
// selected-chord API. General setVoice/group-rest/tie propagation is separate.
pub(crate) fn beam_groups(
    graph: &crate::shared_sig::Graph,
    voices: &[Voice],
    measures: &[Measure],
) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let nodes: std::collections::BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let edges = snapshot["relations"].as_array().unwrap();
    let mut rows = Vec::new();
    for group in nodes.values().filter(|n| n["kind"] == "BeamGroupInter") {
        let gid = group["id"].as_u64().unwrap() as usize;
        let system = group["origin"]["system"].as_u64().unwrap();
        let beams = edges
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == gid)
            .map(|e| {
                nodes
                    .get(&(e["target"].as_u64().unwrap() as usize))
                    .copied()
                    .ok_or("music_group_live_beam_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut chord_ids = Vec::new();
        let mut seen = BTreeSet::new();
        for beam in &beams {
            let mut beam_chords = Vec::new();
            let mut beam_seen = BTreeSet::new();
            if !matches!(beam["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                || beam["origin"]["system"] != system
            {
                return Err("music_group_typed_owned_beam_required".into());
            }
            // Original AbstractBeam collects all linked heads before rests;
            // stable center ties retain that order before the group left sort.
            let relations = edges
                .iter()
                .filter(|e| e["source"] == beam["id"] && e["kind"] == "BeamStemRelation")
                .chain(
                    edges
                        .iter()
                        .filter(|e| e["source"] == beam["id"] && e["kind"] == "BeamRestRelation"),
                );
            for relation in relations {
                let ids = match relation["kind"].as_str() {
                    Some("BeamStemRelation") => {
                        let stem = nodes
                            .get(&(relation["target"].as_u64().unwrap() as usize))
                            .ok_or("music_group_live_stem_required")?;
                        if stem["kind"] != "StemInter" || stem["origin"]["system"] != system {
                            return Err("music_group_typed_owned_stem_required".into());
                        }
                        edges
                            .iter()
                            .filter(|e| {
                                e["kind"] == "ChordStemRelation" && e["target"] == stem["id"]
                            })
                            .map(|e| (e["source"].as_u64().unwrap() as usize, "HeadChordInter"))
                            .collect::<Vec<_>>()
                    }
                    Some("BeamRestRelation") => {
                        // AbstractBeam.getChords follows the actual RestInter's
                        // first incoming Containment, not the beam's target as
                        // if it were already a RestChord.
                        let rest = nodes
                            .get(&(relation["target"].as_u64().unwrap() as usize))
                            .ok_or("music_group_live_rest_required")?;
                        if rest["kind"] != "RestInter" || rest["origin"]["system"] != system {
                            return Err("music_group_typed_owned_rest_required".into());
                        }
                        let parent = edges
                            .iter()
                            .find(|e| e["kind"] == "Containment" && e["target"] == rest["id"])
                            .ok_or("music_group_rest_ensemble_required")?;
                        vec![(
                            parent["source"].as_u64().unwrap() as usize,
                            "RestChordInter",
                        )]
                    }
                    _ => continue,
                };
                for (id, kind) in ids {
                    let chord = nodes.get(&id).ok_or("music_group_live_chord_required")?;
                    if chord["kind"] != kind || chord["origin"]["system"] != system {
                        return Err("music_group_typed_owned_chord_required".into());
                    }
                    if beam_seen.insert(id) {
                        beam_chords.push(id);
                    }
                }
            }
            // Each beam first sorts its head/rest chords by center. The group
            // then collects these lists and stably sorts by left abscissa.
            beam_chords.sort_by_key(|id| {
                nodes[id]["bounds"][0].as_i64().unwrap()
                    + nodes[id]["bounds"][2].as_i64().unwrap() / 2
            });
            for id in beam_chords {
                if seen.insert(id) {
                    chord_ids.push(id);
                }
            }
        }
        // Original getChords uses byAbscissa (left bounds), not the Voice's
        // center comparator. LinkedHashSet preserves ties and removes aliases.
        chord_ids.sort_by_key(|id| nodes[id]["bounds"][0].as_i64().unwrap());
        let owner = chord_ids
            .first()
            .map(|id| -> Result<Option<&Voice>, String> {
                let chord = nodes[id];
                if chord["voice"].is_null() {
                    return Ok(None);
                }
                let voice = voices
                    .iter()
                    .find(|v| {
                        v.chord_ids.contains(id)
                            && chord["voice"] == v.id
                            && measures[v.measure].system == system
                            && chord["part_id"] == measures[v.measure].part
                            && chord["measure_index"] == measures[v.measure].index
                    })
                    .ok_or("music_group_retained_voice_owner_required")?;
                Ok(Some(voice))
            })
            .transpose()?
            .flatten();
        rows.push(json!({"id":gid,"system_id":system,"bounds":group["bounds"],
            "beams":beams.iter().map(|n|json!({"id":n["id"],"kind":n["kind"],"bounds":n["bounds"]})).collect::<Vec<_>>(),
            "chords":chord_ids.iter().map(|id| {let n=nodes[id];json!({"id":id,"kind":n["kind"],"bounds":n["bounds"],"voice":n["voice"],"measure_number":n["measure_number"]})}).collect::<Vec<_>>(),
            "voice":owner.map(|v|v.id),"voice_measure_number":owner.map(|v|v.measure+1)}));
    }
    Ok(json!(rows))
}

/// AbstractChord.setVoice -> Voice.addChord -> BeamGroup.setVoice for the
/// currently admitted ordinary, untied chord family. Identity is the retained
/// Voice and its Measure, not a repeating numeric label. Unsupported tied or
/// multiple-group ownership keeps an explicit guard rather than dropping it.
pub(crate) fn assign_chord_voice(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measures: &[Measure],
    chord_id: usize,
    voice_index: usize,
) -> Result<(), String> {
    let voice = voices
        .get(voice_index)
        .ok_or("music_voice_owner_required")?;
    let measure = measures
        .get(voice.measure)
        .ok_or("music_voice_measure_required")?;
    let belongs = |n: &Value| {
        n["origin"]["system"] == measure.system
            && n["part_id"] == measure.part
            && n["measure_index"] == measure.index
    };
    let snapshot = graph.snapshot();
    let nodes: std::collections::BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    if !nodes.get(&chord_id).is_some_and(|n| belongs(n)) {
        return Err("music_voice_owned_measure_chord_required".into());
    }
    let groups = beam_groups(graph, voices, measures)?;
    let mut staged = graph.clone();
    let mut retained = voices.clone();
    let mut pending = vec![chord_id];
    while let Some(id) = pending.pop() {
        let original = nodes.get(&id).ok_or("music_voice_live_chord_required")?;
        if !matches!(
            original["kind"].as_str(),
            Some("HeadChordInter" | "RestChordInter")
        ) {
            return Err("music_voice_ordinary_chord_required".into());
        }
        let key: crate::shared_sig::Key = serde_json::from_value(original["origin"].clone())
            .map_err(|_| "music_voice_chord_origin_required")?;
        let current = staged.snapshot();
        let mut chord = current["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("music_voice_live_chord_required")?
            .clone();
        if chord["voice"] == voice.id {
            continue;
        }
        let members = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == id)
            .map(|e| e["target"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        if members.is_empty()
            || members.iter().any(|member| {
                nodes.get(member).is_none_or(|n| {
                    n["kind"] == "RestInter"
                        && !matches!(n["shape"].as_str(), Some("QUARTER_REST" | "HALF_REST"))
                })
            })
        {
            return Err("music_voice_measure_rest_special_chord_pending".into());
        }
        if snapshot["relations"].as_array().unwrap().iter().any(|e| {
            e["kind"] == "SlurHeadRelation"
                && members
                    .iter()
                    .any(|id| e["source"] == *id || e["target"] == *id)
        }) {
            return Err("music_voice_following_tied_chords_pending".into());
        }
        chord["voice"] = json!(voice.id);
        staged.vertex(key, chord, false)?;
        let owned = &mut retained[voice_index];
        if !owned.chord_ids.contains(&id) {
            owned.chord_ids.push(id);
            owned.chord_ids.sort_by_key(|id| {
                nodes[id]["bounds"][0].as_i64().unwrap()
                    + nodes[id]["bounds"][2].as_i64().unwrap() / 2
            });
        }
        let linked = groups
            .as_array()
            .unwrap()
            .iter()
            .filter(|g| {
                g["chords"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["id"] == id)
            })
            .collect::<Vec<_>>();
        if linked.len() > 1 {
            return Err("music_voice_multiple_group_order_pending".into());
        }
        if let Some(group) = linked.first() {
            // Same Measure object in original; native ownership is the exact
            // system/part/local measure tuple. Never copy Voice1 across it.
            for member in group["chords"].as_array().unwrap().iter().rev() {
                let other = member["id"].as_u64().unwrap() as usize;
                if belongs(nodes[&other]) {
                    pending.push(other);
                }
            }
        }
    }
    *graph = staged;
    *voices = retained;
    Ok(())
}

#[cfg(test)]
mod group_tests {
    use super::*;
    use crate::{
        GrayImage,
        shared_sig::{Key, Phase},
    };
    fn comparable(rows: Value) -> Value {
        json!(rows.as_array().unwrap().iter().map(|g| json!({
            "system_id":g["system_id"],"bounds":g["bounds"],"voice":g["voice"],"voice_measure_number":g["voice_measure_number"],
            "beams":g["beams"].as_array().unwrap().iter().map(|b|json!({"kind":b["kind"],"bounds":b["bounds"]})).collect::<Vec<_>>(),
            "chords":g["chords"].as_array().unwrap().iter().map(|c|json!({"kind":c["kind"],"bounds":c["bounds"],"voice":c["voice"],"measure_number":c["measure_number"]})).collect::<Vec<_>>()
        })).collect::<Vec<_>>())
    }
    #[test]
    fn original_saved_group_getters_and_declared_beam_rest_follow_actual_ensemble() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/saved-page.json"
        ))
        .unwrap();
        let rest_control: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/controlled-rest.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        let mut export = crate::music_xml::recognize(&image).unwrap();
        let owner = &mut export.owner;
        assert_eq!(
            comparable(owner.owner.prepared.report["beam_group_voices"].clone()),
            original["groups"]
        );
        let snapshot = owner.owner.graph.snapshot();
        let before_cache = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "StemInter" && n["origin"]["system"] == 1)
            .map(|n| {
                let key: Key = serde_json::from_value(n["origin"].clone()).unwrap();
                let stem = owner.owner.caches[&1]
                    .registered_stem(&owner.owner.graph, key.local)
                    .unwrap();
                let state = stem.snapshot().unwrap();
                (key, stem, state)
            })
            .collect::<Vec<_>>();
        let before_registry = format!("{:?}", owner.owner.prepared.registry);
        let beam: Key = serde_json::from_value(
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "BeamInter" && n["origin"]["system"] == 1)
                .unwrap()["origin"]
                .clone(),
        )
        .unwrap();
        let next_local = |phase: Phase| {
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
                .filter(|k| k.system == 1 && k.phase == phase)
                .map(|k| k.local)
                .max()
                .unwrap_or(0)
                + 1
        };
        let rest = Key {
            system: 1,
            phase: Phase::Symbol,
            local: next_local(Phase::Symbol),
        };
        let chord = Key {
            system: 1,
            phase: Phase::Chord,
            local: next_local(Phase::Chord),
        };
        let original_rest = rest_control["groups"][0]["chords"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == "RestChordInter")
            .unwrap();
        owner.owner.graph.vertex(rest,json!({"kind":"RestInter","shape":"QUARTER_REST","bounds":original_rest["bounds"],"grade":0.8,"staff_id":1}),false).unwrap();
        owner.owner.graph.vertex(chord,json!({"kind":"RestChordInter","staff_id":1,"part_id":1,"measure_index":0,"measure_number":1,"voice":null}),false).unwrap();
        owner
            .owner
            .graph
            .relations(
                1,
                Phase::Chord,
                vec![(chord, rest, json!({"kind":"Containment"}))],
            )
            .unwrap();
        owner
            .owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![(beam, rest, json!({"kind":"BeamRestRelation"}))],
            )
            .unwrap();
        let rows = beam_groups(&owner.owner.graph, &owner.voices, &owner.measures).unwrap();
        assert_eq!(comparable(rows), rest_control["groups"]);
        let propagation: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/propagation.json"
        ))
        .unwrap();
        let original_graph = owner.owner.graph.clone();
        let original_voices = owner.voices.clone();
        let first_id = original["groups"][0]["chords"][0]["bounds"].clone();
        let first_id = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == first_id)
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        let rest_id = original_graph.id(chord).unwrap();
        let voice_index = original_voices
            .iter()
            .position(|v| v.chord_ids.contains(&first_id))
            .unwrap();
        let voice_rows = |g: &crate::shared_sig::Graph, v: &Voice| {
            let snapshot = g.snapshot();
            json!(
                v.chord_ids
                    .iter()
                    .map(|id| {
                        let n = snapshot["inters"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|n| n["id"] == *id)
                            .unwrap();
                        json!({"kind":n["kind"],"bounds":n["bounds"]})
                    })
                    .collect::<Vec<_>>()
            )
        };
        for control in propagation["controls"].as_array().unwrap() {
            let mut graph = original_graph.clone();
            let mut voices = original_voices.clone();
            let mut rest_data = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == rest_id)
                .unwrap()
                .clone();
            rest_data["measure_index"] = json!(
                owner.measures[control["rest_measure_number"].as_u64().unwrap() as usize - 1].index
            );
            rest_data["measure_number"] = control["rest_measure_number"].clone();
            graph.vertex(chord, rest_data, false).unwrap();
            assert_eq!(
                voice_rows(&graph, &voices[voice_index]),
                control["voice_chords_before"]
            );
            // Direct BeamGroup.setVoice is equivalent here to its first-chord
            // seed assignment: deliberately unassign that stored field only.
            let first_key: Key = serde_json::from_value(
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == first_id)
                    .unwrap()["origin"]
                    .clone(),
            )
            .unwrap();
            let mut first_data = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == first_id)
                .unwrap()
                .clone();
            first_data["voice"] = Value::Null;
            graph.vertex(first_key, first_data, false).unwrap();
            assign_chord_voice(
                &mut graph,
                &mut voices,
                &owner.measures,
                first_id,
                voice_index,
            )
            .unwrap();
            assert_eq!(
                voice_rows(&graph, &voices[voice_index]),
                control["voice_chords_after"]
            );
            assert_eq!(
                graph.snapshot()["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == rest_id)
                    .unwrap()["voice"],
                control["rest_voice"]
            );
            let before = graph.snapshot();
            let before_members = voices[voice_index].chord_ids.clone();
            assign_chord_voice(
                &mut graph,
                &mut voices,
                &owner.measures,
                first_id,
                voice_index,
            )
            .unwrap();
            assert_eq!(graph.snapshot(), before);
            assert_eq!(voices[voice_index].chord_ids, before_members);
        }
        for (key, stem, state) in before_cache {
            let current = owner.owner.caches[&1]
                .registered_stem(&owner.owner.graph, key.local)
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&stem, &current));
            assert_eq!(current.snapshot().unwrap(), state);
        }
        assert_eq!(
            format!("{:?}", owner.owner.prepared.registry),
            before_registry
        );
    }
}
