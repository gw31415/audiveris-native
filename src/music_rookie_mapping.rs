// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original separated VoiceDistance, ChordsMapper and small InjectionSolver.
//! Owned image adapter currently admits one incoming/one Voice and no implicit
//! tuplets. Implicit-tuplet analysis, new-Voice inference
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
}
struct ValidatedMapping {
    mapping: Option<Mapping>,
    black: Vec<Pair>,
    narrow: Vec<NarrowTime>,
    applied: bool,
}
fn decision(input: ValidationInput, mapping: Option<Mapping>, applied: bool) -> ValidatedMapping {
    ValidatedMapping {
        mapping,
        black: input.black,
        narrow: input.narrow,
        applied,
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
                    let end = input
                        .active_ends
                        .get(&pair.active)
                        .copied()
                        .flatten()
                        .ok_or("music_mapping_unknown_active_time_analysis_pending")?;
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
// Original itself documents brute force for small collections. The production
// adapter is one incoming; larger unsupported searches refuse by an explicit
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
    pub measures: &'a [Measure],
    pub measure_index: usize,
    pub compounds: &'a [CompoundSlot],
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
    fn slot_x(&self, snapshot: &Value, slot: &CompoundSlot) -> Result<i64, String> {
        let xs = slot
            .chord_ids
            .iter()
            .map(|&id| bounds(node(snapshot, id)?).map(|b| b[0] + b[2] / 2))
            .collect::<Result<Vec<_>, _>>()?;
        if xs.is_empty() {
            return Err("music_mapping_compound_center_required".into());
        }
        Ok((xs.iter().sum::<i64>() as f64 / xs.len() as f64
            - self.measures[self.measure_index].left as f64)
            .round_ties_even() as i64)
    }
    pub fn map_slot(&mut self, slot: &CompoundSlot) -> Result<Value, String> {
        if slot.chord_ids.len() != 1 || slot.members.len() != 1 {
            return Err("music_mapping_general_compound_retry_pending".into());
        }
        let snapshot = self.graph.snapshot();
        let edges = snapshot["relations"].as_array().unwrap();
        let mut times: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        for &id in &slot.chord_ids {
            if let Some(t) = time(node(&snapshot, id)?)? {
                times.entry(t).or_default().push(id);
            }
        }
        if times.len() > 1 {
            return Err("music_mapping_implicit_tuplet_or_conflicting_slot_times_pending".into());
        }
        if let Some(&known) = times.keys().next() {
            // Empty observations must not erase a retained narrow clock.
            self.slot_times.set(slot.members[0].id, Some(known))?;
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
        let current_x = self.slot_x(&snapshot, slot)?;
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
                .compounds
                .iter()
                .rev()
                .find(|s| s.chord_ids.contains(&last))
                .ok_or("music_mapping_actual_candidate_slot_required")?;
            if self.slot_x(&snapshot, previous)? > current_x {
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
            if actives.is_empty() {
                return Err("music_mapping_new_voice_time_inference_pending".into());
            }
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
            })?;
            validation = json!({"black":validated.black,"applied":validated.applied});
            let map = validated
                .mapping
                .ok_or("music_mapping_new_voice_time_inference_pending")?;
            if !validated.applied || map.pairs.len() != rookies.len() {
                return Err("music_mapping_new_voice_time_inference_pending".into());
            }
            for narrow in validated.narrow {
                self.slot_times.set(narrow.id, narrow.time_offset)?;
            }
            for pair in &map.pairs {
                let end = active_ends[&pair.active].ok_or("music_mapping_active_time_required")?;
                let own = self
                    .voices
                    .iter()
                    .position(|v| {
                        v.measure == self.measure_index && v.chord_ids.contains(&pair.active)
                    })
                    .ok_or("music_mapping_active_voice_required")?;
                crate::music_voices::assign_chord_voice(
                    self.graph,
                    self.voices,
                    self.measures,
                    pair.rookie,
                    own,
                )?;
                crate::music_chord_time::set_and_push(
                    self.graph,
                    self.voices,
                    self.measures,
                    pair.rookie,
                    Some(end),
                )?;
            }
            result = serde_json::to_value(map).unwrap();
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
        Ok(
            json!({"slot_id":slot.id,"known_times":times.iter().map(|(t,ids)|json!({"time":crate::music_time::format_ticks(*t),"chord_ids":ids})).collect::<Vec<_>>(),"rookies":rookies,"actives":actives,"costs":costs,"mapping":result,"ok":true,"after":after,"validation":validation}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
        }
    }
    #[test]
    fn original_slot_times_reselect_compatible_voices_or_leave_rookies_unassigned() {
        let originals = [
            include_str!("../fixtures/official-mapping-retry/observed.json"),
            include_str!("../fixtures/official-mapping-time-analysis/observed.json"),
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
            assert_eq!(
                json!({"mapping":result.mapping,"black":result.black,"remaining_rookies":remaining,"narrow":narrow}),
                case["output"],
                "{}",
                case["name"]
            );
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
