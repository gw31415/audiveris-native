// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained Voice chord ownership from the actual starting Chord and Part Staff.
//! Merged Parts and automatic tie recognition remain separate dependencies.
use crate::music_measures::{Chord, Measure};
use crate::music_time::{Delta, Fraction};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[cfg(test)]
pub(crate) mod image_tests;
#[cfg(test)]
mod lifecycle_tests;
pub(crate) mod shared_stack;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VoiceKind {
    High,
    Low,
    Infra,
    HighAndLow,
}
impl VoiceKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Low => "LOW",
            Self::Infra => "INFRA",
            Self::HighAndLow => "HIGH_AND_LOW",
        }
    }
    fn id_offset(self) -> usize {
        match self {
            Self::High => 0,
            Self::Low => 4,
            Self::Infra => 8,
            Self::HighAndLow => 12,
        }
    }
}
fn starting_family(measure: &Measure, seed: &Chord) -> Result<(u64, VoiceKind), String> {
    let Some(geometry) = &measure.geometry else {
        return Ok((measure.staff, VoiceKind::High));
    };
    let staff = seed
        .members
        .last()
        .and_then(|n| n["staff_id"].as_u64())
        .ok_or("music_voice_actual_starting_staff_required")?;
    let index = geometry
        .part
        .staffs
        .iter()
        .position(|s| s.id == staff)
        .ok_or("music_voice_actual_Part_Staff_required")?;
    let kind = match index {
        0 => VoiceKind::High,
        1 => VoiceKind::Low,
        2 => VoiceKind::Infra,
        3 => VoiceKind::HighAndLow,
        _ => VoiceKind::High,
    };
    Ok((staff, kind))
}

#[derive(Clone)]
pub(crate) struct Voice {
    // Staging copies keep this owner identity; musical numbers and list entries
    // alone cannot identify the Voice referenced by a cached Chord.
    pub(crate) identity: Arc<()>,
    pub id: usize,
    pub measure: usize,
    pub starting_staff: u64,
    pub(crate) kind: VoiceKind,
    pub chord_ids: Vec<usize>,
    // getChords exposes both null and empty as [], but direct duration reads
    // distinguish resetChords' null list from the last removeChord's empty list.
    pub(crate) chords_initialized: bool,
    pub termination: Option<Delta>,
    pub excess: Option<Fraction>,
    pub slots: BTreeMap<usize, SlotVoice>,
    pub measure_rest_chord: Option<usize>,
}
impl std::fmt::Debug for Voice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Voice")
            .field("id", &self.id)
            .field("measure", &self.measure)
            .field("chord_ids", &self.chord_ids)
            .field("chords_initialized", &self.chords_initialized)
            .field("slot_count", &self.slots.len())
            .finish()
    }
}
impl Voice {
    pub(crate) fn reset_chords(&mut self) {
        self.chord_ids = Vec::new();
        self.chords_initialized = false;
    }
}
/// The admitted one-part/one-staff measure has one stack; general shared
/// multi-part stacks remain pending and must share this state when connected.
#[derive(Clone)]
pub(crate) struct Slot {
    pub identity: Arc<()>,
    pub id: usize,
    pub time_offset: Option<Fraction>,
    pub x_offset: i64,
    pub incoming_ids: Vec<usize>,
}
#[derive(Clone, Copy)]
pub(crate) struct SlotVoice {
    pub chord_id: usize,
    pub continuing: bool,
}
#[derive(Clone, Default)]
pub(crate) struct StackTiming {
    pub expected: Option<Fraction>,
    pub excess: Option<Fraction>,
    pub slots: Vec<Slot>,
    pub actual: Option<Fraction>,
}
#[derive(Clone, Default)]
pub(crate) struct MeasureTiming {
    pub expected: Option<Fraction>,
    pub excess: Option<Fraction>,
    pub abnormal: bool,
    pub slots: Vec<Slot>,
    pub actual: Option<Fraction>,
}
/// SystemInfo.isMeasureRestShape. Current image CLI uses original default
/// partialWholeRests=false; exposing that processing switch and LONG/BREVE
/// raster support remains a separate dependency.
pub(crate) fn is_measure_rest_shape(shape: &str, partial_whole_rests: bool) -> bool {
    matches!(shape, "LONG_REST" | "BREVE_REST") || (shape == "WHOLE_REST" && !partial_whole_rests)
}
/// Voice constructor/createMeasureRestVoice -> inferVoiceKind/generateVoiceId, setVoice.
/// Measure-rest Voice keeps its separate rest chord, not a normal chord list. The
/// caller has already admitted a single voice in this nonmerged one-staff part.
pub(crate) fn single(
    measure_index: usize,
    measure: &Measure,
    chords: &[&Chord],
    existing: &[Voice],
) -> Result<Voice, String> {
    if existing.iter().any(|v| v.measure == measure_index) {
        return Err("music_voice_first_assignment_required".into());
    }
    construct(measure_index, measure, chords, existing)
}
fn construct(
    measure_index: usize,
    measure: &Measure,
    chords: &[&Chord],
    existing: &[Voice],
) -> Result<Voice, String> {
    if chords.is_empty() {
        return Err("music_voice_first_assignment_required".into());
    }
    let mut members = Vec::new();
    let mut seen = BTreeSet::new();
    for chord in chords {
        if chord.measure != measure_index || chord.key.system != measure.system {
            return Err("music_voice_owned_measure_chord_required".into());
        }
        // Original addChord ignores an existing actual chord and stably sorts
        // by integer center abscissa; it does not order by registration ID.
        if seen.insert(chord.id) {
            members.push(*chord);
        }
    }
    members.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
    let (starting_staff, kind) = starting_family(measure, chords[0])?;
    let id = (kind.id_offset() + 1..)
        .find(|id| {
            !existing
                .iter()
                .any(|v| v.measure == measure_index && v.id == *id)
        })
        .unwrap();
    let measure_rests = members
        .iter()
        .filter(|c| {
            c.kind == crate::music_measures::ChordKind::Rest
                && c.members
                    .first()
                    .and_then(|n| n["shape"].as_str())
                    .is_some_and(|s| is_measure_rest_shape(s, false))
        })
        .collect::<Vec<_>>();
    if !measure_rests.is_empty() && members.len() != 1 {
        return Err("music_voice_measure_rest_parallel_voices_pending".into());
    }
    let measure_rest_chord = measure_rests.first().map(|c| c.id);
    Ok(Voice {
        identity: Arc::new(()),
        id,
        measure: measure_index,
        starting_staff,
        kind,
        chord_ids: if measure_rest_chord.is_some() {
            Vec::new()
        } else {
            members.iter().map(|c| c.id).collect()
        },
        chords_initialized: measure_rest_chord.is_none(),
        termination: None,
        excess: None,
        slots: BTreeMap::new(),
        measure_rest_chord,
    })
}
pub(crate) fn create_voice(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measures: &[Measure],
    seed: &Chord,
) -> Result<usize, String> {
    let snapshot = graph.snapshot();
    let node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == seed.id)
        .ok_or("music_voice_live_seed_required")?;
    if !node["voice"].is_null() {
        return Err("music_voice_unassigned_seed_required".into());
    }
    construct_owned_voice(graph, voices, measures, seed)
}
fn construct_owned_voice(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measures: &[Measure],
    seed: &Chord,
) -> Result<usize, String> {
    let voice = if voices.iter().any(|v| v.measure == seed.measure) {
        construct(seed.measure, &measures[seed.measure], &[seed], voices)?
    } else {
        single(seed.measure, &measures[seed.measure], &[seed], voices)?
    };
    let index = voices.len();
    let mut retained = voices.clone();
    let mut staged = graph.clone();
    retained.push(voice);
    assign_chord_voice(&mut staged, &mut retained, measures, seed.id, index)?;
    *graph = staged;
    *voices = retained;
    Ok(index)
}

/// Original processStartingChords constructs every selected Voice, even when
/// an earlier constructor already assigned the seed through a relation. Rookie
/// purging and its unassigned-seed requirement are a different processing phase.
pub(crate) fn process_starting_chords(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measures: &[Measure],
    measure_index: usize,
    selected: &[&Chord],
) -> Result<Vec<usize>, String> {
    let measure = measures
        .get(measure_index)
        .ok_or("music_voice_measure_required")?;
    let snapshot = graph.snapshot();
    let mut ordered = Vec::new();
    for &chord in selected {
        let n = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("music_voice_live_seed_required")?;
        if chord.measure != measure_index
            || graph.id(chord.key) != Some(chord.id)
            || n["origin"]["system"] != measure.system
            || n["part_id"] != measure.part
            || n["measure_index"] != measure.index
        {
            return Err("music_voice_owned_starting_chord_required".into());
        }
        let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
            .map_err(|_| "music_voice_starting_bounds_required")?;
        ordered.push((chord, b[1] + b[3] / 2));
    }
    ordered.sort_by_key(|(_, y)| *y);
    let mut staged = graph.clone();
    let mut retained = voices.clone();
    for &(chord, _) in &ordered {
        construct_owned_voice(&mut staged, &mut retained, measures, chord)?;
        crate::music_chord_time::set_and_push_ticks(
            &mut staged,
            &retained,
            measures,
            chord.id,
            Some(0),
        )?;
    }
    *graph = staged;
    *voices = retained;
    Ok(ordered.iter().map(|(chord, _)| chord.id).collect())
}

/// Measure.removeVoiceChord -> Voice.removeChord. Remove an empty Voice only
/// when this removal actually removed its chord, matching the original loop.
pub(crate) fn remove_chords(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    removed: &[usize],
) {
    for id in removed {
        let mut index = 0;
        while index < voices.len() {
            if let Some(member) = voices[index].chord_ids.iter().position(|c| c == id) {
                voices[index].chord_ids.remove(member);
                if voices[index].chord_ids.is_empty() {
                    graph.retain_voice_owner(voices.remove(index));
                    break;
                }
            }
            index += 1;
        }
    }
}
pub(crate) fn purge_measure_voices(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measure_index: usize,
) {
    let mut i = 0;
    while i < voices.len() {
        let voice = &voices[i];
        if voice.measure == measure_index
            && voice.measure_rest_chord.is_none()
            && voice.chord_ids.is_empty()
        {
            graph.retain_voice_owner(voices.remove(i));
        } else {
            i += 1;
        }
    }
}
pub(crate) fn projection(voices: &[Voice], measures: &[Measure]) -> Value {
    json!(
        voices
            .iter()
            .map(|v| {
                let m = &measures[v.measure];
                json!({"id":v.id,"kind":v.kind.label(),"system_id":m.system,"part_id":m.part,
            "measure_index":m.index,"starting_staff_id":v.starting_staff,"chord_ids":v.chord_ids,"measure_rest_chord_id":v.measure_rest_chord,"is_measure_rest":v.measure_rest_chord.is_some(),"termination":v.termination.map(crate::music_time::format_delta),"excess":v.excess.map(crate::music_time::format_fraction),"slots":v.slots.iter().map(|(id,s)|json!({"slot_id":id,"chord_id":s.chord_id,"status":if s.continuing {"CONTINUE"}else{"BEGIN"}})).collect::<Vec<_>>()})
            })
            .collect::<Vec<_>>()
    )
}

/// StackRhythm.process sets expected duration before Measure processing.
/// Stack reset does not clear retained Measure Voices or Chord timing.
pub(crate) fn reset_stack_rhythm(measure: &mut Measure, expected: Option<Fraction>) {
    measure.timing.expected = expected;
    measure.timing.abnormal = false;
    measure.timing.excess = None;
    measure.timing.slots.clear();
    measure.timing.actual = None;
}

/// Voices.refineSystem: purge/sort/rename by retained Voice kind, then actual
/// tie, previous-Measure BeamGroup, NextInVoice and preferred-ID swaps, in
/// original order. Classified relations are distinct from image discovery.
pub(crate) fn refine_ordinary_system(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measures: &[Measure],
    system: u64,
) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let nodes: BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let groups = beam_group_chords(graph)?;
    let mut staged = graph.clone();
    let mut retained = voices.clone();
    let mut previous = BTreeMap::<u64, usize>::new();
    for (mi, measure) in measures
        .iter()
        .enumerate()
        .filter(|(_, m)| m.system == system)
    {
        let first = |v: &Voice| {
            v.measure_rest_chord
                .or_else(|| v.chord_ids.first().copied())
        };
        purge_measure_voices(&mut staged, &mut retained, mi);
        let positions: Vec<_> = retained
            .iter()
            .enumerate()
            .filter(|(_, v)| v.measure == mi)
            .map(|(i, _)| i)
            .collect();
        let mut ordered: Vec<_> = positions.iter().map(|&i| retained[i].clone()).collect();
        let mut keys = BTreeMap::new();
        for v in &ordered {
            let id = first(v).unwrap();
            let node = nodes
                .get(&id)
                .ok_or("music_refine_live_first_chord_required")?;
            if node["origin"]["system"] != measure.system
                || node["part_id"] != measure.part
                || node["measure_index"] != measure.index
            {
                return Err("music_refine_owned_HIGH_voice_required".into());
            }
            let staff_belongs = if let Some(geometry) = &measure.geometry {
                staged.require_actual_music_measure(geometry)?;
                geometry
                    .part
                    .staffs
                    .iter()
                    .any(|s| s.id == v.starting_staff)
            } else {
                v.starting_staff == measure.staff
            };
            if !staff_belongs {
                return Err("music_refine_owned_HIGH_voice_required".into());
            }
            // Reassignment leaves old member lists and cached kind intact.
            let slot = if v.measure_rest_chord.is_some() {
                1
            } else {
                node["slot_id"]
                    .as_u64()
                    .ok_or("music_refine_first_slot_required")?
            };
            let bounds: [i64; 4] = serde_json::from_value(node["bounds"].clone())
                .map_err(|_| "music_refine_first_chord_bounds_required")?;
            let y = bounds[1] + bounds[3] / 2;
            keys.insert(id, (slot, y));
        }
        ordered.sort_by_key(|v| (v.kind.id_offset(), keys[&first(v).unwrap()]));
        let mut next = BTreeMap::new();
        for voice in ordered.iter_mut() {
            let id = next
                .entry(voice.kind.id_offset())
                .or_insert(voice.kind.id_offset());
            *id += 1;
            set_owned_voice_id(&mut staged, voice, *id)?;
        }
        for (&i, voice) in positions.iter().zip(ordered) {
            retained[i] = voice;
        }
        for &index in &positions {
            let first_id = first(&retained[index]).unwrap();
            if let Some(&prev) = previous.get(&measure.part) {
                let tied = staged.incoming_tied_voice_id(first_id, |other| {
                    chord_voice_id(&staged, &retained, measures, other)
                })?;
                if let Some(wanted) = tied {
                    swap_voice_id(&mut staged, &mut retained, index, wanted)?;
                }
                if let Some(group_id) = chord_group(&snapshot, first_id)? {
                    let group = groups
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|g| g["id"] == group_id)
                        .ok_or("music_refine_live_beam_group_required")?;
                    let chords = group["chords"].as_array().unwrap();
                    let contains_previous = chords.iter().any(|c| {
                        let n = nodes[&(c["id"].as_u64().unwrap() as usize)];
                        n["origin"]["system"] == measures[prev].system
                            && n["part_id"] == measures[prev].part
                            && n["measure_index"] == measures[prev].index
                    });
                    if contains_previous
                        && let Some(position) = chords
                            .iter()
                            .position(|c| c["id"] == first_id)
                            .filter(|&position| position > 0)
                    {
                        let other = chords[position - 1]["id"].as_u64().unwrap() as usize;
                        let wanted = chord_voice_id(&staged, &retained, measures, other)?
                            .ok_or("music_refine_beam_predecessor_owned_voice_required")?;
                        swap_voice_id(&mut staged, &mut retained, index, wanted)?;
                    }
                }
                for other in staged.next_in_voice_neighbors(first_id)? {
                    let node = nodes
                        .get(&other)
                        .ok_or("music_refine_next_chord_required")?;
                    if node["origin"]["system"] != measure.system
                        || node["part_id"] != measure.part
                        || node["measure_index"] != measures[prev].index
                    {
                        continue;
                    }
                    let wanted = chord_voice_id(&staged, &retained, measures, other)?
                        .ok_or("music_refine_previous_owned_voice_required")?;
                    swap_voice_id(&mut staged, &mut retained, index, wanted)?;
                    break;
                }
            }
            if !nodes[&first_id]["preferred_voice_id"].is_null() {
                let wanted = nodes[&first_id]["preferred_voice_id"]
                    .as_u64()
                    .and_then(|id| usize::try_from(id).ok())
                    .ok_or("music_refine_preferred_ID_outside_native_range_pending")?;
                swap_voice_id(&mut staged, &mut retained, index, wanted)?;
            }
        }
        previous.insert(measure.part, mi);
    }
    *graph = staged;
    *voices = retained;
    Ok(())
}

fn chord_voice_id(
    graph: &crate::shared_sig::Graph,
    voices: &[Voice],
    measures: &[Measure],
    id: usize,
) -> Result<Option<usize>, String> {
    let snapshot = graph.snapshot();
    let chord = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .ok_or("music_refine_live_chord_voice_required")?;
    if chord["voice"].is_null() {
        return Ok(None);
    }
    let key = serde_json::from_value(chord["origin"].clone())
        .map_err(|_| "music_refine_actual_chord_voice_required")?;
    let voice = graph
        .chord_voice_owner(key, voices)?
        .ok_or("music_refine_actual_chord_voice_required")?;
    if chord["voice"] != voice.id
        || !measures.get(voice.measure).is_some_and(|m| {
            chord["part_id"] == m.part
                && chord["measure_index"] == m.index
                && chord["origin"]["system"] == m.system
        })
    {
        return Err("music_refine_actual_chord_voice_required".into());
    }
    Ok(Some(voice.id))
}

/// Measure.swapVoiceId changes the selected Voice first, then transfers its old
/// ID to the former owner. Chords retain their Voice object; their native scalar
/// projection follows the actual owned membership, not another numeric label.
pub(crate) fn swap_voice_id(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut [Voice],
    index: usize,
    wanted: usize,
) -> Result<(), String> {
    let old = voices[index].id;
    if old == wanted {
        return Ok(());
    }
    let measure = voices[index].measure;
    let previous = voices
        .iter()
        .position(|v| v.measure == measure && v.id == wanted);
    set_owned_voice_id(graph, &mut voices[index], wanted)?;
    if let Some(previous) = previous {
        set_owned_voice_id(graph, &mut voices[previous], old)?;
    }
    Ok(())
}

fn set_owned_voice_id(
    graph: &mut crate::shared_sig::Graph,
    voice: &mut Voice,
    id: usize,
) -> Result<(), String> {
    graph.rename_voice_references(voice, id)?;
    voice.id = id;
    Ok(())
}

/// Measure.resetRhythm -> AbstractChordInter.resetTiming before rhythm passes.
/// Clear only this measure's Voices, abnormal flag, chord slot/voice/time.
/// Original Measure reset does not reset chord duration or stack timing/slots.
/// Commit after all real chord identities are checked; registry/cache/relations
/// and other measures remain owned and unchanged.
pub(crate) fn reset_rhythm(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measure: &mut Measure,
    chords: &[Chord],
    measure_index: usize,
) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut staged = graph.clone();
    let mut retained = voices.clone();
    let mut index = 0;
    while index < retained.len() {
        if retained[index].measure == measure_index {
            staged.retain_voice_owner(retained.remove(index));
        } else {
            index += 1;
        }
    }
    for chord in chords.iter().filter(|c| c.measure == measure_index) {
        if chord.key.system != measure.system || graph.id(chord.key) != Some(chord.id) {
            return Err("music_rhythm_reset_owned_chord_required".into());
        }
        let mut node = nodes
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("music_rhythm_reset_live_chord_required")?
            .clone();
        if !matches!(
            node["kind"].as_str(),
            Some("HeadChordInter" | "RestChordInter")
        ) {
            return Err("music_rhythm_reset_chord_kind_required".into());
        }
        for field in ["slot_id", "voice", "time_offset"] {
            node[field] = Value::Null;
        }
        staged.vertex(chord.key, node, false)?;
    }
    *graph = staged;
    *voices = retained;
    measure.timing.abnormal = false;
    Ok(())
}

/// StackRhythm.generateStackSlots -> Slot incoming order/back-reference ->
/// Voice.putSlotInfo(BEGIN)/completeSlotTable. Current production admission has
/// one Measure per stack; multi-part stack ownership/inter-staff geometric interpolation remain pending.
/// Build against live chords, then commit atomically with Voice/Measure state.
pub(crate) fn generate_slots(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut [Voice],
    measure: &mut Measure,
    measure_index: usize,
) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let node = |id: usize| {
        nodes
            .iter()
            .find(|n| n["id"] == id)
            .ok_or_else(|| "music_slot_live_chord_required".to_string())
    };
    let mut times: BTreeMap<Fraction, Vec<usize>> = BTreeMap::new();
    let mut staged_measure = measure.clone();
    let mut staged_voices = voices.to_vec();
    for voice in staged_voices
        .iter()
        .filter(|v| v.measure == measure_index && v.measure_rest_chord.is_none())
    {
        for &id in &voice.chord_ids {
            let ch = node(id)?;
            if !matches!(
                ch["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            ) {
                return Err("music_slot_ordinary_chord_required".into());
            }
            if let Some(time) = ch["time_offset"].as_str() {
                times.entry(Fraction::parse(time)?).or_default().push(id);
            } else {
                staged_measure.timing.abnormal = true;
            }
        }
    }
    let mut staged_graph = graph.clone();
    let mut slots = Vec::new();
    let mut time_to_slot = BTreeMap::new();
    for (time, incoming) in &times {
        let mut incoming = incoming.clone();
        let mut ordered = Vec::new();
        for &id in &incoming {
            let b: [i64; 4] = serde_json::from_value(node(id)?["bounds"].clone())
                .map_err(|_| "music_slot_chord_bounds_required")?;
            ordered.push((id, b[1] + b[3] / 2, b[0] + b[2] / 2));
        }
        ordered.sort_by_key(|(_, y, _)| *y);
        // Slot.computeXOffset: original integer chord centers -> mean point ->
        // stack offset -> Math.rint. One staff/zero skew makes y immaterial.
        let mean_x = ordered.iter().map(|(_, _, x)| *x as f64).sum::<f64>() / ordered.len() as f64;
        let x_offset = (mean_x - measure.left as f64).round_ties_even() as i64;
        incoming.clear();
        incoming.extend(ordered.into_iter().map(|(id, _, _)| id));
        let slot_id = slots.len() + 1;
        let slot = Slot {
            identity: Arc::new(()),
            id: slot_id,
            time_offset: Some(*time),
            x_offset,
            incoming_ids: incoming,
        };
        for &id in &slot.incoming_ids {
            let data = node(id)?;
            let key = serde_json::from_value(data["origin"].clone())
                .map_err(|_| "music_slot_actual_origin_required")?;
            staged_graph.set_chord_slot(key, slot_id, &slot.identity)?;
        }
        time_to_slot.insert(*time, slot_id);
        slots.push(slot);
    }
    for voice in staged_voices
        .iter_mut()
        .filter(|v| v.measure == measure_index && v.measure_rest_chord.is_none())
    {
        // Original putSlotInfo overwrites by slot ID; do not silently clear
        // previous entries. Rhythm reset/voice construction owns that reset.
        for &id in &voice.chord_ids {
            if let Some(time) = node(id)?["time_offset"].as_str() {
                let slot = time_to_slot[&Fraction::parse(time)?];
                voice.slots.insert(
                    slot,
                    SlotVoice {
                        chord_id: id,
                        continuing: false,
                    },
                );
            }
        }
        let mut previous = None;
        for slot in &slots {
            if let Some(info) = voice.slots.get(&slot.id) {
                previous = Some(info.chord_id);
            } else if let Some(id) = previous {
                let ch = node(id)?;
                let offset = ch["time_offset"]
                    .as_str()
                    .ok_or("music_slot_offset_required")?;
                let end = slot_chord_end(&snapshot, id, Fraction::parse(offset)?)?;
                if end > slot.time_offset.ok_or("music_slot_time_required")? {
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
    staged_measure.timing.slots = slots;
    *graph = staged_graph;
    voices.clone_from_slice(&staged_voices);
    *measure = staged_measure;
    Ok(())
}
pub(crate) fn slots_projection(slots: &[Slot]) -> Value {
    json!(slots.iter().map(|s|json!({"id":s.id,"time_offset":s.time_offset.map(crate::music_time::format_fraction),"x_offset":s.x_offset,"incoming_chord_ids":s.incoming_ids})).collect::<Vec<_>>())
}

/// MeasureStack.getClosestSlot: strict distance improvement preserves original
/// time-ordered first slot on a tie; empty slots produce no selection.
pub(crate) fn closest_slot(slots: &[Slot], x_offset: f64) -> Option<&Slot> {
    let mut best = None;
    let mut distance = f64::MAX;
    for slot in slots {
        let dx = (slot.x_offset as f64 - x_offset).abs();
        if dx < distance {
            distance = dx;
            best = Some(slot);
        }
    }
    best
}

/// Voice.getDuration uses BEGIN entries and their slot time, not the last
/// horizontally sorted chord. Measure-rest Voice duration is unknown. Nullable
/// slot time propagates an unknown normal Voice duration.
pub(crate) fn slot_duration(
    graph: &crate::shared_sig::Graph,
    voice: &Voice,
    slots: &[Slot],
) -> Result<Option<Fraction>, String> {
    if voice.measure_rest_chord.is_some() {
        return Ok(None);
    }
    let snapshot = graph.snapshot();
    let mut duration = Fraction::ZERO;
    for slot in slots {
        if let Some(info) = voice.slots.get(&slot.id).filter(|s| !s.continuing) {
            let Some(time) = slot.time_offset else {
                return Ok(None);
            };
            let end = slot_chord_end(&snapshot, info.chord_id, time)?;
            duration = duration.max(end);
        }
    }
    Ok(Some(duration))
}
fn slot_chord_end(snapshot: &Value, id: usize, time: Fraction) -> Result<Fraction, String> {
    let duration = crate::music_duration::live_exact(snapshot, id)?
        .intrinsic
        .ok_or("music_slot_duration_required")?;
    time.plus(duration)
}
/// MeasureStack.getSlotsDuration: all incoming chords, missing slot clocks
/// skipped. A CONTINUE entry does not introduce a second chord duration.
pub(crate) fn stack_slots_duration(
    graph: &crate::shared_sig::Graph,
    slots: &[Slot],
) -> Result<Fraction, String> {
    let snapshot = graph.snapshot();
    let mut duration = Fraction::ZERO;
    for slot in slots {
        if let Some(time) = slot.time_offset {
            for &id in &slot.incoming_ids {
                duration = duration.max(slot_chord_end(&snapshot, id, time)?);
            }
        }
    }
    Ok(duration)
}
/// StackRhythm.readStackActualDuration. The current one-part/one-staff Measure
/// owns one stack. A whole-measure-rest Voice has no slots, so its actual
/// stack duration follows the expected duration. Other no-slot Voices use
/// the same original fallback. General multi-part stack ownership is pending.
pub(crate) fn read_stack_actual_duration(
    graph: &crate::shared_sig::Graph,
    voices: &[Voice],
    measure: &mut Measure,
    measure_index: usize,
) -> Result<(), String> {
    let actual = if !voices.iter().any(|v| v.measure == measure_index) {
        Some(Fraction::ZERO)
    } else {
        let duration = stack_slots_duration(graph, &measure.timing.slots)?;
        if duration != Fraction::ZERO {
            Some(duration)
        } else {
            measure.timing.expected
        }
    };
    measure.timing.actual = actual;
    Ok(())
}

/// MeasureRhythm.finalCheck -> Measure.checkDuration on the actual measure owner.
/// A missing nonlast clock still permits all Voice duration updates. Unknown
/// expected duration skips both the duration loop and its abnormal-state gate.
pub(crate) fn final_check(
    graph: &crate::shared_sig::Graph,
    voices: &mut [Voice],
    measure: &mut Measure,
    chords: &[Chord],
    measure_index: usize,
) -> Result<bool, String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut ok = true;
    for chord in chords.iter().filter(|c| c.measure == measure_index) {
        let actual = nodes
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("music_voice_final_live_chord_required")?;
        match actual["kind"].as_str() {
            Some("SmallChordInter") => continue,
            Some("HeadChordInter" | "RestChordInter") => {}
            _ => return Err("music_voice_ordinary_chord_required".into()),
        }
        ok &= !actual["voice"].is_null() && !actual["time_offset"].is_null();
    }
    if measure.timing.expected.is_some() {
        let mut first_error = None;
        for voice in voices.iter_mut().filter(|v| v.measure == measure_index) {
            if let Err(error) = check_duration(graph, voice, measure) {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        ok &= !measure.timing.abnormal;
    }
    Ok(ok)
}

/// Original Voice.checkDuration for normal voices with exact Rational clocks.
/// Underflow sets termination only. Excess persists and updates stack maximum;
/// absent timing marks abnormal without erasing previous termination/excess.
pub(crate) fn check_duration(
    graph: &crate::shared_sig::Graph,
    voice: &mut Voice,
    measure: &mut Measure,
) -> Result<(), String> {
    if voice.measure_rest_chord.is_some() {
        voice.termination = None;
        return Ok(());
    }
    if voice.chord_ids.is_empty() {
        return Ok(());
    }
    let end = match last_end_time(graph, voice) {
        Ok(Some(end)) => end,
        Ok(None) => {
            measure.timing.abnormal = true;
            return Ok(());
        }
        Err(error) => {
            measure.timing.abnormal = true;
            return Err(error);
        }
    };
    let Some(expected) = measure.timing.expected else {
        measure.timing.abnormal = true;
        return Ok(());
    };
    let delta = end.difference(expected).inspect_err(|_| {
        measure.timing.abnormal = true;
    })?;
    voice.termination = Some(delta);
    if let Some(excess) = delta.excess() {
        voice.excess = Some(excess);
        measure.timing.abnormal = true;
        if measure.timing.excess.is_none_or(|old| excess > old) {
            measure.timing.excess = Some(excess);
        }
    }
    Ok(())
}

/// Voice.getLastChord -> actual AbstractChord.getEndTime. Nullable duration or
/// offset is not guessed. Ordinary Rational endTime is live; general tuplets
/// and other special-chord duration remain pending.
pub(crate) fn last_end_time(
    graph: &crate::shared_sig::Graph,
    voice: &Voice,
) -> Result<Option<Fraction>, String> {
    if voice.measure_rest_chord.is_some() {
        return Ok(None);
    }
    let Some(id) = voice.chord_ids.last() else {
        return Ok(None);
    };
    let snapshot = graph.snapshot();
    let last = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == *id)
        .ok_or("music_voice_last_live_chord_required")?;
    if !matches!(
        last["kind"].as_str(),
        Some("HeadChordInter" | "RestChordInter")
    ) {
        return Err("music_voice_ordinary_chord_required".into());
    }
    let key = serde_json::from_value(last["origin"].clone())
        .map_err(|_| "music_voice_last_origin_required")?;
    graph
        .chord_end_time(key)?
        .as_deref()
        .map(Fraction::parse)
        .transpose()
}
pub(crate) fn timed_projection(
    voices: &[Voice],
    measures: &[Measure],
    graph: &crate::shared_sig::Graph,
) -> Result<Value, String> {
    let mut result = projection(voices, measures);
    for (row, voice) in result.as_array_mut().unwrap().iter_mut().zip(voices) {
        row["end_time"] =
            json!(last_end_time(graph, voice)?.map(crate::music_time::format_fraction));
        row["duration"] = json!(
            slot_duration(graph, voice, &measures[voice.measure].timing.slots)?
                .map(crate::music_time::format_fraction)
        );
    }
    Ok(result)
}

// BeamGroupInter.getChords/getFirstChord/getVoice, from actual retained SIG.
// The caller is the native owner after voice assignment, never a serialized
// selected-chord API. General setVoice/group-rest/tie propagation is separate.
pub(crate) fn beam_groups(
    graph: &crate::shared_sig::Graph,
    voices: &[Voice],
    measures: &[Measure],
) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let nodes: BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let mut rows = beam_group_chords(graph)?;
    for row in rows.as_array_mut().unwrap() {
        let system = row["system_id"].as_u64().unwrap();
        let owner = row["chords"]
            .as_array()
            .unwrap()
            .first()
            .map(|c| c["id"].as_u64().unwrap() as usize)
            .as_ref()
            .map(|id| -> Result<Option<&Voice>, String> {
                let chord = nodes[id];
                if chord["voice"].is_null() {
                    return Ok(None);
                }
                let key = serde_json::from_value(chord["origin"].clone())
                    .map_err(|_| "music_group_retained_voice_owner_required")?;
                // Empty member lists and purging do not clear cached Chord ownership.
                let voice = graph
                    .chord_voice_owner(key, voices)?
                    .ok_or("music_group_retained_voice_owner_required")?;
                if chord["voice"] != voice.id
                    || !measures.get(voice.measure).is_some_and(|measure| {
                        measure.system == system
                            && chord["part_id"] == measure.part
                            && chord["measure_index"] == measure.index
                    })
                {
                    return Err("music_group_retained_voice_owner_required".into());
                }
                Ok(Some(voice))
            })
            .transpose()?
            .flatten();
        row["voice"] = json!(owner.map(|v| v.id));
        row["voice_measure_number"] = json!(owner.map(|v| v.measure + 1));
    }
    Ok(rows)
}

/// BeamGroup.getChords does not read getVoice. Slot geometry may be retrieved
/// while other measures still have their retained Voices and clocks.
pub(crate) fn beam_group_chords(graph: &crate::shared_sig::Graph) -> Result<Value, String> {
    collect_beam_group_chords(graph, true, None)
}

pub(crate) fn beam_group_chords_by_member(
    graph: &crate::shared_sig::Graph,
) -> Result<Value, String> {
    // getMeasures collects before the group left sort; stable measure ties
    // would otherwise lose the original beam/member encounter order.
    collect_beam_group_chords(graph, false, None)
}

pub(crate) fn beam_group_chords_for(
    graph: &crate::shared_sig::Graph,
    group: usize,
) -> Result<Value, String> {
    collect_beam_group_chords(graph, true, Some(group))
}

fn collect_beam_group_chords(
    graph: &crate::shared_sig::Graph,
    sort_group_by_left: bool,
    selected_group: Option<usize>,
) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let nodes: std::collections::BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let mut bounds = BTreeMap::new();
    let edges = snapshot["relations"].as_array().unwrap();
    let mut rows = Vec::new();
    for group in nodes.values().filter(|n| n["kind"] == "BeamGroupInter") {
        let gid = group["id"].as_u64().unwrap() as usize;
        if selected_group.is_some_and(|id| id != gid) {
            continue;
        }
        let system = group["origin"]["system"].as_u64().unwrap();
        let beams = edges
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == gid)
            .map(|e| {
                nodes
                    .get(&(e["target"].as_u64().unwrap() as usize))
                    .copied()
                    .ok_or("music_group_live_beam_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut chord_ids = Vec::new();
        let mut seen = BTreeSet::new();
        for beam in &beams {
            let mut beam_chords = Vec::new();
            let mut beam_seen = BTreeSet::new();
            if !matches!(beam["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                || beam["origin"]["system"] != system
            {
                return Err("music_group_typed_owned_beam_required".into());
            }
            // Original AbstractBeam collects all linked heads before rests;
            // stable center ties retain that order before the group left sort.
            let relations = edges
                .iter()
                .filter(|e| e["source"] == beam["id"] && e["kind"] == "BeamStemRelation")
                .chain(
                    edges
                        .iter()
                        .filter(|e| e["source"] == beam["id"] && e["kind"] == "BeamRestRelation"),
                );
            for relation in relations {
                let ids = match relation["kind"].as_str() {
                    Some("BeamStemRelation") => {
                        let stem = nodes
                            .get(&(relation["target"].as_u64().unwrap() as usize))
                            .ok_or("music_group_live_stem_required")?;
                        if stem["kind"] != "StemInter" || stem["origin"]["system"] != system {
                            return Err("music_group_typed_owned_stem_required".into());
                        }
                        edges
                            .iter()
                            .filter(|e| {
                                e["kind"] == "ChordStemRelation" && e["target"] == stem["id"]
                            })
                            .map(|e| (e["source"].as_u64().unwrap() as usize, "HeadChordInter"))
                            .collect::<Vec<_>>()
                    }
                    Some("BeamRestRelation") => {
                        // AbstractBeam.getChords follows the actual RestInter's
                        // first incoming Containment, not the beam's target as
                        // if it were already a RestChord.
                        let rest = nodes
                            .get(&(relation["target"].as_u64().unwrap() as usize))
                            .ok_or("music_group_live_rest_required")?;
                        if rest["kind"] != "RestInter" || rest["origin"]["system"] != system {
                            return Err("music_group_typed_owned_rest_required".into());
                        }
                        let parent = edges
                            .iter()
                            .find(|e| e["kind"] == "Containment" && e["target"] == rest["id"])
                            .ok_or("music_group_rest_ensemble_required")?;
                        vec![(
                            parent["source"].as_u64().unwrap() as usize,
                            "RestChordInter",
                        )]
                    }
                    _ => continue,
                };
                for (id, kind) in ids {
                    let chord = nodes.get(&id).ok_or("music_group_live_chord_required")?;
                    if chord["kind"] != kind || chord["origin"]["system"] != system {
                        return Err("music_group_typed_owned_chord_required".into());
                    }
                    if let std::collections::btree_map::Entry::Vacant(entry) = bounds.entry(id) {
                        let key = serde_json::from_value(chord["origin"].clone())
                            .map_err(|_| "music_group_chord_origin_required")?;
                        entry.insert(graph.retained_or_live_chord_bounds(key)?);
                    }
                    if beam_seen.insert(id) {
                        beam_chords.push(id);
                    }
                }
            }
            // Each beam first sorts its head/rest chords by center. The group
            // then collects these lists and stably sorts by left abscissa.
            beam_chords.sort_by_key(|id| bounds[id][0] + bounds[id][2] / 2);
            for id in beam_chords {
                if seen.insert(id) {
                    chord_ids.push(id);
                }
            }
        }
        // Original getChords uses byAbscissa (left bounds), not the Voice's
        // center comparator. LinkedHashSet preserves ties and removes aliases.
        if sort_group_by_left {
            chord_ids.sort_by_key(|id| bounds[id][0]);
        }
        rows.push(json!({"id":gid,"system_id":system,"bounds":group["bounds"],
            "beams":beams.iter().map(|n|json!({"id":n["id"],"kind":n["kind"],"bounds":n["bounds"]})).collect::<Vec<_>>(),
            "chords":chord_ids.iter().map(|id| {let n=nodes[id];json!({"id":id,"kind":n["kind"],"bounds":bounds[id],"voice":n["voice"],"measure_number":n["measure_number"]})}).collect::<Vec<_>>()
            }));
    }
    Ok(json!(rows))
}

/// AbstractChord/RestChord.getBeams: stable tail-to-head order at the chord's
/// center x. The first beam's actual incoming ensemble is getBeamGroup; do not
/// choose a group by registration ID or by which group lists this chord first.
pub(crate) fn chord_beams(snapshot: &Value, id: usize) -> Result<Vec<usize>, String> {
    let nodes: std::collections::BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let chord = nodes.get(&id).ok_or("music_voice_live_chord_required")?;
    let edges = snapshot["relations"].as_array().unwrap();
    let mut beams = Vec::new();
    let reference_y = match chord["kind"].as_str() {
        Some("HeadChordInter") => {
            let stems = edges
                .iter()
                .filter(|e| e["kind"] == "ChordStemRelation" && e["source"] == id)
                .map(|e| e["target"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            if stems.len() > 1 {
                return Err("music_voice_single_chord_stem_required".into());
            }
            if let Some(stem) = stems.first() {
                beams.extend(
                    edges
                        .iter()
                        .filter(|e| e["kind"] == "BeamStemRelation" && e["target"] == *stem)
                        .map(|e| e["source"].as_u64().unwrap() as usize),
                );
            }
            chord["head_location"][1].as_f64()
        }
        Some("RestChordInter") => {
            let mut members = edges
                .iter()
                .filter(|e| e["kind"] == "Containment" && e["source"] == id)
                .map(|e| e["target"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            // Original returns the empty cached list before asking for center
            // when this live rest ensemble currently has no notes.
            if members.is_empty() {
                return Ok(beams);
            }
            // getMembers is stable bottom-up; RestChord uses its first note.
            members.sort_by_key(|id| {
                std::cmp::Reverse(
                    nodes[id]["bounds"][1].as_i64().unwrap()
                        + nodes[id]["bounds"][3].as_i64().unwrap() / 2,
                )
            });
            if let Some(rest) = members.first() {
                if nodes[rest]["kind"] != "RestInter" {
                    return Err("music_group_typed_owned_rest_required".into());
                }
                beams.extend(
                    edges
                        .iter()
                        .filter(|e| e["kind"] == "BeamRestRelation" && e["target"] == *rest)
                        .map(|e| e["source"].as_u64().unwrap() as usize),
                );
            }
            Some(
                chord["bounds"][1]
                    .as_i64()
                    .ok_or("music_voice_chord_bounds_required")? as f64
                    + (chord["bounds"][3]
                        .as_i64()
                        .ok_or("music_voice_chord_bounds_required")?
                        / 2) as f64,
            )
        }
        _ => return Err("music_voice_ordinary_chord_required".into()),
    };
    if let Some(y) = reference_y {
        let x = chord["bounds"][0]
            .as_i64()
            .ok_or("music_voice_chord_bounds_required")? as f64
            + (chord["bounds"][2]
                .as_i64()
                .ok_or("music_voice_chord_bounds_required")?
                / 2) as f64;
        let mut distances = std::collections::BTreeMap::new();
        for id in &beams {
            let beam = nodes.get(id).ok_or("music_group_live_beam_required")?;
            if !matches!(beam["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                || beam["origin"]["system"] != chord["origin"]["system"]
            {
                return Err("music_group_typed_owned_beam_required".into());
            }
            let median: [f64; 4] = serde_json::from_value(beam["median"].clone())
                .map_err(|_| "music_voice_beam_median_required")?;
            if !median.iter().all(|n| n.is_finite()) || median[0] == median[2] {
                return Err("music_voice_horizontal_beam_median_required".into());
            }
            let at_x =
                median[1] + (x - median[0]) * (median[3] - median[1]) / (median[2] - median[0]);
            distances.insert(*id, (y - at_x).abs());
        }
        beams.sort_by(|a, b| distances[b].total_cmp(&distances[a]));
    }
    Ok(beams)
}
pub(crate) fn chord_group(snapshot: &Value, id: usize) -> Result<Option<usize>, String> {
    let beams = chord_beams(snapshot, id)?;
    Ok(beams.first().and_then(|beam| {
        snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "Containment" && e["target"] == *beam)
            .map(|e| e["source"].as_u64().unwrap() as usize)
    }))
}

/// AbstractChord.getFollowingTiedChords follows typed SlurHead relations,
/// already-classified tie state, LEFT/RIGHT heads and the actual right head's
/// first ensemble; it does not classify slur geometry or export ties.
pub(crate) fn following_tied_chords(snapshot: &Value, id: usize) -> Result<Vec<usize>, String> {
    let nodes: std::collections::BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let chord = nodes.get(&id).ok_or("music_voice_live_chord_required")?;
    let edges = snapshot["relations"].as_array().unwrap();
    let mut members = edges
        .iter()
        .filter(|e| e["kind"] == "Containment" && e["source"] == id)
        .map(|e| e["target"].as_u64().unwrap() as usize)
        .collect::<Vec<_>>();
    members.sort_by_key(|id| {
        std::cmp::Reverse(
            nodes[id]["bounds"][1].as_i64().unwrap() + nodes[id]["bounds"][3].as_i64().unwrap() / 2,
        )
    });
    let mut following = Vec::new();
    for note in members {
        for relation in edges
            .iter()
            .filter(|e| e["kind"] == "SlurHeadRelation" && e["target"] == note)
        {
            if nodes[&note]["kind"] != "HeadInter" {
                return Err("music_voice_typed_slur_head_required".into());
            }
            let slur_id = relation["source"].as_u64().unwrap() as usize;
            let slur = nodes
                .get(&slur_id)
                .ok_or("music_voice_live_slur_required")?;
            if slur["kind"] != "SlurInter" || slur["origin"]["system"] != chord["origin"]["system"]
            {
                return Err("music_voice_typed_owned_slur_required".into());
            }
            let head = |side: &str| {
                edges
                    .iter()
                    .find(|e| {
                        e["kind"] == "SlurHeadRelation"
                            && e["source"] == slur_id
                            && e["side"] == side
                    })
                    .map(|e| e["target"].as_u64().unwrap() as usize)
            };
            let left = head("LEFT");
            let right = head("RIGHT");
            if slur["is_tie"] != true || left != Some(note) {
                continue;
            }
            let Some(right) = right else {
                continue;
            };
            let right_note = nodes
                .get(&right)
                .ok_or("music_voice_live_slur_head_required")?;
            if right_note["kind"] != "HeadInter"
                || right_note["origin"]["system"] != chord["origin"]["system"]
            {
                return Err("music_voice_typed_slur_head_required".into());
            }
            let parent = edges
                .iter()
                .find(|e| e["kind"] == "Containment" && e["target"] == right)
                .ok_or("music_voice_tied_head_ensemble_required")?;
            let other = parent["source"].as_u64().unwrap() as usize;
            if nodes[&other]["kind"] != "HeadChordInter" {
                return Err("music_voice_ordinary_tied_chord_required".into());
            }
            following.push(other);
        }
    }
    // Original retains multiple tied head references, then stably orders by
    // left abscissa. Chord.setVoice's actual identity guard terminates repeats.
    following.sort_by_key(|id| nodes[id]["bounds"][0].as_i64().unwrap());
    Ok(following)
}

/// AbstractChord.setVoice -> Voice.addChord -> BeamGroup.setVoice for the
/// ordinary chord family, including already-classified following ties.
/// Identity is the retained Voice and its Measure, not a repeating numeric
/// label. Special chord ownership keeps its explicit pending guard.
pub(crate) fn assign_chord_voice(
    graph: &mut crate::shared_sig::Graph,
    voices: &mut Vec<Voice>,
    measures: &[Measure],
    chord_id: usize,
    voice_index: usize,
) -> Result<(), String> {
    let voice = voices
        .get(voice_index)
        .ok_or("music_voice_owner_required")?;
    let measure = measures
        .get(voice.measure)
        .ok_or("music_voice_measure_required")?;
    let belongs = |n: &Value| {
        n["origin"]["system"] == measure.system
            && n["part_id"] == measure.part
            && n["measure_index"] == measure.index
    };
    let snapshot = graph.snapshot();
    let nodes: std::collections::BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    if !nodes.get(&chord_id).is_some_and(|n| belongs(n)) {
        return Err("music_voice_owned_measure_chord_required".into());
    }
    let groups = beam_groups(graph, voices, measures)?;
    let mut staged = graph.clone();
    let mut retained = voices.clone();
    let mut pending = vec![chord_id];
    while let Some(id) = pending.pop() {
        let original = nodes.get(&id).ok_or("music_voice_live_chord_required")?;
        if !matches!(
            original["kind"].as_str(),
            Some("HeadChordInter" | "RestChordInter")
        ) {
            return Err("music_voice_ordinary_chord_required".into());
        }
        let key: crate::shared_sig::Key = serde_json::from_value(original["origin"].clone())
            .map_err(|_| "music_voice_chord_origin_required")?;
        let current = staged.snapshot();
        let mut chord = current["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("music_voice_live_chord_required")?
            .clone();
        // Reassignment and renaming can leave overlapping old member lists.
        // Only the actual Voice reference terminates recursive assignment.
        if staged.chord_has_voice(key, voice) {
            continue;
        }
        let members = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == id)
            .map(|e| e["target"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        if members.is_empty()
            || members.iter().any(|member| {
                nodes.get(member).is_none_or(|n| {
                    n["kind"] == "RestInter"
                        && !matches!(
                            n["shape"].as_str(),
                            Some(
                                "HALF_REST"
                                    | "QUARTER_REST"
                                    | "EIGHTH_REST"
                                    | "ONE_16TH_REST"
                                    | "ONE_32ND_REST"
                                    | "ONE_64TH_REST"
                                    | "ONE_128TH_REST"
                            )
                        )
                        && !(voice.measure_rest_chord == Some(id)
                            && n["shape"]
                                .as_str()
                                .is_some_and(|s| is_measure_rest_shape(s, false)))
                })
            })
        {
            return Err("music_voice_measure_rest_special_chord_pending".into());
        }
        chord["voice"] = json!(voice.id);
        staged.vertex(key, chord, false)?;
        staged.bind_chord_to_live_voice(key, voice)?;
        let owned = &mut retained[voice_index];
        if owned.measure_rest_chord != Some(id) && !owned.chord_ids.contains(&id) {
            owned.chords_initialized = true;
            owned.chord_ids.push(id);
            owned.chord_ids.sort_by_key(|id| {
                nodes[id]["bounds"][0].as_i64().unwrap()
                    + nodes[id]["bounds"][2].as_i64().unwrap() / 2
            });
        }
        // Stack the following ties first so the original group recursion is
        // visited before them. Both propagate only within this exact Measure.
        for other in following_tied_chords(&snapshot, id)?.into_iter().rev() {
            if belongs(nodes[&other]) {
                pending.push(other);
            }
        }
        let selected = chord_group(&snapshot, id)?;
        if let Some(group_id) = selected {
            let group = groups
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["id"] == group_id)
                .ok_or("music_voice_live_beam_group_required")?;
            // Same Measure object in original; native ownership is the exact
            // system/part/local measure tuple. Never copy Voice1 across it.
            for member in group["chords"].as_array().unwrap().iter().rev() {
                let other = member["id"].as_u64().unwrap() as usize;
                if belongs(nodes[&other]) {
                    pending.push(other);
                }
            }
        }
    }
    *graph = staged;
    *voices = retained;
    Ok(())
}

#[cfg(test)]
mod group_tests {
    use super::*;
    use crate::{
        GrayImage,
        shared_sig::{Key, Phase},
    };
    fn comparable(rows: Value) -> Value {
        json!(rows.as_array().unwrap().iter().map(|g| json!({
            "system_id":g["system_id"],"bounds":g["bounds"],"voice":g["voice"],"voice_measure_number":g["voice_measure_number"],
            "beams":g["beams"].as_array().unwrap().iter().map(|b|json!({"kind":b["kind"],"bounds":b["bounds"]})).collect::<Vec<_>>(),
            "chords":g["chords"].as_array().unwrap().iter().map(|c|json!({"kind":c["kind"],"bounds":c["bounds"],"voice":c["voice"],"measure_number":c["measure_number"]})).collect::<Vec<_>>()
        })).collect::<Vec<_>>())
    }
    #[test]
    fn original_saved_group_getters_and_declared_beam_rest_follow_actual_ensemble() {
        let rests: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-measure-rest/original-voices.json"
        ))
        .unwrap();
        for state in rests["measure_rest_shapes"].as_array().unwrap() {
            for shape in state["shapes"].as_array().unwrap() {
                assert_eq!(
                    is_measure_rest_shape(
                        shape["shape"].as_str().unwrap(),
                        state["partialWholeRests"].as_bool().unwrap()
                    ),
                    shape["measure_rest"].as_bool().unwrap()
                );
            }
        }
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/saved-page.json"
        ))
        .unwrap();
        let rest_control: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/controlled-rest.json"
        ))
        .unwrap();
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        let mut export = crate::music_xml::recognize(&image).unwrap();
        let owner = &mut export.owner;
        assert_eq!(
            comparable(owner.owner.prepared.report["beam_group_voices"].clone()),
            original["groups"]
        );
        let geometry: Value = serde_json::from_str(include_str!(
            "../fixtures/official-slot-geometry/saved-page.json"
        ))
        .unwrap();
        for (measure, original) in owner
            .measures
            .iter()
            .zip(geometry["measures"].as_array().unwrap())
        {
            let slots = json!(measure.timing.slots.iter().map(|s|json!({"id":s.id,"time_offset":s.time_offset.map(crate::music_time::format_fraction),"x_offset":s.x_offset})).collect::<Vec<_>>());
            assert_eq!(slots, original["slots"]);
            for query in original["queries"].as_array().unwrap() {
                let result = owner
                    .closest_slot_at(
                        original["number"].as_u64().unwrap() as usize,
                        query["point"][0].as_f64().unwrap(),
                    )
                    .unwrap();
                assert_eq!(result["slot"]["id"], query["slot_id"]);
            }
        }
        assert!(owner.closest_slot_at(0, 1.).is_err());
        assert!(owner.closest_slot_at(5, 1.).is_err());
        assert!(owner.closest_slot_at(1, f64::NAN).is_err());
        assert!(closest_slot(&owner.measures[0].timing.slots, f64::MAX).is_none());
        let stem_directions: Value = serde_json::from_str(include_str!(
            "../fixtures/official-chord-stem-direction/saved-pages.json"
        ))
        .unwrap();
        let reference = stem_directions["pages"][0]["chords"].as_array().unwrap();
        let snapshot = owner.owner.graph.snapshot();
        let events = owner.owner.prepared.report["events"].as_array().unwrap();
        assert_eq!(events.len(), reference.len());
        for reference in reference {
            let event =
                events
                    .iter()
                    .find(|e| {
                        e["system_id"] == reference["system_id"]
                            && e["measure_number"] == reference["measure_number"]
                            && snapshot["inters"].as_array().unwrap().iter().any(|n| {
                                n["id"] == e["chord_id"] && n["bounds"] == reference["bounds"]
                            })
                    })
                    .unwrap();
            assert_eq!(
                event["chord_stem_direction"],
                reference["stem_direction_code"]
            );
        }
        let reset_projection =
            |g: &crate::shared_sig::Graph,
             v: &[Voice],
             measure: &Measure,
             mi: usize,
             chords: &[crate::music_measures::Chord]| {
                let snapshot = g.snapshot();
                let mut chords = chords
                    .iter()
                    .filter(|c| c.measure == mi)
                    .collect::<Vec<_>>();
                chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
                json!({"abnormal":measure.timing.abnormal,"voice_count":v.iter().filter(|v|v.measure==mi).count(),"stack_slot_count":measure.timing.slots.len(),"expected":measure.timing.expected.map(crate::music_time::format_fraction),"actual":measure.timing.actual.map(crate::music_time::format_fraction),
                "chords":chords.into_iter().map(|c|{let n=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==c.id).unwrap();json!({"bounds":n["bounds"],"duration":n["duration"],"time_offset":n["time_offset"],"voice":n["voice"],"slot_id":n["slot_id"]})}).collect::<Vec<_>>()})
            };
        let reset_controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rhythm-reset/normal.json"
        ))
        .unwrap();
        let mut reset_graph = owner.owner.graph.clone();
        let mut reset_voices = owner.voices.clone();
        let mut reset_measure = owner.measures[0].clone();
        reset_measure.timing.abnormal = true;
        assert_eq!(
            reset_projection(
                &reset_graph,
                &reset_voices,
                &reset_measure,
                0,
                &owner.chords
            ),
            reset_controls["before"]
        );
        let other_nodes = reset_graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                owner
                    .chords
                    .iter()
                    .any(|c| c.measure != 0 && n["id"] == c.id)
            })
            .cloned()
            .collect::<Vec<_>>();
        let other_voices = projection(
            &reset_voices
                .iter()
                .filter(|v| v.measure != 0)
                .cloned()
                .collect::<Vec<_>>(),
            &owner.measures,
        );
        for stage in ["after", "repeated"] {
            reset_rhythm(
                &mut reset_graph,
                &mut reset_voices,
                &mut reset_measure,
                &owner.chords,
                0,
            )
            .unwrap();
            assert_eq!(
                reset_projection(
                    &reset_graph,
                    &reset_voices,
                    &reset_measure,
                    0,
                    &owner.chords
                ),
                reset_controls[stage]
            );
            let snapshot = reset_graph.snapshot();
            assert!(
                other_nodes
                    .iter()
                    .all(|node| snapshot["inters"].as_array().unwrap().contains(node))
            );
            assert_eq!(projection(&reset_voices, &owner.measures), other_voices);
        }
        let whole = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-measure-rest/quarter-whole-rest.png"
        ))
        .unwrap();
        let mut whole = crate::music_xml::recognize(&whole).unwrap();
        let whole = &mut whole.owner;
        let reset_controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rhythm-reset/measure-rest.json"
        ))
        .unwrap();
        whole.measures[1].timing.abnormal = true;
        assert_eq!(
            reset_projection(
                &whole.owner.graph,
                &whole.voices,
                &whole.measures[1],
                1,
                &whole.chords
            ),
            reset_controls["before"]
        );
        for stage in ["after", "repeated"] {
            reset_rhythm(
                &mut whole.owner.graph,
                &mut whole.voices,
                &mut whole.measures[1],
                &whole.chords,
                1,
            )
            .unwrap();
            assert_eq!(
                reset_projection(
                    &whole.owner.graph,
                    &whole.voices,
                    &whole.measures[1],
                    1,
                    &whole.chords
                ),
                reset_controls[stage]
            );
        }
        // Original final stack-slot collection consumes already solved voices;
        // this is downstream of, and does not pretend to solve, polyphony.
        let slot_control: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stack-slots/controls.json"
        ))
        .unwrap();
        let mut slot_graph = owner.owner.graph.clone();
        let mut slot_voice = vec![owner.voices[0].clone()];
        let mut slot_measure = owner.measures[0].clone();
        let slot_state = |g: &crate::shared_sig::Graph, voices: &[Voice], m: &Measure| {
            let snap = g.snapshot();
            let bounds = |id: usize| {
                snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap()["bounds"]
                    .clone()
            };
            json!({"abnormal":m.timing.abnormal,
                "slots":m.timing.slots.iter().map(|s|json!({"id":s.id,"time_offset":s.time_offset.map(crate::music_time::format_fraction),"incoming_bounds":s.incoming_ids.iter().map(|&id|bounds(id)).collect::<Vec<_>>()})).collect::<Vec<_>>(),
                "voices":voices.iter().map(|v|json!({"id":v.id,"slots":m.timing.slots.iter().filter_map(|s|v.slots.get(&s.id).map(|info|json!({"slot_id":s.id,"status":if info.continuing {"CONTINUE"}else{"BEGIN"},"chord_bounds":bounds(info.chord_id)}))).collect::<Vec<_>>()})).collect::<Vec<_>>()})
        };
        generate_slots(&mut slot_graph, &mut slot_voice, &mut slot_measure, 0).unwrap();
        assert_eq!(
            slot_state(&slot_graph, &slot_voice, &slot_measure),
            slot_control["saved"]
        );
        for slot in &slot_measure.timing.slots {
            for id in &slot.incoming_ids {
                assert_eq!(
                    slot_graph.snapshot()["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == *id)
                        .unwrap()["slot_id"],
                    slot.id
                );
            }
        }
        let last_id = slot_voice[0].chord_ids.pop().unwrap();
        let mut second = slot_voice[0].clone();
        second.id = 2;
        second.chord_ids = vec![last_id];
        second.slots.clear();
        slot_voice.push(second);
        let mut last_node = slot_graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == last_id)
            .unwrap()
            .clone();
        let last_key: Key = serde_json::from_value(last_node["origin"].clone()).unwrap();
        for (name, offset) in [
            ("continued", json!("9/16")),
            ("boundary", json!("3/4")),
            ("missing", Value::Null),
        ] {
            last_node["time_offset"] = offset;
            last_node["voice"] = json!(2);
            slot_graph
                .vertex(last_key, last_node.clone(), false)
                .unwrap();
            for v in &mut slot_voice {
                v.slots.clear();
            }
            generate_slots(&mut slot_graph, &mut slot_voice, &mut slot_measure, 0).unwrap();
            assert_eq!(
                slot_state(&slot_graph, &slot_voice, &slot_measure),
                slot_control[name]
            );
        }
        // Invalid clocks retain every staged owner/graph field.
        last_node["time_offset"] = json!("1/0");
        slot_graph.vertex(last_key, last_node, false).unwrap();
        let before_slots = slot_state(&slot_graph, &slot_voice, &slot_measure);
        let before_graph = slot_graph.snapshot();
        assert!(generate_slots(&mut slot_graph, &mut slot_voice, &mut slot_measure, 0).is_err());
        assert_eq!(
            slot_state(&slot_graph, &slot_voice, &slot_measure),
            before_slots
        );
        assert_eq!(slot_graph.snapshot(), before_graph);
        let duration_controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-slot-duration/controls.json"
        ))
        .unwrap();
        let mut duration_graph = owner.owner.graph.clone();
        let mut duration_voices = vec![owner.voices[0].clone()];
        let mut duration_measure = owner.measures[0].clone();
        let duration_projection = |g: &crate::shared_sig::Graph, v: &[Voice], m: &Measure| {
            json!({"voices":v.iter().map(|voice|json!({"id":voice.id,"duration":slot_duration(g,voice,&m.timing.slots).unwrap().map(crate::music_time::format_fraction).unwrap_or_else(||"null".into())})).collect::<Vec<_>>(),
            "slots_duration":crate::music_time::format_fraction(stack_slots_duration(g,&m.timing.slots).unwrap()),
            "actual_duration":m.timing.actual.map(crate::music_time::format_fraction).unwrap_or_else(||"null".into())})
        };
        read_stack_actual_duration(&duration_graph, &duration_voices, &mut duration_measure, 0)
            .unwrap();
        assert_eq!(
            duration_projection(&duration_graph, &duration_voices, &duration_measure),
            duration_controls["saved"]
        );
        let first_id = duration_voices[0].chord_ids[0];
        let mut first_node = duration_graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == first_id)
            .unwrap()
            .clone();
        let first_key: Key = serde_json::from_value(first_node["origin"].clone()).unwrap();
        first_node["time_offset"] = json!("1");
        duration_graph
            .vertex(first_key, first_node.clone(), false)
            .unwrap();
        for v in &mut duration_voices {
            v.slots.clear();
        }
        generate_slots(
            &mut duration_graph,
            &mut duration_voices,
            &mut duration_measure,
            0,
        )
        .unwrap();
        read_stack_actual_duration(&duration_graph, &duration_voices, &mut duration_measure, 0)
            .unwrap();
        assert_eq!(
            last_end_time(&duration_graph, &duration_voices[0]).unwrap(),
            Some(Fraction::from_ticks(16))
        );
        assert_eq!(
            duration_projection(&duration_graph, &duration_voices, &duration_measure),
            duration_controls["maximum"]
        );
        first_node["time_offset"] = json!("0");
        duration_graph.vertex(first_key, first_node, false).unwrap();
        for v in &mut duration_voices {
            v.slots.clear();
        }
        generate_slots(
            &mut duration_graph,
            &mut duration_voices,
            &mut duration_measure,
            0,
        )
        .unwrap();
        duration_measure.timing.slots[0].time_offset = None;
        read_stack_actual_duration(&duration_graph, &duration_voices, &mut duration_measure, 0)
            .unwrap();
        assert_eq!(
            duration_projection(&duration_graph, &duration_voices, &duration_measure),
            duration_controls["missing_slot_time"]
        );
        duration_measure.timing.slots.clear();
        read_stack_actual_duration(&duration_graph, &duration_voices, &mut duration_measure, 0)
            .unwrap();
        assert_eq!(
            duration_projection(&duration_graph, &duration_voices, &duration_measure),
            duration_controls["no_slots_with_voice"]
        );
        duration_voices.clear();
        read_stack_actual_duration(&duration_graph, &duration_voices, &mut duration_measure, 0)
            .unwrap();
        assert_eq!(
            duration_projection(&duration_graph, &duration_voices, &duration_measure),
            duration_controls["no_voices"]
        );
        let time_controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-chord-time/controls.json"
        ))
        .unwrap();
        let chord_ids = owner.voices[0].chord_ids.clone();
        let other_id = owner.voices[1].chord_ids[0];
        for control in time_controls["controls"].as_array().unwrap() {
            let mode = control["mode"].as_str().unwrap();
            let mut graph = owner.owner.graph.clone();
            let mut nodes = graph.snapshot()["inters"].as_array().unwrap().clone();
            for n in nodes
                .iter_mut()
                .filter(|n| chord_ids.iter().any(|&id| n["id"] == id))
            {
                n["time_offset"] = Value::Null;
                let key: Key = serde_json::from_value(n["origin"].clone()).unwrap();
                graph.vertex(key, n.clone(), false).unwrap();
            }
            let key = |id: usize| -> Key {
                serde_json::from_value(
                    nodes.iter().find(|n| n["id"] == id).unwrap()["origin"].clone(),
                )
                .unwrap()
            };
            let next = |g: &mut crate::shared_sig::Graph, a, b| {
                g.relations(
                    1,
                    Phase::Chord,
                    vec![(key(a), key(b), json!({"kind":"NextInVoiceRelation"}))],
                )
                .unwrap();
            };
            match mode {
                "sequence" => {
                    next(&mut graph, chord_ids[3], chord_ids[4]);
                    next(&mut graph, chord_ids[4], chord_ids[5]);
                }
                "tie" => {
                    let slur = Key {
                        system: 1,
                        phase: Phase::Symbol,
                        local: nodes
                            .iter()
                            .filter(|n| {
                                n["origin"]["system"] == 1
                                    && n["origin"]["phase"]
                                        == serde_json::to_value(Phase::Symbol).unwrap()
                            })
                            .filter_map(|n| n["origin"]["local"].as_u64())
                            .max()
                            .unwrap_or(0) as usize
                            + 1,
                    };
                    graph.vertex(slur,json!({"kind":"SlurInter","shape":"SLUR_ABOVE","grade":0.8,"is_tie":true}),false).unwrap();
                    let note = |id| -> Key {
                        let target = graph.snapshot()["relations"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|e| e["kind"] == "Containment" && e["source"] == id)
                            .unwrap()["target"]
                            .as_u64()
                            .unwrap() as usize;
                        key(target)
                    };
                    let left = note(chord_ids[3]);
                    let right = note(chord_ids[4]);
                    graph
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
                    next(&mut graph, chord_ids[4], chord_ids[5]);
                }
                "cross-measure" => next(&mut graph, chord_ids[3], other_id),
                _ => {}
            }
            crate::music_chord_time::set_and_push_ticks(
                &mut graph,
                &owner.voices,
                &owner.measures,
                chord_ids[0],
                if mode == "null" { None } else { Some(0) },
            )
            .unwrap();
            let actual = graph.snapshot();
            let n = |id| {
                actual["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap()
            };
            assert_eq!(
                json!({"mode":mode,"chords":chord_ids.iter().map(|&id|json!({"bounds":n(id)["bounds"],"time_offset":n(id)["time_offset"]})).collect::<Vec<_>>(),"other_measure_offset":n(other_id)["time_offset"]}),
                *control
            );
        }
        let mut bad_clock = owner.owner.graph.clone();
        let mut first = bad_clock.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == chord_ids[0])
            .unwrap()
            .clone();
        let first_key: Key = serde_json::from_value(first["origin"].clone()).unwrap();
        first["duration"] = json!("1/32");
        bad_clock.vertex(first_key, first, false).unwrap();
        let before = bad_clock.snapshot();
        crate::music_chord_time::set_and_push_ticks(
            &mut bad_clock,
            &owner.voices,
            &owner.measures,
            chord_ids[0],
            Some(0),
        )
        .unwrap();
        // getEndTime reads the live duration, never a forged report/cache value.
        assert_eq!(
            bad_clock.chord_duration(first_key).unwrap(),
            owner.owner.graph.chord_duration(first_key).unwrap()
        );
        assert_eq!(bad_clock.snapshot()["inters"], before["inters"]);
        let timed_page_graph = owner.owner.graph.clone();
        let snapshot = owner.owner.graph.snapshot();
        let before_cache = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "StemInter" && n["origin"]["system"] == 1)
            .map(|n| {
                let key: Key = serde_json::from_value(n["origin"].clone()).unwrap();
                let stem = owner.owner.caches[&1]
                    .registered_stem(&owner.owner.graph, key.local)
                    .unwrap();
                let state = stem.snapshot().unwrap();
                (key, stem, state)
            })
            .collect::<Vec<_>>();
        let before_registry = format!("{:?}", owner.owner.prepared.registry);
        let beam: Key = serde_json::from_value(
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "BeamInter" && n["origin"]["system"] == 1)
                .unwrap()["origin"]
                .clone(),
        )
        .unwrap();
        let next_local = |phase: Phase| {
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
                .filter(|k| k.system == 1 && k.phase == phase)
                .map(|k| k.local)
                .max()
                .unwrap_or(0)
                + 1
        };
        let rest = Key {
            system: 1,
            phase: Phase::Symbol,
            local: next_local(Phase::Symbol),
        };
        let chord = Key {
            system: 1,
            phase: Phase::Chord,
            local: next_local(Phase::Chord),
        };
        let original_rest = rest_control["groups"][0]["chords"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == "RestChordInter")
            .unwrap();
        owner.owner.graph.vertex(rest,json!({"kind":"RestInter","shape":"QUARTER_REST","bounds":original_rest["bounds"],"grade":0.8,"staff_id":1}),false).unwrap();
        owner.owner.graph.vertex(chord,json!({"kind":"RestChordInter","staff_id":1,"part_id":1,"measure_index":0,"measure_number":1,"voice":null}),false).unwrap();
        owner
            .owner
            .graph
            .relations(
                1,
                Phase::Chord,
                vec![(chord, rest, json!({"kind":"Containment"}))],
            )
            .unwrap();
        owner
            .owner
            .graph
            .relations(
                1,
                Phase::Symbol,
                vec![(beam, rest, json!({"kind":"BeamRestRelation"}))],
            )
            .unwrap();
        let rows = beam_groups(&owner.owner.graph, &owner.voices, &owner.measures).unwrap();
        assert_eq!(comparable(rows), rest_control["groups"]);
        let propagation: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/propagation.json"
        ))
        .unwrap();
        let original_graph = owner.owner.graph.clone();
        let original_voices = owner.voices.clone();
        let first_id = original["groups"][0]["chords"][0]["bounds"].clone();
        let first_id = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == first_id)
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        let rest_id = original_graph.id(chord).unwrap();
        let voice_index = original_voices
            .iter()
            .position(|v| v.chord_ids.contains(&first_id))
            .unwrap();
        let voice_rows = |g: &crate::shared_sig::Graph, v: &Voice| {
            let snapshot = g.snapshot();
            json!(
                v.chord_ids
                    .iter()
                    .map(|id| {
                        let n = snapshot["inters"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|n| n["id"] == *id)
                            .unwrap();
                        json!({"kind":n["kind"],"bounds":n["bounds"]})
                    })
                    .collect::<Vec<_>>()
            )
        };
        for control in propagation["controls"].as_array().unwrap() {
            let mut graph = original_graph.clone();
            let mut voices = original_voices.clone();
            let mut rest_data = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == rest_id)
                .unwrap()
                .clone();
            rest_data["measure_index"] = json!(
                owner.measures[control["rest_measure_number"].as_u64().unwrap() as usize - 1].index
            );
            rest_data["measure_number"] = control["rest_measure_number"].clone();
            graph.vertex(chord, rest_data, false).unwrap();
            assert_eq!(
                voice_rows(&graph, &voices[voice_index]),
                control["voice_chords_before"]
            );
            // Direct BeamGroup.setVoice is equivalent here to its first-chord
            // seed assignment: deliberately unassign that stored field only.
            let first_key: Key = serde_json::from_value(
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == first_id)
                    .unwrap()["origin"]
                    .clone(),
            )
            .unwrap();
            let mut first_data = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == first_id)
                .unwrap()
                .clone();
            first_data["voice"] = Value::Null;
            graph.vertex(first_key, first_data, false).unwrap();
            assign_chord_voice(
                &mut graph,
                &mut voices,
                &owner.measures,
                first_id,
                voice_index,
            )
            .unwrap();
            assert_eq!(
                voice_rows(&graph, &voices[voice_index]),
                control["voice_chords_after"]
            );
            assert_eq!(
                graph.snapshot()["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == rest_id)
                    .unwrap()["voice"],
                control["rest_voice"]
            );
            let before = graph.snapshot();
            let before_members = voices[voice_index].chord_ids.clone();
            assign_chord_voice(
                &mut graph,
                &mut voices,
                &owner.measures,
                first_id,
                voice_index,
            )
            .unwrap();
            assert_eq!(graph.snapshot(), before);
            assert_eq!(voices[voice_index].chord_ids, before_members);
        }
        let order_controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-group-voices/beam-order.json"
        ))
        .unwrap();
        for control in order_controls["controls"].as_array().unwrap() {
            let mut graph = original_graph.clone();
            let mut voices = original_voices.clone();
            let y = control["added_median_y"].as_f64().unwrap();
            let added = Key {
                system: 1,
                phase: Phase::Beam,
                local: next_local(Phase::Beam),
            };
            let group = Key {
                system: 1,
                phase: Phase::Group,
                local: next_local(Phase::Group),
            };
            let added_id=graph.vertex(added,json!({"kind":"BeamInter","shape":"BEAM","grade":0.8,"bounds":[288,y as i64-2,32,4],"median":[288.,y,320.,y],"height":4.}),false).unwrap();
            let group_id = graph
                .vertex(group, json!({"kind":"BeamGroupInter"}), false)
                .unwrap();
            let stem_id = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["kind"] == "ChordStemRelation" && e["source"] == first_id)
                .unwrap()["target"]
                .as_u64()
                .unwrap() as usize;
            let stem_key: Key = serde_json::from_value(
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == stem_id)
                    .unwrap()["origin"]
                    .clone(),
            )
            .unwrap();
            graph
                .relations(
                    1,
                    Phase::Group,
                    vec![(group, added, json!({"kind":"Containment"}))],
                )
                .unwrap();
            graph
                .relations(
                    1,
                    Phase::Beam,
                    vec![(
                        added,
                        stem_key,
                        json!({"kind":"BeamStemRelation","beam_portion":"LEFT"}),
                    )],
                )
                .unwrap();
            // The getter-only earlier control used no rhythmic owner. Here the
            // declared rest is registered in the Voice's actual first Measure.
            let mut rest_data = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == rest_id)
                .unwrap()
                .clone();
            rest_data["measure_index"] = json!(owner.measures[voices[voice_index].measure].index);
            graph.vertex(chord, rest_data, false).unwrap();
            let order = chord_beams(&graph.snapshot(), first_id).unwrap();
            assert_eq!(
                order.len(),
                control["ordered_beams"].as_array().unwrap().len()
            );
            assert_eq!(
                order[0] == added_id,
                control["selected_added_group"].as_bool().unwrap()
            );
            assert_eq!(
                chord_group(&graph.snapshot(), first_id).unwrap() == Some(group_id),
                control["selected_added_group"].as_bool().unwrap()
            );
            let first_key: Key = serde_json::from_value(
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == first_id)
                    .unwrap()["origin"]
                    .clone(),
            )
            .unwrap();
            let mut first_data = graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == first_id)
                .unwrap()
                .clone();
            first_data["voice"] = Value::Null;
            graph.vertex(first_key, first_data, false).unwrap();
            assign_chord_voice(
                &mut graph,
                &mut voices,
                &owner.measures,
                first_id,
                voice_index,
            )
            .unwrap();
            assert_eq!(
                graph.snapshot()["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == rest_id)
                    .unwrap()["voice"],
                control["rest_voice"]
            );
        }
        let ties: Value = serde_json::from_str(include_str!(
            "../fixtures/official-voice-tie-dependency/controls.json"
        ))
        .unwrap();
        for control in ties["controls"].as_array().unwrap() {
            let mut graph = original_graph.clone();
            let mut voices = original_voices.clone();
            let find_chord = |bounds: &Value| {
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == *bounds)
                    .unwrap()
                    .clone()
            };
            let mut seed = find_chord(&control["seed_bounds"]);
            let mut target = find_chord(&control["target_bounds"]);
            let seed_id = seed["id"].as_u64().unwrap() as usize;
            let target_id = target["id"].as_u64().unwrap() as usize;
            let owned_voice = voices
                .iter()
                .position(|v| v.chord_ids.contains(&seed_id))
                .unwrap();
            let note_key = |id: usize| {
                let note = snapshot["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["kind"] == "Containment" && e["source"] == id)
                    .unwrap()["target"]
                    .clone();
                serde_json::from_value::<Key>(
                    snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == note)
                        .unwrap()["origin"]
                        .clone(),
                )
                .unwrap()
            };
            let slur = Key {
                system: 1,
                phase: Phase::Symbol,
                local: rest.local + 1,
            };
            graph.vertex(slur,json!({"kind":"SlurInter","shape":"SLUR_ABOVE","grade":0.8,"is_tie":control["is_tie"]}),false).unwrap();
            let reverse = control["mode"] == "source-on-right";
            graph.relations(1,Phase::Symbol,vec![
                (slur,note_key(seed_id),json!({"kind":"SlurHeadRelation","side":if reverse{"RIGHT"}else{"LEFT"}})),
                (slur,note_key(target_id),json!({"kind":"SlurHeadRelation","side":if reverse{"LEFT"}else{"RIGHT"}}))
            ]).unwrap();
            let found = following_tied_chords(&graph.snapshot(), seed_id).unwrap();
            let rows = json!(found.iter().map(|id|{
                let n=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==*id).unwrap();
                json!({"kind":n["kind"],"bounds":n["bounds"],"measure_number":n["measure_number"]})
            }).collect::<Vec<_>>());
            assert_eq!(rows, control["following"]);
            let seed_key: Key = serde_json::from_value(seed["origin"].clone()).unwrap();
            let target_key: Key = serde_json::from_value(target["origin"].clone()).unwrap();
            seed["voice"] = Value::Null;
            target["voice"] = Value::Null;
            graph.vertex(seed_key, seed, false).unwrap();
            graph.vertex(target_key, target, false).unwrap();
            assign_chord_voice(
                &mut graph,
                &mut voices,
                &owner.measures,
                seed_id,
                owned_voice,
            )
            .unwrap();
            assert_eq!(
                graph.snapshot()["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == target_id)
                    .unwrap()["voice"],
                control["target_voice"]
            );
        }
        let timing: Value = serde_json::from_str(include_str!(
            "../fixtures/official-voice-end-time/controls.json"
        ))
        .unwrap();
        owner.owner.graph = timed_page_graph;
        let timed_voice = owner.voices.iter().position(|v| v.measure == 0).unwrap();
        let timing_state = |graph: &crate::shared_sig::Graph, voice: &Voice| {
            let snapshot = graph.snapshot();
            let last = voice.chord_ids.last().unwrap();
            let n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == *last)
                .unwrap();
            json!({"chord_count":voice.chord_ids.len(),"last_bounds":n["bounds"],"last_duration":n["duration"],"last_offset":n["time_offset"].as_str().unwrap_or("null"),"end_time":last_end_time(graph,voice).unwrap().map(crate::music_time::format_fraction)})
        };
        assert_eq!(
            timing_state(&owner.owner.graph, &owner.voices[timed_voice]),
            timing["before"]
        );
        let last_id = *owner.voices[timed_voice].chord_ids.last().unwrap();
        let last_key: Key = serde_json::from_value(
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == last_id)
                .unwrap()["origin"]
                .clone(),
        )
        .unwrap();
        owner.owner.graph.remove_music_inter(last_key).unwrap();
        crate::music_containers::refresh(
            &mut owner.owner.graph,
            &owner.measures,
            &mut owner.chords,
            &mut owner.voices,
        )
        .unwrap();
        assert_eq!(
            timing_state(&owner.owner.graph, &owner.voices[timed_voice]),
            timing["after_last_removal"]
        );
        let duration: Value = serde_json::from_str(include_str!(
            "../fixtures/official-voice-duration/controls.json"
        ))
        .unwrap();
        let duration_state = |voice: &Voice, measure: &Measure| {
            json!({
            "termination":voice.termination.map(crate::music_time::format_delta),
            "voice_excess":voice.excess.map(crate::music_time::format_fraction),
            "measure_abnormal":measure.timing.abnormal,
            "stack_excess":measure.timing.excess.map(crate::music_time::format_fraction),
            "expected_duration":measure.timing.expected.map(crate::music_time::format_fraction)})
        };
        let mut shortened_voice = owner.voices[timed_voice].clone();
        let mut shortened_measure = owner.measures[shortened_voice.measure].clone();
        assert_eq!(
            duration_state(&shortened_voice, &shortened_measure),
            duration["complete"]
        );
        check_duration(
            &owner.owner.graph,
            &mut shortened_voice,
            &mut shortened_measure,
        )
        .unwrap();
        assert_eq!(
            duration_state(&shortened_voice, &shortened_measure),
            duration["shortened"]
        );
        let mut excess_graph = owner.owner.graph.clone();
        let end_id = *shortened_voice.chord_ids.last().unwrap();
        let mut end_node = excess_graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == end_id)
            .unwrap()
            .clone();
        let end_key: Key = serde_json::from_value(end_node["origin"].clone()).unwrap();
        end_node["time_offset"] = json!("1");
        excess_graph
            .vertex(end_key, end_node.clone(), false)
            .unwrap();
        check_duration(&excess_graph, &mut shortened_voice, &mut shortened_measure).unwrap();
        assert_eq!(
            duration_state(&shortened_voice, &shortened_measure),
            duration["excess"]
        );
        end_node["time_offset"] = Value::Null;
        excess_graph.vertex(end_key, end_node, false).unwrap();
        check_duration(&excess_graph, &mut shortened_voice, &mut shortened_measure).unwrap();
        assert_eq!(
            duration_state(&shortened_voice, &shortened_measure),
            duration["missing_offset"]
        );

        let remaining = *owner.voices[timed_voice].chord_ids.last().unwrap();
        let mut data = owner.owner.graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == remaining)
            .unwrap()
            .clone();
        let key: Key = serde_json::from_value(data["origin"].clone()).unwrap();
        data["time_offset"] = Value::Null;
        owner.owner.graph.vertex(key, data, false).unwrap();
        assert_eq!(
            timing_state(&owner.owner.graph, &owner.voices[timed_voice]),
            timing["missing_offset"]
        );
        for (key, stem, state) in before_cache {
            let current = owner.owner.caches[&1]
                .registered_stem(&owner.owner.graph, key.local)
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&stem, &current));
            assert_eq!(current.snapshot().unwrap(), state);
        }
        assert_eq!(
            format!("{:?}", owner.owner.prepared.registry),
            before_registry
        );
    }
}
