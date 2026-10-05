// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original LedgersBuilder.getCandidateFilaments prefix; no ledger acceptance.
use crate::vertical_sticks::{SeedSection, retrieve_oriented};
use audiveris_area::Area;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Already-selected good full beams in original Inters.byAbscissa order.
/// Selection is a caller prerequisite; this prefix does not infer beam quality.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoodBeamArea {
    pub bounds: [i64; 4],
    pub segments: Vec<Vec<[f64; 2]>>,
}

pub fn retrieve_candidates(
    sections: &[SeedSection],
    interline: usize,
    fore: usize,
    good_beams: &[GoodBeamArea],
) -> Result<Vec<Value>, String> {
    if !(3..=128).contains(&interline) || !(1..=128).contains(&fore) || good_beams.len() > 32768 {
        return Err("unsupported_ledger_candidate_scale_or_beams".into());
    }
    let thickness = (3.25 * fore as f64)
        .round_ties_even()
        .min((0.4 * interline as f64).round_ties_even()) as i64;
    let mut areas = Vec::new();
    let mut previous = i64::MIN;
    for beam in good_beams {
        if beam.bounds[0] < previous || beam.bounds[2] <= 0 || beam.bounds[3] <= 0 {
            return Err("invalid_ledger_good_beam_order_or_bounds".into());
        }
        previous = beam.bounds[0];
        areas.push((beam.bounds[0], Area::from_closed_segments(&beam.segments)?));
    }
    let sticks = retrieve_oriented(sections, &[], thickness, interline as i64, false, 0.9)?;
    let mut kept = Vec::new();
    for stick in sticks {
        let midpoint = [
            (stick["start"][0].as_f64().unwrap() + stick["stop"][0].as_f64().unwrap()) / 2.,
            (stick["start"][1].as_f64().unwrap() + stick["stop"][1].as_f64().unwrap()) / 2.,
        ];
        let mut overlap = false;
        for (left, area) in &areas {
            if area.contains(midpoint)? {
                overlap = true;
                break;
            }
            if *left as f64 > midpoint[0] {
                break;
            }
        }
        if !overlap && stick["bounds"][2].as_i64().unwrap() <= 20 * interline as i64 {
            kept.push(stick);
        }
    }
    Ok(kept)
}
