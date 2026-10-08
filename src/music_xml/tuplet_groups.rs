// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    music_measures::{Chord, Measure, OwnedMusic},
    shared_sig::{Graph, Key},
};

/// Actual Chord references in the original extension and first-pivot split.
#[derive(Debug, PartialEq, Eq)]
pub struct TupletGroups {
    pub extended: Vec<Key>,
    pub groups: Vec<Vec<Key>>,
}
impl OwnedMusic {
    /// The existing-object branch of generateOneTuplet. None needs generation;
    /// it is not the return value of the original complete generator.
    pub fn existing_rhythm_tuplet(
        &self,
        number: usize,
        group: &[Key],
    ) -> Result<Option<Key>, String> {
        let mi = measure_index(&self.measures, number)?;
        existing_on(&self.owner.graph, &self.measures, &self.chords, mi, group)
    }
    /// Extend to actual beam-group members, preserving the supplied output list.
    /// Allocation and inferred font bounds are the following dependency.
    pub fn extend_rhythm_tuplet_group(
        &mut self,
        number: usize,
        group: &[Key],
        initial: &[Key],
    ) -> Result<TupletGroups, String> {
        let mi = measure_index(&self.measures, number)?;
        let mut graph = self.owner.graph.clone();
        let result = extend_on(&mut graph, &self.measures, &self.chords, mi, group, initial)?;
        self.owner.graph = graph;
        Ok(result)
    }
}
pub(super) fn measure_index(measures: &[Measure], number: usize) -> Result<usize, String> {
    number
        .checked_sub(1)
        .filter(|mi| *mi < measures.len())
        .ok_or_else(|| "music_actual_tuplet_measure_required".into())
}
pub(super) fn actual<'a>(
    graph: &Graph,
    measures: &[Measure],
    chords: &'a [Chord],
    mi: usize,
    key: Key,
) -> Result<&'a Chord, String> {
    let chord = chords
        .iter()
        .find(|ch| ch.key == key)
        .ok_or("music_actual_tuplet_chord_owner_required")?;
    if chord.key.system != measures[mi].system {
        return Err("music_actual_tuplet_chord_owner_required".into());
    }
    super::beam_rest::validate_measure_chord(
        graph,
        measures
            .get(chord.measure)
            .ok_or("music_actual_tuplet_measure_required")?,
        chord,
    )?;
    Ok(chord)
}
pub(super) fn existing_on(
    graph: &Graph,
    measures: &[Measure],
    chords: &[Chord],
    mi: usize,
    group: &[Key],
) -> Result<Option<Key>, String> {
    for &key in group {
        actual(graph, measures, chords, mi, key)?;
        if let Some(tuplet) = graph.first_chord_tuplet(key)? {
            // Later group members are not read once an actual relation wins.
            return Ok(Some(tuplet));
        }
    }
    Ok(None)
}
pub(super) fn chord_group(graph: &mut Graph, key: Key) -> Result<Option<usize>, String> {
    let Some(beam) = graph.actual_chord_beams(key)?.first().copied() else {
        return Ok(None);
    };
    let snapshot = graph.snapshot();
    let Some(edge) = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "Containment" && e["target"] == beam)
    else {
        return Ok(None);
    };
    let group = edge["source"]
        .as_u64()
        .ok_or("music_actual_tuplet_group_required")? as usize;
    let node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == group)
        .ok_or("music_actual_tuplet_group_required")?;
    if node["kind"] != "BeamGroupInter" || node["origin"]["system"] != key.system {
        return Err("music_actual_tuplet_group_required".into());
    }
    Ok(Some(group))
}
pub(super) fn first_pivot(graph: &mut Graph, groups: &[Vec<Key>]) -> Result<Option<Key>, String> {
    for group in groups {
        let mut first = None;
        for &key in group {
            if let Some(bg) = chord_group(graph, key)? {
                if let Some(previous) = first {
                    if previous != bg {
                        return Ok(Some(key));
                    }
                } else {
                    first = Some(bg);
                }
            }
        }
    }
    Ok(None)
}
pub(super) fn extend_on(
    graph: &mut Graph,
    measures: &[Measure],
    chords: &[Chord],
    mi: usize,
    group: &[Key],
    initial: &[Key],
) -> Result<TupletGroups, String> {
    let mut extended = initial.to_vec();
    for &key in group {
        actual(graph, measures, chords, mi, key)?;
        if let Some(bg) = chord_group(graph, key)? {
            let rows = crate::music_voices::beam_group_chords_for(graph, bg)?;
            let row = rows
                .as_array()
                .unwrap()
                .first()
                .ok_or("music_actual_tuplet_group_required")?;
            for member in row["chords"].as_array().unwrap() {
                let chord = chords
                    .iter()
                    .find(|ch| member["id"] == ch.id)
                    .ok_or("music_actual_tuplet_chord_owner_required")?;
                actual(graph, measures, chords, mi, chord.key)?;
                if !extended.contains(&chord.key) {
                    extended.push(chord.key);
                }
            }
        } else {
            // The source deduplicates beam members only, not lone chords.
            extended.push(key);
        }
    }
    let mut with_centers = Vec::new();
    for &key in &extended {
        actual(graph, measures, chords, mi, key)?;
        let bounds = graph.retained_or_live_chord_bounds(key)?;
        with_centers.push((
            key,
            bounds[0]
                .checked_add(bounds[2] / 2)
                .ok_or("music_actual_tuplet_chord_bounds_required")?,
        ));
    }
    with_centers.sort_by_key(|(_, x)| *x);
    extended = with_centers.into_iter().map(|(key, _)| key).collect();
    let groups = if let Some(pivot) = first_pivot(graph, std::slice::from_ref(&extended))? {
        let index = extended.iter().position(|key| *key == pivot).unwrap();
        vec![extended[..index].to_vec(), extended[index..].to_vec()]
    } else {
        vec![extended.clone()]
    };
    Ok(TupletGroups { extended, groups })
}
