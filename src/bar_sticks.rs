// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// BarFilamentBuilder/BarFilamentFactory and PeakGraph.buildBarSticks/detectCurvedPeaks.
use crate::{
    GrayImage, Run, curves, position_runs,
    scale::Scale,
    sections::{
        self, Bounds, GridPreparation, JunctionPolicy, Orientation, RunTable, Section, SectionId,
    },
};
use serde_json::{Value, json};
pub(crate) fn intersects(a: Bounds, b: Bounds) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}
fn rects(s: &Section) -> Vec<Bounds> {
    s.runs
        .iter()
        .enumerate()
        .map(|(i, r)| match s.orientation {
            Orientation::Horizontal => Bounds {
                x: r.start,
                y: s.first_pos + i,
                width: r.length,
                height: 1,
            },
            Orientation::Vertical => Bounds {
                x: s.first_pos + i,
                y: r.start,
                width: 1,
                height: r.length,
            },
        })
        .collect()
}
pub(crate) fn touch(a: &Section, b: &Section) -> bool {
    for x in rects(a) {
        for y in rects(b) {
            let dx = (x.x + x.width).min(y.x + y.width) as i64 - x.x.max(y.x) as i64;
            let dy = (x.y + x.height).min(y.y + y.height) as i64 - x.y.max(y.y) as i64;
            if dx >= 0 && dy >= 0 && (dx > 0 || dy > 0) {
                return true;
            }
        }
    }
    false
}
pub(crate) fn centroid_x(s: &Section) -> usize {
    let mut cx = 0;
    let mut cy = 0;
    for (i, r) in s.runs.iter().enumerate() {
        cx += r.length * (2 * r.start + r.length);
        cy += r.length * 2 * (s.first_pos + i);
    }
    if s.orientation == Orientation::Horizontal {
        cx / (2 * s.weight)
    } else {
        cy / (2 * s.weight)
    }
}
pub(crate) fn copy(s: &Section) -> Section {
    Section {
        id: s.id,
        orientation: s.orientation,
        first_pos: s.first_pos,
        weight: s.weight,
        runs: s
            .runs
            .iter()
            .map(|r| Run {
                start: r.start,
                length: r.length,
            })
            .collect(),
    }
}
pub fn analyze(image: &GrayImage, sc: &Scale, prep: &GridPreparation, projection: &Value) -> Value {
    if projection["status"] != "bar_candidates" {
        return json!({"status":"blocked","error":"bar_candidates_unavailable"});
    }
    // LinesRetriever.addShortSections: short horizontal runs have their own section factory pass.
    let mut short =
        GrayImage::new(image.width, image.height, vec![255; image.pixels.len()]).unwrap();
    for x in 0..image.width {
        for r in position_runs(image, true, x).filter(|r| r.length < prep.min_vertical_length) {
            for y in r.start..r.start + r.length {
                short.pixels[y * image.width + x] = 0;
            }
        }
    }
    let table = match RunTable::from_binary(&short, Orientation::Horizontal, |r| {
        r.length < prep.min_horizontal_length
    }) {
        Ok(v) => v,
        Err(e) => return json!({"status":"failed","error":e}),
    };
    let mut extra = match sections::build(&table, JunctionPolicy::Ratio(1.25)) {
        Ok(v) => v,
        Err(e) => return json!({"status":"failed","error":e}),
    };
    for s in &mut extra {
        s.id = SectionId(s.id.0 + prep.horizontal.len() as u32);
    }
    let mut all: Vec<_> = prep
        .vertical
        .iter()
        .chain(&prep.horizontal)
        .map(copy)
        .chain(extra)
        .collect();
    all.sort_by_key(|s| {
        let b = s.bounds();
        (b.x, b.y, s.id.0)
    });
    let min_len = (sc.interline.main as f64 * 0.5).round_ties_even() as usize;
    let extension = 2 * sc.interline.main;
    let mut out = Vec::new();
    for staff in projection["staff_projections"].as_array().unwrap() {
        let mut kept = Vec::new();
        let mut removed = Vec::new();
        for (peak_index, peak) in staff["peaks"].as_array().unwrap().iter().enumerate() {
            let v: Vec<_> = peak
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as usize)
                .collect();
            let core = Bounds {
                x: v[0],
                y: v[2],
                width: v[1] - v[0] + 1,
                height: v[3] - v[2] + 1,
            };
            let lookup = Bounds {
                x: core.x,
                y: core.y.saturating_sub(extension),
                width: core.width,
                height: core.height + extension + extension.min(core.y),
            };
            let source: Vec<_> = all
                .iter()
                .enumerate()
                .filter(|(_, s)| s.bounds().width <= core.width && intersects(s.bounds(), lookup))
                .map(|(i, _)| i)
                .collect();
            let mut members: Vec<_> = source
                .iter()
                .copied()
                .filter(|&i| {
                    let s = &all[i];
                    let b = s.bounds();
                    let c = centroid_x(s);
                    b.height >= min_len
                        && intersects(b, core)
                        && c >= core.x
                        && c < core.x + core.width
                })
                .collect();
            if members.is_empty() {
                removed.push(peak.clone());
                continue;
            }
            loop {
                let found = source.iter().copied().find(|i| {
                    !members.contains(i) && {
                        let b = all[*i].bounds();
                        b.x >= core.x
                            && b.x + b.width <= core.x + core.width
                            && members.iter().any(|&j| touch(&all[j], &all[*i]))
                    }
                });
                if let Some(i) = found {
                    members.push(i);
                } else {
                    break;
                }
            }
            members.sort_by_key(|&i| {
                let s = &all[i];
                let b = s.bounds();
                (b.x, b.y, s.id.0)
            });
            let curve = curves::compute_vertical(&all, &members, sc.interline.main);
            let points = curve
                .as_ref()
                .map(|c| c.points.iter().map(|p| [p[1], p[0]]).collect::<Vec<_>>());
            let radius = points.as_ref().map(|p| curves::mean_radius(p));
            kept.push(json!({"peak":peak,"grade":staff["peak_grades"][peak_index],"points":points,"brace":radius.is_some_and(|r|r<(sc.interline.main*10)as f64),"mean_radius":radius.filter(|x|x.is_finite()),"sections":members.iter().map(|&i|{let s=&all[i];let b=s.bounds();json!({"orientation":s.orientation.name(),"id":s.id.0,"bounds":[b.x,b.y,b.width,b.height],"weight":s.weight,"first_pos":s.first_pos,"runs":s.runs.iter().map(|r|[r.start,r.length]).collect::<Vec<_>>()})}).collect::<Vec<_>>()}));
        }
        out.push(json!({"staff_id":staff["staff_id"],"sticks":kept,"removed_peaks":removed}));
    }
    json!({"status":"bar_sticks","grid_complete":false,"staffs":out})
}
