// SPDX-License-Identifier: AGPL-3.0-or-later
//! MeasuresBuilder groups preserve canonical physical bar references and Staff owners.
use crate::{
    grid_ir::StaffGeometry,
    shared_sig::{Graph, Key},
    sheet_skew::SheetSkew,
};
use serde_json::{Value, json};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(crate) struct Group {
    pub(crate) staff: Arc<StaffGeometry>,
    pub(crate) members: Vec<Key>,
    pub(crate) center: [f64; 2],
    pub(crate) deskew_x: f64,
    pub(crate) bounds: [i64; 4],
    pub(crate) left_x: i64,
    pub(crate) right_x: i64,
    pub(crate) grade: f64,
    pub(crate) frozen: bool,
    pub(crate) style: Option<&'static str>,
}
fn bounds(n: &Value) -> Result<[i64; 4], String> {
    let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
        .map_err(|_| "music_bar_group_actual_bounds_required")?;
    if b[2] <= 0 || b[3] <= 0 || b.iter().any(|v| v.unsigned_abs() > 1_000_000) {
        return Err("music_bar_group_actual_bounds_invalid".into());
    }
    Ok(b)
}
pub(crate) fn groups(
    graph: &Graph,
    staff: Arc<StaffGeometry>,
    skew: SheetSkew,
) -> Result<Vec<Group>, String> {
    let mut bars = graph
        .live_inter_keys(staff.system, "BarlineInter")
        .into_iter()
        .filter(|k| {
            graph
                .retained_inter(*k)
                .is_ok_and(|(n, _)| n["staff_id"] == staff.id)
        })
        .collect::<Vec<_>>();
    let mut centers = std::collections::BTreeMap::new();
    for k in &bars {
        let (n, _) = graph.retained_inter(*k)?;
        let b = bounds(n)?;
        centers.insert(*k, [b[0] + b[2] / 2, b[1] + b[3] / 2]);
    }
    bars.sort_by_key(|k| centers[k][0]);
    let mut result = Vec::new();
    let mut start = 0;
    while start < bars.len() {
        let mut end = start + 1;
        while end < bars.len() && graph.has_relation(bars[end - 1], bars[end], "BarGroupRelation") {
            end += 1;
        }
        let members = bars[start..end].to_vec();
        let mut center = [0., 0.];
        let mut union = None::<[i64; 4]>;
        let mut grade = 0.;
        let mut frozen = false;
        let mut shapes = Vec::new();
        for k in &members {
            let (n, _) = graph.retained_inter(*k)?;
            let b = bounds(n)?;
            center[0] += centers[k][0] as f64;
            center[1] += centers[k][1] as f64;
            union = Some(if let Some(a) = union {
                let l = a[0].min(b[0]);
                let t = a[1].min(b[1]);
                [
                    l,
                    t,
                    (a[0] + a[2]).max(b[0] + b[2]) - l,
                    (a[1] + a[3]).max(b[1] + b[3]) - t,
                ]
            } else {
                b
            });
            grade += n["grade"]
                .as_f64()
                .filter(|v| (0.0..=1.).contains(v))
                .ok_or("music_bar_group_grade_required")?;
            frozen |= n["frozen"] == true;
            shapes.push(
                n["shape"]
                    .as_str()
                    .ok_or("music_bar_group_shape_required")?,
            );
        }
        let count = members.len() as f64;
        center[0] /= count;
        center[1] /= count;
        grade /= count;
        let style = match shapes.as_slice() {
            ["THIN_BARLINE"] => Some("REGULAR"),
            ["THICK_BARLINE"] => Some("HEAVY"),
            [_] => Some("NONE"),
            ["THIN_BARLINE", "THIN_BARLINE"] => Some("LIGHT_LIGHT"),
            ["THIN_BARLINE", _] => Some("LIGHT_HEAVY"),
            [_, "THIN_BARLINE"] => Some("HEAVY_LIGHT"),
            [_, _] => Some("HEAVY_HEAVY"),
            [_, _, _] => Some("LIGHT_HEAVY_LIGHT"),
            _ => None,
        };
        // Original affine origin translation cancels from column joins and ordering.
        let angle = skew.slope().atan();
        let deskew_x = angle.cos() * center[0] + angle.sin() * center[1];
        let left_x = centers[&members[0]][0];
        let right_x = centers[members.last().unwrap()][0];
        result.push(Group {
            staff: staff.clone(),
            members,
            center,
            deskew_x,
            bounds: union.unwrap(),
            left_x,
            right_x,
            grade,
            frozen,
            style,
        });
        start = end;
    }
    Ok(result)
}
pub(crate) fn complete_columns(
    groups: &[Group],
    staffs: &[Arc<StaffGeometry>],
    interline: usize,
) -> Result<(), String> {
    let mut columns = Vec::<Vec<&Group>>::new();
    for staff in staffs {
        for group in groups.iter().filter(|g| Arc::ptr_eq(&g.staff, staff)) {
            if let Some(column) = columns.iter_mut().find(|c| {
                (c.iter().map(|g| g.deskew_x).sum::<f64>() / c.len() as f64 - group.deskew_x).abs()
                    <= interline as f64
            }) {
                if !column.iter().any(|g| Arc::ptr_eq(&g.staff, staff)) {
                    column.push(group);
                }
            } else {
                columns.push(vec![group]);
                columns.sort_by(|a, b| {
                    (a.iter().map(|g| g.deskew_x).sum::<f64>() / a.len() as f64)
                        .total_cmp(&(b.iter().map(|g| g.deskew_x).sum::<f64>() / b.len() as f64))
                });
            }
        }
    }
    if columns.iter().any(|c| c.len() != staffs.len()) {
        return Err("music_bar_column_minority_delete_or_majority_expand_pending".into());
    }
    Ok(())
}
impl Group {
    pub(crate) fn register(
        &self,
        graph: &mut Graph,
        interline: usize,
    ) -> Result<(Key, usize), String> {
        let mut next = graph.clone();
        let key = Key {
            system: self.staff.system,
            phase: crate::shared_sig::Phase::StaffBarline,
            local: graph
                .id(self.members[0])
                .ok_or("music_bar_group_canonical_member_required")?,
        };
        let member_ids = self
            .members
            .iter()
            .map(|k| {
                let (n, removed) = graph.retained_inter(*k)?;
                if removed
                    || k.system != self.staff.system
                    || n["kind"] != "BarlineInter"
                    || n["staff_id"] != self.staff.id
                {
                    return Err("music_bar_group_canonical_member_required".to_owned());
                }
                graph
                    .id(*k)
                    .ok_or_else(|| "music_bar_group_canonical_member_required".to_owned())
            })
            .collect::<Result<Vec<_>, String>>()?;
        let center_x = self.bounds[0] + self.bounds[2] / 2;
        let leading = (center_x - self.staff.abscissae[0] as i64).abs() <= interline as i64;
        let id=next.vertex(key,json!({"kind":"StaffBarlineInter","staff_id":self.staff.id,"bounds":self.bounds,"member_ids":member_ids,"style":self.style,"grade":self.grade,"frozen":self.frozen,"manual":false,"back_to_back":self.members.len()>=3,"leading":leading,"group_center":self.center,"left_x":self.left_x,"right_x":self.right_x}),false)?;
        next.relations(
            self.staff.system,
            crate::shared_sig::Phase::StaffBarline,
            self.members
                .iter()
                .map(|k| (key, *k, json!({"kind":"Containment"})))
                .collect(),
        )?;
        *graph = next;
        Ok((key, id))
    }
}
#[cfg(test)]
mod tests;
