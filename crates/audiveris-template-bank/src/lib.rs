// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded, hash-pinned loading of official Audiveris template assets.
//! No font rasterization, network, Java execution, interpolation or input-specific rules.
use audiveris_template_match::{AnchorOffset, AnchorSide, KeyPoint, Limits, Template};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};
pub const MAX_MANIFEST_BYTES: usize = 65_536;
pub const MAX_ASSET_BYTES: usize = 1_048_576;
const MAX_ASSETS: usize = 128;
const MAX_POINTS: usize = 65_536;
const MAX_TOTAL_POINTS: usize = 131_072;
include!("builtin_identity.rs");
include!("coverage_identity.rs");
include!("cue_identity.rs");
include!("drum_identity.rs");
include!("all_heads_identity.rs");
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    TooLarge,
    NotRegularFile,
    Integrity,
    Identity,
    Invalid(&'static str),
    UnsupportedTemplate,
    UnsupportedCalibration,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
/// Obtain this from a trusted application release, never from the untrusted bank directory.
#[derive(Clone, Copy)]
pub struct Trust<'a> {
    pub manifest_sha256: &'a str,
    pub upstream_commit: &'a str,
    pub font_sha256: &'a str,
    pub model_sha256: &'a str,
    pub coverage_input_sha256: &'a str,
}
#[derive(Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub shape: String,
    pub point_size: u16,
}
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub bank_id: String,
    pub upstream_version: String,
    pub upstream_commit: String,
    pub font_family: String,
    pub font_version: String,
    pub font_sha256: String,
    pub model_name: String,
    pub model_sha256: String,
    pub coverage_input_sha256: String,
    pub generator: String,
    pub asset_sha256: String,
    pub keys: Vec<Key>,
    pub calibration_sizes: Vec<u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    schema: u32,
    templates: Vec<RawAsset>,
    calibration: Vec<Metric>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAsset {
    key: Key,
    width: u32,
    height: u32,
    fat_bounds: [u32; 4],
    slim_bounds: [u32; 4],
    anchors: Vec<Anchor>,
    points: Vec<Point>,
}
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub name: String,
    pub x: f64,
    pub y: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Point {
    x: u32,
    y: u32,
    d: f64,
}
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Metric {
    point_size: u16,
    width: f64,
}
#[derive(Debug)]
pub struct Asset {
    key: Key,
    width: u32,
    height: u32,
    fat_bounds: [u32; 4],
    slim_bounds: [u32; 4],
    anchors: BTreeMap<String, Anchor>,
    template: Template,
}
impl Asset {
    pub fn key(&self) -> &Key {
        &self.key
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn fat_bounds(&self) -> [u32; 4] {
        self.fat_bounds
    }
    pub fn slim_bounds(&self) -> [u32; 4] {
        self.slim_bounds
    }
    pub fn template(&self) -> &Template {
        &self.template
    }
    pub fn anchor(&self, name: &str) -> Result<AnchorOffset, Error> {
        let a = self
            .anchors
            .get(name)
            .ok_or(Error::Invalid("anchor not present"))?;
        let side = anchor_side(name).ok_or(Error::Invalid("anchor name"))?;
        Ok(AnchorOffset {
            x: a.x,
            y: a.y,
            side,
        })
    }
}
#[derive(Debug)]
pub struct Bank {
    manifest: Manifest,
    assets: BTreeMap<Key, Asset>,
    metrics: BTreeMap<u16, f64>,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn is_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn key_valid(k: &Key) -> bool {
    (1..=1024).contains(&k.point_size)
        && !k.shape.is_empty()
        && k.shape.len() <= 64
        && k.shape
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c == b'_' || c.is_ascii_digit())
}
fn rect_valid(r: [u32; 4], w: u32, h: u32) -> bool {
    r[2] > 0
        && r[3] > 0
        && r[0].checked_add(r[2]).is_some_and(|x| x <= w)
        && r[1].checked_add(r[3]).is_some_and(|y| y <= h)
}
fn anchor_side(s: &str) -> Option<AnchorSide> {
    Some(match s {
        "CENTER" => AnchorSide::Center,
        "MIDDLE_LEFT" | "LEFT_STEM" | "TOP_LEFT_STEM" | "BOTTOM_LEFT_STEM" => AnchorSide::Left,
        "MIDDLE_RIGHT" | "RIGHT_STEM" | "TOP_RIGHT_STEM" | "BOTTOM_RIGHT_STEM" => AnchorSide::Right,
        _ => return None,
    })
}
fn read_bounded(path: &Path, max: usize) -> Result<Vec<u8>, Error> {
    let before = std::fs::symlink_metadata(path)?;
    if !before.file_type().is_file() {
        return Err(Error::NotRegularFile);
    }
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file: File = opts.open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(Error::NotRegularFile);
    }
    if meta.len() > max as u64 {
        return Err(Error::TooLarge);
    }
    let mut bytes = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(Error::TooLarge);
    }
    Ok(bytes)
}
impl Bank {
    pub fn load_builtin() -> Result<Self, Error> {
        Self::from_bytes(
            include_bytes!("../assets/manifest.json"),
            include_bytes!("../assets/templates.json"),
            BUILTIN_TRUST,
        )
    }
    /// The directory is caller-selected; only two fixed filenames are opened, without following symlinks.
    pub fn load_dir(path: &Path, trust: Trust<'_>) -> Result<Self, Error> {
        let m = read_bounded(&path.join("manifest.json"), MAX_MANIFEST_BYTES)?;
        // Authenticate and validate manifest before reading the (potentially larger) payload.
        Self::manifest(&m, trust)?;
        let a = read_bounded(&path.join("templates.json"), MAX_ASSET_BYTES)?;
        Self::from_bytes(&m, &a, trust)
    }
    fn manifest(bytes: &[u8], trust: Trust<'_>) -> Result<Manifest, Error> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(Error::TooLarge);
        }
        if !is_hash(trust.manifest_sha256) || digest(bytes) != trust.manifest_sha256 {
            return Err(Error::Integrity);
        }
        let m: Manifest = serde_json::from_slice(bytes)?;
        if m.schema != 1
            || m.upstream_version != "5.11.0"
            || !((m.font_family == "Bravura"
                && m.font_version == "1.392"
                && m.font_sha256 == BUILTIN_TRUST.font_sha256)
                || COVERAGE_FONTS.iter().any(|&(family, version, sha)| {
                    m.font_family == family && m.font_version == version && m.font_sha256 == sha
                }))
            || m.model_name != "basic-classifier.zip"
            || m.upstream_commit != trust.upstream_commit
            || m.font_sha256 != trust.font_sha256
            || m.model_sha256 != trust.model_sha256
            || m.coverage_input_sha256 != trust.coverage_input_sha256
        {
            return Err(Error::Identity);
        }
        if m.upstream_commit.len() != 40
            || !m.upstream_commit.bytes().all(|c| c.is_ascii_hexdigit())
            || [
                &m.font_sha256,
                &m.model_sha256,
                &m.coverage_input_sha256,
                &m.asset_sha256,
            ]
            .iter()
            .any(|s| !is_hash(s))
        {
            return Err(Error::Identity);
        }
        if m.bank_id.is_empty()
            || m.bank_id.len() > 128
            || m.generator.is_empty()
            || m.generator.len() > 256
        {
            return Err(Error::Invalid("metadata length"));
        }
        if m.keys.is_empty() || m.keys.len() > MAX_ASSETS || m.calibration_sizes.len() > 256 {
            return Err(Error::Invalid("coverage count"));
        }
        let mut keys = BTreeSet::new();
        for k in &m.keys {
            if !key_valid(k) || !keys.insert(k) {
                return Err(Error::Invalid("duplicate or noncanonical manifest key"));
            }
        }
        let mut sizes = BTreeSet::new();
        for &s in &m.calibration_sizes {
            if s == 0 || s > 1024 || !sizes.insert(s) {
                return Err(Error::Invalid("calibration coverage"));
            }
        }
        Ok(m)
    }
    pub fn from_bytes(manifest: &[u8], payload: &[u8], trust: Trust<'_>) -> Result<Self, Error> {
        if payload.len() > MAX_ASSET_BYTES {
            return Err(Error::TooLarge);
        }
        let manifest = Self::manifest(manifest, trust)?;
        if digest(payload) != manifest.asset_sha256 {
            return Err(Error::Integrity);
        }
        let p: Payload = serde_json::from_slice(payload)?;
        if p.schema != 1
            || p.templates.len() != manifest.keys.len()
            || p.calibration.len() != manifest.calibration_sizes.len()
        {
            return Err(Error::Invalid("missing or extra assets"));
        }
        let mut assets = BTreeMap::new();
        let mut total = 0usize;
        for a in p.templates {
            if !key_valid(&a.key) || !manifest.keys.contains(&a.key) {
                return Err(Error::Invalid("asset key"));
            }
            if a.width == 0
                || a.height == 0
                || a.width > 2048
                || a.height > 2048
                || !rect_valid(a.fat_bounds, a.width, a.height)
                || !rect_valid(a.slim_bounds, a.width, a.height)
            {
                return Err(Error::Invalid("dimensions or bounds"));
            }
            if a.points.is_empty() || a.points.len() > MAX_POINTS {
                return Err(Error::Invalid("point count"));
            }
            total += a.points.len();
            if total > MAX_TOTAL_POINTS {
                return Err(Error::Invalid("total point count"));
            }
            let mut seen = BTreeSet::new();
            let mut points = Vec::with_capacity(a.points.len());
            for p in a.points {
                if p.x >= a.width
                    || p.y >= a.height
                    || !p.d.is_finite()
                    || p.d.fract() != 0.
                    || p.d.abs() > 32768.
                    || !seen.insert((p.x, p.y))
                {
                    return Err(Error::Invalid("keypoint"));
                }
                points.push(KeyPoint {
                    x: p.x as i32,
                    y: p.y as i32,
                    distance: p.d,
                });
            }
            if ![3, 9].contains(&a.anchors.len()) {
                return Err(Error::Invalid("anchor count"));
            }
            let mut anchors = BTreeMap::new();
            for v in a.anchors {
                if anchor_side(&v.name).is_none()
                    || !v.x.is_finite()
                    || !v.y.is_finite()
                    || v.x < 0.
                    || v.y < 0.
                    || v.x > a.width as f64
                    || v.y > a.height as f64
                {
                    return Err(Error::Invalid("anchor"));
                }
                if anchors.insert(v.name.clone(), v).is_some() {
                    return Err(Error::Invalid("duplicate anchor"));
                }
            }
            if ["CENTER", "MIDDLE_LEFT", "MIDDLE_RIGHT"]
                .iter()
                .any(|s| !anchors.contains_key(*s))
            {
                return Err(Error::Invalid("missing base anchor"));
            }
            let template =
                Template::new(points, Limits::default()).map_err(|_| Error::Invalid("template"))?;
            let asset = Asset {
                key: a.key.clone(),
                width: a.width,
                height: a.height,
                fat_bounds: a.fat_bounds,
                slim_bounds: a.slim_bounds,
                anchors,
                template,
            };
            if assets.insert(a.key, asset).is_some() {
                return Err(Error::Invalid("duplicate asset key"));
            }
        }
        if manifest.keys.iter().any(|k| !assets.contains_key(k)) {
            return Err(Error::Invalid("missing required asset"));
        }
        let mut metrics = BTreeMap::new();
        for m in p.calibration {
            if !manifest.calibration_sizes.contains(&m.point_size)
                || !m.width.is_finite()
                || m.width <= 0.
                || m.width > 2048.
                || metrics.insert(m.point_size, m.width).is_some()
            {
                return Err(Error::Invalid("calibration metric"));
            }
        }
        if manifest
            .calibration_sizes
            .iter()
            .any(|s| !metrics.contains_key(s))
        {
            return Err(Error::Invalid("missing calibration"));
        }
        Ok(Self {
            manifest,
            assets,
            metrics,
        })
    }
    pub fn manifest_info(&self) -> &Manifest {
        &self.manifest
    }
    pub fn get(&self, shape: &str, point_size: u16) -> Result<&Asset, Error> {
        if shape.is_empty()
            || shape.len() > 64
            || !shape
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c == b'_' || c.is_ascii_digit())
        {
            return Err(Error::UnsupportedTemplate);
        }
        self.assets
            .get(&Key {
                shape: shape.to_owned(),
                point_size,
            })
            .ok_or(Error::UnsupportedTemplate)
    }
    /// Original MusicFont.computePointSize calculation over pinned AWT widths.
    /// Missing metric sizes return UnsupportedCalibration; no font calls or synthesized metrics.
    pub fn point_size_for_width(&self, width: f64) -> Result<u16, Error> {
        if !width.is_finite() || width <= 0. || width > 256. {
            return Err(Error::Invalid("head width"));
        }
        let v1 = (width * 3.3).round_ties_even() as i32;
        let dv = (width * 0.25).round_ties_even() as i32;
        let w1 = *self
            .metrics
            .get(&(v1 as u16))
            .ok_or(Error::UnsupportedCalibration)?;
        let v2 = if w1 < width { v1 + dv } else { v1 - dv };
        if !(1..=1024).contains(&v2) {
            return Err(Error::UnsupportedCalibration);
        }
        let w2 = *self
            .metrics
            .get(&(v2 as u16))
            .ok_or(Error::UnsupportedCalibration)?;
        let v = if (w2 - w1).abs() < 0.01 {
            v1 as f64
        } else {
            (v1 as f64 + (v2 - v1) as f64 * ((width - w1) / (w2 - w1))).round_ties_even()
        };
        if !(1. ..=1024.).contains(&v) {
            return Err(Error::UnsupportedCalibration);
        }
        Ok(v as u16)
    }
    /// MusicFont.getHeadPointSize's scale interpolation; not template interpolation.
    pub fn staff_point_size(
        staff_interline: f64,
        main_interline: f64,
        stored: u16,
    ) -> Result<u16, Error> {
        if !staff_interline.is_finite()
            || !main_interline.is_finite()
            || staff_interline <= 0.
            || main_interline <= 0.
            || stored == 0
        {
            return Err(Error::Invalid("staff scale"));
        }
        let n = (staff_interline / main_interline * stored as f64).round_ties_even();
        if !(1. ..=1024.).contains(&n) {
            return Err(Error::UnsupportedTemplate);
        }
        Ok(n as u16)
    }
}
