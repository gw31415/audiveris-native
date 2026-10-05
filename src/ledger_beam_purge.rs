// SPDX-License-Identifier: AGPL-3.0-or-later
//! LedgersFilter area purge and LedgersBuilder good full beam candidate prefix.
use crate::{
    GrayImage, beam_candidates::SystemCandidates, ledger_candidates::GoodBeamArea,
    vertical_sticks::SeedSection,
};
use audiveris_area::LinearArea;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LedgerBeam {
    pub id: u64,
    pub kind: String,
    pub bounds: [i64; 4],
    pub median: [f64; 4],
    pub height: f64,
    pub grade: f64,
    pub contextual_grade: Option<f64>,
    pub removed: bool,
}
impl LedgerBeam {
    fn area(&self) -> Result<LinearArea, String> {
        let [x1, y1, x2, y2] = self.median;
        if !matches!(
            self.kind.as_str(),
            "BeamInter" | "BeamHookInter" | "SmallBeamInter"
        ) || !self.grade.is_finite()
            || self.contextual_grade.is_some_and(|g| !g.is_finite())
            || self.bounds.iter().any(|v| v.unsigned_abs() > 4_000_000)
            || self.bounds[2] <= 0
            || self.bounds[3] <= 0
            || self.median.iter().any(|v| !v.is_finite())
            || x1 >= x2
            || !self.height.is_finite()
            || self.height <= 0.
        {
            return Err("unsupported_ledger_beam".into());
        }
        let h = self.height / 2.;
        LinearArea::polygon(&[[x1, y1 - h], [x2, y2 - h], [x2, y2 + h], [x1, y1 + h]])
    }
}
pub fn from_native(system: &SystemCandidates) -> Vec<LedgerBeam> {
    system
        .inters()
        .iter()
        .map(|b| LedgerBeam {
            id: b.id() as u64,
            kind: b.kind.into(),
            bounds: b.bounds.map(i64::from),
            median: b.item.median,
            height: b.item.height,
            grade: b.impacts.grade,
            contextual_grade: None,
            removed: b.is_removed(),
        })
        .collect()
}
/// Original Rectangle.intersects before Area.intersects, preserving section order.
pub fn filter_sections(
    sections: &[SeedSection],
    beams: &[LedgerBeam],
) -> Result<Vec<SeedSection>, String> {
    if sections.len() > crate::sections::MAX_RUNS
        || beams.len() > 32768
        || sections.len().saturating_mul(beams.len()) > 8_000_000
    {
        return Err("ledger_beam_filter_budget".into());
    }
    let areas = beams
        .iter()
        .filter(|b| !b.removed)
        .map(|b| Ok((b.bounds, b.area()?)))
        .collect::<Result<Vec<_>, String>>()?;
    let mut output = Vec::new();
    for section in sections {
        if section.vertical
            || section.runs.is_empty()
            || section
                .runs
                .iter()
                .any(|r| r[1] <= 0 || r.iter().any(|v| v.unsigned_abs() > 4_000_000))
            || section.first_pos.unsigned_abs() > 4_000_000
        {
            return Err("invalid_ledger_beam_section".into());
        }
        let a = section.bounds();
        let mut discard = false;
        for (b, area) in &areas {
            if a[0] < b[0] + b[2]
                && b[0] < a[0] + a[2]
                && a[1] < b[1] + b[3]
                && b[1] < a[1] + a[3]
                && area.intersects_rect(a.map(|v| v as f64))?
            {
                discard = true;
                break;
            }
        }
        if !discard {
            output.push(section.clone());
        }
    }
    Ok(output)
}
pub fn good_full_beams(beams: &[LedgerBeam]) -> Result<Vec<GoodBeamArea>, String> {
    if beams.len() > 32768 {
        return Err("ledger_good_beam_budget".into());
    }
    let mut selected = Vec::new();
    for b in beams.iter().filter(|b| !b.removed) {
        b.area()?;
        if b.kind == "BeamInter" && b.grade >= 0.35 {
            let [x1, y1, x2, y2] = b.median;
            let h = b.height / 2.;
            let p = [[x1, y1 - h], [x1, y1 + h], [x2, y2 + h], [x2, y2 - h]];
            selected.push(GoodBeamArea {
                bounds: b.bounds,
                segments: (0..4).map(|i| vec![p[i], p[(i + 1) % 4]]).collect(),
            });
        }
    }
    selected.sort_by_key(|b| b.bounds[0]);
    Ok(selected)
}
/// Native prerequisite chain through candidates; acceptance/registration are pending.
pub fn candidate_prefix(
    image: &GrayImage,
    grid: &Value,
    interline: usize,
    fore: usize,
    system_beams: &[(u64, Vec<LedgerBeam>)],
) -> Result<Value, String> {
    let input = crate::ledger_inputs::staff_distance_sections(image, grid, interline)?;
    let mut systems = Vec::new();
    for row in input["systems"].as_array().unwrap() {
        let id = row["system_id"].as_u64().unwrap();
        let matching = system_beams
            .iter()
            .filter(|b| b.0 == id)
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err("missing_or_duplicate_ledger_system_beams".into());
        }
        let sections: Vec<SeedSection> = serde_json::from_value(row["sections"].clone())
            .map_err(|_| "invalid_ledger_input_sections")?;
        let filtered = filter_sections(&sections, &matching[0].1)?;
        let good = good_full_beams(&matching[0].1)?;
        let candidates =
            crate::ledger_candidates::retrieve_candidates(&filtered, interline, fore, &good)?;
        systems.push(json!({"system_id":id,"sections":filtered,"candidates":candidates}));
    }
    Ok(
        json!({"stage":"LEDGERS.native_candidate_prefix","beam_section_filter_complete":true,"candidate_prefix_complete":true,"ledgers_complete":false,"systems":systems}),
    )
}
