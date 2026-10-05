// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original LedgersFilter staff-distance run filter, Shift(0) lag and dispatch.
//! The later AbstractBeamInter area purge remains a separate prerequisite.
use crate::{
    GrayImage,
    curves::Curve,
    sections::{JunctionPolicy, Orientation, RunTable},
    vertical_sticks::SeedSection,
};
use serde_json::{Value, json};

pub fn staff_distance_sections(
    image: &GrayImage,
    grid: &Value,
    main_interline: usize,
) -> Result<Value, String> {
    if image.width.checked_mul(image.height) != Some(image.pixels.len()) {
        return Err("invalid_ledger_input_image".into());
    }
    let layout =
        crate::system_area_geometry::build(grid, image.width, image.height, main_interline)?;
    let rows = grid["staffs"].as_array().ok_or("invalid_ledger_staffs")?;
    let mut staffs = Vec::new();
    for row in rows {
        let lines = row["lines"]
            .as_array()
            .filter(|l| l.len() == 5)
            .ok_or("invalid_ledger_staff_lines")?;
        let curve = |n: usize| -> Result<Curve, String> {
            let points = serde_json::from_value(lines[n]["points"].clone())
                .map_err(|_| "invalid_ledger_staff_points")?;
            Curve::interpolate(points).ok_or_else(|| "invalid_ledger_staff_curve".into())
        };
        let il = row["specific_interline"]
            .as_u64()
            .filter(|n| (1..=128).contains(n))
            .ok_or("invalid_ledger_staff_interline")?;
        staffs.push((
            curve(0)?,
            curve(4)?,
            (0.5 * il as f64).round_ties_even() as i32,
        ));
    }
    let mut sequences = Vec::new();
    let mut run_count = 0;
    for y in 0..image.height {
        let mut kept = Vec::new();
        for run in crate::position_runs(image, false, y) {
            let x = run.start + run.length / 2;
            let mut closest = None;
            let mut best = i32::MAX;
            for (i, ((top, bottom, _), area)) in staffs.iter().zip(&layout.staff_areas).enumerate()
            {
                if area.contains([x as f64, y as f64])? {
                    let d = (top.y(x as f64) - y as f64).max(y as f64 - bottom.y(x as f64)) as i32;
                    if closest.is_none() || d < best {
                        closest = Some(i);
                        best = d;
                    }
                }
            }
            if closest.is_some_and(|i| best >= staffs[i].2) {
                kept.push(run);
                run_count += 1;
            }
            if run_count > crate::sections::MAX_RUNS {
                return Err("ledger_run_budget".into());
            }
        }
        sequences.push(kept);
    }
    let table = RunTable {
        orientation: Orientation::Horizontal,
        width: image.width,
        height: image.height,
        sequences,
    };
    let sections = crate::sections::build(&table, JunctionPolicy::Shift(0))?;
    let systems = grid["systems"].as_array().ok_or("invalid_ledger_systems")?;
    let mut output = Vec::new();
    for (system, area) in systems.iter().zip(&layout.system_areas) {
        let c = system["coordinates"]
            .as_array()
            .ok_or("invalid_ledger_system_coordinates")?;
        let left = c[0].as_i64().ok_or("invalid_ledger_system_left")?;
        let right = left + c[2].as_i64().ok_or("invalid_ledger_system_width")?;
        let mut selected = Vec::new();
        for section in &sections {
            let [x, y] = crate::grid_ir::centroid(section);
            if x as i64 >= left && x as i64 <= right && area.contains([x as f64, y as f64])? {
                selected.push(SeedSection {
                    id: section.id.0 as u64,
                    vertical: false,
                    first_pos: section.first_pos as i64,
                    runs: section
                        .runs
                        .iter()
                        .map(|r| [r.start as i64, r.length as i64])
                        .collect(),
                });
            }
        }
        output.push(json!({"system_id":system["id"],"sections":selected}));
    }
    Ok(
        json!({"stage":"LEDGERS.staff_distance_lag_dispatch","staff_distance_lag_complete":true,"beam_section_filter_complete":false,"ledgers_complete":false,"systems":output}),
    )
}
