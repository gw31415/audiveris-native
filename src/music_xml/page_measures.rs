// SPDX-License-Identifier: AGPL-3.0-or-later
//! Narrow PAGE MeasureFixer continuation over actual shared Stack owners.
use super::*;
use crate::music_measure_layout::PageMeasureIdentity;
use crate::music_time::Delta;
use std::sync::Arc;
mod freshness;

fn common_termination(values: impl IntoIterator<Item = Option<Delta>>) -> Option<Delta> {
    let mut common = None;
    for value in values.into_iter().flatten() {
        match common {
            Some(old) if old != value => return None,
            None => common = Some(value),
            _ => {}
        }
    }
    common
}

fn next_identity(
    first_in_page: bool,
    previous: Option<i32>,
    actual: Option<Fraction>,
    termination: Option<Delta>,
) -> Result<PageMeasureIdentity, String> {
    if actual == Some(Fraction::ZERO) {
        return Err("music_page_empty_merge_or_cautionary_pending".into());
    }
    if actual.is_none() {
        return Err("music_page_actual_duration_required".into());
    }
    let pickup = first_in_page && termination.is_some_and(Delta::is_negative);
    let number = if pickup {
        0
    } else {
        previous
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("music_page_measure_id_overflow")?
    };
    Ok(PageMeasureIdentity { number, pickup })
}

fn guard_special_symbols(snapshot: &Value) -> Result<(), String> {
    for inter in snapshot["inters"]
        .as_array()
        .ok_or("music_page_actual_SIG_required")?
    {
        if matches!(
            inter["kind"].as_str(),
            Some("MultipleRestInter" | "MeasureRepeatInter")
        ) {
            return Err("music_page_multiple_rest_or_measure_repeat_pending".into());
        }
        if inter["kind"] == "RepeatDotInter"
            || matches!(
                inter["shape"].as_str(),
                Some("LEFT_REPEAT_SIGN" | "RIGHT_REPEAT_SIGN" | "BACK_TO_BACK_REPEAT_SIGN")
            )
        {
            return Err("music_page_repeat_halves_pending".into());
        }
    }
    if snapshot["relations"]
        .as_array()
        .ok_or("music_page_actual_SIG_required")?
        .iter()
        .any(|r| r["kind"] == "RepeatDotBarRelation")
    {
        return Err("music_page_repeat_halves_pending".into());
    }
    Ok(())
}

impl OwnedMusic {
    /// Recognition-only PAGE subset. Public factory admission remains required;
    /// unseen special symbols are not certified absent on an unadmitted image.
    pub(crate) fn fix_recognized_page_measures(&mut self) -> Result<Value, String> {
        let graph = &self.owner.graph;
        guard_special_symbols(&graph.snapshot())?;
        let first_system = self
            .owner
            .systems
            .first()
            .ok_or("music_page_actual_System_required")?
            .id;
        if self.measure_layout.stacks.is_empty() {
            return Err("music_page_actual_Stack_required".into());
        }
        let mut voices = self.voices.clone();
        let mut measures = self.measures.clone();
        let mut stacks = self.measure_layout.stacks.clone();
        let mut previous = None;
        let mut rows = Vec::new();
        for (si, stack) in stacks.iter_mut().enumerate() {
            if let Some(previous) = &mut stack.page_identity {
                // MeasureFixer resets special before duration checks, retaining
                // the old number until this staged Stack receives its new ID.
                previous.pickup = false;
            }
            let timing = stack
                .timing
                .as_mut()
                .ok_or("music_page_actual_rhythm_required")?;
            if timing.expected.is_none() {
                return Err("music_page_expected_duration_required".into());
            }
            let mut indices = Vec::new();
            for geometry in &stack.measures {
                graph.require_actual_music_measure(geometry)?;
                if !Arc::ptr_eq(&geometry.stack, &stack.stack) {
                    return Err("music_page_actual_Stack_Measure_required".into());
                }
                let matches = measures
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| {
                        m.geometry
                            .as_ref()
                            .is_some_and(|g| Arc::ptr_eq(g, geometry))
                    })
                    .map(|(mi, _)| mi)
                    .collect::<Vec<_>>();
                if matches.len() != 1 || indices.contains(&matches[0]) {
                    return Err("music_page_actual_Stack_Measure_required".into());
                }
                let mi = matches[0];
                if measures[mi].timing.expected != timing.expected
                    || measures[mi].timing.actual != timing.actual
                    || measures[mi].timing.excess != timing.excess
                {
                    return Err("music_page_shared_Stack_clock_required".into());
                }
                indices.push(mi);
            }
            if indices.is_empty() {
                return Err("music_page_actual_Stack_Measure_required".into());
            }
            let before = json!({"expected":timing.expected,"actual":timing.actual,"excess":timing.excess,
                "terminations":voices.iter().filter(|v|indices.contains(&v.measure)).map(|v|v.termination.map(crate::music_time::format_delta)).collect::<Vec<_>>()});
            // Every completed Voice check publishes its shared excess to all
            // staged views before the next callback can observe this Stack.
            for &mi in &indices {
                let voice_indices = voices
                    .iter()
                    .enumerate()
                    .filter_map(|(vi, voice)| (voice.measure == mi).then_some(vi))
                    .collect::<Vec<_>>();
                for vi in voice_indices {
                    crate::music_voices::check_duration(graph, &mut voices[vi], &mut measures[mi])?;
                    timing.excess = measures[mi].timing.excess;
                    for &view in &indices {
                        measures[view].timing.excess = timing.excess;
                    }
                }
            }
            let terminations = voices
                .iter()
                .filter(|v| indices.contains(&v.measure))
                .map(|v| v.termination)
                .collect::<Vec<_>>();
            let common = common_termination(terminations.iter().copied());
            let first_in_page =
                si == 0 && stack.stack.system == first_system && stack.stack.index == 1;
            if si == 0 && !first_in_page {
                return Err("music_page_actual_first_Stack_required".into());
            }
            let identity = next_identity(first_in_page, previous, timing.actual, common)?;
            stack.page_identity = Some(identity);
            previous = Some(identity.number);
            rows.push(json!({"system":stack.stack.system,"stack_index":stack.stack.index,
                "number":identity.number,"implicit":identity.pickup,"before":before,
                "terminations":terminations.into_iter().map(|d|d.map(crate::music_time::format_delta)).collect::<Vec<_>>(),
                "common_termination":common.map(crate::music_time::format_delta),
                "expected":timing.expected,"actual":timing.actual,"excess":timing.excess}));
        }
        for stack in &mut stacks {
            stack.page_classification = Some(freshness::capture(
                graph,
                &self.chords,
                &voices,
                &measures,
                stack,
            )?);
        }
        // No Graph, actual Chord or immutable layout owner is replaced. Failure
        // before here leaves Voice, Slot, timing and prior PAGE metadata intact.
        self.voices.clone_from_slice(&voices);
        self.measures.clone_from_slice(&measures);
        for (held, staged) in self.measure_layout.stacks.iter_mut().zip(stacks) {
            held.timing = staged.timing;
            held.page_identity = staged.page_identity;
            held.page_classification = staged.page_classification;
        }
        Ok(
            json!({"stacks":rows,"measure_count":self.measure_layout.stacks.len(),
            "delta_measure_id":previous,"scope":"single recognized page; pickup and ordinary nonempty Stacks",
            "full_PAGE_step_complete":false}),
        )
    }
}

#[cfg(test)]
pub(crate) mod tests;
