// SPDX-License-Identifier: AGPL-3.0-or-later
//! PartwiseBuilder.processMeasure's Voice/BEGIN traversal on retained owners.
//! The existing one-staff image admission remains separate from this traversal.
use crate::music_time::Fraction;
use crate::{music_measures::Measure, music_voices::Voice, shared_sig::Graph};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fmt::Write};

pub(crate) enum Step {
    Chord {
        id: usize,
        voice: usize,
        duration: Fraction,
    },
    Backup(Fraction),
    Forward {
        voice: usize,
        duration: Fraction,
        staff_number: Option<usize>,
    },
}
impl Step {
    fn duration(&self) -> Fraction {
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
    let note = nodes
        .iter()
        .find(|n| n["id"] == event["note_id"])
        .ok_or("musicxml_voice_live_note_required")?;
    let staff_id = note["staff_id"]
        .as_u64()
        .ok_or("musicxml_voice_actual_note_Staff_required")?;
    let staff_valid = if let Some(geometry) = &measure.geometry {
        graph.require_actual_music_measure(geometry)?;
        geometry
            .part
            .staffs
            .iter()
            .any(|s| chord["staff_id"] == s.id)
            && geometry.part.staffs.iter().any(|s| s.id == staff_id)
    } else {
        chord["staff_id"] == measure.staff && staff_id == measure.staff
    };
    if !staff_valid {
        return Err("musicxml_voice_owned_Part_Staff_required".into());
    }
    let index = chord_notes(nodes, chord)?
        .iter()
        .position(|n| n["id"] == event["note_id"])
        .ok_or("musicxml_voice_owned_event_required")?;
    let onset = Fraction::parse(
        chord["time_offset"]
            .as_str()
            .ok_or("musicxml_unknown_voice_clock_pending")?,
    )?;
    let (duration, note_type) = if measure_rest {
        // The current owned writer requires computed stack duration. Original
        // fallback ONE for an unresolved measure-rest stack is not admitted.
        (
            measure
                .timing
                .actual
                .ok_or("musicxml_measure_rest_actual_duration_required")?,
            None,
        )
    } else {
        let live = crate::music_duration::live_exact(&snapshot, id)?;
        (
            live.intrinsic
                .ok_or("musicxml_voice_chord_duration_required")?,
            Some(live.note_type),
        )
    };
    if duration == Fraction::ZERO {
        return Err("musicxml_exact_positive_division_required".into());
    }
    if let Some(note_type) = note_type {
        event["type"] = json!(note_type);
    }
    event["voice"] = json!(voice.id);
    event["chord"] = json!(index != 0);
    event["dots"] = json!(if measure_rest {
        0
    } else {
        chord["dots_number"].as_u64().unwrap_or(0)
    });
    event["time_offset"] = json!(onset.to_string());
    event["offset_quarters"] = onset.reported_units(4);
    event["duration"] = json!(duration.to_string());
    event["duration_quarters"] = duration.reported_units(4);
    event["duration_eighths"] = duration.reported_units(8);
    event["measure_rest"] = json!(measure_rest);
    event["measure_number"] = json!(measure_index + 1);
    event["measure_index"] = json!(measure.index);
    event["system_id"] = json!(measure.system);
    event["staff_id"] = json!(staff_id);
    Ok(())
}

pub(crate) fn chord_notes<'a>(nodes: &'a [Value], chord: &Value) -> Result<Vec<&'a Value>, String> {
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
    let mut cursor = Fraction::ZERO;
    for voice in voices.iter().filter(|v| v.measure == measure_index) {
        if cursor != Fraction::ZERO {
            steps.push(Step::Backup(cursor));
            cursor = Fraction::ZERO;
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
                let onset = Fraction::parse(
                    n["time_offset"]
                        .as_str()
                        .ok_or("musicxml_unknown_voice_clock_pending")?,
                )?;
                if slot.time_offset != Some(onset) || n["slot_id"] != slot.id {
                    return Err("musicxml_voice_owned_slot_clock_required".into());
                }
                if cursor < onset {
                    let staff_number = if let Some(geometry) = measure
                        .geometry
                        .as_ref()
                        .filter(|g| g.part.staffs.len() > 1)
                    {
                        graph.require_actual_music_measure(geometry)?;
                        let notes = chord_notes(snapshot["inters"].as_array().unwrap(), n)?;
                        let top = notes
                            .iter()
                            .map(|note| {
                                geometry
                                    .part
                                    .staffs
                                    .iter()
                                    .position(|s| note["staff_id"] == s.id)
                                    .map(|i| i + 1)
                                    .ok_or("musicxml_forward_actual_Part_Staff_required")
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .min()
                            .ok_or("musicxml_forward_actual_Chord_members_required")?;
                        Some(top)
                    } else {
                        None
                    };
                    steps.push(Step::Forward {
                        voice: voice.id,
                        staff_number,
                        duration: onset
                            .difference(cursor)?
                            .excess()
                            .ok_or("musicxml_exact_positive_division_required")?,
                    });
                    cursor = onset;
                }
                let duration = crate::music_duration::live_exact(&snapshot, info.chord_id)?
                    .intrinsic
                    .ok_or("musicxml_voice_chord_duration_required")?;
                steps.push(Step::Chord {
                    id: info.chord_id,
                    voice: voice.id,
                    duration,
                });
                cursor = cursor.plus(duration)?;
            }
        }
    }
    // Original ending-forward code is disabled: underfilled Voice stays short.
    Ok(steps)
}

/// Page.computeDurationDivisor uses standard Chord durations, not inserted
/// backup/forward steps; PartwiseBuilder converts QUARTER by that denominator.
pub(crate) fn divisions(
    graph: &Graph,
    measures: &[Measure],
    chords: &[crate::music_measures::Chord],
) -> Result<u32, String> {
    let snapshot = graph.snapshot();
    let mut divisor = 1_u32;
    for chord in chords {
        let measure = measures
            .get(chord.measure)
            .ok_or("musicxml_divisions_owned_measure_required")?;
        let node = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("musicxml_divisions_live_chord_required")?;
        if graph.id(chord.key) != Some(chord.id)
            || node["origin"]["system"] != measure.system
            || node["part_id"] != measure.part
            || node["measure_index"] != measure.index
        {
            return Err("musicxml_divisions_owned_chord_required".into());
        }
        let duration = crate::music_duration::live_exact(&snapshot, chord.id)?
            .intrinsic
            .or(measure.timing.expected);
        if let Some(duration) = duration {
            let denominator = duration.denominator();
            let (mut a, mut b) = (divisor, denominator);
            while b != 0 {
                (a, b) = (b, a % b);
            }
            divisor = divisor
                .checked_mul(denominator / a)
                .ok_or("music_time_rational_overflow")?;
        }
    }
    // Preserve existing positive divisions for coarser-only native pages.
    // Original zero-division rest pages remain separately unverified/rejected.
    if divisor < 4 {
        return Ok(1);
    }
    if !divisor.is_multiple_of(4) {
        return Err("musicxml_nonintegral_quarter_divisions_pending".into());
    }
    Ok(divisor / 4)
}
fn units(duration: Fraction, divisions: u32) -> Result<u32, String> {
    let units = duration.units(
        divisions
            .checked_mul(4)
            .ok_or("music_time_rational_overflow")?,
    )?;
    if units == 0 {
        return Err("musicxml_exact_positive_division_required".into());
    }
    Ok(units)
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
            Step::Forward {
                voice,
                staff_number,
                ..
            } => writeln!(
                xml,
                "<forward><duration>{duration}</duration><voice>{voice}</voice>{}</forward>",
                staff_number
                    .map(|s| format!("<staff>{s}</staff>"))
                    .unwrap_or_default()
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
                    if index == 0 {
                        for d in e["directions"].as_array().into_iter().flatten() {
                            write!(
                                xml,
                                "<direction placement=\"{}\"><direction-type>",
                                d["placement"].as_str().unwrap()
                            )
                            .unwrap();
                            if d["kind"] == "dynamics" {
                                write!(
                                    xml,
                                    "<dynamics default-y=\"{}\" relative-x=\"{}\"><{}/></dynamics>",
                                    d["default_y"],
                                    d["relative_x"],
                                    d["signature"].as_str().unwrap()
                                )
                                .unwrap();
                            } else {
                                write!(xml,"<pedal type=\"{}\" line=\"no\" halign=\"left\" default-x=\"{}\" default-y=\"{}\"/>",d["type"].as_str().unwrap(),d["default_x"],d["default_y"]).unwrap();
                            }
                            xml.push_str("</direction-type>");
                            if let Some(staff) = d["staff_number"].as_u64() {
                                write!(xml, "<staff>{staff}</staff>").unwrap();
                            }
                            if d["kind"] == "pedal" {
                                write!(
                                    xml,
                                    "<sound damper-pedal=\"{}\"/>",
                                    if d["type"] == "start" { "yes" } else { "no" }
                                )
                                .unwrap();
                            } else if let Some(sound) = d["sound"].as_u64() {
                                write!(xml, "<sound dynamics=\"{sound}\"/>").unwrap();
                            }
                            xml.push_str("</direction>\n");
                        }
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
                        "<duration>{duration}</duration><voice>{voice}</voice>{}{}{}{}{}{}{}</note>",
                        e["type"]
                            .as_str()
                            .map(|t| format!("<type>{t}</type>"))
                            .unwrap_or_default(),
                        "<dot/>".repeat(dots as usize),
                        match e["accidental"].as_str() {
                            None => String::new(),
                            Some(shape) => {
                                let value=match shape {"NATURAL"=>"natural","FLAT"=>"flat","DOUBLE_FLAT"=>"flat-flat","SHARP"=>"sharp","DOUBLE_SHARP"=>"double-sharp",_=>return Err("musicxml_accidental_shape_pending".into())};
                                format!("<accidental>{value}</accidental>")
                            }
                        },
                        e["stem_direction"]
                            .as_str()
                            .map(|d| format!("<stem>{d}</stem>"))
                            .unwrap_or_default(),
                        e["staff_number"].as_u64().map(|s|format!("<staff>{s}</staff>")).unwrap_or_default(),
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
                            .unwrap_or_default(),
                        e["articulations"].as_array().filter(|a|!a.is_empty()).map(|arts| {
                            let mut value=String::from("<notations><articulations>");
                            for a in arts {
                                write!(value,"<{} placement=\"{}\"{} />",a["name"].as_str().unwrap(),a["placement"].as_str().unwrap(),a["type"].as_str().map(|t|format!(" type=\"{t}\"")).unwrap_or_default()).unwrap();
                            }
                            value.push_str("</articulations></notations>");value
                        }).unwrap_or_default()
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
