// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{beam_structure::intersection_at_x, curves::Curve};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreviousLedger {
    pub id: u64,
    pub index: i32,
    pub bounds: [i64; 4],
    pub center_line: [f64; 4],
    pub start: [f64; 2],
    pub stop: [f64; 2],
}
#[derive(Clone, Debug, Serialize)]
pub struct Reference {
    pub y: f64,
    pub previous_id: Option<u64>,
}

/// Original getYReference: signed adjacent index, strict overlap, first encounter.
pub fn select(
    first: &Curve,
    last: &Curve,
    index: i32,
    bounds: [i64; 4],
    main_interline: usize,
    ledgers: &[PreviousLedger],
) -> Result<Option<Reference>, String> {
    if index == 0
        || index.unsigned_abs() > 1024
        || !(1..=128).contains(&main_interline)
        || bounds[2] <= 0
        || bounds[3] <= 0
        || bounds.iter().any(|x| x.unsigned_abs() > 4_000_000)
        || ledgers.len() > 32768
    {
        return Err("invalid_ledger_reference_input".into());
    }
    let x = bounds[0] as f64 + bounds[2] as f64 / 2.;
    let previous = index - index.signum();
    if previous == 0 {
        return Ok(Some(Reference {
            y: if index < 0 { first.y(x) } else { last.y(x) },
            previous_id: None,
        }));
    }
    let minimum = (0.75 * main_interline as f64).round_ties_even() as i64;
    for ledger in ledgers.iter().filter(|l| l.index == previous) {
        if ledger.bounds[2] <= 0
            || ledger.bounds[3] <= 0
            || ledger.bounds.iter().any(|v| v.unsigned_abs() > 4_000_000)
            || ledger
                .center_line
                .iter()
                .chain(&ledger.start)
                .chain(&ledger.stop)
                .any(|v| !v.is_finite())
        {
            return Err("invalid_previous_ledger_reference".into());
        }
        let b = ledger.bounds;
        let overlap = (bounds[0] + bounds[2]).min(b[0] + b[2]) - bounds[0].max(b[0]);
        if overlap > minimum {
            let line = if x >= b[0] as f64 && x < (b[0] + b[2]) as f64 {
                ledger.center_line
            } else {
                [
                    ledger.start[0],
                    ledger.start[1],
                    ledger.stop[0],
                    ledger.stop[1],
                ]
            };
            let y = intersection_at_x(line, x)[1];
            if !y.is_finite() {
                return Err("unsupported_ledger_reference_line".into());
            }
            return Ok(Some(Reference {
                y,
                previous_id: Some(ledger.id),
            }));
        }
    }
    Ok(None)
}
