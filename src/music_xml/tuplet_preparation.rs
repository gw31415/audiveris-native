// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_measures::OwnedMusic, shared_sig::Key};

impl OwnedMusic {
    /// Register an existing actual Tuplet object in an admitted Measure/Stack.
    pub fn register_rhythm_tuplet(
        &mut self,
        number: usize,
        key: Key,
        unassigned: bool,
    ) -> Result<(), String> {
        let mi = number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_actual_tuplet_measure_required")?;
        self.owner.graph.tuplet_status(key)?;
        let measure = &mut self.measures[mi];
        if key.system != measure.system {
            return Err("music_actual_tuplet_measure_required".into());
        }
        if self.measures.iter().enumerate().any(|(index, m)| {
            index != mi && (m.tuplets.contains(&key) || m.stack_tuplets.contains(&key))
        }) {
            return Err("music_actual_tuplet_foreign_owner_required".into());
        }
        let measure = &mut self.measures[mi];
        let collection = if unassigned {
            &mut measure.stack_tuplets
        } else {
            &mut measure.tuplets
        };
        if !collection.contains(&key) {
            collection.push(key);
        }
        if !unassigned {
            measure.stack_tuplets.retain(|k| *k != key);
        }
        Ok(())
    }
    pub fn implicit_rhythm_tuplets(&self, number: usize) -> Result<Vec<Key>, String> {
        implicit_on(
            &self.owner.graph,
            self.measures
                .get(
                    number
                        .checked_sub(1)
                        .ok_or("music_actual_tuplet_measure_required")?,
                )
                .ok_or("music_actual_tuplet_measure_required")?,
        )
    }
    pub fn remove_implicit_rhythm_tuplets(&mut self, number: usize) -> Result<(), String> {
        remove_on(
            &mut self.owner.graph,
            &mut self.measures,
            &self.chords,
            number,
        )
    }
    /// Prepare on borrowed real owner parts before the legitimate Measure reset.
    pub fn prepare_measure_rhythm(&mut self, number: usize) -> Result<Vec<usize>, String> {
        prepare_on(
            &mut self.owner.graph,
            &mut self.measures,
            &self.chords,
            super::beam_rest::RestGeometry {
                staffs: &self.owner.staffs,
                skew: self.owner.sheet_skew,
                interline: self.owner.interline,
            },
            number,
        )
    }
}
fn implicit_on(
    graph: &crate::shared_sig::Graph,
    measure: &crate::music_measures::Measure,
) -> Result<Vec<Key>, String> {
    measure
        .tuplets
        .iter()
        .filter_map(|key| match graph.tuplet_status(*key) {
            Ok((true, _)) => Some(Ok(*key)),
            Ok((false, _)) => None,
            Err(e) => Some(Err(e)),
        })
        .collect()
}
fn remove_on(
    graph: &mut crate::shared_sig::Graph,
    measures: &mut [crate::music_measures::Measure],
    chords: &[crate::music_measures::Chord],
    number: usize,
) -> Result<(), String> {
    let implicit = implicit_on(
        graph,
        measures
            .get(
                number
                    .checked_sub(1)
                    .ok_or("music_actual_tuplet_measure_required")?,
            )
            .ok_or("music_actual_tuplet_measure_required")?,
    )?;
    let mi = number - 1;
    let mut staged = graph.clone();
    let mut measure = measures[mi].tuplets.clone();
    let mut stack = measures[mi].stack_tuplets.clone();
    for key in implicit {
        for source in staged.tuplet_chords(key)? {
            let chord = chords
                .iter()
                .find(|ch| ch.key == source)
                .ok_or("music_actual_tuplet_chord_owner_required")?;
            super::beam_rest::validate_measure_chord(
                &staged,
                measures
                    .get(chord.measure)
                    .ok_or("music_actual_tuplet_measure_required")?,
                chord,
            )?;
        }
        measure.retain(|k| *k != key);
        stack.retain(|k| *k != key);
        let mut all = measure.clone();
        for &key in &stack {
            if !all.contains(&key) {
                all.push(key);
            }
        }
        staged.remove_owned_implicit_tuplet(key, &measure, &all)?;
    }
    *graph = staged;
    measures[mi].tuplets = measure;
    measures[mi].stack_tuplets = stack;
    Ok(())
}
pub(super) fn prepare_on(
    graph: &mut crate::shared_sig::Graph,
    measures: &mut [crate::music_measures::Measure],
    chords: &[crate::music_measures::Chord],
    geometry: super::beam_rest::RestGeometry<'_>,
    number: usize,
) -> Result<Vec<usize>, String> {
    let mi = number
        .checked_sub(1)
        .filter(|mi| *mi < measures.len())
        .ok_or("music_actual_tuplet_measure_required")?;
    let old_graph = graph.clone();
    let stage = measures[mi].tuplets.clone();
    let tuplets = std::mem::replace(&mut measures[mi].tuplets, stage);
    let stage = measures[mi].stack_tuplets.clone();
    let stack = std::mem::replace(&mut measures[mi].stack_tuplets, stage);
    let result = (|| {
        remove_on(graph, measures, chords, number)?;
        let owners = super::beam_rest::RestOwners {
            measures,
            chords,
            geometry,
        };
        owners.clear_on(graph, number)?;
        owners.detect_on(graph, number)
    })();
    if result.is_err() {
        *graph = old_graph;
        measures[mi].tuplets = tuplets;
        measures[mi].stack_tuplets = stack;
    }
    result
}
