// SPDX-License-Identifier: AGPL-3.0-or-later
//! MeasureRhythm.purgeExtinctVoices and SlotMapper's explicit NextInVoice
//! exception, with actual measure-scoped Voice identity and candidate abscissae.
use crate::{
    music_measure_slots::CompoundSlot, music_measures::Measure, music_voices::Voice,
    shared_sig::Graph,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
struct Chord {
    same_measure: bool,
    time_offset: Option<String>,
    duration: Option<String>,
    slot_x: Option<i64>,
    next: Option<usize>,
}
#[derive(Deserialize)]
struct Owner {
    id: usize,
    measure_rest: bool,
    last_chord: Option<usize>,
}
#[derive(Default)]
pub(crate) struct Lifetime {
    extinct: BTreeSet<usize>,
}
impl Lifetime {
    pub(crate) fn allows_active(
        &self,
        voice: usize,
        last: usize,
        rookies: &[usize],
        next: impl Iterator<Item = (usize, usize)>,
    ) -> bool {
        !self.extinct.contains(&voice)
            || next
                .into_iter()
                .any(|(rookie, active)| active == last && rookies.contains(&rookie))
    }
    fn inspect(
        &mut self,
        first_time: Option<u32>,
        current_x: i64,
        voices: &[Owner],
        chords: &BTreeMap<usize, Chord>,
    ) -> Result<bool, String> {
        let Some(first_time) = first_time else {
            return Ok(false);
        };
        let mut staged = self.extinct.clone();
        let mut abnormal = false;
        'voice: for voice in voices {
            if voice.measure_rest || staged.contains(&voice.id) {
                continue;
            }
            let last_id = voice
                .last_chord
                .ok_or("music_voice_lifetime_last_chord_required")?;
            let last = chords
                .get(&last_id)
                .ok_or("music_voice_lifetime_live_chord_required")?;
            if !last.same_measure {
                return Err("music_voice_lifetime_owned_last_required".into());
            }
            let Some(offset) = last.time_offset.as_deref() else {
                abnormal = true;
                continue;
            };
            let offset = crate::music_time::parse_ticks(offset)?;
            let duration = crate::music_time::parse_ticks(
                last.duration
                    .as_deref()
                    .ok_or("music_voice_lifetime_duration_required")?,
            )?;
            let end = offset
                .checked_add(duration)
                .ok_or("music_voice_lifetime_time_overflow")?;
            if end < first_time {
                let mut next = last.next;
                let mut visited = BTreeSet::new();
                while let Some(id) = next {
                    if !visited.insert(id) {
                        return Err("music_voice_lifetime_next_cycle_refused".into());
                    }
                    let chord = chords
                        .get(&id)
                        .ok_or("music_voice_lifetime_live_next_required")?;
                    if !chord.same_measure {
                        break;
                    }
                    if current_x
                        <= chord
                            .slot_x
                            .ok_or("music_voice_lifetime_candidate_slot_required")?
                    {
                        continue 'voice;
                    }
                    next = chord.next;
                }
                staged.insert(voice.id);
            }
        }
        self.extinct = staged;
        Ok(abnormal)
    }
    pub(crate) fn purge(
        &mut self,
        graph: &Graph,
        voices: &[Voice],
        measure: &mut Measure,
        measure_index: usize,
        candidates: &crate::music_measure_slots::CandidateReferences,
        (slot, first_time): (&CompoundSlot, Option<u32>),
    ) -> Result<Value, String> {
        let snapshot = graph.snapshot();
        let nodes = snapshot["inters"].as_array().unwrap();
        let edges = snapshot["relations"].as_array().unwrap();
        // MeasureRhythm reads the retained first narrow object's own clock,
        // supplied by the caller, not a reconstructed chord/event clock.
        let current_x = candidates.x_offset(slot.key)?;
        let mut chords = BTreeMap::new();
        for n in nodes.iter().filter(|n| {
            matches!(
                n["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        }) {
            let id = n["id"]
                .as_u64()
                .ok_or("music_voice_lifetime_chord_id_required")? as usize;
            let same_measure = n["origin"]["system"] == measure.system
                && n["part_id"] == measure.part
                && n["measure_index"] == measure.index;
            let slot_x = if same_measure {
                candidates
                    .reference(id)
                    .map(|key| candidates.x_offset(key))
                    .transpose()?
            } else {
                None
            };
            let next = edges
                .iter()
                .find(|e| e["kind"] == "NextInVoiceRelation" && e["source"] == id)
                .and_then(|e| e["target"].as_u64())
                .map(|id| id as usize);
            chords.insert(
                id,
                Chord {
                    same_measure,
                    time_offset: n["time_offset"].as_str().map(str::to_owned),
                    duration: n["duration"].as_str().map(str::to_owned),
                    slot_x,
                    next,
                },
            );
        }
        let owners = voices
            .iter()
            .filter(|v| v.measure == measure_index)
            .map(|v| Owner {
                id: v.id,
                measure_rest: v.measure_rest_chord.is_some(),
                last_chord: v.chord_ids.last().copied(),
            })
            .collect::<Vec<_>>();
        let abnormal = self.inspect(first_time, current_x, &owners, &chords)?;
        measure.timing.abnormal |= abnormal;
        Ok(
            json!({"slot_id":slot.id,"current_x":current_x,"first_time":first_time.map(crate::music_time::format_ticks),"extinct_voices":self.extinct,"measure_abnormal":measure.timing.abnormal}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_gap_boundary_sequence_exception_and_unknown_clocks() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-voice-extinction/observed.json"
        ))
        .unwrap();
        for c in original["controls"].as_array().unwrap() {
            let mut lifetime = Lifetime {
                extinct: serde_json::from_value(c["extinct_before"].clone()).unwrap(),
            };
            let owners: Vec<Owner> = serde_json::from_value(c["voices"].clone()).unwrap();
            let chords = c["chords"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| {
                    (
                        n["id"].as_u64().unwrap() as usize,
                        serde_json::from_value(n.clone()).unwrap(),
                    )
                })
                .collect();
            let first_time = c["first_time"]
                .as_str()
                .map(crate::music_time::parse_ticks)
                .transpose()
                .unwrap();
            let abnormal = lifetime
                .inspect(
                    first_time,
                    c["current_x"].as_i64().unwrap(),
                    &owners,
                    &chords,
                )
                .unwrap();
            assert_eq!(json!(lifetime.extinct), c["extinct_after"], "{}", c["name"]);
            assert_eq!(json!(abnormal), c["measure_abnormal"]);
            let rookies: Vec<usize> = serde_json::from_value(c["rookies"].clone()).unwrap();
            let actives = owners
                .iter()
                .filter(|v| !v.measure_rest)
                .filter_map(|v| {
                    v.last_chord.filter(|last| {
                        lifetime.allows_active(
                            v.id,
                            *last,
                            &rookies,
                            c["next"].as_array().unwrap().iter().map(|p| {
                                (
                                    p["rookie"].as_u64().unwrap() as usize,
                                    p["active"].as_u64().unwrap() as usize,
                                )
                            }),
                        )
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(json!(actives), c["actives"], "{}", c["name"]);
        }
    }
}
