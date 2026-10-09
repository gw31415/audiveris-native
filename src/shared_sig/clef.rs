// SPDX-License-Identifier: AGPL-3.0-or-later
//! Recognition header Clef ownership and deferred mixed MeasureFiller membership.
use super::{Graph, Key, Phase};
use crate::{
    grid_ir::{PartGeometry, StaffGeometry},
    music_measure_layout::{Layout, MeasureLayout, StackOwner},
    system_area_geometry::Band,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod filler;

#[derive(Clone, Debug)]
struct ClefOwner {
    staff: Arc<StaffGeometry>,
    part: Arc<PartGeometry>,
    header: bool,
    registered: bool,
}
#[derive(Clone, Debug)]
struct MeasureClefs {
    measure: Arc<MeasureLayout>,
    members: Vec<Key>,
    keys: Vec<Key>,
}
#[derive(Clone, Default)]
pub(super) struct ClefOwnership {
    staffs: Vec<Arc<StaffGeometry>>,
    areas: Vec<Arc<Band>>,
    parts: Vec<Arc<PartGeometry>>,
    stacks: Vec<StackOwner>,
    measures: Vec<MeasureClefs>,
    clefs: BTreeMap<Key, ClefOwner>,
    key_signatures: BTreeMap<Key, filler::KeyOwner>,
    filled_systems: BTreeSet<u64>,
}
impl std::fmt::Debug for ClefOwnership {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClefOwnership")
            .field("measures", &self.measures)
            .field("clefs", &self.clefs)
            .finish_non_exhaustive()
    }
}

impl Graph {
    pub(crate) fn bind_clef_layout(
        &mut self,
        layout: &Layout,
        staffs: &[Arc<StaffGeometry>],
        areas: &[Arc<Band>],
        parts: &[Arc<PartGeometry>],
    ) -> Result<(), String> {
        if self.clef_ownership.is_some() || staffs.len() != areas.len() {
            return Err("music_clef_fresh_layout_required".into());
        }
        let mut measures = Vec::new();
        for part in &layout.parts {
            if !parts.iter().any(|p| Arc::ptr_eq(p, &part.part))
                || part
                    .part
                    .staffs
                    .iter()
                    .any(|s| !staffs.iter().any(|a| Arc::ptr_eq(a, s)))
            {
                return Err("music_clef_actual_Part_Staff_required".into());
            }
            for measure in &part.measures {
                if !Arc::ptr_eq(&measure.part, &part.part)
                    || !layout.stacks.iter().any(|stack| {
                        Arc::ptr_eq(&stack.stack, &measure.stack)
                            && stack.measures.iter().any(|m| Arc::ptr_eq(m, measure))
                    })
                    || measures
                        .iter()
                        .any(|m: &MeasureClefs| Arc::ptr_eq(&m.measure, measure))
                {
                    return Err("music_clef_actual_Measure_Stack_required".into());
                }
                measures.push(MeasureClefs {
                    measure: measure.clone(),
                    members: Vec::new(),
                    keys: Vec::new(),
                });
            }
        }
        for stack in &layout.stacks {
            let system_parts = parts
                .iter()
                .filter(|p| p.system == stack.stack.system)
                .collect::<Vec<_>>();
            if system_parts.len() != stack.measures.len()
                || system_parts.iter().zip(&stack.measures).any(|(p, m)| {
                    !Arc::ptr_eq(p, &m.part)
                        || !Arc::ptr_eq(&m.stack, &stack.stack)
                        || !measures.iter().any(|row| Arc::ptr_eq(&row.measure, m))
                })
            {
                return Err("music_clef_actual_Stack_Part_order_required".into());
            }
        }
        self.clef_ownership = Some(ClefOwnership {
            staffs: staffs.to_vec(),
            areas: areas.to_vec(),
            parts: parts.to_vec(),
            stacks: layout.stacks.clone(),
            measures,
            clefs: BTreeMap::new(),
            key_signatures: BTreeMap::new(),
            filled_systems: BTreeSet::new(),
        });
        Ok(())
    }

    // Called at the native vertex registration boundary, never from a report.
    // Existing header objects predate Measures in Java, so binding their identity
    // does not backdate MeasureFiller's later membership mutation.
    pub(crate) fn bind_clef_identity(
        &mut self,
        key: Key,
        staff: Arc<StaffGeometry>,
        header: bool,
    ) -> Result<(), String> {
        let id = self
            .id(key)
            .ok_or("music_clef_registered_identity_required")?;
        let vertex = &self.vertices[id - 1];
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_clef_layout_required")?;
        let part = owners
            .parts
            .iter()
            .find(|p| p.staffs.iter().any(|s| Arc::ptr_eq(s, &staff)))
            .ok_or("music_clef_actual_Part_Staff_required")?
            .clone();
        if vertex.removed
            || vertex.data["kind"] != "ClefInter"
            || vertex.data["staff_id"] != staff.id
            || key.system != staff.system
            || !owners.staffs.iter().any(|s| Arc::ptr_eq(s, &staff))
            || owners.clefs.contains_key(&key)
            || !matches!(
                (key.phase, header),
                (Phase::MusicHeader, true) | (Phase::Symbol, false)
            )
            || (header
                && owners
                    .clefs
                    .values()
                    .any(|c| c.header && Arc::ptr_eq(&c.staff, &staff)))
        {
            return Err("music_clef_fresh_actual_identity_required".into());
        }
        self.clef_ownership
            .as_mut()
            .unwrap()
            .filled_systems
            .remove(&key.system);
        self.clef_ownership.as_mut().unwrap().clefs.insert(
            key,
            ClefOwner {
                staff,
                part,
                header,
                registered: true,
            },
        );
        Ok(())
    }

    pub(crate) fn clef_staff(&self, key: Key) -> Result<Arc<StaffGeometry>, String> {
        let id = self.id(key).ok_or("music_clef_actual_identity_required")?;
        let owner = self
            .clef_ownership
            .as_ref()
            .and_then(|o| o.clefs.get(&key))
            .ok_or("music_clef_actual_identity_required")?;
        let body = &self.vertices[id - 1].data;
        if body["kind"] != "ClefInter"
            || body["staff_id"] != owner.staff.id
            || owner.staff.system != key.system
            || !owner
                .part
                .staffs
                .iter()
                .any(|s| Arc::ptr_eq(s, &owner.staff))
        {
            return Err("music_clef_actual_identity_changed".into());
        }
        Ok(owner.staff.clone())
    }

    pub(crate) fn owned_measure_clefs(
        &self,
        measure: &Arc<MeasureLayout>,
    ) -> Result<&[Key], String> {
        self.clef_ownership
            .as_ref()
            .and_then(|o| {
                o.measures
                    .iter()
                    .find(|row| Arc::ptr_eq(&row.measure, measure))
            })
            .map(|row| row.members.as_slice())
            .ok_or_else(|| "music_clef_actual_Measure_required".into())
    }

    fn clef_center(&self, key: Key) -> Result<Option<[i64; 2]>, String> {
        let id = self.id(key).ok_or("music_clef_actual_identity_required")?;
        let body = &self.vertices[id - 1].data;
        if body["bounds"].is_null() {
            return Ok(None);
        }
        let b: [i64; 4] = serde_json::from_value(body["bounds"].clone())
            .map_err(|_| "music_clef_bounds_required")?;
        if b[2] <= 0 || b[3] <= 0 {
            return Err("music_clef_positive_bounds_required".into());
        }
        Ok(Some([b[0] + b[2] / 2, b[1] + b[3] / 2]))
    }

    // Same primitive for ClefInter.added and the later MeasureFiller dispatch.
    // The latter must pass its actual selected Measure at its original boundary.
    pub(crate) fn add_clef_to_measure(
        &mut self,
        key: Key,
        measure: &Arc<MeasureLayout>,
    ) -> Result<(), String> {
        self.clef_staff(key)?;
        let id = self.id(key).unwrap();
        let owners = self.clef_ownership.as_ref().unwrap();
        let owner = &owners.clefs[&key];
        let index = owners
            .measures
            .iter()
            .position(|m| Arc::ptr_eq(&m.measure, measure))
            .ok_or("music_clef_actual_Measure_required")?;
        if self.vertices[id - 1].removed
            || !owner.registered
            || !Arc::ptr_eq(&owner.part, &measure.part)
        {
            return Err("music_clef_live_Part_member_required".into());
        }
        let mut members = owners.measures[index].members.clone();
        if members.contains(&key) {
            return Ok(());
        }
        members.push(key);
        if members.len() > 1 && self.clef_center(key)?.is_some() {
            let mut ordered = members
                .iter()
                .map(|k| {
                    let [x, y] = self
                        .clef_center(*k)?
                        .ok_or("music_clef_null_sort_center_pending")?;
                    Ok::<_, String>((
                        x,
                        y,
                        self.id(*k).ok_or("music_clef_actual_identity_required")?,
                        *k,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            ordered.sort_by_key(|row| (row.0, row.1, row.2));
            members = ordered.into_iter().map(|row| row.3).collect();
        }
        self.clef_ownership.as_mut().unwrap().measures[index].members = members;
        Ok(())
    }
}
