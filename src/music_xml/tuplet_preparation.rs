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
        let mi = number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_actual_tuplet_measure_required")?;
        self.measures[mi]
            .tuplets
            .iter()
            .filter_map(|key| match self.owner.graph.tuplet_status(*key) {
                Ok((true, _)) => Some(Ok(*key)),
                Ok((false, _)) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }
    pub fn remove_implicit_rhythm_tuplets(&mut self, number: usize) -> Result<(), String> {
        let implicit = self.implicit_rhythm_tuplets(number)?;
        let mi = number - 1;
        let mut graph = self.owner.graph.clone();
        let mut measure = self.measures[mi].tuplets.clone();
        let mut stack = self.measures[mi].stack_tuplets.clone();
        for key in implicit {
            for source in graph.tuplet_chords(key)? {
                let chord = self
                    .chords
                    .iter()
                    .find(|ch| ch.key == source)
                    .ok_or("music_actual_tuplet_chord_owner_required")?;
                super::beam_rest::validate_measure_chord(
                    &graph,
                    self.measures
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
            graph.remove_owned_implicit_tuplet(key, &measure, &all)?;
        }
        self.owner.graph = graph;
        self.measures[mi].tuplets = measure;
        self.measures[mi].stack_tuplets = stack;
        Ok(())
    }
    /// Original pre-pass preparation, with the complete operation staged before
    /// any legitimate Measure reset. Implicit generation/merge remains separate.
    pub fn prepare_measure_rhythm(&mut self, number: usize) -> Result<Vec<usize>, String> {
        let mi = number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_actual_tuplet_measure_required")?;
        let graph = self.owner.graph.clone();
        let stage = self.measures[mi].tuplets.clone();
        let tuplets = std::mem::replace(&mut self.measures[mi].tuplets, stage);
        let stage = self.measures[mi].stack_tuplets.clone();
        let stack = std::mem::replace(&mut self.measures[mi].stack_tuplets, stage);
        let result = (|| {
            self.remove_implicit_rhythm_tuplets(number)?;
            self.clear_interleaved_rests(number)?;
            self.detect_interleaved_rests(number)
        })();
        if result.is_err() {
            self.owner.graph = graph;
            self.measures[mi].tuplets = tuplets;
            self.measures[mi].stack_tuplets = stack;
        }
        result
    }
}
