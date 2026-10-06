// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded monophonic continuation and native MusicXML writer.
//! Shape.getNoteDuration, MeasureRhythm.processStartingChords, Voice.addChord,
//! AbstractChordInter.getEndTime, Page.numberMeasures/simpleDurationOf and
//! PartwiseBuilder note/attribute ordering guide this explicitly bounded path.
use crate::{GrayImage, music_measures::OwnedMusic, music_symbols, shared_sig::Key};
use serde_json::{Value, json};
use std::fmt::Write;

pub struct Export {
    pub owner: OwnedMusic,
    pub xml: String,
}
use crate::music_time::format_ticks as rational;
fn quarters(ticks: u32) -> Value {
    if ticks.is_multiple_of(4) {
        json!(ticks / 4)
    } else {
        json!(ticks as f64 / 4.)
    }
}
fn eighths(ticks: u32) -> Value {
    if ticks.is_multiple_of(2) {
        json!(ticks / 2)
    } else {
        json!(ticks as f64 / 2.)
    }
}
// PartwiseBuilder.buildClef / areEqual: semantic identity, not header Inter id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClefAttributes {
    sign: &'static str,
    line: i32,
    octave_change: i32,
}
fn clef_attributes(clef: &Value) -> Result<ClefAttributes, String> {
    let pitch = clef["pitch"]
        .as_f64()
        .ok_or("musicxml_native_clef_pitch_required")?;
    if !pitch.is_finite() || pitch.abs() > 12. {
        return Err("musicxml_native_clef_pitch_outside_verified_range".into());
    }
    let (sign, octave_change) = match clef["shape"].as_str() {
        Some("G_CLEF" | "G_CLEF_SMALL") => ("G", 0),
        Some("G_CLEF_8VA") => ("G", 1),
        Some("G_CLEF_8VB") => ("G", -1),
        Some("C_CLEF") => ("C", 0),
        Some("F_CLEF" | "F_CLEF_SMALL") => ("F", 0),
        Some("F_CLEF_8VA") => ("F", 1),
        Some("F_CLEF_8VB") => ("F", -1),
        Some("PERCUSSION_CLEF") => {
            return Err("unpitched_percussion_requires_separate_mapping".into());
        }
        _ => return Err("unsupported_diatonic_clef".into()),
    };
    Ok(ClefAttributes {
        sign,
        line: 3 - (pitch / 2.).round_ties_even() as i32,
        octave_change,
    })
}
fn write_clef(xml: &mut String, clef: ClefAttributes) {
    write!(
        xml,
        "<clef><sign>{}</sign><line>{}</line>",
        clef.sign, clef.line
    )
    .unwrap();
    if clef.octave_change != 0 {
        write!(
            xml,
            "<clef-octave-change>{}</clef-octave-change>",
            clef.octave_change
        )
        .unwrap();
    }
    xml.push_str("</clef>");
}
fn note_duration(shape: &str) -> Result<(u32, &'static str), String> {
    match shape {
        "NOTEHEAD_BLACK" | "QUARTER_REST" => Ok((1, "quarter")),
        "NOTEHEAD_VOID" | "HALF_REST" => Ok((2, "half")),
        _ => Err(format!(
            "musicxml_note_duration_outside_quarter_half_scope:{shape}"
        )),
    }
}
// Resolve the admitted original AbstractChord.getDuration before starting
// Voice/group time propagation. All existing shape/flag/beam guards remain;
// general dotted/tuplet/special-head duration is a separate dependency.
fn resolve_durations(
    graph: &mut crate::shared_sig::Graph,
    chords: &[&crate::music_measures::Chord],
) -> Result<std::collections::BTreeMap<usize, (Option<u32>, &'static str)>, String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let edges = snapshot["relations"].as_array().unwrap();
    let mut resolved = std::collections::BTreeMap::new();
    let mut staged = graph.clone();
    for chord in chords {
        if chord.members.len() != 1 {
            return Err("musicxml_polyphonic_head_chord_pending".into());
        }
        let note = nodes
            .iter()
            .find(|n| n["id"] == chord.members[0]["id"])
            .ok_or("musicxml_duration_live_note_required")?;
        let shape = note["shape"]
            .as_str()
            .ok_or("musicxml_note_shape_required")?;
        let rest = note["kind"] == "RestInter";
        let whole = rest && crate::music_voices::is_measure_rest_shape(shape, false);
        let result = if whole {
            (None, "whole")
        } else {
            let (base, mut note_type) = note_duration(shape)?;
            let mut duration = base * 4;
            if !rest {
                let beams = crate::music_voices::chord_beams(&snapshot, chord.id)?;
                let stem = chord
                    .stem
                    .map(|key| graph.id(key).ok_or("musicxml_actual_chord_stem_required"))
                    .transpose()?;
                let flags = nodes
                    .iter()
                    .filter(|n| {
                        n["kind"] == "FlagInter"
                            && stem.is_some_and(|id| {
                                edges.iter().any(|e| {
                                    e["kind"] == "FlagStemRelation"
                                        && e["source"] == n["id"]
                                        && e["target"] == id
                                })
                            })
                    })
                    .collect::<Vec<_>>();
                if !flags.is_empty() || !beams.is_empty() {
                    if shape != "NOTEHEAD_BLACK"
                        || (!flags.is_empty() && !beams.is_empty())
                        || flags.len() > 1
                        || beams.len() > 2
                        || flags.first().is_some_and(|f| f["value"] != 1)
                    {
                        return Err("musicxml_flagged_void_rest_or_multiple_flags_pending".into());
                    }
                    let count = flags.len() + beams.len();
                    duration /= 1 << count;
                    note_type = if count == 1 { "eighth" } else { "16th" };
                }
            }
            (Some(duration), note_type)
        };
        let mut node = nodes
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("musicxml_duration_live_chord_required")?
            .clone();
        node["duration"] = json!(result.0.map(rational));
        staged.vertex(chord.key, node, false)?;
        resolved.insert(chord.id, result);
    }
    *graph = staged;
    Ok(resolved)
}
// ArpeggiatoInter.lookupLink builds a right-side half-open AWT rectangle.
// The existing automatic path uses system profile1 (xGapMax_p1=2.5).
// This only proves an empty chord search; positive arpeggiation stays pending.
fn arpeggiato_may_link(bounds: [i64; 4], interline: usize, chords: &[[i64; 4]]) -> bool {
    let lookup = [
        bounds[0] + bounds[2],
        bounds[1],
        (2.5 * interline as f64).round_ties_even() as i64,
        bounds[3],
    ];
    chords.iter().any(|b| {
        lookup[0] < b[0] + b[2]
            && b[0] < lookup[0] + lookup[2]
            && lookup[1] < b[1] + b[3]
            && b[1] < lookup[1] + lookup[3]
    })
}
fn admit_symbols(m: &OwnedMusic) -> Result<(), String> {
    let staffs = m.owner.upstream_report["grid_ir"]["staffs"]
        .as_array()
        .ok_or("musicxml_staffs_required")?;
    let trials = m.owner.prepared.report["classifier_trials"]
        .as_array()
        .ok_or("musicxml_symbol_trials_required")?;
    if m.owner.prepared.report["symbol_cluster_scope"] != "isolated-and-two-parts" {
        return Err("musicxml_compound_symbol_parts_pending".into());
    }
    for t in trials {
        for e in t["evaluations"]
            .as_array()
            .ok_or("musicxml_symbol_evaluations_required")?
        {
            let shape = e["shape"]
                .as_str()
                .ok_or("musicxml_symbol_shape_required")?;
            match shape {
                "CLUTTER" | "NOISE" => {}
                "ARPEGGIATO" => {
                    let bounds = serde_json::from_value(t["bounds"].clone())
                        .map_err(|_| "musicxml_arpeggiato_bounds_required")?;
                    let chords = m
                        .chords
                        .iter()
                        .filter(|c| c.kind == crate::music_measures::ChordKind::Head)
                        .map(|c| c.bounds)
                        .collect::<Vec<_>>();
                    if arpeggiato_may_link(bounds, m.owner.interline, &chords) {
                        return Err("musicxml_arpeggiato_chord_link_pending".into());
                    }
                    // InterFactory.createValidAdded returns null without a link.
                }
                "DOT_set" => {
                    let b: [i64; 4] = serde_json::from_value(t["bounds"].clone())
                        .map_err(|_| "musicxml_dot_bounds_required")?;
                    let y = b[1] as f64 + b[3] as f64 / 2.;
                    let s = staffs
                        .iter()
                        .map(|s| Ok(((y - music_symbols::flat_staff_y(s, 2)?).abs(), s)))
                        .collect::<Result<Vec<_>, String>>()?
                        .into_iter()
                        .min_by(|a, b| a.0.total_cmp(&b.0))
                        .ok_or("musicxml_dot_staff_required")?
                        .1;
                    let pitch =
                        2. * (y - music_symbols::flat_staff_y(s, 2)?) / m.owner.interline as f64;
                    let nearest = 2. * (pitch / 2.).round_ties_even();
                    // DotFactory.checkDistanceToConcreteLine plus
                    // Staff.getConcreteLedgerNearby. Use concrete owned ledgers,
                    // not hypothetical lines outside the ordinary five-line staff.
                    let distance = if pitch.abs() <= 5. {
                        (pitch - nearest).abs() / 2.
                    } else {
                        let index = (nearest as i64 - nearest.signum() as i64 * 4) / 2;
                        let x = b[0] as f64 + b[2] as f64 / 2.;
                        let il = m.owner.interline as f64;
                        m.owner.graph.snapshot()["inters"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter(|n| {
                                n["kind"] == "LedgerInter"
                                    && n["staff_id"] == s["id"]
                                    && n["index"] == index
                            })
                            .filter_map(|n| {
                                let r: [f64; 4] =
                                    serde_json::from_value(n["bounds"].clone()).ok()?;
                                (r[0] < x + il
                                    && x - il < r[0] + r[2]
                                    && r[1] < y + il
                                    && y - il < r[1] + r[3])
                                    .then_some((y - (r[1] + r[3] / 2.)).abs() / il)
                            })
                            .min_by(f64::total_cmp)
                            .unwrap_or(f64::INFINITY)
                    };
                    // A possible real dot still refuses until actual late checks.
                    if distance >= 0.3 {
                        return Err("musicxml_possible_dot_lateChecks_pending".into());
                    }
                }
                s if music_symbols::time(s).is_some()
                    || matches!(
                        s,
                        "QUARTER_REST" | "HALF_REST" | "WHOLE_REST" | "FLAG_1" | "FLAG_1_DOWN"
                    ) =>
                {
                    if t["created_inter_ids"]
                        .as_array()
                        .is_none_or(|ids| ids.is_empty())
                    {
                        return Err("musicxml_registered_symbol_identity_required".into());
                    }
                    if matches!(s, "FLAG_1" | "FLAG_1_DOWN") {
                        let snapshot = m.owner.graph.snapshot();
                        let ids = t["created_inter_ids"].as_array().unwrap();
                        if !snapshot["inters"].as_array().unwrap().iter().any(|n| {
                            n["kind"] == "FlagInter"
                                && n["shape"] == s
                                && ids.contains(&n["id"])
                                && snapshot["relations"].as_array().unwrap().iter().any(|e| {
                                    e["kind"] == "FlagStemRelation" && e["source"] == n["id"]
                                })
                        }) {
                            return Err("musicxml_surviving_flag_stem_identity_required".into());
                        }
                    }
                }
                _ => return Err(format!("musicxml_unported_symbol_interpretation:{shape}")),
            }
        }
    }
    Ok(())
}
/// No oracle, serialized SIG import, selected notes, guessed meter, or invented
/// rests. Rejects incomplete/ambiguous measures instead of emitting false music.
pub fn continue_export(m: OwnedMusic) -> Result<Export, String> {
    continue_export_named(m, "Music")
}
fn continue_export_named(mut m: OwnedMusic, part_name: &str) -> Result<Export, String> {
    if part_name.trim().is_empty()
        || part_name.len() > 256
        || part_name.chars().any(char::is_control)
    {
        return Err("musicxml_part_name_requires_nonempty_single_line_up_to256_UTF8_bytes".into());
    }
    if m.owner.prepared.report["schema"] != "audiveris-native.music-symbols-isolated.v1"
        || m.owner.prepared.report["shared_sig"] != m.owner.graph.snapshot()
    {
        return Err("musicxml_retained_symbol_owner_required".into());
    }
    m = crate::music_links::continue_links(m)?;
    if m.owner.interline != 11
        || m.owner.catalog.family != "Bravura"
        || !matches!(m.owner.catalog.stored_point_size, 44 | 47)
    {
        return Err("musicxml_verified_Bravura44_or47_interline11_scope_required".into());
    }
    let grid = &m.owner.upstream_report["grid_ir"];
    if grid["pages"].as_array().is_none_or(|p| p.len() != 1)
        || grid["systems"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|s| s["parts"].as_array().is_none_or(|p| p.len() != 1))
    {
        return Err("musicxml_single_page_one_part_scope_required".into());
    }
    admit_symbols(&m)?;
    // MeasureRhythm.process starts each admitted measure pass with the original
    // ownership/timing reset. General slot matching and correction reprocessing
    // entry remain pending; do not substitute clearing the entire SIG/cache.
    for mi in 0..m.measures.len() {
        crate::music_voices::reset_rhythm(
            &mut m.owner.graph,
            &mut m.voices,
            &mut m.measures[mi],
            &m.chords,
            mi,
        )?;
    }
    let initial = m.owner.graph.snapshot();
    let nodes = initial["inters"].as_array().unwrap();
    let edges = initial["relations"].as_array().unwrap();
    let mut graph = m.owner.graph.clone();
    let mut events = Vec::new();
    let mut measures = Vec::new();
    let mut preliminary_slots = Vec::new();
    let mut carried_meter = None;
    let mut carried_time_symbol = None;
    let mut carried_key: Option<(Value, i64)> = None;
    for mi in 0..m.measures.len() {
        let measure = m.measures[mi].clone();
        let mut chords = m
            .chords
            .iter()
            .filter(|c| c.measure == mi)
            .collect::<Vec<_>>();
        // Original geometric retrieval precedes rhythm clocks/Voice matching.
        let (narrow, wide) = crate::music_measure_slots::from_owned(
            &m.owner.graph,
            &measure,
            &chords,
            m.owner.interline,
        )?;
        let centers = chords
            .iter()
            .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
            .collect();
        let compounds =
            crate::music_measure_slots::build_compounds(&narrow.slots, &wide.slots, &centers)?;
        let ordered_ids = compounds
            .iter()
            .flat_map(|c| c.members.iter())
            .flat_map(|s| s.chord_ids.iter())
            .copied()
            .collect::<Vec<_>>();
        if ordered_ids
            != narrow
                .slots
                .iter()
                .flat_map(|s| s.chord_ids.iter())
                .copied()
                .collect::<Vec<_>>()
        {
            return Err("musicxml_inconsistent_compound_partition_pending".into());
        }
        // Until ChordsMapper is connected, actual multi-chord slots require
        // parallel voice assignment; preserve refusal instead of serializing them.
        if compounds.iter().any(|s| s.chord_ids.len() != 1) {
            return Err("musicxml_general_slot_voice_assignment_pending".into());
        }
        chords.sort_by_key(|c| ordered_ids.iter().position(|id| *id == c.id));
        preliminary_slots.push(json!({"measure_number":mi+1,"narrow":narrow,"wide":wide,"compounds":compounds.iter().map(|c|c.projection()).collect::<Vec<_>>(),"voice_assignment":"original separated ChordsMapper on owned one-incoming single-Voice path; general retries/new Voices/implicit tuplets pending"}));
        if chords.is_empty() {
            return Err(format!(
                "musicxml_empty_measure:{}:{}",
                measure.system, measure.index
            ));
        }
        let times = nodes
            .iter()
            .filter(|n| {
                (n["kind"] == "TimeWholeInter" || n["kind"] == "TimePairInter")
                    && n["origin"]["system"] == measure.system
                    && n["staff_id"] == measure.staff
                    && n["bounds"][0]
                        .as_i64()
                        .is_some_and(|x| measure.left < x && x <= measure.right)
            })
            .collect::<Vec<_>>();
        if times.len() > 1 {
            return Err("musicxml_time_column_competition_pending".into());
        }
        let previous_meter = carried_meter;
        let previous_time_symbol = carried_time_symbol;
        if let Some(t) = times.first() {
            if t["bounds"][0].as_i64().unwrap() >= chords[0].bounds[0] {
                return Err("musicxml_mid_measure_meter_pending".into());
            }
            carried_time_symbol = match t["shape"].as_str() {
                Some("COMMON_TIME") => Some("common"),
                Some("CUT_TIME") => Some("cut"),
                _ => None,
            };
            carried_meter = if t["kind"] == "TimePairInter" || t["origin"]["phase"] == "MUSIC_TIME"
            {
                if t["frozen"] != true || t["origin"]["phase"] != "MUSIC_TIME" {
                    return Err("musicxml_selected_header_time_required".into());
                }
                Some((
                    t["numerator"]
                        .as_u64()
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or("musicxml_time_numerator_required")?,
                    t["denominator"]
                        .as_u64()
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or("musicxml_time_denominator_required")?,
                ))
            } else {
                music_symbols::time(t["shape"].as_str().unwrap())
            };
        }
        let meter = carried_meter.ok_or("musicxml_recognized_meter_required_no_default")?;
        if previous_meter.is_some_and(|previous| {
            previous != meter || previous_time_symbol != carried_time_symbol
        }) {
            return Err("musicxml_meter_change_range_propagation_pending".into());
        }
        // PageRhythm.populateTimeSignatures: TimeRational.getValue supplies
        // the range duration, inherited by stacks until the next time event.
        // Exact binary fractions through sixteenths. Finer/tuplets remain pending.
        let whole_ticks = meter.0.checked_mul(16).ok_or("musicxml_meter_overflow")?;
        if meter.0 == 0 || meter.1 == 0 || whole_ticks % meter.1 != 0 {
            return Err("musicxml_measure_duration_finer_than_sixteenth_pending".into());
        }
        let expected_ticks = whole_ticks / meter.1;
        // PartwiseBuilder.processKeys updates current.keys only on an actual
        // key event. Its absent-key case preserves the previous part/staff key.
        let keys = nodes
            .iter()
            .filter(|n| {
                n["kind"] == "KeyInter"
                    && n["origin"]["system"] == measure.system
                    && n["staff_id"] == measure.staff
                    && n["bounds"][0]
                        .as_i64()
                        .is_some_and(|x| measure.left < x && x <= measure.right)
            })
            .collect::<Vec<_>>();
        if keys.len() > 1 {
            return Err("musicxml_key_competition_pending".into());
        }
        if let Some(key) = keys.first() {
            if key["frozen"] != true
                || key["origin"]["phase"] != "MUSIC_KEY"
                || key["bounds"][0].as_i64().unwrap() >= chords[0].bounds[0]
            {
                return Err("musicxml_nonheader_or_mid_measure_key_pending".into());
            }
            let fifths = key["fifths"]
                .as_i64()
                .ok_or("musicxml_key_fifths_required")?;
            crate::music_headers::key_alter("F", Some(fifths))?;
            carried_key = Some((key["id"].clone(), fifths));
        }
        let fifths = carried_key.as_ref().map(|k| k.1);
        let key_id = carried_key.as_ref().map(|k| k.0.clone());
        let mut offset = 0_u32;
        let mut previous_right = None;
        // PartwiseBuilder processes the measure clef even for rest-only music.
        // Use an actual native header Inter, never a writer default.
        let initial_clef = nodes
            .iter()
            .filter(|n| {
                n["kind"] == "ClefInter"
                    && n["origin"]["system"] == measure.system
                    && n["staff_id"] == measure.staff
                    && n["bounds"][0]
                        .as_i64()
                        .is_some_and(|x| x < chords[0].bounds[0])
            })
            .max_by_key(|n| n["bounds"][0].as_i64())
            .ok_or("musicxml_measure_local_clef_required")?;
        let initial_attributes = clef_attributes(initial_clef)?;
        let clef_id = initial_clef["id"].as_u64();
        let durations = resolve_durations(&mut graph, &chords)?;
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
        if starting.len() != 1 {
            return Err("musicxml_starting_parallel_voice_mapping_pending".into());
        }
        let seed = *chords
            .iter()
            .find(|c| c.id == starting[0])
            .ok_or("musicxml_starting_owned_chord_required")?;
        let voice = crate::music_voices::single(mi, &measure, &[seed], &m.voices)?;
        let voice_id = voice.id;
        let voice_index = m.voices.len();
        m.voices.push(voice);
        crate::music_voices::assign_chord_voice(
            &mut graph,
            &mut m.voices,
            &m.measures,
            seed.id,
            voice_index,
        )?;
        crate::music_chord_time::set_and_push(
            &mut graph,
            &m.voices,
            &m.measures,
            seed.id,
            Some(0),
        )?;
        let start_snapshot = graph.snapshot();
        preliminary_slots[mi]["starting_chord_ids"] = json!(starting);
        preliminary_slots[mi]["starting_state"] = json!({"voices":[{"id":voice_id,"is_measure_rest":m.voices[voice_index].measure_rest_chord.is_some(),"chord_ids":m.voices[voice_index].chord_ids,"measure_rest_chord_id":m.voices[voice_index].measure_rest_chord}],"chords":chords.iter().map(|c| {let n=start_snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==c.id).unwrap();json!({"id":c.id,"bounds":n["bounds"],"duration":n["duration"],"time_offset":n["time_offset"],"voice":n["voice"]})}).collect::<Vec<_>>()});
        preliminary_slots[mi]["mapping_steps"] = json!([]);
        preliminary_slots[mi]["mapping_validation_steps"] = json!([]);
        preliminary_slots[mi]["extinction_steps"] = json!([]);
        let mut lifetime = crate::music_voice_lifetime::Lifetime::default();
        let mut slot_times = crate::music_measure_slots::ClockState::new(&compounds)?;
        preliminary_slots[mi]["candidate_time_steps"] = json!([]);
        for &chord in &chords {
            if let Some(slot) = compounds.iter().find(|s| s.chord_ids.contains(&chord.id)) {
                let before = slot_times.projection(slot)?;
                let mut step = crate::music_rookie_mapping::Context {
                    graph: &mut graph,
                    voices: &mut m.voices,
                    measures: &m.measures,
                    chords: &chords,
                    measure_index: mi,
                    compounds: &compounds,
                    expected: expected_ticks,
                    clef: initial_clef,
                    interline: m.owner.interline,
                    lifetime: &lifetime,
                    slot_times: &mut slot_times,
                }
                .map_slot(slot)?;
                let validation = step.as_object_mut().unwrap().remove("validation").unwrap();
                preliminary_slots[mi]["mapping_validation_steps"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"slot_id":slot.id,"validation":validation}));
                preliminary_slots[mi]["mapping_steps"]
                    .as_array_mut()
                    .unwrap()
                    .push(step);
                preliminary_slots[mi]["candidate_time_steps"].as_array_mut().unwrap().push(json!({"slot_id":slot.id,"before":before,"after":slot_times.projection(slot)?,"ok":true}));
                let first_time = slot_times.first_time(slot)?;
                let extinction = lifetime.purge(
                    &graph,
                    &m.voices,
                    &mut m.measures[mi],
                    mi,
                    &compounds,
                    (slot, first_time),
                )?;
                preliminary_slots[mi]["extinction_steps"]
                    .as_array_mut()
                    .unwrap()
                    .push(extinction);
            }
            if chord.members.len() != 1 {
                return Err("musicxml_polyphonic_head_chord_pending".into());
            }
            if previous_right.is_some_and(|x| chord.bounds[0] <= x) {
                return Err("musicxml_parallel_or_overlapping_chords_pending".into());
            }
            previous_right = Some(chord.bounds[0] + chord.bounds[2] - 1);
            let note = &chord.members[0];
            let shape = note["shape"]
                .as_str()
                .ok_or("musicxml_note_shape_required")?;
            let measure_rest = m.voices[voice_index].measure_rest_chord == Some(chord.id);
            let (intrinsic, note_type) = durations[&chord.id];
            let duration = intrinsic.unwrap_or(expected_ticks);
            let rest = note["kind"] == "RestInter";
            // AbstractChordInter.getBeams: actual beam→stem and chord→stem
            // relations supply membership; x positions only order members.
            let mut beams = if let Some(stem) = chord.stem {
                let id = graph
                    .id(stem)
                    .ok_or("musicxml_actual_beamed_stem_required")?;
                nodes
                    .iter()
                    .filter(|n| {
                        matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                            && edges.iter().any(|e| {
                                e["kind"] == "BeamStemRelation"
                                    && e["source"] == n["id"]
                                    && e["target"] == id
                            })
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            // AbstractChordInter.getBeams orders from tail to head (greatest
            // distance from head first), independent of SIG insertion IDs.
            let (head_y, stem_direction, chord_stem_direction) = if let Some(stem) = chord.stem {
                let stem_node = nodes
                    .iter()
                    .find(|n| n["id"] == graph.id(stem).unwrap())
                    .ok_or("musicxml_actual_chord_stem_required")?;
                let geometry = crate::head_chord_geometry::calculate(
                    &chord.members.iter().collect::<Vec<_>>(),
                    Some(stem_node),
                )?;
                let head_y = geometry
                    .head_location
                    .ok_or("musicxml_chord_head_location_required")?[1];
                // PartwiseBuilder emits a stem for every actual stemmed note.
                // Current one-head admission makes head ordinate the note's
                // integer center; negative tail-head is up, equality is down.
                let dir = geometry.stem_dir(true)?;
                (head_y, Some(if dir < 0 { "up" } else { "down" }), dir)
            } else {
                (chord.bounds[1] + chord.bounds[3] / 2, None, 0)
            };
            let chord_x = chord.bounds[0] + chord.bounds[2] / 2;
            let distance = |beam: &&Value| {
                let b = &beam["median"];
                let x1 = b[0].as_f64().unwrap();
                let y1 = b[1].as_f64().unwrap();
                let x2 = b[2].as_f64().unwrap();
                let y2 = b[3].as_f64().unwrap();
                (head_y as f64 - (y1 + (chord_x as f64 - x1) * (y2 - y1) / (x2 - x1))).abs()
            };
            beams.sort_by(|a, b| distance(b).total_cmp(&distance(a)));
            let flags = if let Some(stem) = chord.stem {
                let stem_id = graph
                    .id(stem)
                    .ok_or("musicxml_actual_chord_stem_required")?;
                nodes
                    .iter()
                    .filter(|n| {
                        n["kind"] == "FlagInter"
                            && edges.iter().any(|e| {
                                e["kind"] == "FlagStemRelation"
                                    && e["source"] == n["id"]
                                    && e["target"] == stem_id
                            })
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            // Shape/count validation and intrinsic duration were resolved
            // before the original starting Voice/time propagation above.
            let flag_ids = flags.iter().map(|f| f["id"].clone()).collect::<Vec<_>>();
            let mut beam_marks = Vec::new();
            for (number, beam) in beams.iter().enumerate() {
                let mut members = m
                    .chords
                    .iter()
                    .filter(|c| {
                        c.stem.is_some_and(|s| {
                            edges.iter().any(|e| {
                                e["kind"] == "BeamStemRelation"
                                    && e["source"] == beam["id"]
                                    && e["target"] == graph.id(s).unwrap()
                            })
                        })
                    })
                    .collect::<Vec<_>>();
                members.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
                if (beam["kind"] != "BeamHookInter" && members.len() < 2)
                    || members.iter().any(|c| c.measure != chord.measure)
                {
                    return Err("musicxml_incomplete_or_cross_measure_beam_group_pending".into());
                }
                let index = members
                    .iter()
                    .position(|c| c.id == chord.id)
                    .ok_or("musicxml_beam_chord_membership_required")?;
                let value = if beam["kind"] == "BeamHookInter" {
                    // PartwiseBuilder.processBeam uses integer Inter centers.
                    let stem_id = graph.id(chord.stem.unwrap()).unwrap();
                    let stem = nodes
                        .iter()
                        .find(|n| n["id"] == stem_id)
                        .ok_or("musicxml_hook_actual_stem_required")?;
                    let center = |n: &Value| -> Result<i64, String> {
                        Ok(n["bounds"][0]
                            .as_i64()
                            .ok_or("musicxml_hook_bounds_required")?
                            + n["bounds"][2]
                                .as_i64()
                                .ok_or("musicxml_hook_bounds_required")?
                                / 2)
                    };
                    if center(beam)? > center(stem)? {
                        "forward hook"
                    } else {
                        "backward hook"
                    }
                } else if index == 0 {
                    "begin"
                } else if index + 1 == members.len() {
                    "end"
                } else {
                    "continue"
                };
                beam_marks.push(json!({"number":number+1,"value":value}));
                let stem_id = graph.id(chord.stem.unwrap()).unwrap();
                edges
                    .iter()
                    .find(|e| {
                        e["kind"] == "HeadStemRelation"
                            && e["source"] == note["id"]
                            && e["target"] == stem_id
                    })
                    .ok_or("musicxml_beam_head_direction_required")?;
            }
            let beam_ids = beams.iter().map(|b| b["id"].clone()).collect::<Vec<_>>();
            let mut step = None;
            let mut octave = None;
            let mut cid = None;
            if !rest {
                let x = chord.bounds[0] + chord.bounds[2] / 2;
                // AbstractNoteInter resolves Measure.getClefBefore. Current
                // admitted scope contains only imported native header clefs.
                let clef = nodes
                    .iter()
                    .filter(|n| {
                        n["kind"] == "ClefInter"
                            && n["origin"]["system"] == measure.system
                            && n["staff_id"] == measure.staff
                            && n["bounds"][0].as_i64().is_some_and(|a| a < x)
                    })
                    .max_by_key(|n| n["bounds"][0].as_i64())
                    .ok_or("musicxml_measure_local_clef_required")?;
                // A measure-start header change can be exported in attributes.
                // A mid-measure change needs ordered clef events, still pending.
                if clef_attributes(clef)? != initial_attributes {
                    return Err("musicxml_mid_measure_clef_change_pending".into());
                }
                let pitch = note["pitch"]
                    .as_f64()
                    .ok_or("musicxml_native_note_pitch_required")?;
                if pitch.abs() > 4. {
                    // The existing Java ClefInter arithmetic covers ledger pitches.
                    // Admit only the first lower ledger C4 checked on an independent
                    // original-successful image, retaining its actual native ledger.
                    let hx = note["bounds"][0].as_f64().unwrap_or(f64::NAN)
                        + note["bounds"][2].as_f64().unwrap_or(f64::NAN) / 2.;
                    let concrete = clef["shape"] == "G_CLEF"
                        && clef["pitch"] == 2.
                        && pitch == 6.
                        && nodes.iter().any(|n| {
                            n["kind"] == "LedgerInter"
                                && n["staff_id"] == measure.staff
                                && n["index"] == 1
                                && n["bounds"][0].as_f64().is_some_and(|x| {
                                    x <= hx && hx < x + n["bounds"][2].as_f64().unwrap_or(0.)
                                })
                        });
                    if !concrete {
                        return Err("musicxml_ledger_pitch_scope_pending".into());
                    }
                }
                let (s, o) = crate::head_musical_pitch::diatonic(
                    clef["shape"].as_str().unwrap(),
                    clef["pitch"].as_f64().unwrap(),
                    pitch,
                )
                .map_err(str::to_owned)?;
                step = Some(s);
                octave = Some(o);
                cid = clef["id"].as_u64();
            }
            let key: Key = serde_json::from_value(note["origin"].clone())
                .map_err(|_| "musicxml_native_note_origin_required")?;
            let mut data = note.clone();
            data["step"] = json!(step);
            data["octave"] = json!(octave);
            let alter = if rest {
                None
            } else {
                Some(crate::music_headers::key_alter(step.unwrap(), fifths)?)
            };
            data["alter"] = json!(alter);
            data["effective_key_id"] = json!(key_id);
            data["effective_key_fifths"] = json!(fifths);
            data["chord_id"] = json!(chord.id);
            data["measure_number"] = json!(mi + 1);
            data["effective_clef_id"] = json!(cid);
            graph.vertex(key, data, false)?;
            let current_chords = graph.snapshot();
            let mut c = current_chords["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == chord.id)
                .ok_or("musicxml_actual_SIG_chord_required")?
                .clone();
            // A newly retained Voice must not be overwritten by the current
            // single-Voice writer; general Voice export is a later dependency.
            if c["voice"] != voice_id {
                return Err("musicxml_general_new_voice_export_pending".into());
            }
            c["duration"] = if measure_rest {
                Value::Null
            } else {
                json!(rational(duration))
            };
            if c["time_offset"]
                .as_str()
                .is_some_and(|t| t != rational(offset))
            {
                return Err("musicxml_starting_propagated_chord_clock_mismatch".into());
            }
            c["time_offset"] = json!(rational(offset));
            c["measure_number"] = json!(mi + 1);
            if !beams.is_empty() {
                c["beam_ids"] = json!(beam_ids);
            }
            graph.vertex(chord.key, c, false)?;
            crate::music_voices::assign_chord_voice(
                &mut graph,
                &mut m.voices,
                &m.measures,
                chord.id,
                voice_index,
            )?;
            let mut event = json!({"note_id":note["id"],"chord_id":chord.id,"system_id":measure.system,"staff_id":measure.staff,"measure_number":mi+1,"measure_index":measure.index,"bounds":note["bounds"],"shape":shape,"rest":rest,"measure_rest":measure_rest,"step":step,"octave":octave,"alter":alter,"effective_key_id":key_id,"effective_key_fifths":fifths,"duration":rational(duration),"duration_quarters":quarters(duration),"duration_eighths":eighths(duration),"flag_ids":flag_ids,"beam_ids":beam_ids,"stem_direction":stem_direction,"chord_stem_direction":chord_stem_direction,"time_offset":rational(offset),"offset_quarters":quarters(offset),"voice":voice_id,"type":if measure_rest {Value::Null}else{json!(note_type)},"dots":0,"effective_clef_id":cid});
            if !beam_marks.is_empty() {
                event["beams"] = json!(beam_marks);
            }
            events.push(event);
            offset = offset
                .checked_add(duration)
                .ok_or("musicxml_voice_duration_overflow")?;
        }
        // Voice.checkDuration uses the last actual AbstractChord.getEndTime.
        // Normal-voice termination/excess updates precede the retained strict
        // admission. Default whole-rest Voice uses unknown intrinsic duration;
        // general Rational/geometric slots/other special rests/pickup remain pending.
        // Consume original live relation-based clock propagation before final
        // stack slots. Sequential single-voice seeding/admission still applies;
        // this does not solve general voice matching or automatic ties.
        for chord in m.chords.iter().filter(|c| c.measure == mi) {
            let current = graph.snapshot();
            let time = current["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == chord.id)
                .and_then(|n| n["time_offset"].as_str())
                .ok_or("musicxml_chord_clock_required")?;
            crate::music_chord_time::set_and_push(
                &mut graph,
                &m.voices,
                &m.measures,
                chord.id,
                Some(crate::music_time::parse_ticks(time)?),
            )?;
        }
        let timed = graph.snapshot();
        for event in events.iter().filter(|e| e["measure_number"] == mi + 1) {
            let actual = timed["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == event["chord_id"])
                .ok_or("musicxml_chord_clock_required")?;
            if actual["time_offset"] != event["time_offset"] {
                return Err("musicxml_propagated_chord_clock_mismatch".into());
            }
        }
        crate::music_voices::generate_slots(&mut graph, &mut m.voices, &mut m.measures[mi], mi)?;
        m.measures[mi].timing.expected = Some(expected_ticks);
        crate::music_voices::read_stack_actual_duration(
            &graph,
            &m.voices,
            &mut m.measures[mi],
            mi,
        )?;
        crate::music_voices::check_duration(
            &graph,
            &mut m.voices[voice_index],
            &mut m.measures[mi],
        )?;
        let voice_end = crate::music_voices::last_end_ticks(&graph, &m.voices[voice_index])?;
        if m.measures[mi].timing.actual != Some(offset) {
            return Err("musicxml_owned_stack_timing_mismatch".into());
        }
        if if m.voices[voice_index].measure_rest_chord.is_some() {
            voice_end.is_some()
        } else {
            voice_end != Some(offset)
        } {
            return Err("musicxml_owned_voice_timing_mismatch".into());
        }
        if offset != expected_ticks {
            return Err(format!(
                "musicxml_incomplete_or_excess_measure:{}:{}:quarters={}:expected={}",
                measure.system,
                measure.index,
                quarters(offset),
                quarters(expected_ticks)
            ));
        }
        measures.push(json!({"number":mi+1,"system_id":measure.system,"staff_id":measure.staff,"index":measure.index,"left":measure.left,"right":measure.right,"numerator":meter.0,"denominator":meter.1,"time_symbol":carried_time_symbol,"expected_duration":rational(expected_ticks),"actual_duration":m.measures[mi].timing.actual.map(rational),"abnormal":m.measures[mi].timing.abnormal,"stack_excess":m.measures[mi].timing.excess.map(rational),"slots":crate::music_voices::slots_projection(&m.measures[mi].timing.slots),"voice":voice_id,"clef_id":clef_id,"key_id":key_id,"fifths":fifths}));
    }
    if measures.is_empty() {
        return Err("musicxml_no_measures".into());
    }
    // PartwiseBuilder uses a positive division value for exact durations.
    // Page.computeDurationDivisor uses exact rational denominators; binary
    // eighths/sixteenths need2/4, preserving1 for quarter/half-only pages.
    let divisions = if events.iter().any(|e| {
        e["duration_quarters"]
            .as_f64()
            .is_some_and(|d| (d * 2.).fract() != 0.)
    }) {
        4
    } else if events.iter().any(|e| {
        e["duration_quarters"]
            .as_f64()
            .is_some_and(|d| d.fract() != 0.)
    }) {
        2
    } else {
        1
    };
    let escaped_name = part_name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;");
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<score-partwise version=\"4.0\"><part-list><score-part id=\"P1\"><part-name>{escaped_name}</part-name></score-part></part-list><part id=\"P1\">\n"
    );
    let mut previous_system = None;
    let mut written_key = None;
    let mut written_clef = None;
    for measure in &measures {
        let number = measure["number"].as_u64().unwrap();
        writeln!(&mut xml, "<measure number=\"{number}\">").unwrap();
        if previous_system.is_some_and(|s| s != measure["system_id"]) {
            xml.push_str("<print new-system=\"yes\"/>\n");
        }
        let new_key = measure["fifths"].as_i64();
        let key_changed = new_key.is_some() && new_key != written_key;
        let clef = nodes
            .iter()
            .find(|n| n["id"] == measure["clef_id"])
            .ok_or("musicxml_export_clef_identity_required")?;
        let new_clef = clef_attributes(clef)?;
        let clef_changed = written_clef != Some(new_clef);
        if number == 1 || key_changed || clef_changed {
            xml.push_str("<attributes>");
            if number == 1 {
                write!(&mut xml, "<divisions>{divisions}</divisions>").unwrap();
            }
            if key_changed {
                write!(&mut xml, "<key><fifths>{}</fifths></key>", new_key.unwrap()).unwrap();
                written_key = new_key;
            }
            if number == 1 {
                // PartwiseBuilder.processTime sets TimeSymbol for COMMON/CUT.
                let symbol = match measure["time_symbol"].as_str() {
                    Some("common") => " symbol=\"common\"",
                    Some("cut") => " symbol=\"cut\"",
                    _ => "",
                };
                write!(
                    &mut xml,
                    "<time{symbol}><beats>{}</beats><beat-type>{}</beat-type></time>",
                    measure["numerator"], measure["denominator"]
                )
                .unwrap();
            }
            if clef_changed {
                write_clef(&mut xml, new_clef);
                written_clef = Some(new_clef);
            }
            xml.push_str("</attributes>\n");
        }
        for e in events.iter().filter(|e| e["measure_number"] == number) {
            xml.push_str("<note>");
            if e["rest"] == true {
                xml.push_str(if e["measure_rest"] == true {
                    r#"<rest measure="yes"/>"#
                } else {
                    "<rest/>"
                });
            } else {
                write!(
                    &mut xml,
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
                &mut xml,
                "<duration>{}</duration><voice>{}</voice>{}{}{}</note>",
                (e["duration_quarters"].as_f64().unwrap() * divisions as f64) as u32,
                e["voice"].as_u64().unwrap(),
                e["type"]
                    .as_str()
                    .map(|t| format!("<type>{t}</type>"))
                    .unwrap_or_default(),
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
        xml.push_str("</measure>\n");
        previous_system = Some(measure["system_id"].clone());
    }
    xml.push_str("</part></score-partwise>\n");
    let beam_group_voices = crate::music_voices::beam_groups(&graph, &m.voices, &m.measures)?;
    let voice_projection = crate::music_voices::timed_projection(&m.voices, &m.measures, &graph)?;
    m.owner.graph = graph;
    let previous = std::mem::take(&mut m.owner.prepared.report);
    m.owner.prepared.report = json!({"schema":"audiveris-native.musicxml-limited.v1","symbols":previous,"measures":measures,"events":events,"beam_group_voices":beam_group_voices,"voices":voice_projection,"shared_sig":m.owner.graph.snapshot(),"head_template_catalog":m.owner.catalog,"MusicXML_written_from_native_image_recognition":true,"original_XML_output_reused":false,"scope":"first limited clean Bravura44/47/interline11 zero-skew ordinary five-line pitched-native-header-clef/C-major-or-frozen-header-key/recognized constant binary meter single-voice sixteenth-eighth-quarter-half/default-whole-measure-rest complete-measure path; independent two/four-measure quarter-rest and half-note pages with system break, plus two controlled half-rest positions and one independent first-lower-ledger C4 page and one controlled split-quarter-rest input and one independent frozen-header G-major/F# page and one controlled frozen-header 4/4 TimePair variant and one 3/4 six-note G-major input and one fixed COMMON_TIME eight-note G-major input and two bass/alto-clef C-major eight-note inputs verified; one ordinary up-flag eighth/quarter and side/intermediate/sibling/separate-group/hook/same-glyph-competition images and one Bravura47 two-system beam image verified; arbitrary real-score admission remains unverified","pending":["broader real-score admission and general MusicXML engraving","other time/clef image comparisons, mid-measure clef changes and unpitched percussion, meter changes and finer/tuplet rhythms/full lateChecks/time columns/clusters above two parts/supported-frozen-cross-kind symbol conflicts/weak optionals","accidentals/other flags/unverified hook configurations/general shared-glyph aliases/cross-staff alien scenarios/more than two levels/dots/ties/tuplets/polyphony/pickup/multi-staff","full reduction/cleaner/epilog and original parity","nonzero skew/real scans/special staff/full24shapes/32sources"],"limited_scope_practical_validation_complete":false,"heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
    m.owner.prepared.report["preliminary_measure_slots"] = json!(preliminary_slots);
    m.owner.prepared.report["part_name"] = json!(part_name);
    m.owner.prepared.report["part_name_is_user_metadata_not_instrument_recognition"] = json!(true);
    if m.owner.prepared.report["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["beam_ids"].as_array().is_some_and(|b| !b.is_empty()))
    {
        m.owner.prepared.report["beam_comparison_scope"] = json!(
            "Original-successful ordinary beam images include side/intermediate/sibling/groups/hooks/competition and one measured Bravura47 two-system20-event input with one shared-glyph cross-system orphan removal; pitch/duration/onset/voice/beam and beamed stem-direction comparisons are separate from declared SIG conditions; full BEAMS/STEMS/SIG/cleaner/rhythm/engraving parity unclaimed"
        );
    }
    Ok(Export { owner: m, xml })
}
pub fn recognize(image: &GrayImage) -> Result<Export, String> {
    recognize_named(image, "Music")
}
/// Part name labels the single nontransposing part; it does not select notes,
/// override recognized meter, infer an instrument or enable transposition.
pub fn recognize_named(image: &GrayImage, part_name: &str) -> Result<Export, String> {
    if image.pixels.len() > 4_000_000 {
        return Err("musicxml_input_exceeds_verified_four_million_pixels_scope".into());
    }
    let owner = crate::stem_image_owned::prepare_items(image).map_err(|e| e.to_string())?;
    let owner = crate::music_links::prepare_beam_linking(owner)?;
    let owner = crate::stem_image_head_linking::continue_process(owner)?;
    let owner = crate::stem_foundation_consistency::continue_reduction(owner)?;
    let owner = crate::music_links::finalize_stems(owner)?;
    let m = crate::music_measures::continue_measures(owner)?;
    continue_export_named(crate::music_symbols::continue_symbols(m)?, part_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_clef_export_and_arpeggiato_empty_search_boundaries() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-clefs/dependency-controls.json"
        ))
        .unwrap();
        for c in o["clef_attributes"].as_array().unwrap() {
            let a = clef_attributes(c).unwrap();
            assert_eq!(a.sign, c["sign"].as_str().unwrap());
            assert_eq!(a.line, c["line"].as_i64().unwrap() as i32);
            assert_eq!(a.octave_change, c["octave_change"].as_i64().unwrap() as i32);
        }
        assert_eq!(o["profile"], 1);
        let chords: Vec<[i64; 4]> = serde_json::from_value(o["head_chord_bounds"].clone()).unwrap();
        for c in o["arpeggiato"].as_array().unwrap() {
            assert_eq!(
                arpeggiato_may_link(
                    serde_json::from_value(c["bounds"].clone()).unwrap(),
                    11,
                    &chords
                ),
                c["link_found"].as_bool().unwrap()
            );
        }
        assert!(clef_attributes(&json!({"shape":"PERCUSSION_CLEF","pitch":0})).is_err());
        assert!(clef_attributes(&json!({"shape":"C_CLEF"})).is_err());
        assert!(clef_attributes(&json!({"shape":"C_CLEF","pitch":100})).is_err());
    }
}
