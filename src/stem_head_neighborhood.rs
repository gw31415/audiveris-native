// SPDX-License-Identifier: AGPL-3.0-or-later
//! Page-state beam neighborhoods and initial-area candidate selection.
//! Head corner references and complete beam/group state are supplied, not
//! discovered from a raster. No physical registry/cache/SIG or epilog writes.
use crate::{
    stem_head_targets::{Beam, Catalog, Group, Request as TargetRequest, Selected},
    stem_lookup_area,
};
use audiveris_area::LinearArea;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const REQUEST_SCHEMA: &str = "audiveris-native.stem-head-plan-request.v1";
pub const MAX_REQUEST_BYTES: usize = 16_000_000;
pub const MAX_REPORT_BYTES: usize = 32_000_000;
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub id: usize,
    pub bounds: [i32; 4],
    pub removed: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub head_bounds: [i32; 4],
    pub head: stem_lookup_area::HeadRequest,
    pub min_distance: i32,
    pub small_head: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub system: u64,
    pub system_bounds: [i32; 4],
    pub vicinity_margin: i32,
    pub beam_ids: Vec<usize>,
    pub beams: Vec<Beam>,
    pub beam_bounds: Vec<Bounds>,
    pub groups: Vec<Group>,
    pub cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub systems: Vec<System>,
}
#[derive(Serialize)]
pub struct Planned {
    pub neighbors: Vec<usize>,
    pub candidates: Vec<usize>,
    pub target: Selected,
}
#[derive(Serialize)]
pub struct PlannedSystem {
    pub system: u64,
    pub cases: Vec<Planned>,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub input_context: &'static str,
    pub systems: Vec<PlannedSystem>,
    pub automatic_raster_pipeline_complete: bool,
    pub page_cache_sig_complete: bool,
    pub epilog_complete: bool,
    pub heads_complete: bool,
    pub stems_complete: bool,
}
fn rectangle(r: [i32; 4]) -> Result<(), String> {
    if r[..2].iter().any(|v| i64::from(*v).abs() > 4_000_000)
        || r[2..].iter().any(|v| !(0..=4_000_000).contains(v))
    {
        return Err("unsupported neighborhood rectangle".into());
    }
    Ok(())
}
pub struct Neighborhood<'a> {
    bounds: BTreeMap<usize, &'a Bounds>,
}
impl<'a> Neighborhood<'a> {
    pub fn new(bounds: &'a [Bounds]) -> Result<Self, String> {
        if bounds.len() > 4096 {
            return Err("neighborhood catalog budget".into());
        }
        let mut map = BTreeMap::new();
        for b in bounds {
            rectangle(b.bounds)?;
            if b.id == 0 || map.insert(b.id, b).is_some() {
                return Err("invalid neighborhood identity".into());
            }
        }
        Ok(Self { bounds: map })
    }
    fn validate_ids(&self, ids: &[usize]) -> Result<(), String> {
        if ids.len() > 4096 {
            return Err("neighborhood beam budget".into());
        }
        for id in ids {
            if !self.bounds.contains_key(id) {
                return Err("unknown neighborhood beam".into());
            }
        }
        Ok(())
    }
    pub fn sort(&self, ids: &[usize]) -> Result<Vec<usize>, String> {
        self.validate_ids(ids)?;
        let mut ids = ids.to_vec();
        ids.sort_by_key(|id| self.bounds[id].bounds[0]);
        Ok(ids)
    }
    /// Original BY_ABSCISSA scan, including removed skips and early termination.
    /// The page planner sorts first; direct callers retain their declared order.
    pub fn neighbors(
        &self,
        ids: &[usize],
        head: [i32; 4],
        system: [i32; 4],
        margin: i32,
    ) -> Result<Vec<usize>, String> {
        self.validate_ids(ids)?;
        rectangle(head)?;
        rectangle(system)?;
        if !(0..=1024).contains(&margin) || system[3] == 0 {
            return Err("unsupported neighborhood system/margin".into());
        }
        let area = [
            i64::from(head[0]) - i64::from(margin),
            i64::from(system[1]),
            i64::from(head[2]) + 2 * i64::from(margin),
            i64::from(system[3]),
        ];
        let max_x = area[0] + area[2] - 1;
        let mut selected = Vec::new();
        for id in ids {
            let b = self.bounds[id];
            if b.removed {
                continue;
            }
            let [x, y, w, h] = b.bounds.map(i64::from);
            let overlaps = area[2] > 0
                && area[3] > 0
                && w > 0
                && h > 0
                && x + w > area[0]
                && x < area[0] + area[2]
                && y + h > area[1]
                && y < area[1] + area[3];
            if overlaps {
                selected.push(*id);
            } else if x > max_x {
                break;
            }
        }
        Ok(selected)
    }
    /// Positive Area/rectangle overlap, not median intersection. Bounds and
    /// early-break limits use original integer Area.getBounds rounding.
    pub fn initial_candidates(
        &self,
        ids: &[usize],
        polygon: &[[f64; 2]],
    ) -> Result<Vec<usize>, String> {
        self.validate_ids(ids)?;
        if polygon.is_empty() {
            return Ok(vec![]);
        }
        if polygon.len() > 128 {
            return Err("neighborhood polygon budget".into());
        }
        let area = LinearArea::polygon(polygon)?;
        let b = area.bounds();
        let max_x = f64::from(b[0]) + f64::from(b[2]);
        let mut selected = Vec::new();
        for id in ids {
            let b = self.bounds[id];
            if b.removed {
                continue;
            }
            if area.intersects_rect(b.bounds.map(f64::from))? {
                selected.push(*id);
            } else if f64::from(b.bounds[0]) > max_x {
                break;
            }
        }
        Ok(selected)
    }
}
pub fn evaluate(r: &Request) -> Result<Report, String> {
    if r.schema != REQUEST_SCHEMA || r.systems.len() > 64 {
        return Err("unsupported head planner request".into());
    }
    let mut seen = BTreeSet::new();
    let mut total_beams = 0usize;
    let mut cases = 0usize;
    let mut work = 0usize;
    let mut systems = Vec::new();
    for s in &r.systems {
        if s.system == 0 || !seen.insert(s.system) {
            return Err("invalid head planner system identity".into());
        }
        total_beams = total_beams
            .checked_add(s.beams.len())
            .filter(|v| *v <= 16_384)
            .ok_or("head planner catalog budget")?;
        cases = cases
            .checked_add(s.cases.len())
            .filter(|v| *v <= 8192)
            .ok_or("head planner case budget")?;
        work = work
            .checked_add(
                s.cases
                    .len()
                    .checked_mul(s.beams.len().max(s.beam_ids.len()))
                    .ok_or("head planner work overflow")?,
            )
            .filter(|v| *v <= 20_000_000)
            .ok_or("head planner work budget")?;
        rectangle(s.system_bounds)?;
        if s.system_bounds[3] == 0 || !(0..=1024).contains(&s.vicinity_margin) {
            return Err("unsupported neighborhood system/margin".into());
        }
        let catalog = Catalog::new(&s.beams, &s.groups)?;
        let neighborhood = Neighborhood::new(&s.beam_bounds)?;
        if neighborhood.bounds.keys().copied().collect::<BTreeSet<_>>()
            != s.beams.iter().map(|b| b.id).collect()
        {
            return Err("head planner bounds/catalog mismatch".into());
        }
        let ids = neighborhood.sort(&s.beam_ids)?;
        let mut planned = Vec::new();
        for c in &s.cases {
            let initial = stem_lookup_area::head(&c.head)?;
            let neighbors =
                neighborhood.neighbors(&ids, c.head_bounds, s.system_bounds, s.vicinity_margin)?;
            let candidates = neighborhood.initial_candidates(&neighbors, &initial.polygon)?;
            let target = catalog.select(&TargetRequest {
                head: c.head.clone(),
                min_distance: c.min_distance,
                small_head: c.small_head,
                candidates: candidates.clone(),
            })?;
            planned.push(Planned {
                neighbors,
                candidates,
                target,
            });
        }
        systems.push(PlannedSystem {
            system: s.system,
            cases: planned,
        });
    }
    Ok(Report {
        schema: "audiveris-native.stem-head-plan-report.v1",
        input_context: "caller_supplied_page_head_corner_references_and_complete_beam_group_state",
        systems,
        automatic_raster_pipeline_complete: false,
        page_cache_sig_complete: false,
        epilog_complete: false,
        heads_complete: false,
        stems_complete: false,
    })
}
pub fn encode_report(r: &Report) -> Result<Vec<u8>, String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if b.len() > MAX_REPORT_BYTES - self.0.len() {
                return Err(std::io::Error::other("head planner report budget"));
            }
            self.0.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut b = Bounded(Vec::new());
    serde_json::to_writer(&mut b, r).map_err(|e| e.to_string())?;
    if b.0.len() == MAX_REPORT_BYTES {
        return Err("head planner report budget".into());
    }
    b.0.push(b'\n');
    Ok(b.0)
}
