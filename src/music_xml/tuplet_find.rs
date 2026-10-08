// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, music_time::Fraction,
    shared_sig::Key,
};

impl OwnedMusic {
    /// Actual find/force order with explicit controlled beat/font resources.
    /// Ordinary source signature/font/switch ownership follows separately.
    pub fn find_rhythm_implicit_tuplets(
        &mut self,
        number: usize,
        beat: &str,
        font: &TupletFontSymbols,
    ) -> Result<Vec<Key>, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        self.with_tuplet_owners(|owner| {
            let voices = owner
                .voices
                .iter()
                .filter(|v| v.measure == mi)
                .map(|v| {
                    (
                        v.identity.clone(),
                        v.measure_rest_chord.is_some(),
                        v.chord_ids.clone(),
                    )
                })
                .collect::<Vec<_>>();
            let mut created = Vec::new();
            for (identity, measure_rest, chords) in voices {
                if measure_rest {
                    continue;
                }
                let id = chords
                    .first()
                    .ok_or("music_tuplet_empty_voice_chord_list")?;
                let chord = owner
                    .chords
                    .iter()
                    .find(|ch| ch.id == *id)
                    .ok_or("music_actual_tuplet_chord_owner_required")?;
                if owner.owner.graph.actual_chord_time_offset(chord.key)? != Some(Fraction::ZERO) {
                    // Rational.ZERO.equals(null) is false. This original
                    // boundary bypasses sum, expected, signature and font;
                    // it does not synthesize a missing source/Voice.
                    continue;
                }
                let sum = owner
                    .sans_tuplet_rhythm_voice_duration(number, &identity)?
                    .ok_or("music_tuplet_null_voice_chord_duration")?;
                let sum = Fraction::parse(&sum)?;
                let expected = owner.measures[mi]
                    .timing
                    .expected
                    .ok_or("music_tuplet_expected_duration_required")?;
                if expected == Fraction::ZERO {
                    return Err("music_tuplet_zero_expected_duration".into());
                }
                if sum == expected.times(Fraction::new(3, 2)?)? {
                    created
                        .extend(owner.force_rhythm_voice_tuplets(number, &identity, beat, font)?);
                }
            }
            Ok(created)
        })
    }
}
