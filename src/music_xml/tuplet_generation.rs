// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    TupletPlan,
    tuplet_groups::{existing_on, measure_index},
};
use crate::{music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, shared_sig::Key};

impl OwnedMusic {
    /// Fresh implicit lifecycle on retained ordinary single-staff owners.
    /// SheetStub font selection and automatic rhythm/export admission follow.
    pub fn generate_rhythm_tuplet(
        &mut self,
        number: usize,
        group: &[Key],
        font: &TupletFontSymbols,
    ) -> Result<Key, String> {
        let mi = measure_index(&self.measures, number)?;
        if let Some(key) = existing_on(&self.owner.graph, &self.measures, &self.chords, mi, group)?
        {
            return Ok(key);
        }
        let old_graph = self.owner.graph.clone();
        let stage = self.measures[mi].tuplets.clone();
        let old_measure = std::mem::replace(&mut self.measures[mi].tuplets, stage);
        let stage = self.measures[mi].stack_tuplets.clone();
        let old_stack = std::mem::replace(&mut self.measures[mi].stack_tuplets, stage);
        let result = self.generate_tuplet_on(number, mi, group, font);
        if result.is_err() {
            self.owner.graph = old_graph;
            self.measures[mi].tuplets = old_measure;
            self.measures[mi].stack_tuplets = old_stack;
        }
        result
    }
    fn generate_tuplet_on(
        &mut self,
        number: usize,
        mi: usize,
        group: &[Key],
        font: &TupletFontSymbols,
    ) -> Result<Key, String> {
        let TupletPlan::New { shape, bounds } = self.plan_rhythm_tuplet(number, group, font)?
        else {
            return Err("music_tuplet_fresh_plan_required".into());
        };
        let measure = &self.measures[mi];
        let system = measure.system;
        let staffs = self
            .owner
            .staffs
            .iter()
            .filter(|s| s.system == system)
            .collect::<Vec<_>>();
        if staffs.len() != 1
            || staffs[0].id != measure.staff
            || self
                .measures
                .iter()
                .filter(|m| m.system == system)
                .any(|m| m.staff != measure.staff || m.part != measure.part)
        {
            return Err("music_tuplet_shared_stack_allocation_pending".into());
        }
        let x = bounds[0]
            .checked_add(bounds[2] / 2)
            .ok_or("music_tuplet_actual_center_required")?;
        // Actual bar/staff abscissa boundary comparison is the next coverage
        // unit; no guessed Stack ownership is admitted at its boundary.
        let margin = self.owner.interline as i64;
        if x <= measure.left + margin || x >= measure.right - margin {
            return Err("music_tuplet_stack_abscissa_boundary_pending".into());
        }
        let key = self.owner.graph.fresh_tuplet_key(system)?;
        #[cfg(test)]
        let clocks = self
            .chords
            .iter()
            .filter(|ch| ch.measure == mi)
            .map(|ch| ch.key)
            .collect::<Vec<_>>();
        self.owner.graph.register_fresh_tuplet(key, shape, bounds)?;
        #[cfg(test)]
        {
            let stack = self.tuplet_stack_union(mi);
            self.owner.graph.observe_tuplet_addition(
                key,
                None,
                "vertex_registered",
                &self.measures[mi].tuplets,
                &stack,
                &clocks,
            )?;
        }
        self.owner.graph.attach_fresh_tuplet(key)?;
        if self.owner.graph.tuplet_part(key)?.is_some() {
            return Err("music_tuplet_fresh_part_must_be_null".into());
        }
        self.measures[mi].stack_tuplets.push(key);
        self.owner.graph.set_tuplet_abnormal(key, true)?;
        self.owner.graph.set_tuplet_abnormal(key, false)?;
        self.measures[mi].tuplets.push(key);
        for &chord in group {
            let _edge = self
                .owner
                .graph
                .add_generated_chord_tuplet(chord, key, shape)?;
            #[cfg(test)]
            {
                let stack = self.tuplet_stack_union(mi);
                self.owner.graph.observe_tuplet_addition(
                    key,
                    Some(_edge),
                    "chord_tuplet_added",
                    &self.measures[mi].tuplets,
                    &stack,
                    &clocks,
                )?;
            }
        }
        Ok(key)
    }
    #[cfg(test)]
    fn tuplet_stack_union(&self, mi: usize) -> Vec<Key> {
        let mut keys = self.measures[mi].tuplets.clone();
        for &key in &self.measures[mi].stack_tuplets {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }
}
