// SPDX-License-Identifier: AGPL-3.0-or-later
//! MeasuresBuilder keeps PartBarlines and Measures on the actual GRID owners.
use crate::{
    grid_ir::{PartGeometry, StaffGeometry},
    music_barline_groups::{self, Group},
    shared_sig::{Graph, Key},
    sheet_skew::SheetSkew,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug)]
pub(crate) struct StaffBarline {
    pub(crate) group: Group,
    pub(crate) key: Key,
}
#[derive(Debug)]
pub(crate) struct PartBarline {
    pub(crate) part: Arc<PartGeometry>,
    pub(crate) bars: Vec<Arc<StaffBarline>>,
}
impl PartBarline {
    fn right_x(&self, staff: &Arc<StaffGeometry>) -> Result<i64, String> {
        let index = self
            .part
            .staffs
            .iter()
            .position(|s| Arc::ptr_eq(s, staff))
            .ok_or("music_PartBarline_actual_Staff_required")?;
        let bar = self
            .bars
            .get(index)
            .ok_or("music_PartBarline_StaffBarline_required")?;
        if !Arc::ptr_eq(&bar.group.staff, staff) {
            return Err("music_PartBarline_StaffBarline_owner_mismatch".into());
        }
        Ok(bar.group.right_x)
    }
    fn projection(&self, graph: &Graph) -> Value {
        json!(
            self.bars
                .iter()
                .map(|b| json!({"staff":b.group.staff.id,
            "member_ids":b.group.members.iter().map(|k|graph.id(*k)).collect::<Vec<_>>(),
            "staff_barline_id":graph.id(b.key)}))
                .collect::<Vec<_>>()
        )
    }
}
#[derive(Debug)]
pub(crate) struct Stack {
    pub(crate) system: u64,
    pub(crate) index: usize,
}
#[derive(Debug)]
pub(crate) struct MeasureLayout {
    pub(crate) part: Arc<PartGeometry>,
    pub(crate) index: usize,
    pub(crate) stack: Arc<Stack>,
    pub(crate) contained_left: Option<Arc<PartBarline>>,
    pub(crate) effective_left: Option<Arc<PartBarline>>,
    pub(crate) right: Option<Arc<PartBarline>>,
}
impl MeasureLayout {
    pub(crate) fn abscissae(&self, staff: &Arc<StaffGeometry>) -> Result<[i64; 2], String> {
        if !self.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)) {
            return Err("music_Measure_actual_Part_Staff_required".into());
        }
        let left = self
            .effective_left
            .as_ref()
            .map(|b| b.right_x(staff))
            .transpose()?
            .unwrap_or(staff.abscissae[0] as i64);
        let right = self
            .right
            .as_ref()
            .map(|b| b.right_x(staff))
            .transpose()?
            .unwrap_or(staff.abscissae[1] as i64);
        if right <= left {
            return Err("invalid_music_measure_width".into());
        }
        Ok([left, right])
    }
}
pub(crate) struct PartLayout {
    pub(crate) part: Arc<PartGeometry>,
    pub(crate) left: Option<Arc<PartBarline>>,
    pub(crate) measures: Vec<Arc<MeasureLayout>>,
}
impl PartLayout {
    /// Part.getMeasureAt selects the first containing Staff area, then the first
    /// right bar after the point. A null Staff source stays null for the fallback.
    pub(crate) fn measure_at(
        &self,
        staffs: &[Arc<StaffGeometry>],
        areas: &[Arc<crate::system_area_geometry::Band>],
        point: [f64; 2],
    ) -> Result<Option<Arc<MeasureLayout>>, String> {
        if staffs.len() != areas.len() || point.iter().any(|v| !v.is_finite()) {
            return Err("music_actual_Part_Staff_area_required".into());
        }
        for staff in &self.part.staffs {
            let index = staffs
                .iter()
                .position(|s| Arc::ptr_eq(s, staff))
                .ok_or("music_actual_Part_Staff_area_required")?;
            if !areas[index].contains(point)? {
                continue;
            }
            if point[0] < staff.abscissae[0] as f64 || point[0] > staff.abscissae[1] as f64 {
                return Ok(None);
            }
            for measure in &self.measures {
                if measure
                    .right
                    .as_ref()
                    .map(|b| b.right_x(staff))
                    .transpose()?
                    .is_none_or(|x| point[0] <= x as f64)
                {
                    return Ok(Some(measure.clone()));
                }
            }
            return Ok(None);
        }
        Ok(None)
    }
}
pub(crate) struct StackOwner {
    pub(crate) stack: Arc<Stack>,
    pub(crate) measures: Vec<Arc<MeasureLayout>>,
}
pub(crate) struct Layout {
    pub(crate) parts: Vec<PartLayout>,
    pub(crate) stacks: Vec<StackOwner>,
}
impl Layout {
    pub(crate) fn projection(&self, graph: &Graph) -> Value {
        json!({"parts":self.parts.iter().map(|p|json!({"system_id":p.part.system,
            "part_id":p.part.id,"staff_ids":p.part.staffs.iter().map(|s|s.id).collect::<Vec<_>>(),
            "left_bar":p.left.as_ref().map(|b|b.projection(graph)),
            "measures":p.measures.iter().map(|m|json!({"index":m.index,"stack_index":m.stack.index,
                "abscissae":m.part.staffs.iter().map(|s|json!({"staff":s.id,"bounds":m.abscissae(s).expect("validated actual Measure")})).collect::<Vec<_>>(),
                "contained_left":m.contained_left.as_ref().map(|b|b.projection(graph)),
                "effective_left":m.effective_left.as_ref().map(|b|b.projection(graph)),
                "right_bar":m.right.as_ref().map(|b|b.projection(graph))})).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "stacks":self.stacks.iter().map(|s|json!({"system_id":s.stack.system,"index":s.stack.index,
                "part_ids":s.measures.iter().map(|m|m.part.id).collect::<Vec<_>>()})).collect::<Vec<_>>()})
    }
}
fn staff_end(bar: &StaffBarline, side: usize, interline: usize) -> bool {
    (bar.group.bounds[0] + bar.group.bounds[2] / 2 - bar.group.staff.abscissae[side] as i64).abs()
        <= interline as i64
}
/// Registration and container allocation commit together; reports are never inputs.
pub(crate) fn build(
    graph: &mut Graph,
    staffs: &[Arc<StaffGeometry>],
    parts: &[Arc<PartGeometry>],
    skew: SheetSkew,
    interline: usize,
) -> Result<Layout, String> {
    let mut next = graph.clone();
    let mut result = Layout {
        parts: Vec::new(),
        stacks: Vec::new(),
    };
    let systems = staffs
        .iter()
        .map(|s| s.system)
        .collect::<std::collections::BTreeSet<_>>();
    for system in systems {
        let staves = staffs
            .iter()
            .filter(|s| s.system == system)
            .cloned()
            .collect::<Vec<_>>();
        let mut groups = Vec::new();
        for staff in &staves {
            groups.extend(music_barline_groups::groups(&next, staff.clone(), skew)?);
        }
        music_barline_groups::complete_columns(&groups, &staves, interline)?;
        let count = groups
            .iter()
            .filter(|g| Arc::ptr_eq(&g.staff, &staves[0]))
            .count();
        let mut bars = BTreeMap::<u64, Vec<Arc<StaffBarline>>>::new();
        for staff in &staves {
            bars.insert(staff.id, Vec::new());
        }
        for column in 0..count {
            for staff in &staves {
                let group = groups
                    .iter()
                    .filter(|g| Arc::ptr_eq(&g.staff, staff))
                    .nth(column)
                    .ok_or("music_complete_StaffBarline_columns_required")?
                    .clone();
                let (key, _) = group.register(&mut next, interline)?;
                bars.get_mut(&staff.id)
                    .unwrap()
                    .push(Arc::new(StaffBarline { group, key }));
            }
        }
        let stack_start = result.stacks.len();
        for part in parts.iter().filter(|p| p.system == system) {
            let top = part
                .staffs
                .first()
                .ok_or("music_actual_Part_Staff_required")?;
            for staff in &part.staffs {
                if !staves.iter().any(|s| Arc::ptr_eq(s, staff)) || staff.lines.len() != 5 {
                    return Err("music_actual_ordinary_Part_Staff_required".into());
                }
            }
            let top_bars = bars
                .get(&top.id)
                .ok_or("music_actual_top_Staff_bars_required")?;
            let mut layout = PartLayout {
                part: part.clone(),
                left: None,
                measures: Vec::new(),
            };
            let mut pending = None::<Arc<PartBarline>>;
            let mut previous = None::<Arc<PartBarline>>;
            for (column, top_bar) in top_bars.iter().enumerate() {
                let bar = Arc::new(PartBarline {
                    part: part.clone(),
                    bars: part
                        .staffs
                        .iter()
                        .map(|s| {
                            bars.get(&s.id)
                                .and_then(|b| b.get(column))
                                .cloned()
                                .ok_or("music_actual_PartBarline_member_required")
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                });
                let leading = staff_end(top_bar, 0, interline);
                if leading {
                    layout.left = Some(bar.clone());
                } else {
                    let index = layout.measures.len() + 1;
                    while result.stacks.len() - stack_start < index {
                        result.stacks.push(StackOwner {
                            stack: Arc::new(Stack {
                                system,
                                index: result.stacks.len() - stack_start + 1,
                            }),
                            measures: Vec::new(),
                        });
                    }
                    let stack = &mut result.stacks[stack_start + index - 1];
                    let contained_left = pending.take();
                    let effective_left = contained_left
                        .clone()
                        .or_else(|| previous.clone())
                        .or_else(|| layout.left.clone());
                    let measure = Arc::new(MeasureLayout {
                        part: part.clone(),
                        index,
                        stack: stack.stack.clone(),
                        contained_left,
                        effective_left,
                        right: Some(bar.clone()),
                    });
                    for staff in &part.staffs {
                        measure.abscissae(staff)?;
                    }
                    stack.measures.push(measure.clone());
                    layout.measures.push(measure);
                    previous = Some(bar.clone());
                }
                if top_bar.group.members.len() >= 3 {
                    pending = Some(bar);
                }
            }
            if top_bars.is_empty() || !staff_end(top_bars.last().unwrap(), 1, interline) {
                let index = layout.measures.len() + 1;
                while result.stacks.len() - stack_start < index {
                    result.stacks.push(StackOwner {
                        stack: Arc::new(Stack {
                            system,
                            index: result.stacks.len() - stack_start + 1,
                        }),
                        measures: Vec::new(),
                    });
                }
                let stack = &mut result.stacks[stack_start + index - 1];
                let effective_left = pending.clone().or(previous).or_else(|| layout.left.clone());
                let measure = Arc::new(MeasureLayout {
                    part: part.clone(),
                    index,
                    stack: stack.stack.clone(),
                    contained_left: pending,
                    effective_left,
                    right: None,
                });
                for staff in &part.staffs {
                    measure.abscissae(staff)?;
                }
                stack.measures.push(measure.clone());
                layout.measures.push(measure);
            }
            result.parts.push(layout);
        }
        if let Some(last) = result.stacks[stack_start..].last() {
            let first = last
                .measures
                .first()
                .ok_or("music_actual_Stack_Measure_required")?;
            let [left, right] = first.abscissae(&first.part.staffs[0])?;
            if first.right.is_none() && right - left < 4 * interline as i64 {
                return Err("music_courtesy_measure_Head_purge_pending".into());
            }
        }
    }
    *graph = next;
    Ok(result)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) fn assert_real_layout(owner: &crate::stem_image_owned::OwnedItems) {
    tests::assert_real_layout(owner);
}
