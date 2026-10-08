// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_measures::OwnedMusic, shared_sig::Key};
impl OwnedMusic {
    /// Original collection order includes duplicate actual references and rejects a null member.
    pub fn remove_rhythm_tuplets(
        &mut self,
        number: usize,
        tuplets: &[Option<Key>],
    ) -> Result<(), String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        self.with_tuplet_owners(|owner| {
            for &reference in tuplets {
                let key = reference.ok_or("music_tuplet_null_collection_member")?;
                owner.remove_tuplet_reference(mi, key)?;
            }
            Ok(())
        })
    }
    fn remove_tuplet_reference(&mut self, mi: usize, key: Key) -> Result<(), String> {
        let (implicit, _, removed) = self.owner.graph.retained_tuplet_status(key)?;
        if key.system != self.measures[mi].system {
            return Err("music_actual_tuplet_measure_required".into());
        }
        if !removed {
            for source in self.owner.graph.tuplet_chords(key)? {
                let ch = self
                    .chords
                    .iter()
                    .find(|ch| ch.key == source)
                    .ok_or("music_actual_tuplet_chord_owner_required")?;
                super::beam_rest::validate_measure_chord(
                    &self.owner.graph,
                    self.measures
                        .get(ch.measure)
                        .ok_or("music_actual_tuplet_measure_required")?,
                    ch,
                )?;
            }
        }
        self.measures[mi].tuplets.retain(|&t| t != key);
        self.remove_tuplet_from_stack(mi, key)?;
        // Repeated removal still detaches caller containers before the actual Inter's no-op.
        if !removed
            && !implicit
            && let Some(x) = self.owner.graph.tuplet_center_x(key)?
        {
            let measure = &self.measures[mi];
            let staves = self
                .owner
                .staffs
                .iter()
                .filter(|s| s.system == key.system)
                .collect::<Vec<_>>();
            if staves.len() != 1
                || staves[0].id != measure.staff
                || self
                    .measures
                    .iter()
                    .filter(|m| m.system == key.system)
                    .any(|m| m.staff != measure.staff || m.part != measure.part)
            {
                return Err("music_tuplet_shared_stack_removal_pending".into());
            }
            if let Some(si) = self
                .measures
                .iter()
                .position(|m| m.system == key.system && x > m.left && x <= m.right)
            {
                self.remove_tuplet_from_stack(si, key)?;
            }
        }
        let measure = &self.measures[mi];
        let mut all = measure.tuplets.clone();
        for &t in &measure.stack_tuplets {
            if !all.contains(&t) {
                all.push(t);
            }
        }
        #[cfg(test)]
        self.owner.graph.trace_tuplet_removal_containers(Some(
            self.measures
                .iter()
                .map(|m| (m.tuplets.clone(), m.stack_tuplets.clone()))
                .collect(),
        ));
        self.owner
            .graph
            .remove_owned_tuplet(key, &self.measures[mi].tuplets, &all)?;
        #[cfg(test)]
        self.owner.graph.trace_tuplet_removal_containers(None);
        Ok(())
    }
    fn remove_tuplet_from_stack(&mut self, si: usize, key: Key) -> Result<(), String> {
        let part = self.owner.graph.retained_tuplet_part(key)?;
        if let Some(part) = part {
            let stack = &self.measures[si];
            let index = self
                .measures
                .iter()
                .position(|m| m.system == stack.system && m.index == stack.index && m.part == part)
                .ok_or("music_actual_tuplet_stack_part_required")?;
            self.measures[index].tuplets.retain(|&t| t != key);
        }
        self.measures[si].stack_tuplets.retain(|&t| t != key);
        Ok(())
    }
}
