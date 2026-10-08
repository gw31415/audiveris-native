// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measure_slots::{CompoundSlot, Retrieval};
use crate::music_measures::{Chord, Measure};
use crate::music_voices::Voice;

pub(crate) struct Context<'a> {
    pub graph: &'a mut crate::shared_sig::Graph,
    pub voices: &'a mut Vec<Voice>,
    pub measures: &'a mut [Measure],
    pub chords: &'a [Chord],
    pub interline: usize,
}
pub(super) struct Prepared<'a> {
    pub chords: &'a [&'a Chord],
    pub narrow: &'a Retrieval,
    pub compounds: &'a [CompoundSlot],
    pub expected: Fraction,
    pub clef: &'a Value,
    pub measure_index: usize,
}
impl Context<'_> {
    pub(super) fn map(&mut self, prepared: Prepared<'_>) -> Result<Value, String> {
        let Prepared {
            chords,
            narrow,
            compounds,
            expected: expected_duration,
            clef: initial_clef,
            measure_index: mi,
        } = prepared;
        let graph = &mut *self.graph;
        let voices = &mut *self.voices;
        let measures = &mut *self.measures;
        let interline = self.interline;
        let centers = chords
            .iter()
            .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
            .collect::<BTreeMap<_, _>>();
        for slot in compounds {
            for id in &slot.chord_ids {
                let chord = chords
                    .iter()
                    .find(|c| c.id == *id)
                    .ok_or("music_candidate_actual_chord_required")?;
                graph.set_chord_slot(chord.key, slot.id, &slot.identity)?;
            }
        }
        let mut observations = json!({});
        // MeasureRhythm.processStartingChords selects measure rests plus the
        // first compound's chords, in stable center-ordinate order.
        let mut starting = chords
            .iter()
            .filter(|c| {
                c.kind == crate::music_measures::ChordKind::Rest
                    && c.members
                        .first()
                        .and_then(|n| n["shape"].as_str())
                        .is_some_and(|s| crate::music_voices::is_measure_rest_shape(s, false))
            })
            .map(|c| c.id)
            .collect::<Vec<_>>();
        if let Some(first) = compounds.first() {
            starting.extend(&first.chord_ids);
        }
        starting.sort_by_key(|id| centers[id]);
        if starting.len() != 1
            && starting.iter().any(|id| {
                chords
                    .iter()
                    .any(|c| c.id == *id && c.kind == crate::music_measures::ChordKind::Rest)
            })
        {
            return Err("musicxml_starting_parallel_voice_mapping_pending".into());
        }
        let seeds = starting
            .iter()
            .map(|id| {
                chords
                    .iter()
                    .find(|c| c.id == *id)
                    .copied()
                    .ok_or("musicxml_starting_owned_chord_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        crate::music_voices::process_starting_chords(graph, voices, measures, mi, &seeds)?;
        let start_snapshot = graph.snapshot();
        observations["starting_chord_ids"] = json!(starting);
        observations["starting_state"] = json!({"voices":voices.iter().filter(|v| v.measure == mi).map(|v| json!({"id":v.id,"is_measure_rest":v.measure_rest_chord.is_some(),"chord_ids":v.chord_ids,"measure_rest_chord_id":v.measure_rest_chord})).collect::<Vec<_>>(),"chords":chords.iter().map(|c| {let n=start_snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==c.id).unwrap();json!({"id":c.id,"bounds":n["bounds"],"duration":n["duration"],"time_offset":n["time_offset"],"voice":n["voice"]})}).collect::<Vec<_>>()});
        observations["mapping_steps"] = json!([]);
        observations["mapping_validation_steps"] = json!([]);
        observations["extinction_steps"] = json!([]);
        let mut lifetime = crate::music_voice_lifetime::Lifetime::default();
        let candidates =
            crate::music_measure_slots::CandidateReferences::new(graph, &measures[mi], compounds)?;
        let mut slot_times = crate::music_measure_slots::ClockState::new(compounds)?;
        let mut order = crate::music_measure_slots::CandidateOrder::new(compounds, &candidates)?;
        observations["candidate_time_steps"] = json!([]);
        let mut candidate_index = 0;
        while let Some(slot) = order.at(candidate_index).cloned() {
            let before = slot_times.projection(&slot)?;
            let mut step = crate::music_rookie_mapping::Context {
                graph,
                voices,
                measures,
                chords,
                narrow,
                measure_index: mi,
                candidates: &candidates,
                order: &mut order,
                expected: expected_duration,
                clef: initial_clef,
                interline,
                lifetime: &lifetime,
                slot_times: &mut slot_times,
            }
            .map_slot(&slot)?;
            let validation = step.as_object_mut().unwrap().remove("validation").unwrap();
            observations["mapping_validation_steps"]
                .as_array_mut()
                .unwrap()
                .push(json!({"slot_id":slot.id,"validation":validation}));
            observations["mapping_steps"]
                .as_array_mut()
                .unwrap()
                .push(step);
            observations["candidate_time_steps"].as_array_mut().unwrap().push(json!({"slot_id":slot.id,"before":before,"after":slot_times.projection(&slot)?,"ok":true}));
            if order.contains(slot.key) {
                let first_time = slot_times.first_time(&slot)?;
                let extinction = lifetime.purge(
                    graph,
                    voices,
                    &mut measures[mi],
                    mi,
                    &candidates,
                    (&slot, first_time),
                )?;
                observations["extinction_steps"]
                    .as_array_mut()
                    .unwrap()
                    .push(extinction);
                candidate_index += 1;
            }
        }
        for chord in chords {
            if let Some(key) = candidates.reference(chord.id) {
                let slot = order.get(key)?;
                graph.set_chord_slot(chord.key, slot.id, &slot.identity)?;
            }
        }
        observations["candidate_order_after"] =
            json!(order.ordered().map(|s| s.projection()).collect::<Vec<_>>());
        Ok(observations)
    }
    pub(crate) fn process_measure(&mut self, mi: usize) -> Result<Value, String> {
        let measure = self
            .measures
            .get(mi)
            .ok_or("music_selected_actual_measure_required")?
            .clone();
        let expected = measure
            .timing
            .expected
            .ok_or("music_reprocess_unknown_expected_export_pending")?;
        let owned_chords = self.chords;
        let mut chords = owned_chords
            .iter()
            .filter(|c| c.measure == mi)
            .collect::<Vec<_>>();
        if chords.is_empty() {
            return Err("music_selected_actual_chords_required".into());
        }
        crate::music_voices::reset_rhythm(
            self.graph,
            self.voices,
            &mut self.measures[mi],
            owned_chords,
            mi,
        )?;
        let (narrow, wide) =
            crate::music_measure_slots::from_owned(self.graph, &measure, &chords, self.interline)?;
        let centers = chords
            .iter()
            .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
            .collect();
        let compounds =
            crate::music_measure_slots::build_compounds(&narrow.slots, &wide.slots, &centers)?;
        let ordered = compounds
            .iter()
            .flat_map(|c| c.members.iter())
            .flat_map(|s| s.chord_ids.iter())
            .copied()
            .collect::<Vec<_>>();
        if ordered
            != narrow
                .slots
                .iter()
                .flat_map(|s| s.chord_ids.iter())
                .copied()
                .collect::<Vec<_>>()
        {
            return Err("musicxml_inconsistent_compound_partition_pending".into());
        }
        chords.sort_by_key(|c| ordered.iter().position(|id| *id == c.id));
        let snapshot = self.graph.snapshot();
        let first_x = chords.iter().map(|c| c.bounds[0]).min().unwrap();
        let clef = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                n["kind"] == "ClefInter"
                    && n["origin"]["system"] == measure.system
                    && n["staff_id"] == measure.staff
                    && n["bounds"][0].as_i64().is_some_and(|x| x < first_x)
            })
            .max_by_key(|n| n["bounds"][0].as_i64())
            .ok_or("musicxml_measure_local_clef_required")?;
        resolve_durations(self.graph, &chords, RhythmScope::OwnedRhythm)?;
        let observations = self.map(Prepared {
            chords: &chords,
            narrow: &narrow,
            compounds: &compounds,
            expected,
            clef,
            measure_index: mi,
        })?;
        if !crate::music_voices::final_check(
            self.graph,
            self.voices,
            &mut self.measures[mi],
            owned_chords,
            mi,
        )? {
            return Err("musicxml_original_measure_final_check_failed".into());
        }
        Ok(observations)
    }
}
impl OwnedMusic {
    /// Rebuild a compared ordinary one-Measure stack from real Chords before
    /// system Voice refinement. Image/XML admission remains a separate stage.
    pub fn reprocess_stack_rhythm(&mut self, measure_number: usize) -> Result<(), String> {
        let mi = measure_number
            .checked_sub(1)
            .filter(|i| *i < self.measures.len())
            .ok_or("music_selected_actual_measure_required")?;
        let expected = self.measures[mi]
            .timing
            .expected
            .ok_or("music_reprocess_unknown_expected_export_pending")?;
        let mut graph = self.owner.graph.clone();
        let mut voices = self.voices.clone();
        let mut measures = self.measures.clone();
        crate::music_voices::reset_stack_rhythm(&mut measures[mi], Some(expected));
        Context {
            graph: &mut graph,
            voices: &mut voices,
            measures: &mut measures,
            chords: &self.chords,
            interline: self.owner.interline,
        }
        .process_measure(mi)?;
        crate::music_voices::generate_slots(&mut graph, &mut voices, &mut measures[mi], mi)?;
        crate::music_voices::read_stack_actual_duration(&graph, &voices, &mut measures[mi], mi)?;
        self.owner.graph = graph;
        self.voices = voices;
        self.measures[mi].timing = measures.remove(mi).timing;
        Ok(())
    }
}
