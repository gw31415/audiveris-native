// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, shared_sig::Key};
use std::sync::Arc;

impl OwnedMusic {
    /// Compared force sequence on actual Voice/Chord/container owners.
    /// Actual signature/font binding and eligibility gate follow separately.
    pub fn force_rhythm_voice_tuplets(
        &mut self,
        number: usize,
        identity: &Arc<()>,
        beat: &str,
        font: &TupletFontSymbols,
    ) -> Result<Vec<Key>, String> {
        self.with_tuplet_owners(|owner| {
            let plan = owner.plan_rhythm_voice_beats(number, identity, beat)?;
            let mut created = Vec::new();
            for group in plan.groups {
                let extended = owner.extend_rhythm_tuplet_group(number, &group, &[])?;
                for group in extended.groups {
                    // Source retains repeated returns of the same existing
                    // Tuplet. Deduplication changes the force result contract.
                    created.push(owner.generate_rhythm_tuplet(number, &group, font)?);
                }
            }
            Ok(created)
        })
    }
}
