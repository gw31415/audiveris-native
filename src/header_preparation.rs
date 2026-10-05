// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// HeaderBuilder.computeHeaderStarts only; clef/key/time recognition is not implemented.
use crate::curves::Curve;
use serde_json::{Value, json};

pub fn from_grid(grid: &Value) -> Result<Value, String> {
    let systems = grid["bar_semantics"]["systems"]
        .as_array()
        .ok_or("missing_grid_systems")?;
    let mut headers = Vec::new();
    for staff in grid["staffs"].as_array().ok_or("missing_grid_staffs")? {
        let sid = &staff["id"];
        let sys = systems
            .iter()
            .find(|s| s["id"] == staff["system_id"])
            .ok_or("missing_header_system")?;
        let mut bars: Vec<_> = sys["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["kind"] == "BarlineInter" && v["staff_id"] == *sid)
            .collect();
        // Staff.addBarline orders by integer center abscissa.
        bars.sort_by_key(|v| {
            v["bounds"][0].as_i64().unwrap() + v["bounds"][2].as_i64().unwrap() / 2
        });
        let left = staff["left"].as_f64().ok_or("missing_header_left")? as i64;
        let side = bars.first().copied().filter(|v| {
            let b = &v["bounds"];
            b[0].as_i64().unwrap() <= left && left < b[0].as_i64().unwrap() + b[2].as_i64().unwrap()
        });
        let (start, source, dummy) = if let Some(mut last) = side {
            let relations = sys["relations"].as_array().unwrap();
            let mut visited = Vec::new();
            loop {
                let id = last["id"].as_u64().unwrap();
                if visited.contains(&id) {
                    return Err("unsupported_header_bar_group_cycle".into());
                }
                visited.push(id);
                let mut moved = false;
                // Snapshot incident relations of the bar at loop entry, matching getRelations.
                for r in relations.iter().filter(|r| {
                    r["kind"] == "BarGroupRelation" && (r["source"] == id || r["target"] == id)
                }) {
                    if last["id"] == r["source"] {
                        last = bars
                            .iter()
                            .copied()
                            .find(|v| v["id"] == r["target"])
                            .ok_or("unsupported_header_bar_group_target")?;
                        moved = true;
                    }
                }
                if !moved {
                    break;
                }
            }
            (
                last["bounds"][0].as_i64().unwrap() + last["bounds"][2].as_i64().unwrap(),
                last["bounds"].clone(),
                Value::Null,
            )
        } else {
            let lines = staff["lines"].as_array().ok_or("missing_header_lines")?;
            let y = |line: &Value| -> Result<i64, String> {
                let points = line["points"]
                    .as_array()
                    .ok_or("missing_header_line_points")?
                    .iter()
                    .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
                    .collect();
                let c = Curve::interpolate(points).ok_or("invalid_header_curve")?;
                Ok(c.y(left as f64).round_ties_even() as i64)
            };
            let first = y(lines.first().ok_or("empty_header_staff")?)?;
            let last = y(lines.last().unwrap())?;
            (
                left,
                Value::Null,
                json!({"kind":"BarlineInter","shape":"DUMMY_BARLINE","staff_id":sid,"bounds":[left,first,1,last-first],"grade":1.0,"frozen":false}),
            )
        };
        headers.push(json!({"staff_id":sid,"start":start,"stop":start,"last_group_bar_bounds":source,"dummy_barline":dummy}));
    }
    Ok(
        json!({"stage":"HEADERS.computeHeaderStarts","staff_headers":headers,"headers_complete":false,"note_recognition_complete":false,"pending":["clef recognition and column selection","key signatures","time signatures","header contextualization and freezing"]}),
    )
}
