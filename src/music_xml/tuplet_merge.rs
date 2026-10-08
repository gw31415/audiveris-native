// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, shared_sig::Key};
impl OwnedMusic {
    pub fn merge_rhythm_tuplets(
        &mut self,
        number: usize,
        font: &TupletFontSymbols,
    ) -> Result<Vec<Key>, String> {
        super::tuplet_groups::measure_index(&self.measures, number)?;
        self.with_tuplet_owners(|owner| {
            let mut generated = Vec::new();
            for identity in owner.rhythm_voices_with_implicit_tuplets(number)? {
                let tuplets = owner.rhythm_voice_tuplets(number, &identity)?;
                let mut previous = None;
                let mut previous_group = Vec::new();
                let mut last_group = None;
                for tuplet in tuplets {
                    let chords = owner.owner.graph.tuplet_chords(tuplet)?;
                    let mut group = chords
                        .into_iter()
                        .map(|key| {
                            owner
                                .owner
                                .graph
                                .retained_or_live_chord_bounds(key)
                                .map(|b| (key, b[0]))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    group.sort_by_key(|(_, x)| *x);
                    let group = group.into_iter().map(|(key, _)| key).collect::<Vec<_>>();
                    if let Some(last_group) = last_group {
                        let first = *group.first().ok_or("music_tuplet_empty_merge_group")?;
                        if super::tuplet_groups::chord_group(&mut owner.owner.graph, first)?
                            == Some(last_group)
                        {
                            owner.remove_rhythm_tuplets(number, &[previous, Some(tuplet)])?;
                            let mut combined = previous_group;
                            combined.extend(&group);
                            let plan = owner.extend_rhythm_tuplet_group(number, &combined, &[])?;
                            for group in plan.groups {
                                generated.push(owner.generate_rhythm_tuplet(number, &group, font)?);
                            }
                        }
                    }
                    // Source keeps the original object/group even after removal or later-object reuse.
                    previous = Some(tuplet);
                    let last = *group.last().ok_or("music_tuplet_empty_merge_group")?;
                    last_group = super::tuplet_groups::chord_group(&mut owner.owner.graph, last)?;
                    previous_group = group;
                }
            }
            Ok(generated)
        })
    }
}
