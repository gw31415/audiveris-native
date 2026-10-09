// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measure_layout::StackOwner;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct TimeRange {
    pub start: usize,
    pub stop: usize,
    pub signature: Option<Key>,
    pub meter: Option<(u32, u32)>,
    pub duration: Option<Fraction>,
}

impl OwnedMusic {
    pub(crate) fn recognized_time_ranges(&self) -> Result<Vec<TimeRange>, String> {
        let graph = &self.owner.graph;
        let snapshot = graph.snapshot();
        let mut times = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                matches!(
                    n["kind"].as_str(),
                    Some("TimePairInter" | "TimeWholeInter" | "TimeCustomInter")
                )
            })
            .collect::<Vec<_>>();
        times.sort_by_key(|n| n["bounds"][0].as_i64());
        let mut ranges = Vec::new();
        for (si, stack) in self.measure_layout.stacks.iter().enumerate() {
            let mut selected = None;
            for geometry in &stack.measures {
                graph.require_actual_music_measure(geometry)?;
                let part = self
                    .measure_layout
                    .parts
                    .iter()
                    .find(|p| Arc::ptr_eq(&p.part, &geometry.part))
                    .ok_or("music_actual_time_Part_required")?;
                for n in &times {
                    if n["origin"]["system"] != stack.stack.system {
                        continue;
                    }
                    let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
                        .map_err(|_| "music_time_bounds_required")?;
                    let point = [(b[0] + b[2] / 2) as f64, (b[1] + b[3] / 2) as f64];
                    let Some(held) =
                        part.measure_at(&self.owner.staffs, &self.owner.staff_areas, point)?
                    else {
                        continue;
                    };
                    if !Arc::ptr_eq(&held, geometry) {
                        continue;
                    }
                    let staff = if let Some(id) = n["staff_id"].as_u64() {
                        let Some(s) = geometry.part.staffs.iter().find(|s| s.id == id) else {
                            continue;
                        };
                        s
                    } else {
                        return Err("music_time_null_Staff_original_fallback_pending".into());
                    };
                    let [left, right] = geometry.abscissae(staff)?;
                    if !((left as f64) < point[0] && point[0] <= right as f64) {
                        continue;
                    }
                    let meter = match (n["numerator"].as_u64(), n["denominator"].as_u64()) {
                        (Some(a), Some(b)) => Some((
                            u32::try_from(a).map_err(|_| "music_time_rational_overflow")?,
                            u32::try_from(b).map_err(|_| "music_time_rational_overflow")?,
                        )),
                        _ => n["shape"].as_str().and_then(music_symbols::time),
                    };
                    if let Some(meter) = meter {
                        selected = Some((
                            serde_json::from_value(n["origin"].clone())
                                .map_err(|_| "music_time_actual_origin_required")?,
                            meter,
                        ));
                        break;
                    }
                }
                if selected.is_some() {
                    break;
                }
            }
            if let Some((signature, meter)) = selected {
                ranges.push(TimeRange {
                    start: si + 1,
                    stop: 0,
                    signature: Some(signature),
                    meter: Some(meter),
                    duration: Some(Fraction::new(meter.0, meter.1)?),
                });
            }
        }
        if ranges.first().is_none_or(|r| r.start > 1) {
            ranges.insert(
                0,
                TimeRange {
                    start: 1,
                    stop: 0,
                    signature: None,
                    meter: None,
                    duration: None,
                },
            );
        }
        for i in 0..ranges.len() {
            ranges[i].stop = ranges
                .get(i + 1)
                .map_or(self.measure_layout.stacks.len(), |r| r.start - 1);
        }
        Ok(ranges)
    }

    pub fn process_recognized_page_rhythm(&mut self) -> Result<Value, String> {
        self.process_actual_page_rhythm(|_, _, _, _, _, _| Ok(()))
    }

    pub(super) fn process_actual_page_rhythm<F>(&mut self, mut observe: F) -> Result<Value, String>
    where
        F: FnMut(
            &str,
            u64,
            &crate::shared_sig::Graph,
            &[Voice],
            &[Measure],
            &[StackOwner],
        ) -> Result<(), String>,
    {
        let ranges = self.recognized_time_ranges()?;
        if ranges.iter().any(|r| r.duration.is_none()) {
            return Err("music_recognized_page_unknown_time_range_no_default".into());
        }
        let mut graph = self.owner.graph.clone();
        let mut voices = self.voices.clone();
        let mut measures = self.measures.clone();
        let mut stacks = self.measure_layout.stacks.clone();
        let mut results = Vec::new();
        for si in 0..stacks.len() {
            let expected = ranges
                .iter()
                .find(|r| r.start <= si + 1 && si < r.stop)
                .ok_or("music_actual_time_range_required")?
                .duration;
            let system = stacks[si].stack.system;
            let mut indices = Vec::new();
            for geometry in &stacks[si].measures {
                graph.require_actual_music_measure(geometry)?;
                let mi = measures
                    .iter()
                    .position(|m| {
                        m.geometry
                            .as_ref()
                            .is_some_and(|g| Arc::ptr_eq(g, geometry))
                    })
                    .ok_or("music_actual_page_Measure_required")?;
                indices.push(mi);
            }
            stacks[si].timing = Some(crate::music_voices::StackTiming {
                expected,
                ..Default::default()
            });
            for &mi in &indices {
                // Stack reset precedes callbacks; Measure reset alone owns old
                // Voice/Chord clock clearing and its abnormal flag.
                measures[mi].timing.expected = expected;
                measures[mi].timing.excess = None;
                measures[mi].timing.slots.clear();
                measures[mi].timing.actual = None;
            }
            for mi in indices {
                let (_, ok) = Context {
                    graph: &mut graph,
                    voices: &mut voices,
                    measures: &mut measures,
                    chords: &self.chords,
                    interline: self.owner.interline,
                    rest_geometry: super::super::beam_rest::RestGeometry {
                        staffs: &self.owner.staffs,
                        skew: self.owner.sheet_skew,
                        interline: self.owner.interline,
                    },
                }
                .process_recognition_measure(mi)?;
                if !ok {
                    measures[mi].timing.abnormal = true;
                }
                results.push(json!({"system":system,"part":measures[mi].part,"measure_index":measures[mi].index,"ok":ok}));
            }
            crate::music_voices::shared_stack::gather_actual_stack_rhythm(
                &mut graph,
                &mut voices,
                &mut measures,
                &mut stacks[si],
            )?;
            if stacks.get(si + 1).is_none_or(|s| s.stack.system != system) {
                observe(
                    "BEFORE_SYSTEM_REFINEMENT",
                    system,
                    &graph,
                    &voices,
                    &measures,
                    &stacks,
                )?;
                crate::music_voices::refine_ordinary_system(
                    &mut graph,
                    &mut voices,
                    &measures,
                    system,
                )?;
                observe(
                    "AFTER_SYSTEM_REFINEMENT",
                    system,
                    &graph,
                    &voices,
                    &measures,
                    &stacks,
                )?;
            }
        }
        self.owner.graph = graph;
        self.voices = voices;
        self.measures.clone_from(&measures);
        for (held, staged) in self.measure_layout.stacks.iter_mut().zip(stacks) {
            held.timing = staged.timing;
        }
        Ok(
            json!({"time_ranges":ranges.iter().map(|r|json!({"start":r.start,"stop":r.stop,"signature":r.signature,"meter":r.meter,"duration":r.duration})).collect::<Vec<_>>(),"measure_checks":results}),
        )
    }
}

#[cfg(test)]
pub(crate) mod image_tests;
