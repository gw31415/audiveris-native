// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded ordinary-Voice continuation and native MusicXML writer.
//! Shape.getNoteDuration, MeasureRhythm.processStartingChords, Voice.addChord,
//! AbstractChordInter.getEndTime, Page.numberMeasures/simpleDurationOf and
//! PartwiseBuilder note/attribute ordering guide this explicitly bounded path.
pub(crate) mod beam_rest;
mod beam_rest_detect;
mod beam_rest_lookup;
#[cfg(test)]
mod selected_cli_tests;
pub(crate) mod stack_rhythm;
mod tuplet_getters;
mod tuplet_groups;
pub use tuplet_getters::TupletRead;
mod tuplet_generation;
mod tuplet_placement;
mod tuplet_preparation;
pub use tuplet_groups::TupletGroups;
pub use tuplet_placement::TupletPlan;

use crate::{GrayImage, music_measures::OwnedMusic, music_symbols, shared_sig::Key};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt::Write};

pub struct Export {
    pub owner: OwnedMusic,
    pub xml: String,
    provenance: [u8; 32],
}
struct OwnedCheckpoint {
    graph: crate::shared_sig::Graph,
    measures: Vec<crate::music_measures::Measure>,
    chords: Vec<crate::music_measures::Chord>,
    voices: Vec<crate::music_voices::Voice>,
    report: Value,
}
impl OwnedCheckpoint {
    fn capture(owner: &OwnedMusic) -> Self {
        Self {
            graph: owner.owner.graph.clone(),
            measures: owner.measures.clone(),
            chords: owner.chords.clone(),
            voices: owner.voices.clone(),
            report: owner.owner.prepared.report.clone(),
        }
    }
    fn restore(self, owner: &mut OwnedMusic) {
        owner.owner.graph = self.graph;
        owner.measures.clone_from(&self.measures);
        owner.chords.clone_from(&self.chords);
        owner.voices.clone_from(&self.voices);
        // This is the pre-edit projection, not an input for rebuilding owners.
        owner.owner.prepared.report = self.report;
    }
}
impl Export {
    /// Rebuild selected admitted image stacks in first encounter order, then
    /// bind MusicXML to the resulting actual Voice/Slot clocks.
    pub fn reprocess_stack_rhythms(&mut self, stack_numbers: &[usize]) -> Result<Value, String> {
        if provenance(&self.owner) != self.provenance {
            return Err("music_reprocess_unedited_retained_export_required".into());
        }
        if stack_numbers.is_empty() {
            return Err("music_reprocess_actual_stack_owner_required".into());
        }
        let mut selected = Vec::new();
        for number in stack_numbers {
            let mi = number
                .checked_sub(1)
                .filter(|mi| *mi < self.owner.measures.len())
                .ok_or("music_reprocess_actual_stack_owner_required")?;
            if !selected.contains(&mi) {
                selected.push(mi);
            }
        }
        let rhythm = Rhythm::retained(&self.owner)?;
        let part_name = self.owner.owner.prepared.report["part_name"]
            .as_str()
            .ok_or("music_reprocess_retained_part_name_required")?
            .to_owned();
        let report = self.owner.owner.prepared.report.clone();
        let graph = self.owner.owner.graph.clone();
        let graph = std::mem::replace(&mut self.owner.owner.graph, graph);
        let voices = self.owner.voices.clone();
        let voices = std::mem::replace(&mut self.owner.voices, voices);
        let measures = self.owner.measures.clone();
        let mut measures = std::mem::replace(&mut self.owner.measures, measures);
        // Stage rhythm containers without replacing actual Chords. Keep the
        // original Slot storage intact until XML binding also succeeds.
        let result = self.reprocess_selected(&selected, rhythm, &part_name);
        if result.is_ok() {
            for &mi in &selected {
                measures[mi].timing = std::mem::take(&mut self.owner.measures[mi].timing);
                measures[mi].tuplets = std::mem::take(&mut self.owner.measures[mi].tuplets);
                measures[mi].stack_tuplets =
                    std::mem::take(&mut self.owner.measures[mi].stack_tuplets);
            }
        }
        self.owner.measures = measures;
        match result {
            Ok(xml) => {
                let actual = json!({"requested_stack_numbers":stack_numbers,
                    "processed_stack_numbers":selected.iter().map(|mi| mi+1).collect::<Vec<_>>(),
                    "scope":"retained admitted image stacks",
                    "controller_impact_complete":false,"PageStep_complete":false});
                if let Some(editing) = report.get("editing_reprocess") {
                    self.owner.owner.prepared.report["editing_reprocess"] = editing.clone();
                }
                self.owner.owner.prepared.report["selected_rhythm_reprocess"] = actual.clone();
                self.xml = xml;
                self.provenance = provenance(&self.owner);
                Ok(actual)
            }
            Err(error) => {
                self.owner.owner.graph = graph;
                self.owner.voices = voices;
                self.owner.owner.prepared.report = report;
                Err(error)
            }
        }
    }
    /// Prepare an explicit transfer using this retained image's live identities.
    /// No recognition search, manual membership rebuilding or report input.
    pub fn dot_support_transfer(
        &self,
        dot_id: usize,
        head_id: usize,
    ) -> Result<crate::shared_sig::EditingSequence, String> {
        self.support_transfer(dot_id, head_id, None)
    }
    /// Declared relation correction: preserve supplied status and use actual
    /// UnlinkTask then LinkTask, not the editor's add-before-remove sequence.
    pub fn dot_relation_transfer(
        &self,
        dot_id: usize,
        head_id: usize,
        manual: bool,
    ) -> Result<crate::shared_sig::EditingSequence, String> {
        self.support_transfer(dot_id, head_id, Some(manual))
    }
    fn support_transfer(
        &self,
        dot_id: usize,
        head_id: usize,
        relation_manual: Option<bool>,
    ) -> Result<crate::shared_sig::EditingSequence, String> {
        use crate::shared_sig::{
            DefaultEditingTask, EditingLink, EditingSequence, EditingTask, RelationTask,
        };
        let graph = &self.owner.owner.graph;
        let snapshot = graph.snapshot();
        let nodes = snapshot["inters"].as_array().unwrap();
        let dot = nodes
            .iter()
            .find(|n| n["id"] == dot_id && n["kind"] == "AugmentationDotInter")
            .ok_or("editing_live_dot_identity_required")?;
        let head = nodes
            .iter()
            .find(|n| n["id"] == head_id && n["kind"] == "HeadInter")
            .ok_or("editing_live_head_identity_required")?;
        let edges = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["source"] == dot_id && e["kind"] == "AugmentationRelation")
            .collect::<Vec<_>>();
        if edges.len() != 1 || edges[0]["target"] == head_id {
            return Err("editing_distinct_single_dot_support_required".into());
        }
        let key: Key = serde_json::from_value(dot["origin"].clone())
            .map_err(|_| "editing_inter_identity_required")?;
        let target: Key = serde_json::from_value(head["origin"].clone())
            .map_err(|_| "editing_partner_identity_required")?;
        let edge = edges[0]["id"].as_u64().unwrap() as usize;
        let sequence = if let Some(manual) = relation_manual {
            EditingSequence::from_tasks(vec![
                EditingTask::Relation(RelationTask::unlink(graph, edge)?),
                EditingTask::Relation(RelationTask::link(
                    graph,
                    key,
                    target,
                    json!({"kind":"AugmentationRelation","manual":manual}),
                )?),
            ])
        } else {
            let unlink = EditingLink::from_registered(graph, key, edge)?;
            let link = EditingLink::new(
                target,
                true,
                json!({"kind":"AugmentationRelation","manual":true}),
            );
            let bounds = serde_json::from_value(dot["bounds"].clone())
                .map_err(|_| "music_dot_bounds_required")?;
            EditingSequence::new(vec![DefaultEditingTask::new(
                graph,
                key,
                bounds,
                vec![link],
                vec![unlink],
            )?])
        };
        sequence.require_ordinary_reprocess(graph)?;
        Ok(sequence)
    }
    /// Apply compared ordinary dot bounds/support tasks, then process their actual
    /// stacks with cached expected durations. This does not rerun FRATs/LINKS
    /// or complete PageStep/history/general timed support correction. Ordinary
    /// single-head same-stack support transfers retain callbacks before reset.
    pub fn apply_editing_sequence(
        &mut self,
        sequence: &mut crate::shared_sig::EditingSequence,
        operation: crate::shared_sig::EditingOperation,
    ) -> Result<Value, String> {
        if provenance(&self.owner) != self.provenance {
            return Err("music_reprocess_unedited_retained_export_required".into());
        }
        sequence.require_ordinary_reprocess(&self.owner.owner.graph)?;
        let old_rhythm = Rhythm::retained(&self.owner)?;
        let part_name = self.owner.owner.prepared.report["part_name"]
            .as_str()
            .ok_or("music_reprocess_retained_part_name_required")?
            .to_owned();
        let before_graph = self.owner.owner.graph.clone();
        let mut graph = before_graph.clone();
        let mut next_sequence = sequence.clone();
        let planned = next_sequence.perform(&mut graph, operation, "PAGE")?;
        next_sequence.require_same_stack(&before_graph, &graph)?;
        next_sequence.require_single_support(&graph)?;
        let targets = planned["rhythm_stack_targets"]
            .as_array()
            .ok_or("music_reprocess_stack_targets_required")?;
        let selected = targets
            .iter()
            .map(|t| {
                self.owner
                    .measures
                    .iter()
                    .position(|m| {
                        t["system_id"] == m.system
                            && t["measure_index"] == m.index
                            && t["left"] == m.left
                            && t["right"] == m.right
                    })
                    .ok_or_else(|| "music_reprocess_actual_stack_owner_required".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if selected.is_empty() {
            return Err("music_reprocess_actual_stack_owner_required".into());
        }
        let checkpoint = OwnedCheckpoint::capture(&self.owner);
        self.owner.owner.graph = graph;
        let result = self.reprocess_selected(&selected, old_rhythm, &part_name);
        match result {
            Ok(xml) => {
                let actual = json!({"planned_impact":planned,"processed_stack_targets":targets,"native_stack_rhythm":"existing admitted owned Measure pass -> finalCheck -> stack slots/actual","system_voice_refinement":"original HIGH purge/sort/rename, classified tie, previous-Measure BeamGroup predecessor, NextInVoice, then preferred-ID swaps; general Voice kinds and automatic link discovery pending","populateFRATs_called":false,"LINKS_called":false,"controller_impact_complete":false,"PageStep_complete":false});
                self.owner.owner.prepared.report["editing_reprocess"] = actual.clone();
                self.xml = xml;
                self.provenance = provenance(&self.owner);
                *sequence = next_sequence;
                Ok(actual)
            }
            Err(error) => {
                checkpoint.restore(&mut self.owner);
                Err(error)
            }
        }
    }
    fn reprocess_selected(
        &mut self,
        selected: &[usize],
        old_rhythm: Rhythm,
        part_name: &str,
    ) -> Result<String, String> {
        self.reprocess_selected_scoped(selected, old_rhythm, part_name, RhythmScope::Image)
    }
    fn reprocess_selected_scoped(
        &mut self,
        selected: &[usize],
        old_rhythm: Rhythm,
        part_name: &str,
        scope: RhythmScope,
    ) -> Result<String, String> {
        let mut rhythm = old_rhythm;
        let mut systems = Vec::new();
        for &mi in selected {
            rhythm = analyze_rhythm(
                &mut self.owner,
                Some((std::slice::from_ref(&mi), &rhythm)),
                scope,
            )?;
            // Original reprocessStack refines after each processed stack,
            // even if the same system occurs again in the impact list.
            let system = stack_rhythm::Context {
                graph: &mut self.owner.owner.graph,
                voices: &mut self.owner.voices,
                measures: &mut self.owner.measures,
                chords: &self.owner.chords,
                interline: self.owner.owner.interline,
                rest_geometry: beam_rest::RestGeometry {
                    staffs: &self.owner.owner.staffs,
                    skew: self.owner.owner.sheet_skew,
                    interline: self.owner.owner.interline,
                },
            }
            .refine_system(mi)?;
            systems.push(system);
        }
        for (mi, measure) in self.owner.measures.iter().enumerate() {
            if !systems.contains(&measure.system) {
                continue;
            }
            for event in rhythm
                .events
                .iter_mut()
                .filter(|e| e["measure_number"] == mi + 1)
            {
                crate::music_voice_export::bind_event(
                    &self.owner.owner.graph,
                    &self.owner.voices,
                    measure,
                    mi,
                    event,
                )?;
            }
            rhythm.measures[mi]["voice"] = json!(
                self.owner
                    .voices
                    .iter()
                    .find(|v| v.measure == mi)
                    .map(|v| v.id)
            );
        }
        render(&mut self.owner, part_name, rhythm, false)
    }
}
#[cfg(test)]
use crate::music_time::format_ticks as rational;
use crate::music_time::{Fraction, format_fraction};
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
use crate::music_duration::ExactChordDuration;

fn resolve_durations(
    graph: &mut crate::shared_sig::Graph,
    chords: &[&crate::music_measures::Chord],
    scope: RhythmScope,
) -> Result<std::collections::BTreeMap<usize, ExactChordDuration>, String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let edges = snapshot["relations"].as_array().unwrap();
    let mut resolved = std::collections::BTreeMap::new();
    let mut staged = graph.clone();
    for chord in chords {
        let live_members = edges
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == chord.id)
            .map(|e| e["target"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let owned_members = chord
            .members
            .iter()
            .map(|n| n["id"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        if live_members != owned_members || owned_members.len() != chord.members.len() {
            return Err("musicxml_actual_chord_members_required".into());
        }
        let live_stem = edges
            .iter()
            .filter(|e| e["kind"] == "ChordStemRelation" && e["source"] == chord.id)
            .map(|e| e["target"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let owned_stem = chord
            .stem
            .map(|key| graph.id(key).ok_or("musicxml_actual_chord_stem_required"))
            .transpose()?
            .into_iter()
            .collect::<Vec<_>>();
        if live_stem != owned_stem {
            return Err("musicxml_actual_chord_stem_required".into());
        }
        // Original ordinary getDuration reads the actual chord's getBeams.
        // A retained cache must be warmed by that read, not by report creation.
        staged.warm_retained_chord_beams(chord.key)?;
        let result = crate::music_duration::live_exact(&snapshot, chord.id)?;
        if scope == RhythmScope::Image {
            result
                .intrinsic
                .map(|d| {
                    d.ticks()
                        .map_err(|_| "musicxml_dotted_duration_finer_than_sixteenth_pending")
                })
                .transpose()?;
        }
        let mut node = nodes
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("musicxml_duration_live_chord_required")?
            .clone();
        node["duration"] = json!(result.intrinsic.map(format_fraction));
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
    if m.owner.prepared.report["symbol_cluster_scope"] != "original-seven-part-cap-recursive" {
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
                    let snapshot = m.owner.graph.snapshot();
                    // Admission follows the actual countDots removal, never
                    // an absent classifier candidate or a guessed undotted note.
                    let discarded = m.owner.prepared.report["limited_links"]["dot_counting"]
                        ["discarded_inconsistent_dots"]
                        .as_array()
                        .ok_or("musicxml_owned_dot_counting_required")?
                        .iter()
                        .any(|n| n["glyph_id"] == t["glyph_id"]
                            && n["bounds"] == t["bounds"]
                            && n["kind"] == "AugmentationDotInter"
                            && n["reason"] == "AbstractChord.countDots_inconsistent_nonmanual_zero");
                    if crate::music_dots::distant_from_line(b, s, m.owner.interline, &snapshot)?
                        && !discarded
                        && !snapshot["inters"].as_array().unwrap().iter().any(|n| {
                            n["kind"] == "AugmentationDotInter"
                                && n["glyph_id"] == t["glyph_id"]
                                && snapshot["relations"].as_array().unwrap().iter().any(|r| {
                                    matches!(
                                        r["kind"].as_str(),
                                        Some("AugmentationRelation" | "DoubleDotRelation")
                                    ) && r["source"] == n["id"]
                                })
                        })
                    {
                        return Err("musicxml_possible_dot_lateChecks_pending".into());
                    }
                }
                "CAESURA" | "BREATH_MARK" => {
                    // The original factory creates a real pause and LINKS
                    // contextualizes/purges it; a surviving pause needs notation.
                    let removed=m.owner.prepared.report["limited_links"]["pause_weak_purge"]["removed_weak_pauses"].as_array().ok_or("musicxml_pause_purge_required")?;
                    if !removed
                        .iter()
                        .any(|n| n["glyph_id"] == t["glyph_id"] && n["shape"] == shape)
                    {
                        return Err("musicxml_positive_pause_notation_pending".into());
                    }
                }
                "FLAT" => {
                    let removed=m.owner.prepared.report["limited_links"]["alter_weak_purge"]["removed_weak_alters"].as_array().ok_or("musicxml_alter_purge_required")?;
                    if !removed
                        .iter()
                        .any(|n| n["glyph_id"] == t["glyph_id"] && n["shape"] == shape)
                    {
                        return Err("musicxml_positive_accidental_notation_pending".into());
                    }
                }
                s if music_symbols::time(s).is_some()
                    || matches!(
                        s,
                        "QUARTER_REST"
                            | "HALF_REST"
                            | "WHOLE_REST"
                            | "FLAG_1"
                            | "FLAG_1_DOWN"
                            | "FLAG_2"
                            | "FLAG_2_DOWN"
                    ) =>
                {
                    if t["created_inter_ids"]
                        .as_array()
                        .is_none_or(|ids| ids.is_empty())
                    {
                        return Err("musicxml_registered_symbol_identity_required".into());
                    }
                    if crate::music_flags::value(s).is_some() {
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
    m.owner.graph.populate_dot_measures()?;
    // A new retained editing owner is not an imported/persisted Book. Do not
    // infer prior dirty flags or clear them when writing MusicXML.
    m.owner
        .graph
        .bind_editing_book(m.measures.iter().map(|m| m.system).collect())?;
    m.owner.graph.bind_retained_chord_caches(
        &m.chords
            .iter()
            .filter(|c| c.kind == crate::music_measures::ChordKind::Head)
            .map(|c| (c.key, c.stem))
            .collect::<Vec<_>>(),
    )?;
    let rhythm = process_rhythm(&mut m, None)?;
    let xml = render(&mut m, part_name, rhythm, true)?;
    let provenance = provenance(&m);
    Ok(Export {
        owner: m,
        xml,
        provenance,
    })
}

struct Rhythm {
    measures: Vec<Value>,
    events: Vec<Value>,
    preliminary_slots: Vec<Value>,
}
impl Rhythm {
    fn retained(m: &OwnedMusic) -> Result<Self, String> {
        let report = &m.owner.prepared.report;
        let rows = |name| {
            report[name]
                .as_array()
                .cloned()
                .ok_or_else(|| format!("music_reprocess_retained_{name}_required"))
        };
        Ok(Self {
            measures: rows("measures")?,
            events: rows("events")?,
            preliminary_slots: rows("preliminary_measure_slots")?,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RhythmScope {
    Image,
    OwnedRhythm,
    // Public correction admission still needs original RelationTask/controller.
    #[cfg(test)]
    ObservedOwnedCorrection,
}
fn process_rhythm(
    m: &mut OwnedMusic,
    retained: Option<(&[usize], &Rhythm)>,
) -> Result<Rhythm, String> {
    analyze_rhythm(m, retained, RhythmScope::Image)
}
fn analyze_rhythm(
    m: &mut OwnedMusic,
    retained: Option<(&[usize], &Rhythm)>,
    scope: RhythmScope,
) -> Result<Rhythm, String> {
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
        // Exact binary fractions through sixteenths. Finer/tuplets remain pending.
        let whole_ticks = meter.0.checked_mul(16).ok_or("musicxml_meter_overflow")?;
        if meter.0 == 0 || meter.1 == 0 || whole_ticks % meter.1 != 0 {
            return Err("musicxml_measure_duration_finer_than_sixteenth_pending".into());
        }
        let expected_duration = if retained.is_some() {
            m.measures[mi]
                .timing
                .expected
                .ok_or("music_reprocess_unknown_expected_export_pending")?
        } else {
            Fraction::new(meter.0, meter.1)?
        };
        if scope == RhythmScope::Image {
            expected_duration.ticks()?;
        }
        if let Some((selected, old)) = retained
            && !selected.contains(&mi)
        {
            let row = old
                .measures
                .get(mi)
                .ok_or("music_reprocess_retained_measure_required")?
                .clone();
            carried_key = row["fifths"].as_i64().map(|f| (row["key_id"].clone(), f));
            measures.push(row);
            preliminary_slots.push(
                old.preliminary_slots
                    .get(mi)
                    .ok_or("music_reprocess_retained_slots_required")?
                    .clone(),
            );
            events.extend(
                old.events
                    .iter()
                    .filter(|e| e["measure_number"] == mi + 1)
                    .cloned(),
            );
            continue;
        }
        crate::music_voices::reset_stack_rhythm(&mut m.measures[mi], Some(expected_duration));
        tuplet_preparation::prepare_on(
            &mut graph,
            &mut m.measures,
            &m.chords,
            beam_rest::RestGeometry {
                staffs: &m.owner.staffs,
                skew: m.owner.sheet_skew,
                interline: m.owner.interline,
            },
            mi + 1,
        )?;
        crate::music_voices::reset_rhythm(
            &mut graph,
            &mut m.voices,
            &mut m.measures[mi],
            &m.chords,
            mi,
        )?;
        // Original geometric retrieval precedes rhythm clocks/Voice matching.
        let (narrow, wide) =
            crate::music_measure_slots::from_owned(&graph, &measure, &chords, m.owner.interline)?;
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
        chords.sort_by_key(|c| ordered_ids.iter().position(|id| *id == c.id));
        preliminary_slots.push(json!({"measure_number":mi+1,"narrow":narrow,"wide":wide,"compounds":compounds.iter().map(|c|c.projection()).collect::<Vec<_>>(),"voice_assignment":"original separated ChordsMapper on owned ordinary Voice/compound path; general Rational/implicit tuplets/Merged/shared stacks pending"}));
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
        let mut offsets = BTreeMap::<usize, Fraction>::new();
        let mut previous_right = BTreeMap::<usize, i64>::new();
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
        let durations = resolve_durations(&mut graph, &chords, scope)?;
        // Parallel admission follows the verified original owned duration and
        // group propagation path. Higher flags, levels above two and other meters
        // need their own acquisition/Voice comparisons before joining it.
        if scope == RhythmScope::Image
            && compounds.iter().any(|s| s.chord_ids.len() > 1)
            && (meter != (4, 4)
                || chords.iter().any(|c| {
                    let d = durations[&c.id];
                    c.kind != crate::music_measures::ChordKind::Head
                        || !match d.beams_or_flags {
                            0 => d.intrinsic.is_some_and(|v| {
                                v == Fraction::from_ticks(4) || v == Fraction::from_ticks(8)
                            }),
                            1 | 2 => {
                                d.intrinsic == Some(Fraction::from_ticks(4 >> d.beams_or_flags))
                            }
                            _ => false,
                        }
                }))
        {
            return Err("musicxml_parallel_quarter_voice_scope_pending".into());
        }
        let mapped = stack_rhythm::Context {
            graph: &mut graph,
            voices: &mut m.voices,
            measures: &mut m.measures,
            chords: &m.chords,
            interline: m.owner.interline,
            rest_geometry: beam_rest::RestGeometry {
                staffs: &m.owner.staffs,
                skew: m.owner.sheet_skew,
                interline: m.owner.interline,
            },
        }
        .map(stack_rhythm::Prepared {
            chords: &chords,
            narrow: &narrow,
            compounds: &compounds,
            expected: expected_duration,
            clef: initial_clef,
            measure_index: mi,
        })?;
        preliminary_slots[mi]
            .as_object_mut()
            .unwrap()
            .extend(mapped.as_object().unwrap().clone());
        let voice_id = m
            .voices
            .iter()
            .find(|v| v.measure == mi)
            .ok_or("musicxml_starting_actual_voice_required")?
            .id;
        for &chord in &chords {
            let current_chords = graph.snapshot();
            let mut c = current_chords["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == chord.id)
                .ok_or("musicxml_actual_SIG_chord_required")?
                .clone();
            let owned_voice = m
                .voices
                .iter()
                .find(|v| v.measure == mi && c["voice"] == v.id)
                .ok_or("musicxml_general_new_voice_export_pending")?;
            let event_voice = owned_voice.id;
            let cursor = offsets.get(&event_voice).copied().unwrap_or(Fraction::ZERO);
            let offset = Fraction::parse(
                c["time_offset"]
                    .as_str()
                    .ok_or("musicxml_unknown_voice_clock_pending")?,
            )?;
            // PartwiseBuilder advances to a known onset with a forward. A
            // backwards/overlapping Voice clock still requires rhythm repair.
            if offset < cursor {
                return Err("musicxml_starting_propagated_chord_clock_mismatch".into());
            }
            if previous_right
                .get(&event_voice)
                .is_some_and(|&x| chord.bounds[0] <= x)
            {
                return Err("musicxml_parallel_or_overlapping_chords_pending".into());
            }
            previous_right.insert(event_voice, chord.bounds[0] + chord.bounds[2] - 1);
            let measure_rest = owned_voice.measure_rest_chord == Some(chord.id);
            let ExactChordDuration {
                intrinsic,
                note_type,
                dots,
                ..
            } = durations[&chord.id];
            let duration = intrinsic.unwrap_or(expected_duration);
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
            let (head_y, tail_y, chord_stem_direction) = if let Some(stem) = chord.stem {
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
                let tail_y = geometry
                    .tail_location
                    .ok_or("musicxml_chord_tail_location_required")?[1];
                (head_y, Some(tail_y), geometry.stem_dir(true)?)
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
                for note in &chord.members {
                    edges
                        .iter()
                        .find(|e| {
                            e["kind"] == "HeadStemRelation"
                                && e["source"] == note["id"]
                                && e["target"] == stem_id
                        })
                        .ok_or("musicxml_beam_head_direction_required")?;
                }
            }
            let beam_ids = beams.iter().map(|b| b["id"].clone()).collect::<Vec<_>>();
            let intrinsic_duration = intrinsic.map(format_fraction);
            if c["duration"] != json!(intrinsic_duration) {
                return Err("musicxml_chord_intrinsic_duration_mismatch".into());
            }
            c["measure_number"] = json!(mi + 1);
            if !beams.is_empty() {
                c["beam_ids"] = json!(beam_ids);
            }
            graph.vertex(chord.key, c, false)?;
            // AbstractChord.getNotes uses reverse center ordinate, already
            // retained by CHORDS. Each member owns its pitch; clocks belong to
            // the ensemble and advance once after every member is emitted.
            for note in &chord.members {
                let shape = note["shape"]
                    .as_str()
                    .ok_or("musicxml_note_shape_required")?;
                let rest = note["kind"] == "RestInter";
                let stem_direction = tail_y.map(|tail| {
                    let center_y = note["bounds"][1].as_i64().unwrap()
                        + note["bounds"][3].as_i64().unwrap() / 2;
                    if tail < center_y { "up" } else { "down" }
                });
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
                let mut event = json!({"note_id":note["id"],"chord_id":chord.id,"bounds":note["bounds"],"shape":shape,"rest":rest,"step":step,"octave":octave,"alter":alter,"effective_key_id":key_id,"effective_key_fifths":fifths,"flag_ids":flag_ids,"beam_ids":beam_ids,"stem_direction":stem_direction,"chord_stem_direction":chord_stem_direction,"time_offset":format_fraction(offset),"type":if measure_rest {Value::Null}else{json!(note_type)},"dots":dots,"effective_clef_id":cid});
                if !beam_marks.is_empty() {
                    event["beams"] = json!(beam_marks);
                }
                events.push(event);
            }
            offsets.insert(event_voice, offset.plus(duration)?);
        }
        // Voice.checkDuration uses the last actual AbstractChord.getEndTime.
        // Normal-voice termination/excess updates precede the retained strict
        // admission. Default whole-rest Voice uses unknown intrinsic duration;
        // general Rational/geometric slots/other special rests/pickup remain pending.
        // Consume original live relation-based clock propagation before final
        // stack slots. Complete ordinary-Voice admission still applies; general
        // gaps/implicit tuplets/Merged/shared stacks/automatic ties remain pending.
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
                Some(Fraction::parse(time)?),
            )?;
        }
        let timed = graph.snapshot();
        for event in events
            .iter()
            .filter(|e| chords.iter().any(|c| e["chord_id"] == c.id))
        {
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
        if !crate::music_voices::final_check(
            &graph,
            &mut m.voices,
            &mut m.measures[mi],
            &m.chords,
            mi,
        )? {
            return Err("musicxml_original_measure_final_check_failed".into());
        }
        crate::music_voices::generate_slots(&mut graph, &mut m.voices, &mut m.measures[mi], mi)?;
        crate::music_voices::read_stack_actual_duration(
            &graph,
            &m.voices,
            &mut m.measures[mi],
            mi,
        )?;
        for event in events
            .iter_mut()
            .filter(|e| chords.iter().any(|c| e["chord_id"] == c.id))
        {
            crate::music_voice_export::bind_event(&graph, &m.voices, &m.measures[mi], mi, event)?;
        }
        let offset = offsets.values().copied().max().unwrap_or(Fraction::ZERO);
        if m.measures[mi].timing.actual != Some(offset) {
            return Err("musicxml_owned_stack_timing_mismatch".into());
        }
        for voice in m.voices.iter().filter(|v| v.measure == mi) {
            let voice_end = crate::music_voices::last_end_time(&graph, voice)?;
            let offset = offsets
                .get(&voice.id)
                .copied()
                .ok_or("musicxml_voice_event_required")?;
            if if voice.measure_rest_chord.is_some() {
                voice_end.is_some()
            } else {
                voice_end != Some(offset)
            } {
                return Err("musicxml_owned_voice_timing_mismatch".into());
            }
            if scope == RhythmScope::Image && offset != expected_duration {
                return Err(format!(
                    "musicxml_incomplete_or_excess_measure:{}:{}:quarters={}:expected={}",
                    measure.system,
                    measure.index,
                    offset.reported_units(4),
                    expected_duration.reported_units(4)
                ));
            }
        }
        measures.push(json!({"number":mi+1,"system_id":measure.system,"staff_id":measure.staff,"index":measure.index,"left":measure.left,"right":measure.right,"numerator":meter.0,"denominator":meter.1,"time_symbol":carried_time_symbol,"expected_duration":format_fraction(expected_duration),"actual_duration":m.measures[mi].timing.actual.map(format_fraction),"abnormal":m.measures[mi].timing.abnormal,"stack_excess":m.measures[mi].timing.excess.map(format_fraction),"slots":crate::music_voices::slots_projection(&m.measures[mi].timing.slots),"voice":voice_id,"clef_id":clef_id,"key_id":key_id,"fifths":fifths}));
    }
    if measures.is_empty() {
        return Err("musicxml_no_measures".into());
    }
    m.owner.graph = graph;
    Ok(Rhythm {
        measures,
        events,
        preliminary_slots,
    })
}

fn render(
    m: &mut OwnedMusic,
    part_name: &str,
    rhythm: Rhythm,
    image_entry: bool,
) -> Result<String, String> {
    let Rhythm {
        measures,
        events,
        preliminary_slots,
    } = rhythm;
    let initial = m.owner.graph.snapshot();
    let nodes = initial["inters"].as_array().unwrap();
    let graph = m.owner.graph.clone();
    let sequences = m
        .measures
        .iter()
        .enumerate()
        .map(|(mi, measure)| crate::music_voice_export::sequence(&graph, &m.voices, measure, mi))
        .collect::<Result<Vec<_>, _>>()?;
    let divisions = crate::music_voice_export::divisions(&graph, &m.measures, &m.chords)?;
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
    for (mi, measure) in measures.iter().enumerate() {
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
        let measure_events = events
            .iter()
            .filter(|e| e["measure_number"] == number)
            .collect::<Vec<_>>();
        crate::music_voice_export::write_measure(
            &graph,
            &mut xml,
            &sequences[mi],
            &measure_events,
            divisions,
        )?;
        xml.push_str("</measure>\n");
        previous_system = Some(measure["system_id"].clone());
    }
    xml.push_str("</part></score-partwise>\n");
    let beam_group_voices = crate::music_voices::beam_groups(&graph, &m.voices, &m.measures)?;
    let voice_projection = crate::music_voices::timed_projection(&m.voices, &m.measures, &graph)?;
    m.owner.graph = graph;
    let mut previous = std::mem::take(&mut m.owner.prepared.report);
    if !image_entry {
        previous = previous["symbols"].take();
    }
    m.owner.prepared.report = json!({"schema":"audiveris-native.musicxml-limited.v1","symbols":previous,"measures":measures,"events":events,"beam_group_voices":beam_group_voices,"voices":voice_projection,"shared_sig":m.owner.graph.snapshot(),"head_template_catalog":m.owner.catalog,"MusicXML_written_from_native_image_recognition":true,"original_XML_output_reused":false,"scope":"first limited clean Bravura44/47/interline11 zero-skew ordinary five-line pitched-native-header-clef/C-major-or-frozen-header-key/recognized constant binary meter ordinary single-Voice sixteenth-eighth-quarter-half/default-whole-measure-rest complete-measure path plus complete parallel quarter/half/one-flag eighth/one-two-level beam Voices in4/4 and actual same-stem uniform multi-head quarter/eighth-beam/FLAG2-sixteenth chords and ordinary unshared first/double augmentation dots; independent two/four-measure quarter-rest and half-note pages with system break, plus two controlled half-rest positions and one independent first-lower-ledger C4 page and one controlled split-quarter-rest input and one independent frozen-header G-major/F# page and one controlled frozen-header 4/4 TimePair variant and one 3/4 six-note G-major input and one fixed COMMON_TIME eight-note G-major input and two bass/alto-clef C-major eight-note inputs verified; one ordinary up-flag eighth/quarter and side/intermediate/sibling/separate-group/hook/same-glyph-competition images and one Bravura47 two-system beam image and one independent two-Voice/two-measure16-quarter image verified; arbitrary real-score admission remains unverified","pending":["broader real-score admission and general MusicXML engraving","other time/clef image comparisons, mid-measure clef changes and unpitched percussion, meter changes and finer/tuplet rhythms/full lateChecks/time columns/clusters above two parts/supported-frozen-cross-kind symbol conflicts/weak optionals","accidentals/other flags/unverified hook configurations/general shared-glyph aliases/cross-staff alien scenarios/more than two levels/shared-rest-repeat-staccato dots/ties/tuplets/general polyphony including gaps/other meters/mixed rests/mixed mirror or special-head chords/pickup/multi-staff","full reduction/cleaner/epilog and original parity","nonzero skew/real scans/special staff/full24shapes/32sources"],"limited_scope_practical_validation_complete":false,"heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
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
    Ok(xml)
}

fn provenance(m: &OwnedMusic) -> [u8; 32] {
    let state = json!({"graph":m.owner.graph.snapshot(),"layout":m.layout(),"voices":crate::music_voices::projection(&m.voices,&m.measures),"timing":m.measures.iter().map(|v|json!({"expected":v.timing.expected,"actual":v.timing.actual,"abnormal":v.timing.abnormal,"excess":v.timing.excess,"slots":crate::music_voices::slots_projection(&v.timing.slots)})).collect::<Vec<_>>(),"report":m.owner.prepared.report});
    Sha256::digest(serde_json::to_vec(&state).unwrap()).into()
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
    fn retained_manual_owner_reprocess_refuses_fine_grid_and_restores_actual_chord() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-manual-chord-cache/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let initial = export.owner.owner.graph.snapshot();
        let nodes = initial["inters"].as_array().unwrap();
        let head = |bounds: &Value| {
            nodes
                .iter()
                .find(|n| n["kind"] == "HeadInter" && n["bounds"] == *bounds)
                .unwrap()
        };
        let h = head(&original["head_bounds"]);
        let target = head(&original["target_head_bounds"]);
        let edges = initial["relations"].as_array().unwrap();
        let stem_edge = |id: &Value| {
            edges
                .iter()
                .find(|e| e["kind"] == "HeadStemRelation" && e["source"] == *id)
                .unwrap()
        };
        let old = stem_edge(&h["id"]);
        let sid = &stem_edge(&target["id"])["target"];
        let hk = serde_json::from_value(h["origin"].clone()).unwrap();
        let sk = serde_json::from_value(
            nodes.iter().find(|n| n["id"] == *sid).unwrap()["origin"].clone(),
        )
        .unwrap();
        let cid = &edges
            .iter()
            .find(|e| e["kind"] == "Containment" && e["target"] == h["id"])
            .unwrap()["source"];
        let ci = export
            .owner
            .chords
            .iter()
            .position(|c| json!(c.id) == *cid)
            .unwrap();
        let mi = export.owner.chords[ci].measure;
        let chord_address = &export.owner.chords[ci] as *const _;
        let before_layout = export.owner.layout();
        let before_rhythm = rhythm_projection(&export.owner, mi);
        let before_xml = export.xml.clone();
        let before_report = export.owner.owner.prepared.report.clone();
        let retained = Rhythm::retained(&export.owner).unwrap();
        let checkpoint = OwnedCheckpoint::capture(&export.owner);
        let mut relation = json!({"kind":"HeadStemRelation","manual":true,"grade":0.7,"head_side":old["head_side"],"extension_point":old["extension_point"]});
        relation["local_relation_id"] = json!(
            edges
                .iter()
                .filter_map(|e| e["local_relation_id"].as_u64())
                .max()
                .unwrap()
                + 1
        );
        let key = export
            .owner
            .apply_manual_head_stem_callback(hk, sk, relation)
            .unwrap();
        assert_eq!(export.owner.chords[ci].stem, Some(sk));
        assert_eq!(
            export
                .owner
                .owner
                .graph
                .chord_duration(key)
                .unwrap()
                .as_deref(),
            original["after"]["duration"].as_str()
        );
        assert_eq!(
            export.owner.measures[mi]
                .timing
                .expected
                .map(format_fraction),
            original["rhythm"]["expected"].as_str().map(str::to_owned)
        );
        let error = process_rhythm(&mut export.owner, Some((&[mi], &retained)))
            .err()
            .unwrap();
        // Original reprocess succeeds with Rational7/64. Native fails at its
        // grid boundary, after callbacks, not at stale Chord ownership checks.
        assert_eq!(
            error,
            "musicxml_dotted_duration_finer_than_sixteenth_pending"
        );
        assert_eq!(export.owner.chords[ci].stem, Some(sk));
        assert_ne!(rhythm_projection(&export.owner, mi), before_rhythm);
        assert_eq!(
            export.owner.measures[mi]
                .timing
                .expected
                .map(format_fraction),
            original["rhythm"]["expected"].as_str().map(str::to_owned)
        );
        checkpoint.restore(&mut export.owner);
        assert_eq!(export.owner.owner.graph.snapshot(), initial);
        assert_eq!(export.owner.layout(), before_layout);
        assert_eq!(rhythm_projection(&export.owner, mi), before_rhythm);
        assert_eq!(export.owner.owner.prepared.report, before_report);
        assert_eq!(export.xml, before_xml);
        assert_eq!(provenance(&export.owner), export.provenance);
        assert_eq!(&export.owner.chords[ci] as *const _, chord_address);
        let returned: Value = serde_json::from_str(include_str!(
            "../fixtures/official-manual-chord-identity/observed.json"
        ))
        .unwrap();
        let retained = Rhythm::retained(&export.owner).unwrap();
        let part_name = export.owner.owner.prepared.report["part_name"]
            .as_str()
            .unwrap()
            .to_owned();
        let old_stem = export.owner.chords[ci].stem.unwrap();
        let base = edges
            .iter()
            .filter_map(|e| e["local_relation_id"].as_u64())
            .max()
            .unwrap();
        for (i, target) in [sk, old_stem].into_iter().enumerate() {
            export.owner.apply_manual_head_stem_callback(hk,target,json!({"kind":"HeadStemRelation","manual":true,"grade":0.7,"head_side":old["head_side"],"extension_point":old["extension_point"],"local_relation_id":base+1+i as u64})).unwrap();
        }
        assert_eq!(export.owner.chords[ci].stem, Some(old_stem));
        let xml = export
            .reprocess_selected(&[mi], retained, &part_name)
            .unwrap();
        assert_eq!(xml, before_xml);
        let actual = rhythm_projection(&export.owner, mi);
        for field in [
            "abnormal",
            "voice_count",
            "stack_slot_count",
            "expected",
            "actual",
            "chords",
        ] {
            assert_eq!(actual[field], returned["rhythm"][field], "{field}");
        }
        let events = export.owner.owner.prepared.report["events"]
            .as_array()
            .unwrap();
        let stream = returned["original_export_stream"].as_array().unwrap();
        assert_eq!(events.len(), stream.len());
        for (a, b) in events.iter().zip(stream) {
            assert_eq!(b["kind"], "note");
            assert_eq!(a["measure_number"], b["measure"]);
            assert_eq!(
                a["voice"].as_u64(),
                b["voice"].as_str().map(|s| s.parse().unwrap())
            );
            for field in ["step", "octave", "type", "dots", "chord"] {
                assert_eq!(a[field], b[field], "{field}");
            }
            assert_eq!(
                rational((a["duration_quarters"].as_f64().unwrap() * 4.) as u32),
                b["duration"].as_str().unwrap()
            );
        }
        assert_eq!(export.owner.chords[ci].stem, Some(old_stem));
        assert_eq!(&export.owner.chords[ci] as *const _, chord_address);
        let current = export.owner.owner.graph.snapshot();
        let chord = current["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == *cid)
            .unwrap();
        assert_eq!(
            chord["retained_chord_cache"]["beams_null"],
            original["after_reprocess"]["cache_after_getters"]["beams_null"]
        );
        let mut empty = crate::shared_sig::EditingSequence::new(vec![]);
        assert_eq!(
            export
                .apply_editing_sequence(&mut empty, crate::shared_sig::EditingOperation::Do)
                .unwrap_err(),
            "music_reprocess_unedited_retained_export_required"
        );
        if let Ok(path) = std::env::var("AUDIVERIS_MANUAL_RETURN_XML") {
            std::fs::write(path, xml).unwrap();
        }
    }
    fn rhythm_projection(owner: &OwnedMusic, mi: usize) -> Value {
        let snapshot = owner.owner.graph.snapshot();
        let measure = &owner.measures[mi];
        let mut chords: Vec<_> = owner.chords.iter().filter(|c| c.measure == mi).collect();
        chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
        json!({"abnormal":measure.timing.abnormal,"voice_count":owner.voices.iter().filter(|v|v.measure==mi).count(),"stack_slot_count":measure.timing.slots.len(),"expected":measure.timing.expected.map(format_fraction),"actual":measure.timing.actual.map(format_fraction),"chords":chords.iter().map(|c|{let n=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==c.id).unwrap();json!({"bounds":n["bounds"],"duration":n["duration"],"time_offset":n["time_offset"],"voice":n["voice"],"slot_id":n["slot_id"]})}).collect::<Vec<_>>()})
    }
    #[test]
    fn mixed_owned_tasks_publish_before_one_impact_and_rollback_automatic_undo() {
        use crate::shared_sig::{DefaultEditingTask, EditingOperation, EditingTask};
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-relation-tasks/mixed.json"
        ))
        .unwrap();
        let case = &oracle["cases"][0];
        let original = &case["steps"][0];
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let initial = export.owner.owner.graph.snapshot();
        let nodes = initial["inters"].as_array().unwrap();
        let chord = nodes
            .iter()
            .find(|n| {
                n["kind"] == "HeadChordInter"
                    && n["bounds"] == case["before"]["chords"][1]["bounds"]
            })
            .unwrap();
        let target = chord["member_ids"][0].as_u64().unwrap() as usize;
        let dot = nodes
            .iter()
            .filter(|n| n["kind"] == "AugmentationDotInter")
            .min_by_key(|n| n["bounds"][0].as_i64())
            .unwrap();
        let id = dot["id"].as_u64().unwrap() as usize;
        let key: Key = serde_json::from_value(dot["origin"].clone()).unwrap();
        let mut bounds: [i64; 4] = serde_json::from_value(dot["bounds"].clone()).unwrap();
        bounds[0] += 3;
        let mut sequence = export.dot_relation_transfer(id, target, true).unwrap();
        sequence.prepend(EditingTask::Inter(
            DefaultEditingTask::new(&export.owner.owner.graph, key, bounds, vec![], vec![])
                .unwrap(),
        ));
        export.owner.owner.graph.enable_dot_callback_trace();
        let actual = export
            .apply_editing_sequence(&mut sequence, EditingOperation::Do)
            .unwrap();
        assert_eq!(
            actual["planned_impact"]["first_impacted_step"],
            original["first_impacted_step"]
        );
        assert_eq!(
            actual["processed_stack_targets"].as_array().unwrap().len(),
            1
        );
        assert_eq!(actual["planned_impact"]["inter_ids"], json!([id, id, id]));
        assert_eq!(actual["populateFRATs_called"], false);
        assert_eq!(actual["controller_impact_complete"], false);
        let state = export.owner.owner.graph.snapshot();
        assert_eq!(
            state["edit_publications"].as_array().unwrap().len(),
            original["publication_count"].as_u64().unwrap() as usize
        );
        let current = state["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap();
        assert_eq!(current["bounds"], original["dot_bounds"]);
        assert!(current["glyph_id"].is_null());
        assert_eq!(
            state["augmentation_dot_ownership"],
            initial["augmentation_dot_ownership"]
        );
        let trace = export.owner.owner.graph.dot_callback_trace();
        assert_eq!(trace.first().unwrap()["event"], "publish");
        assert_eq!(trace.last().unwrap()["event"], "publish");
        let callbacks = trace
            .iter()
            .filter(|e| e["event"] != "publish")
            .collect::<Vec<_>>();
        for (a, b) in callbacks
            .iter()
            .zip(case["callback_trace"].as_array().unwrap())
        {
            for f in ["event", "stage", "depth"] {
                assert_eq!(a[f], b[f]);
            }
            for expected in b["state"]["chords"].as_array().unwrap() {
                let n = a["state"]["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == expected["bounds"])
                    .unwrap();
                for f in ["duration", "time_offset", "voice", "slot_id"] {
                    assert_eq!(n[f], expected[f]);
                }
                assert_eq!(n["dots_number"], expected["dots"]);
            }
        }
        assert_eq!(
            rhythm_projection(&export.owner, 0),
            original["after_rhythm_impact"]["rhythm"]
        );
        let events = export.owner.owner.prepared.report["events"]
            .as_array()
            .unwrap();
        let stream = original["original_export_stream"].as_array().unwrap();
        assert_eq!(events.len(), stream.len());
        for (a, b) in events.iter().zip(stream) {
            for f in [
                "measure_number",
                "step",
                "octave",
                "voice",
                "time_offset",
                "type",
                "dots",
                "chord",
            ] {
                assert_eq!(a[f], b[f]);
            }
            assert_eq!(
                rational((a["duration_quarters"].as_f64().unwrap() * 4.) as u32),
                b["duration"].as_str().unwrap()
            );
        }
        let xml = export.xml.clone();
        let provenance = export.provenance;
        let report = export.owner.owner.prepared.report.clone();
        assert_eq!(
            case["steps"][1]["after_rhythm_impact"]["rhythm"]["actual"],
            "13/16"
        );
        assert!(
            export
                .apply_editing_sequence(&mut sequence, EditingOperation::Undo)
                .unwrap_err()
                .starts_with("musicxml_incomplete_or_excess_measure:")
        );
        assert_eq!(export.owner.owner.graph.snapshot(), state);
        assert_eq!(export.xml, xml);
        assert_eq!(export.provenance, provenance);
        assert_eq!(export.owner.owner.prepared.report, report);
    }
    #[test]
    fn original_relation_tasks_preserve_identity_status_clocks_and_publication() {
        use crate::shared_sig::{EditingOperation, EditingSequence, EditingTask, RelationTask};
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-relation-tasks/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let export = recognize(&image).unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut graph = export.owner.owner.graph.clone();
            let before = graph.snapshot();
            let nodes = before["inters"].as_array().unwrap();
            let chords = case["before"]["chords"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    nodes
                        .iter()
                        .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == c["bounds"])
                        .unwrap()
                        .clone()
                })
                .collect::<Vec<_>>();
            let edge = before["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| {
                    e["kind"] == "AugmentationRelation" && e["target"] == chords[0]["member_ids"][0]
                })
                .unwrap();
            let dot = nodes.iter().find(|n| n["id"] == edge["source"]).unwrap();
            let source: Key = serde_json::from_value(dot["origin"].clone()).unwrap();
            let note = |i: usize| {
                nodes
                    .iter()
                    .find(|n| n["id"] == chords[i]["member_ids"][0])
                    .unwrap()
            };
            if case["old_manual"] == true {
                let mut declared = edge.clone();
                declared["manual"] = json!(true);
                graph
                    .relations(
                        source.system,
                        crate::shared_sig::Phase::Symbol,
                        vec![(
                            source,
                            serde_json::from_value(note(0)["origin"].clone()).unwrap(),
                            declared,
                        )],
                    )
                    .unwrap();
            }
            let mut sequence = EditingSequence::from_tasks(vec![
                EditingTask::Relation(RelationTask::unlink(&graph, edge["id"].as_u64().unwrap() as usize).unwrap()),
                EditingTask::Relation(RelationTask::link(&graph, source,
                    serde_json::from_value(note(1)["origin"].clone()).unwrap(),
                    json!({"kind":"AugmentationRelation","grade":0.7,"manual":case["new_manual"]})).unwrap())]);
            for (op_index, op) in [EditingOperation::Do, EditingOperation::Undo]
                .into_iter()
                .enumerate()
            {
                graph.enable_dot_callback_trace();
                let impact = sequence.perform(&mut graph, op, "PAGE").unwrap();
                assert_eq!(impact["planned_steps"], case["relation_impacted_steps"]);
                assert_eq!(
                    impact["rhythm_stack_targets"].as_array().unwrap().len(),
                    case["impact_stack_count"].as_u64().unwrap() as usize
                );
                let trace = graph.dot_callback_trace();
                assert_eq!(
                    trace.iter().filter(|t| t["event"] == "publish").count(),
                    case["steps"][op_index]["publications"]
                        .as_array()
                        .unwrap()
                        .len()
                );
                assert_eq!(trace.last().unwrap()["event"], "publish");
                let callbacks = trace
                    .iter()
                    .filter(|t| t["event"] != "publish")
                    .collect::<Vec<_>>();
                assert_eq!(callbacks.len(), 4);
                for (a, b) in callbacks
                    .iter()
                    .zip(case["callback_trace"].as_array().unwrap()[op_index * 4..].iter())
                {
                    for field in ["event", "stage", "depth"] {
                        assert_eq!(a[field], b[field]);
                    }
                    assert_eq!(
                        a["state"]["augmentation_dot_ownership"],
                        before["augmentation_dot_ownership"]
                    );
                    for (initial, expected) in
                        chords.iter().zip(b["state"]["chords"].as_array().unwrap())
                    {
                        let current = a["state"]["inters"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|n| n["id"] == initial["id"])
                            .unwrap();
                        assert_eq!(current["dots_number"], expected["dots"]);
                        assert_eq!(current["duration"], expected["duration"]);
                        for f in [
                            "time_offset",
                            "voice",
                            "slot_id",
                            "staff_id",
                            "part_id",
                            "measure_index",
                        ] {
                            assert_eq!(current[f], initial[f]);
                        }
                        if op_index == 0 {
                            let live = crate::music_duration::live(
                                &a["state"],
                                current["id"].as_u64().unwrap() as usize,
                            )
                            .unwrap();
                            assert_eq!(json!(live.intrinsic.map(rational)), expected["duration"]);
                            assert_eq!(
                                rational(
                                    live.intrinsic.unwrap()
                                        + crate::music_time::parse_ticks(
                                            current["time_offset"].as_str().unwrap()
                                        )
                                        .unwrap()
                                ),
                                expected["end_time"].as_str().unwrap()
                            );
                        }
                    }
                }
                let after = graph.snapshot();
                let retained_dot = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == dot["id"])
                    .unwrap();
                for f in ["bounds", "glyph_id", "glyph"] {
                    assert_eq!(retained_dot[f], dot[f]);
                }
                let live_edge = after["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["source"] == dot["id"] && e["kind"] == "AugmentationRelation")
                    .unwrap();
                assert_eq!(
                    live_edge["manual"],
                    if op_index == 0 {
                        case["new_manual"].clone()
                    } else if case["old_manual"] == true {
                        json!(true)
                    } else {
                        edge["manual"].clone()
                    }
                );
                if op_index == 1 {
                    assert_eq!(live_edge["id"], edge["id"]);
                }
            }
        }
    }
    #[test]
    fn original_relation_task_impact_reprocesses_owned_voice_without_promoting_automatic_support() {
        use crate::shared_sig::EditingOperation;
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-relation-tasks/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let before = export.owner.owner.graph.snapshot();
        let edge = before["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "AugmentationRelation")
            .unwrap();
        let dot = edge["source"].as_u64().unwrap() as usize;
        let head = |i: usize| {
            let bounds = &oracle["cases"][0]["before"]["chords"][i]["bounds"];
            before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == *bounds)
                .unwrap()["member_ids"][0]
                .as_u64()
                .unwrap() as usize
        };
        let mut automatic = export.dot_relation_transfer(dot, head(1), false).unwrap();
        assert!(
            export
                .apply_editing_sequence(&mut automatic, EditingOperation::Do)
                .unwrap_err()
                .starts_with("musicxml_incomplete_or_excess_measure:")
        );
        assert_eq!(export.owner.owner.graph.snapshot(), before);
        assert_eq!(
            oracle["cases"][2]["steps"][0]["after_rhythm_impact"]["rhythm"]["actual"],
            "13/16"
        );
        let compare = |e: &Export, case: usize, op: usize| {
            let original = &oracle["cases"][case]["steps"][op];
            assert_eq!(
                rhythm_projection(&e.owner, 0),
                original["after_rhythm_impact"]["rhythm"]
            );
            assert_eq!(
                e.owner.owner.graph.snapshot()["augmentation_dot_ownership"],
                before["augmentation_dot_ownership"]
            );
            let events = e.owner.owner.prepared.report["events"].as_array().unwrap();
            let stream = original["original_export_stream"].as_array().unwrap();
            assert_eq!(events.len(), stream.len());
            for (a, b) in events.iter().zip(stream) {
                for f in [
                    "measure_number",
                    "step",
                    "octave",
                    "voice",
                    "time_offset",
                    "type",
                    "dots",
                    "chord",
                ] {
                    assert_eq!(a[f], b[f]);
                }
                assert_eq!(
                    rational((a["duration_quarters"].as_f64().unwrap() * 4.) as u32),
                    b["duration"].as_str().unwrap()
                );
            }
        };
        let mut sequence = export.dot_relation_transfer(dot, head(1), true).unwrap();
        let impact = export
            .apply_editing_sequence(&mut sequence, EditingOperation::Do)
            .unwrap();
        assert_eq!(impact["populateFRATs_called"], false);
        compare(&export, 0, 0);
        let xml = export.xml.clone();
        let graph = export.owner.owner.graph.snapshot();
        assert!(
            export
                .apply_editing_sequence(&mut sequence, EditingOperation::Undo)
                .unwrap_err()
                .starts_with("musicxml_incomplete_or_excess_measure:")
        );
        assert_eq!(
            oracle["cases"][0]["steps"][1]["after_rhythm_impact"]["rhythm"]["actual"],
            "13/16"
        );
        assert_eq!(export.owner.owner.graph.snapshot(), graph);
        assert_eq!(export.xml, xml);
        // Only a genuinely applied manual relation becomes the old relation.
        let mut back = export.dot_relation_transfer(dot, head(0), true).unwrap();
        export
            .apply_editing_sequence(&mut back, EditingOperation::Do)
            .unwrap();
        let mut manual = export.dot_relation_transfer(dot, head(1), true).unwrap();
        for (i, op) in [
            EditingOperation::Do,
            EditingOperation::Undo,
            EditingOperation::Redo,
        ]
        .into_iter()
        .enumerate()
        {
            export.apply_editing_sequence(&mut manual, op).unwrap();
            compare(&export, 1, i);
        }
    }
    #[test]
    fn original_timed_dot_callbacks_keep_owned_clocks_and_derive_live_duration() {
        use crate::shared_sig::{DefaultEditingTask, EditingLink, EditingOperation};
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-timed-dot-support/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let export = recognize(&image).unwrap();
        let rest_image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/half-rest-augmentation.png"
        ))
        .unwrap();
        let mut rest = recognize(&rest_image).unwrap();
        let saved = rest.owner.owner.graph.snapshot();
        let edge = saved["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "AugmentationRelation")
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        assert_eq!(
            rest.owner
                .owner
                .graph
                .remove_augmentation_edge(edge)
                .unwrap_err(),
            "music_dot_timed_special_or_multihead_callback_pending"
        );
        assert_eq!(rest.owner.owner.graph.snapshot(), saved);
        for case in oracle["cases"].as_array().unwrap() {
            let mut graph = export.owner.owner.graph.clone();
            let before = graph.snapshot();
            let nodes = before["inters"].as_array().unwrap();
            let selected = case["before"]["chords"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    nodes
                        .iter()
                        .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == c["bounds"])
                        .unwrap()
                        .clone()
                })
                .collect::<Vec<_>>();
            let first = before["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| {
                    e["kind"] == "AugmentationRelation"
                        && selected[0]["member_ids"]
                            .as_array()
                            .unwrap()
                            .contains(&e["target"])
                })
                .unwrap();
            let dot = nodes.iter().find(|n| n["id"] == first["source"]).unwrap();
            let key: Key = serde_json::from_value(dot["origin"].clone()).unwrap();
            let target = nodes
                .iter()
                .find(|n| n["id"] == selected[1]["member_ids"][0])
                .unwrap();
            let target_key: Key = serde_json::from_value(target["origin"].clone()).unwrap();
            if case["original_support_manual"] == true {
                let note = nodes.iter().find(|n| n["id"] == first["target"]).unwrap();
                let mut edge = first.clone();
                edge["manual"] = json!(true);
                graph
                    .relations(
                        key.system,
                        crate::shared_sig::Phase::Symbol,
                        vec![(
                            key,
                            serde_json::from_value(note["origin"].clone()).unwrap(),
                            edge,
                        )],
                    )
                    .unwrap();
            }
            let unlink =
                EditingLink::from_registered(&graph, key, first["id"].as_u64().unwrap() as usize)
                    .unwrap();
            let link = EditingLink::new(
                target_key,
                true,
                json!({"kind":"AugmentationRelation","grade":0.7,"manual":true}),
            );
            let mut task = DefaultEditingTask::new(
                &graph,
                key,
                serde_json::from_value(dot["bounds"].clone()).unwrap(),
                vec![link],
                vec![unlink],
            )
            .unwrap();
            let ownership = graph.snapshot()["augmentation_dot_ownership"].clone();
            graph.enable_dot_callback_trace();
            task.perform(&mut graph, EditingOperation::Do).unwrap();
            let traces = graph
                .dot_callback_trace()
                .iter()
                .filter(|t| t["event"] != "publish")
                .collect::<Vec<_>>();
            assert_eq!(traces.len(), 4);
            for (actual, original) in traces.iter().zip(case["trace"].as_array().unwrap()) {
                assert_eq!(actual["event"], original["event"]);
                assert_eq!(actual["stage"], original["stage"]);
                assert_eq!(actual["depth"], original["depth"]);
                assert_eq!(actual["state"]["augmentation_dot_ownership"], ownership);
                for (source, expected) in selected
                    .iter()
                    .zip(original["state"]["chords"].as_array().unwrap())
                {
                    let node = actual["state"]["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == source["id"])
                        .unwrap();
                    let live = crate::music_duration::live(
                        &actual["state"],
                        node["id"].as_u64().unwrap() as usize,
                    )
                    .unwrap();
                    assert_eq!(json!(live.intrinsic.map(rational)), expected["duration"]);
                    assert_eq!(node["duration"], expected["duration"]);
                    let end = live.intrinsic.unwrap()
                        + crate::music_time::parse_ticks(node["time_offset"].as_str().unwrap())
                            .unwrap();
                    assert_eq!(rational(end), expected["end_time"].as_str().unwrap());
                    for f in ["time_offset", "voice", "slot_id"] {
                        assert_eq!(node[f], expected[f]);
                        assert_eq!(node[f], source[f]);
                    }
                    for f in ["part_id", "measure_index", "staff_id"] {
                        assert_eq!(node[f], source[f]);
                    }
                    assert_eq!(node["dots_number"], expected["dots"]);
                    assert_eq!(expected["same_voice"], true);
                    assert_eq!(expected["same_measure"], true);
                    assert_eq!(expected["same_part"], true);
                }
            }
            let mut stale = graph.clone();
            let selected_key: Key = serde_json::from_value(selected[1]["origin"].clone()).unwrap();
            let mut projected = stale.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == selected[1]["id"])
                .unwrap()
                .clone();
            projected["duration"] = json!("99");
            stale.vertex(selected_key, projected, false).unwrap();
            assert_eq!(
                stale.chord_duration(selected_key).unwrap(),
                Some("7/16".into())
            );
            assert_eq!(
                stale.chord_end_time(selected_key).unwrap(),
                Some("15/16".into())
            );
            // Undo is observed before any rhythm reset: automatic re-add does
            // not count dots, whereas manual re-add does. Neither changes clocks.
            graph.enable_dot_callback_trace();
            task.perform(&mut graph, EditingOperation::Undo).unwrap();
            let expected = if case["original_support_manual"] == true {
                2
            } else {
                0
            };
            let selected_key: Key = serde_json::from_value(selected[0]["origin"].clone()).unwrap();
            assert_eq!(
                graph.chord_duration(selected_key).unwrap(),
                Some(if expected == 2 { "7/16" } else { "1/4" }.into())
            );
            assert_eq!(
                graph.chord_end_time(selected_key).unwrap(),
                graph.chord_duration(selected_key).unwrap()
            );
            let after = graph.snapshot();
            for source in &selected {
                let node = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == source["id"])
                    .unwrap();
                for f in [
                    "time_offset",
                    "voice",
                    "slot_id",
                    "part_id",
                    "measure_index",
                    "staff_id",
                ] {
                    assert_eq!(node[f], source[f]);
                }
            }
            assert_eq!(after["augmentation_dot_ownership"], ownership);
        }
    }
    #[test]
    fn original_timed_support_transfer_reprocesses_owned_stack_and_rolls_back_incomplete_undo() {
        use crate::shared_sig::{
            DefaultEditingTask, EditingLink, EditingOperation, EditingSequence,
        };
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-timed-support-reprocess/observed.json"
        ))
        .unwrap();
        let callback: Value = serde_json::from_str(include_str!(
            "../fixtures/official-timed-dot-support/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let before_xml = export.xml.clone();
        let before = export.owner.owner.graph.snapshot();
        let first = &callback["cases"][0]["before"]["chords"][0];
        let second = &callback["cases"][0]["before"]["chords"][1];
        let nodes = before["inters"].as_array().unwrap();
        let chord = |bounds: &Value| {
            nodes
                .iter()
                .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == *bounds)
                .unwrap()
        };
        let source = chord(&first["bounds"])["member_ids"][0].as_u64().unwrap() as usize;
        let target = chord(&second["bounds"])["member_ids"][0].as_u64().unwrap() as usize;
        let dot = before["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "AugmentationRelation" && e["target"] == source)
            .unwrap()["source"]
            .as_u64()
            .unwrap() as usize;
        let other_measure = rhythm_projection(&export.owner, 1);
        let compare = |export: &Export, case: usize, op: usize| {
            let original = &oracle["cases"][case]["steps"][op];
            assert_eq!(
                rhythm_projection(&export.owner, 0),
                callback["cases"][case]["steps"][op]["after_reprocess"]["rhythm"]
            );
            assert_eq!(rhythm_projection(&export.owner, 1), other_measure);
            assert_eq!(
                export.owner.owner.graph.snapshot()["augmentation_dot_ownership"],
                before["augmentation_dot_ownership"]
            );
            let events = export.owner.owner.prepared.report["events"]
                .as_array()
                .unwrap();
            let original_events = original["original_export_stream"].as_array().unwrap();
            assert_eq!(events.len(), original_events.len());
            for (a, b) in events.iter().zip(original_events) {
                for field in [
                    "measure_number",
                    "step",
                    "octave",
                    "voice",
                    "time_offset",
                    "type",
                    "dots",
                    "chord",
                ] {
                    assert_eq!(a[field], b[field]);
                }
                assert_eq!(
                    rational((a["duration_quarters"].as_f64().unwrap() * 4.) as u32),
                    b["duration"].as_str().unwrap()
                );
            }
        };
        let mut sequence = export.dot_support_transfer(dot, target).unwrap();
        export.owner.owner.graph.enable_dot_callback_trace();
        let actual = export
            .apply_editing_sequence(&mut sequence, EditingOperation::Do)
            .unwrap();
        assert_eq!(actual["populateFRATs_called"], false);
        assert_eq!(actual["LINKS_called"], false);
        assert_eq!(
            actual["processed_stack_targets"].as_array().unwrap().len(),
            1
        );
        compare(&export, 0, 0);
        let trace = export
            .owner
            .owner
            .graph
            .dot_callback_trace()
            .iter()
            .filter(|t| t["event"] != "publish")
            .collect::<Vec<_>>();
        assert_eq!(trace.len(), 4);
        for (a, b) in trace
            .iter()
            .zip(callback["cases"][0]["trace"].as_array().unwrap())
        {
            assert_eq!(a["event"], b["event"]);
            assert_eq!(a["stage"], b["stage"]);
            assert_eq!(
                a["state"]["augmentation_dot_ownership"],
                before["augmentation_dot_ownership"]
            );
            for original in b["state"]["chords"].as_array().unwrap() {
                let n = a["state"]["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == original["bounds"])
                    .unwrap();
                for f in ["duration", "time_offset", "voice", "slot_id"] {
                    assert_eq!(n[f], original[f]);
                }
                assert_eq!(n["dots_number"], original["dots"]);
            }
        }
        let shifted_xml = export.xml.clone();
        assert_ne!(shifted_xml, before_xml);
        // Original automatic re-add leaves dotsNumber0 and actual13/16.
        // Native complete-measure export remains guarded and rolls back owners,
        // report, XML, provenance and the retained task, rather than inventing dots.
        assert_eq!(
            callback["cases"][0]["steps"][1]["after_reprocess"]["rhythm"]["actual"],
            "13/16"
        );
        let saved_provenance = export.provenance;
        let saved_report = export.owner.owner.prepared.report.clone();
        let saved_graph = export.owner.owner.graph.snapshot();
        let saved_voice =
            crate::music_voices::projection(&export.owner.voices, &export.owner.measures);
        let saved_sequence = format!("{sequence:?}");
        let error = export
            .apply_editing_sequence(&mut sequence, EditingOperation::Undo)
            .unwrap_err();
        assert!(
            error.starts_with("musicxml_incomplete_or_excess_measure:"),
            "{error}"
        );
        assert_eq!(export.owner.owner.graph.snapshot(), saved_graph);
        assert_eq!(export.owner.owner.prepared.report, saved_report);
        assert_eq!(
            crate::music_voices::projection(&export.owner.voices, &export.owner.measures),
            saved_voice
        );
        assert_eq!(export.provenance, saved_provenance);
        assert_eq!(format!("{sequence:?}"), saved_sequence);
        assert_eq!(export.xml, shifted_xml);
        compare(&export, 0, 0);
        // The next transfer unlinks the genuinely manual support created by Do.
        // Both directions can now recount under original callback conditions;
        // no setter, forged report or test-only provenance reset is involved.
        let mut manual = export.dot_support_transfer(dot, source).unwrap();
        export
            .apply_editing_sequence(&mut manual, EditingOperation::Do)
            .unwrap();
        compare(&export, 1, 1);
        assert_eq!(export.xml, before_xml);
        export
            .apply_editing_sequence(&mut manual, EditingOperation::Undo)
            .unwrap();
        compare(&export, 1, 0);
        assert_eq!(export.xml, shifted_xml);
        export
            .apply_editing_sequence(&mut manual, EditingOperation::Redo)
            .unwrap();
        compare(&export, 1, 1);
        assert_eq!(export.xml, before_xml);
        let now = export.owner.owner.graph.snapshot();
        let other = export
            .owner
            .chords
            .iter()
            .find(|c| c.measure == 1)
            .unwrap()
            .members[0]["id"]
            .as_u64()
            .unwrap() as usize;
        assert_eq!(
            export.dot_support_transfer(dot, other).unwrap_err(),
            "music_reprocess_timed_support_edit_pending"
        );
        let dot_node = now["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == dot)
            .unwrap();
        let partner = now["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == target)
            .unwrap();
        let key = serde_json::from_value(dot_node["origin"].clone()).unwrap();
        let link = EditingLink::new(
            serde_json::from_value(partner["origin"].clone()).unwrap(),
            true,
            json!({"kind":"AugmentationRelation","manual":true}),
        );
        let task = DefaultEditingTask::new(
            &export.owner.owner.graph,
            key,
            serde_json::from_value(dot_node["bounds"].clone()).unwrap(),
            vec![link],
            vec![],
        )
        .unwrap();
        assert_eq!(
            export
                .apply_editing_sequence(&mut EditingSequence::new(vec![task]), EditingOperation::Do)
                .unwrap_err(),
            "music_reprocess_shared_dot_support_edit_pending"
        );
        assert_eq!(export.owner.owner.graph.snapshot(), now);
    }
    #[test]
    fn original_bounds_edit_reprocesses_retained_stack_and_undo_without_frat_repopulation() {
        use crate::shared_sig::{
            DefaultEditingTask, EditingLink, EditingOperation, EditingSequence,
        };
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let before_xml = export.xml.clone();
        let before = export.owner.owner.graph.snapshot();
        let ownership = before["augmentation_dot_ownership"].clone();
        let dot = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "AugmentationDotInter")
            .min_by_key(|n| n["bounds"][0].as_i64())
            .unwrap();
        let key: Key = serde_json::from_value(dot["origin"].clone()).unwrap();
        let id = dot["id"].as_u64().unwrap();
        let gid = dot["glyph_id"].as_u64().unwrap() as usize;
        let physical = crate::beam_dispatch::glyph_json(
            &export.owner.owner.prepared.registry.get(gid).unwrap().glyph,
        );
        let mut bounds: [i64; 4] = serde_json::from_value(dot["bounds"].clone()).unwrap();
        bounds[0] += 3;
        let task = DefaultEditingTask::new(&export.owner.owner.graph, key, bounds, vec![], vec![])
            .unwrap();
        let mut sequence = EditingSequence::new(vec![task]);
        let actual = export
            .apply_editing_sequence(&mut sequence, EditingOperation::Do)
            .unwrap();
        assert_eq!(
            actual["processed_stack_targets"].as_array().unwrap().len(),
            1
        );
        assert_eq!(actual["populateFRATs_called"], false);
        assert_eq!(actual["LINKS_called"], false);
        assert_eq!(
            actual["planned_impact"]["planned_steps"],
            json!(["RHYTHMS", "PAGE"])
        );
        assert_eq!(actual["planned_impact"]["reprocessed_steps"], json!([]));
        assert_eq!(actual["controller_impact_complete"], false);
        assert_eq!(
            rhythm_projection(&export.owner, 0),
            original["after_actual_reprocessStack"]["rhythm"]
        );
        assert_eq!(
            rhythm_projection(&export.owner, 1),
            original["after_actual_reprocessStack"]["other_measure"]
        );
        let after = export.owner.owner.graph.snapshot();
        assert_eq!(after["augmentation_dot_ownership"], ownership);
        assert_eq!(after["relations"], before["relations"]);
        assert_eq!(export.xml, before_xml);
        assert_eq!(
            crate::beam_dispatch::glyph_json(
                &export.owner.owner.prepared.registry.get(gid).unwrap().glyph
            ),
            physical
        );
        let node = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap();
        assert_eq!(node["bounds"], json!(bounds));
        assert!(node["glyph_id"].is_null());
        export
            .apply_editing_sequence(&mut sequence, EditingOperation::Undo)
            .unwrap();
        assert_eq!(export.xml, before_xml);
        assert_eq!(
            rhythm_projection(&export.owner, 0),
            original["before"]["rhythm"]
        );
        let undone = export.owner.owner.graph.snapshot();
        let edge = undone["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["source"] == id && e["kind"] == "AugmentationRelation")
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        let unlink = EditingLink::from_registered(&export.owner.owner.graph, key, edge).unwrap();
        let task =
            DefaultEditingTask::new(&export.owner.owner.graph, key, bounds, vec![], vec![unlink])
                .unwrap();
        let mut unsupported = EditingSequence::new(vec![task]);
        let error = export
            .apply_editing_sequence(&mut unsupported, EditingOperation::Do)
            .unwrap_err();
        assert!(
            error.starts_with("musicxml_incomplete_or_excess_measure:"),
            "{error}"
        );
        assert_eq!(export.owner.owner.graph.snapshot(), undone);
        // A report cannot grant provenance to an already edited owner.
        export
            .owner
            .owner
            .graph
            .edit_note_owner(key, crate::shared_sig::NoteOwnerEdit::Bounds(bounds))
            .unwrap();
        export.owner.owner.prepared.report["shared_sig"] = export.owner.owner.graph.snapshot();
        assert_eq!(
            export
                .apply_editing_sequence(&mut sequence, EditingOperation::Redo)
                .unwrap_err(),
            "music_reprocess_unedited_retained_export_required"
        );
        assert_eq!(export.xml, before_xml);
    }
    #[test]
    fn original_voice_tie_beam_next_preference_keep_owned_clocks_and_export_binding() {
        use crate::shared_sig::{Key, Phase};
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/voice-tie-beam.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-parallel-voices/parallel-beam.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let m = &mut export.owner;
        let first_chord = |m: &OwnedMusic, bounds: &Value| {
            m.chords
                .iter()
                .find(|c| json!(c.bounds) == *bounds)
                .map(|c| (c.id, c.key))
                .unwrap()
        };
        let (source_id, source) = first_chord(m, &original["tie_source"]["bounds"]);
        let (target_id, target) = first_chord(m, &original["target"]["bounds"]);
        let projection = |m: &OwnedMusic| {
            json!(m.voices.iter().filter(|v|v.measure==1).map(|v|json!({"id":v.id,"first_chord_bounds":m.chords.iter().find(|c|c.id==v.chord_ids[0]).unwrap().bounds})).collect::<Vec<_>>())
        };
        assert_eq!(projection(m), original["before"]);
        let before = m.owner.graph.snapshot();
        let note_key = |m: &OwnedMusic, id| {
            let member = m
                .chords
                .iter()
                .find(|c| c.id == id)
                .unwrap()
                .members
                .first()
                .unwrap();
            serde_json::from_value::<Key>(member["origin"].clone()).unwrap()
        };
        let left = note_key(m, source_id);
        let right = note_key(m, target_id);
        let slur = Key {
            system: 1,
            phase: Phase::Symbol,
            local: before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["origin"]["phase"] == "SYMBOL")
                .filter_map(|n| n["origin"]["local"].as_u64())
                .max()
                .unwrap_or(0) as usize
                + 1,
        };
        m.owner
            .graph
            .vertex(
                slur,
                json!({"kind":"SlurInter","is_tie":true,"grade":0.8}),
                false,
            )
            .unwrap();
        m.owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![
                    (slur, left, json!({"kind":"SlurHeadRelation","side":"LEFT"})),
                    (
                        slur,
                        right,
                        json!({"kind":"SlurHeadRelation","side":"RIGHT"}),
                    ),
                ],
            )
            .unwrap();
        crate::music_voices::refine_ordinary_system(
            &mut m.owner.graph,
            &mut m.voices,
            &m.measures,
            1,
        )
        .unwrap();
        assert_eq!(projection(m), original["after_tie"]);
        let beam = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "BeamInter" && n["bounds"] == original["beam_bounds"])
            .unwrap();
        let stem = before["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "ChordStemRelation" && e["source"] == target_id)
            .unwrap()["target"]
            .as_u64()
            .unwrap();
        let stem = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == stem)
            .unwrap();
        m.owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![(
                    serde_json::from_value(beam["origin"].clone()).unwrap(),
                    serde_json::from_value(stem["origin"].clone()).unwrap(),
                    json!({"kind":"BeamStemRelation","beam_portion":"LEFT"}),
                )],
            )
            .unwrap();
        let groups = crate::music_voices::beam_group_chords(&m.owner.graph).unwrap();
        let group = groups
            .as_array()
            .unwrap()
            .iter()
            .find(|g| {
                g["chords"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["id"] == target_id)
            })
            .unwrap();
        let chords = group["chords"].as_array().unwrap();
        let at = chords.iter().position(|c| c["id"] == target_id).unwrap();
        assert_eq!(
            chords[at - 1]["bounds"],
            original["beam_predecessor"]["bounds"]
        );
        crate::music_voices::refine_ordinary_system(
            &mut m.owner.graph,
            &mut m.voices,
            &m.measures,
            1,
        )
        .unwrap();
        assert_eq!(projection(m), original["after_beam"]);
        m.owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![(source, target, json!({"kind":"NextInVoiceRelation"}))],
            )
            .unwrap();
        crate::music_voices::refine_ordinary_system(
            &mut m.owner.graph,
            &mut m.voices,
            &m.measures,
            1,
        )
        .unwrap();
        assert_eq!(projection(m), original["after_next"]);
        let mut events = m.owner.prepared.report["events"]
            .as_array()
            .unwrap()
            .clone();
        for event in &mut events {
            let mi = event["measure_number"].as_u64().unwrap() as usize - 1;
            crate::music_voice_export::bind_event(
                &m.owner.graph,
                &m.voices,
                &m.measures[mi],
                mi,
                event,
            )
            .unwrap();
            let chord = m
                .chords
                .iter()
                .find(|c| c.id == event["chord_id"].as_u64().unwrap() as usize)
                .unwrap();
            let voice = m
                .voices
                .iter()
                .find(|v| v.measure == mi && v.chord_ids.contains(&chord.id))
                .unwrap();
            assert_eq!(event["voice"], voice.id);
        }
        let steps =
            crate::music_voice_export::sequence(&m.owner.graph, &m.voices, &m.measures[1], 1)
                .unwrap();
        assert!(
            matches!(steps.first(),Some(crate::music_voice_export::Step::Chord{id,voice:2,..}) if *id==target_id)
        );
        let snapshot = m.owner.graph.snapshot();
        let mut chord = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == target_id)
            .unwrap()
            .clone();
        chord["preferred_voice_id"] = json!(1);
        m.owner.graph.vertex(target, chord, false).unwrap();
        crate::music_voices::refine_ordinary_system(
            &mut m.owner.graph,
            &mut m.voices,
            &m.measures,
            1,
        )
        .unwrap();
        assert_eq!(projection(m), original["after_preferred"]);
        let after = m.owner.graph.snapshot();
        let source_node = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == source_id)
            .unwrap();
        assert_eq!(source_node["voice"], original["tie_source"]["voice_id"]);
        for node in after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["origin"]["phase"] == "CHORD")
        {
            let old = before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == node["id"])
                .unwrap();
            for field in ["duration", "time_offset", "slot_id", "bounds", "member_ids"] {
                assert_eq!(node[field], old[field]);
            }
        }
        let failures: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/voice-null-predecessors.json"
        ))
        .unwrap();
        let snapshot = m.owner.graph.snapshot();
        let mut predecessor = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| {
                n["kind"] == "HeadChordInter"
                    && n["bounds"] == failures["controls"][1]["source_bounds"]
            })
            .unwrap()
            .clone();
        predecessor["voice"] = Value::Null;
        let key = serde_json::from_value(predecessor["origin"].clone()).unwrap();
        m.owner.graph.vertex(key, predecessor, false).unwrap();
        let before_failure = m.owner.graph.snapshot();
        let before_voices = crate::music_voices::projection(&m.voices, &m.measures);
        assert_eq!(
            crate::music_voices::refine_ordinary_system(
                &mut m.owner.graph,
                &mut m.voices,
                &m.measures,
                1
            )
            .unwrap_err(),
            failures["controls"][1]["native_error"].as_str().unwrap()
        );
        assert_eq!(m.owner.graph.snapshot(), before_failure);
        assert_eq!(
            crate::music_voices::projection(&m.voices, &m.measures),
            before_voices
        );
    }
    #[test]
    fn original_null_chord_voice_stays_null_during_refinement_and_id_collision() {
        use crate::shared_sig::{Key, Phase};
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/voice-null-reference.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-parallel-voices/parallel-quarters.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let m = &mut export.owner;
        let chord = |m: &OwnedMusic, field: &str| {
            let c = m
                .chords
                .iter()
                .find(|c| json!(c.bounds) == original[field])
                .unwrap();
            (c.id, c.key)
        };
        let (source, source_key) = chord(m, "source_bounds");
        let (target, target_key) = chord(m, "target_bounds");
        let (fallback, _) = chord(m, "fallback_bounds");
        let (swap_null, swap_null_key) = chord(m, "swap_null_bounds");
        let add_tie = |m: &mut OwnedMusic, from| {
            let snapshot = m.owner.graph.snapshot();
            let key = Key {
                system: 1,
                phase: Phase::Symbol,
                local: snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| n["origin"]["phase"] == "SYMBOL")
                    .filter_map(|n| n["origin"]["local"].as_u64())
                    .max()
                    .unwrap_or(0) as usize
                    + 1,
            };
            let note = |id| {
                serde_json::from_value(
                    m.chords.iter().find(|c| c.id == id).unwrap().members[0]["origin"].clone(),
                )
                .unwrap()
            };
            let left = note(from);
            let right = note(target);
            m.owner
                .graph
                .vertex(
                    key,
                    json!({"kind":"SlurInter","is_tie":true,"grade":0.8}),
                    false,
                )
                .unwrap();
            m.owner
                .graph
                .relations(
                    1,
                    Phase::Symbol,
                    vec![
                        (key, left, json!({"kind":"SlurHeadRelation","side":"LEFT"})),
                        (
                            key,
                            right,
                            json!({"kind":"SlurHeadRelation","side":"RIGHT"}),
                        ),
                    ],
                )
                .unwrap();
        };
        let edit = |m: &mut OwnedMusic, key: Key, field: &str, value: Value| {
            let snapshot = m.owner.graph.snapshot();
            let mut node = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["origin"] == json!(key))
                .unwrap()
                .clone();
            node[field] = value.clone();
            m.owner.graph.vertex(key, node, false).unwrap();
            if field == "voice"
                && let Some(number) = value.as_u64()
            {
                let chord = m.chords.iter().find(|c| c.key == key).unwrap();
                if let Some(voice) = m
                    .voices
                    .iter()
                    .find(|v| v.measure == chord.measure && v.id == number as usize)
                {
                    // Original justAssignVoice restores a reference, not a list entry.
                    m.owner.graph.bind_chord_to_live_voice(key, voice).unwrap();
                }
            }
        };
        let projection = |m: &OwnedMusic| {
            json!(m.voices.iter().filter(|v|v.measure==1).map(|v|json!({"id":v.id,"first_chord_bounds":m.chords.iter().find(|c|c.id==v.chord_ids[0]).unwrap().bounds})).collect::<Vec<_>>())
        };
        let refine = |m: &mut OwnedMusic| {
            crate::music_voices::refine_ordinary_system(
                &mut m.owner.graph,
                &mut m.voices,
                &m.measures,
                1,
            )
            .unwrap()
        };
        let tied = |m: &OwnedMusic| {
            let snapshot = m.owner.graph.snapshot();
            m.owner
                .graph
                .incoming_tied_voice_id(target, |id| {
                    Ok(snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == id)
                        .unwrap()["voice"]
                        .as_u64()
                        .map(|id| id as usize))
                })
                .unwrap()
        };
        let before = m.owner.graph.snapshot();
        let members: Vec<_> = m.voices.iter().map(|v| v.chord_ids.clone()).collect();
        add_tie(m, source);
        assert_eq!(json!(tied(m)), original["initial_tied_id"]);
        refine(m);
        assert_eq!(projection(m), original["after_tie"]);
        edit(m, source_key, "voice", Value::Null);
        assert_eq!(json!(tied(m)), original["null_tied_id"]);
        refine(m);
        assert_eq!(projection(m), original["after_null"]);
        assert_eq!(json!(tied(m)), original["null_tied_id"]);
        add_tie(m, fallback);
        assert_eq!(json!(tied(m)), original["fallback_tied_id"]);
        refine(m);
        assert_eq!(projection(m), original["after_fallback"]);
        let source_voice = m
            .voices
            .iter()
            .find(|v| v.chord_ids.contains(&source))
            .unwrap()
            .id;
        edit(m, source_key, "voice", json!(source_voice));
        assert_eq!(json!(tied(m)), original["restored_tied_id"]);
        refine(m);
        assert_eq!(projection(m), original["after_restored"]);
        edit(m, swap_null_key, "voice", Value::Null);
        edit(m, target_key, "preferred_voice_id", json!(1));
        refine(m);
        assert_eq!(projection(m), original["after_preferred_swap"]);
        let after = m.owner.graph.snapshot();
        assert_eq!(
            after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == swap_null)
                .unwrap()["voice"],
            original["null_reference_after_preferred_swap"]
        );
        assert_eq!(
            m.voices
                .iter()
                .map(|v| v.chord_ids.clone())
                .collect::<Vec<_>>(),
            members
        );
        for node in after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["origin"]["phase"] == "CHORD")
        {
            let old = before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == node["id"])
                .unwrap();
            for field in [
                "duration",
                "time_offset",
                "slot_id",
                "bounds",
                "member_ids",
                "part_id",
                "measure_index",
            ] {
                assert_eq!(node[field], old[field]);
            }
        }
        let first_null: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/voice-first-chord-null.json"
        ))
        .unwrap();
        edit(m, target_key, "voice", Value::Null);
        edit(m, target_key, "preferred_voice_id", Value::Null);
        assert_eq!(json!(tied(m)), first_null["first_null_tied_id"]);
        refine(m);
        assert_eq!(projection(m), first_null["after_first_null"]);
        let first_snapshot = m.owner.graph.snapshot();
        assert_eq!(
            first_snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == target)
                .unwrap()["voice"],
            first_null["first_chord_voice_after_refinement"]
        );
        assert!(
            m.voices
                .iter()
                .any(|v| v.measure == 1 && v.chord_ids.first() == Some(&target))
        );
        let mut event = m.owner.prepared.report["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["chord_id"] == target)
            .unwrap()
            .clone();
        let unbound = event.clone();
        assert_eq!(
            crate::music_voice_export::bind_event(
                &m.owner.graph,
                &m.voices,
                &m.measures[1],
                1,
                &mut event
            )
            .unwrap_err(),
            "musicxml_voice_owned_chord_required"
        );
        assert_eq!(event, unbound);
        let actual_voice = m
            .voices
            .iter()
            .find(|v| v.chord_ids.contains(&target))
            .unwrap()
            .id;
        edit(m, target_key, "voice", json!(actual_voice));
        edit(m, target_key, "preferred_voice_id", json!(1));
        refine(m);
        assert_eq!(projection(m), original["after_preferred_swap"]);
        let failures: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/voice-null-predecessors.json"
        ))
        .unwrap();
        edit(m, source_key, "voice", Value::Null);
        m.owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![(
                    source_key,
                    target_key,
                    json!({"kind":"NextInVoiceRelation"}),
                )],
            )
            .unwrap();
        let before_failure = m.owner.graph.snapshot();
        let before_voices = crate::music_voices::projection(&m.voices, &m.measures);
        assert_eq!(
            crate::music_voices::refine_ordinary_system(
                &mut m.owner.graph,
                &mut m.voices,
                &m.measures,
                1
            )
            .unwrap_err(),
            failures["controls"][0]["native_error"].as_str().unwrap()
        );
        assert_eq!(m.owner.graph.snapshot(), before_failure);
        assert_eq!(
            crate::music_voices::projection(&m.voices, &m.measures),
            before_voices
        );
    }
    #[test]
    fn original_voice_refinement_purges_actual_empty_owner_and_renames_rhythm_created_high() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/voice-refine.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let m = &mut export.owner;
        let first = m.voices.iter().position(|v| v.measure == 0).unwrap();
        // Original setup renames the actual Voice, retaining Chord references.
        crate::music_voices::swap_voice_id(&mut m.owner.graph, &mut m.voices, first, 7).unwrap();
        let mut empty = m.voices[first].clone();
        empty.identity = std::sync::Arc::new(());
        empty.id = 99;
        empty.chord_ids.clear();
        m.voices.push(empty);
        crate::music_voices::refine_ordinary_system(
            &mut m.owner.graph,
            &mut m.voices,
            &m.measures,
            1,
        )
        .unwrap();
        assert_eq!(rhythm_projection(m, 0), original["rhythm"]);
        assert_eq!(json!(m.voices.iter().filter(|v|v.measure==0).map(|v|json!({"id":v.id,"first_chord_bounds":m.chords.iter().find(|c|c.id==v.chord_ids[0]).unwrap().bounds})).collect::<Vec<_>>()),original["after"]);
    }
    #[test]
    fn original_stack_reset_preserves_owned_voices_and_expected_duration_before_measure_reset() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-reprocess/observed.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = recognize(&image).unwrap();
        let owner = &mut export.owner;
        let expected = owner.measures[0].timing.expected;
        let before = owner.owner.graph.snapshot();
        let voices = crate::music_voices::projection(&owner.voices, &owner.measures);
        crate::music_voices::reset_stack_rhythm(&mut owner.measures[0], expected);
        let snapshot = owner.owner.graph.snapshot();
        assert_eq!(snapshot, before);
        assert_eq!(
            crate::music_voices::projection(&owner.voices, &owner.measures),
            voices
        );
        let measure = &owner.measures[0];
        let mut chords: Vec<_> = owner.chords.iter().filter(|c| c.measure == 0).collect();
        chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
        let actual = json!({"abnormal":measure.timing.abnormal,"voice_count":owner.voices.iter().filter(|v|v.measure==0).count(),"stack_slot_count":measure.timing.slots.len(),"expected":measure.timing.expected.map(format_fraction),"actual":measure.timing.actual.map(format_fraction),"chords":chords.iter().map(|c|{let n=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==c.id).unwrap();json!({"bounds":n["bounds"],"duration":n["duration"],"time_offset":n["time_offset"],"voice":n["voice"],"slot_id":n["slot_id"]})}).collect::<Vec<_>>()});
        assert_eq!(actual, original["stages"][0]["state"]["rhythm"]);
        assert_eq!(
            measure.timing.excess.map(format_fraction),
            original["stages"][0]["state"]["stack_excess"]
                .as_str()
                .map(str::to_owned)
        );
        assert_eq!(
            json!(owner.measures[1].timing.slots.len()),
            original["stages"][0]["state"]["other_measure"]["stack_slot_count"]
        );
        assert_eq!(
            original["after_actual_reprocessStack"],
            original["stages"].as_array().unwrap().last().unwrap()["state"]
        );
    }
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

#[cfg(test)]
mod rational_owned_export_tests {
    use super::*;
    use crate::music_voices::{last_end_time, slot_duration};
    #[test]
    fn actual_rational_slots_and_underfilled_voice_termination_match_original() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rational-voice-clocks/observed.json"
        ))
        .unwrap();
        let callback: Value = serde_json::from_str(include_str!(
            "../fixtures/official-headstem-tasks/observed.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = crate::music_xml::recognize(&image).unwrap();
        let owner = &mut export.owner;
        let initial = owner.owner.graph.snapshot();
        let nodes = initial["inters"].as_array().unwrap();
        let edges = initial["relations"].as_array().unwrap();
        let head = |bounds: &Value| {
            nodes
                .iter()
                .find(|n| n["kind"] == "HeadInter" && n["bounds"] == *bounds)
                .unwrap()
        };
        let source = head(&callback["head_bounds"]);
        let target = head(&callback["target_head_bounds"]);
        let old = edges
            .iter()
            .find(|e| e["kind"] == "HeadStemRelation" && e["source"] == source["id"])
            .unwrap();
        let sid = &edges
            .iter()
            .find(|e| e["kind"] == "HeadStemRelation" && e["source"] == target["id"])
            .unwrap()["target"];
        let stem = nodes.iter().find(|n| n["id"] == *sid).unwrap();
        let hk = serde_json::from_value(source["origin"].clone()).unwrap();
        let sk = serde_json::from_value(stem["origin"].clone()).unwrap();
        let cached_expected = owner.measures[0].timing.expected;
        use crate::shared_sig::{EditingOperation, EditingSequence, EditingTask, RelationTask};
        let relation = json!({"kind":"HeadStemRelation","manual":true,"grade":0.7,
            "head_side":old["head_side"],"extension_point":old["extension_point"]});
        let (ci, ck) = owner.manual_head_stem_owner(hk, sk).unwrap();
        let cid = owner.chords[ci].id;
        let chord_ptr = &owner.chords[ci] as *const _;
        let measure_ptr = &owner.measures[0] as *const _;
        let canonical = owner.owner.caches[&sk.system]
            .registered_stem(&owner.owner.graph, sk.local)
            .unwrap();
        let task = RelationTask::manual_head_stem(owner, hk, sk, relation.clone()).unwrap();
        let mut sequence = EditingSequence::from_tasks(vec![EditingTask::Relation(task)]);
        // This owner task cannot run against a detached Graph or enter the
        // public correction path before its original task integration is ready.
        assert_eq!(
            sequence
                .clone()
                .perform(
                    &mut owner.owner.graph.clone(),
                    EditingOperation::Do,
                    "RHYTHMS"
                )
                .unwrap_err(),
            "editing_actual_music_owner_required"
        );
        let before = owner.owner.graph.snapshot();
        let old_chord = owner.chords[ci].clone();
        let bad = RelationTask::manual_head_stem(
            owner,
            hk,
            sk,
            json!({"kind":"HeadStemRelation","manual":true}),
        )
        .unwrap();
        let good = RelationTask::manual_head_stem(owner, hk, sk, relation).unwrap();
        let mut rollback = EditingSequence::from_tasks(vec![
            EditingTask::Relation(good),
            EditingTask::Relation(bad),
        ]);
        assert!(
            rollback
                .perform_owned(owner, EditingOperation::Do, "RHYTHMS")
                .is_err()
        );
        assert_eq!(owner.owner.graph.snapshot(), before);
        assert_eq!(owner.chords[ci].stem, old_chord.stem);
        assert_eq!(owner.chords[ci].bounds, old_chord.bounds);
        assert_eq!(owner.chords[ci].members, old_chord.members);
        let unlink: Value = serde_json::from_str(include_str!(
            "../fixtures/official-headstem-unlink-tasks/observed.json"
        ))
        .unwrap();
        for (unlinking, oracle) in [(false, &callback), (true, &unlink)] {
            if unlinking {
                let snapshot = export.owner.owner.graph.snapshot();
                let id = snapshot["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| {
                        e["source"] == source["id"]
                            && e["kind"] == "HeadStemRelation"
                            && e["manual"] == true
                    })
                    .unwrap()["id"]
                    .as_u64()
                    .unwrap() as usize;
                sequence = EditingSequence::from_tasks(vec![EditingTask::Relation(
                    RelationTask::unlink_manual_head_stem(&export.owner, id).unwrap(),
                )]);
            }
            for (operation, expected) in [
                EditingOperation::Do,
                EditingOperation::Undo,
                EditingOperation::Redo,
            ]
            .into_iter()
            .zip(oracle["steps"].as_array().unwrap())
            {
                let owner = &mut export.owner;
                let before = owner.owner.graph.snapshot();
                let chord_before = before["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == cid)
                    .unwrap();
                let publication_count = before["edit_publications"].as_array().map_or(0, Vec::len);
                owner.owner.graph.observe_chord_callbacks(ck);
                let planned = sequence.perform_owned(owner, operation, "RHYTHMS").unwrap();
                assert_eq!(
                    planned["first_impacted_step"],
                    callback["impacted_steps"][0]
                );
                assert_eq!(planned["planned_steps"], json!(["RHYTHMS"]));
                assert_eq!(planned["reprocessed_steps"], json!([]));
                assert_eq!(planned["rhythm_stack_targets"].as_array().unwrap().len(), 1);
                assert_eq!(&owner.chords[ci] as *const _, chord_ptr);
                assert_eq!(&owner.measures[0] as *const _, measure_ptr);
                assert!(std::sync::Arc::ptr_eq(
                    &canonical,
                    &owner.owner.caches[&sk.system]
                        .registered_stem(&owner.owner.graph, sk.local)
                        .unwrap()
                ));
                let graph = &owner.owner.graph;
                let actual = graph.dot_callback_trace();
                let frames = actual
                    .iter()
                    .filter(|f| f["event"] != "publish")
                    .collect::<Vec<_>>();
                assert_eq!(
                    frames.len(),
                    expected["callbacks"].as_array().unwrap().len()
                );
                fn numbers(v: &Value) -> Value {
                    match v {
                        Value::Number(n) => json!(n.as_f64().unwrap()),
                        Value::Array(a) => json!(a.iter().map(numbers).collect::<Vec<_>>()),
                        Value::Object(o) => {
                            Value::Object(o.iter().map(|(k, v)| (k.clone(), numbers(v))).collect())
                        }
                        _ => v.clone(),
                    }
                }
                for (frame, oracle) in frames.iter().zip(expected["callbacks"].as_array().unwrap())
                {
                    assert_eq!(frame["event"], oracle["event"]);
                    assert_eq!(frame["stage"], oracle["stage"]);
                    assert_eq!(frame["kind"], oracle["kind"]);
                    let observation = &frame["chord_observation"];
                    for field in ["duration", "end_time"] {
                        assert_eq!(observation[field], oracle["owner"]["identity"][field]);
                    }
                    for field in ["cache_before_getters", "cache_after_getters"] {
                        assert_eq!(
                            numbers(&observation[field]),
                            numbers(&oracle["owner"]["cache"][field])
                        );
                    }
                    let chord = frame["state"]["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == cid)
                        .unwrap();
                    for field in [
                        "part_id",
                        "measure_index",
                        "member_ids",
                        "voice",
                        "slot_id",
                        "time_offset",
                    ] {
                        assert_eq!(chord[field], chord_before[field]);
                    }
                    assert_eq!(
                        frame["state"]["editing_book"]["sheet_modified"],
                        oracle["owner"]["identity"]["sheet_modified"]
                    );
                    assert_eq!(
                        frame["state"]["editing_book"]["book_dirty"],
                        oracle["owner"]["identity"]["book_dirty"]
                    );
                }
                let task_snapshot = graph.snapshot();
                let publications =
                    &task_snapshot["edit_publications"].as_array().unwrap()[publication_count..];
                assert_eq!(
                    publications.len(),
                    expected["publications"].as_array().unwrap().len()
                );
                for (actual, oracle) in publications
                    .iter()
                    .zip(expected["publications"].as_array().unwrap())
                {
                    assert_eq!(actual["inter_id"], source["id"]);
                    assert_eq!(
                        actual["task"],
                        if unlinking { "UnlinkTask" } else { "LinkTask" }
                    );
                    assert_eq!(oracle["source_is_head"], true);
                    for field in ["sheet_modified", "book_dirty"] {
                        assert_eq!(
                            actual["actual_dirty_state"][field],
                            oracle["owner"]["identity"][field]
                        );
                    }
                }
                let after = graph.snapshot();
                let identity = &expected["after_task"]["identity"];
                assert_eq!(owner.chords[ci].stem, Some(sk));
                assert_eq!(
                    graph.chord_duration(ck).unwrap().as_deref(),
                    identity["duration"].as_str()
                );
                let support = after["relations"].as_array().unwrap().iter().find(|e| {
                    e["source"] == source["id"]
                        && e["kind"] == "HeadStemRelation"
                        && e["manual"] == true
                });
                assert_eq!(
                    support.is_some(),
                    expected["fresh_registered"].as_bool().unwrap()
                );
                if let Some(e) = support {
                    assert_eq!(e["manual"], expected["fresh_manual"]);
                    for field in ["grade", "source_ratio", "target_ratio"] {
                        assert_eq!(e[field], identity["manual_supports"][0][field]);
                    }
                }
                for field in ["sheet_modified", "book_dirty"] {
                    assert_eq!(
                        graph.editing_book_state().unwrap()[field],
                        expected["after_impact"]["identity"][field]
                    );
                }
                assert_eq!(owner.measures[0].timing.expected, cached_expected);
                owner.reprocess_page_stack_rhythm(1).unwrap();
                assert_eq!(owner.measures[0].timing.expected, cached_expected);
                let rebuilt_timing =
                    crate::music_voices::projection(&owner.voices, &owner.measures);
                let retained = Rhythm::retained(owner).unwrap();
                let xml = export
                    .reprocess_selected_scoped(
                        &[0],
                        retained,
                        "Music",
                        RhythmScope::ObservedOwnedCorrection,
                    )
                    .unwrap();
                let owner = &mut export.owner;
                assert_eq!(
                    crate::music_voices::projection(&owner.voices, &owner.measures),
                    rebuilt_timing
                );
                let snapshot = owner.owner.graph.snapshot();
                let bounds = |id| {
                    let ch = snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == id)
                        .unwrap();
                    snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == ch["member_ids"][0])
                        .unwrap()["bounds"]
                        .clone()
                };
                let m = &owner.measures[0];
                let voices = owner.voices.iter().filter(|v|v.measure==0).map(|v|json!({"id":v.id,"heads":v.chord_ids.iter().map(|id| bounds(*id)).collect::<Vec<_>>(),"duration":slot_duration(&owner.owner.graph,v,&m.timing.slots).unwrap().map(crate::music_time::format_fraction),"end_time":last_end_time(&owner.owner.graph,v).unwrap().map(crate::music_time::format_fraction),"termination":v.termination.map(crate::music_time::format_delta),"excess":v.excess.map(crate::music_time::format_fraction),"slots":m.timing.slots.iter().filter_map(|s|v.slots.get(&s.id).map(|info|json!({"slot_id":s.id,"status":if info.continuing{"CONTINUE"}else{"BEGIN"},"head_bounds":bounds(info.chord_id)}))).collect::<Vec<_>>()})).collect::<Vec<_>>();
                let slots = m.timing.slots.iter().map(|s|json!({"id":s.id,"time_offset":s.time_offset.map(crate::music_time::format_fraction),"x_offset":s.x_offset,"incoming_heads":s.incoming_ids.iter().map(|id|bounds(*id)).collect::<Vec<_>>()})).collect::<Vec<_>>();
                assert_eq!(
                    json!({"expected":m.timing.expected.map(crate::music_time::format_fraction),"actual":m.timing.actual.map(crate::music_time::format_fraction),"stack_excess":m.timing.excess.map(crate::music_time::format_fraction),"abnormal":m.timing.abnormal,"voices":voices,"slots":slots}),
                    original["timing"]
                );
                assert_eq!(cached_expected, m.timing.expected);
                assert_eq!(original["same_expected"], true);
                let rhythm = Rhythm::retained(owner).unwrap();
                let sequences = owner
                    .measures
                    .iter()
                    .enumerate()
                    .map(|(mi, m)| {
                        crate::music_voice_export::sequence(
                            &owner.owner.graph,
                            &owner.voices,
                            m,
                            mi,
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>();
                let divisions = crate::music_voice_export::divisions(
                    &owner.owner.graph,
                    &owner.measures,
                    &owner.chords,
                )
                .unwrap();
                assert_eq!(json!(divisions), original["quarter_divisions"]);
                let mut stream = Vec::new();
                for (mi, steps) in sequences.iter().enumerate() {
                    for step in steps {
                        use crate::music_voice_export::Step;
                        stream.push(match *step {
                    Step::Chord {id,voice,duration} => {
                        let e = rhythm.events.iter().find(|e|e["chord_id"]==id).unwrap();
                        json!({"kind":"note","measure":mi+1,"voice":voice.to_string(),"duration":duration.to_string(),"step":e["step"],"octave":e["octave"],"type":e["type"],"dots":e["dots"],"chord":e["chord"]})
                    },
                    Step::Backup(d) => json!({"kind":"backup","measure":mi+1,"duration":d.to_string()}),
                    Step::Forward {duration,..} => json!({"kind":"forward","measure":mi+1,"duration":duration.to_string()}),
                });
                    }
                }
                assert_eq!(json!(stream), original["original_export_stream"]);
                assert_eq!(json!(stream), expected["music"]);
                assert_eq!(xml.matches("<note>").count(), 8);
                assert!(xml.contains("<divisions>16</divisions>"));
                assert!(xml.contains("<backup><duration>36</duration></backup>"));
                assert!(!xml.contains("<forward>"));
                assert_eq!(
                    owner.owner.prepared.report["MusicXML_written_from_native_image_recognition"],
                    true
                );
                for flag in [
                    "heads_complete",
                    "stems_complete",
                    "musicxml_complete",
                    "epilog_complete",
                    "automatic_raster_pipeline_complete",
                ] {
                    assert_eq!(owner.owner.prepared.report[flag], false);
                }
                if let Ok(path) = std::env::var("AUDIVERIS_RATIONAL_OWNER_XML") {
                    std::fs::write(path, xml).unwrap();
                }
                let mut empty = crate::shared_sig::EditingSequence::new(vec![]);
                assert_eq!(
                    export
                        .apply_editing_sequence(&mut empty, crate::shared_sig::EditingOperation::Do)
                        .unwrap_err(),
                    "music_reprocess_unedited_retained_export_required"
                );
                assert_eq!(
                    export.reprocess_stack_rhythms(&[1]).unwrap_err(),
                    "music_reprocess_unedited_retained_export_required"
                );
            }
        }
    }
}

mod tuplet_sans_duration;

mod tuplet_beats;
pub use tuplet_beats::TupletBeatPlan;

mod tuplet_force;

mod rhythm_candidates;
mod tuplet_find;
mod voice_chords;
mod voice_tuplets;
pub use rhythm_candidates::RhythmCandidates;
mod tuplet_shrink;

mod tuplet_inspect;

mod tuplet_collection;

mod tuplet_merge;

mod tuplet_second_pass;
