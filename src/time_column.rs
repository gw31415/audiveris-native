// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Fresh-header TimeBuilder.filterCandidates and TimeColumn selection/mutation.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{
    GrayImage,
    key_preparation::StaffInput,
    time_candidates,
    time_preparation::{self, Preparation},
    time_registration,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn integer(v: &Value) -> Result<i64, String> {
    v.as_i64().ok_or("invalid_time_column_integer".into())
}
fn number(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|x| x.is_finite())
        .ok_or("invalid_time_column_number".into())
}
fn rect(v: &Value) -> Result<[i64; 4], String> {
    crate::key_state::rect(v)
}
fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
fn union(a: [i64; 4], b: [i64; 4]) -> [i64; 4] {
    let x = a[0].min(b[0]);
    let y = a[1].min(b[1]);
    [
        x,
        y,
        (a[0] + a[2]).max(b[0] + b[2]) - x,
        (a[1] + a[3]).max(b[1] + b[3]) - y,
    ]
}
pub(crate) fn time_number(shape: &str) -> Result<i64, String> {
    Ok(match shape {
        "TIME_ZERO" => 0,
        "TIME_ONE" => 1,
        "TIME_TWO" => 2,
        "TIME_THREE" => 3,
        "TIME_FOUR" => 4,
        "TIME_FIVE" => 5,
        "TIME_SIX" => 6,
        "TIME_SEVEN" => 7,
        "TIME_EIGHT" => 8,
        "TIME_NINE" => 9,
        "TIME_TWELVE" => 12,
        "TIME_SIXTEEN" => 16,
        _ => return Err("unsupported_time_number_shape".into()),
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct TimeValue {
    specific: Option<&'static str>,
    num: i64,
    den: i64,
}
impl TimeValue {
    fn whole(shape: &str) -> Result<Self, String> {
        let (specific, num, den) = match shape {
            "COMMON_TIME" => (Some("COMMON_TIME"), 4, 4),
            "CUT_TIME" => (Some("CUT_TIME"), 2, 2),
            "TIME_TWO_TWO" => (None, 2, 2),
            "TIME_TWO_FOUR" => (None, 2, 4),
            "TIME_THREE_FOUR" => (None, 3, 4),
            "TIME_FOUR_FOUR" => (None, 4, 4),
            "TIME_FIVE_FOUR" => (None, 5, 4),
            "TIME_SIX_FOUR" => (None, 6, 4),
            "TIME_THREE_EIGHT" => (None, 3, 8),
            "TIME_SIX_EIGHT" => (None, 6, 8),
            "TIME_TWELVE_EIGHT" => (None, 12, 8),
            _ => return Err("unsupported_time_whole_shape".into()),
        };
        Ok(Self { specific, num, den })
    }
    fn label(&self) -> String {
        self.specific
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{}/{}", self.num, self.den))
    }
    fn hash(&self) -> u32 {
        let h = (83 * 7 + 97 * (97 * 7 + self.num) + self.den) as u32;
        h ^ (h >> 16)
    }
}
pub(crate) fn supported(num: i64, den: i64) -> bool {
    matches!(
        (num, den),
        (2, 2)
            | (3, 2)
            | (2, 4)
            | (3, 4)
            | (4, 4)
            | (5, 4)
            | (6, 4)
            | (3, 8)
            | (6, 8)
            | (7, 8)
            | (9, 8)
            | (12, 8)
    )
}
/// Java HashMap final bucket traversal; stable chain insertion and power-of-two resize.
fn hash_order(values: &[TimeValue]) -> Result<Vec<usize>, String> {
    if values.len() > 16 {
        return Err("unsupported_time_value_map_budget".into());
    }
    let mut capacity = 16;
    while values.len() > capacity * 3 / 4 {
        capacity *= 2;
    }
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by_key(|&i| values[i].hash() as usize & (capacity - 1));
    let mut buckets = BTreeMap::new();
    for v in values {
        let n = buckets
            .entry(v.hash() as usize & (capacity - 1))
            .or_insert(0);
        *n += 1;
        if *n >= 8 {
            return Err("unsupported_time_hash_treeification".into());
        }
    }
    Ok(order)
}
#[derive(Clone)]
struct Edge {
    kind: String,
    source: usize,
    target: usize,
    cause: Option<String>,
}
#[derive(Default)]
struct Graph {
    nodes: Vec<Value>,
    edges: Vec<Edge>,
}
impl Graph {
    fn add(&mut self, mut node: Value) -> Result<usize, String> {
        if self.nodes.len() >= 32768 {
            return Err("unsupported_time_graph_budget".into());
        }
        let id = self.nodes.len();
        node["id"] = json!(id + 1);
        if node.get("removed").is_none() {
            node["removed"] = json!(false);
        }
        if node.get("frozen").is_none() {
            node["frozen"] = json!(false);
        }
        self.nodes.push(node);
        Ok(id)
    }
    fn live(&self, id: usize) -> bool {
        self.nodes[id]["removed"] != true
    }
    fn ensemble(&self, id: usize) -> bool {
        matches!(
            self.nodes[id]["kind"].as_str(),
            Some("KeyInter" | "TimePairInter")
        )
    }
    fn members(&self, id: usize) -> Vec<usize> {
        let mut out = self
            .edges
            .iter()
            .filter(|e| e.kind == "Containment" && e.source == id)
            .map(|e| e.target)
            .collect::<Vec<_>>();
        let axis = usize::from(self.nodes[id]["kind"] == "TimePairInter");
        out.sort_by_key(|&i| {
            self.nodes[i]["bounds"][axis].as_i64().unwrap()
                + self.nodes[i]["bounds"][axis + 2].as_i64().unwrap() / 2
        });
        out
    }
    fn edge(
        &mut self,
        kind: &str,
        source: usize,
        target: usize,
        cause: Option<&str>,
    ) -> Result<(), String> {
        if self.edges.len() >= 65536 {
            return Err("unsupported_time_edge_budget".into());
        }
        self.edges.push(Edge {
            kind: kind.into(),
            source,
            target,
            cause: cause.map(str::to_owned),
        });
        if kind == "Containment" {
            self.refresh(source)?;
        }
        Ok(())
    }
    fn refresh(&mut self, id: usize) -> Result<(), String> {
        if !self.live(id) {
            return Ok(());
        }
        let members = self.members(id);
        let mut bounds = None;
        let mut sum = 0.;
        for &m in &members {
            let n = &self.nodes[m];
            sum += n["contextual_grade"]
                .as_f64()
                .unwrap_or(number(&n["grade"])?);
            let b = rect(&n["bounds"])?;
            bounds = Some(bounds.map(|a| union(a, b)).unwrap_or(b));
        }
        if self.nodes[id]["kind"] == "TimePairInter" {
            let grade = if members.len() == 2 { sum / 2. } else { 0. };
            self.nodes[id]["grade"] = json!(grade);
            if members.len() == 2 {
                let num = integer(&self.nodes[members[0]]["value"])?;
                let den = integer(&self.nodes[members[1]]["value"])?;
                self.nodes[id]["numerator"] = json!(num);
                self.nodes[id]["denominator"] = json!(den);
                self.nodes[id]["value"] = json!(format!("{num}/{den}"));
            } else {
                self.nodes[id]["numerator"] = Value::Null;
                self.nodes[id]["denominator"] = Value::Null;
                self.nodes[id]["value"] = Value::Null;
            }
        } else if self.nodes[id]["kind"] == "KeyInter" {
            self.nodes[id]["grade"] = if members.is_empty() {
                Value::Null
            } else {
                json!(sum / members.len() as f64)
            };
            self.nodes[id]["fifths"] = json!(
                members
                    .iter()
                    .map(|&m| if self.nodes[m]["shape"] == "FLAT" {
                        -1
                    } else {
                        1
                    })
                    .sum::<i64>()
            );
        }
        self.nodes[id]["bounds"] = json!(bounds);
        Ok(())
    }
    fn remove(&mut self, id: usize, extensive: bool) -> Result<(), String> {
        if !self.live(id) {
            return Ok(());
        }
        self.nodes[id]["removed"] = json!(true);
        let ensembles = self
            .edges
            .iter()
            .filter(|e| e.kind == "Containment" && e.target == id)
            .map(|e| e.source)
            .collect::<Vec<_>>();
        if extensive {
            for &parent in &ensembles {
                if self.members(parent).len() == 1 {
                    self.remove(parent, false)?;
                }
            }
            if self.ensemble(id) {
                for m in self.members(id) {
                    if !self
                        .edges
                        .iter()
                        .any(|e| e.kind == "Containment" && e.target == m && e.source != id)
                    {
                        self.remove(m, false)?;
                    }
                }
            }
        }
        self.edges.retain(|e| e.source != id && e.target != id);
        for parent in ensembles {
            self.refresh(parent)?;
        }
        Ok(())
    }
    fn snapshot(&self, id: usize) -> Value {
        let mut node = self.nodes[id].clone();
        if self.ensemble(id) {
            node["members"] = json!(
                self.members(id)
                    .into_iter()
                    .map(|m| self.snapshot(m))
                    .collect::<Vec<_>>()
            );
        }
        node
    }
    fn relations(&self) -> Value {
        json!(
            self.edges
                .iter()
                .map(|e| json!({"kind":e.kind,"source":e.source+1,"target":e.target+1,"cause":e.cause}))
                .collect::<Vec<_>>()
        )
    }
    fn partner_context(&mut self, id: usize) -> Result<(), String> {
        let mut partners = self
            .edges
            .iter()
            .filter(|e| e.kind == "TimeTopBottomRelation" && (e.source == id || e.target == id))
            .map(|e| if e.source == id { e.target } else { e.source })
            .collect::<Vec<_>>();
        partners.sort_by(|&a, &b| {
            number(&self.nodes[b]["grade"])
                .unwrap()
                .total_cmp(&number(&self.nodes[a]["grade"]).unwrap())
        });
        // Fresh automatic half relations have no same-side exclusion partners. Reject an expanded graph rather than skip partitions.
        for e in &self.edges {
            if e.kind == "Exclusion" && partners.contains(&e.source) && partners.contains(&e.target)
            {
                return Err("unsupported_time_partner_exclusion_partition".into());
            }
        }
        let mut contribution = 0.;
        for p in partners {
            contribution += 5. * number(&self.nodes[p]["grade"])?;
        }
        let grade = number(&self.nodes[id]["grade"])?;
        self.nodes[id]["contextual_grade"] =
            json!(((1. + contribution) * grade) / (1. + contribution * grade));
        Ok(())
    }
    fn overlaps(&self, a: usize, b: usize) -> Result<bool, String> {
        let na = &self.nodes[a];
        let nb = &self.nodes[b];
        if na["implicit"] == true || nb["implicit"] == true {
            return Ok(false);
        }
        if !intersects(rect(&na["bounds"])?, rect(&nb["bounds"])?) {
            return Ok(false);
        }
        if self.ensemble(a) {
            if self.members(a).contains(&b) {
                return Ok(false);
            }
            for m in self.members(a) {
                if self.overlaps(m, b)?
                    && self.overlaps(b, m)?
                    && !self.edges.iter().any(|e| {
                        e.kind != "Containment"
                            && e.kind != "Exclusion"
                            && ((e.source == m && e.target == b)
                                || (e.source == b && e.target == m))
                    })
                {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if !na["glyph"].is_null() && !nb["glyph"].is_null() {
            return glyph_intersection(&na["glyph"], &nb["glyph"]);
        }
        if matches!(
            na["kind"].as_str(),
            Some("BarConnectorInter" | "BracketConnectorInter" | "BarlineInter" | "BracketInter")
        ) {
            return Err("unsupported_time_neighbor_area_intersection".into());
        }
        if !na["glyph"].is_null() {
            return glyph_hits_rect(&na["glyph"], rect(&nb["bounds"])?);
        }
        if !nb["glyph"].is_null() {
            return glyph_hits_rect(&nb["glyph"], rect(&na["bounds"])?);
        }
        Ok(true)
    }
    fn pair(&mut self, num: usize, den: usize) -> Result<usize, String> {
        let id=self.add(json!({"kind":"TimePairInter","shape":null,"grade":null,"contextual_grade":null,"bounds":null,"glyph":null,"origin":"TIME","staff_id":self.nodes[num]["staff_id"]}))?;
        self.edge("Containment", id, num, None)?;
        self.edge("Containment", id, den, None)?;
        Ok(id)
    }
    fn time_value(&self, id: usize) -> Result<TimeValue, String> {
        let n = &self.nodes[id];
        if n["kind"] == "TimePairInter" {
            Ok(TimeValue {
                specific: None,
                num: integer(&n["numerator"])?,
                den: integer(&n["denominator"])?,
            })
        } else {
            TimeValue::whole(n["shape"].as_str().ok_or("missing_time_shape")?)
        }
    }
}
fn glyph_pixels(g: &Value) -> Result<BTreeSet<(i64, i64)>, String> {
    let b = rect(&g["bounds"])?;
    let vertical = match g["orientation"].as_str() {
        Some("VERTICAL") => true,
        Some("HORIZONTAL") => false,
        _ => return Err("unsupported_time_glyph_orientation".into()),
    };
    let runs = g["runs"]
        .as_array()
        .filter(|a| a.len() <= 65536)
        .ok_or("unsupported_time_glyph_runs")?;
    let mut out = BTreeSet::new();
    for r in runs {
        let p = integer(&r[0])?;
        let s = integer(&r[1])?;
        let n = integer(&r[2])?;
        if p < 0 || s < 0 || n <= 0 || n > 65536 || out.len() + n as usize > 1_048_576 {
            return Err("unsupported_time_glyph_budget".into());
        }
        for v in s..s + n {
            out.insert(if vertical {
                (b[0] + p, b[1] + v)
            } else {
                (b[0] + v, b[1] + p)
            });
        }
    }
    Ok(out)
}
fn glyph_intersection(a: &Value, b: &Value) -> Result<bool, String> {
    if !intersects(rect(&a["bounds"])?, rect(&b["bounds"])?) {
        return Ok(false);
    }
    let a = glyph_pixels(a)?;
    let b = glyph_pixels(b)?;
    Ok(a.iter().any(|p| b.contains(p)))
}
fn glyph_hits_rect(g: &Value, b: [i64; 4]) -> Result<bool, String> {
    Ok(glyph_pixels(g)?
        .iter()
        .any(|&(x, y)| x >= b[0] && x < b[0] + b[2] && y >= b[1] && y < b[1] + b[3]))
}

struct Builder {
    staff: StaffInput,
    prep: Preparation,
    range: Value,
    adapters: Vec<Value>,
    lists: [Vec<usize>; 3],
    selected: Option<usize>,
    time_stop: Option<i64>,
}
impl Builder {
    fn new(image: &GrayImage, il: usize, staff: StaffInput) -> Result<Self, String> {
        let prep = time_preparation::prepare_staff(image, il, &staff)?;
        let range = json!({"valid":false,"browse_start":prep.browse_start,"browse_stop":prep.browse_stop,"start":null,"stop":null});
        Ok(Self {
            staff,
            prep,
            range,
            adapters: vec![],
            lists: std::array::from_fn(|_| vec![]),
            selected: None,
            time_stop: None,
        })
    }
    fn find(&mut self, image: &GrayImage, il: usize, g: &mut Graph) -> Result<(), String> {
        let found = time_candidates::find_candidates(image, il, &self.staff)?;
        self.range["start"] = json!(self.prep.start);
        self.range["stop"] = json!(self.prep.stop);
        for (index, a) in found["adapters"]
            .as_array()
            .ok_or("invalid_time_adapters")?
            .iter()
            .enumerate()
        {
            if index >= 3 {
                return Err("unsupported_time_adapter_count".into());
            }
            let mut best = vec![];
            for c in a["best"]
                .as_array()
                .filter(|a| a.len() <= 11)
                .ok_or("unsupported_time_candidate_count")?
            {
                let shape = c["shape"].as_str().ok_or("invalid_time_shape")?;
                let value = if index == 0 {
                    let t = TimeValue::whole(shape)?;
                    json!({"kind":"TimeWholeInter","value":t.label(),"numerator":t.num,"denominator":t.den})
                } else {
                    json!({"kind":"TimeNumberInter","value":time_number(shape)?})
                };
                let mut node = c.clone();
                for (k, v) in value.as_object().unwrap() {
                    node[k] = v.clone();
                }
                node["contextual_grade"] = Value::Null;
                node["origin"] = json!("TIME");
                node["staff_id"] = json!(self.staff.staff_id);
                let id = g.add(node)?;
                self.lists[index].push(id);
                best.push(json!({"shape":shape,"inter_id":id+1}));
            }
            self.adapters.push(json!({"kind":a["kind"],"search_rect":a["rectangle"],"trials":a["trials"],"parts":a["parts"],"best_per_shape":best}));
        }
        Ok(())
    }
    fn filter(&mut self, g: &mut Graph) -> Result<bool, String> {
        for &top in &self.lists[1] {
            let b = rect(&g.nodes[top]["bounds"])?;
            let x = b[0] + b[2] / 2;
            for &bottom in &self.lists[2] {
                let b = rect(&g.nodes[bottom]["bounds"])?;
                if (b[0] + b[2] / 2 - x).abs() <= self.prep.parameters.max_halves_dx {
                    if supported(
                        integer(&g.nodes[top]["value"])?,
                        integer(&g.nodes[bottom]["value"])?,
                    ) {
                        g.edge("TimeTopBottomRelation", top, bottom, None)?;
                    } else {
                        g.edge(
                            "Exclusion",
                            top.min(bottom),
                            top.max(bottom),
                            Some("INCOMPATIBLE"),
                        )?;
                    }
                }
            }
        }
        for side in [1, 2] {
            let mut retained = vec![];
            for id in self.lists[side].clone() {
                if g.edges.iter().any(|e| {
                    e.kind == "TimeTopBottomRelation" && (e.source == id || e.target == id)
                }) {
                    retained.push(id);
                } else {
                    g.remove(id, true)?;
                }
            }
            self.lists[side] = retained;
        }
        for side in [1, 2] {
            for &id in &self.lists[side] {
                g.partner_context(id)?;
            }
        }
        let mut all = self.lists.iter().flatten().copied().collect::<Vec<_>>();
        let mut i = 0;
        while i < all.len() {
            let time = all[i];
            let grade = number(&g.nodes[time]["grade"])?;
            let bounds = rect(&g.nodes[time]["bounds"])?;
            let neighbors = (0..g.nodes.len())
                .filter(|&n| {
                    g.live(n)
                        && !g.ensemble(n)
                        && !all.contains(&n)
                        && rect(&g.nodes[n]["bounds"]).is_ok_and(|b| intersects(b, bounds))
                })
                .collect::<Vec<_>>();
            let mut deleted = false;
            for n in neighbors {
                if g.overlaps(n, time)? {
                    if number(&g.nodes[n]["grade"])? <= grade {
                        g.remove(n, true)?;
                    } else {
                        g.remove(time, true)?;
                        all.remove(i);
                        deleted = true;
                        break;
                    }
                }
            }
            if !deleted {
                i += 1;
            }
        }
        self.lists[0].retain(|id| all.contains(id));
        self.lists[1].retain(|id| all.contains(id)); // Original deliberately leaves dens unchanged here.
        Ok(!self.lists[0].is_empty() || !self.lists[1].is_empty())
    }
    fn cleanup(&self, g: &mut Graph) -> Result<(), String> {
        for a in &self.adapters {
            for b in a["best_per_shape"].as_array().unwrap() {
                g.remove(b["inter_id"].as_u64().unwrap() as usize - 1, true)?;
            }
        }
        Ok(())
    }
    fn select(&mut self, g: &mut Graph, id: usize, metrics: &[Value]) -> Result<(), String> {
        self.selected = Some(id);
        let symbol = if g.nodes[id]["kind"] == "TimePairInter" {
            let members = g.members(id);
            if members.len() != 2 {
                return Err("unsupported_time_partial_selected_pair".into());
            }
            let mut boxes = vec![];
            for m in members {
                boxes.push(time_registration::symbol_bounds(
                    g.nodes[m]["shape"].as_str().unwrap(),
                    self.staff.specific_interline,
                    rect(&g.nodes[m]["bounds"])?,
                    metrics,
                )?);
            }
            union(boxes[0], boxes[1])
        } else {
            time_registration::symbol_bounds(
                g.nodes[id]["shape"].as_str().unwrap(),
                self.staff.specific_interline,
                rect(&g.nodes[id]["bounds"])?,
                metrics,
            )?
        };
        let stop = symbol[0] + symbol[2];
        let bounds = rect(&g.nodes[id]["bounds"])?;
        self.time_stop = Some(bounds[0] + bounds[2] - 1);
        self.range["stop"] = json!(stop);
        self.range["valid"] = json!(true);
        if g.nodes[id]["kind"] == "TimePairInter" {
            for n in self.lists[0].clone() {
                g.remove(n, true)?;
            }
            let members = g.members(id);
            for (side, chosen) in [(1, members[0]), (2, members[1])] {
                for n in self.lists[side].clone() {
                    if n != chosen {
                        g.remove(n, true)?;
                    }
                }
            }
        } else {
            for n in self.lists.iter().flatten().copied().collect::<Vec<_>>() {
                if n != id {
                    g.remove(n, true)?;
                }
            }
        }
        Ok(())
    }
    fn snapshot(&self, g: &Graph) -> Value {
        let mut adapters = self.adapters.clone();
        for a in &mut adapters {
            for b in a["best_per_shape"].as_array_mut().unwrap() {
                let id = b["inter_id"].as_u64().unwrap() as usize - 1;
                b["inter"] = g.snapshot(id);
                b.as_object_mut().unwrap().remove("inter_id");
            }
        }
        let p = &self.prep.parameters;
        json!({"staff_id":self.staff.staff_id,"header_start":self.staff.header_start,"header_stop":self.staff.header_stop,"clef_stop":self.staff.clef_stop,"key_stop":null,"time_stop":self.time_stop,"header_time":self.selected.map(|id|g.snapshot(id)),"time_range":self.range,"builder":{"range":self.range,"roi":self.prep.roi,"params":{"maxEvalRank":p.max_eval_rank,"roiWidth":p.roi_width,"maxFirstSpaceWidth":p.max_first_space_width,"maxInnerSpace":p.max_inner_space,"yMargin":p.y_margin,"minTimeWidth":p.min_time_width,"maxTimeWidth":p.max_time_width,"maxHalvesDx":p.max_halves_dx,"minPartWeight":p.min_part_weight,"maxPartGap":p.max_part_gap,"minWholeTimeWeight":p.min_whole_time_weight,"minHalfTimeWeight":p.min_half_time_weight,"maxSpaceCumul":p.max_space_cumul},"projection":{"x_min":self.prep.roi[0],"x_max":self.prep.roi[0]+self.prep.roi[2]-1,"values":self.prep.projection},"spaces":self.prep.spaces.iter().map(|s|[s.start,s.stop]).collect::<Vec<_>>(),"adapters":adapters,"wholes":self.lists[0].iter().map(|&id|g.snapshot(id)).collect::<Vec<_>>(),"nums":self.lists[1].iter().map(|&id|g.snapshot(id)).collect::<Vec<_>>(),"dens":self.lists[2].iter().map(|&id|g.snapshot(id)).collect::<Vec<_>>(),"selected_time":self.selected.map(|id|g.snapshot(id))}})
    }
}
type ValueVectors = (Vec<TimeValue>, Vec<Vec<Option<usize>>>);
fn value_vectors(builders: &[Builder], g: &mut Graph) -> Result<ValueVectors, String> {
    let mut values: Vec<TimeValue> = vec![];
    let mut vectors: Vec<Vec<Option<usize>>> = vec![];
    for (index, b) in builders.iter().enumerate() {
        let mut candidates = b.lists[0].clone();
        for &num in &b.lists[1] {
            let mut relations = g
                .edges
                .iter()
                .filter(|e| e.kind == "TimeTopBottomRelation" && e.target == num)
                .map(|e| e.source)
                .collect::<Vec<_>>();
            relations.extend(
                g.edges
                    .iter()
                    .filter(|e| e.kind == "TimeTopBottomRelation" && e.source == num)
                    .map(|e| e.target),
            );
            for den in relations {
                candidates.push(g.pair(num, den)?);
            }
        }
        for id in candidates {
            let value = g.time_value(id)?;
            let vi = if let Some(i) = values.iter().position(|v| *v == value) {
                i
            } else {
                values.push(value);
                vectors.push(vec![None; builders.len()]);
                values.len() - 1
            };
            if vectors[vi][index].is_none_or(|old| {
                number(&g.nodes[id]["grade"]).unwrap() > number(&g.nodes[old]["grade"]).unwrap()
            }) {
                vectors[vi][index] = Some(id);
            }
        }
    }
    Ok((values, vectors))
}
fn consistent(
    builders: &mut [Builder],
    g: &mut Graph,
    metrics: &[Value],
) -> Result<Option<TimeValue>, String> {
    let (values, vectors) = value_vectors(builders, g)?;
    let mut eligible = vec![];
    let mut grades = vec![];
    let mut vi = vec![];
    for i in hash_order(&values)? {
        if vectors[i].iter().all(Option::is_some) {
            let mut sum = 0.;
            for &id in &vectors[i] {
                sum += number(&g.nodes[id.unwrap()]["grade"])?;
            }
            eligible.push(values[i].clone());
            grades.push(sum / builders.len() as f64);
            vi.push(i);
        }
    }
    let mut best = None;
    let mut best_grade = 0.;
    for i in hash_order(&eligible)? {
        if grades[i] > best_grade {
            best = Some(i);
            best_grade = grades[i];
        }
    }
    let Some(best) = best else {
        return Ok(None);
    };
    for (i, b) in builders.iter_mut().enumerate() {
        b.select(g, vectors[vi[best]][i].unwrap(), metrics)?;
    }
    Ok(Some(eligible[best].clone()))
}
fn discard_neighbors(builders: &[Builder], g: &mut Graph) -> Result<(), String> {
    let times = builders
        .iter()
        .filter_map(|b| b.selected)
        .collect::<Vec<_>>();
    if times.is_empty() {
        return Ok(());
    }
    let mut box_all = rect(&g.nodes[times[0]]["bounds"])?;
    let mut own = times.clone();
    for &id in &times {
        box_all = union(box_all, rect(&g.nodes[id]["bounds"])?);
        if g.nodes[id]["kind"] == "TimePairInter" {
            own.extend(g.members(id));
        }
    }
    let mut neighbors = (0..g.nodes.len())
        .filter(|&id| {
            g.live(id)
                && !g.ensemble(id)
                && !own.contains(&id)
                && rect(&g.nodes[id]["bounds"]).is_ok_and(|b| intersects(box_all, b))
        })
        .collect::<Vec<_>>();
    for time in times {
        let mut remaining = vec![];
        for n in neighbors {
            if g.overlaps(n, time)?
                && number(&g.nodes[n]["grade"])? < number(&g.nodes[time]["grade"])?
            {
                g.remove(n, true)?;
            } else {
                remaining.push(n);
            }
        }
        neighbors = remaining;
    }
    Ok(())
}
fn import_graph(grid: &Value, key: &Value, system_id: u64) -> Result<Graph, String> {
    let mut g = Graph::default();
    let bars = grid["bar_semantics"]["systems"]
        .as_array()
        .ok_or("missing_time_grid_systems")?
        .iter()
        .find(|s| s["id"] == system_id)
        .ok_or("missing_time_grid_system")?;
    let glyphs = grid["glyphs"]
        .as_array()
        .ok_or("missing_time_grid_glyphs")?;
    let mut ids = BTreeMap::new();
    for v in bars["inters"]
        .as_array()
        .filter(|a| a.len() <= 16384)
        .ok_or("unsupported_time_grid_inters")?
    {
        let mut node = v.clone();
        let old = v["id"].as_u64().ok_or("invalid_time_grid_id")?;
        if let Some(gid) = v["glyph_id"].as_u64() {
            let glyph = glyphs
                .iter()
                .find(|v| v["id"] == gid)
                .ok_or("missing_time_grid_glyph")?;
            node["glyph"] = json!({"bounds":[glyph["left"],glyph["top"],glyph["width"],glyph["height"]],"orientation":glyph["orientation"],"runs":glyph["runs"].as_array().ok_or("invalid_time_grid_runs")?.iter().map(|r|json!([r["position"],r["start"],r["length"]])).collect::<Vec<_>>()});
        } else {
            node["glyph"] = Value::Null;
        }
        node["origin"] = json!({"stage":"GRID","id":old});
        let id = g.add(node)?;
        if ids.insert(old, id).is_some() {
            return Err("duplicate_time_grid_id".into());
        }
    }
    for e in bars["relations"]
        .as_array()
        .ok_or("missing_time_grid_relations")?
    {
        let s = *ids
            .get(&e["source"].as_u64().ok_or("invalid_time_edge")?)
            .ok_or("missing_time_edge_endpoint")?;
        let t = *ids
            .get(&e["target"].as_u64().ok_or("invalid_time_edge")?)
            .ok_or("missing_time_edge_endpoint")?;
        g.edge(
            e["kind"].as_str().ok_or("invalid_time_edge_kind")?,
            s,
            t,
            e["cause"].as_str(),
        )?;
    }
    let header_prep = crate::header_preparation::from_grid(grid)?;
    let staff_rows = key["final_state"]["staffs"]
        .as_array()
        .ok_or("missing_time_key_staffs")?;
    for row in header_prep["staff_headers"]
        .as_array()
        .ok_or("missing_time_header_rows")?
    {
        if !row["dummy_barline"].is_null()
            && staff_rows.iter().any(|s| s["staff_id"] == row["staff_id"])
        {
            let mut dummy = row["dummy_barline"].clone();
            dummy["contextual_grade"] = Value::Null;
            dummy["glyph"] = Value::Null;
            dummy["origin"] = json!({"stage":"HEADER_PREPARATION","staff_id":row["staff_id"]});
            g.add(dummy)?;
        }
    }
    // All staff clefs exist before any KeyBuilder runs. Preserve this live insertion order.
    let mut staff_maps = BTreeMap::new();
    for row in staff_rows {
        let sid = row["staff_id"].as_u64().ok_or("invalid_time_key_staff")?;
        let mut map = BTreeMap::new();
        for clef in row["builder_state"]["remaining_sig_clefs"]
            .as_array()
            .ok_or("missing_time_clefs")?
        {
            let mut node = clef.clone();
            let old = node["id"].as_u64().ok_or("invalid_time_clef_id")?;
            node["staff_id"] = json!(sid);
            node["origin"] = json!({"stage":"KEY_COLUMN","staff_id":sid,"id":old});
            map.insert(old, g.add(node)?);
        }
        staff_maps.insert(sid, map);
    }
    for staff in key["final_state"]["staffs"]
        .as_array()
        .ok_or("missing_time_key_staffs")?
    {
        let sid = staff["staff_id"].as_u64().ok_or("invalid_time_key_staff")?;
        let state = &staff["builder_state"];
        let mut map = staff_maps
            .remove(&sid)
            .ok_or("missing_time_clef_namespace")?;
        let mut originals = vec![];
        for shape in state["shape_states"]
            .as_array()
            .ok_or("missing_time_key_shapes")?
        {
            for s in shape["slices"]
                .as_array()
                .ok_or("missing_time_key_slices")?
            {
                if !s["alter"].is_null() && s["alter"]["removed"] != true {
                    originals.push(s["alter"].clone());
                }
            }
            let k = &shape["key_inter"];
            if !k.is_null() && k["removed"] != true {
                for member in k["members"].as_array().ok_or("missing_time_key_members")? {
                    originals.push(member.clone());
                }
                originals.push(k.clone());
            }
        }
        originals.sort_by_key(|v| v["id"].as_u64().unwrap());
        for mut node in originals {
            let old = node["id"].as_u64().ok_or("invalid_time_key_id")?;
            if map.contains_key(&old) {
                continue;
            }
            node["staff_id"] = json!(sid);
            node["origin"] = json!({"stage":"KEY_COLUMN","staff_id":sid,"id":old});
            if let Some(m) = node.as_object_mut() {
                m.remove("members");
            }
            let id = g.add(node)?;
            map.insert(old, id);
        }
        for e in state["relations"]
            .as_array()
            .ok_or("missing_time_key_relations")?
        {
            let s = *map
                .get(&e["source"].as_u64().ok_or("invalid_time_key_edge")?)
                .ok_or("missing_time_key_edge_endpoint")?;
            let t = *map
                .get(&e["target"].as_u64().ok_or("invalid_time_key_edge")?)
                .ok_or("missing_time_key_edge_endpoint")?;
            g.edge(
                e["kind"].as_str().ok_or("invalid_time_key_edge_kind")?,
                s,
                t,
                e["cause"].as_str(),
            )?;
        }
    }
    Ok(g)
}
fn snapshot(
    builders: &[Builder],
    g: &Graph,
    time: &Option<TimeValue>,
    key: &Value,
) -> Result<Value, String> {
    let mut staffs = vec![];
    for b in builders {
        let mut row = b.snapshot(g);
        let previous = key["final_state"]["staffs"]
            .as_array()
            .ok_or("missing_time_key_staffs")?
            .iter()
            .find(|r| r["staff_id"] == b.staff.staff_id)
            .ok_or("missing_time_key_staff")?;
        row["key_stop"] = previous["key_stop"].clone();
        row["key_range"] = previous["key_range"].clone();
        row["header_clef"] = Value::Null;
        row["header_key"] = if let Some(id) = previous["header_key"]["id"].as_u64() {
            let current = g
                .nodes
                .iter()
                .position(|n| {
                    n["origin"]["stage"] == "KEY_COLUMN"
                        && n["origin"]["staff_id"] == b.staff.staff_id
                        && n["origin"]["id"] == id
                })
                .ok_or("missing_time_header_key_identity")?;
            g.snapshot(current)
        } else {
            Value::Null
        };
        staffs.push(row);
    }
    Ok(
        json!({"time_value":time.as_ref().map(TimeValue::label),"staffs":staffs,"sig_inters":(0..g.nodes.len()).filter(|&id|g.live(id)).map(|id|g.snapshot(id)).collect::<Vec<_>>(),"relations":g.relations(),"removed_existing_inters":g.nodes.iter().filter(|n|n["removed"]==true&&n["origin"]!="TIME").map(|n|n["origin"].clone()).collect::<Vec<_>>()}),
    )
}
/// Complete bounded fresh-header time column. Native key output supplies mutable SIG reconstruction.
/// Stops before final HeaderBuilder clef selection/interior-bar purge/header freeze.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    key_column: &Value,
    il: usize,
) -> Result<Value, String> {
    let inputs = time_preparation::after_key_staffs(grid, registration, key_column, il)?;
    let metrics = time_registration::builtin_metrics()?;
    let mut output = vec![];
    for key in key_column["systems"]
        .as_array()
        .filter(|s| s.len() <= 1024)
        .ok_or("unsupported_time_system_count")?
    {
        let system_id = key["system_id"].as_u64().ok_or("invalid_time_system_id")?;
        let ids = key["final_state"]["staffs"]
            .as_array()
            .filter(|a| !a.is_empty() && a.len() <= 1024)
            .ok_or("unsupported_time_staff_count")?;
        let mut builders = vec![];
        for row in ids {
            let sid = row["staff_id"].as_u64().ok_or("invalid_time_staff_id")?;
            let input = inputs
                .iter()
                .find(|s| s.staff_id == sid)
                .ok_or("missing_time_staff_input")?
                .clone();
            builders.push(Builder::new(image, il, input)?);
        }
        builders.sort_by_key(|b| b.staff.staff_id);
        let mut g = import_graph(grid, key, system_id)?;
        let mut time = None;
        let mut stages = vec![];
        let mut failed = false;
        for i in 0..builders.len() {
            builders[i].find(image, il, &mut g)?;
            stages.push(json!({"phase":"before_filter","staff_id":builders[i].staff.staff_id,"state":snapshot(&builders,&g,&time,key)?}));
            let good = builders[i].filter(&mut g)?;
            stages.push(json!({"phase":"after_filter","staff_id":builders[i].staff.staff_id,"accepted":good,"state":snapshot(&builders,&g,&time,key)?}));
            if !good {
                for b in &builders {
                    b.cleanup(&mut g)?;
                }
                failed = true;
                break;
            }
        }
        if !failed {
            time = consistent(&mut builders, &mut g, &metrics)?;
            if time.is_some() {
                discard_neighbors(&builders, &mut g)?;
            }
        }
        let returned = if time.is_some() {
            builders
                .iter()
                .filter_map(|b| b.time_stop.map(|stop| stop - b.staff.header_start))
                .max()
                .unwrap_or(0)
                .max(0)
        } else {
            -1
        };
        let after = snapshot(&builders, &g, &time, key)?;
        for b in &mut builders {
            if returned > 0 {
                b.staff.header_stop = b.staff.header_start + returned;
            }
        }
        let propagated = snapshot(&builders, &g, &time, key)?;
        output.push(json!({"system_id":system_id,"clef_offset":key["clef_column_offset"],"key_offset":key["returned_key_column_offset"],"returned_time_offset":returned,"after_retrieve_time":after,"after_system_time_stop":propagated,"stages":stages}));
    }
    Ok(
        json!({"stage":"HEADERS.time_column","systems":output,"time_column_complete":true,"headers_complete":false,"pending":["final clef selection","interior header bar purge","header freezing"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields(a: &Value, b: &Value, path: &str) {
        match b {
            Value::Object(m) => {
                for (k, v) in m {
                    fields(&a[k], v, &format!("{path}.{k}"));
                }
            }
            Value::Array(v) => {
                assert_eq!(a.as_array().unwrap().len(), v.len(), "{path}: {a} != {b}");
                for (i, (x, y)) in a.as_array().unwrap().iter().zip(v).enumerate() {
                    fields(x, y, &format!("{path}[{i}]"));
                }
            }
            Value::Number(_) => {
                if b.as_i64().is_some() {
                    assert_eq!(a, b, "{path}");
                } else {
                    let x = a.as_f64().unwrap();
                    let y = b.as_f64().unwrap();
                    assert!((x - y).abs() <= 1e-14, "{path}: {x:?} != {y:?}");
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    fn fixture_builders(case: &Value) -> (Vec<Builder>, Graph, usize) {
        let image = GrayImage::new(520, 420, vec![255; 520 * 420]).unwrap();
        let mut builders = vec![];
        for i in 0..case["staff_count"].as_u64().unwrap() {
            let top = 121. + 130. * i as f64;
            let staff = StaffInput {
                staff_id: i + 1,
                header_start: 102,
                header_stop: 158,
                clef_stop: None,
                specific_interline: 16,
                barlines: vec![],
                lines: (0..5)
                    .map(|j| vec![[102., top + j as f64 * 16.], [495., top + j as f64 * 16.]])
                    .collect(),
            };
            builders.push(Builder::new(&image, 16, staff).unwrap());
        }
        let mut g = Graph::default();
        for input in case["inputs"].as_array().unwrap() {
            let side = input["kind"].as_str().unwrap();
            let shape = input["shape"].as_str().unwrap();
            let mut node = json!({"label":input["label"],"shape":shape,"grade":input["grade"],"bounds":input["bounds"],"contextual_grade":null,"glyph":null,"origin":"TIME","staff_id":input["staff_index"].as_u64().unwrap()+1});
            let list = match side {
                "WHOLE" => {
                    let tv = TimeValue::whole(shape).unwrap();
                    node["kind"] = json!("TimeWholeInter");
                    node["value"] = json!(tv.label());
                    node["numerator"] = json!(tv.num);
                    node["denominator"] = json!(tv.den);
                    Some(0)
                }
                "NUM" | "DEN" => {
                    node["kind"] = json!("TimeNumberInter");
                    node["value"] = json!(time_number(shape).unwrap());
                    Some(if side == "NUM" { 1 } else { 2 })
                }
                "NEIGHBOR" => {
                    node["kind"] = json!("KeyAlterInter");
                    None
                }
                _ => panic!("unsupported fixture kind"),
            };
            let id = g.add(node).unwrap();
            if let Some(list) = list {
                builders[input["staff_index"].as_u64().unwrap() as usize].lists[list].push(id);
            }
        }
        let base = g.nodes.len();
        (builders, g, base)
    }
    fn label(g: &Graph, id: usize, base: usize) -> String {
        g.nodes[id]["label"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("generated_{}", id + 1 - base))
    }
    fn control_inter(g: &Graph, id: usize, base: usize) -> Value {
        let n = &g.nodes[id];
        let mut out = json!({"label":label(g,id,base),"kind":n["kind"],"shape":n["shape"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"removed":n["removed"],"bounds":n["bounds"]});
        if n["kind"] == "TimeNumberInter" {
            out["value"] = n["value"].clone();
        }
        if n["kind"] == "TimePairInter" || n["kind"] == "TimeWholeInter" {
            out["time_value"] = n["value"].clone();
        }
        if n["kind"] == "TimePairInter" {
            out["members"] = json!(
                g.members(id)
                    .into_iter()
                    .map(|m| label(g, m, base))
                    .collect::<Vec<_>>()
            );
        }
        out
    }
    fn control_snapshot(builders: &[Builder], g: &Graph, base: usize) -> Value {
        json!({"inters":(0..g.nodes.len()).filter(|&id|g.live(id)).map(|id|control_inter(g,id,base)).collect::<Vec<_>>(),"relations":g.edges.iter().map(|e|json!({"kind":e.kind,"source":label(g,e.source,base),"target":label(g,e.target,base),"cause":e.cause})).collect::<Vec<_>>(),"staffs":builders.iter().enumerate().map(|(i,b)|json!({"staff_index":i,"selected":b.selected.map(|id|control_inter(g,id,base)),"time_stop":b.time_stop,"wholes":b.lists[0].iter().map(|&id|label(g,id,base)).collect::<Vec<_>>(),"nums":b.lists[1].iter().map(|&id|label(g,id,base)).collect::<Vec<_>>(),"dens":b.lists[2].iter().map(|&id|label(g,id,base)).collect::<Vec<_>>()})).collect::<Vec<_>>()})
    }
    #[test]
    fn independent_original_filter_vector_and_selection_controls() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-grid/time-selection-controls.json"
        ))
        .unwrap();
        let cases = oracle["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 37);
        let metrics = time_registration::builtin_metrics().unwrap();
        for case in cases {
            for selection in [false, true] {
                let phase = if selection {
                    "selection_pass"
                } else {
                    "vector_pass"
                };
                let expected = &case[phase];
                let path = format!("{}.{}", case["name"], phase);
                let (mut builders, mut g, base) = fixture_builders(case);
                let results = builders
                    .iter_mut()
                    .map(|b| b.filter(&mut g).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(json!(results), expected["filter_results"], "{path}");
                fields(
                    &control_snapshot(&builders, &g, base),
                    &expected["after_filter"],
                    &format!("{path}.after_filter"),
                );
                fields(
                    &json!(
                        (0..base)
                            .map(|id| control_inter(&g, id, base))
                            .collect::<Vec<_>>()
                    ),
                    &expected["input_inter_states_after_filter"],
                    &format!("{path}.input_states"),
                );
                if selection {
                    let time = consistent(&mut builders, &mut g, &metrics).unwrap();
                    assert_eq!(json!(time.is_some()), expected["returned_bool"], "{path}");
                    assert_eq!(
                        json!(time.map(|t| t.label())),
                        expected["time_value"],
                        "{path}"
                    );
                } else {
                    let (values, vectors) = value_vectors(&builders, &mut g).unwrap();
                    let rows=hash_order(&values).unwrap().into_iter().map(|i|json!({"time_value":values[i].label(),"candidates_by_staff":vectors[i].iter().map(|id|id.map(|id|control_inter(&g,id,base))).collect::<Vec<_>>() })).collect::<Vec<_>>();
                    fields(
                        &json!(rows),
                        &expected["values_in_original_hashmap_iteration_order"],
                        &format!("{path}.vectors"),
                    );
                }
                fields(
                    &control_snapshot(&builders, &g, base),
                    &expected["after"],
                    &format!("{path}.after"),
                );
            }
        }
    }
}
