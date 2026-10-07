// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use crate::music_measures::Measure;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Membership {
    system: u64,
    part: u64,
    staff: u64,
    index: usize,
    left: i64,
    right: i64,
    dots: Vec<usize>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct Ownership {
    measures: Vec<Membership>,
    staff_parts: BTreeMap<(u64, u64), u64>,
}
impl Ownership {
    pub(super) fn contains_chord(&self, system: u64, chord: &Value) -> bool {
        self.measures.iter().any(|m| {
            m.system == system
                && chord["measure_index"] == m.index
                && chord["part_id"] == m.part
                && chord["staff_id"] == m.staff
        })
    }
}
/// AbstractInter setters for retained ordinary note/dot ownership. Higher-level
/// correction commands must separately perform their original task lifecycle.
#[derive(Clone, Debug)]
pub enum NoteOwnerEdit {
    Part(Option<u64>),
    Staff(Option<u64>),
    Grade(f64),
    Bounds([i64; 4]),
}
impl Graph {
    pub fn edit_note_owner(&mut self, key: Key, edit: NoteOwnerEdit) -> Result<(), String> {
        let id = self.id(key).ok_or("music_note_owner_identity_required")?;
        let v = &self.vertices[id - 1];
        if !matches!(
            (v.key.phase, v.data["kind"].as_str()),
            (Phase::Head, Some("HeadInter"))
                | (Phase::Symbol, Some("RestInter" | "AugmentationDotInter"))
        ) {
            return Err("music_note_owner_edit_pending".into());
        }
        match edit {
            NoteOwnerEdit::Part(part) => {
                if let Some(part) = part
                    && !self.dot_ownership.as_ref().is_some_and(|owner| {
                        owner
                            .measures
                            .iter()
                            .any(|m| m.system == key.system && m.part == part)
                    })
                {
                    return Err("music_note_owned_part_required".into());
                }
                self.vertices[id - 1].cached_part = part;
            }
            NoteOwnerEdit::Staff(staff) => {
                if let Some(staff) = staff
                    && !self
                        .dot_ownership
                        .as_ref()
                        .is_some_and(|owner| owner.staff_parts.contains_key(&(key.system, staff)))
                {
                    return Err("music_note_owned_staff_required".into());
                }
                self.vertices[id - 1].data["staff_id"] = json!(staff);
            }
            NoteOwnerEdit::Grade(grade) => {
                if !grade.is_finite() || !(0.0..=1.0).contains(&grade) {
                    return Err("music_note_grade_required".into());
                }
                self.vertices[id - 1].data["grade"] = json!(grade);
            }
            NoteOwnerEdit::Bounds(bounds) => {
                if bounds[2] <= 0
                    || bounds[3] <= 0
                    || bounds[0].checked_add(bounds[2]).is_none()
                    || bounds[1].checked_add(bounds[3]).is_none()
                {
                    return Err("music_note_bounds_required".into());
                }
                self.vertices[id - 1].data["bounds"] = json!(bounds);
            }
        }
        // These setters do not relocate Measure membership or invalidate Part/ctx.
        // Doing so here would hide the original task/callback lifecycle boundaries.
        Ok(())
    }
    pub(crate) fn bind_dot_measures(&mut self, measures: &[Measure]) -> Result<(), String> {
        let mut ownership = Ownership::default();
        let mut identities = BTreeSet::new();
        let mut system_staves = BTreeMap::new();
        for m in measures {
            if m.left >= m.right
                || m.part == 0
                || m.staff == 0
                || !identities.insert((m.system, m.part, m.index))
            {
                return Err("music_dot_owned_measure_identity_required".into());
            }
            if system_staves
                .insert(m.system, (m.staff, m.part))
                .is_some_and(|old| old != (m.staff, m.part))
            {
                return Err("music_dot_multistaff_stack_filter_pending".into());
            }
            ownership.staff_parts.insert((m.system, m.staff), m.part);
            ownership.measures.push(Membership {
                system: m.system,
                part: m.part,
                staff: m.staff,
                index: m.index,
                left: m.left,
                right: m.right,
                dots: Vec::new(),
            });
        }
        ownership.measures.sort_by_key(|m| (m.system, m.index));
        if let Some(old) = &self.dot_ownership {
            if old.staff_parts != ownership.staff_parts
                || old.measures.iter().zip(&ownership.measures).any(|(a, b)| {
                    (a.system, a.part, a.index, a.left, a.right)
                        != (b.system, b.part, b.index, b.left, b.right)
                })
                || old.measures.len() != ownership.measures.len()
            {
                return Err("music_dot_measure_rebinding_pending".into());
            }
            return Ok(());
        }
        self.dot_ownership = Some(ownership);
        Ok(())
    }
    fn dot_center(&self, id: usize) -> Result<[i64; 2], String> {
        let b: [i64; 4] = serde_json::from_value(self.vertices[id - 1].data["bounds"].clone())
            .map_err(|_| "music_dot_bounds_required")?;
        Ok([b[0] + b[2] / 2, b[1] + b[3] / 2])
    }
    fn stack_for_dot(&self, id: usize) -> Result<Option<usize>, String> {
        let Some(owner) = &self.dot_ownership else {
            return Ok(None);
        };
        let x = self.dot_center(id)?[0];
        Ok(owner.measures.iter().position(|m| {
            m.system == self.vertices[id - 1].key.system && m.left < x && x <= m.right
        }))
    }
    pub(super) fn dot_edit_stack_target(&self, inter: Key) -> Result<Option<Value>, String> {
        let id = self.id(inter).ok_or("editing_inter_identity_required")?;
        if self.vertices[id - 1].data["kind"] != "AugmentationDotInter" {
            return Err("editing_stack_task_class_pending".into());
        }
        self.inter_edit_stack_target(inter)
    }
    pub(super) fn inter_edit_stack_target(&self, inter: Key) -> Result<Option<Value>, String> {
        let id = self.id(inter).ok_or("editing_inter_identity_required")?;
        // A retained empty HeadChord can have no center after undo. Original
        // RhythmsStep skips InterTasks with a null center; partner inters still
        // select their real stack in the declared relation task closure.
        if self.vertices[id - 1].data["kind"] == "HeadChordInter"
            && self.vertices[id - 1].data["bounds"].is_null()
        {
            return Ok(None);
        }
        let owner = self
            .dot_ownership
            .as_ref()
            .ok_or("editing_owned_stack_selection_required")?;
        // SystemInfo.getStackAt uses current center, not cached Part membership.
        // Binding already enforces the retained one-staff/Part stack geometry.
        Ok(self.stack_for_dot(id)?.map(|index| {
            let m = &owner.measures[index];
            json!({"system_id":m.system,"measure_index":m.index,"left":m.left,"right":m.right})
        }))
    }
    pub(crate) fn augmentation_part(&mut self, key: Key) -> Result<Option<u64>, String> {
        let id = self
            .id(key)
            .ok_or("music_dot_part_inter_identity_required")?;
        if self.vertices[id - 1].data["kind"] != "AugmentationDotInter" {
            return Err("music_dot_part_inter_required".into());
        }
        self.part_for_inter(id)
    }
    pub(super) fn part_for_inter(&mut self, id: usize) -> Result<Option<u64>, String> {
        let v = &self.vertices[id - 1];
        if v.cached_part.is_some() {
            return Ok(v.cached_part);
        }
        let Some(owner) = &self.dot_ownership else {
            return Ok(None);
        };
        let system = v.key.system;
        if v.data["kind"] == "AugmentationDotInter" {
            if let Some(note) = self
                .incident_edges(id)
                .into_iter()
                .map(|i| &self.edges[i - 1])
                .find(|e| {
                    !e.removed
                        && e.key.kind == "AugmentationRelation"
                        && (e.key.source == id || e.key.target == id)
                })
                .map(|e| {
                    if e.key.source == id {
                        e.key.target
                    } else {
                        e.key.source
                    }
                })
            {
                let n = &self.vertices[note - 1];
                if n.key.system != system
                    || !matches!(
                        (n.key.phase, n.data["kind"].as_str()),
                        (Phase::Head, Some("HeadInter")) | (Phase::Symbol, Some("RestInter"))
                    )
                {
                    return Err("music_dot_typed_part_note_required".into());
                }
                let part = self.part_for_inter(note)?;
                self.vertices[id - 1].cached_part = part;
                return Ok(part);
            }
            let x = self.dot_center(id)?[0];
            let incident = self
                .incident_edges(id)
                .into_iter()
                .map(|i| &self.edges[i - 1])
                .filter(|e| e.key.kind == "DoubleDotRelation")
                .map(|e| {
                    if e.key.source == id {
                        e.key.target
                    } else {
                        e.key.source
                    }
                })
                .collect::<Vec<_>>();
            for first in incident {
                if self.dot_center(first)?[0] < x {
                    let part = self.part_for_inter(first)?;
                    self.vertices[id - 1].cached_part = part;
                    return Ok(part);
                }
            }
        }
        let part = v.data["staff_id"]
            .as_u64()
            .and_then(|staff| owner.staff_parts.get(&(system, staff)))
            .copied();
        self.vertices[id - 1].cached_part = part;
        Ok(part)
    }
    pub(super) fn add_dot_to_measure(&mut self, id: usize) -> Result<(), String> {
        if let Some(index) = self.stack_for_dot(id)?
            && let Some(part) = self.augmentation_part(self.vertices[id - 1].key)?
        {
            let m = &mut self.dot_ownership.as_mut().unwrap().measures[index];
            if m.part != part {
                return Err("music_dot_part_stack_owner_required".into());
            }
            if !m.dots.contains(&id) {
                m.dots.push(id)
            }
        }
        Ok(())
    }
    pub(super) fn remove_dot_from_measure(&mut self, id: usize) -> Result<(), String> {
        if let Some(index) = self.stack_for_dot(id)?
            && let Some(part) = self.augmentation_part(self.vertices[id - 1].key)?
        {
            let m = &mut self.dot_ownership.as_mut().unwrap().measures[index];
            if m.part != part {
                return Err("music_dot_part_stack_owner_required".into());
            }
            m.dots.retain(|n| *n != id);
        }
        Ok(())
    }
    pub(crate) fn populate_dot_measures(&mut self) -> Result<(), String> {
        if self.dot_ownership.is_none() {
            return Err("music_dot_owned_measures_required".into());
        }
        let mut next = self.clone();
        let mut dots = next
            .vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.removed && v.data["kind"] == "AugmentationDotInter")
            .map(|(i, _)| Ok((next.dot_center(i + 1)?[0], i + 1)))
            .collect::<Result<Vec<_>, String>>()?;
        dots.sort_by_key(|(x, _)| *x);
        for (_, id) in dots {
            next.add_dot_to_measure(id)?;
        }
        *self = next;
        Ok(())
    }
    pub(super) fn dot_ownership_snapshot(&self) -> Value {
        let owner = self.dot_ownership.as_ref().unwrap();
        json!({"scope":"retained ordinary one-staff zero-skew dot subset of PageRhythm.populateFRATs; full FRAT/Part/stack lifecycle partial","measures":owner.measures.iter().map(|m|json!({"system_id":m.system,"part_id":m.part,"staff_id":m.staff,"measure_index":m.index,"left":m.left,"right":m.right,"augmentation_dot_ids":m.dots})).collect::<Vec<_>>(),"part_cache":self.vertices.iter().enumerate().filter(|(_,v)|v.data["kind"]=="AugmentationDotInter").map(|(i,v)|json!({"id":i+1,"part_id":v.cached_part})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ownership_state(snapshot: &Value, ids: &[usize]) -> Value {
        json!(ids.iter().map(|id| {
            let n=snapshot["inters"].as_array().unwrap().iter().find(|n|n["id"]==*id);
            let owner=&snapshot["augmentation_dot_ownership"];
            let cached=owner["part_cache"].as_array().unwrap().iter().find(|n|n["id"]==*id).map(|n|n["part_id"].clone()).unwrap_or(Value::Null);
            let member=owner["measures"].as_array().unwrap().iter().any(|m|m["augmentation_dot_ids"].as_array().unwrap().contains(&json!(id)));
            json!({"cached_part_id":cached,"staff_id":n.map(|n|n["staff_id"].clone()).unwrap_or(Value::Null),"measure_member":member,"registered":n.is_some_and(|n|n["registered"]!=false),"removed":n.is_some_and(|n|n["lifecycle_removed"]==true)})
        }).collect::<Vec<_>>())
    }
    fn live_state(graph: &Graph, ids: &[usize]) -> Value {
        let mut s = graph.snapshot();
        s["inters"] = json!(
            graph
                .vertices
                .iter()
                .map(|v| {
                    let mut n = v.data.clone();
                    n["registered"] = json!(!v.removed);
                    n["lifecycle_removed"] = json!(v.removed || v.removing);
                    n
                })
                .collect::<Vec<_>>()
        );
        ownership_state(&s, ids)
    }
    #[test]
    fn original_lazy_part_measure_membership_precedes_relation_callbacks() {
        let original: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-part-ownership/observed.json"
        ))
        .unwrap();
        for c in original["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let measure = Measure {
                system: 1,
                part: c["measure"]["part_id"].as_u64().unwrap(),
                staff: c["measure"]["staff_id"].as_u64().unwrap(),
                index: 1,
                left: c["measure"]["left"].as_i64().unwrap(),
                right: c["measure"]["right"].as_i64().unwrap(),
                timing: Default::default(),
            };
            graph.bind_dot_measures(&[measure]).unwrap();
            let hk = Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            };
            let ck = Key {
                system: 1,
                phase: Phase::Chord,
                local: 1,
            };
            graph.vertex(hk, c["head"].clone(), false).unwrap();
            graph.vertex(ck,json!({"kind":"HeadChordInter","bounds":c["chord_bounds"],"part_id":1,"staff_id":1,"duration":null,"voice":null,"dots_number":0}),false).unwrap();
            graph
                .relations(
                    1,
                    Phase::Chord,
                    vec![(ck, hk, json!({"kind":"Containment"}))],
                )
                .unwrap();
            let dots = [
                Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: 1,
                },
                Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: 2,
                },
            ];
            let ids = [graph.vertices.len() + 1, graph.vertices.len() + 2];
            let first = json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":c["dot_bounds"][0],"grade":0.8,"glyph_id":1});
            graph.vertex(dots[0], first.clone(), false).unwrap();
            let mut stages =
                vec![json!({"action":"SIG.addVertex(first)","ownership":live_state(&graph,&ids)})];
            let mut first = first;
            first["staff_id"] = c["head"]["staff_id"].clone();
            graph.vertex(dots[0], first, false).unwrap();
            stages.push(json!({"action":"first.setStaff","ownership":live_state(&graph,&ids)}));
            graph
                .relations(
                    1,
                    Phase::Symbol,
                    vec![(
                        dots[0],
                        hk,
                        json!({"kind":"AugmentationRelation","grade":0.7}),
                    )],
                )
                .unwrap();
            stages.push(
                json!({"action":"first support applied","ownership":live_state(&graph,&ids)}),
            );
            graph.vertex(dots[1],json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":c["dot_bounds"][1],"grade":0.8,"glyph_id":2}),false).unwrap();
            stages.push(
                json!({"action":"SIG.addVertex(second)","ownership":live_state(&graph,&ids)}),
            );
            graph
                .relations(
                    1,
                    Phase::Symbol,
                    vec![(
                        dots[1],
                        dots[0],
                        json!({"kind":"DoubleDotRelation","grade":0.7}),
                    )],
                )
                .unwrap();
            stages.push(
                json!({"action":"second support applied","ownership":live_state(&graph,&ids)}),
            );
            if c["mode"] == "factory_order_then_FRAT" {
                graph.populate_dot_measures().unwrap();
                stages.push(json!({"action":"PageRhythm.populateFRATs","ownership":live_state(&graph,&ids)}));
                assert_eq!(json!(stages), c["stages"]);
                let once = graph.snapshot();
                graph.populate_dot_measures().unwrap();
                assert_eq!(graph.snapshot(), once);
            } else {
                // The saved XML Measure collection is independent of transient Part caches.
                graph.dot_ownership.as_mut().unwrap().measures[0].dots = ids.to_vec();
                if c["mode"] == "saved_warm" {
                    for k in dots {
                        graph.augmentation_part(k).unwrap();
                    }
                }
            }
            graph.count_owned_augmentation(ck).unwrap();
            assert_eq!(live_state(&graph, &ids), c["before"]);
            let edges = graph.snapshot()["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "AugmentationRelation" || e["kind"] == "DoubleDotRelation")
                .map(|e| e["id"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            graph.enable_dot_callback_trace();
            graph.remove_music_inter(dots[0]).unwrap();
            assert_eq!(live_state(&graph, &ids), c["after_first"]);
            graph.remove_music_inter(dots[1]).unwrap();
            assert_eq!(live_state(&graph, &ids), c["after"]);
            let trace=graph.dot_callback_trace().iter().map(|event| {
                let (prefix,entities)=if event["event"]=="edgeRemoved" {("edge",edges.as_slice())}else{("dot",ids.as_slice())};
                let index=entities.iter().position(|id|event["id"]==*id).unwrap();
                json!({"event":event["event"],"stage":event["stage"],"entity":format!("{prefix}{index}"),"depth":event["depth"],"ownership":ownership_state(&event["state"],&ids)})
            }).collect::<Vec<_>>();
            assert_eq!(json!(trace), c["trace"], "{}", c["mode"]);
        }
    }
    fn readd_state(graph: &Graph, ids: &[usize]) -> Value {
        let order = graph.dot_ownership.as_ref().unwrap().measures[0]
            .dots
            .iter()
            .filter_map(|id| ids.iter().position(|d| d == id))
            .collect::<Vec<_>>();
        json!({"ownership":live_state(graph,ids),"measure_order":order,"contextual_grades":ids.iter().map(|id|graph.vertices[id-1].data["contextual_grade"].clone()).collect::<Vec<_>>()})
    }
    #[test]
    fn original_same_dot_readd_retains_context_part_and_insertion_order_at_stack_edges() {
        let original: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-owner-readd/observed.json"
        ))
        .unwrap();
        let lifecycle: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-part-ownership/observed.json"
        ))
        .unwrap();
        let c = &lifecycle["cases"][0];
        let empty_graph = || {
            let mut graph = Graph::default();
            let measures = original["stack_bounds"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, b)| Measure {
                    system: 1,
                    part: 1,
                    staff: 1,
                    index: i + 1,
                    left: b["left"].as_i64().unwrap(),
                    right: b["right"].as_i64().unwrap(),
                    timing: Default::default(),
                })
                .collect::<Vec<_>>();
            graph.bind_dot_measures(&measures).unwrap();
            graph
        };
        for control in original["cases"].as_array().unwrap() {
            let mut graph = empty_graph();
            let hk = Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            };
            let ck = Key {
                system: 1,
                phase: Phase::Chord,
                local: 1,
            };
            graph.vertex(hk, c["head"].clone(), false).unwrap();
            graph.vertex(ck,json!({"kind":"HeadChordInter","bounds":c["chord_bounds"],"part_id":1,"staff_id":1,"duration":null,"voice":null,"dots_number":2}),false).unwrap();
            graph
                .relations(
                    1,
                    Phase::Chord,
                    vec![(ck, hk, json!({"kind":"Containment"}))],
                )
                .unwrap();
            let dots = [
                Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: 1,
                },
                Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: 2,
                },
            ];
            let mut data = Vec::new();
            let mut ids = Vec::new();
            for i in 0..2 {
                let mut dot = json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":c["dot_bounds"][i],"grade":control["dot_grades"][i],"contextual_grade":control["before"]["contextual_grades"][i],"glyph_id":i+1});
                ids.push(graph.vertex(dots[i], dot.clone(), false).unwrap());
                if i == 0 {
                    dot["staff_id"] = json!(1);
                    graph.vertex(dots[i], dot.clone(), false).unwrap();
                }
                // Re-registration gets intrinsic/glyph state; context lives on the same Inter.
                dot.as_object_mut().unwrap().remove("contextual_grade");
                data.push(dot);
            }
            let support = vec![
                (
                    dots[0],
                    hk,
                    json!({"kind":"AugmentationRelation","grade":0.7}),
                ),
                (
                    dots[1],
                    dots[0],
                    json!({"kind":"DoubleDotRelation","grade":0.7}),
                ),
            ];
            graph.relations(1, Phase::Symbol, support.clone()).unwrap();
            graph.dot_ownership.as_mut().unwrap().measures[0].dots = ids.clone();
            if control["warm"] == true {
                for k in dots {
                    graph.augmentation_part(k).unwrap();
                }
            }
            assert_eq!(readd_state(&graph, &ids), control["before"]);
            graph.remove_music_inter(dots[0]).unwrap();
            graph.remove_music_inter(dots[1]).unwrap();
            let mut steps = vec![json!({"action":"remove both","state":readd_state(&graph,&ids)})];
            graph.vertex(dots[1], data[1].clone(), false).unwrap();
            steps.push(json!({"action":"readd second","state":readd_state(&graph,&ids)}));
            graph.vertex(dots[0], data[0].clone(), false).unwrap();
            steps.push(json!({"action":"readd first","state":readd_state(&graph,&ids)}));
            graph.relations(1, Phase::Symbol, support).unwrap();
            steps.push(json!({"action":"restore support","state":readd_state(&graph,&ids)}));
            graph.populate_dot_measures().unwrap();
            steps.push(json!({"action":"populateFRATs","state":readd_state(&graph,&ids)}));
            assert_eq!(json!(steps), control["steps"], "warm={}", control["warm"]);
            let once = graph.snapshot();
            graph.populate_dot_measures().unwrap();
            assert_eq!(graph.snapshot(), once);
        }
        for boundary in original["boundary"].as_array().unwrap() {
            let mut graph = empty_graph();
            let k = Key {
                system: 1,
                phase: Phase::Symbol,
                local: 1,
            };
            let x = boundary["center_x"].as_i64().unwrap();
            let id=graph.vertex(k,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":[x-2,240,5,4],"staff_id":1,"grade":0.8}),false).unwrap();
            assert_eq!(
                json!(graph.stack_for_dot(id).unwrap().map(|i| i + 1)),
                boundary["getStackAt_index"]
            );
            let membership = graph
                .dot_ownership
                .as_ref()
                .unwrap()
                .measures
                .iter()
                .map(|m| m.dots.contains(&id))
                .collect::<Vec<_>>();
            assert_eq!(json!(membership), boundary["added_membership"]);
            assert_eq!(
                json!(graph.vertices[id - 1].cached_part),
                boundary["cached_part_id"]
            );
            graph.populate_dot_measures().unwrap();
            assert_eq!(
                json!(
                    graph
                        .dot_ownership
                        .as_ref()
                        .unwrap()
                        .measures
                        .iter()
                        .map(|m| m.dots.contains(&id))
                        .collect::<Vec<_>>()
                ),
                boundary["stack_filter"]
            );
        }
    }
    fn edit_state(snapshot: &Value, ids: &[usize]) -> Value {
        let nodes = snapshot["inters"].as_array().unwrap();
        let order = snapshot["augmentation_dot_ownership"]["measures"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                m["augmentation_dot_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|id| ids.iter().position(|i| *id == *i))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        json!({"ownership":ownership_state(snapshot,ids),"measure_order":order,
            "nodes":ids.iter().map(|id| {let n=nodes.iter().find(|n|n["id"]==*id).unwrap();
                json!({"bounds":n["bounds"],"grade":n["grade"],"contextual_grade":n["contextual_grade"]})}).collect::<Vec<_>>()})
    }
    fn all_vertices(graph: &Graph) -> Value {
        let mut s = graph.snapshot();
        s["inters"] = json!(
            graph
                .vertices
                .iter()
                .map(|v| {
                    let mut n = v.data.clone();
                    n["registered"] = json!(!v.removed);
                    n["lifecycle_removed"] = json!(v.removed || v.removing);
                    n
                })
                .collect::<Vec<_>>()
        );
        s
    }
    #[test]
    fn original_explicit_owner_setters_preserve_caches_and_callback_membership() {
        let observed: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-owner-edit/observed.json"
        ))
        .unwrap();
        let prior: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-part-ownership/observed.json"
        ))
        .unwrap();
        let stacks: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-owner-readd/observed.json"
        ))
        .unwrap();
        let c = &prior["cases"][0];
        let mut graph = Graph::default();
        let measures = stacks["stack_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, b)| Measure {
                system: 1,
                part: 1,
                staff: 1,
                index: i + 1,
                left: b["left"].as_i64().unwrap(),
                right: b["right"].as_i64().unwrap(),
                timing: Default::default(),
            })
            .collect::<Vec<_>>();
        graph.bind_dot_measures(&measures).unwrap();
        let hk = Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        };
        let ck = Key {
            system: 1,
            phase: Phase::Chord,
            local: 1,
        };
        graph.vertex(hk, c["head"].clone(), false).unwrap();
        graph.vertex(ck,json!({"kind":"HeadChordInter","bounds":c["chord_bounds"],"part_id":1,"staff_id":1,"duration":null,"voice":null,"dots_number":2}),false).unwrap();
        graph
            .relations(
                1,
                Phase::Chord,
                vec![(ck, hk, json!({"kind":"Containment"}))],
            )
            .unwrap();
        let dots = [
            Key {
                system: 1,
                phase: Phase::Symbol,
                local: 1,
            },
            Key {
                system: 1,
                phase: Phase::Symbol,
                local: 2,
            },
        ];
        let mut ids = Vec::new();
        for (i, k) in dots.iter().enumerate() {
            let n = &observed["before"]["nodes"][i];
            ids.push(graph.vertex(*k,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":n["bounds"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"staff_id":if i==0 {json!(1)}else{Value::Null},"glyph_id":i+1}),false).unwrap());
            graph.vertices[*ids.last().unwrap() - 1].cached_part = None;
        }
        graph.dot_ownership.as_mut().unwrap().measures[0].dots = ids.clone();
        graph
            .relations(
                1,
                Phase::Symbol,
                vec![
                    (
                        dots[0],
                        hk,
                        json!({"kind":"AugmentationRelation","grade":0.7}),
                    ),
                    (
                        dots[1],
                        dots[0],
                        json!({"kind":"DoubleDotRelation","grade":0.7}),
                    ),
                ],
            )
            .unwrap();
        assert_eq!(edit_state(&all_vertices(&graph), &ids), observed["before"]);
        let mut stages = Vec::new();
        for (index, edit) in [
            NoteOwnerEdit::Part(Some(1)),
            NoteOwnerEdit::Staff(None),
            NoteOwnerEdit::Grade(0.4),
            NoteOwnerEdit::Part(None),
        ]
        .into_iter()
        .enumerate()
        {
            graph.edit_note_owner(dots[0], edit).unwrap();
            stages.push(json!({"action":observed["steps"][index]["action"],"state":edit_state(&all_vertices(&graph),&ids)}));
        }
        assert_eq!(graph.augmentation_part(dots[0]).unwrap(), Some(1));
        stages.push(
            json!({"action":"getPart(support)","state":edit_state(&all_vertices(&graph),&ids)}),
        );
        graph
            .edit_note_owner(
                dots[0],
                NoteOwnerEdit::Bounds(
                    serde_json::from_value(
                        observed["steps"][5]["state"]["nodes"][0]["bounds"].clone(),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        stages.push(json!({"action":"setBounds(next stack)","state":edit_state(&all_vertices(&graph),&ids)}));
        let edges = graph
            .edges
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                matches!(
                    e.key.kind.as_str(),
                    "AugmentationRelation" | "DoubleDotRelation"
                )
            })
            .map(|(i, _)| i + 1)
            .collect::<Vec<_>>();
        graph.enable_dot_callback_trace();
        for (k, action) in dots.iter().zip(["remove moved dot", "remove second"]) {
            graph.remove_music_inter(*k).unwrap();
            stages.push(json!({"action":action,"state":edit_state(&all_vertices(&graph),&ids)}));
        }
        assert_eq!(json!(stages), observed["steps"]);
        let trace=graph.dot_callback_trace().iter().map(|e| {
            let (prefix,keys)=if e["event"]=="edgeRemoved" {("edge",&edges)}else{("dot",&ids)};
            let i=keys.iter().position(|id|e["id"]==*id).unwrap();
            json!({"event":e["event"],"stage":e["stage"],"entity":format!("{prefix}{i}"),"depth":e["depth"],"state":edit_state(&e["state"],&ids)})
        }).collect::<Vec<_>>();
        assert_eq!(json!(trace), observed["trace"]);
    }
    fn support_state(
        snapshot: &Value,
        notes: &[usize],
        chords: &[usize],
        dots: &[usize],
        edges: &[usize],
    ) -> Value {
        let nodes = snapshot["inters"].as_array().unwrap();
        let node = |id| nodes.iter().find(|n| n["id"] == id).unwrap();
        let relations = snapshot["relations"].as_array().unwrap();
        let mut incident = Vec::new();
        for incoming in [true, false] {
            for r in relations.iter().filter(|r| {
                if incoming {
                    r["target"] == dots[0]
                } else {
                    r["source"] == dots[0]
                }
            }) {
                if let Some(i) = edges.iter().position(|id| r["id"] == *id)
                    && !incident.contains(&i)
                {
                    incident.push(i);
                }
            }
        }
        json!({"owners":{"note_dot_counts":notes.iter().map(|id|crate::music_dots::note_count(snapshot,node(*id))).collect::<Vec<_>>(),
            "chord_dot_counts":chords.iter().map(|id|node(*id)["dots_number"].clone()).collect::<Vec<_>>(),
            "dots":dots.iter().map(|id| {let n=node(*id);json!({"removed":n["lifecycle_removed"],"registered":n["registered"],"abnormal":n["abnormal"]})}).collect::<Vec<_>>(),
            "edges_live":edges.iter().map(|id|relations.iter().any(|r|r["id"]==*id)).collect::<Vec<_>>()},
            "ownership":edit_state(snapshot,dots),"incident_order":incident})
    }
    #[test]
    fn original_readded_support_changes_first_part_and_copied_callback_order() {
        let observed: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-support-readd/observed.json"
        ))
        .unwrap();
        let bounds: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-owner-readd/observed.json"
        ))
        .unwrap();
        let mut graph = Graph::default();
        let measures = bounds["stack_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, b)| Measure {
                system: 1,
                part: 1,
                staff: 1,
                index: i + 1,
                left: b["left"].as_i64().unwrap(),
                right: b["right"].as_i64().unwrap(),
                timing: Default::default(),
            })
            .collect::<Vec<_>>();
        graph.bind_dot_measures(&measures).unwrap();
        let mut notes = Vec::new();
        let mut note_ids = Vec::new();
        let mut chords = Vec::new();
        let mut chord_ids = Vec::new();
        for (i, n) in observed["notes"].as_array().unwrap().iter().enumerate() {
            let hk = Key {
                system: 1,
                phase: Phase::Head,
                local: i + 1,
            };
            let ck = Key {
                system: 1,
                phase: Phase::Chord,
                local: i + 1,
            };
            let mut h = n.clone();
            h["staff_id"] = json!(1);
            note_ids.push(graph.vertex(hk, h, false).unwrap());
            chord_ids.push(graph.vertex(ck,json!({"kind":"HeadChordInter","bounds":observed["owner_bounds"][i],"part_id":1,"staff_id":1,"duration":null,"voice":null,"dots_number":2}),false).unwrap());
            graph
                .relations(
                    1,
                    Phase::Chord,
                    vec![(ck, hk, json!({"kind":"Containment"}))],
                )
                .unwrap();
            notes.push(hk);
            chords.push(ck);
        }
        let dots = [
            Key {
                system: 1,
                phase: Phase::Symbol,
                local: 1,
            },
            Key {
                system: 1,
                phase: Phase::Symbol,
                local: 2,
            },
        ];
        let mut ids = Vec::new();
        for (i, k) in dots.iter().enumerate() {
            let mut n = observed["before"]["ownership"]["nodes"][i].clone();
            n["kind"] = json!("AugmentationDotInter");
            n["shape"] = json!("AUGMENTATION_DOT");
            n["glyph_id"] = json!(i + 1);
            n["staff_id"] = observed["before"]["ownership"]["ownership"][i]["staff_id"].clone();
            ids.push(graph.vertex(*k, n, false).unwrap());
            graph.vertices[*ids.last().unwrap() - 1].cached_part = None;
        }
        graph.dot_ownership.as_mut().unwrap().measures[0].dots = ids.clone();
        let aug = (
            dots[0],
            notes[0],
            json!({"kind":"AugmentationRelation","grade":0.7}),
        );
        graph
            .relations(
                1,
                Phase::Symbol,
                vec![
                    aug.clone(),
                    (
                        dots[1],
                        dots[0],
                        json!({"kind":"DoubleDotRelation","grade":0.7}),
                    ),
                    (
                        dots[0],
                        notes[1],
                        json!({"kind":"AugmentationRelation","grade":0.7}),
                    ),
                ],
            )
            .unwrap();
        let snap = graph.snapshot();
        let edges = vec![
            snap["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["kind"] == "AugmentationRelation" && r["target"] == note_ids[0])
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize,
            snap["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["kind"] == "AugmentationRelation" && r["target"] == note_ids[1])
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize,
            snap["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["kind"] == "DoubleDotRelation")
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize,
        ];
        for k in &chords {
            graph.count_owned_augmentation(*k).unwrap();
        }
        graph
            .edit_note_owner(notes[0], NoteOwnerEdit::Part(None))
            .unwrap();
        graph
            .edit_note_owner(notes[0], NoteOwnerEdit::Staff(None))
            .unwrap();
        graph
            .edit_note_owner(dots[0], NoteOwnerEdit::Part(None))
            .unwrap();
        assert_eq!(
            support_state(&all_vertices(&graph), &note_ids, &chord_ids, &ids, &edges),
            observed["before"]
        );
        let part = graph.augmentation_part(dots[0]).unwrap();
        assert_eq!(json!(part), observed["steps"][0]["result"]);
        let mut stages = vec![
            json!({"action":"getPart before readd","result":part,"state":support_state(&all_vertices(&graph),&note_ids,&chord_ids,&ids,&edges)}),
        ];
        graph.enable_dot_callback_trace();
        graph.remove_augmentation_edge(edges[0]).unwrap();
        stages.push(json!({"action":"remove first support","state":support_state(&all_vertices(&graph),&note_ids,&chord_ids,&ids,&edges)}));
        graph
            .relations(1, Phase::Symbol, vec![aug.clone()])
            .unwrap();
        stages.push(json!({"action":"readd same support","state":support_state(&all_vertices(&graph),&note_ids,&chord_ids,&ids,&edges)}));
        assert_eq!(stages[2], observed["steps"][2]);
        let same = graph.snapshot();
        graph.relations(1, Phase::Symbol, vec![aug]).unwrap();
        assert_eq!(same, graph.snapshot());
        stages.push(json!({"action":"repeat live support","added":false,"state":support_state(&all_vertices(&graph),&note_ids,&chord_ids,&ids,&edges)}));
        graph
            .edit_note_owner(dots[0], NoteOwnerEdit::Part(None))
            .unwrap();
        let part = graph.augmentation_part(dots[0]).unwrap();
        assert_eq!(json!(part), observed["steps"][4]["result"]);
        stages.push(json!({"action":"getPart after readd","result":part,"state":support_state(&all_vertices(&graph),&note_ids,&chord_ids,&ids,&edges)}));
        graph.remove_music_inter(dots[0]).unwrap();
        stages.push(json!({"action":"remove dot","state":support_state(&all_vertices(&graph),&note_ids,&chord_ids,&ids,&edges)}));
        assert_eq!(json!(stages), observed["steps"]);
        // The original re-add observer watches removal callbacks only.
        let trace=graph.dot_callback_trace().iter().filter(|e|e["event"]=="edgeRemoved" || e["event"]=="vertexRemoved").map(|e| {
            let (prefix,keys)=if e["event"]=="edgeRemoved" {("edge",edges.as_slice())}else{("dot",ids.as_slice())};let i=keys.iter().position(|id|e["id"]==*id).unwrap();
            json!({"event":e["event"],"stage":e["stage"],"entity":format!("{prefix}{i}"),"depth":e["depth"],"state":support_state(&e["state"],&note_ids,&chord_ids,&ids,&edges)})
        }).collect::<Vec<_>>();
        assert_eq!(json!(trace), observed["trace"]);
    }
}
