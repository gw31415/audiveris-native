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
fn rational(quarters: u32) -> String {
    match quarters % 4 {
        0 => format!("{}", quarters / 4),
        2 => format!("{}/2", quarters / 2),
        _ => format!("{quarters}/4"),
    }
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
                    || matches!(s, "QUARTER_REST" | "HALF_REST") =>
                {
                    if t["created_inter_ids"]
                        .as_array()
                        .is_none_or(|ids| ids.is_empty())
                    {
                        return Err("musicxml_registered_symbol_identity_required".into());
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
        || m.owner.catalog.stored_point_size != 44
    {
        return Err("musicxml_verified_Bravura44_interline11_scope_required".into());
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
    let initial = m.owner.graph.snapshot();
    let nodes = initial["inters"].as_array().unwrap();
    let mut graph = m.owner.graph.clone();
    let mut events = Vec::new();
    let mut measures = Vec::new();
    let mut carried_meter = None;
    let mut carried_time_symbol = None;
    let mut carried_key: Option<(Value, i64)> = None;
    for (mi, measure) in m.measures.iter().enumerate() {
        let mut chords = m
            .chords
            .iter()
            .filter(|c| c.measure == mi)
            .collect::<Vec<_>>();
        chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
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
        // The present quarter/half-only voice uses integer quarter units.
        let whole_quarters = meter.0.checked_mul(4).ok_or("musicxml_meter_overflow")?;
        if meter.0 == 0 || meter.1 == 0 || whole_quarters % meter.1 != 0 {
            return Err("musicxml_fractional_quarter_measure_duration_pending".into());
        }
        let expected_quarters = whole_quarters / meter.1;
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
        let mut offset = 0;
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
        if initial_clef["shape"] != "G_CLEF" || initial_clef["pitch"] != 2. {
            return Err("musicxml_verified_treble_clef_scope_required".into());
        }
        let mut clef_id = initial_clef["id"].as_u64();
        for chord in chords {
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
            let (duration, note_type) = note_duration(shape)?;
            let rest = note["kind"] == "RestInter";
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
                if clef["shape"] != "G_CLEF" || clef["pitch"] != 2. {
                    return Err("musicxml_verified_treble_clef_scope_required".into());
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
                    let concrete = pitch == 6.
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
                clef_id = cid;
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
            let mut c = nodes
                .iter()
                .find(|n| n["id"] == chord.id)
                .ok_or("musicxml_actual_SIG_chord_required")?
                .clone();
            c["duration"] = json!(rational(duration));
            c["time_offset"] = json!(rational(offset));
            c["voice"] = json!(1);
            c["measure_number"] = json!(mi + 1);
            graph.vertex(chord.key, c, false)?;
            events.push(json!({"note_id":note["id"],"chord_id":chord.id,"system_id":measure.system,"staff_id":measure.staff,"measure_number":mi+1,"measure_index":measure.index,"bounds":note["bounds"],"shape":shape,"rest":rest,"step":step,"octave":octave,"alter":alter,"effective_key_id":key_id,"effective_key_fifths":fifths,"duration":rational(duration),"duration_quarters":duration,"time_offset":rational(offset),"offset_quarters":offset,"voice":1,"type":note_type,"dots":0,"effective_clef_id":cid}));
            offset += duration;
        }
        // Voice.getEndTime/checkDuration + strict complete-measure admission.
        // Pickup/underfilled measures require MeasureFixer and refuse here.
        if offset != expected_quarters {
            return Err(format!(
                "musicxml_incomplete_or_excess_measure:{}:{}:quarters={offset}:expected={expected_quarters}",
                measure.system, measure.index
            ));
        }
        measures.push(json!({"number":mi+1,"system_id":measure.system,"staff_id":measure.staff,"index":measure.index,"left":measure.left,"right":measure.right,"numerator":meter.0,"denominator":meter.1,"time_symbol":carried_time_symbol,"expected_duration":rational(expected_quarters),"actual_duration":rational(offset),"voice":1,"clef_id":clef_id,"key_id":key_id,"fifths":fifths}));
    }
    if measures.is_empty() {
        return Err("musicxml_no_measures".into());
    }
    // Quarter and half durations have integer quarter units. Keep divisions
    // positive even for all-half pages (unlike an unguarded divisor/4=0).
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
    for measure in &measures {
        let number = measure["number"].as_u64().unwrap();
        writeln!(&mut xml, "<measure number=\"{number}\">").unwrap();
        if previous_system.is_some_and(|s| s != measure["system_id"]) {
            xml.push_str("<print new-system=\"yes\"/>\n");
        }
        let new_key = measure["fifths"].as_i64();
        let key_changed = new_key.is_some() && new_key != written_key;
        if number == 1 || key_changed {
            xml.push_str("<attributes>");
            if number == 1 {
                xml.push_str("<divisions>1</divisions>");
            }
            if key_changed {
                write!(&mut xml, "<key><fifths>{}</fifths></key>", new_key.unwrap()).unwrap();
                written_key = new_key;
            }
            if number == 1 {
                let clef = nodes
                    .iter()
                    .find(|n| n["id"] == measure["clef_id"])
                    .ok_or("musicxml_export_clef_identity_required")?;
                // Original PartwiseBuilder uses 3 - rint(clef.pitch / 2).
                let line = 3 - (clef["pitch"].as_f64().unwrap() / 2.).round_ties_even() as i32;
                // PartwiseBuilder.processTime sets TimeSymbol for COMMON/CUT.
                let symbol = match measure["time_symbol"].as_str() {
                    Some("common") => " symbol=\"common\"",
                    Some("cut") => " symbol=\"cut\"",
                    _ => "",
                };
                write!(&mut xml, "<time{symbol}><beats>{}</beats><beat-type>{}</beat-type></time><clef><sign>G</sign><line>{line}</line></clef>", measure["numerator"], measure["denominator"]).unwrap();
            }
            xml.push_str("</attributes>\n");
        }
        for e in events.iter().filter(|e| e["measure_number"] == number) {
            xml.push_str("<note>");
            if e["rest"] == true {
                xml.push_str("<rest/>");
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
                "<duration>{}</duration><voice>1</voice><type>{}</type></note>",
                e["duration_quarters"],
                e["type"].as_str().unwrap()
            )
            .unwrap();
        }
        xml.push_str("</measure>\n");
        previous_system = Some(measure["system_id"].clone());
    }
    xml.push_str("</part></score-partwise>\n");
    m.owner.graph = graph;
    let previous = std::mem::take(&mut m.owner.prepared.report);
    m.owner.prepared.report = json!({"schema":"audiveris-native.musicxml-limited.v1","symbols":previous,"measures":measures,"events":events,"shared_sig":m.owner.graph.snapshot(),"MusicXML_written_from_native_image_recognition":true,"original_XML_output_reused":false,"scope":"first limited clean Bravura44/interline11 zero-skew ordinary five-line treble/C-major-or-frozen-header-key/recognized constant integer-quarter meter single-voice quarter-half complete-measure path; independent two/four-measure quarter-rest and half-note pages with system break, plus two controlled half-rest positions and one independent first-lower-ledger C4 page and one controlled split-quarter-rest input and one independent frozen-header G-major/F# page and one controlled frozen-header 4/4 TimePair variant and one 3/4 six-note G-major input and one fixed COMMON_TIME eight-note G-major input verified; arbitrary real-score admission remains unverified","pending":["broader real-score admission and general MusicXML engraving","other time/clef image comparisons, meter changes and fractional-quarter rhythms/full lateChecks/time columns/clusters above two parts/supported-frozen-cross-kind symbol conflicts/weak optionals","accidentals/flags/beams/dots/ties/tuplets/polyphony/pickup/multi-staff","full reduction/cleaner/epilog and original parity","nonzero skew/real scans/special staff/full24shapes/32sources"],"limited_scope_practical_validation_complete":false,"heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
    m.owner.prepared.report["part_name"] = json!(part_name);
    m.owner.prepared.report["part_name_is_user_metadata_not_instrument_recognition"] = json!(true);
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
    let owner = crate::music_links::remove_unreachable_beams(owner)?;
    let owner = crate::stem_image_head_linking::continue_linking(owner)?;
    let owner = crate::stem_foundation_purge::continue_purge(owner)?;
    let owner = crate::stem_head_foundations::continue_reduction(owner)?;
    let owner = crate::music_links::finalize_stems(owner)?;
    let m = crate::music_measures::continue_measures(owner)?;
    continue_export_named(crate::music_symbols::continue_symbols(m)?, part_name)
}
