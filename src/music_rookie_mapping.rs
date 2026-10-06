// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original separated VoiceDistance, ChordsMapper and small InjectionSolver.
//! Owned image adapter currently admits one incoming/one Voice and no implicit
//! tuplets. Implicit-tuplet analysis, unknown-time new-Voice inference
//! and Merged remain pending.
use crate::{
    music_measure_slots::CompoundSlot, music_measures::Measure, music_voices::Voice,
    shared_sig::Graph,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
struct Fact {
    head: bool,
    staff: u64,
    part: u64,
    stem_dir: i64,
    voice: Option<usize>,
    starting_staff: Option<u64>,
    absolute_pitch: Option<i64>,
    heads_center_y: i64,
}
fn separated(left: &Fact, right: &Fact, interline: usize, next: bool) -> Result<i32, String> {
    if next {
        return Ok(0);
    }
    if left.voice.zip(right.voice).is_some_and(|(a, b)| a != b) {
        return Ok(10000);
    }
    let mut staff = 0;
    if left.staff != right.staff {
        if left.part != right.part || !left.head || !right.head {
            return Ok(10000);
        }
        staff = 20;
    }
    let nis = if left.voice.is_some() && left.starting_staff != Some(right.staff) {
        10
    } else {
        0
    };
    let nar = 4 * i32::from(left.head) + 4 * i32::from(right.head);
    let dp = if left.head && right.head {
        (left
            .absolute_pitch
            .ok_or("music_mapping_absolute_pitch_required")?
            - right
                .absolute_pitch
                .ok_or("music_mapping_absolute_pitch_required")?)
        .abs()
    } else {
        if interline == 0 {
            return Err("music_mapping_interline_required".into());
        }
        (2 * (right.heads_center_y - left.heads_center_y) / interline as i64).abs()
    };
    let stem = if (left.stem_dir - right.stem_dir).abs() == 2 {
        6
    } else {
        0
    };
    i32::try_from(dp)
        .ok()
        .and_then(|dp| dp.checked_add(staff + nis + nar + stem))
        .ok_or_else(|| "music_mapping_distance_overflow".into())
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
struct Pair {
    rookie: usize,
    active: usize,
}
#[derive(Serialize)]
struct Mapping {
    cost: i32,
    pairs: Vec<Pair>,
}
struct NarrowTime {
    id: usize,
    chord_ids: Vec<usize>,
    time_offset: Option<u32>,
}
struct ValidationInput {
    rookies: Vec<usize>,
    actives: Vec<usize>,
    distances: BTreeMap<(usize, usize), i32>,
    black: Vec<Pair>,
    next: Vec<Pair>,
    narrow: Vec<NarrowTime>,
    active_ends: BTreeMap<usize, Option<u32>>,
    rookie_durations: BTreeMap<usize, u32>,
    expected: Option<u32>,
    abnormal: bool,
}
struct ValidatedMapping {
    mapping: Option<Mapping>,
    black: Vec<Pair>,
    narrow: Vec<NarrowTime>,
    applied: bool,
    abnormal: bool,
}
fn decision(input: ValidationInput, mapping: Option<Mapping>, applied: bool) -> ValidatedMapping {
    ValidatedMapping {
        mapping,
        black: input.black,
        narrow: input.narrow,
        applied,
        abnormal: input.abnormal,
    }
}
fn validated_mapping(mut input: ValidationInput) -> Result<ValidatedMapping, String> {
    if input.actives.is_empty() {
        return Ok(decision(input, None, false));
    }
    'retry: loop {
        let map = mapping(
            &input.rookies,
            &input.actives,
            &input.distances,
            &input.black,
            &input.next,
        )?;
        if map.pairs.is_empty() {
            return Ok(decision(input, Some(map), false));
        }
        for narrow in &input.narrow {
            let Some(slot_time) = narrow.time_offset else {
                continue;
            };
            for pair in map
                .pairs
                .iter()
                .filter(|p| narrow.chord_ids.contains(&p.rookie))
            {
                let Some(end) = input.active_ends.get(&pair.active).copied().flatten() else {
                    return Ok(decision(input, Some(map), false));
                };
                let duration = *input
                    .rookie_durations
                    .get(&pair.rookie)
                    .ok_or("music_mapping_rookie_duration_required")?;
                let chord_end = end
                    .checked_add(duration)
                    .ok_or("music_mapping_time_overflow")?;
                if input.expected.is_some_and(|expected| chord_end > expected)
                    && !input.next.contains(pair)
                    && !input.black.contains(pair)
                {
                    input.black.push(*pair);
                    continue 'retry;
                }
                if end != slot_time {
                    // processNext may keep an explicit pair even after blacklisting.
                    // Original breaks this loop; a repeated forced pair must not hang.
                    if input.black.contains(pair) {
                        return Ok(decision(input, Some(map), false));
                    }
                    input.black.push(*pair);
                    continue 'retry;
                }
            }
        }
        let mut retry_inferred = false;
        for narrow in &mut input.narrow {
            if narrow.time_offset.is_none() {
                let mut inferred = BTreeMap::<u32, Vec<usize>>::new();
                for rookie in &narrow.chord_ids {
                    let Some(pair) = map.pairs.iter().find(|p| p.rookie == *rookie) else {
                        continue;
                    };
                    let Some(end) = input.active_ends.get(&pair.active).copied().flatten() else {
                        input.abnormal = true;
                        continue;
                    };
                    inferred.entry(end).or_default().push(pair.rookie);
                }
                narrow.time_offset = inferred.keys().next().copied();
                for rookies in inferred.values().skip(1) {
                    for rookie in rookies {
                        let pair = *map.pairs.iter().find(|p| p.rookie == *rookie).unwrap();
                        if !input.black.contains(&pair) {
                            input.black.push(pair);
                        }
                        retry_inferred = true;
                    }
                }
            }
        }
        if retry_inferred {
            continue 'retry;
        }
        return Ok(decision(input, Some(map), true));
    }
}
// Strict improvement and ascending range enumeration preserve original ties.
// Original itself documents brute force for small collections. Larger searches
// refuse by an explicit
// visit budget, never return a suboptimal mapping as a successful result.
fn injection(costs: &[Vec<i32>]) -> Result<(i32, Vec<usize>), String> {
    if costs.is_empty() {
        return Ok((0, vec![]));
    } // ChordsMapper skips solver here.
    let width = costs[0].len();
    if width < costs.len() || costs.iter().any(|r| r.len() != width) {
        return Err("music_mapping_cost_matrix_required".into());
    }
    struct Search<'a> {
        costs: &'a [Vec<i32>],
        free: Vec<bool>,
        config: Vec<usize>,
        best: Vec<usize>,
        best_cost: i32,
        visits: usize,
    }
    impl Search<'_> {
        fn inspect(&mut self, id: usize, cost: i32) -> Result<(), String> {
            for ir in 0..self.free.len() {
                if self.free[ir] {
                    self.visits += 1;
                    if self.visits > 1_000_000 {
                        return Err("music_mapping_exhaustive_search_budget".into());
                    }
                    self.free[ir] = false;
                    self.config[id] = ir;
                    let next = cost
                        .checked_add(self.costs[id][ir])
                        .ok_or("music_mapping_cost_overflow")?;
                    if id + 1 < self.costs.len() {
                        self.inspect(id + 1, next)?;
                    } else if next < self.best_cost {
                        self.best.clone_from(&self.config);
                        self.best_cost = next;
                    }
                    self.free[ir] = true;
                }
            }
            Ok(())
        }
    }
    let mut search = Search {
        costs,
        free: vec![true; width],
        config: vec![0; costs.len()],
        best: vec![0; costs.len()],
        best_cost: i32::MAX,
        visits: 0,
    };
    search.inspect(0, 0)?;
    Ok((search.best_cost, search.best))
}
fn mapping(
    rookies: &[usize],
    actives: &[usize],
    distances: &BTreeMap<(usize, usize), i32>,
    black: &[Pair],
    next: &[Pair],
) -> Result<Mapping, String> {
    let mut remaining_r = rookies.to_vec();
    let mut remaining_a = actives.to_vec();
    let mut pairs = Vec::new();
    for &p in next {
        if rookies.contains(&p.rookie) && actives.contains(&p.active) {
            if !pairs.contains(&p) {
                pairs.push(p);
            }
            remaining_r.retain(|r| *r != p.rookie);
            remaining_a.retain(|a| *a != p.active);
        }
    }
    let mut cost = 0;
    if !remaining_r.is_empty() {
        let costs = remaining_r
            .iter()
            .map(|&r| {
                let mut row = remaining_a
                    .iter()
                    .map(|&a| {
                        let p = Pair {
                            rookie: r,
                            active: a,
                        };
                        if next.contains(&p) {
                            Ok(0)
                        } else if black.contains(&p) {
                            Ok(10000)
                        } else {
                            distances
                                .get(&(r, a))
                                .copied()
                                .ok_or("music_mapping_distance_required")
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                row.extend(std::iter::repeat_n(60, remaining_r.len()));
                Ok(row)
            })
            .collect::<Result<Vec<_>, &str>>()?;
        let (best, links) = injection(&costs)?;
        cost = best;
        for (i, link) in links.into_iter().enumerate() {
            if let Some(&a) = remaining_a.get(link) {
                let p = Pair {
                    rookie: remaining_r[i],
                    active: a,
                };
                if !pairs.contains(&p) {
                    pairs.push(p);
                }
            }
        }
    }
    Ok(Mapping { cost, pairs })
}
pub(crate) struct Context<'a> {
    pub graph: &'a mut Graph,
    pub voices: &'a mut Vec<Voice>,
    pub measures: &'a mut [Measure],
    pub chords: &'a [&'a crate::music_measures::Chord],
    pub narrow: &'a crate::music_measure_slots::Retrieval,
    pub measure_index: usize,
    pub candidates: &'a crate::music_measure_slots::CandidateReferences,
    pub order: &'a mut crate::music_measure_slots::CandidateOrder,
    pub expected: u32,
    pub clef: &'a Value,
    pub interline: usize,
    pub lifetime: &'a crate::music_voice_lifetime::Lifetime,
    pub slot_times: &'a mut crate::music_measure_slots::ClockState,
}
fn node(snapshot: &Value, id: usize) -> Result<&Value, String> {
    snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .ok_or_else(|| "music_mapping_live_chord_required".into())
}
fn bounds(n: &Value) -> Result<[i64; 4], String> {
    serde_json::from_value(n["bounds"].clone()).map_err(|_| "music_mapping_bounds_required".into())
}
fn time(n: &Value) -> Result<Option<u32>, String> {
    n["time_offset"]
        .as_str()
        .map(crate::music_time::parse_ticks)
        .transpose()
}
impl Context<'_> {
    fn read_grouped_slot_times(&mut self, slot: &CompoundSlot) -> Result<bool, String> {
        let snapshot = self.graph.snapshot();
        let mut ok = true;
        for member in &slot.members {
            let mut times = std::collections::BTreeSet::new();
            for &id in &member.chord_ids {
                if let Some(t) = time(node(&snapshot, id)?)? {
                    times.insert(t);
                }
            }
            if times.len() == 1 {
                self.slot_times.set(member.id, times.first().copied())?;
            } else if times.len() > 1 {
                // Original nonimplicit analyzeTimes has no pair to blacklist before mapping.
                ok = false;
            }
        }
        Ok(ok)
    }
    fn merge_with_previous_slot(
        &mut self,
        slot: &CompoundSlot,
        rookie: usize,
    ) -> Result<bool, String> {
        let Some(previous) = self.order.previous(slot.key)? else {
            return Ok(false);
        };
        let snapshot = self.graph.snapshot();
        let mut target = None;
        for &id in &previous.chord_ids {
            if self.narrow.equal(id, rookie)
                && let Some(t) = time(node(&snapshot, id)?)?
            {
                target = Some(t);
                break;
            }
        }
        let Some(t) = target else { return Ok(false) };
        crate::music_chord_time::set_and_push(
            self.graph,
            self.voices,
            self.measures,
            rookie,
            Some(t),
        )?;
        self.order.merge_previous(slot.key, rookie)?;
        Ok(true)
    }
    fn fact(&self, snapshot: &Value, id: usize) -> Result<Fact, String> {
        let n = node(snapshot, id)?;
        let m = &self.measures[self.measure_index];
        if n["origin"]["system"] != m.system
            || n["part_id"] != m.part
            || n["measure_index"] != m.index
        {
            return Err("music_mapping_owned_measure_required".into());
        }
        let head = n["kind"] == "HeadChordInter";
        if !head && n["kind"] != "RestChordInter" {
            return Err("music_mapping_ordinary_chord_required".into());
        }
        let voice = n["voice"].as_u64().map(|id| id as usize);
        let owner = voice
            .map(|voice_id| {
                self.voices
                    .iter()
                    .find(|v| {
                        v.measure == self.measure_index
                            && v.id == voice_id
                            && (v.chord_ids.contains(&id)
                                || v.measure_rest_chord == Some(n["id"].as_u64().unwrap() as usize))
                    })
                    .ok_or("music_mapping_actual_voice_owner_required")
            })
            .transpose()?;
        let members = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == id)
            .map(|e| node(snapshot, e["target"].as_u64().unwrap() as usize))
            .collect::<Result<Vec<_>, _>>()?;
        if members.len() != 1 {
            return Err("music_mapping_multi_note_chord_pending".into());
        }
        let member = members[0];
        let note_bounds = bounds(member)?;
        let stem = n["stem_id"]
            .as_u64()
            .map(|id| node(snapshot, id as usize))
            .transpose()?;
        let geometry = if head {
            crate::head_chord_geometry::calculate(&members, stem)?
        } else {
            crate::head_chord_geometry::calculate_rest(&members)?
        };
        let absolute_pitch = if head {
            let (step, octave) = crate::head_musical_pitch::diatonic(
                self.clef["shape"]
                    .as_str()
                    .ok_or("music_mapping_clef_required")?,
                self.clef["pitch"]
                    .as_f64()
                    .ok_or("music_mapping_clef_pitch_required")?,
                member["pitch"]
                    .as_f64()
                    .ok_or("music_mapping_head_pitch_required")?,
            )
            .map_err(str::to_owned)?;
            Some(
                i64::from(octave) * 7
                    + match step {
                        "C" => 0,
                        "D" => 1,
                        "E" => 2,
                        "F" => 3,
                        "G" => 4,
                        "A" => 5,
                        "B" => 6,
                        _ => unreachable!(),
                    },
            )
        } else {
            None
        };
        Ok(Fact {
            head,
            staff: m.staff,
            part: m.part,
            stem_dir: geometry.stem_dir(stem.is_some())?,
            voice,
            starting_staff: owner.map(|v| v.starting_staff),
            absolute_pitch,
            heads_center_y: note_bounds[1] + note_bounds[3] / 2,
        })
    }
    fn create_new_voices(
        &mut self,
        slot: &CompoundSlot,
        rookies: &[usize],
    ) -> Result<Vec<usize>, String> {
        let centers = self
            .chords
            .iter()
            .map(|c| (c.id, c.bounds[0] + c.bounds[2] / 2))
            .collect();
        let mut created = Vec::new();
        for &id in rookies {
            let current = self.graph.snapshot();
            if !node(&current, id)?["voice"].is_null() {
                continue;
            }
            let chord = self
                .chords
                .iter()
                .find(|c| c.id == id)
                .ok_or("music_mapping_owned_new_voice_chord_required")?;
            crate::music_voices::create_voice(self.graph, self.voices, self.measures, chord)?;
            created.push(id);
            let assigned = self.graph.snapshot();
            if time(node(&assigned, id)?)?.is_none() {
                let siblings = self.narrow.ordered_siblings(slot, id, &centers)?;
                for &sibling in &siblings {
                    let actual = self.graph.snapshot();
                    if let Some(t) = time(node(&actual, sibling)?)? {
                        crate::music_chord_time::set_and_push(
                            self.graph,
                            self.voices,
                            self.measures,
                            id,
                            Some(t),
                        )?;
                        break;
                    }
                }
                if siblings.is_empty() && time(node(&self.graph.snapshot(), id)?)?.is_none() {
                    if self.merge_with_previous_slot(slot, id)? {
                        return Ok(created);
                    }
                    return Err("music_mapping_new_voice_time_inference_pending".into());
                }
            }
        }
        Ok(created)
    }
    pub fn map_slot(&mut self, slot: &CompoundSlot) -> Result<Value, String> {
        let (mut step, rookies) = self.map_rookies(slot)?;
        let created = self.create_new_voices(slot, &rookies)?;
        if !created.is_empty() {
            step["validation"]["new_voice_chord_ids"] = json!(created);
        }
        let actual = self.graph.snapshot();
        step["after"] = json!(
            slot.chord_ids
                .iter()
                .map(|&id| {
                    let n = node(&actual, id).unwrap();
                    json!({"chord_id":id,"time_offset":n["time_offset"],"voice":n["voice"]})
                })
                .collect::<Vec<_>>()
        );
        Ok(step)
    }
    fn map_rookies(&mut self, slot: &CompoundSlot) -> Result<(Value, Vec<usize>), String> {
        let retained = self.order.get(slot.key)?;
        let incoming = slot
            .chord_ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let members = slot
            .members
            .iter()
            .flat_map(|s| &s.chord_ids)
            .copied()
            .collect::<Vec<_>>();
        if slot.id != retained.id
            || slot.chord_ids != retained.chord_ids
            || incoming.is_empty()
            || incoming.len() != slot.chord_ids.len()
            || members.len() != incoming.len()
            || members
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                != incoming
            || slot.members.iter().any(|s| s.chord_ids.is_empty())
        {
            return Err("music_mapping_owned_compound_members_required".into());
        }
        for &id in &slot.chord_ids {
            let chord = self
                .chords
                .iter()
                .find(|c| c.id == id)
                .ok_or("music_mapping_owned_measure_chord_required")?;
            if chord.measure != self.measure_index || self.graph.id(chord.key) != Some(id) {
                return Err("music_mapping_owned_measure_chord_required".into());
            }
            self.slot_times.get(
                slot.members
                    .iter()
                    .find(|s| s.chord_ids.contains(&id))
                    .unwrap()
                    .id,
            )?;
        }
        let snapshot = self.graph.snapshot();
        for &id in &slot.chord_ids {
            self.fact(&snapshot, id)?;
        }
        let edges = snapshot["relations"].as_array().unwrap();
        let mut times: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        for &id in &slot.chord_ids {
            if let Some(t) = time(node(&snapshot, id)?)? {
                times.entry(t).or_default().push(id);
            }
        }
        if !self.read_grouped_slot_times(slot)? {
            return Err("music_mapping_implicit_tuplet_or_conflicting_slot_times_pending".into());
        }
        let rookies = slot
            .chord_ids
            .iter()
            .copied()
            .filter(|&id| node(&snapshot, id).unwrap()["voice"].is_null())
            .collect::<Vec<_>>();
        let pairs = |kind: &str| {
            rookies
                .iter()
                .flat_map(|&r| {
                    edges
                        .iter()
                        .filter(move |e| {
                            e["kind"] == kind && (e["source"] == r || e["target"] == r)
                        })
                        .map(move |e| Pair {
                            rookie: r,
                            active: if e["source"] == r {
                                e["target"].as_u64().unwrap() as usize
                            } else {
                                e["source"].as_u64().unwrap() as usize
                            },
                        })
                })
                .collect::<Vec<_>>()
        };
        let next = pairs("NextInVoiceRelation");
        let black = pairs("SeparateVoiceRelation");
        let current_x = self.candidates.x_offset(slot.key)?;
        let mut actives = Vec::new();
        for voice in self
            .voices
            .iter()
            .filter(|v| v.measure == self.measure_index && v.measure_rest_chord.is_none())
        {
            let &last = voice
                .chord_ids
                .last()
                .ok_or("music_mapping_last_chord_required")?;
            // Current compared image scope explicitly has implicitTuplets=false.
            if crate::music_voices::last_end_ticks(self.graph, voice)?
                .is_some_and(|t| t >= self.expected)
            {
                continue;
            }
            let previous = self
                .candidates
                .reference(last)
                .ok_or("music_mapping_actual_candidate_slot_required")?;
            if self.candidates.x_offset(previous)? > current_x {
                continue;
            }
            if self.lifetime.allows_active(
                voice.id,
                last,
                &rookies,
                next.iter().map(|p| (p.rookie, p.active)),
            ) {
                actives.push(last);
            }
        }
        let mut distances = BTreeMap::new();
        let mut costs = Vec::new();
        for &r in &rookies {
            for &a in &actives {
                let left = self.fact(&snapshot, a)?;
                let right = self.fact(&snapshot, r)?;
                let directed = edges.iter().any(|e| {
                    e["kind"] == "NextInVoiceRelation" && e["source"] == a && e["target"] == r
                });
                let d = separated(&left, &right, self.interline, directed)?;
                distances.insert((r, a), d);
                costs.push(json!({"rookie":r,"active":a,"left":left,"right":right,"distance":d}));
            }
        }
        let mut result = Value::Null;
        let mut validation = Value::Null;
        if !rookies.is_empty() {
            let active_ends = actives
                .iter()
                .map(|&id| {
                    let n = node(&snapshot, id)?;
                    let end = match (time(n)?, n["duration"].as_str()) {
                        (Some(start), Some(duration)) => Some(
                            start
                                .checked_add(crate::music_time::parse_ticks(duration)?)
                                .ok_or("music_mapping_time_overflow")?,
                        ),
                        _ => None,
                    };
                    Ok((id, end))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()?;
            let rookie_durations = rookies
                .iter()
                .map(|&id| {
                    let n = node(&snapshot, id)?;
                    Ok((
                        id,
                        crate::music_time::parse_ticks(
                            n["duration"]
                                .as_str()
                                .ok_or("music_mapping_rookie_duration_required")?,
                        )?,
                    ))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()?;
            let narrow = slot
                .members
                .iter()
                .map(|s| {
                    self.slot_times.get(s.id).map(|t| NarrowTime {
                        id: s.id,
                        chord_ids: s.chord_ids.clone(),
                        time_offset: t,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let validated = validated_mapping(ValidationInput {
                rookies: rookies.clone(),
                actives: actives.clone(),
                distances,
                black,
                next,
                narrow,
                active_ends: active_ends.clone(),
                rookie_durations,
                expected: Some(self.expected),
                abnormal: self.measures[self.measure_index].timing.abnormal,
            })?;
            self.measures[self.measure_index].timing.abnormal |= validated.abnormal;
            validation = json!({"black":validated.black,"applied":validated.applied});
            for narrow in validated.narrow {
                self.slot_times.set(narrow.id, narrow.time_offset)?;
            }
            if let Some(map) = validated.mapping {
                if validated.applied {
                    for pair in &map.pairs {
                        let end = active_ends[&pair.active];
                        let own = self
                            .voices
                            .iter()
                            .position(|v| {
                                v.measure == self.measure_index
                                    && v.chord_ids.contains(&pair.active)
                            })
                            .ok_or("music_mapping_active_voice_required")?;
                        crate::music_voices::assign_chord_voice(
                            self.graph,
                            self.voices,
                            self.measures,
                            pair.rookie,
                            own,
                        )?;
                        if let Some(end) = end {
                            crate::music_chord_time::set_and_push(
                                self.graph,
                                self.voices,
                                self.measures,
                                pair.rookie,
                                Some(end),
                            )?;
                        }
                    }
                }
                result = serde_json::to_value(map).unwrap();
            }
        }
        let final_snapshot = self.graph.snapshot();
        let after = slot
            .chord_ids
            .iter()
            .map(|&id| {
                let n = node(&final_snapshot, id).unwrap();
                json!({"chord_id":id,"time_offset":n["time_offset"],"voice":n["voice"]})
            })
            .collect::<Vec<_>>();
        Ok((
            json!({"slot_id":slot.id,"known_times":times.iter().map(|(t,ids)|json!({"time":crate::music_time::format_ticks(*t),"chord_ids":ids})).collect::<Vec<_>>(),"rookies":rookies,"actives":actives,"costs":costs,"mapping":result,"ok":true,"after":after,"validation":validation}),
            rookies,
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starting_voice_constructors_preserve_original_parallel_order_and_reassignment() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-parallel-starting/observed.json"
        ))
        .unwrap();
        let mut owner = image_owner();
        let mi = 1;
        for case in oracle["controls"].as_array().unwrap() {
            reset_owner(&mut owner);
            let mut chords = owner
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            let ordinal = |id| chords.iter().position(|c| c.id == id).unwrap();
            if case["name"] == "already-owned-start" {
                crate::music_voices::create_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    chords[0],
                )
                .unwrap();
                crate::music_chord_time::set_and_push(
                    &mut owner.owner.graph,
                    &owner.voices,
                    &owner.measures,
                    chords[0].id,
                    Some(4),
                )
                .unwrap();
                assert_eq!(
                    crate::music_voices::create_voice(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &owner.measures,
                        chords[0]
                    )
                    .unwrap_err(),
                    "music_voice_unassigned_seed_required"
                );
            }
            let selected = case["input_starting"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| chords[i.as_u64().unwrap() as usize])
                .collect::<Vec<_>>();
            let starting = crate::music_voices::process_starting_chords(
                &mut owner.owner.graph,
                &mut owner.voices,
                &owner.measures,
                mi,
                &selected,
            )
            .unwrap();
            assert_eq!(
                json!(starting.iter().map(|&id| ordinal(id)).collect::<Vec<_>>()),
                case["starting_order"]
            );
            let snapshot = owner.owner.graph.snapshot();
            let states = chords
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let n = node(&snapshot, c.id).unwrap();
                    json!({"id":i,"voice":n["voice"],"time_offset":n["time_offset"]})
                })
                .collect::<Vec<_>>();
            let voices = owner.voices.iter().filter(|v| v.measure == mi).map(|v| json!({"id":v.id,"chord_ids":v.chord_ids.iter().map(|&id| ordinal(id)).collect::<Vec<_>>()})).collect::<Vec<_>>();
            assert_eq!(json!(states), case["chord_states"], "{}", case["name"]);
            assert_eq!(json!(voices), case["voices"], "{}", case["name"]);
            if case["name"] == "already-owned-start" {
                crate::music_voices::generate_slots(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &mut owner.measures[mi],
                    mi,
                )
                .unwrap();
                // Original raw constructor state retains old membership; it is
                // not evidence for a valid automatically recognized score.
                assert_eq!(
                    crate::music_voice_export::sequence(
                        &owner.owner.graph,
                        &owner.voices,
                        &owner.measures[mi],
                        mi
                    )
                    .err()
                    .unwrap(),
                    "musicxml_voice_owned_chord_required"
                );
            }
        }
    }
    #[test]
    fn multiple_rookies_retain_original_assignment_retry_and_voice_ownership() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-compound-mapping/observed.json"
        ))
        .unwrap();
        let export_oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-compound-voice-export/observed.json"
        ))
        .unwrap();
        let mut owner = image_owner();
        let source_events = owner.owner.prepared.report["events"]
            .as_array()
            .unwrap()
            .clone();
        let mi = 1;
        let mut controls = oracle["controls"].as_array().unwrap().clone();
        controls.extend(
            export_oracle["controls"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|c| c.get("mapping"))
                .cloned(),
        );
        for case in &controls {
            reset_owner(&mut owner);
            let mut chords = owner
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            let ordinal = |id| chords.iter().position(|c| c.id == id).unwrap();
            let (narrow, wide) = crate::music_measure_slots::from_owned(
                &owner.owner.graph,
                &owner.measures[mi],
                &chords,
                owner.owner.interline,
            )
            .unwrap();
            let centers = chords
                .iter()
                .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
                .collect();
            let mut compounds =
                crate::music_measure_slots::build_compounds(&narrow.slots, &wide.slots, &centers)
                    .unwrap();
            compounds.truncate(2);
            let members = case["input"]["narrow"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| crate::music_measure_slots::MeasureSlot {
                    id: m["id"].as_u64().unwrap() as usize,
                    chord_ids: m["chord_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|id| chords[id.as_u64().unwrap() as usize].id)
                        .collect(),
                })
                .collect::<Vec<_>>();
            let mut ids = members
                .iter()
                .flat_map(|m| &m.chord_ids)
                .copied()
                .collect::<Vec<_>>();
            ids.sort_by_key(|id| centers[id]);
            compounds.push(CompoundSlot {
                key: 3,
                id: 3,
                members,
                chord_ids: ids,
            });
            for (i, active) in case["input"]["active_ends"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                assert_eq!(active["id"], i);
                crate::music_voices::create_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    chords[i],
                )
                .unwrap();
                let end = crate::music_time::parse_ticks(active["end"].as_str().unwrap()).unwrap();
                crate::music_chord_time::set_and_push(
                    &mut owner.owner.graph,
                    &owner.voices,
                    &owner.measures,
                    chords[i].id,
                    Some(end - 4),
                )
                .unwrap();
            }
            let candidates = crate::music_measure_slots::CandidateReferences::new(
                &owner.owner.graph,
                &owner.measures[mi],
                &compounds,
            )
            .unwrap();
            let mut order =
                crate::music_measure_slots::CandidateOrder::new(&compounds, &candidates).unwrap();
            let mut slot_times = crate::music_measure_slots::ClockState::new(&compounds).unwrap();
            let snapshot = owner.owner.graph.snapshot();
            let clef = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "ClefInter" && n["origin"]["system"] == 1)
                .unwrap();
            let lifetime = crate::music_voice_lifetime::Lifetime::default();
            let mut context = Context {
                graph: &mut owner.owner.graph,
                voices: &mut owner.voices,
                measures: &mut owner.measures,
                chords: &chords,
                narrow: &narrow,
                measure_index: mi,
                candidates: &candidates,
                order: &mut order,
                expected: 16,
                clef,
                interline: owner.owner.interline,
                lifetime: &lifetime,
                slot_times: &mut slot_times,
            };
            let mut invalid = compounds[2].clone();
            invalid.members[0].chord_ids.push(chords[0].id);
            assert_eq!(
                context.map_slot(&invalid).unwrap_err(),
                "music_mapping_owned_compound_members_required"
            );
            assert_eq!(context.graph.snapshot(), snapshot);
            let step = context.map_slot(&compounds[2]).unwrap();
            for key in ["rookies", "actives"] {
                assert_eq!(
                    json!(
                        step[key]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|id| ordinal(id.as_u64().unwrap() as usize))
                            .collect::<Vec<_>>()
                    ),
                    case["input"][key]
                );
            }
            let costs = step["costs"].as_array().unwrap().iter().map(|c| json!({"rookie":ordinal(c["rookie"].as_u64().unwrap() as usize),"active":ordinal(c["active"].as_u64().unwrap() as usize),"distance":c["distance"]})).collect::<Vec<_>>();
            assert_eq!(json!(costs), case["input"]["costs"]);
            let translate_pairs = |v: &Value| {
                v.as_array().unwrap().iter().map(|p| json!({"rookie":ordinal(p["rookie"].as_u64().unwrap() as usize),"active":ordinal(p["active"].as_u64().unwrap() as usize)})).collect::<Vec<_>>()
            };
            let actual = context.graph.snapshot();
            let states = chords
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let n = node(&actual, c.id).unwrap();
                    json!({"id":i,"voice":n["voice"],"time_offset":n["time_offset"]})
                })
                .collect::<Vec<_>>();
            let remaining = chords[2..]
                .iter()
                .filter(|c| {
                    !step["mapping"]["pairs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|p| p["rookie"] == c.id)
                })
                .map(|c| ordinal(c.id))
                .collect::<Vec<_>>();
            let output = json!({"mapping":{"cost":step["mapping"]["cost"],"pairs":translate_pairs(&step["mapping"]["pairs"])},"black":translate_pairs(&step["validation"]["black"]),"remaining_rookies":remaining,"narrow":context.slot_times.projection(&compounds[2]).unwrap()});
            assert_eq!(output, case["output"], "{}", case["name"]);
            assert_eq!(json!(states), case["chord_states"], "{}", case["name"]);
            assert_eq!(step["ok"], case["ok"]);
            assert_eq!(
                json!(context.measures[mi].timing.abnormal),
                case["abnormal"]
            );
            let voices = context.voices.iter().filter(|v| v.measure == mi).map(|v| json!({"id":v.id,"chord_ids":v.chord_ids.iter().map(|&id| ordinal(id)).collect::<Vec<_>>()})).collect::<Vec<_>>();
            assert_eq!(json!(voices), case["voices"]);
            for (i, c) in chords.iter().enumerate() {
                assert_eq!(
                    context.candidates.reference(c.id),
                    Some(if i < 2 {
                        compounds[i].key
                    } else {
                        compounds[2].key
                    })
                );
            }
            let export = export_oracle["controls"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["name"] == case["name"])
                .unwrap();
            crate::music_voices::generate_slots(
                context.graph,
                context.voices,
                &mut context.measures[mi],
                mi,
            )
            .unwrap();
            crate::music_voices::read_stack_actual_duration(
                context.graph,
                context.voices,
                &mut context.measures[mi],
                mi,
            )
            .unwrap();
            let snapshot = context.graph.snapshot();
            let bounds = |id| node(&snapshot, id).unwrap()["bounds"].clone();
            let slots = &context.measures[mi].timing.slots;
            let stack = json!({"slots":slots.iter().map(|s| json!({"id":s.id,"time_offset":s.time_offset.map(crate::music_time::format_ticks),"incoming_bounds":s.incoming_ids.iter().map(|&id| bounds(id)).collect::<Vec<_>>()})).collect::<Vec<_>>(),"voices":context.voices.iter().filter(|v| v.measure == mi).map(|v| json!({"id":v.id,"slots":slots.iter().filter_map(|s| v.slots.get(&s.id).map(|info| json!({"slot_id":s.id,"status":if info.continuing {"CONTINUE"} else {"BEGIN"},"chord_bounds":bounds(info.chord_id)}))).collect::<Vec<_>>()})).collect::<Vec<_>>(),"abnormal":context.measures[mi].timing.abnormal});
            assert_eq!(stack, export["stack"], "{}", case["name"]);
            let events = source_events
                .iter()
                .filter(|e| chords.iter().any(|c| e["chord_id"] == c.id))
                .map(|e| {
                    let mut e = e.clone();
                    let n = node(&snapshot, e["chord_id"].as_u64().unwrap() as usize).unwrap();
                    e["voice"] = n["voice"].clone();
                    e["time_offset"] = n["time_offset"].clone();
                    e
                })
                .collect::<Vec<_>>();
            let sequences = vec![
                crate::music_voice_export::sequence(
                    context.graph,
                    context.voices,
                    &context.measures[mi],
                    mi,
                )
                .unwrap(),
            ];
            let stream = sequences[0].iter().map(|s| match *s {
                crate::music_voice_export::Step::Chord { id, voice, duration } => {
                    let e = events.iter().find(|e| e["chord_id"] == id).unwrap();
                    json!({"kind":"chord","voice":voice,"step":e["step"],"octave":e["octave"],"type":e["type"],"stem":e["stem_direction"],"duration":crate::music_time::format_ticks(duration)})
                },
                crate::music_voice_export::Step::Backup(duration) => json!({"kind":"backup","duration":crate::music_time::format_ticks(duration)}),
                crate::music_voice_export::Step::Forward { voice, duration } => json!({"kind":"forward","voice":voice,"duration":crate::music_time::format_ticks(duration)}),
            }).collect::<Vec<_>>();
            assert_eq!(json!(stream), export["export_stream"], "{}", case["name"]);
            let divisions = crate::music_voice_export::divisions(&sequences);
            let mut xml = String::new();
            crate::music_voice_export::write_measure(
                &mut xml,
                &sequences[0],
                &events.iter().collect::<Vec<_>>(),
                divisions,
            )
            .unwrap();
            let expected = export["export_stream"].as_array().unwrap().iter().map(|s| {
                let duration = crate::music_time::parse_ticks(s["duration"].as_str().unwrap()).unwrap() * divisions / 4;
                match s["kind"].as_str().unwrap() {
                    "backup" => format!("<backup><duration>{duration}</duration></backup>\n"),
                    "forward" => format!("<forward><duration>{duration}</duration><voice>{}</voice></forward>\n", s["voice"]),
                    "chord" => format!("<note><pitch><step>{}</step><octave>{}</octave></pitch><duration>{duration}</duration><voice>{}</voice><type>{}</type>{}</note>\n", s["step"].as_str().unwrap(), s["octave"], s["voice"], s["type"].as_str().unwrap(), s["stem"].as_str().map(|stem| format!("<stem>{stem}</stem>")).unwrap_or_default()),
                    _ => unreachable!(),
                }
            }).collect::<String>();
            assert_eq!(xml, expected, "{}", case["name"]);
            if case["name"] == "separate-member-times" {
                let final_oracle: Value = serde_json::from_str(include_str!(
                    "../fixtures/official-measure-final-check/observed.json"
                ))
                .unwrap();
                assert_eq!(final_oracle["mapping"], *case);
                for observation in final_oracle["observations"].as_array().unwrap() {
                    let mutation = &observation["mutation"];
                    if let Some(id) = mutation["chord"].as_u64() {
                        let chord = chords[id as usize];
                        let snapshot = context.graph.snapshot();
                        let mut actual = node(&snapshot, chord.id).unwrap().clone();
                        actual["time_offset"] = mutation["time"].clone();
                        context.graph.vertex(chord.key, actual, false).unwrap();
                    }
                    if let Some(expected) = mutation.get("expected") {
                        context.measures[mi].timing.expected = expected
                            .as_str()
                            .map(crate::music_time::parse_ticks)
                            .transpose()
                            .unwrap();
                    }
                    if let Some(abnormal) = mutation["abnormal"].as_bool() {
                        context.measures[mi].timing.abnormal = abnormal;
                    }
                    let ok = crate::music_voices::final_check(
                        context.graph,
                        context.voices,
                        &mut context.measures[mi],
                        &owner.chords,
                        mi,
                    )
                    .unwrap();
                    let voices = context.voices.iter().filter(|v| v.measure == mi).map(|v| json!({"id":v.id,"termination":v.termination.map(crate::music_time::format_delta_ticks),"excess":v.excess.map(crate::music_time::format_ticks)})).collect::<Vec<_>>();
                    let timing = &context.measures[mi].timing;
                    let actual = json!({"ok":ok,"voices":voices,"expected":timing.expected.map(crate::music_time::format_ticks),"abnormal":timing.abnormal,"stack_excess":timing.excess.map(crate::music_time::format_ticks)});
                    let mut expected = observation.clone();
                    expected.as_object_mut().unwrap().remove("name");
                    expected.as_object_mut().unwrap().remove("mutation");
                    assert_eq!(actual, expected, "{}", observation["name"]);
                }
            }
        }
    }
    fn image_owner() -> crate::music_measures::OwnedMusic {
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        crate::music_xml::recognize(&image).unwrap().owner
    }
    fn reset_owner(owner: &mut crate::music_measures::OwnedMusic) {
        for i in 0..owner.measures.len() {
            crate::music_voices::reset_rhythm(
                &mut owner.owner.graph,
                &mut owner.voices,
                &mut owner.measures[i],
                &owner.chords,
                i,
            )
            .unwrap();
        }
    }
    fn observed_input(input: &Value) -> ValidationInput {
        let ticks = |v: &Value| {
            v.as_str()
                .map(crate::music_time::parse_ticks)
                .transpose()
                .unwrap()
        };
        ValidationInput {
            rookies: serde_json::from_value(input["rookies"].clone()).unwrap(),
            actives: serde_json::from_value(input["actives"].clone()).unwrap(),
            distances: input["costs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    (
                        (
                            c["rookie"].as_u64().unwrap() as usize,
                            c["active"].as_u64().unwrap() as usize,
                        ),
                        c["distance"].as_i64().unwrap() as i32,
                    )
                })
                .collect(),
            black: serde_json::from_value(input["black"].clone()).unwrap(),
            next: serde_json::from_value(input["next"].clone()).unwrap(),
            narrow: input["narrow"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| NarrowTime {
                    id: s["id"].as_u64().unwrap() as usize,
                    chord_ids: serde_json::from_value(s["chord_ids"].clone()).unwrap(),
                    time_offset: ticks(&s["time_offset"]),
                })
                .collect(),
            active_ends: input["active_ends"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| (n["id"].as_u64().unwrap() as usize, ticks(&n["end"])))
                .collect(),
            rookie_durations: input["rookie_durations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| {
                    (
                        n["id"].as_u64().unwrap() as usize,
                        ticks(&n["duration"]).unwrap(),
                    )
                })
                .collect(),
            expected: ticks(&input["expected"]),
            abnormal: false,
        }
    }
    #[test]
    fn original_slot_times_reselect_compatible_voices_or_leave_rookies_unassigned() {
        let originals = [
            include_str!("../fixtures/official-mapping-retry/observed.json"),
            include_str!("../fixtures/official-mapping-time-analysis/observed.json"),
            include_str!("../fixtures/official-mapping-unknown-clock/observed.json"),
        ];
        for case in originals.iter().flat_map(|s| {
            let original: Value = serde_json::from_str(s).unwrap();
            original["controls"].as_array().unwrap().clone()
        }) {
            let input = observed_input(&case["input"]);
            let rookies = input.rookies.clone();
            let result = validated_mapping(input).unwrap();
            let remaining = rookies
                .iter()
                .copied()
                .filter(|r| {
                    !result.applied
                        || !result
                            .mapping
                            .as_ref()
                            .is_some_and(|m| m.pairs.iter().any(|p| p.rookie == *r))
                })
                .collect::<Vec<_>>();
            let narrow=result.narrow.iter().map(|n|json!({"id":n.id,"time_offset":n.time_offset.map(crate::music_time::format_ticks)})).collect::<Vec<_>>();
            let mut output = json!({"mapping":result.mapping,"black":result.black,"remaining_rookies":remaining,"narrow":narrow});
            if case["output"].get("measure_abnormal").is_some() {
                output["measure_abnormal"] = json!(result.abnormal);
            }
            assert_eq!(output, case["output"], "{}", case["name"]);
        }
    }
    #[test]
    fn owned_mapping_preserves_original_voice_members_and_unknown_times() {
        let originals = [
            include_str!("../fixtures/official-new-voice-known-time/observed.json"),
            include_str!("../fixtures/official-mapping-unknown-clock/observed.json"),
        ];
        let mut owner = image_owner();
        let mi = 1;
        for case in originals.iter().flat_map(|s| {
            let oracle: Value = serde_json::from_str(s).unwrap();
            oracle["controls"].as_array().unwrap().clone()
        }) {
            let unknown_active = case.get("original_owner_state").is_some();
            reset_owner(&mut owner);
            let mut chords = owner
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            if !unknown_active {
                assert_eq!(
                    json!(chords.iter().map(|c| c.bounds).collect::<Vec<_>>()),
                    case["chord_bounds"]
                );
            }
            let (narrow, wide) = crate::music_measure_slots::from_owned(
                &owner.owner.graph,
                &owner.measures[mi],
                &chords,
                owner.owner.interline,
            )
            .unwrap();
            let centers = chords
                .iter()
                .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
                .collect();
            let compounds =
                crate::music_measure_slots::build_compounds(&narrow.slots, &wide.slots, &centers)
                    .unwrap();
            let seed =
                crate::music_voices::single(mi, &owner.measures[mi], &[chords[0]], &owner.voices)
                    .unwrap();
            let own = owner.voices.len();
            owner.voices.push(seed);
            crate::music_voices::assign_chord_voice(
                &mut owner.owner.graph,
                &mut owner.voices,
                &owner.measures,
                chords[0].id,
                own,
            )
            .unwrap();
            crate::music_chord_time::set_and_push(
                &mut owner.owner.graph,
                &owner.voices,
                &owner.measures,
                chords[0].id,
                if unknown_active { None } else { Some(12) },
            )
            .unwrap();
            crate::music_chord_time::set_and_push(
                &mut owner.owner.graph,
                &owner.voices,
                &owner.measures,
                chords[2].id,
                if unknown_active { None } else { Some(4) },
            )
            .unwrap();
            if case["name"] == "already-voiced" {
                crate::music_voices::assign_chord_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    chords[2].id,
                    own,
                )
                .unwrap();
            }
            let candidates = crate::music_measure_slots::CandidateReferences::new(
                &owner.owner.graph,
                &owner.measures[mi],
                &compounds,
            )
            .unwrap();
            let mut slot_times = crate::music_measure_slots::ClockState::new(&compounds).unwrap();
            let mut order =
                crate::music_measure_slots::CandidateOrder::new(&compounds, &candidates).unwrap();
            if unknown_active {
                let time = case["input"]["narrow"][0]["time_offset"]
                    .as_str()
                    .map(crate::music_time::parse_ticks)
                    .transpose()
                    .unwrap();
                slot_times.set(compounds[2].members[0].id, time).unwrap();
            }
            let snapshot = owner.owner.graph.snapshot();
            let clef = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "ClefInter" && n["origin"]["system"] == 1)
                .unwrap();
            let lifetime = crate::music_voice_lifetime::Lifetime::default();
            let mut context = Context {
                graph: &mut owner.owner.graph,
                voices: &mut owner.voices,
                measures: &mut owner.measures,
                chords: &chords,
                narrow: &narrow,
                measure_index: mi,
                candidates: &candidates,
                order: &mut order,
                expected: 16,
                clef,
                interline: owner.owner.interline,
                lifetime: &lifetime,
                slot_times: &mut slot_times,
            };
            let step = if unknown_active {
                context.map_rookies(&compounds[2]).map(|(step, _)| step)
            } else {
                context.map_slot(&compounds[2])
            }
            .unwrap();
            let ordinal = |id| chords.iter().position(|c| c.id == id).unwrap();
            for key in ["rookies", "actives"] {
                assert_eq!(
                    json!(
                        step[key]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|id| ordinal(id.as_u64().unwrap() as usize))
                            .collect::<Vec<_>>()
                    ),
                    case["input"][key]
                );
            }
            let current = owner.owner.graph.snapshot();
            let rookie = node(&current, chords[2].id).unwrap();
            let voices = owner.voices.iter().filter(|v|v.measure==mi).map(|v|
                json!({"id":v.id,"kind":"HIGH","starting_staff":v.starting_staff,"chord_ids":v.chord_ids.iter().map(|&id|ordinal(id)).collect::<Vec<_>>()})
            ).collect::<Vec<_>>();
            assert_eq!(
                json!({"voices":voices,"rookie_voice":rookie["voice"],"rookie_time":rookie["time_offset"],"abnormal":owner.measures[mi].timing.abnormal}),
                case[if unknown_active {
                    "original_owner_state"
                } else {
                    "output"
                }],
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn new_voice_uses_first_known_equal_sibling_before_a_nearer_other_chord() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-new-voice-siblings/observed.json"
        ))
        .unwrap();
        let mut owner = image_owner();
        let mi = 1;
        for case in oracle["controls"].as_array().unwrap() {
            reset_owner(&mut owner);
            let mut chords = owner
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            assert_eq!(
                json!(chords.iter().map(|c| c.bounds).collect::<Vec<_>>()),
                case["input"]["chord_bounds"]
            );
            owner
                .owner
                .graph
                .relations(
                    1,
                    crate::shared_sig::Phase::Chord,
                    vec![(
                        chords[2].key,
                        chords[0].key,
                        json!({"kind":"SameTimeRelation"}),
                    )],
                )
                .unwrap();
            let (narrow, _) = crate::music_measure_slots::from_owned(
                &owner.owner.graph,
                &owner.measures[mi],
                &chords,
                owner.owner.interline,
            )
            .unwrap();
            for group in case["input"]["initial_voice_members"].as_array().unwrap() {
                let ordinals = group
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_u64().unwrap() as usize)
                    .collect::<Vec<_>>();
                let index = crate::music_voices::create_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    chords[ordinals[0]],
                )
                .unwrap();
                for &ordinal in &ordinals[1..] {
                    crate::music_voices::assign_chord_voice(
                        &mut owner.owner.graph,
                        &mut owner.voices,
                        &owner.measures,
                        chords[ordinal].id,
                        index,
                    )
                    .unwrap();
                }
            }
            for (i, t) in case["input"]["times"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let t = t
                    .as_str()
                    .map(crate::music_time::parse_ticks)
                    .transpose()
                    .unwrap();
                crate::music_chord_time::set_and_push(
                    &mut owner.owner.graph,
                    &owner.voices,
                    &owner.measures,
                    chords[i].id,
                    t,
                )
                .unwrap();
            }
            let ordinals: Vec<usize> =
                serde_json::from_value(case["input"]["slot_chord_ids"].clone()).unwrap();
            let ids = ordinals.iter().map(|&i| chords[i].id).collect::<Vec<_>>();
            let declared = CompoundSlot {
                key: 0,
                id: 3,
                members: vec![crate::music_measure_slots::MeasureSlot {
                    id: 3,
                    chord_ids: ids.clone(),
                }],
                chord_ids: ids,
            };
            let compounds = [declared];
            let candidates = crate::music_measure_slots::CandidateReferences::new(
                &owner.owner.graph,
                &owner.measures[mi],
                &compounds,
            )
            .unwrap();
            let mut slot_times = crate::music_measure_slots::ClockState::new(&compounds).unwrap();
            let mut order =
                crate::music_measure_slots::CandidateOrder::new(&compounds, &candidates).unwrap();
            let actual = owner.owner.graph.snapshot();
            let clef = actual["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "ClefInter" && n["origin"]["system"] == 1)
                .unwrap();
            let lifetime = crate::music_voice_lifetime::Lifetime::default();
            let centers = chords
                .iter()
                .map(|c| (c.id, c.bounds[0] + c.bounds[2] / 2))
                .collect();
            let ordered = narrow
                .ordered_siblings(&compounds[0], chords[2].id, &centers)
                .unwrap();
            let created = Context {
                graph: &mut owner.owner.graph,
                voices: &mut owner.voices,
                measures: &mut owner.measures,
                chords: &chords,
                narrow: &narrow,
                measure_index: mi,
                candidates: &candidates,
                order: &mut order,
                expected: 16,
                clef,
                interline: owner.owner.interline,
                lifetime: &lifetime,
                slot_times: &mut slot_times,
            }
            .create_new_voices(&compounds[0], &[chords[2].id])
            .unwrap();
            assert_eq!(created, vec![chords[2].id]);
            let ordinal = |id| chords.iter().position(|c| c.id == id).unwrap();
            let actual = owner.owner.graph.snapshot();
            let rookie = node(&actual, chords[2].id).unwrap();
            let voices = owner.voices.iter().filter(|v|v.measure==mi).map(|v|json!({"id":v.id,"chord_ids":v.chord_ids.iter().map(|&id|ordinal(id)).collect::<Vec<_>>()})).collect::<Vec<_>>();
            assert_eq!(
                json!({"ordered_siblings":ordered.iter().map(|&id|ordinal(id)).collect::<Vec<_>>(),"voices":voices,"rookie_voice":rookie["voice"],"rookie_time":rookie["time_offset"]}),
                case["output"],
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn new_voice_merges_only_with_a_known_equal_previous_slot_and_retains_references() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-previous-slot-merge/observed.json"
        ))
        .unwrap();
        let mut owner = image_owner();
        let base = owner.owner.graph.clone();
        let mi = 1;
        for case in oracle["controls"].as_array().unwrap() {
            owner.owner.graph = base.clone();
            reset_owner(&mut owner);
            let mut chords = owner
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            if case["input"]["relation"] == "=" {
                owner
                    .owner
                    .graph
                    .relations(
                        1,
                        crate::shared_sig::Phase::Chord,
                        vec![(
                            chords[0].key,
                            chords[1].key,
                            json!({"kind":"SameTimeRelation"}),
                        )],
                    )
                    .unwrap();
            }
            let (narrow, _) = crate::music_measure_slots::from_owned(
                &owner.owner.graph,
                &owner.measures[mi],
                &chords,
                owner.owner.interline,
            )
            .unwrap();
            assert_eq!(
                narrow.equal(chords[0].id, chords[1].id),
                case["input"]["relation"] == "="
            );
            let compounds = case["input"]["constructors"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, constructor)| CompoundSlot {
                    key: i,
                    id: constructor["id"].as_u64().unwrap() as usize,
                    members: vec![crate::music_measure_slots::MeasureSlot {
                        id: i + 1,
                        chord_ids: constructor["narrow_chord_ids"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|i| chords[i.as_u64().unwrap() as usize].id)
                            .collect(),
                    }],
                    chord_ids: constructor["chord_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|i| chords[i.as_u64().unwrap() as usize].id)
                        .collect(),
                })
                .collect::<Vec<_>>();
            let candidates = crate::music_measure_slots::CandidateReferences::new(
                &owner.owner.graph,
                &owner.measures[mi],
                &compounds,
            )
            .unwrap();
            let mut order =
                crate::music_measure_slots::CandidateOrder::new(&compounds, &candidates).unwrap();
            let mut slot_times = crate::music_measure_slots::ClockState::new(&compounds).unwrap();
            crate::music_voices::create_voice(
                &mut owner.owner.graph,
                &mut owner.voices,
                &owner.measures,
                chords[0],
            )
            .unwrap();
            let previous_time = case["input"]["previous_time"]
                .as_str()
                .map(crate::music_time::parse_ticks)
                .transpose()
                .unwrap();
            crate::music_chord_time::set_and_push(
                &mut owner.owner.graph,
                &owner.voices,
                &owner.measures,
                chords[0].id,
                previous_time,
            )
            .unwrap();
            let positive = case["name"] == "known-equal";
            if !positive {
                crate::music_voices::create_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    chords[1],
                )
                .unwrap();
            }
            let snapshot = owner.owner.graph.snapshot();
            let clef = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "ClefInter" && n["origin"]["system"] == 1)
                .unwrap();
            let lifetime = crate::music_voice_lifetime::Lifetime::default();
            let merged = {
                let mut context = Context {
                    graph: &mut owner.owner.graph,
                    voices: &mut owner.voices,
                    measures: &mut owner.measures,
                    chords: &chords,
                    narrow: &narrow,
                    measure_index: mi,
                    candidates: &candidates,
                    order: &mut order,
                    expected: 16,
                    clef,
                    interline: owner.owner.interline,
                    lifetime: &lifetime,
                    slot_times: &mut slot_times,
                };
                if positive {
                    assert_eq!(
                        context
                            .create_new_voices(&compounds[1], &[chords[1].id])
                            .unwrap(),
                        vec![chords[1].id]
                    );
                    !context.order.contains(compounds[1].key)
                } else {
                    context
                        .merge_with_previous_slot(&compounds[1], chords[1].id)
                        .unwrap()
                }
            };
            let ordinal = |id| chords.iter().position(|c| c.id == id).unwrap();
            let actual = owner.owner.graph.snapshot();
            let rookie = node(&actual, chords[1].id).unwrap();
            let view = order
                .ordered()
                .map(|s| {
                    json!({"object_key":s.key,"id":s.id,
                "chord_ids":s.chord_ids.iter().map(|&id|ordinal(id)).collect::<Vec<_>>()})
                })
                .collect::<Vec<_>>();
            let references = chords.iter().enumerate().map(|(i,c)| {
                let key = candidates.reference(c.id).unwrap();
                json!({"chord":i,"object_key":key,"id":order.get(key).unwrap().id,"x_offset":candidates.x_offset(key).unwrap()})
            }).collect::<Vec<_>>();
            let voices = owner
                .voices
                .iter()
                .filter(|v| v.measure == mi)
                .map(|v| {
                    json!({"id":v.id,
                "chord_ids":v.chord_ids.iter().map(|&id|ordinal(id)).collect::<Vec<_>>()})
                })
                .collect::<Vec<_>>();
            let members = &order.get(0).unwrap().members[0].chord_ids;
            assert_eq!(
                json!({"merged":merged,"ordered":view,"references":references,"voices":voices,
                "rookie_time":rookie["time_offset"],"rookie_voice":rookie["voice"],
                "previous_narrow_chord_ids":members.iter().map(|&id|ordinal(id)).collect::<Vec<_>>() }),
                case["output"],
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn grouped_known_clocks_are_member_scoped_and_empty_reads_preserve_them() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-grouped-slot-clock/observed.json"
        ))
        .unwrap();
        let mut owner = image_owner();
        let mi = 1;
        for case in oracle["controls"].as_array().unwrap() {
            reset_owner(&mut owner);
            let mut chords = owner
                .chords
                .iter()
                .filter(|c| c.measure == mi)
                .collect::<Vec<_>>();
            chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
            let (narrow, _) = crate::music_measure_slots::from_owned(
                &owner.owner.graph,
                &owner.measures[mi],
                &chords,
                owner.owner.interline,
            )
            .unwrap();
            for &chord in &chords[..2] {
                crate::music_voices::create_voice(
                    &mut owner.owner.graph,
                    &mut owner.voices,
                    &owner.measures,
                    chord,
                )
                .unwrap();
            }
            for (i, t) in case["input"]["chord_times"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let time = t
                    .as_str()
                    .map(crate::music_time::parse_ticks)
                    .transpose()
                    .unwrap();
                crate::music_chord_time::set_and_push(
                    &mut owner.owner.graph,
                    &owner.voices,
                    &owner.measures,
                    chords[i].id,
                    time,
                )
                .unwrap();
            }
            let members = case["input"]["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| crate::music_measure_slots::MeasureSlot {
                    id: m["id"].as_u64().unwrap() as usize,
                    chord_ids: m["chord_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|id| chords[id.as_u64().unwrap() as usize].id)
                        .collect(),
                })
                .collect::<Vec<_>>();
            let mut ids = members
                .iter()
                .flat_map(|m| m.chord_ids.iter())
                .copied()
                .collect::<Vec<_>>();
            ids.sort_by_key(|id| {
                let c = chords.iter().find(|c| c.id == *id).unwrap();
                c.bounds[1] + c.bounds[3] / 2
            });
            let compounds = [CompoundSlot {
                key: 0,
                id: 1,
                members,
                chord_ids: ids,
            }];
            let candidates = crate::music_measure_slots::CandidateReferences::new(
                &owner.owner.graph,
                &owner.measures[mi],
                &compounds,
            )
            .unwrap();
            let mut order =
                crate::music_measure_slots::CandidateOrder::new(&compounds, &candidates).unwrap();
            let mut slot_times = crate::music_measure_slots::ClockState::new(&compounds).unwrap();
            for member in case["input"]["members"].as_array().unwrap() {
                slot_times
                    .set(
                        member["id"].as_u64().unwrap() as usize,
                        member["time_offset"]
                            .as_str()
                            .map(crate::music_time::parse_ticks)
                            .transpose()
                            .unwrap(),
                    )
                    .unwrap();
            }
            let snapshot = owner.owner.graph.snapshot();
            let clef = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == "ClefInter" && n["origin"]["system"] == 1)
                .unwrap();
            let lifetime = crate::music_voice_lifetime::Lifetime::default();
            let ok = Context {
                graph: &mut owner.owner.graph,
                voices: &mut owner.voices,
                measures: &mut owner.measures,
                chords: &chords,
                narrow: &narrow,
                measure_index: mi,
                candidates: &candidates,
                order: &mut order,
                expected: 16,
                clef,
                interline: owner.owner.interline,
                lifetime: &lifetime,
                slot_times: &mut slot_times,
            }
            .read_grouped_slot_times(&compounds[0])
            .unwrap();
            assert_eq!(
                json!({"ok":ok,"members":slot_times.projection(&compounds[0]).unwrap()}),
                json!({"ok":case["output"]["ok"],"members":case["output"]["members"]}),
                "{}",
                case["name"]
            );
            assert_eq!(case["output"]["remaining_rookies"], json!([]));
            assert!(case["output"]["mapping"].is_null());
            assert_eq!(case["output"]["black"], json!([]));
        }
    }
    #[test]
    fn original_saved_distance_and_mapping_costs() {
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rookie-mapping/injection.json"
        ))
        .unwrap();
        for c in controls["cases"].as_array().unwrap() {
            let costs: Vec<Vec<i32>> = serde_json::from_value(c["costs"].clone()).unwrap();
            let (cost, links) = injection(&costs).unwrap();
            assert_eq!(json!(cost), c["cost"]);
            assert_eq!(json!(links), c["links"]);
        }
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rookie-mapping/constraints.json"
        ))
        .unwrap();
        for c in controls["cases"].as_array().unwrap() {
            let r: Vec<usize> = serde_json::from_value(c["rookies"].clone()).unwrap();
            let a: Vec<usize> = serde_json::from_value(c["actives"].clone()).unwrap();
            let black = serde_json::from_value::<Vec<Pair>>(c["black"].clone()).unwrap();
            let next = serde_json::from_value::<Vec<Pair>>(c["next"].clone()).unwrap();
            let distances =
                BTreeMap::from([((r[0], a[0]), c["distance"].as_i64().unwrap() as i32)]);
            assert_eq!(
                serde_json::to_value(mapping(&r, &a, &distances, &black, &next).unwrap()).unwrap(),
                c["mapping"]
            );
        }
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rookie-mapping/observed.json"
        ))
        .unwrap();
        for p in o["pages"].as_array().unwrap() {
            for m in p["measures"].as_array().unwrap() {
                for step in m["steps"].as_array().unwrap() {
                    let mut distances = BTreeMap::new();
                    for c in step["costs"].as_array().unwrap() {
                        let left = serde_json::from_value(c["left"].clone()).unwrap();
                        let right = serde_json::from_value(c["right"].clone()).unwrap();
                        let d = separated(
                            &left,
                            &right,
                            m["interline"].as_u64().unwrap() as usize,
                            false,
                        )
                        .unwrap();
                        assert_eq!(json!(d), c["distance"]);
                        distances.insert(
                            (
                                c["rookie"].as_u64().unwrap() as usize,
                                c["active"].as_u64().unwrap() as usize,
                            ),
                            d,
                        );
                    }
                    if !step["mapping"].is_null() {
                        let r: Vec<usize> =
                            serde_json::from_value(step["rookies"].clone()).unwrap();
                        let a: Vec<usize> =
                            serde_json::from_value(step["actives"].clone()).unwrap();
                        assert_eq!(
                            serde_json::to_value(mapping(&r, &a, &distances, &[], &[]).unwrap())
                                .unwrap(),
                            step["mapping"]
                        );
                    }
                }
            }
        }
    }
}
