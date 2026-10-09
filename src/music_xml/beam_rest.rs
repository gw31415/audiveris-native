// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::music_measures::{ChordKind, OwnedMusic};
use crate::shared_sig::{Graph, Key};
use std::collections::BTreeMap;

pub(super) struct RestGroupCandidates {
    pub group: Key,
    pub seeds: Vec<Key>,
    pub black: Vec<Key>,
    pub white: Vec<Key>,
    pub plain: Vec<Key>,
    pub measures: Vec<usize>,
}

#[derive(Clone, Copy)]
pub(crate) struct RestGeometry<'a> {
    pub staffs: &'a [std::sync::Arc<crate::grid_ir::StaffGeometry>],
    pub skew: crate::sheet_skew::SheetSkew,
    pub interline: usize,
}
pub(super) struct RestOwners<'a> {
    pub measures: &'a [crate::music_measures::Measure],
    pub chords: &'a [crate::music_measures::Chord],
    pub geometry: RestGeometry<'a>,
}
impl OwnedMusic {
    pub fn staff_midline_at(&self, measure_number: usize, x: i32) -> Result<i32, String> {
        let measure = measure_number
            .checked_sub(1)
            .and_then(|mi| self.measures.get(mi))
            .ok_or("music_rest_actual_measure_required")?;
        let staff = self
            .owner
            .staffs
            .iter()
            .find(|staff| staff.id == measure.staff && staff.system == measure.system)
            .ok_or("music_actual_staff_owner_required")?;
        staff.middle_at(x)
    }
    /// Stage original addition callbacks on real measure/group/rest owners.
    /// Full detection still needs plain lookup and the original preparation pass.
    pub fn add_interleaved_rests(
        &mut self,
        measure_number: usize,
        links: &[(crate::shared_sig::Key, crate::shared_sig::Key, bool)],
    ) -> Result<Vec<usize>, String> {
        let mi = measure_number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let mut graph = self.owner.graph.clone();
        let groups = measure_groups(&mut graph, &self.measures[mi], &self.chords, mi)?;
        let mut ids = Vec::new();
        for &(beam, rest, manual) in links {
            let snapshot = graph.snapshot();
            let bid = graph.id(beam).ok_or("music_rest_actual_beam_required")?;
            if !snapshot["relations"].as_array().unwrap().iter().any(|e| {
                e["kind"] == "Containment"
                    && e["target"] == bid
                    && groups.iter().any(|g| e["source"] == *g)
            }) {
                return Err("music_rest_actual_measure_group_required".into());
            }
            let chord = self
                .chords
                .iter()
                .find(|c| c.key == rest && c.kind == ChordKind::Rest && c.measure == mi)
                .ok_or("music_rest_actual_measure_chord_required")?;
            let mut members = Vec::new();
            for member in &chord.members {
                let key: crate::shared_sig::Key = serde_json::from_value(member["origin"].clone())
                    .map_err(|_| "music_rest_actual_note_required")?;
                let note_id = graph.id(key).ok_or("music_rest_actual_note_required")?;
                let note = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == note_id)
                    .ok_or("music_rest_actual_note_required")?;
                if note["kind"] != "RestInter"
                    || member["id"] != note_id
                    || !snapshot["relations"].as_array().unwrap().iter().any(|e| {
                        e["kind"] == "Containment"
                            && e["source"] == chord.id
                            && e["target"] == note_id
                    })
                {
                    return Err("music_rest_actual_note_required".into());
                }
                let bounds: [i32; 4] = serde_json::from_value(note["bounds"].clone())
                    .map_err(|_| "music_rest_actual_note_bounds_required")?;
                let y = bounds[1]
                    .checked_add(bounds[3] / 2)
                    .ok_or("music_rest_actual_note_bounds_required")?;
                members.push((key, y));
            }
            members.sort_by_key(|(_, y)| std::cmp::Reverse(*y));
            let key = members.first().ok_or("music_rest_actual_note_required")?.0;
            ids.push(graph.add_owned_beam_rest(
                beam,
                key,
                manual,
                self.owner.sheet_skew,
                self.owner.interline,
            )?);
        }
        self.owner.graph = graph;
        Ok(ids)
    }
    /// Query a real member group using the sheet geometry retained by the image
    /// owner. Full detection still requires the original add/lookup passes.
    pub fn nearest_interleaved_rest_beam(
        &mut self,
        measure_number: usize,
        group: crate::shared_sig::Key,
        point: [i32; 2],
    ) -> Result<Option<(crate::shared_sig::Key, f64)>, String> {
        let mi = measure_number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let mut graph = self.owner.graph.clone();
        let groups = measure_groups(&mut graph, &self.measures[mi], &self.chords, mi)?;
        let id = graph.id(group).ok_or("music_rest_actual_group_required")?;
        if !groups.contains(&id) {
            return Err("music_rest_actual_measure_group_required".into());
        }
        let result = graph.nearest_rest_beam(group, point, self.owner.sheet_skew)?;
        self.owner.graph = graph;
        Ok(result)
    }
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
        let mut candidates = Vec::new();
        for group in groups {
            let c = self.rest_owners().group_rest_candidates(&graph, group)?;
            let ids = |keys: &[Key]| {
                keys.iter()
                    .map(|k| graph.id(*k).unwrap())
                    .collect::<Vec<_>>()
            };
            candidates.push(serde_json::json!({"group_id":group,"seed_chord_ids":ids(&c.seeds),
                "black_rest_ids":ids(&c.black),"white_rest_ids":ids(&c.white),"plain_rest_ids":ids(&c.plain),
                "measure_numbers":c.measures.iter().map(|mi|mi+1).collect::<Vec<_>>(),"rest_detection_complete":false}));
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

    pub fn clear_interleaved_rests(&mut self, number: usize) -> Result<(), String> {
        let mut graph = self.owner.graph.clone();
        self.rest_owners().clear_on(&mut graph, number)?;
        self.owner.graph = graph;
        Ok(())
    }
    pub(super) fn rest_owners(&self) -> RestOwners<'_> {
        RestOwners {
            measures: &self.measures,
            chords: &self.chords,
            geometry: RestGeometry {
                staffs: &self.owner.staffs,
                skew: self.owner.sheet_skew,
                interline: self.owner.interline,
            },
        }
    }
}
impl RestOwners<'_> {
    pub(super) fn group_rest_candidates(
        &self,
        graph: &Graph,
        group: usize,
    ) -> Result<RestGroupCandidates, String> {
        let rows = crate::music_voices::beam_group_chords(graph)?;
        let member_rows = crate::music_voices::beam_group_chords_by_member(graph)?;
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
            graph,
            self.chords,
            &seeds,
            false,
            &["SeparateVoiceRelation"],
        )?;
        // The original call site supplies Next only, despite its broader
        // SameVoice wording in the surrounding Javadoc.
        let mut white =
            linked_rest_ids(graph, self.chords, &seeds, true, &["NextInVoiceRelation"])?;
        white.retain(|id| !black.contains(id));
        let member_row = member_rows
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == group)
            .ok_or("music_rest_actual_group_required")?;
        let mut measure_indices = Vec::new();
        for row in member_row["chords"].as_array().unwrap() {
            let chord = self
                .chords
                .iter()
                .find(|c| row["id"] == c.id)
                .ok_or("music_rest_actual_chord_required")?;
            let measure = self
                .measures
                .get(chord.measure)
                .ok_or("music_rest_actual_measure_required")?;
            validate_measure_chord(graph, measure, chord)?;
            if !measure_indices.contains(&chord.measure) {
                measure_indices.push(chord.measure);
            }
        }
        // Original sorts by Stack.left. The admitted single-staff part
        // shares that retained Measure boundary; multipart stacks remain pending.
        measure_indices.sort_by_key(|mi| self.measures[*mi].left);
        let mut plain = Vec::new();
        for &mi in &measure_indices {
            let measure = &self.measures[mi];
            let snapshot = graph.snapshot();
            if snapshot["inters"].as_array().unwrap().iter().any(|node| {
                node["kind"] == "RestChordInter"
                    && node["origin"]["system"] == measure.system
                    && node["part_id"] == measure.part
                    && node["staff_id"] == measure.staff
                    && node["measure_index"] == measure.index
                    && !self
                        .chords
                        .iter()
                        .any(|c| c.measure == mi && c.kind == ChordKind::Rest && node["id"] == c.id)
            }) {
                return Err("music_rest_actual_chord_required".into());
            }
            for chord in self
                .chords
                .iter()
                .filter(|c| c.measure == mi && c.kind == ChordKind::Rest)
            {
                validate_measure_chord(graph, &self.measures[mi], chord)?;
                if !black.contains(&chord.id)
                    && !white.contains(&chord.id)
                    && !plain.contains(&chord.id)
                {
                    plain.push(chord.id);
                }
            }
        }
        let keys = |ids: &[usize]| -> Result<Vec<Key>, String> {
            ids.iter()
                .map(|id| {
                    let chord = self
                        .chords
                        .iter()
                        .find(|c| c.id == *id)
                        .ok_or("music_rest_actual_chord_required")?;
                    validate_measure_chord(
                        graph,
                        self.measures
                            .get(chord.measure)
                            .ok_or("music_rest_actual_measure_required")?,
                        chord,
                    )?;
                    Ok(chord.key)
                })
                .collect()
        };
        let snapshot = graph.snapshot();
        let group_key = serde_json::from_value(
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == group)
                .ok_or("music_rest_actual_group_required")?["origin"]
                .clone(),
        )
        .map_err(|_| "music_rest_actual_group_required")?;
        Ok(RestGroupCandidates {
            group: group_key,
            seeds: keys(&seeds)?,
            black: keys(&black)?,
            white: keys(&white)?,
            plain: keys(&plain)?,
            measures: measure_indices,
        })
    }
    pub(super) fn clear_on(&self, graph: &mut Graph, measure_number: usize) -> Result<(), String> {
        let mi = measure_number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let measure = &self.measures[mi];
        let groups = measure_groups(graph, measure, self.chords, mi)?;
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

pub(super) fn measure_groups(
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
        validate_measure_chord(graph, measure, chord)?;
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

pub(super) fn validate_measure_chord(
    graph: &crate::shared_sig::Graph,
    measure: &crate::music_measures::Measure,
    chord: &crate::music_measures::Chord,
) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == chord.id)
        .ok_or("music_rest_actual_chord_required")?;
    let staff_belongs = if let Some(geometry) = &measure.geometry {
        graph.require_actual_music_measure(geometry)?;
        geometry.part.system == measure.system
            && geometry.part.id == measure.part
            && geometry.index == measure.index
            && node["staff_id"]
                .as_u64()
                .is_some_and(|staff| geometry.part.staffs.iter().any(|s| s.id == staff))
    } else {
        node["staff_id"] == measure.staff
    };
    if graph.id(chord.key) != Some(chord.id)
        || chord.key.system != measure.system
        || node["part_id"] != measure.part
        || !staff_belongs
        || node["measure_index"] != measure.index
        || node["kind"]
            != match chord.kind {
                ChordKind::Head => "HeadChordInter",
                ChordKind::Rest => "RestChordInter",
            }
    {
        return Err("music_rest_actual_measure_chord_required".into());
    }
    Ok(())
}
