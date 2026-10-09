// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{music_measures::ChordKind, music_voice_export::Step};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Default, Debug, PartialEq, Eq)]
struct HeaderState {
    keys: Vec<Option<i64>>,
    clefs: Vec<Option<ClefAttributes>>,
}

#[derive(Debug, PartialEq, Eq)]
struct HeaderChanges {
    keys: Vec<(Option<usize>, i64)>,
    clefs: Vec<(usize, ClefAttributes)>,
}

type MeasureHeaders = (Vec<Option<i64>>, Vec<Option<ClefAttributes>>);

impl HeaderState {
    fn advance(
        &mut self,
        keys: &[Option<i64>],
        clefs: &[Option<ClefAttributes>],
        system_start: bool,
    ) -> Result<HeaderChanges, String> {
        if keys.is_empty() || keys.len() != clefs.len() {
            return Err("musicxml_logical_header_Staff_count_required".into());
        }
        if self.keys.is_empty() {
            self.keys.resize(keys.len(), None);
            self.clefs.resize(keys.len(), None);
        }
        if self.keys.len() != keys.len() {
            return Err("musicxml_logical_Part_Staff_continuity_required".into());
        }
        let mut changes = HeaderChanges {
            keys: Vec::new(),
            clefs: Vec::new(),
        };
        if keys.iter().any(Option::is_some) {
            if let Some(first) = keys[0]
                && keys.iter().all(|k| *k == Some(first))
            {
                if self.keys.iter().any(|k| *k != Some(first)) {
                    changes.keys.push((None, first));
                    self.keys.fill(Some(first));
                }
            } else {
                for (index, &key) in keys.iter().enumerate() {
                    if let Some(key) = key
                        && self.keys[index] != Some(key)
                    {
                        changes.keys.push((Some(index + 1), key));
                        self.keys[index] = Some(key);
                    }
                }
            }
        } else if system_start && self.keys.iter().any(|k| k.is_some_and(|k| k != 0)) {
            // processKey(null) does nothing. Only the whole-Part void at a
            // system start cancels a previously exported nonzero signature.
            changes.keys.push((None, 0));
            self.keys.fill(Some(0));
        }
        for (index, &clef) in clefs.iter().enumerate() {
            if let Some(clef) = clef
                && self.clefs[index] != Some(clef)
            {
                changes.clefs.push((index + 1, clef));
                self.clefs[index] = Some(clef);
            }
        }
        Ok(changes)
    }
}

// The as-built image path has actual Measure/Staff ownership, but no ordered
// Measure Key/Clef registry yet. Do not claim this spatial observation handles
// add/remove lifecycle or substitute Graph order for multiple registered keys.
fn actual_headers(
    music: &OwnedMusic,
    snapshot: &Value,
    measure_index: usize,
    first_slot: Option<i64>,
) -> Result<MeasureHeaders, String> {
    let measure = &music.measures[measure_index];
    let geometry = measure
        .geometry
        .as_ref()
        .ok_or("musicxml_actual_Measure_required")?;
    music.owner.graph.require_actual_music_measure(geometry)?;
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("musicxml_actual_header_SIG_required")?;
    let mut keys = Vec::new();
    let mut clefs = Vec::new();
    for staff in &geometry.part.staffs {
        if staff.system != measure.system
            || !music
                .owner
                .staffs
                .iter()
                .any(|held| Arc::ptr_eq(staff, held))
        {
            return Err("musicxml_actual_header_Staff_required".into());
        }
        let [left, right] = geometry.abscissae(staff)?;
        let mut actual_keys = Vec::new();
        let mut actual_clefs = Vec::new();
        for inter in nodes.iter().filter(|n| {
            n["origin"]["system"] == measure.system
                && n["staff_id"] == staff.id
                && matches!(n["kind"].as_str(), Some("KeyInter" | "ClefInter"))
        }) {
            let point = center(inter)?;
            if left < point[0] && point[0] <= right {
                if inter["kind"] == "KeyInter" {
                    actual_keys.push(inter);
                } else {
                    actual_clefs.push((point, inter));
                }
            }
        }
        if actual_keys.len() > 1 {
            return Err("musicxml_multiple_measure_Key_registration_pending".into());
        }
        keys.push(
            actual_keys
                .first()
                .map(|k| {
                    k["fifths"]
                        .as_i64()
                        .ok_or("musicxml_actual_key_fifths_required")
                })
                .transpose()?,
        );
        // ClefIterators can emit several timed changes. Until that is connected,
        // only one actual clef per Staff/Measure before the first Slot is admitted.
        if actual_clefs.len() > 1 {
            return Err("musicxml_multiple_measure_Clef_events_pending".into());
        }
        let clef = actual_clefs
            .first()
            .map(|(point, inter)| {
                if let Some(first_slot) = first_slot {
                    // ClefIterators uses getXOffset(point, singletonList(staff)),
                    // rather than the surrounding-Staff interpolation overload.
                    let side = geometry.left_point(&music.owner.graph, staff)?;
                    let angle = geometry.skew.slope().atan();
                    let offset = angle.cos() * (point[0] as f64 - side[0])
                        + angle.sin() * (point[1] as f64 - side[1]);
                    if !offset.is_finite() || offset > first_slot as f64 {
                        return Err("musicxml_mid_measure_clef_change_pending".into());
                    }
                }
                clef_attributes(inter)
            })
            .transpose()?;
        clefs.push(clef);
    }
    Ok((keys, clefs))
}

fn node(snapshot: &Value, id: usize) -> Result<&Value, String> {
    snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .ok_or("musicxml_recognized_live_Inter_required".into())
}
fn center(n: &Value) -> Result<[i64; 2], String> {
    let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
        .map_err(|_| "musicxml_recognized_bounds_required")?;
    Ok([b[0] + b[2] / 2, b[1] + b[3] / 2])
}
fn beams(
    music: &OwnedMusic,
    snapshot: &Value,
    chord: &crate::music_measures::Chord,
) -> Result<Value, String> {
    let mut marks = Vec::new();
    for (index, id) in crate::music_voices::chord_beams(snapshot, chord.id)?
        .iter()
        .enumerate()
    {
        let beam = node(snapshot, *id)?;
        let value = if beam["kind"] == "BeamHookInter" {
            let stem = chord.stem.ok_or("musicxml_hook_actual_stem_required")?;
            if center(beam)?[0]
                > center(node(
                    snapshot,
                    music
                        .owner
                        .graph
                        .id(stem)
                        .ok_or("musicxml_hook_actual_stem_required")?,
                )?)?[0]
            {
                "forward hook"
            } else {
                "backward hook"
            }
        } else {
            let mut members = Vec::new();
            for c in &music.chords {
                if crate::music_voices::chord_beams(snapshot, c.id)?.contains(id) {
                    members.push(c);
                }
            }
            members.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            if members.len() < 2 {
                return Err("musicxml_incomplete_beam_group_pending".into());
            }
            let position = members
                .iter()
                .position(|c| c.key == chord.key)
                .ok_or("musicxml_beam_actual_membership_required")?;
            if position == 0 {
                "begin"
            } else if position + 1 == members.len() {
                "end"
            } else {
                "continue"
            }
        };
        marks.push(json!({"number":index+1,"value":value}));
    }
    Ok(json!(marks))
}
fn articulations(snapshot: &Value, id: usize, first_note: &Value) -> Result<Value, String> {
    let mut result = Vec::new();
    for edge in snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            matches!(
                e["kind"].as_str(),
                Some("ChordArticulationRelation" | "ChordPauseRelation")
            ) && e["source"] == id
        })
    {
        let art = node(snapshot, edge["target"].as_u64().unwrap() as usize)?;
        let (name, strong_type) = match art["shape"].as_str() {
            Some("STACCATO") => ("staccato", None),
            Some("CAESURA") => ("caesura", None),
            Some("BREATH_MARK") => ("breath-mark", None),
            Some("TENUTO") => ("tenuto", None),
            Some("ACCENT") => ("accent", None),
            Some("STACCATISSIMO" | "STACCATISSIMO_BELOW") => ("staccatissimo", None),
            Some("MARCATO") => ("strong-accent", Some("up")),
            Some("MARCATO_BELOW") => ("strong-accent", Some("down")),
            _ => return Err("musicxml_articulation_shape_pending".into()),
        };
        result.push(json!({"name":name,"type":strong_type,"placement":if center(art)?[1]<center(first_note)?[1] {"above"} else {"below"}}));
    }
    Ok(json!(result))
}
fn directions(
    music: &OwnedMusic,
    snapshot: &Value,
    chord: &crate::music_measures::Chord,
    note: &Value,
) -> Result<Value, String> {
    let measure = &music.measures[chord.measure];
    let g = measure
        .geometry
        .as_ref()
        .ok_or("musicxml_direction_actual_Measure_required")?;
    let staff = g
        .part
        .staffs
        .iter()
        .find(|s| note["staff_id"] == s.id)
        .ok_or("musicxml_direction_actual_Staff_required")?;
    let staff_number = g
        .part
        .staffs
        .iter()
        .position(|s| Arc::ptr_eq(s, staff))
        .unwrap()
        + 1;
    let tenths = |d: f64| (10. * d / music.owner.interline as f64).round_ties_even() as i64;
    let mut directions = Vec::new();
    for edge in snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            e["source"] == chord.id
                && matches!(
                    e["kind"].as_str(),
                    Some("ChordDynamicsRelation" | "ChordPedalRelation")
                )
        })
    {
        let n = node(snapshot, edge["target"].as_u64().unwrap() as usize)?;
        let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
            .map_err(|_| "musicxml_direction_bounds_required")?;
        let dynamics = edge["kind"] == "ChordDynamicsRelation";
        let y = if dynamics {
            b[1] + b[3] / 2
        } else {
            b[1] + b[3]
        };
        let mut v = json!({"kind":if dynamics{"dynamics"}else{"pedal"},"shape":n["shape"],"placement":if y<center(note)?[1]{"above"}else{"below"},"staff_number":if g.part.staffs.len()>1{Some(staff_number)}else{None},"default_y":tenths(staff.lines.first().unwrap().y(b[0]as f64)-y as f64)});
        if dynamics {
            v["signature"] = json!(
                crate::music_directions::signature(n["shape"].as_str().unwrap())
                    .ok_or("musicxml_dynamic_shape_pending")?
            );
            v["relative_x"] = json!(tenths((b[0] - note["bounds"][0].as_i64().unwrap()) as f64));
            v["sound"] = json!(crate::music_directions::sound(n["shape"].as_str().unwrap()));
        } else {
            v["type"] = json!(if n["shape"] == "PEDAL_MARK" {
                "start"
            } else {
                "stop"
            });
            v["default_x"] = json!(tenths((b[0] - g.abscissae(staff)?[0]) as f64));
        }
        directions.push(v);
    }
    Ok(json!(directions))
}
impl OwnedMusic {
    /// Export the live recognized Voice/BEGIN stream. Publication admission is
    /// separate until all required image factories and quality checks are met.
    pub fn write_recognized_musicxml(&self, part_name: &str) -> Result<(String, Value), String> {
        if part_name.trim().is_empty()
            || part_name.len() > 256
            || part_name.chars().any(char::is_control)
        {
            return Err(
                "musicxml_part_name_requires_nonempty_single_line_up_to256_UTF8_bytes".into(),
            );
        }
        let snapshot = self.owner.graph.snapshot();
        let reader = note_pitch::Reader::new(self);
        let divisions =
            crate::music_voice_export::divisions(&self.owner.graph, &self.measures, &self.chords)?;
        let ranges = self.recognized_time_ranges()?;
        let parts = self
            .measures
            .iter()
            .map(|m| m.part)
            .collect::<BTreeSet<_>>();
        if parts.is_empty() {
            return Err("musicxml_actual_Parts_required".into());
        }
        let name = part_name
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;");
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<score-partwise version=\"4.0\"><part-list>",
        );
        for part in &parts {
            write!(
                xml,
                "<score-part id=\"P{part}\"><part-name>{name} {part}</part-name></score-part>"
            )
            .unwrap();
        }
        xml.push_str("</part-list>\n");
        let mut events = Vec::new();
        let mut omitted = Vec::new();
        for part in parts {
            writeln!(xml, "<part id=\"P{part}\">").unwrap();
            let mut staff_count = None;
            let mut previous_meter = None;
            let mut headers = HeaderState::default();
            let mut previous_system = None;
            for (number, (mi, measure)) in self
                .measures
                .iter()
                .enumerate()
                .filter(|(_, m)| m.part == part)
                .enumerate()
            {
                let geometry = measure
                    .geometry
                    .as_ref()
                    .ok_or("musicxml_actual_Measure_required")?;
                self.owner.graph.require_actual_music_measure(geometry)?;
                let count = geometry.part.staffs.len();
                if count == 0 || staff_count.is_some_and(|old| old != count) {
                    return Err("musicxml_logical_Part_Staff_continuity_required".into());
                }
                staff_count = Some(count);
                let si = self
                    .measure_layout
                    .stacks
                    .iter()
                    .position(|s| Arc::ptr_eq(&s.stack, &geometry.stack))
                    .ok_or("musicxml_actual_Stack_required")?;
                let stack = &self.measure_layout.stacks[si];
                let timing = stack
                    .timing
                    .as_ref()
                    .ok_or("musicxml_recognized_Stack_rhythm_required")?;
                if measure.timing.slots.len() != timing.slots.len()
                    || measure
                        .timing
                        .slots
                        .iter()
                        .zip(&timing.slots)
                        .any(|(a, b)| !Arc::ptr_eq(&a.identity, &b.identity))
                {
                    return Err("musicxml_canonical_Stack_Slots_required".into());
                }
                let range = ranges
                    .iter()
                    .find(|r| r.start <= si + 1 && si < r.stop)
                    .ok_or("musicxml_actual_time_range_required")?;
                let meter = range
                    .meter
                    .ok_or("musicxml_unknown_time_range_no_default")?;
                if measure.timing.expected != range.duration {
                    return Err("musicxml_recognized_expected_duration_required".into());
                }
                let (measure_keys, measure_clefs) = actual_headers(
                    self,
                    &snapshot,
                    mi,
                    timing.slots.first().map(|slot| slot.x_offset),
                )?;
                let changes = headers.advance(
                    &measure_keys,
                    &measure_clefs,
                    previous_system != Some(measure.system),
                )?;
                let steps = crate::music_voice_export::sequence(
                    &self.owner.graph,
                    &self.voices,
                    measure,
                    mi,
                )?;
                let mut emitted = BTreeSet::new();
                let mut local = Vec::new();
                for step in &steps {
                    let Step::Chord { id, .. } = *step else {
                        continue;
                    };
                    let chord = self
                        .chords
                        .iter()
                        .find(|c| c.id == id && c.measure == mi)
                        .ok_or("musicxml_actual_Chord_required")?;
                    let live = node(&snapshot, id)?;
                    let members = crate::music_voice_export::chord_notes(
                        snapshot["inters"].as_array().unwrap(),
                        live,
                    )?;
                    let beam_marks = if chord.kind == ChordKind::Head {
                        beams(self, &snapshot, chord)?
                    } else {
                        json!([])
                    };
                    for (index, note) in members.iter().enumerate() {
                        let id = note["id"].as_u64().unwrap() as usize;
                        let staff = note["staff_id"]
                            .as_u64()
                            .ok_or("musicxml_actual_note_Staff_required")?;
                        let staff_number = geometry
                            .part
                            .staffs
                            .iter()
                            .position(|s| {
                                s.id == staff
                                    && self.owner.staffs.iter().any(|held| Arc::ptr_eq(s, held))
                            })
                            .ok_or("musicxml_actual_note_Part_Staff_required")?
                            + 1;
                        let rest = chord.kind == ChordKind::Rest;
                        let mut event = json!({"chord_id":chord.id,"note_id":id,"rest":rest,"beams":beam_marks,"stem_direction":null});
                        if !rest {
                            let pitch =
                                reader.read_for_export(id, headers.keys[staff_number - 1])?;
                            event
                                .as_object_mut()
                                .unwrap()
                                .extend(pitch.as_object().unwrap().clone());
                            event["accidental"] = pitch["local_accidental"]["shape"].clone();
                            if chord.stem.is_some() {
                                let tail = live["tail_location"][1]
                                    .as_i64()
                                    .ok_or("musicxml_actual_Chord_tail_required")?;
                                event["stem_direction"] = json!(if tail < center(note)?[1] {
                                    "up"
                                } else {
                                    "down"
                                });
                            }
                        }
                        event["directions"] = if index == 0 {
                            directions(self, &snapshot, chord, note)?
                        } else {
                            json!([])
                        };
                        event["articulations"] = if index == 0 {
                            articulations(&snapshot, chord.id, note)?
                        } else {
                            json!([])
                        };
                        crate::music_voice_export::bind_event(
                            &self.owner.graph,
                            &self.voices,
                            measure,
                            mi,
                            &mut event,
                        )?;
                        if count > 1 {
                            event["staff_number"] = json!(staff_number);
                        }
                        event["part_id"] = json!(part);
                        event["measure_number"] = json!(number + 1);
                        local.push(event);
                        emitted.insert(chord.id);
                    }
                }
                for chord in self
                    .chords
                    .iter()
                    .filter(|c| c.measure == mi && !emitted.contains(&c.id))
                {
                    let live = node(&snapshot, chord.id)?;
                    omitted.push(json!({"chord":chord.key,"bounds":live["bounds"],"part_id":part,"measure_number":number+1,"voice":live["voice"],"time_offset":live["time_offset"],"slot_id":live["slot_id"],"reason":"no_actual_Voice_BEGIN_in_canonical_Stack_slots"}));
                }
                writeln!(xml, "<measure number=\"{}\">", number + 1).unwrap();
                if previous_system.is_some_and(|s| s != measure.system) {
                    xml.push_str("<print new-system=\"yes\"/>\n");
                }
                if number == 0
                    || !changes.keys.is_empty()
                    || !changes.clefs.is_empty()
                    || previous_meter != Some(meter)
                {
                    xml.push_str("<attributes>");
                    if number == 0 {
                        write!(xml, "<divisions>{divisions}</divisions>").unwrap();
                    }
                    for (staff_number, fifths) in &changes.keys {
                        if let Some(staff_number) = staff_number {
                            write!(
                                xml,
                                "<key number=\"{staff_number}\"><fifths>{fifths}</fifths></key>"
                            )
                            .unwrap();
                        } else {
                            write!(xml, "<key><fifths>{fifths}</fifths></key>").unwrap();
                        }
                    }
                    if previous_meter != Some(meter) {
                        write!(
                            xml,
                            "<time><beats>{}</beats><beat-type>{}</beat-type></time>",
                            meter.0, meter.1
                        )
                        .unwrap();
                    }
                    if number == 0 && count > 1 {
                        write!(xml, "<staves>{count}</staves>").unwrap();
                    }
                    for (staff_number, clef) in &changes.clefs {
                        let mut value = String::new();
                        write_clef(&mut value, *clef);
                        if count > 1 {
                            value = value.replacen(
                                "<clef>",
                                &format!("<clef number=\"{staff_number}\">"),
                                1,
                            );
                        }
                        xml.push_str(&value);
                    }
                    xml.push_str("</attributes>\n");
                }
                crate::music_voice_export::write_measure(
                    &self.owner.graph,
                    &mut xml,
                    &steps,
                    &local.iter().collect::<Vec<_>>(),
                    divisions,
                )?;
                xml.push_str("</measure>\n");
                events.extend(local);
                previous_meter = Some(meter);
                previous_system = Some(measure.system);
            }
            xml.push_str("</part>\n");
        }
        xml.push_str("</score-partwise>\n");
        Ok((
            xml,
            json!({"events":events,"omitted_actual_Chords":omitted,"divisions":divisions,"required_directions_connected":true,"quality_accepted":false}),
        ))
    }
}

#[cfg(test)]
mod header_tests;
