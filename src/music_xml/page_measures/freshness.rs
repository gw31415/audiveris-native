// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measure_layout::{PageClassification, PageMeasureIdentity, StackOwner};
use crate::music_measures::{Chord, Measure};
use crate::music_voices::{Slot, StackTiming, Voice};
use crate::shared_sig::Graph;

fn same_slots(held: &[Slot], now: &[Slot]) -> bool {
    held.len() == now.len()
        && held.iter().zip(now).all(|(a, b)| {
            Arc::ptr_eq(&a.identity, &b.identity)
                && a.id == b.id
                && a.time_offset == b.time_offset
                && a.x_offset == b.x_offset
                && a.incoming_ids == b.incoming_ids
        })
}
fn same_timing(held: &StackTiming, now: &StackTiming) -> bool {
    Arc::ptr_eq(&held.generation, &now.generation)
        && held.expected == now.expected
        && held.actual == now.actual
        && held.excess == now.excess
        && same_slots(&held.slots, &now.slots)
}
fn same_voice(held: &Voice, now: &Voice) -> bool {
    Arc::ptr_eq(&held.identity, &now.identity)
        && held.id == now.id
        && held.measure == now.measure
        && held.starting_staff == now.starting_staff
        && held.kind == now.kind
        && held.chord_ids == now.chord_ids
        && held.chords_initialized == now.chords_initialized
        && held.measure_rest_chord == now.measure_rest_chord
        && held.termination == now.termination
        && held.excess == now.excess
        && held.slots.len() == now.slots.len()
        && held.slots.iter().zip(&now.slots).all(|((ka, a), (kb, b))| {
            ka == kb && a.chord_id == b.chord_id && a.continuing == b.continuing
        })
}
fn same_classification(held: &PageClassification, now: &PageClassification) -> bool {
    held.identity == now.identity
        && same_timing(&held.timing, &now.timing)
        && held.measures.len() == now.measures.len()
        && held
            .measures
            .iter()
            .zip(&now.measures)
            .all(|(a, b)| Arc::ptr_eq(a, b))
        && held.voices.len() == now.voices.len()
        && held
            .voices
            .iter()
            .zip(&now.voices)
            .all(|(a, b)| same_voice(a, b))
        && held.voice_last_offsets == now.voice_last_offsets
        && held.voice_ends == now.voice_ends
}

pub(super) fn capture(
    graph: &Graph,
    chords: &[Chord],
    voices: &[Voice],
    measures: &[Measure],
    stack: &StackOwner,
) -> Result<PageClassification, String> {
    let timing = stack
        .timing
        .as_ref()
        .ok_or("music_page_actual_rhythm_required")?;
    let mut indices = Vec::new();
    for geometry in &stack.measures {
        graph.require_actual_music_measure(geometry)?;
        if !Arc::ptr_eq(&geometry.stack, &stack.stack) {
            return Err("music_page_actual_Stack_Measure_required".into());
        }
        let matches = measures
            .iter()
            .enumerate()
            .filter_map(|(mi, measure)| {
                measure
                    .geometry
                    .as_ref()
                    .is_some_and(|g| Arc::ptr_eq(g, geometry))
                    .then_some(mi)
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 || indices.contains(&matches[0]) {
            return Err("music_page_actual_Stack_Measure_required".into());
        }
        let mi = matches[0];
        let view = &measures[mi].timing;
        if timing.expected != view.expected
            || timing.actual != view.actual
            || timing.excess != view.excess
            || !same_slots(&timing.slots, &view.slots)
        {
            return Err("music_page_shared_Stack_clock_required".into());
        }
        indices.push(mi);
    }
    if indices.is_empty() {
        return Err("music_page_actual_Stack_Measure_required".into());
    }
    let actual_voices = indices
        .iter()
        .flat_map(|mi| voices.iter().filter(move |v| v.measure == *mi))
        .cloned()
        .collect::<Vec<_>>();
    let mut voice_last_offsets = Vec::new();
    let mut voice_ends = Vec::new();
    for voice in &actual_voices {
        for id in voice
            .chord_ids
            .iter()
            .chain(voice.measure_rest_chord.iter())
        {
            let chord = chords
                .iter()
                .find(|c| c.id == *id && c.measure == voice.measure)
                .ok_or("music_page_actual_Chord_Voice_required")?;
            let owner = graph
                .chord_voice_owner(chord.key, voices)?
                .ok_or("music_page_actual_Chord_Voice_required")?;
            if !Arc::ptr_eq(&owner.identity, &voice.identity) {
                return Err("music_page_actual_Chord_Voice_required".into());
            }
        }
        let last_offset = if voice.measure_rest_chord.is_none() {
            voice
                .chord_ids
                .last()
                .map(|id| {
                    let chord = chords
                        .iter()
                        .find(|c| c.id == *id && c.measure == voice.measure)
                        .ok_or("music_page_actual_Chord_Voice_required")?;
                    graph.actual_chord_time_offset(chord.key)
                })
                .transpose()?
                .flatten()
        } else {
            None
        };
        voice_last_offsets.push(last_offset);
        voice_ends.push(crate::music_voices::last_end_time(graph, voice)?);
    }
    Ok(PageClassification {
        identity: stack
            .page_identity
            .ok_or("musicxml_actual_PAGE_measure_identity_required")?,
        timing: timing.clone(),
        measures: stack.measures.clone(),
        voices: actual_voices,
        voice_last_offsets,
        voice_ends,
    })
}

impl OwnedMusic {
    pub(crate) fn require_recognized_page_classification(
        &self,
        stack: &StackOwner,
    ) -> Result<PageMeasureIdentity, String> {
        let identity = stack
            .page_identity
            .ok_or("musicxml_actual_PAGE_measure_identity_required")?;
        let held = stack
            .page_classification
            .as_ref()
            .ok_or("musicxml_PAGE_classification_required")?;
        let now = capture(
            &self.owner.graph,
            &self.chords,
            &self.voices,
            &self.measures,
            stack,
        )?;
        if !same_classification(held, &now) {
            return Err("musicxml_stale_PAGE_classification".into());
        }
        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn timing() -> StackTiming {
        StackTiming {
            expected: Some(Fraction::new(1, 2).unwrap()),
            actual: Some(Fraction::new(7, 64).unwrap()),
            slots: vec![Slot {
                identity: Arc::new(()),
                id: 1,
                time_offset: Some(Fraction::ZERO),
                x_offset: 10,
                incoming_ids: vec![17],
            }],
            ..Default::default()
        }
    }
    fn voice() -> Voice {
        Voice {
            identity: Arc::new(()),
            id: 1,
            measure: 0,
            starting_staff: 1,
            kind: crate::music_voices::VoiceKind::High,
            chord_ids: vec![17],
            chords_initialized: true,
            termination: Some(
                Fraction::new(7, 64)
                    .unwrap()
                    .difference(Fraction::new(1, 2).unwrap())
                    .unwrap(),
            ),
            excess: None,
            slots: BTreeMap::new(),
            measure_rest_chord: None,
        }
    }
    #[test]
    fn new_rhythm_generation_invalidates_even_identical_empty_clock_values() {
        let old = StackTiming::default();
        assert!(same_timing(&old, &old.clone()));
        assert!(!same_timing(&old, &StackTiming::default()));
    }
    #[test]
    fn same_slot_number_and_clock_cannot_replace_the_actual_slot() {
        let old = timing();
        let mut next = old.clone();
        next.slots[0].identity = Arc::new(());
        assert!(!same_timing(&old, &next));
    }
    #[test]
    fn exact_clock_and_excess_changes_require_reclassification() {
        let old = timing();
        for (actual, excess) in [
            (Some(Fraction::new(1, 8).unwrap()), None),
            (old.actual, Some(Fraction::new(1, 64).unwrap())),
        ] {
            let mut next = old.clone();
            next.actual = actual;
            next.excess = excess;
            assert!(!same_timing(&old, &next));
        }
        let mut next = old.clone();
        next.expected = Some(Fraction::new(3, 4).unwrap());
        assert!(!same_timing(&old, &next));
    }
    #[test]
    fn reused_voice_number_does_not_reuse_page_classification() {
        let old = voice();
        let mut next = old.clone();
        assert!(same_voice(&old, &next));
        next.identity = Arc::new(());
        assert!(!same_voice(&old, &next));
    }
    #[test]
    fn changed_actual_voice_ownership_or_last_end_requires_reclassification() {
        let old = PageClassification {
            identity: PageMeasureIdentity {
                number: 0,
                pickup: true,
            },
            timing: timing(),
            measures: vec![],
            voices: vec![voice()],
            voice_last_offsets: vec![Some(Fraction::ZERO)],
            voice_ends: vec![Some(Fraction::new(7, 64).unwrap())],
        };
        let mut next = old.clone();
        next.identity.pickup = false;
        assert!(!same_classification(&old, &next));
        let mut next = old.clone();
        next.voices[0].chord_ids.push(18);
        assert!(!same_classification(&old, &next));
        let mut next = old.clone();
        next.voice_last_offsets[0] = None;
        assert!(!same_classification(&old, &next));
        let mut next = old.clone();
        next.voice_ends[0] = Some(Fraction::new(1, 8).unwrap());
        assert!(!same_classification(&old, &next));
    }
}
