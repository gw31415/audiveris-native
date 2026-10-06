// SPDX-License-Identifier: AGPL-3.0-or-later
//! PartwiseBuilder.processMeasure's Voice/BEGIN traversal on retained owners.
//! The existing one-staff image admission remains separate from this traversal.
use crate::{music_measures::Measure, music_voices::Voice, shared_sig::Graph};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fmt::Write};

pub(crate) enum Step {
    Chord {
        id: usize,
        voice: usize,
        duration: u32,
    },
    Backup(u32),
    Forward {
        voice: usize,
        duration: u32,
    },
}
impl Step {
    fn duration(&self) -> u32 {
        match *self {
            Self::Chord { duration, .. }
            | Self::Backup(duration)
            | Self::Forward { duration, .. } => duration,
        }
    }
}
/// PartwiseBuilder reads Voice and duration from the actual chord; its slot
/// traversal reads that chord's onset. Event geometry/pitch/notation are already
/// recognized, so binding timing must not change mapper ownership or its clock.
pub(crate) fn bind_event(
    graph: &Graph,
    voices: &[Voice],
    measure: &Measure,
    measure_index: usize,
    event: &mut Value,
) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let chord = nodes
        .iter()
        .find(|n| n["id"] == event["chord_id"])
        .ok_or("musicxml_voice_live_chord_required")?;
    let id = chord["id"]
        .as_u64()
        .ok_or("musicxml_voice_live_chord_required")? as usize;
    let voice = voices
        .iter()
        .find(|v| v.measure == measure_index && chord["voice"] == v.id)
        .ok_or("musicxml_voice_owned_chord_required")?;
    let measure_rest = voice.measure_rest_chord == Some(id);
    if (!measure_rest && !voice.chord_ids.contains(&id))
        || chord["origin"]["system"] != measure.system
        || chord["part_id"] != measure.part
        || chord["measure_index"] != measure.index
        || chord["staff_id"] != measure.staff
        || !matches!(
            chord["kind"].as_str(),
            Some("HeadChordInter" | "RestChordInter")
        )
        || !chord["member_ids"]
            .as_array()
            .is_some_and(|ids| ids.contains(&event["note_id"]))
    {
        return Err("musicxml_voice_owned_event_required".into());
    }
    let index = chord_notes(nodes, chord)?
        .iter()
        .position(|n| n["id"] == event["note_id"])
        .ok_or("musicxml_voice_owned_event_required")?;
    let onset = crate::music_time::parse_ticks(
        chord["time_offset"]
            .as_str()
            .ok_or("musicxml_unknown_voice_clock_pending")?,
    )?;
    let duration = if measure_rest {
        // The current owned writer requires computed stack duration. Original
        // fallback ONE for an unresolved measure-rest stack is not admitted.
        measure
            .timing
            .actual
            .ok_or("musicxml_measure_rest_actual_duration_required")?
    } else {
        crate::music_time::parse_ticks(
            chord["duration"]
                .as_str()
                .ok_or("musicxml_voice_chord_duration_required")?,
        )?
    };
    if duration == 0 {
        return Err("musicxml_exact_positive_division_required".into());
    }
    event["voice"] = json!(voice.id);
    event["chord"] = json!(index != 0);
    event["dots"] = json!(if measure_rest {
        0
    } else {
        chord["dots_number"].as_u64().unwrap_or(0)
    });
    event["time_offset"] = json!(crate::music_time::format_ticks(onset));
    event["offset_quarters"] = reported_units(onset, 4);
    event["duration"] = json!(crate::music_time::format_ticks(duration));
    event["duration_quarters"] = reported_units(duration, 4);
    event["duration_eighths"] = reported_units(duration, 2);
    event["measure_rest"] = json!(measure_rest);
    event["measure_number"] = json!(measure_index + 1);
    event["measure_index"] = json!(measure.index);
    event["system_id"] = json!(measure.system);
    event["staff_id"] = json!(measure.staff);
    Ok(())
}
fn reported_units(ticks: u32, divisor: u32) -> Value {
    if ticks.is_multiple_of(divisor) {
        json!(ticks / divisor)
    } else {
        json!(f64::from(ticks) / f64::from(divisor))
    }
}

fn chord_notes<'a>(nodes: &'a [Value], chord: &Value) -> Result<Vec<&'a Value>, String> {
    let members = chord["member_ids"]
        .as_array()
        .ok_or("musicxml_voice_owned_event_required")?;
    if members.is_empty() {
        return Err("musicxml_voice_event_required".into());
    }
    let notes = members
        .iter()
        .map(|id| {
            nodes
                .iter()
                .find(|n| n["id"] == *id)
                .ok_or("musicxml_voice_owned_event_required")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if chord["kind"] == "HeadChordInter" {
        crate::head_chord_geometry::ordered(&notes)
    } else if chord["kind"] == "RestChordInter" && notes.len() == 1 {
        Ok(notes)
    } else {
        Err("musicxml_voice_owned_event_required".into())
    }
}

pub(crate) fn sequence(
    graph: &Graph,
    voices: &[Voice],
    measure: &Measure,
    measure_index: usize,
) -> Result<Vec<Step>, String> {
    let snapshot = graph.snapshot();
    let chord = |id, voice: &Voice| -> Result<&Value, String> {
        let n = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("musicxml_voice_live_chord_required")?;
        if n["voice"] != voice.id
            || n["origin"]["system"] != measure.system
            || n["part_id"] != measure.part
            || n["measure_index"] != measure.index
            || !matches!(
                n["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("musicxml_voice_owned_chord_required".into());
        }
        Ok(n)
    };
    let mut steps = Vec::new();
    let mut cursor = 0;
    for voice in voices.iter().filter(|v| v.measure == measure_index) {
        if cursor != 0 {
            steps.push(Step::Backup(cursor));
            cursor = 0;
        }
        if let Some(id) = voice.measure_rest_chord {
            chord(id, voice)?;
            let duration = measure
                .timing
                .actual
                .ok_or("musicxml_measure_rest_actual_duration_required")?;
            steps.push(Step::Chord {
                id,
                voice: voice.id,
                duration,
            });
            cursor = duration;
        } else {
            for slot in &measure.timing.slots {
                let Some(info) = voice.slots.get(&slot.id).filter(|info| !info.continuing) else {
                    continue;
                };
                if !voice.chord_ids.contains(&info.chord_id)
                    || !slot.incoming_ids.contains(&info.chord_id)
                {
                    return Err("musicxml_voice_owned_BEGIN_required".into());
                }
                let n = chord(info.chord_id, voice)?;
                let onset = crate::music_time::parse_ticks(
                    n["time_offset"]
                        .as_str()
                        .ok_or("musicxml_unknown_voice_clock_pending")?,
                )?;
                if slot.time_offset != Some(onset) || n["slot_id"] != slot.id {
                    return Err("musicxml_voice_owned_slot_clock_required".into());
                }
                if cursor < onset {
                    steps.push(Step::Forward {
                        voice: voice.id,
                        duration: onset - cursor,
                    });
                    cursor = onset;
                }
                let duration = crate::music_time::parse_ticks(
                    n["duration"]
                        .as_str()
                        .ok_or("musicxml_voice_chord_duration_required")?,
                )?;
                steps.push(Step::Chord {
                    id: info.chord_id,
                    voice: voice.id,
                    duration,
                });
                cursor = cursor
                    .checked_add(duration)
                    .ok_or("music_time_tick_overflow")?;
            }
        }
    }
    Ok(steps)
}

pub(crate) fn divisions(sequences: &[Vec<Step>]) -> u32 {
    sequences
        .iter()
        .flatten()
        .map(|s| match s.duration() % 4 {
            0 => 1,
            2 => 2,
            _ => 4,
        })
        .max()
        .unwrap_or(1)
}
fn units(ticks: u32, divisions: u32) -> Result<u32, String> {
    let scaled = ticks
        .checked_mul(divisions)
        .ok_or("music_time_tick_overflow")?;
    if divisions == 0 || scaled == 0 || !scaled.is_multiple_of(4) {
        return Err("musicxml_exact_positive_division_required".into());
    }
    Ok(scaled / 4)
}
pub(crate) fn write_measure(
    graph: &Graph,
    xml: &mut String,
    steps: &[Step],
    events: &[&Value],
    divisions: u32,
) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("music_voice_SIG_required")?;
    let mut emitted = BTreeSet::new();
    for step in steps {
        let duration = units(step.duration(), divisions)?;
        match *step {
            Step::Backup(_) => {
                writeln!(xml, "<backup><duration>{duration}</duration></backup>").unwrap()
            }
            Step::Forward { voice, .. } => writeln!(
                xml,
                "<forward><duration>{duration}</duration><voice>{voice}</voice></forward>"
            )
            .unwrap(),
            Step::Chord { id, voice, .. } => {
                let chord = nodes
                    .iter()
                    .find(|n| n["id"] == id)
                    .ok_or("musicxml_voice_owned_event_required")?;
                let notes = chord_notes(nodes, chord)?;
                for (index, note) in notes.iter().enumerate() {
                    let matching = events
                        .iter()
                        .filter(|e| e["chord_id"] == id && e["note_id"] == note["id"])
                        .collect::<Vec<_>>();
                    if matching.len() != 1 {
                        return Err("musicxml_voice_unique_owned_event_required".into());
                    }
                    let e = matching[0];
                    let note_id = note["id"]
                        .as_u64()
                        .ok_or("musicxml_voice_owned_event_required")?;
                    if chord["voice"] != voice || e["voice"] != voice || !emitted.insert(note_id) {
                        return Err("musicxml_voice_unique_owned_event_required".into());
                    }
                    let dots = chord["dots_number"].as_u64().unwrap_or(0);
                    if dots > 2 || e["dots"] != dots {
                        return Err("musicxml_voice_owned_dot_count_required".into());
                    }
                    xml.push_str("<note>");
                    if index != 0 {
                        xml.push_str("<chord/>");
                    }
                    if e["rest"] == true {
                        xml.push_str(if e["measure_rest"] == true {
                            r#"<rest measure="yes"/>"#
                        } else {
                            "<rest/>"
                        });
                    } else {
                        write!(
                            xml,
                            "<pitch><step>{}</step>{}<octave>{}</octave></pitch>",
                            e["step"].as_str().unwrap(),
                            if e["alter"] == 0 {
                                String::new()
                            } else {
                                format!("<alter>{}</alter>", e["alter"])
                            },
                            e["octave"]
                        )
                        .unwrap();
                    }
                    writeln!(
                        xml,
                        "<duration>{duration}</duration><voice>{voice}</voice>{}{}{}{}</note>",
                        e["type"]
                            .as_str()
                            .map(|t| format!("<type>{t}</type>"))
                            .unwrap_or_default(),
                        "<dot/>".repeat(dots as usize),
                        e["stem_direction"]
                            .as_str()
                            .map(|d| format!("<stem>{d}</stem>"))
                            .unwrap_or_default(),
                        e["beams"]
                            .as_array()
                            .map(|beams| beams
                                .iter()
                                .map(|b| format!(
                                    "<beam number=\"{}\">{}</beam>",
                                    b["number"],
                                    b["value"].as_str().unwrap()
                                ))
                                .collect::<String>())
                            .unwrap_or_default()
                    )
                    .unwrap();
                }
            }
        }
    }
    if emitted.len() != events.len() {
        return Err("musicxml_voice_unexported_event_required".into());
    }
    Ok(())
}
