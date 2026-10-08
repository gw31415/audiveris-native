// SPDX-License-Identifier: AGPL-3.0-or-later
use super::RhythmCandidates;
use crate::{
    music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, music_time::Fraction,
    shared_sig::Key,
};
impl OwnedMusic {
    pub fn inspect_rhythm_voice_ends(
        &mut self,
        candidates: &mut RhythmCandidates,
        font: &TupletFontSymbols,
    ) -> Result<Vec<Key>, String> {
        let number = candidates.number;
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        let expected = self.measures[mi].timing.expected;
        let mut staged = candidates.clone();
        let result = self.with_tuplet_owners(|owner| {
            let mut generated = Vec::new();
            for identity in owner.rhythm_voices_with_implicit_tuplets(number)? {
                let Some(last_related) = owner.last_rhythm_voice_tuplet_chord(number, &identity)?
                else {
                    continue;
                };
                let Some(first) =
                    owner.rhythm_voice_chord_after(number, &identity, Some(last_related))?
                else {
                    continue;
                };
                let start = owner.owner.graph.actual_chord_time_offset(first)?;
                let apply = if let Some(expected) = expected {
                    let voice = owner
                        .voices
                        .iter()
                        .find(|v| std::sync::Arc::ptr_eq(&v.identity, &identity))
                        .ok_or("music_actual_tuplet_voice_owner_required")?;
                    let last = voice
                        .chord_ids
                        .last()
                        .ok_or("music_tuplet_empty_voice_chord_list")?;
                    let key = owner
                        .chords
                        .iter()
                        .find(|ch| ch.id == *last)
                        .ok_or("music_actual_tuplet_chord_owner_required")?
                        .key;
                    let end = owner
                        .owner
                        .graph
                        .chord_end_time(key)?
                        .as_deref()
                        .map(Fraction::parse)
                        .transpose()?
                        .ok_or("music_tuplet_null_inspect_chord_end")?;
                    let start = start.ok_or("music_tuplet_null_inspect_chord_start")?;
                    let actual = end.difference(start)?;
                    let normal = expected.difference(start)?;
                    !normal.is_zero() && actual.is_positive_ratio(normal, Fraction::new(3, 2)?)?
                } else {
                    true
                };
                if apply {
                    let sync = start.map(crate::music_time::format_fraction);
                    generated.extend(owner.shrink_rhythm_voice(
                        &mut staged,
                        &identity,
                        None,
                        sync.as_deref(),
                        font,
                    )?);
                }
            }
            Ok(generated)
        });
        if result.is_ok() {
            candidates.clocks = staged.clocks;
        }
        result
    }
}
