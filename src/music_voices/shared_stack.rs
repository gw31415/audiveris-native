// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
#[cfg(test)]
pub(crate) mod image_tests;
use crate::music_measure_layout::StackOwner;

pub(crate) fn gather_actual_stack_rhythm(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut [Voice],
    measures: &mut [Measure],
    stack: &mut StackOwner,
) -> Result<(), String> {
    let mut indices = Vec::new();
    for geometry in &stack.measures {
        graph.require_actual_music_measure(geometry)?;
        if !Arc::ptr_eq(&geometry.stack, &stack.stack) {
            return Err("music_actual_shared_Stack_required".into());
        }
        let mi = measures
            .iter()
            .position(|m| {
                m.geometry
                    .as_ref()
                    .is_some_and(|g| Arc::ptr_eq(g, geometry))
            })
            .ok_or("music_actual_shared_Stack_Measure_required")?;
        if indices.contains(&mi) {
            return Err("music_actual_shared_Stack_Measure_required".into());
        }
        indices.push(mi);
    }
    if indices.is_empty() {
        return Err("music_actual_shared_Stack_Measure_required".into());
    }
    let expected = measures[indices[0]].timing.expected;
    if indices
        .iter()
        .any(|&mi| measures[mi].timing.expected != expected)
    {
        return Err("music_actual_shared_Stack_expected_required".into());
    }
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let node = |id: usize| {
        nodes
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("music_slot_live_chord_required")
    };
    let mut retained = voices.to_vec();
    let mut staged = graph.clone();
    let mut staged_measures = measures.to_vec();
    let mut times = BTreeMap::<Fraction, Vec<usize>>::new();
    for &mi in &indices {
        for voice in retained
            .iter()
            .filter(|v| v.measure == mi && v.measure_rest_chord.is_none())
        {
            for &id in &voice.chord_ids {
                let ch = node(id)?;
                if ch["origin"]["system"] != measures[mi].system
                    || ch["part_id"] != measures[mi].part
                    || ch["measure_index"] != measures[mi].index
                    || !matches!(
                        ch["kind"].as_str(),
                        Some("HeadChordInter" | "RestChordInter")
                    )
                {
                    return Err("music_slot_ordinary_chord_required".into());
                }
                if let Some(time) = ch["time_offset"].as_str() {
                    times.entry(Fraction::parse(time)?).or_default().push(id);
                } else {
                    staged_measures[mi].timing.abnormal = true;
                }
            }
        }
    }
    let mut slots = Vec::new();
    let mut time_to_slot = BTreeMap::new();
    for (time, incoming) in times {
        let mut ordered = Vec::new();
        for id in incoming {
            let b: [i64; 4] = serde_json::from_value(node(id)?["bounds"].clone())
                .map_err(|_| "music_slot_chord_bounds_required")?;
            ordered.push((id, b[1] + b[3] / 2, b[0] + b[2] / 2));
        }
        ordered.sort_by_key(|(_, y, _)| *y);
        let mean = [
            ordered.iter().map(|(_, _, x)| *x as f64).sum::<f64>() / ordered.len() as f64,
            ordered.iter().map(|(_, y, _)| *y as f64).sum::<f64>() / ordered.len() as f64,
        ];
        let x_offset = staged
            .actual_music_stack_x_offset(&stack.measures[0], mean)?
            .round_ties_even() as i64;
        let id = slots.len() + 1;
        let slot = Slot {
            identity: Arc::new(()),
            id,
            time_offset: Some(time),
            x_offset,
            incoming_ids: ordered.into_iter().map(|(id, _, _)| id).collect(),
        };
        for &id in &slot.incoming_ids {
            let key = serde_json::from_value(node(id)?["origin"].clone())
                .map_err(|_| "music_slot_actual_origin_required")?;
            staged.set_chord_slot(key, slot.id, &slot.identity)?;
        }
        time_to_slot.insert(time, slot.id);
        slots.push(slot);
    }
    for &mi in &indices {
        for voice in retained
            .iter_mut()
            .filter(|v| v.measure == mi && v.measure_rest_chord.is_none())
        {
            for &id in &voice.chord_ids {
                if let Some(time) = node(id)?["time_offset"].as_str() {
                    voice.slots.insert(
                        time_to_slot[&Fraction::parse(time)?],
                        SlotVoice {
                            chord_id: id,
                            continuing: false,
                        },
                    );
                }
            }
            // Slot IDs overwrite existing BEGIN entries. The legitimate Measure
            // reset owns table clearing; unresolved Chords keep their candidate Slot.
            let mut previous = None;
            for slot in &slots {
                if let Some(info) = voice.slots.get(&slot.id) {
                    previous = Some(info.chord_id);
                } else if let Some(id) = previous {
                    let time = Fraction::parse(
                        node(id)?["time_offset"]
                            .as_str()
                            .ok_or("music_slot_offset_required")?,
                    )?;
                    if slot_chord_end(&snapshot, id, time)?
                        > slot.time_offset.ok_or("music_slot_time_required")?
                    {
                        voice.slots.insert(
                            slot.id,
                            SlotVoice {
                                chord_id: id,
                                continuing: true,
                            },
                        );
                    }
                }
            }
        }
    }
    let slots_duration = stack_slots_duration(&staged, &slots)?;
    let actual = if !retained.iter().any(|v| indices.contains(&v.measure)) {
        Some(Fraction::ZERO)
    } else if slots_duration == Fraction::ZERO {
        expected
    } else {
        Some(slots_duration)
    };
    let excess = indices
        .iter()
        .filter_map(|&mi| staged_measures[mi].timing.excess)
        .max();
    let timing = StackTiming {
        generation: Arc::new(()),
        expected,
        excess,
        slots,
        actual,
    };
    for &mi in &indices {
        staged_measures[mi].timing.expected = timing.expected;
        staged_measures[mi].timing.excess = timing.excess;
        staged_measures[mi].timing.actual = timing.actual;
        staged_measures[mi].timing.slots = timing.slots.clone();
    }
    *graph = staged;
    voices.clone_from_slice(&retained);
    measures.clone_from_slice(&staged_measures);
    stack.timing = Some(timing);
    Ok(())
}
