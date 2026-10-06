// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained Voice chord ownership in the admitted ordinary single-staff rhythm
//! path. VoiceKind HIGH is justified by the existing first-staff-only guard;
//! general voice matching/merged parts/slots/tie propagation remain pending.
use crate::music_measures::{Chord, Measure};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone)]
pub(crate) struct Voice {
    pub id: usize,
    pub measure: usize,
    pub starting_staff: u64,
    pub chord_ids: Vec<usize>,
}
/// Voice constructor -> Measure.generateVoiceId(HIGH), Voice.addChord. The
/// caller has already admitted a single voice in this nonmerged one-staff part.
pub(crate) fn single(
    measure_index: usize,
    measure: &Measure,
    chords: &[&Chord],
    existing: &[Voice],
) -> Result<Voice, String> {
    if chords.is_empty() || existing.iter().any(|v| v.measure == measure_index) {
        return Err("music_voice_first_assignment_required".into());
    }
    let mut members = Vec::new();
    let mut seen = BTreeSet::new();
    for chord in chords {
        if chord.measure != measure_index || chord.key.system != measure.system {
            return Err("music_voice_owned_measure_chord_required".into());
        }
        // Original addChord ignores an existing actual chord and stably sorts
        // by integer center abscissa; it does not order by registration ID.
        if seen.insert(chord.id) {
            members.push(*chord);
        }
    }
    members.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
    let id = (1..)
        .find(|id| {
            !existing
                .iter()
                .any(|v| v.measure == measure_index && v.id == *id)
        })
        .unwrap();
    Ok(Voice {
        id,
        measure: measure_index,
        starting_staff: measure.staff,
        chord_ids: members.iter().map(|c| c.id).collect(),
    })
}
/// Measure.removeVoiceChord -> Voice.removeChord. Remove an empty Voice only
/// when this removal actually removed its chord, matching the original loop.
pub(crate) fn remove_chords(voices: &mut Vec<Voice>, removed: &[usize]) {
    for id in removed {
        let mut index = 0;
        while index < voices.len() {
            if let Some(member) = voices[index].chord_ids.iter().position(|c| c == id) {
                voices[index].chord_ids.remove(member);
                if voices[index].chord_ids.is_empty() {
                    voices.remove(index);
                    break;
                }
            }
            index += 1;
        }
    }
}
pub(crate) fn projection(voices: &[Voice], measures: &[Measure]) -> Value {
    json!(
        voices
            .iter()
            .map(|v| {
                let m = &measures[v.measure];
                json!({"id":v.id,"kind":"HIGH","system_id":m.system,"part_id":m.part,
            "measure_index":m.index,"starting_staff_id":v.starting_staff,"chord_ids":v.chord_ids})
            })
            .collect::<Vec<_>>()
    )
}
