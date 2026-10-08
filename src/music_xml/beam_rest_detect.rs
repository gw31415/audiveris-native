// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    beam_rest::measure_groups,
    beam_rest_lookup::{actual_rest_note, chord_geometry},
};
use crate::{music_measures::OwnedMusic, shared_sig::Key};

impl OwnedMusic {
    /// Detect on retained real groups without resetting the callback clocks.
    /// The complete MeasureRhythm preparation also removes implicit tuplets.
    pub fn detect_interleaved_rests(&mut self, number: usize) -> Result<Vec<usize>, String> {
        let mi = number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let mut graph = self.owner.graph.clone();
        let groups = measure_groups(&mut graph, &self.measures[mi], &self.chords, mi)?;
        let mut added = Vec::new();
        for id in groups {
            // Hold this group's original Chord list before white additions;
            // later groups must see the effects of earlier group detection.
            let candidates = self.group_rest_candidates(&graph, id)?;
            for pair in candidates.seeds.windows(2) {
                let actual = |key: Key| {
                    self.chords
                        .iter()
                        .find(|ch| ch.key == key)
                        .ok_or("music_rest_actual_chord_required")
                };
                let tail_x = |geometry: crate::shared_sig::ChordRead| {
                    geometry
                        .tail
                        .map(|p| p[0])
                        .ok_or("music_rest_actual_chord_geometry_required")
                };
                let left = tail_x(chord_geometry(&mut graph, actual(pair[0])?)?)?;
                let right = tail_x(chord_geometry(&mut graph, actual(pair[1])?)?)?;
                for &key in &candidates.white {
                    let chord = actual(key)?;
                    let x = tail_x(chord_geometry(&mut graph, chord)?)?;
                    if x > left && x < right {
                        let (rest, center) = actual_rest_note(&graph, chord)?;
                        if let Some((beam, _)) = graph.nearest_rest_beam(
                            candidates.group,
                            center,
                            self.owner.sheet_skew,
                        )? {
                            added.push(graph.add_owned_beam_rest(
                                beam,
                                rest,
                                false,
                                self.owner.sheet_skew,
                                self.owner.interline,
                            )?);
                        }
                    }
                }
                added.extend(self.lookup_interleaved_rests_on(
                    &mut graph,
                    number,
                    candidates.group,
                    [pair[0], pair[1]],
                    &candidates.plain,
                    &candidates.white,
                )?);
            }
        }
        self.owner.graph = graph;
        Ok(added)
    }
}
