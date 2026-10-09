// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit image Tuplets retain their actual Stack until ordinary LINKS lookup.
use crate::{
    beam_dispatch,
    grid_ir::StaffGeometry,
    music_font_symbols::TupletShape,
    music_measure_layout::Stack,
    music_measures::{Chord, ChordKind, OwnedMusic},
    music_time::Fraction,
    shared_sig::{ChordRead, Graph, Key},
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) fn count(shape: &str) -> Option<usize> {
    match shape {
        "TUPLET_THREE" => Some(3),
        "TUPLET_SIX" => Some(6),
        _ => None,
    }
}
fn rectangle(value: &Value) -> Result<[i64; 4], String> {
    serde_json::from_value(value["bounds"].clone())
        .map_err(|_| "music_tuplet_actual_bounds_required".into())
}
fn center(b: [i64; 4]) -> [i64; 2] {
    [b[0] + b[2] / 2, b[1] + b[3] / 2]
}
fn overlap(a: [i64; 4], b: [i64; 4], axis: usize) -> i64 {
    (a[axis] + a[axis + 2]).min(b[axis] + b[axis + 2]) - a[axis].max(b[axis])
}
fn geometry(graph: &mut Graph, chord: &Chord) -> Result<ChordRead, String> {
    if chord.kind == ChordKind::Head {
        graph.bind_recognition_chord_caches(&[(chord.key, chord.stem)])?;
        graph.retained_chord_getters(chord.key)
    } else {
        let b = graph.retained_or_live_chord_bounds(chord.key)?;
        Ok(ChordRead {
            stem: None,
            bounds: Some(b),
            head: Some(center(b)),
            tail: Some(center(b)),
        })
    }
}
fn stack_at(m: &OwnedMusic, system: u64, point: [i64; 2]) -> Result<Option<Arc<Stack>>, String> {
    let around = crate::grid_staff_neighborhood::around(
        &m.owner.staffs,
        &m.owner.staff_areas,
        &m.owner.parts,
        system,
        point.map(|v| v as f64),
    )?;
    let Some(staff) = around.staffs.first() else {
        return Ok(None);
    };
    for stack in m
        .measure_layout
        .stacks
        .iter()
        .filter(|s| s.stack.system == system)
    {
        if let Some(measure) = stack
            .measures
            .iter()
            .find(|g| g.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)))
        {
            let [left, right] = measure.abscissae(staff)?;
            if point[0] > left && point[0] <= right {
                return Ok(Some(stack.stack.clone()));
            }
        }
    }
    Ok(None)
}
fn stack_measure(m: &OwnedMusic, stack: &Arc<Stack>) -> Result<usize, String> {
    m.measures
        .iter()
        .position(|measure| {
            measure
                .geometry
                .as_ref()
                .is_some_and(|g| Arc::ptr_eq(&g.stack, stack))
        })
        .ok_or_else(|| "music_tuplet_actual_shared_Stack_required".into())
}
fn factory_observation(m: &mut OwnedMusic, observation: Value) {
    let rows = &mut m.owner.prepared.report["explicit_tuplet_factories"];
    if rows.is_null() {
        *rows = json!([]);
    }
    rows.as_array_mut()
        .expect("owned Tuplet observations")
        .push(observation);
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn create(
    m: &mut OwnedMusic,
    graph: &mut Graph,
    glyph: &GlyphPixels,
    system: u64,
    shape: &str,
    grade: f64,
    glyph_id: usize,
) -> Result<Option<Key>, String> {
    count(shape).ok_or("music_tuplet_shape_required")?;
    let b = [
        glyph.left as i64,
        glyph.top as i64,
        glyph.width as i64,
        glyph.height as i64,
    ];
    let d = (3. * m.owner.interline as f64).round_ties_even() as i64;
    let lookup = [b[0] - d, b[1] - d, b[2] + 2 * d, b[3] + 2 * d];
    let mut nearby = Vec::new();
    for chord in m.chords.iter().filter(|c| c.key.system == system) {
        let (node, removed) = graph.retained_inter(chord.key)?;
        if removed {
            continue;
        }
        if node["id"] != chord.id {
            return Err("music_tuplet_actual_Chord_identity_required".into());
        }
        let bounds = graph.retained_or_live_chord_bounds(chord.key)?;
        if overlap(bounds, lookup, 0) > 0 && overlap(bounds, lookup, 1) > 0 {
            nearby.push(json!({"id":chord.id,"bounds":bounds}));
        }
    }
    nearby.sort_by_key(|n| n["bounds"][0].as_i64());
    if nearby.is_empty() {
        factory_observation(
            m,
            json!({"system":system,"shape":shape,"bounds":b,"lookup_bounds":lookup,"nearby":nearby,"returns_null":true}),
        );
        return Ok(None);
    }
    let stack = stack_at(m, system, center(b))?;
    let owner = stack.as_ref().map(|s| stack_measure(m, s)).transpose()?;
    let key = graph.next_symbol_key(system)?;
    let id = graph.vertex(key,json!({"kind":"TupletInter","shape":shape,"bounds":b,"glyph":beam_dispatch::glyph_json(glyph),"glyph_id":glyph_id,"grade":grade,"staff_id":null,"part_id":null,"manual":false,"implicit":false,"frozen":false,"abnormal":false}),false)?;
    // One unassigned set belongs to the shared Stack, not one copy per Part.
    if let Some(index) = owner {
        m.measures[index].stack_tuplets.push(key);
    }
    graph.set_tuplet_abnormal(key, true)?;
    factory_observation(
        m,
        json!({"system":system,"shape":shape,"bounds":b,"lookup_bounds":lookup,"nearby":nearby,"returns_null":false,"id":id,"stack":stack.as_ref().map(|s|s.index),"grade":grade}),
    );
    Ok(Some(key))
}

#[derive(Clone)]
struct Candidate {
    index: usize,
    id: usize,
    bounds: [i64; 4],
    head: [i64; 2],
    tail: [i64; 2],
    stem: Option<Key>,
    staffs: Vec<Arc<StaffGeometry>>,
}
impl Candidate {
    fn order(&self) -> (i64, i64, usize) {
        (self.bounds[0], self.bounds[1], self.id)
    }
    fn row(&self) -> Value {
        json!({"id":self.id,"bounds":self.bounds,"head":self.head,"tail":self.tail,"staffs":self.staffs.iter().map(|s|s.id).collect::<Vec<_>>()})
    }
}
fn chord_candidate(m: &OwnedMusic, graph: &mut Graph, index: usize) -> Result<Candidate, String> {
    let chord = &m.chords[index];
    let measure = m
        .measures
        .get(chord.measure)
        .ok_or("music_tuplet_actual_Measure_required")?;
    let layout = measure
        .geometry
        .as_ref()
        .ok_or("music_tuplet_actual_Measure_geometry_required")?;
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("music_tuplet_actual_SIG_required")?;
    let edges = snapshot["relations"].as_array().unwrap();
    let mut staffs = Vec::new();
    let mut members = 0;
    for edge in edges
        .iter()
        .filter(|e| e["kind"] == "Containment" && e["source"] == chord.id)
    {
        members += 1;
        let note = nodes
            .iter()
            .find(|n| n["id"] == edge["target"])
            .ok_or("music_tuplet_actual_member_required")?;
        let staff = layout
            .part
            .staffs
            .iter()
            .find(|s| note["staff_id"] == s.id)
            .ok_or("music_tuplet_actual_member_Staff_required")?;
        if !staffs.iter().any(|s| Arc::ptr_eq(s, staff)) {
            staffs.push(staff.clone());
        }
    }
    staffs.sort_by_key(|s| {
        layout
            .part
            .staffs
            .iter()
            .position(|p| Arc::ptr_eq(p, s))
            .unwrap()
    });
    if staffs.is_empty() {
        return Err("music_tuplet_actual_members_required".into());
    }
    // Cross-staff endpoint order and multi-rest geometry need their own
    // source comparison before an unsupported candidate can become no-link.
    if staffs.len() != 1 {
        return Err("music_tuplet_cross_staff_chord_lookup_pending".into());
    }
    if chord.kind == ChordKind::Rest && members != 1 {
        return Err("music_tuplet_multi_rest_chord_lookup_pending".into());
    }
    let read = geometry(graph, chord)?;
    Ok(Candidate {
        index,
        id: chord.id,
        bounds: read
            .bounds
            .ok_or("music_tuplet_actual_Chord_bounds_required")?,
        head: read.head.ok_or("music_tuplet_actual_Chord_head_required")?,
        tail: read.tail.ok_or("music_tuplet_actual_Chord_tail_required")?,
        stem: read.stem,
        staffs,
    })
}
fn distance(bounds: [i64; 4], point: [i64; 2]) -> f64 {
    let dx = (bounds[0] - point[0])
        .max(point[0] - (bounds[0] + bounds[2] - 1))
        .max(0) as f64;
    let dy = (bounds[1] - point[1])
        .max(point[1] - (bounds[1] + bounds[3] - 1))
        .max(0) as f64;
    dx * dx + dy * dy
}
fn squared(a: [i64; 2], b: [i64; 2]) -> f64 {
    let x = (a[0] - b[0]) as f64;
    let y = (a[1] - b[1]) as f64;
    x * x + y * y
}
fn beam_siblings(
    m: &OwnedMusic,
    graph: &mut Graph,
    chord: &Candidate,
    point: [i64; 2],
) -> Result<Vec<usize>, String> {
    let mut result = vec![chord.index];
    let Some(stem) = chord.stem else {
        return Ok(result);
    };
    if squared(point, chord.tail) >= squared(point, chord.head) {
        return Ok(result);
    }
    let snapshot = graph.snapshot();
    let edges = snapshot["relations"].as_array().unwrap();
    let nodes = snapshot["inters"].as_array().unwrap();
    let sid = graph.id(stem).ok_or("music_tuplet_actual_Stem_required")?;
    let mut smallest = None;
    for e in edges
        .iter()
        .filter(|e| e["kind"] == "BeamStemRelation" && e["target"] == sid)
    {
        let beam = nodes
            .iter()
            .find(|b| b["id"] == e["source"])
            .ok_or("music_tuplet_actual_Beam_required")?;
        let width = rectangle(beam)?[2];
        if smallest.is_none_or(|(_, old)| width < old) {
            smallest = Some((beam["id"].as_u64().unwrap() as usize, width));
        }
    }
    let Some((beam, _)) = smallest else {
        return Ok(result);
    };
    for edge in edges
        .iter()
        .filter(|e| e["kind"] == "BeamStemRelation" && e["source"] == beam)
    {
        for member in edges
            .iter()
            .filter(|e| e["kind"] == "ChordStemRelation" && e["target"] == edge["target"])
        {
            let index = m
                .chords
                .iter()
                .position(|c| member["source"] == c.id)
                .ok_or("music_tuplet_actual_beam_Chord_required")?;
            if !result.contains(&index) {
                result.push(index);
            }
        }
    }
    Ok(result)
}
fn lookup(
    m: &OwnedMusic,
    graph: &mut Graph,
    key: Key,
    stack: &Arc<Stack>,
) -> Result<(Vec<Key>, Value), String> {
    let node = graph.retained_inter(key)?.0;
    let point = center(rectangle(node)?);
    let around = crate::grid_staff_neighborhood::around(
        &m.owner.staffs,
        &m.owner.staff_areas,
        &m.owner.parts,
        key.system,
        point.map(|v| v as f64),
    )?;
    let mut candidates = Vec::new();
    for (mi, _) in m.measures.iter().enumerate().filter(|(_, g)| {
        g.geometry
            .as_ref()
            .is_some_and(|g| Arc::ptr_eq(&g.stack, stack))
    }) {
        for kind in [ChordKind::Head, ChordKind::Rest] {
            for (index, chord) in m
                .chords
                .iter()
                .enumerate()
                .filter(|(_, c)| c.measure == mi && c.kind == kind)
            {
                let (n, removed) = graph.retained_inter(chord.key)?;
                if removed {
                    continue;
                }
                if n["kind"] == "SmallChordInter" {
                    continue;
                }
                let candidate = chord_candidate(m, graph, index)?;
                if candidate
                    .staffs
                    .iter()
                    .any(|s| around.staffs.iter().any(|a| Arc::ptr_eq(s, a)))
                {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates.sort_by(|a, b| distance(a.bounds, point).total_cmp(&distance(b.bounds, point)));
    let (links, mut observation) = embraced(m, graph, key, candidates)?;
    observation["stack"] = json!(stack.index);
    Ok((links, observation))
}
fn embraced(
    m: &OwnedMusic,
    graph: &mut Graph,
    key: Key,
    mut candidates: Vec<Candidate>,
) -> Result<(Vec<Key>, Value), String> {
    let node = graph.retained_inter(key)?.0;
    let expected_count = count(
        node["shape"]
            .as_str()
            .ok_or("music_tuplet_shape_required")?,
    )
    .ok_or("music_tuplet_shape_required")?;
    let point = center(rectangle(node)?);
    let around_rows = candidates.iter().map(Candidate::row).collect::<Vec<_>>();
    let mut i = 0;
    while i < candidates.len() {
        let mut j = i + 1;
        let mut removed_first = false;
        while j < candidates.len() {
            if overlap(candidates[i].bounds, candidates[j].bounds, 0) > 0 {
                let d1 = (center(candidates[i].bounds)[1] - point[1]).abs();
                let d2 = (center(candidates[j].bounds)[1] - point[1]).abs();
                if d1 < d2 {
                    candidates.remove(j);
                } else {
                    candidates.remove(i);
                    removed_first = true;
                    break;
                }
            } else {
                j += 1;
            }
        }
        if !removed_first {
            i += 1;
        }
    }
    let target = candidates
        .iter()
        .find(|c| m.chords[c.index].kind == ChordKind::Head)
        .and_then(|c| c.staffs.first())
        .cloned();
    let filtered = candidates.iter().map(Candidate::row).collect::<Vec<_>>();
    let mut collected: Vec<Candidate> = Vec::new();
    let mut base = Fraction::new(i32::MAX as u32, 1)?;
    let mut expected = base;
    let mut total = Fraction::ZERO;
    let mut status = None;
    let mut trace = Vec::new();
    for candidate in candidates {
        if !target
            .as_ref()
            .is_some_and(|s| Arc::ptr_eq(s, &candidate.staffs[0]))
        {
            continue;
        }
        if collected.iter().any(|c| c.id == candidate.id) {
            continue;
        }
        let siblings = beam_siblings(m, graph, &candidate, point)?;
        for index in &siblings {
            if collected.iter().any(|c| c.index == *index) {
                continue;
            }
            let chord = chord_candidate(m, graph, *index)?;
            let duration = graph
                .sans_tuplet_chord_duration(m.chords[*index].key, false)?
                .ok_or("music_tuplet_null_Chord_duration")?;
            total = total.plus(duration)?;
            if duration < base {
                base = duration;
                expected = base.times(Fraction::new(expected_count as u32, 1)?)?;
            }
            collected.push(chord);
            collected.sort_by_key(Candidate::order);
        }
        status = if collected.len() > expected_count {
            Some("TOO_MANY")
        } else if total > expected {
            Some("TOO_LONG")
        } else if total == expected {
            Some(
                if point[0] >= collected.first().unwrap().tail[0]
                    && point[0] <= collected.last().unwrap().tail[0]
                {
                    "OK"
                } else {
                    "OUTSIDE"
                },
            )
        } else {
            None
        };
        trace.push(json!({"chord_id":candidate.id,"beam_siblings":siblings.iter().map(|i|m.chords[*i].id).collect::<Vec<_>>(),"base":base.to_string(),"total":total.to_string(),"expected_total":expected.to_string(),"status":status,"chords":collected.iter().map(Candidate::row).collect::<Vec<_>>()}));
        if status.is_some() {
            break;
        }
    }
    let links = if status == Some("OK") {
        collected.iter().map(|c| m.chords[c.index].key).collect()
    } else {
        Vec::new()
    };
    Ok((
        links,
        json!({"id":graph.id(key),"bounds":rectangle(graph.retained_inter(key)?.0)?,"around":around_rows,"filtered":filtered,"target_staff":target.map(|s|s.id),"collector":trace,"status":status,"search_links":if status==Some("OK"){collected.iter().map(|c|c.id).collect::<Vec<_>>()}else{Vec::new()}}),
    ))
}

pub(crate) type ContainerPlan = Vec<(usize, Vec<Key>, Vec<Key>)>;

fn raw_callback_frame(m: &OwnedMusic, graph: &Graph, key: Key) -> Result<Value, String> {
    let (tuplet, removed) = graph.retained_inter(key)?;
    let mut chords = Vec::new();
    for chord in m.chords.iter().filter(|c| c.key.system == key.system) {
        let (node, removed) = graph.retained_inter(chord.key)?;
        chords.push(json!({"id":chord.id,"bounds":node["bounds"],"voice":node["voice"],"slot_id":node["slot_id"],"time_offset":node["time_offset"],"removed":removed}));
    }
    Ok(
        json!({"tuplet":tuplet,"removed":removed,"chords":chords,"expected":m.measures.iter().filter(|m|m.system==key.system).map(|m|json!({"part":m.part,"index":m.index,"expected":m.timing.expected.map(crate::music_time::format_fraction)})).collect::<Vec<_>>()}),
    )
}

fn check_abnormal(m: &OwnedMusic, graph: &mut Graph, key: Key) -> Result<Value, String> {
    let mut candidates = Vec::new();
    for chord in graph.tuplet_chords(key)? {
        let index = m
            .chords
            .iter()
            .position(|c| c.key == chord)
            .ok_or("music_tuplet_actual_callback_Chord_required")?;
        candidates.push(chord_candidate(m, graph, index)?);
    }
    // getChords preserves relation order on equal left abscissae. It does not
    // reuse lookup's distance ordering or include unlinked nearby candidates.
    candidates.sort_by_key(|c| c.bounds[0]);
    let (embraced, observation) = embraced(m, graph, key, candidates)?;
    graph.set_tuplet_abnormal(key, embraced.is_empty())?;
    Ok(observation)
}

fn remove_from_containers(
    m: &OwnedMusic,
    graph: &mut Graph,
    key: Key,
    stack: &Arc<Stack>,
    containers: &mut ContainerPlan,
) -> Result<(), String> {
    if graph.retained_inter(key)?.0["frozen"] == true {
        return Err("music_removal_frozen_inter".into());
    }
    let canonical = stack_measure(m, stack)?;
    let members = m
        .measures
        .iter()
        .enumerate()
        .filter(|(_, measure)| {
            measure
                .geometry
                .as_ref()
                .is_some_and(|g| Arc::ptr_eq(&g.stack, stack))
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    for &mi in &members {
        containers[mi].1.retain(|k| *k != key);
    }
    containers[canonical].2.retain(|k| *k != key);
    let assigned = members
        .iter()
        .flat_map(|mi| containers[*mi].1.iter().copied())
        .collect::<Vec<_>>();
    let mut all = assigned.clone();
    all.extend(containers[canonical].2.iter().copied());
    graph.remove_owned_tuplet(key, &assigned, &all)
}

pub(crate) fn link_and_remove(
    m: &OwnedMusic,
    graph: &mut Graph,
) -> Result<(Value, ContainerPlan), String> {
    let mut next = graph.clone();
    let mut containers = m
        .measures
        .iter()
        .enumerate()
        .map(|(i, m)| (i, m.tuplets.clone(), m.stack_tuplets.clone()))
        .collect::<Vec<_>>();
    if m.owner
        .systems
        .iter()
        .all(|s| graph.live_inter_keys(s.id, "TupletInter").is_empty())
        && containers
            .iter()
            .all(|(_, assigned, unassigned)| assigned.is_empty() && unassigned.is_empty())
    {
        return Ok((
            json!({"tuplets":[],"linked_positive_complete":false}),
            containers,
        ));
    }
    let mut observations = Vec::new();
    let mut visited = Vec::new();
    for held in &m.measure_layout.stacks {
        let canonical = stack_measure(m, &held.stack)?;
        let members = m
            .measures
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.geometry
                    .as_ref()
                    .is_some_and(|g| Arc::ptr_eq(&g.stack, &held.stack))
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let mut keys = Vec::new();
        for &mi in &members {
            if mi != canonical && !containers[mi].2.is_empty() {
                return Err("music_tuplet_shared_Stack_single_container_required".into());
            }
            for &key in &containers[mi].1 {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        for &key in &containers[canonical].2 {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        for key in keys {
            if visited.contains(&key) {
                return Err("music_tuplet_unique_actual_Stack_required".into());
            }
            visited.push(key);
            let (implicit, manual) = next.tuplet_status(key)?;
            if implicit {
                return Err("music_tuplet_implicit_LINKS_relookup_pending".into());
            }
            // Reprocessing must preserve manual edges and remove each old
            // automatic edge through its callback; it is not a fresh link pass.
            if !next.tuplet_chords(key)?.is_empty() {
                return Err("music_tuplet_existing_links_relookup_pending".into());
            }
            let (links, mut observation) = lookup(m, &mut next, key, &held.stack)?;
            let shape = match next.retained_inter(key)?.0["shape"].as_str() {
                Some("TUPLET_THREE") => TupletShape::Three,
                Some("TUPLET_SIX") => TupletShape::Six,
                _ => return Err("music_tuplet_shape_required".into()),
            };
            let mut additions = Vec::new();
            for chord in &links {
                let mut callback = Value::Null;
                let edge = next.add_explicit_chord_tuplet(*chord, key, shape, |graph, key| {
                    let lookup = check_abnormal(m, graph, key)?;
                    callback = json!({"lookup":lookup,"after":raw_callback_frame(m,graph,key)?});
                    Ok(())
                })?;
                callback["relation_id"] = json!(edge);
                additions.push(callback);
            }
            observation["additions"] = json!(additions);
            observation["removed"] = json!(links.is_empty() && !manual);
            if links.is_empty() && !manual {
                remove_from_containers(m, &mut next, key, &held.stack, &mut containers)?;
            }
            observations.push(observation);
        }
    }
    for system in &m.owner.systems {
        for key in next.live_inter_keys(system.id, "TupletInter") {
            if !visited.contains(&key) {
                return Err("music_tuplet_unassigned_stack_lookup_pending".into());
            }
        }
    }
    *graph = next;
    Ok((
        json!({"tuplets":observations,"linked_positive_complete":false}),
        containers,
    ))
}

pub(crate) fn contextualize_and_purge(
    m: &OwnedMusic,
    graph: &mut Graph,
    containers: &mut ContainerPlan,
) -> Result<Value, String> {
    let mut next = graph.clone();
    let mut planned = containers.clone();
    if planned.len() != m.measures.len()
        || planned.iter().enumerate().any(|(i, (mi, _, _))| i != *mi)
    {
        return Err("music_tuplet_actual_container_plan_required".into());
    }
    let keys = m
        .owner
        .systems
        .iter()
        .flat_map(|s| next.live_inter_keys(s.id, "TupletInter"))
        .collect::<Vec<_>>();
    let mut observations = Vec::new();
    for &key in &keys {
        let (implicit, manual) = next.tuplet_status(key)?;
        if implicit {
            return Err("music_tuplet_implicit_LINKS_context_pending".into());
        }
        if manual && next.tuplet_chords(key)?.is_empty() {
            return Err("music_tuplet_unlinked_manual_pending".into());
        }
        observations.push(next.contextualize_explicit_tuplet(key)?);
    }
    // Context is materialized for every target before the first weak-removal
    // callback. Container membership is removed before the SIG edges disappear.
    for (&key, observation) in keys.iter().zip(&mut observations) {
        let node = next.retained_inter(key)?.0;
        let weak = node["contextual_grade"]
            .as_f64()
            .ok_or("music_tuplet_context_required")?
            < 0.5
            && node["frozen"] != true;
        observation["removed"] = json!(weak);
        if weak {
            let owner = planned
                .iter()
                .position(|(_, a, u)| a.contains(&key) || u.contains(&key))
                .ok_or("music_tuplet_actual_removal_Stack_required")?;
            let stack = m.measures[owner]
                .geometry
                .as_ref()
                .ok_or("music_tuplet_actual_Measure_geometry_required")?
                .stack
                .clone();
            remove_from_containers(m, &mut next, key, &stack, &mut planned)?;
        }
    }
    if m.owner
        .systems
        .iter()
        .any(|s| !next.live_inter_keys(s.id, "TupletInter").is_empty())
    {
        return Err("music_tuplet_surviving_linked_rhythm_pending".into());
    }
    *graph = next;
    *containers = planned;
    Ok(json!({"contextualization":observations,"surviving_positive_rhythm_complete":false}))
}

#[cfg(test)]
mod tests;
