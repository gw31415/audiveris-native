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
impl Graph {
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
    pub(crate) fn augmentation_part(&mut self, key: Key) -> Result<Option<u64>, String> {
        let id = self
            .id(key)
            .ok_or("music_dot_part_inter_identity_required")?;
        if self.vertices[id - 1].data["kind"] != "AugmentationDotInter" {
            return Err("music_dot_part_inter_required".into());
        }
        self.part_for_inter(id)
    }
    fn part_for_inter(&mut self, id: usize) -> Result<Option<u64>, String> {
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
                .edges
                .iter()
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
            let mut incident = Vec::new();
            for incoming in [true, false] {
                for e in self.edges.iter().filter(|e| {
                    !e.removed
                        && e.key.kind == "DoubleDotRelation"
                        && if incoming {
                            e.key.target == id
                        } else {
                            e.key.source == id
                        }
                }) {
                    incident.push(if e.key.source == id {
                        e.key.target
                    } else {
                        e.key.source
                    });
                }
            }
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
}
