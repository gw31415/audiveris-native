// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original processStaff ordering and Scanner shape selection for the default
//! drum map. A plan is a dependency contract, not full special-staff page parity.
use audiveris_template_bank::Bank;
use serde::{Deserialize, Serialize};

/// Explicit, bounded oval template catalog carried between HEADS dependencies.
/// This does not infer a page font or certify upstream font/staff processing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateCatalog {
    pub family: String,
    pub stored_point_size: u16,
}
impl TemplateCatalog {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.family.as_str(), "Bravura" | "Leland" | "FinaleJazz")
            || (!matches!(self.stored_point_size, 32 | 40 | 51 | 64)
                && !(self.family == "Bravura" && matches!(self.stored_point_size, 44 | 47)))
        {
            return Err("unsupported_head_template_catalog".into());
        }
        Ok(())
    }
    pub fn from_head_scale(scale: &serde_json::Value) -> Result<Self, String> {
        let family = scale["music_font"]["name"]
            .as_str()
            .ok_or("unsupported_head_template_catalog")?
            .to_owned();
        let stored_point_size = scale["music_font"]["pointSize"]
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .ok_or("unsupported_head_template_catalog")?;
        let catalog = Self {
            family,
            stored_point_size,
        };
        catalog.validate()?;
        Ok(catalog)
    }
    pub fn from_report(report: &serde_json::Value) -> Result<Option<Self>, String> {
        if report["template_catalog"].is_null() {
            if report["template_catalog_required"] == true {
                return Err("missing_head_template_catalog".into());
            }
            return Ok(None);
        }
        let catalog: Self = serde_json::from_value(report["template_catalog"].clone())
            .map_err(|_| "invalid_head_template_catalog")?;
        catalog.validate()?;
        Ok(Some(catalog))
    }
    pub fn bank(&self) -> Result<Bank, String> {
        self.validate()?;
        if self.family == "Bravura" && self.stored_point_size == 44 {
            Bank::load_cue_default().map_err(|e| e.to_string())
        } else if self.family == "Bravura" && self.stored_point_size == 47 {
            Bank::load_calibrated_bravura(47).map_err(|e| e.to_string())
        } else {
            Bank::load_coverage(&self.family).map_err(|e| e.to_string())
        }
    }
    pub fn staff_point_size(&self, specific: usize, main: usize) -> Result<u16, String> {
        self.validate()?;
        if !(1..=128).contains(&specific) || !(1..=128).contains(&main) {
            return Err("unsupported_head_staff_template_catalog".into());
        }
        let size = Bank::staff_point_size(specific as f64, main as f64, self.stored_point_size)
            .map_err(|e| e.to_string())?;
        let supported = if self.family == "Bravura" && self.stored_point_size == 44 {
            matches!(size, 28 | 44)
        } else if self.family == "Bravura" && self.stored_point_size == 47 {
            size == 47
        } else {
            matches!(size, 32 | 40 | 51 | 64)
        };
        if !supported {
            return Err("unsupported_head_staff_template_catalog".into());
        }
        Ok(size)
    }
}

include!("head_staff_catalog_data.rs");
#[derive(Clone, Copy, Debug, Default)]
pub struct Processing {
    pub small_heads: bool,
    pub drum_notation: bool,
    pub one_line_staves: bool,
    pub cross_heads: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Position {
    pub line_index: usize,
    pub dir: i32,
    pub pitch: i32,
    pub second_line_index: Option<usize>,
    pub is_open: bool,
    pub y_offsets: Vec<i32>,
}
/// One-line staffs have three pitches and no ledgers. Tablature is skipped
/// before scale/template lookup, as in original buildHeads. Offsets use the
/// sheet's main interline; staff interline controls the physical pitch geometry.
pub fn positions(
    line_count: usize,
    tablature: bool,
    main_interline: usize,
) -> Result<Vec<Position>, String> {
    if tablature {
        return Ok(Vec::new());
    }
    if !(1..=128).contains(&line_count) || !(1..=128).contains(&main_interline) {
        return Err("unsupported_head_staff_plan".into());
    }
    let mut out = Vec::with_capacity(2 * line_count + 1);
    let mut pitch = -(line_count as i32);
    for line_index in 0..line_count {
        for dir in if line_index + 1 == line_count {
            &[-1, 0, 1][..]
        } else {
            &[-1, 0][..]
        } {
            let second_line_index = if *dir == -1 {
                line_index.checked_sub(1)
            } else {
                None
            };
            let is_open = pitch % 2 != 0
                && (second_line_index.is_none() || pitch.unsigned_abs() == line_count as u32);
            out.push(Position {
                line_index,
                dir: *dir,
                pitch,
                second_line_index,
                is_open,
                y_offsets: if is_open && *dir == 0 {
                    vec![0; 1 + ((0.2 * main_interline as f64).round_ties_even() as usize).max(1)]
                } else {
                    crate::head_seed_search::y_offsets(main_interline, *dir, is_open)?
                },
            });
            pitch += 1;
        }
    }
    Ok(out)
}
#[derive(Clone, Debug, Serialize)]
pub struct Catalog {
    pub all_shapes: Vec<&'static str>,
    pub stem_shapes: Vec<&'static str>,
    pub hollow_shapes: Vec<&'static str>,
}
/// Original buildShapeList -> MATCHED_SHAPES -> sheet processing switches.
/// Unknown drum pitch/staff map yields an empty catalog, without oval fallback.
/// Custom user drum-set.xml overrides require a separately supplied future map.
pub fn catalog(line_count: usize, pitch: i32, drum: bool, switches: Processing) -> Catalog {
    let drum = drum || line_count == 1;
    let motifs = if drum {
        DRUM_MOTIFS
            .iter()
            .find(|&&(n, p, _)| n == line_count && p == pitch)
            .map_or(0, |&(_, _, m)| m)
    } else {
        u32::MAX
    };
    let mut result = Catalog {
        all_shapes: Vec::new(),
        stem_shapes: Vec::new(),
        hollow_shapes: Vec::new(),
    };
    for &(name, flags, motif) in HEAD_SHAPES {
        if (drum && (flags & MATCHED == 0 || motifs & motif == 0))
            || (!switches.small_heads && flags & SMALL != 0)
            || (!switches.drum_notation
                && !switches.one_line_staves
                && (flags & DRUM_ONLY != 0 || (!switches.cross_heads && flags & CROSS != 0)))
        {
            continue;
        }
        result.all_shapes.push(name);
        if flags & STEM != 0 {
            result.stem_shapes.push(name);
        }
        if flags & HOLLOW != 0 {
            result.hollow_shapes.push(name);
        }
    }
    result
}
#[derive(Clone, Copy)]
pub struct StaffSpec {
    pub line_count: usize,
    pub tablature: bool,
    pub drum: bool,
    pub specific_interline: usize,
    pub main_interline: usize,
    pub stored_point_size: u16,
}
#[derive(Debug, Serialize)]
pub struct Plan {
    pub tablature_skipped: bool,
    pub ledger_allowed: bool,
    pub point_size: Option<u16>,
    pub scanners: Vec<serde_json::Value>,
}
pub fn plan(spec: StaffSpec, switches: Processing) -> Result<Plan, String> {
    let positions = positions(spec.line_count, spec.tablature, spec.main_interline)?;
    if spec.tablature {
        return Ok(Plan {
            tablature_skipped: true,
            ledger_allowed: false,
            point_size: None,
            scanners: Vec::new(),
        });
    }
    if !(1..=128).contains(&spec.specific_interline) {
        return Err("unsupported_head_staff_plan".into());
    }
    let point_size = Bank::staff_point_size(
        spec.specific_interline as f64,
        spec.main_interline as f64,
        spec.stored_point_size,
    )
    .map_err(|e| e.to_string())?;
    let mut scanners = Vec::with_capacity(positions.len());
    for position in positions {
        let shapes = catalog(spec.line_count, position.pitch, spec.drum, switches);
        let mut value = serde_json::to_value(position).map_err(|e| e.to_string())?;
        let fields = serde_json::to_value(shapes).map_err(|e| e.to_string())?;
        value
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        scanners.push(value);
    }
    Ok(Plan {
        tablature_skipped: false,
        ledger_allowed: spec.line_count != 1,
        point_size: Some(point_size),
        scanners,
    })
}
