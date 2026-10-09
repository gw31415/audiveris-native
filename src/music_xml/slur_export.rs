// SPDX-License-Identifier: AGPL-3.0-or-later
//! PartwiseBuilder Slur visitation on the same retained automatic two-ended owner.
use crate::{
    music_measures::{Chord, OwnedMusic},
    music_slurs::{
        Slur,
        lookup::{Curve, Side},
    },
    shared_sig::{Graph, Key},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
#[derive(Default)]
struct Numbers {
    active: BTreeMap<Key, u8>,
}
impl Numbers {
    fn visit(&mut self, key: Key) -> Option<u8> {
        if let Some(number) = self.active.remove(&key) {
            return Some(number);
        }
        let number = (1..=16).find(|n| !self.active.values().any(|held| held == n))?;
        self.active.insert(key, number);
        Some(number)
    }
}
fn tenths(dist: f64, interline: usize) -> Result<i32, String> {
    if !dist.is_finite() || interline == 0 {
        return Err("musicxml_slur_finite_scale_geometry_required".into());
    }
    let v = (10.0 * dist / interline as f64).round_ties_even();
    if v < i32::MIN as f64 || v > i32::MAX as f64 {
        return Err("musicxml_slur_geometry_budget".into());
    }
    Ok(v as i32)
}
#[allow(clippy::too_many_arguments)]
fn mark(
    curve: Curve,
    side: Side,
    continuation: bool,
    number: Option<u8>,
    head_left: i64,
    staff_top_at_end: f64,
    interline: usize,
) -> Result<Value, String> {
    let left = side == Side::Left;
    let end = curve.bezier[if left { 0 } else { 3 }];
    let ctrl = curve.bezier[if left { 1 } else { 2 }];
    let mut value = json!({"type":if continuation{"continue"}else if left{"start"}else{"stop"},
        "number":number,"placement":if left{Some(if curve.above{"above"}else{"below"})}else{None},
        "default-x":tenths(end[0]-head_left as f64,interline)?,
        "default-y":tenths(staff_top_at_end-end[1],interline)?});
    let suffix = if left && continuation { "2" } else { "" };
    value[format!("bezier-x{suffix}")] = json!(tenths(ctrl[0] - end[0], interline)?);
    value[format!("bezier-y{suffix}")] = json!(tenths(end[1] - ctrl[1], interline)?);
    Ok(value)
}
fn validate(m: &OwnedMusic, graph: &Graph, slur: &Arc<Slur>) -> Result<(), String> {
    let state = m
        .slurs
        .as_ref()
        .ok_or("musicxml_slur_retained_state_required")?;
    let (body, removed) = graph.retained_inter(slur.key)?;
    if removed || body["kind"] != "SlurInter" || body["is_tie"] != false {
        return Err("musicxml_slur_live_nontie_owner_required".into());
    }
    if body["manual"] == true || body["abnormal"] != false {
        return Err("musicxml_slur_manual_or_orphan_export_pending".into());
    }
    if !m.owner.systems.iter().any(|s| Arc::ptr_eq(s, &slur.system))
        || !m.owner.parts.iter().any(|p| Arc::ptr_eq(p, &slur.part))
        || slur.part.system != slur.system.id
        || slur.key.system != slur.system.id
        || body["part_id"] != slur.part.id
        || !state.slurs.iter().any(|s| Arc::ptr_eq(s, slur))
        || state
            .parts
            .iter()
            .filter(|p| {
                Arc::ptr_eq(&p.part, &slur.part) && p.slurs.iter().any(|s| Arc::ptr_eq(s, slur))
            })
            .count()
            != 1
    {
        return Err("musicxml_slur_actual_System_Part_membership_required".into());
    }
    if slur.endpoints[0].head == slur.endpoints[1].head {
        return Err("musicxml_slur_same_Head_both_sides_pending".into());
    }
    if !Arc::ptr_eq(&slur.endpoints[0].staff, &slur.endpoints[1].staff) {
        return Err("musicxml_slur_cross_Staff_export_pending".into());
    }
    let curve = slur.curve();
    if body["curve"] != json!(curve.bezier)
        || body["above"] != curve.above
        || curve.bezier.iter().flatten().any(|v| !v.is_finite())
    {
        return Err("musicxml_slur_same_retained_curve_required".into());
    }
    for (side, endpoint) in slur.endpoints.iter().enumerate() {
        if endpoint.head.system != slur.system.id
            || endpoint.chord.system != slur.system.id
            || !Arc::ptr_eq(&endpoint.measure.part, &slur.part)
            || !endpoint
                .measure
                .part
                .staffs
                .iter()
                .any(|s| Arc::ptr_eq(s, &endpoint.staff))
            || !m
                .owner
                .staffs
                .iter()
                .any(|s| Arc::ptr_eq(s, &endpoint.staff))
        {
            return Err("musicxml_slur_cross_System_or_unowned_endpoint_pending".into());
        }
        let chord = m
            .chords
            .iter()
            .find(|c| c.key == endpoint.chord)
            .ok_or("musicxml_slur_actual_endpoint_Chord_required")?;
        let measure = m
            .measures
            .get(chord.measure)
            .and_then(|m| m.geometry.as_ref())
            .ok_or("musicxml_slur_actual_endpoint_Measure_required")?;
        if !Arc::ptr_eq(measure, &endpoint.measure) {
            return Err("musicxml_slur_actual_endpoint_Measure_identity_required".into());
        }
        graph.require_actual_music_measure(measure)?;
        let hid = graph
            .id(endpoint.head)
            .ok_or("musicxml_slur_actual_endpoint_Head_required")?;
        if graph.slur_export_head(endpoint.chord, hid)? != endpoint.head {
            return Err("musicxml_slur_actual_endpoint_Head_identity_required".into());
        }
        let head = graph.retained_inter(endpoint.head)?.0;
        if head["staff_id"] != endpoint.staff.id {
            return Err("musicxml_slur_actual_endpoint_Staff_identity_required".into());
        }
        let relation = &slur.relations[side];
        if relation.side.index() != side
            || graph.fresh_slur_relation_id(slur.key, endpoint.head, relation.side)?
                != slur.relation_ids[side]
        {
            return Err("musicxml_slur_retained_relation_identity_required".into());
        }
        if !Arc::ptr_eq(
            graph.fresh_slur_relation_owner(slur.relation_ids[side])?,
            relation,
        ) {
            return Err("musicxml_slur_same_retained_relation_Arc_required".into());
        }
        let registered = graph.slur_export_relation(slur.relation_ids[side])?;
        for (field, expected) in relation.projection().as_object().unwrap() {
            if registered[field] != *expected {
                return Err("musicxml_slur_same_retained_relation_required".into());
            }
        }
    }
    Ok(())
}
pub(super) struct PartWriter<'a> {
    m: &'a OwnedMusic,
    part: u64,
    numbers: Numbers,
    expected: BTreeSet<Key>,
    seen: BTreeMap<Key, [bool; 2]>,
}
impl<'a> PartWriter<'a> {
    pub(super) fn new(m: &'a OwnedMusic, part: u64) -> Result<Self, String> {
        let mut expected = BTreeSet::new();
        for system in &m.owner.systems {
            for key in m.owner.graph.live_inter_keys(system.id, "SlurInter") {
                let state = m
                    .slurs
                    .as_ref()
                    .ok_or("musicxml_unowned_Slur_export_pending")?;
                let owners = state
                    .slurs
                    .iter()
                    .filter(|s| s.key == key)
                    .collect::<Vec<_>>();
                if owners.len() != 1 {
                    return Err("musicxml_slur_unique_retained_owner_required".into());
                }
                let owner = owners[0];
                if !m.measures.iter().any(|measure| {
                    measure
                        .geometry
                        .as_ref()
                        .is_some_and(|geometry| Arc::ptr_eq(&geometry.part, &owner.part))
                }) {
                    return Err("musicxml_slur_actual_export_Part_required".into());
                }
                if owner.part.id == part {
                    validate(m, &m.owner.graph, owner)?;
                    expected.insert(key);
                }
            }
        }
        Ok(Self {
            m,
            part,
            numbers: Numbers::default(),
            expected,
            seen: BTreeMap::new(),
        })
    }
    pub(super) fn note(&mut self, chord: &Chord, note_id: usize) -> Result<Value, String> {
        if !self.m.chords.iter().any(|c| std::ptr::eq(c, chord)) {
            return Err("musicxml_slur_actual_export_Chord_required".into());
        }
        let graph = &self.m.owner.graph;
        let head = graph.slur_export_head(chord.key, note_id)?;
        let mut result = Vec::new();
        for (key, relation_id, side) in graph.slur_export_incident(head)? {
            let slur = self
                .m
                .slurs
                .as_ref()
                .and_then(|s| s.slurs.iter().find(|s| s.key == key))
                .cloned()
                .ok_or("musicxml_slur_actual_retained_relation_owner_required")?;
            validate(self.m, graph, &slur)?;
            let index = side.index();
            let endpoint = &slur.endpoints[index];
            if slur.part.id != self.part
                || !self.expected.contains(&key)
                || endpoint.head != head
                || endpoint.chord != chord.key
                || slur.relation_ids[index] != relation_id
            {
                return Err("musicxml_slur_actual_export_endpoint_required".into());
            }
            let seen = self.seen.entry(key).or_insert([false; 2]);
            if seen[index] {
                return Err("musicxml_slur_duplicate_actual_endpoint_export".into());
            }
            seen[index] = true;
            let number = self.numbers.visit(key);
            let curve = slur.curve();
            let body = graph.retained_inter(head)?.0;
            let head_bounds: [i64; 4] = serde_json::from_value(body["bounds"].clone())
                .map_err(|_| "musicxml_slur_actual_Head_bounds_required")?;
            let x = curve.bezier[if side == Side::Left { 0 } else { 3 }][0];
            let top = endpoint
                .staff
                .lines
                .first()
                .ok_or("musicxml_slur_actual_Staff_curve_required")?
                .y(x);
            let mut item = mark(
                curve,
                side,
                false,
                number,
                head_bounds[0],
                top,
                self.m.owner.interline,
            )?;
            item["slur_key"] = json!(slur.key);
            item["relation_id"] = json!(relation_id);
            item["head_key"] = json!(head);
            item["head_bounds"] = json!(head_bounds);
            result.push(item);
        }
        Ok(json!(result))
    }
    pub(super) fn finish(self) -> Result<(), String> {
        if self
            .expected
            .iter()
            .any(|key| self.seen.get(key) != Some(&[true, true]))
        {
            return Err("musicxml_slur_endpoint_outside_actual_Voice_BEGIN_pending".into());
        }
        Ok(())
    }
}
#[cfg(test)]
pub(crate) mod tests;
