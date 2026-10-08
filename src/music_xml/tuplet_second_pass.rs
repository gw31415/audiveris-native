// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, shared_sig::Key};
impl OwnedMusic {
    /// Compared first-pass recheck with explicit actual switch/beat/font resources.
    /// A nonempty return requests the next legitimate Measure reset, even when
    /// it contains an old sign that this same recheck removes.
    pub fn recheck_rhythm_implicit_tuplets(
        &mut self,
        number: usize,
        pass: usize,
        enabled: bool,
        beat: &str,
        font: &TupletFontSymbols,
    ) -> Result<Vec<Key>, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        if !enabled || pass != 1 || self.measures[mi].timing.expected.is_none() {
            return Ok(Vec::new());
        }
        self.with_tuplet_owners(|owner| {
            let old = owner.implicit_rhythm_tuplets(number)?;
            let returned = owner.find_rhythm_implicit_tuplets(number, beat, font)?;
            if !returned.is_empty() {
                // find can return the same existing object more than once.
                // Removal follows the captured old collection, not these returns
                // or the now-mutated Measure collection containing fresh signs.
                let captured = old.into_iter().map(Some).collect::<Vec<_>>();
                owner.remove_rhythm_tuplets(number, &captured)?;
            }
            Ok(returned)
        })
    }
}
