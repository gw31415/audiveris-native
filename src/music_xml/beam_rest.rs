// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::music_measures::{ChordKind, OwnedMusic};
use std::collections::BTreeMap;

impl OwnedMusic {
    /// Observe relation candidates on actual owned groups before geometry and
    /// rhythm. This does not add BeamRest edges or construct Voice owners.
    pub fn interleaved_rest_candidates(
        &mut self,
        measure_number: usize,
    ) -> Result<serde_json::Value, String> {
        let mi = measure_number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let mut graph = self.owner.graph.clone();
        let groups = measure_groups(&mut graph, &self.measures[mi], &self.chords, mi)?;
        let rows = crate::music_voices::beam_group_chords(&graph)?;
        let mut candidates = Vec::new();
        for group in groups {
            let row = rows
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["id"] == group)
                .ok_or("music_rest_actual_group_required")?;
            let seeds = row["chords"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["id"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            let black = linked_rest_ids(
                &graph,
                &self.chords,
                &seeds,
                false,
                &["SeparateVoiceRelation"],
            )?;
            // The original call site supplies Next only, despite its broader
            // SameVoice wording in the surrounding Javadoc.
            let mut white =
                linked_rest_ids(&graph, &self.chords, &seeds, true, &["NextInVoiceRelation"])?;
            white.retain(|id| !black.contains(id));
            candidates.push(serde_json::json!({"group_id":group,"seed_chord_ids":seeds,
                "black_rest_ids":black,"white_rest_ids":white,"rest_detection_complete":false}));
        }
        self.owner.graph = graph;
        Ok(serde_json::json!(candidates))
    }
    #[cfg(test)]
    pub(crate) fn actual_measure_beam_groups(
        &mut self,
        number: usize,
    ) -> Result<Vec<usize>, String> {
        let mi = number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        measure_groups(&mut self.owner.graph, &self.measures[mi], &self.chords, mi)
    }

    /// Original pre-reset clear pass on actual measure/beam owners. Detection
    /// must be compared before this pass is connected to normal rhythm.
    pub fn clear_interleaved_rests(&mut self, measure_number: usize) -> Result<(), String> {
        let mi = measure_number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let measure = &self.measures[mi];
        let mut graph = self.owner.graph.clone();
        let groups = measure_groups(&mut graph, measure, &self.chords, mi)?;
        let snapshot = graph.snapshot();
        let nodes = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| (n["id"].as_u64().unwrap() as usize, n))
            .collect::<BTreeMap<_, _>>();
        let edges = snapshot["relations"].as_array().unwrap();
        for group in groups {
            for member in edges
                .iter()
                .filter(|e| e["kind"] == "Containment" && e["source"] == group)
            {
                let beam = member["target"].as_u64().unwrap() as usize;
                if nodes.get(&beam).is_none_or(|n| {
                    !matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                }) {
                    return Err("music_rest_actual_group_beam_required".into());
                }
                for relation in edges.iter().filter(|e| {
                    e["kind"] == "BeamRestRelation" && e["source"] == beam && e["manual"] != true
                }) {
                    graph.remove_owned_beam_rest(relation["id"].as_u64().unwrap() as usize)?;
                }
            }
        }
        self.owner.graph = graph;
        Ok(())
    }
}

pub(crate) fn linked_rest_ids(
    graph: &crate::shared_sig::Graph,
    chords: &[crate::music_measures::Chord],
    seeds: &[usize],
    transitive: bool,
    kinds: &[&str],
) -> Result<Vec<usize>, String> {
    if kinds.iter().any(|k| {
        !matches!(
            *k,
            "NextInVoiceRelation" | "SameVoiceRelation" | "SeparateVoiceRelation"
        )
    }) {
        return Err("music_rest_voice_relation_class_required".into());
    }
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect::<BTreeMap<_, _>>();
    let actual = |id: usize| -> Result<ChordKind, String> {
        let chord = chords
            .iter()
            .find(|c| c.id == id)
            .ok_or("music_rest_actual_chord_required")?;
        let node = nodes.get(&id).ok_or("music_rest_actual_chord_required")?;
        if graph.id(chord.key) != Some(id)
            || node["kind"]
                != match chord.kind {
                    ChordKind::Head => "HeadChordInter",
                    ChordKind::Rest => "RestChordInter",
                }
        {
            return Err("music_rest_actual_chord_required".into());
        }
        Ok(chord.kind)
    };
    let edges = snapshot["relations"].as_array().unwrap();
    let neighbors = |id: usize| -> Result<Vec<usize>, String> {
        actual(id)?;
        let incident = edges.iter().filter(|e| e["target"] == id).chain(
            edges
                .iter()
                .filter(|e| e["source"] == id && e["target"] != id),
        );
        let mut rests = Vec::new();
        for edge in incident.filter(|e| e["kind"].as_str().is_some_and(|k| kinds.contains(&k))) {
            let other = if edge["source"] == id {
                edge["target"].as_u64().unwrap() as usize
            } else {
                edge["source"].as_u64().unwrap() as usize
            };
            if actual(other)? == ChordKind::Rest && !rests.contains(&other) {
                rests.push(other);
            }
        }
        Ok(rests)
    };
    let mut all = Vec::new();
    for &seed in seeds {
        for rest in neighbors(seed)? {
            if !all.contains(&rest) {
                all.push(rest);
            }
        }
    }
    if transitive {
        loop {
            let mut next = Vec::new();
            for &rest in &all {
                for other in neighbors(rest)? {
                    if !all.contains(&other) && !next.contains(&other) {
                        next.push(other);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            all.extend(next);
        }
    }
    Ok(all)
}

fn measure_groups(
    graph: &mut crate::shared_sig::Graph,
    measure: &crate::music_measures::Measure,
    chords: &[crate::music_measures::Chord],
    mi: usize,
) -> Result<Vec<usize>, String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect::<BTreeMap<_, _>>();
    let edges = snapshot["relations"].as_array().unwrap();
    let mut groups = Vec::new();
    for chord in chords.iter().filter(|c| c.measure == mi) {
        let node = nodes
            .get(&chord.id)
            .ok_or("music_rest_actual_chord_required")?;
        if graph.id(chord.key) != Some(chord.id)
            || chord.key.system != measure.system
            || node["part_id"] != measure.part
            || node["staff_id"] != measure.staff
            || node["measure_index"] != measure.index
            || node["kind"]
                != match chord.kind {
                    ChordKind::Head => "HeadChordInter",
                    ChordKind::Rest => "RestChordInter",
                }
        {
            return Err("music_rest_actual_measure_chord_required".into());
        }
        if let Some(beam) = graph.actual_chord_beams(chord.key)?.first() {
            let member = nodes.get(beam).ok_or("music_rest_actual_beam_required")?;
            if !matches!(member["kind"].as_str(), Some("BeamInter" | "BeamHookInter")) {
                return Err("music_rest_actual_beam_required".into());
            }
            if let Some(edge) = edges
                .iter()
                .find(|e| e["kind"] == "Containment" && e["target"] == *beam)
            {
                let group = edge["source"].as_u64().unwrap() as usize;
                if nodes.get(&group).is_none_or(|n| {
                    n["kind"] != "BeamGroupInter" || n["origin"]["system"] != measure.system
                }) {
                    return Err("music_rest_actual_group_required".into());
                }
                if !groups.contains(&group) {
                    groups.push(group);
                }
            }
        }
    }
    Ok(groups)
}
