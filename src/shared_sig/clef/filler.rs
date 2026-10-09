// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mixed Clef/Key MeasureFiller for retained fresh recognition owners.
use super::*;
use serde_json::{Value, json};

#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub(super) struct KeyOwner {
    staff: Arc<StaffGeometry>,
    part: Arc<PartGeometry>,
}

impl Graph {
    #[cfg(test)]
    pub(crate) fn assert_real_owned_filler(music: &crate::music_measures::OwnedMusic) -> Value {
        tests::assert_real_owned_filler(music)
    }

    #[cfg(test)]
    pub(crate) fn assert_completed_header_filler(
        music: &crate::music_measures::OwnedMusic,
        control: &Value,
    ) {
        tests::assert_completed_header_filler(music, control);
    }

    // Called inside the existing actual Key registration, not by parsing its
    // report return value or by spatially guessing a replacement Staff.
    pub(crate) fn bind_filler_key_identity(
        &mut self,
        key: Key,
        staff: Arc<StaffGeometry>,
    ) -> Result<(), String> {
        let id = self.id(key).ok_or("music_filler_registered_Key_required")?;
        let vertex = &self.vertices[id - 1];
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_clef_layout_required")?;
        let part = owners
            .parts
            .iter()
            .find(|p| p.staffs.iter().any(|s| Arc::ptr_eq(s, &staff)))
            .ok_or("music_filler_actual_Key_Part_Staff_required")?
            .clone();
        if vertex.removed
            || vertex.removing
            || key.phase != Phase::MusicKey
            || vertex.data["kind"] != "KeyInter"
            || vertex.data["staff_id"] != staff.id
            || key.system != staff.system
            || !owners.staffs.iter().any(|s| Arc::ptr_eq(s, &staff))
            || owners.key_signatures.contains_key(&key)
        {
            return Err("music_filler_fresh_actual_Key_required".into());
        }
        self.clef_ownership
            .as_mut()
            .unwrap()
            .filled_systems
            .remove(&key.system);
        self.clef_ownership
            .as_mut()
            .unwrap()
            .key_signatures
            .insert(key, KeyOwner { staff, part });
        Ok(())
    }

    fn filler_owner(&self, key: Key) -> Result<(Arc<StaffGeometry>, Arc<PartGeometry>), String> {
        let id = self
            .id(key)
            .ok_or("music_filler_registered_inter_required")?;
        let vertex = &self.vertices[id - 1];
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_clef_layout_required")?;
        if vertex.removed || vertex.removing {
            return Err("music_filler_stable_live_inter_required".into());
        }
        match vertex.data["kind"].as_str() {
            Some("ClefInter") => {
                let staff = self.clef_staff(key)?;
                let owner = &owners.clefs[&key];
                if !owner.registered {
                    return Err("music_filler_registered_Clef_required".into());
                }
                Ok((staff, owner.part.clone()))
            }
            Some("KeyInter") => {
                let owner = owners
                    .key_signatures
                    .get(&key)
                    .ok_or("music_filler_bound_Key_required")?;
                if vertex.data["staff_id"] != owner.staff.id
                    || key.system != owner.staff.system
                    || !owner
                        .part
                        .staffs
                        .iter()
                        .any(|s| Arc::ptr_eq(s, &owner.staff))
                {
                    return Err("music_filler_actual_Key_identity_changed".into());
                }
                Ok((owner.staff.clone(), owner.part.clone()))
            }
            _ => Err("music_filler_Clef_or_Key_required".into()),
        }
    }

    pub(crate) fn owned_measure_keys(
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
            .map(|row| row.keys.as_slice())
            .ok_or_else(|| "music_filler_actual_Measure_required".into())
    }

    fn add_key_to_measure(&mut self, key: Key, measure: &Arc<MeasureLayout>) -> Result<(), String> {
        let (_, part) = self.filler_owner(key)?;
        if !Arc::ptr_eq(&part, &measure.part) {
            return Err("music_filler_actual_Key_Measure_required".into());
        }
        let owners = self.clef_ownership.as_mut().unwrap();
        let row = owners
            .measures
            .iter_mut()
            .find(|m| Arc::ptr_eq(&m.measure, measure))
            .ok_or("music_filler_actual_Measure_required")?;
        // Measure.keys is a LinkedHashSet: identity dedup, insertion order.
        if !row.keys.contains(&key) {
            row.keys.push(key);
        }
        Ok(())
    }

    fn filler_center(&self, key: Key) -> Result<[i64; 2], String> {
        let id = self
            .id(key)
            .ok_or("music_filler_registered_inter_required")?;
        let bounds: [i64; 4] = serde_json::from_value(self.vertices[id - 1].data["bounds"].clone())
            .map_err(|_| "music_filler_actual_cached_bounds_required")?;
        if bounds[2] <= 0 || bounds[3] <= 0 {
            return Err("music_filler_positive_bounds_required".into());
        }
        Ok([bounds[0] + bounds[2] / 2, bounds[1] + bounds[3] / 2])
    }

    // The source cached Stack envelope was built with these same immutable
    // Measure/Staff owners. No Stack merge/split or moved bar is admitted here.
    fn filler_stack_bounds(stack: &StackOwner) -> Result<[i64; 2], String> {
        if stack.measures.is_empty() {
            return Err("music_filler_nonempty_Stack_required".into());
        }
        let mut bounds = [i32::MAX as i64, 0];
        for measure in &stack.measures {
            for staff in &measure.part.staffs {
                let [left, right] = measure.abscissae(staff)?;
                bounds[0] = bounds[0].min(left);
                bounds[1] = bounds[1].max(right);
            }
        }
        Ok(bounds)
    }

    fn filter_filler_inter(
        &self,
        layout: &Layout,
        stack: &StackOwner,
        key: Key,
    ) -> Result<Option<Arc<MeasureLayout>>, String> {
        let point = self.filler_center(key)?;
        let [left, right] = Self::filler_stack_bounds(stack)?;
        if point[0] < left || point[0] > right {
            return Ok(None);
        }
        let (staff, part) = self.filler_owner(key)?;
        let owners = self.clef_ownership.as_ref().unwrap();
        let part_layout = layout
            .parts
            .iter()
            .find(|p| Arc::ptr_eq(&p.part, &part))
            .ok_or("music_filler_actual_Part_layout_required")?;
        // Part.getMeasureAt can choose another Staff's first containing area.
        // The precise test below still uses the actual Inter's original Staff.
        let Some(measure) =
            part_layout.measure_at(&owners.staffs, &owners.areas, point.map(|n| n as f64))?
        else {
            return Ok(None);
        };
        if !stack.measures.iter().any(|m| Arc::ptr_eq(m, &measure)) {
            return Ok(None);
        }
        let [left, right] = measure.abscissae(&staff)?;
        if left < point[0] && point[0] <= right {
            Ok(Some(measure))
        } else {
            Ok(None)
        }
    }

    // One source process call, with all mixed Clef+Key filtering and additions
    // staged together. It does not run links, contexts, weak purge, or editing.
    pub(crate) fn fill_owned_clefs_and_keys(
        &mut self,
        layout: &Layout,
        system: u64,
    ) -> Result<Value, String> {
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_clef_layout_required")?;
        let stacks = layout
            .stacks
            .iter()
            .filter(|s| s.stack.system == system)
            .collect::<Vec<_>>();
        let retained = owners
            .stacks
            .iter()
            .filter(|s| s.stack.system == system)
            .collect::<Vec<_>>();
        if stacks.len() != retained.len()
            || stacks.iter().zip(&retained).any(|(a, b)| {
                !Arc::ptr_eq(&a.stack, &b.stack)
                    || a.measures.len() != b.measures.len()
                    || a.measures
                        .iter()
                        .zip(&b.measures)
                        .any(|(x, y)| !Arc::ptr_eq(x, y))
            })
        {
            return Err("music_filler_same_actual_Stack_order_required".into());
        }
        let input_parts = layout
            .parts
            .iter()
            .filter(|p| p.part.system == system)
            .collect::<Vec<_>>();
        let held_parts = owners
            .parts
            .iter()
            .filter(|p| p.system == system)
            .collect::<Vec<_>>();
        if input_parts.len() != held_parts.len() {
            return Err("music_filler_same_actual_Part_order_required".into());
        }
        for (part, held_part) in input_parts.iter().zip(held_parts) {
            let held = owners
                .measures
                .iter()
                .filter(|m| Arc::ptr_eq(&m.measure.part, held_part))
                .collect::<Vec<_>>();
            if !Arc::ptr_eq(&part.part, held_part)
                || held.len() != part.measures.len()
                || held
                    .iter()
                    .zip(&part.measures)
                    .any(|(a, b)| !Arc::ptr_eq(&a.measure, b))
            {
                return Err("music_filler_same_actual_Part_Measure_order_required".into());
            }
        }
        // Source SIGraph.inters preserves vertexSet order, not center or class order.
        let keys = self
            .vertices
            .iter()
            .filter(|v| {
                !v.removed
                    && v.key.system == system
                    && matches!(v.data["kind"].as_str(), Some("ClefInter" | "KeyInter"))
            })
            .map(|v| v.key)
            .collect::<Vec<_>>();
        for key in &keys {
            self.filler_owner(*key)?;
        }
        let mut next = self.clone();
        let mut observations = Vec::new();
        for stack in stacks {
            let mut kept = Vec::new();
            // Original stack.filter completes before this Stack's first addInter.
            for key in &keys {
                if let Some(measure) = next.filter_filler_inter(layout, stack, *key)? {
                    kept.push((*key, measure));
                }
            }
            for (key, filtered_measure) in kept {
                let (_, part) = next.filler_owner(key)?;
                let measure = stack
                    .measures
                    .iter()
                    .find(|m| Arc::ptr_eq(&m.part, &part))
                    .ok_or("music_filler_actual_Part_Measure_required")?;
                if !Arc::ptr_eq(measure, &filtered_measure) {
                    return Err("music_filler_actual_filter_owner_changed".into());
                }
                #[cfg(test)]
                let before_peers = tests::observe_peer_memberships(&next, layout, system);
                let before_clefs = next.owned_measure_clefs(measure)?.to_vec();
                let before_keys = next.owned_measure_keys(measure)?.to_vec();
                let id = next.id(key).unwrap();
                if next.vertices[id - 1].data["kind"] == "ClefInter" {
                    next.add_clef_to_measure(key, measure)?;
                } else {
                    next.add_key_to_measure(key, measure)?;
                }
                let observation = json!({"stack":stack.stack.index,"inter":key,
                    "part":part.id,"measure_index":measure.index,
                    "before_clefs":before_clefs,"after_clefs":next.owned_measure_clefs(measure)?,
                    "before_keys":before_keys,"after_keys":next.owned_measure_keys(measure)?});
                #[cfg(test)]
                let observation = {
                    let mut observation = observation;
                    observation["all_peer_memberships_before"] = before_peers;
                    observation["all_peer_memberships_after"] =
                        tests::observe_peer_memberships(&next, layout, system);
                    observation
                };
                observations.push(observation);
            }
        }
        next.clef_ownership
            .as_mut()
            .unwrap()
            .filled_systems
            .insert(system);
        *self = next;
        Ok(json!({"system":system,"filling_keys":keys,"additions":observations}))
    }

    pub(crate) fn require_header_filler_boundary(&self, system: u64) -> Result<(), String> {
        let owners = self
            .clef_ownership
            .as_ref()
            .ok_or("music_clef_layout_required")?;
        if !owners.filled_systems.contains(&system) {
            return Err("music_actual_initial_header_filler_required".into());
        }
        for vertex in self.vertices.iter().filter(|v| {
            !v.removed
                && v.key.system == system
                && matches!(v.data["kind"].as_str(), Some("ClefInter" | "KeyInter"))
        }) {
            self.filler_owner(vertex.key)?;
        }
        Ok(())
    }
}
