// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fresh recognition Flag's actual Stack/Part/Measure lifecycle.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FlagRemovalStage {
    ContainerLookupComplete,
    EdgeRemoved,
    VertexRemoved,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct FlagRemovalEvent {
    pub(crate) stage: FlagRemovalStage,
    pub(crate) key: Key,
    pub(crate) edge: Option<usize>,
}

impl Graph {
    pub(in crate::shared_sig) fn requires_owned_flag_removal(&self, key: Key) -> bool {
        self.clef_ownership
            .as_ref()
            .is_some_and(|owners| owners.flag_added_keys.contains(&key))
    }
    fn owned_flag_id(&self, key: Key) -> Result<usize, String> {
        let id = self
            .id(key)
            .ok_or("music_flag_registered_identity_required")?;
        if key.phase != Phase::Symbol || self.vertices[id - 1].data["kind"] != "FlagInter" {
            return Err("music_flag_actual_standard_Flag_required".into());
        }
        Ok(id)
    }
    fn flag_actual_part(&self, system: u64, part_id: u64) -> Result<Arc<PartGeometry>, String> {
        self.clef_ownership
            .as_ref()
            .ok_or("music_flag_actual_layout_required")?
            .parts
            .iter()
            .find(|part| part.system == system && part.id == part_id)
            .cloned()
            .ok_or_else(|| "music_flag_actual_Part_required".into())
    }
    /// AbstractInter.getPart, used on the actual first Head or as Flag fallback.
    /// A nonnull raw Staff identifies an already-retained geometry owner; it is
    /// never filled from the classifier's closest-Staff suggestion.
    fn flag_base_part(&mut self, id: usize) -> Result<Option<Arc<PartGeometry>>, String> {
        let system = self.vertices[id - 1].key.system;
        if let Some(part) = self.vertices[id - 1].cached_part {
            return Ok(Some(self.flag_actual_part(system, part)?));
        }
        let Some(staff_id) = self.vertices[id - 1].data["staff_id"].as_u64() else {
            return Ok(None);
        };
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_flag_actual_layout_required")?;
        let staff = owners
            .staffs
            .iter()
            .find(|s| s.system == system && s.id == staff_id)
            .ok_or("music_flag_actual_raw_Staff_required")?;
        let part = owners
            .parts
            .iter()
            .find(|p| p.system == system && p.staffs.iter().any(|s| Arc::ptr_eq(s, staff)))
            .cloned()
            .ok_or("music_flag_actual_raw_Staff_Part_required")?;
        self.vertices[id - 1].cached_part = Some(part.id);
        Ok(Some(part))
    }
    /// Flag.getPart: first FlagStem, then first HeadStem in actual relation
    /// encounter order. Return even when that first Head's Part is null.
    pub(crate) fn owned_flag_part(
        &mut self,
        key: Key,
    ) -> Result<Option<Arc<PartGeometry>>, String> {
        let id = self.owned_flag_id(key)?;
        if let Some(part) = self.vertices[id - 1].cached_part {
            return Ok(Some(self.flag_actual_part(key.system, part)?));
        }
        if self.vertices[id - 1].removed || self.vertices[id - 1].removing {
            return Err("music_flag_uncached_removed_Part_lookup_pending".into());
        }
        for flag_edge in self.incident_edges(id) {
            let edge = &self.edges[flag_edge - 1];
            if edge.key.kind != "FlagStemRelation" {
                continue;
            }
            if edge.key.source != id {
                return Err("music_flag_source_relation_required".into());
            }
            let stem_id = edge.key.target;
            let stem = &self.vertices[stem_id - 1];
            if stem.key.phase != Phase::Stem
                || stem.key.system != key.system
                || stem.data["kind"] != "StemInter"
                || stem.removed
                || stem.removing
            {
                return Err("music_flag_actual_Stem_required".into());
            }
            for head_edge in self.incident_edges(stem_id) {
                let edge = &self.edges[head_edge - 1];
                if edge.key.kind != "HeadStemRelation" {
                    continue;
                }
                let head_id = edge.key.source;
                let head = &self.vertices[head_id - 1];
                if edge.key.target != stem_id
                    || head.key.phase != Phase::Head
                    || head.key.system != key.system
                    || head.data["kind"] != "HeadInter"
                    || head.removed
                    || head.removing
                {
                    return Err("music_flag_actual_first_Head_required".into());
                }
                let part = self.flag_base_part(head_id)?;
                self.vertices[id - 1].cached_part = part.as_ref().map(|p| p.id);
                return Ok(part);
            }
        }
        self.flag_base_part(id)
    }
    #[cfg(test)]
    pub(crate) fn owned_flag_cached_part(
        &self,
        key: Key,
    ) -> Result<Option<Arc<PartGeometry>>, String> {
        let id = self.owned_flag_id(key)?;
        self.vertices[id - 1]
            .cached_part
            .map(|part| self.flag_actual_part(key.system, part))
            .transpose()
    }
    fn flag_stack_at_center(&self, key: Key) -> Result<Option<StackOwner>, String> {
        let id = self.owned_flag_id(key)?;
        let bounds: [i64; 4] = serde_json::from_value(self.vertices[id - 1].data["bounds"].clone())
            .map_err(|_| "music_flag_actual_cached_bounds_required")?;
        if bounds[2] <= 0 || bounds[3] <= 0 {
            return Err("music_flag_positive_bounds_required".into());
        }
        let point = [
            (bounds[0] + bounds[2] / 2) as f64,
            (bounds[1] + bounds[3] / 2) as f64,
        ];
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_flag_actual_layout_required")?;
        let around = crate::grid_staff_neighborhood::around(
            &owners.staffs,
            &owners.areas,
            &owners.parts,
            key.system,
            point,
        )?;
        let Some(staff) = around.staffs.first() else {
            return Ok(None);
        };
        for stack in owners
            .stacks
            .iter()
            .filter(|stack| stack.stack.system == key.system)
        {
            let Some(measure) = stack
                .measures
                .iter()
                .find(|m| m.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)))
            else {
                continue;
            };
            let [left, right] = measure.abscissae(staff)?;
            if (left as f64) < point[0] && point[0] <= right as f64 {
                return Ok(Some(stack.clone()));
            }
        }
        Ok(None)
    }
    fn flag_measure_in_stack(
        &self,
        stack: &StackOwner,
        part: &Arc<PartGeometry>,
    ) -> Result<Arc<MeasureLayout>, String> {
        stack
            .measures
            .iter()
            .find(|measure| Arc::ptr_eq(&measure.part, part))
            .cloned()
            .ok_or_else(|| "music_flag_actual_Stack_Part_Measure_required".into())
    }
    #[cfg(test)]
    pub(crate) fn owned_measure_flags(
        &self,
        measure: &Arc<MeasureLayout>,
    ) -> Result<&[Key], String> {
        self.clef_ownership
            .as_ref()
            .and_then(|owners| {
                owners
                    .measures
                    .iter()
                    .find(|row| Arc::ptr_eq(&row.measure, measure))
            })
            .map(|row| row.flags.as_slice())
            .ok_or_else(|| "music_flag_actual_Measure_required".into())
    }
    /// Ordinary createValidAdded only: invoke after vertexAdded, before Link.applyTo.
    pub(crate) fn fresh_flag_added(&mut self, key: Key) -> Result<(), String> {
        let id = self.owned_flag_id(key)?;
        if self.vertices[id - 1].removed
            || self.vertices[id - 1].removing
            || !self.vertices[id - 1].data["staff_id"].is_null()
            || self.vertices[id - 1].cached_part.is_some()
            || self
                .incident_edges(id)
                .iter()
                .any(|eid| self.edges[*eid - 1].key.kind == "FlagStemRelation")
            || self
                .clef_ownership
                .as_ref()
                .ok_or("music_flag_actual_layout_required")?
                .flag_added_keys
                .contains(&key)
        {
            return Err("music_flag_fresh_unlinked_raw_null_added_required".into());
        }
        let mut next = self.clone();
        if let Some(stack) = next.flag_stack_at_center(key)?
            && let Some(part) = next.owned_flag_part(key)?
        {
            let measure = next.flag_measure_in_stack(&stack, &part)?;
            let row = next
                .clef_ownership
                .as_mut()
                .unwrap()
                .measures
                .iter_mut()
                .find(|row| Arc::ptr_eq(&row.measure, &measure))
                .unwrap();
            if !row.flags.contains(&key) {
                row.flags.push(key);
            }
        }
        next.set_owned_abnormal(id, true);
        next.clef_ownership
            .as_mut()
            .unwrap()
            .flag_added_keys
            .insert(key);
        *self = next;
        Ok(())
    }
    /// The admitted factory's Link.applyTo + FlagStemRelation.added boundary.
    /// Existing actual same-class endpoint links are not replaced by a new body.
    pub(crate) fn apply_owned_flag_stem(
        &mut self,
        flag: Key,
        stem: Key,
        relation: serde_json::Value,
    ) -> Result<bool, String> {
        let source = self.owned_flag_id(flag)?;
        let target = self.endpoint(stem)?;
        if self.vertices[source - 1].removed
            || self.vertices[source - 1].removing
            || relation["kind"] != "FlagStemRelation"
            || stem.phase != Phase::Stem
            || stem.system != flag.system
            || self.vertices[target - 1].data["kind"] != "StemInter"
            || self.vertices[target - 1].removing
            || !self
                .clef_ownership
                .as_ref()
                .ok_or("music_flag_actual_layout_required")?
                .flag_added_keys
                .contains(&flag)
        {
            return Err("music_flag_actual_added_link_required".into());
        }
        if self.has_relation(flag, stem, "FlagStemRelation") {
            return Ok(false);
        }
        if self.edges.iter().any(|edge| {
            edge.key.source == source
                && edge.key.target == target
                && edge.key.kind == "FlagStemRelation"
        }) {
            return Err("music_flag_relation_readdition_pending".into());
        }
        let mut next = self.clone();
        next.relations(flag.system, Phase::Symbol, vec![(flag, stem, relation)])?;
        next.set_owned_abnormal(source, false);
        *self = next;
        Ok(true)
    }
    pub(crate) fn remove_owned_flag(
        &mut self,
        key: Key,
        mut observe: impl FnMut(&Graph, FlagRemovalEvent) -> Result<(), String>,
    ) -> Result<(), String> {
        let id = self.owned_flag_id(key)?;
        if self.vertices[id - 1].removed || self.vertices[id - 1].removing {
            return Ok(());
        }
        if !self
            .clef_ownership
            .as_ref()
            .ok_or("music_flag_actual_layout_required")?
            .flag_added_keys
            .contains(&key)
        {
            return Err("music_flag_original_added_boundary_required".into());
        }
        if self.vertices[id - 1].data["frozen"] == true {
            return Err("music_flag_frozen_removal_pending".into());
        }
        let incident = self.incident_edges(id);
        for eid in &incident {
            let edge = &self.edges[*eid - 1];
            match edge.key.kind.as_str() {
                "Exclusion" | "NoExclusion" => {}
                "FlagStemRelation" if edge.key.source == id => {
                    let stem = &self.vertices[edge.key.target - 1];
                    if stem.key.system != key.system
                        || stem.key.phase != Phase::Stem
                        || stem.data["kind"] != "StemInter"
                        || stem.removed
                        || stem.removing
                    {
                        return Err("music_flag_actual_Stem_required".into());
                    }
                }
                _ => return Err("music_flag_incident_callback_pending".into()),
            }
        }
        let mut next = self.clone();
        // Null Stack short-circuits getPart, exactly as FlagInter.remove does.
        if let Some(stack) = next.flag_stack_at_center(key)?
            && let Some(part) = next.owned_flag_part(key)?
        {
            let measure = next.flag_measure_in_stack(&stack, &part)?;
            let row = next
                .clef_ownership
                .as_mut()
                .unwrap()
                .measures
                .iter_mut()
                .find(|row| Arc::ptr_eq(&row.measure, &measure))
                .unwrap();
            row.flags.retain(|member| *member != key);
        }
        observe(
            &next,
            FlagRemovalEvent {
                stage: FlagRemovalStage::ContainerLookupComplete,
                key,
                edge: None,
            },
        )?;
        next.vertices[id - 1].removing = true;
        for eid in incident {
            next.edges[eid - 1].removed = true;
            // FlagStemRelation.removed tests the dying Flag, not the Stem.
            // Source does not invalidate Chord caches or recompute clocks here.
            observe(
                &next,
                FlagRemovalEvent {
                    stage: FlagRemovalStage::EdgeRemoved,
                    key,
                    edge: Some(eid),
                },
            )?;
        }
        next.vertices[id - 1].removed = true;
        next.vertices[id - 1].removing = false;
        observe(
            &next,
            FlagRemovalEvent {
                stage: FlagRemovalStage::VertexRemoved,
                key,
                edge: None,
            },
        )?;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod image_tests;
