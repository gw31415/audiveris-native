// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    grid_ir::StaffGeometry,
    ledger_registration::StaffMembership,
    shared_sig::{Graph, Key, Phase},
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone, Default)]
pub(crate) struct LedgerMemberships {
    pub(crate) maps: Vec<StaffLedgers>,
}
#[derive(Clone)]
pub(crate) struct StaffLedgers {
    pub(crate) staff: Arc<StaffGeometry>,
    pub(crate) lines: BTreeMap<i32, Vec<Key>>,
}
impl LedgerMemberships {
    pub(crate) fn record(
        &mut self,
        system: u64,
        membership: StaffMembership,
        staffs: &[Arc<StaffGeometry>],
    ) -> Result<(), String> {
        let matches = staffs
            .iter()
            .filter(|s| s.system == system && s.id == membership.staff_id)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err("ledger_constructor_actual_staff_required".into());
        }
        let staff = matches[0].clone();
        let mut lines = BTreeMap::new();
        for (index, members) in membership.lines {
            if index == 0 || index.unsigned_abs() > 1024 || members.contains(&0) {
                return Err("invalid_constructor_ledger_membership".into());
            }
            if lines
                .insert(
                    index,
                    members
                        .into_iter()
                        .map(|local| Key {
                            system,
                            phase: Phase::Ledger,
                            local,
                        })
                        .collect(),
                )
                .is_some()
            {
                return Err("duplicate_constructor_ledger_line".into());
            }
        }
        if let Some(map) = self
            .maps
            .iter_mut()
            .find(|m| m.staff.system == system && m.staff.id == staff.id)
        {
            if !Arc::ptr_eq(&map.staff, &staff) {
                return Err("stale_constructor_actual_staff".into());
            }
            map.lines = lines;
        } else {
            self.maps.push(StaffLedgers { staff, lines });
        }
        Ok(())
    }
    pub(crate) fn retire_post_analysis(&mut self, system: u64, ledgers: &[usize]) {
        for map in self.maps.iter_mut().filter(|m| m.staff.system == system) {
            for ids in map.lines.values_mut() {
                ids.retain(|k| !ledgers.contains(&k.local));
            }
        }
    }
    pub(crate) fn remove_from_owner(&mut self, key: Key, staff_id: Option<u64>) {
        let Some(staff_id) = staff_id else { return };
        let Some(map) = self
            .maps
            .iter_mut()
            .find(|m| m.staff.system == key.system && m.staff.id == staff_id)
        else {
            return;
        };
        let found = map.lines.iter().find_map(|(index, ids)| {
            ids.iter()
                .position(|k| *k == key)
                .map(|position| (*index, position))
        });
        if let Some((index, position)) = found {
            let ids = map.lines.get_mut(&index).unwrap();
            ids.remove(position);
            if ids.is_empty() {
                map.lines.remove(&index);
            }
        }
    }
    pub(crate) fn validate(&self, graph: &Graph) -> Result<(), String> {
        for map in &self.maps {
            for ids in map.lines.values() {
                for key in ids {
                    let (body, _) = graph.retained_inter(*key)?;
                    if key.system != map.staff.system
                        || key.phase != Phase::Ledger
                        || body["kind"] != "LedgerInter"
                    {
                        return Err("ledger_constructor_actual_body_required".into());
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
