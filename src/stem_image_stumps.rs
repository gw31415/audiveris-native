// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image GRID sections -> automatic corner stump assembly, retaining an owned
//! physical page index for following builder/cache stages. Final stems/SIG pending.
use crate::{
    GrayImage,
    beam_dispatch::{self, Group, Registry},
    stem_glyph, stem_image_seeds,
};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|a| a.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image stump {label}"))
}
fn id(v: &Value) -> Result<u64, String> {
    v.as_u64()
        .filter(|i| *i > 0)
        .ok_or("invalid image stump identity".into())
}
#[derive(Clone)]
struct Section {
    id: u64,
    first: i32,
    runs: Vec<[i32; 2]>,
    bounds: [i32; 4],
}
impl Section {
    fn from_value(v: &Value) -> Result<Self, String> {
        let id = id(&v["id"])?;
        let first = v["first_pos"]
            .as_i64()
            .filter(|x| (0..=4_000_000).contains(x))
            .ok_or("invalid image stump section position")? as i32;
        let mut runs = Vec::new();
        for r in rows(&v["runs"], 8192, "section runs")? {
            let p: [i32; 2] =
                serde_json::from_value(r.clone()).map_err(|_| "invalid image stump section run")?;
            if !(0..=4_000_000).contains(&p[0])
                || !(1..=4_000_000).contains(&p[1])
                || i64::from(p[0]) + i64::from(p[1]) > 4_000_000
            {
                return Err("invalid image stump section run bounds".into());
            }
            runs.push(p);
        }
        if runs.is_empty() || first as usize + runs.len() > 4_000_000 {
            return Err("invalid image stump empty/large section".into());
        }
        let s = Self::from_runs(id, first, runs);
        if json!(s.bounds) != v["bounds"] {
            return Err("inconsistent image stump section bounds".into());
        }
        Ok(s)
    }
    fn from_runs(id: u64, first: i32, runs: Vec<[i32; 2]>) -> Self {
        let top = runs.iter().map(|r| r[0]).min().expect("nonempty section");
        let bottom = runs
            .iter()
            .map(|r| r[0] + r[1])
            .max()
            .expect("nonempty section");
        Self {
            id,
            first,
            bounds: [first, top, runs.len() as i32, bottom - top],
            runs,
        }
    }
    fn glyph(&self) -> GlyphPixels {
        GlyphPixels {
            left: self.first,
            top: self.bounds[1],
            width: self.runs.len(),
            height: self.bounds[3] as usize,
            orientation: Orientation::Vertical,
            runs: self
                .runs
                .iter()
                .enumerate()
                .map(|(position, r)| Run {
                    position,
                    start: (r[0] - self.bounds[1]) as usize,
                    length: r[1] as usize,
                })
                .collect(),
        }
    }
    // For an integer Point, the oriented run contour's half-open pixel cells
    // agree with BasicSection.getPolygon().contains. Fractional/bounds-only
    // membership is not substituted here.
    fn contains(&self, p: [i32; 2]) -> bool {
        let x = i64::from(p[0]) - i64::from(self.first);
        x >= 0
            && self
                .runs
                .get(x as usize)
                .is_some_and(|r| p[1] >= r[0] && p[1] < r[0] + r[1])
    }
    // BasicSection.intersects(Shape) tests each oriented run rectangle, not
    // the section bounding box (which can include background at a head corner).
    fn intersects(&self, area: [f64; 4]) -> bool {
        overlaps(area, self.bounds)
            && self
                .runs
                .iter()
                .enumerate()
                .any(|(i, r)| overlaps(area, [self.first + i as i32, r[0], 1, r[1]]))
    }
    fn clipped(&self, x: f64, thickness: i32) -> Option<Self> {
        let left = (x - f64::from(thickness) / 2.).round_ties_even() as i32;
        let start = (left - self.first).max(0);
        let stop = (left + thickness - self.first).min(self.runs.len() as i32);
        (stop > start).then(|| {
            Self::from_runs(
                self.id,
                left,
                self.runs[start as usize..stop as usize].to_vec(),
            )
        })
    }
}
fn overlaps(a: [f64; 4], b: [i32; 4]) -> bool {
    a[2] > 0.
        && a[3] > 0.
        && a[0] < f64::from(b[0] + b[2])
        && f64::from(b[0]) < a[0] + a[2]
        && a[1] < f64::from(b[1] + b[3])
        && f64::from(b[1]) < a[1] + a[3]
}
fn compound_width(selected: &BTreeSet<usize>, sections: &[&Section]) -> i32 {
    let left = selected
        .iter()
        .map(|i| sections[*i].bounds[0])
        .min()
        .expect("nonempty compound");
    let right = selected
        .iter()
        .map(|i| sections[*i].bounds[0] + sections[*i].bounds[2])
        .max()
        .expect("nonempty compound");
    right - left
}
fn assemble(
    sections: &[&Section],
    reference: [f64; 2],
    vertical: i32,
    interline: f64,
    thickness: i32,
) -> Result<(Option<GlyphPixels>, Value), String> {
    let dy = (interline * 0.2).round_ties_even();
    let dx = interline * 0.1;
    let left = reference[0] - dx;
    let right = reference[0] + dx;
    let area = [
        left,
        if vertical > 0 {
            reference[1]
        } else {
            reference[1] - dy
        },
        right - left,
        dy,
    ];
    let mut candidates = sections
        .iter()
        .copied()
        .filter(|s| s.intersects(area))
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        let dist = |s: &Section| (f64::from(s.bounds[0] + s.bounds[2] / 2) - reference[0]).abs();
        dist(a).total_cmp(&dist(b))
    });
    if candidates.is_empty() {
        return Ok((
            None,
            json!({"area":area,"candidate_section_ids":[],"retained_section_ids":[],"subsection":null}),
        ));
    }
    let mut kept = BTreeSet::new();
    let mut removed = Vec::new();
    let point = [reference[0] as i32, reference[1] as i32];
    if let Some(index) = candidates.iter().position(|s| s.contains(point)) {
        kept.insert(index);
    }
    for index in 0..candidates.len() {
        kept.insert(index);
        if compound_width(&kept, &candidates) > thickness {
            kept.remove(&index);
            removed.push(candidates[index].id);
        }
    }
    let mut crop = None;
    let parts = if kept.is_empty() {
        if let Some(s) = candidates[0].clipped(reference[0], thickness) {
            crop = Some(
                json!({"source_section_id":s.id,"first_pos":s.first,"runs":s.runs,"glyph":beam_dispatch::glyph_json(&s.glyph())}),
            );
            vec![s.glyph()]
        } else {
            Vec::new()
        }
    } else {
        kept.iter().map(|i| candidates[*i].glyph()).collect()
    };
    let meta = json!({"area":area,"candidate_section_ids":candidates.iter().map(|s|s.id).collect::<Vec<_>>(),"retained_section_ids":kept.iter().map(|i|candidates[*i].id).collect::<Vec<_>>(),"width_removed_section_ids":removed,"subsection":crop});
    Ok((
        if parts.is_empty() {
            None
        } else {
            Some(stem_glyph::assemble(&parts)?)
        },
        meta,
    ))
}
/// Registry ownership is retained across later phases by callers of prepare.
/// STUMP is attached only to a newly registered glyph; canonical reuse preserves
/// the original existing groups, matching GlyphIndex.registerOriginal.
pub struct Prepared {
    pub registry: Registry,
    pub report: Value,
    stump_groups: BTreeSet<usize>,
}
impl Prepared {
    pub fn snapshot_registry(&self) -> Value {
        let mut v = self.registry.snapshot();
        for id in &self.stump_groups {
            v[*id - 1]["groups"]
                .as_array_mut()
                .expect("registry groups")
                .push(json!("STUMP"));
        }
        v
    }
    pub(crate) fn register_stump(&mut self, g: &GlyphPixels) -> Result<usize, String> {
        let before = self.registry.len();
        let id = self.registry.register(g)?;
        if self.registry.len() > before {
            self.stump_groups.insert(id);
        }
        Ok(id)
    }
}
pub fn prepare(image: &GrayImage, page: &Value) -> Result<Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<Registry>,
) -> Result<Prepared, String> {
    let seeds = stem_image_seeds::analyze(image, page)?;
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let interline = scale.interline.main as f64;
    let thickness = page["stem_seeds"]["stem_scaling"]["stem_scale"]["main"]
        .as_i64()
        .filter(|v| (1..=128).contains(v))
        .ok_or("invalid image stump stem thickness")? as i32;
    let min_stump = (interline * 0.5).round_ties_even() as i32;
    let live = registry.is_some();
    let mut prepared = Prepared {
        registry: registry.unwrap_or_default(),
        report: Value::Null,
        stump_groups: BTreeSet::new(),
    };
    if live {
        if prepared.registry.snapshot() != seeds["physical_registry"] {
            return Err("owned upstream physical registry does not match image HEADS".into());
        }
    } else {
        for r in rows(&seeds["physical_registry"], 100_000, "registry")? {
            let g = beam_dispatch::glyph_from_json(&r["glyph"])?;
            let n = prepared.registry.register(&g)?;
            if id(&r["id"])? != n as u64 {
                return Err("invalid image stump registry order".into());
            }
            for group in rows(&r["groups"], 4, "groups")? {
                prepared.registry.add_group(
                    n,
                    match group.as_str() {
                        Some("VERTICAL_SEED") => Group::VerticalSeed,
                        Some("BEAM_SPOT") => Group::BeamSpot,
                        Some("BLACK_HEAD_SPOT") => Group::BlackHeadSpot,
                        Some("BLACK_STACK_SPOT") => Group::BlackStackSpot,
                        _ => return Err("unsupported image stump upstream group".into()),
                    },
                )?;
            }
        }
    }
    let initial = prepared.registry.len();
    let mut sections = BTreeMap::new();
    let mut run_budget = 0usize;
    for v in rows(&page["grid_ir"]["vlag"], 32768, "vertical lag")? {
        let s = Section::from_value(v)?;
        run_budget = run_budget
            .checked_add(s.runs.len())
            .filter(|v| *v <= 1_048_576)
            .ok_or("unsupported image stump run budget")?;
        if sections.insert(s.id, s).is_some() {
            return Err("duplicate image stump section identity".into());
        }
    }
    let grid_systems = rows(&page["grid_ir"]["systems"], 64, "grid systems")?;
    let seed_systems = rows(&seeds["systems"], 64, "seed systems")?;
    if grid_systems.len() != seed_systems.len() {
        return Err("invalid image stump system count".into());
    }
    let mut systems = Vec::new();
    let mut accepted = 0;
    let mut fallback = 0;
    let mut rejected = 0;
    let mut work = 0usize;
    for (grid, sys) in grid_systems.iter().zip(seed_systems) {
        if grid["id"] != sys["system"] {
            return Err("invalid image stump system identity/order".into());
        }
        let mut members = Vec::new();
        let mut unique = BTreeSet::new();
        for v in rows(
            &grid["vertical_section_ids"],
            32768,
            "system vertical sections",
        )? {
            let n = id(v)?;
            if !unique.insert(n) {
                return Err("duplicate image stump system section".into());
            }
            members.push(
                sections
                    .get(&n)
                    .ok_or("unknown image stump system section")?,
            );
        }
        let cases = rows(&sys["cases"], 8192, "corners")?;
        work = work
            .checked_add(cases.len() * members.len())
            .filter(|v| *v <= 20_000_000)
            .ok_or("unsupported image stump work budget")?;
        let mut result = Vec::new();
        for c in cases {
            let mut selected = c["selected_seed_id"].as_u64().map(|n| n as usize);
            let mut assembly = Value::Null;
            let mut registered = None;
            let mut out = None;
            if selected.is_none() {
                let reference: [f64; 2] = serde_json::from_value(c["head"]["reference"].clone())
                    .map_err(|_| "invalid image stump reference")?;
                let vertical = if c["head"]["vertical"] == "TOP" {
                    -1
                } else {
                    1
                };
                let (glyph, meta) = assemble(&members, reference, vertical, interline, thickness)?;
                assembly = meta;
                if let Some(glyph) = glyph {
                    let n = prepared.register_stump(&glyph)?;
                    registered = Some(n);
                    let ref_y = reference[1].round_ties_even() as i32;
                    let extent = if vertical > 0 {
                        glyph.top + glyph.height as i32 - 1 - ref_y
                    } else {
                        ref_y - glyph.top
                    };
                    let yes = extent >= min_stump;
                    out = Some(yes);
                    if yes {
                        selected = Some(n);
                        fallback += 1;
                    } else {
                        rejected += 1;
                    }
                }
            }
            if selected.is_some() {
                accepted += 1;
            }
            result.push(json!({"head":c["head"],"seed_id":c["selected_seed_id"],"assembly":assembly,"registered_fallback_id":registered,"fallback_stands_out":out,"selected_stump_id":selected,"selected_stump":selected.map(|n|beam_dispatch::glyph_json(&prepared.registry.get(n).expect("owned stump").glyph))}));
        }
        systems.push(json!({"system":sys["system"],"cases":result}));
    }
    prepared.report = json!({"schema":"audiveris-native.stem-image-stumps.v1","seed_plan":seeds,"systems":systems,"physical_registry":prepared.snapshot_registry(),"upstream_registry_count":initial,"new_registered_fallback_glyphs":prepared.registry.len()-initial,"selected_stump_corners":accepted,"selected_section_stump_corners":fallback,"registered_rejected_fallback_corners":rejected,"corner_count":seeds["corner_count"],"image_corner_stump_selection_complete":true,"final_stem_glyph_selection_complete":false,"automatic_raster_pipeline_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["other-head and beam sibling linker discovery; automatic items/chunks/gap/length/final stem prefix glyph","live HEADS Graph initialization and per-system owned cache/head relations","original contextual cleaner/checkNeededStems/beam epilog","fresh original transient context; nonzero skew/scans/special staff/full24 pages"]});
    Ok(prepared)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_run_intersection_polygon_crop_and_registry_boundaries() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-stumps/controls.json"
        ))
        .unwrap();
        let section = |c: &Value| {
            Section::from_runs(
                1,
                c["first_pos"].as_i64().unwrap() as i32,
                serde_json::from_value(c["runs"].clone()).unwrap(),
            )
        };
        for c in o["run_intersection_controls"].as_array().unwrap() {
            let area = serde_json::from_value(c["area"].clone()).unwrap();
            assert_eq!(
                section(c).intersects(area),
                c["intersects"].as_bool().unwrap()
            );
        }
        for c in o["polygon_contains_controls"].as_array().unwrap() {
            let point = serde_json::from_value(c["point"].clone()).unwrap();
            assert_eq!(section(c).contains(point), c["contains"].as_bool().unwrap());
        }
        for c in o["subsection_controls"].as_array().unwrap() {
            let clip = section(c).clipped(
                c["reference_x"].as_f64().unwrap(),
                c["stem_thickness"].as_i64().unwrap() as i32,
            );
            assert_eq!(
                clip.as_ref()
                    .map(|s| beam_dispatch::glyph_json(&s.glyph()))
                    .unwrap_or(Value::Null),
                c["glyph"]
            );
        }
        let mut p = Prepared {
            registry: Registry::default(),
            report: Value::Null,
            stump_groups: BTreeSet::new(),
        };
        let seed = Section::from_runs(1, 2000, vec![[2500, 3]; 2]).glyph();
        let id = p.registry.register(&seed).unwrap();
        p.registry.add_group(id, Group::VerticalSeed).unwrap();
        assert_eq!(p.register_stump(&seed).unwrap(), id);
        assert_eq!(
            p.snapshot_registry()[id - 1]["groups"],
            o["registry_reuse_control"]["groups"]
        );
        assert_eq!(o["registry_reuse_control"]["same_original_object"], true);
        let rejected = Section::from_runs(2, 2010, vec![[2500, 3]; 2]).glyph();
        let id = p.register_stump(&rejected).unwrap();
        assert_eq!(p.registry.get(id).unwrap().glyph, rejected);
        assert_eq!(
            p.snapshot_registry()[id - 1]["groups"],
            o["registered_rejected_control"]["groups"]
        );
        assert_eq!(o["registered_rejected_control"]["stands_out"], false);
        assert_eq!(
            o["registered_rejected_control"]["same_registered_entity"],
            true
        );
    }
}
