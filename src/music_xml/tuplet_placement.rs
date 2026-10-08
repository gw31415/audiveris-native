// SPDX-License-Identifier: AGPL-3.0-or-later
use super::tuplet_groups::{actual, existing_on, measure_index};
use crate::{
    music_font_symbols::{TupletFontSymbols, TupletShape},
    music_measures::{Chord, Measure, OwnedMusic},
    shared_sig::{Graph, Key},
};

/// Existing-object or pre-add placement boundary of the compared generator.
#[derive(Debug, PartialEq, Eq)]
pub enum TupletPlan {
    Existing(Key),
    New {
        shape: TupletShape,
        bounds: [i64; 4],
    },
}
impl OwnedMusic {
    /// Allocation callbacks and SheetStub family binding remain separate.
    pub fn plan_rhythm_tuplet(
        &mut self,
        number: usize,
        group: &[Key],
        font: &TupletFontSymbols,
    ) -> Result<TupletPlan, String> {
        let mi = measure_index(&self.measures, number)?;
        if let Some(key) = existing_on(&self.owner.graph, &self.measures, &self.chords, mi, group)?
        {
            return Ok(TupletPlan::Existing(key));
        }
        if font.interline() != self.owner.interline {
            return Err("music_tuplet_actual_sheet_scale_required".into());
        }
        let shape = TupletShape::inferred(group.len());
        let mut graph = self.owner.graph.clone();
        let bounds = infer_on(
            &mut graph,
            &self.measures,
            &self.chords,
            mi,
            group,
            font.dimension(shape),
        )?;
        self.owner.graph = graph;
        Ok(TupletPlan::New { shape, bounds })
    }
}
pub(super) fn infer_on(
    graph: &mut Graph,
    measures: &[Measure],
    chords: &[Chord],
    mi: usize,
    group: &[Key],
    dim: [i64; 2],
) -> Result<[i64; 4], String> {
    let mut dirs = std::collections::BTreeMap::new();
    for &key in group {
        let ch = actual(graph, measures, chords, mi, key)?;
        let read = super::beam_rest_lookup::chord_geometry(graph, ch)?;
        let dir = if read.stem.is_none() {
            0
        } else {
            let h = read
                .head
                .ok_or("music_tuplet_actual_chord_direction_required")?;
            let t = read
                .tail
                .ok_or("music_tuplet_actual_chord_direction_required")?;
            (t[1] - h[1]).signum()
        };
        if dir != 0 {
            *dirs.entry(dir).or_insert(0usize) += 1;
        }
    }
    let (mut dir, mut best) = (0, 0);
    for (direction, count) in dirs {
        if dir == 0 || best < count {
            dir = direction;
            best = count;
        }
    }
    if dir == 0 {
        dir = -1;
    }
    let mut bounds: Option<[i64; 4]> = None;
    for &key in group {
        let ch = actual(graph, measures, chords, mi, key)?;
        let b = super::beam_rest_lookup::chord_geometry(graph, ch)?
            .bounds
            .ok_or("music_tuplet_actual_group_bounds_required")?;
        bounds = Some(if let Some(a) = bounds {
            let x = a[0].min(b[0]);
            let y = a[1].min(b[1]);
            [
                x,
                y,
                (a[0] + a[2]).max(b[0] + b[2]) - x,
                (a[1] + a[3]).max(b[1] + b[3]) - y,
            ]
        } else {
            b
        });
    }
    let b = bounds.ok_or("music_tuplet_actual_group_bounds_required")?;
    let x = b[0] + b[2] / 2 - dim[0] / 2;
    let y = b[1] + b[3] / 2 + dir * (b[3] / 2 + dim[1] / 10 + if dir < 0 { dim[1] } else { 0 });
    Ok([x, y, dim[0], dim[1]])
}
