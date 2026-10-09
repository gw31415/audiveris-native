// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-comparable Slur endpoint lookup over borrowed recognition owners.
//! Geometry and per-chord Head lookup from Audiveris pin 9e1e55cd2746037d059345881c53e6a6754bffbd.
//! Geometry/lookup and observed nonmirror pair cases match Java; positive mirror/orphan,
//! native ownership, obstacle/clump/tie lifecycle and graph insertion remain unverified.
use audiveris_area::{LinearArea, hypot};
pub type Point = [f64; 2];
pub type Rect = [i64; 4];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
impl Side {
    pub fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Curve {
    pub bezier: [Point; 4],
    pub above: bool,
}
#[derive(Clone, Debug)]
pub struct LookupGeometry {
    pub horizontal: bool,
    pub endpoint: [Point; 2],
    pub target: [Point; 2],
    pub bisector_unit: Point,
    pub area: [LinearArea; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InvalidInterline,
    NonFiniteGeometry,
    DegenerateGeometry,
    InvalidBounds,
    Owner(String),
    Area(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInterline => f.write_str("InvalidInterline"),
            Self::NonFiniteGeometry => f.write_str("NonFiniteGeometry"),
            Self::DegenerateGeometry => f.write_str("DegenerateGeometry"),
            Self::InvalidBounds => f.write_str("InvalidBounds"),
            Self::Owner(message) => write!(f, "Owner({message:?})"),
            Self::Area(message) => write!(f, "Area({message:?})"),
        }
    }
}
fn add(a: Point, b: Point) -> Point {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}
fn mul(a: Point, b: f64) -> Point {
    [a[0] * b, a[1] * b]
}
fn sq(a: Point) -> f64 {
    a[0] * a[0] + a[1] * a[1]
}
fn point_distance(a: Point, b: Point) -> f64 {
    sq(sub(a, b)).sqrt()
}
fn finite(p: Point) -> bool {
    p.iter().all(|v| v.is_finite())
}
fn unit(a: Point) -> Point {
    let d = hypot(a[0], a[1]);
    [a[0] / d, a[1] / d]
}
fn extension(a: Point, b: Point, d: f64) -> Point {
    let seg = point_distance(a, b);
    [
        b[0] + d * (b[0] - a[0]) / seg,
        b[1] + d * (b[1] - a[1]) / seg,
    ]
}
fn bisector(a: Point, b: Point) -> [Point; 2] {
    let hx = (b[0] - a[0]) / 2.;
    let hy = (b[1] - a[1]) / 2.;
    let mx = a[0] + hx;
    let my = a[1] + hy;
    [[mx + hy, my - hx], [mx - hy, my + hx]]
}
fn intersection(a: Point, b: Point, c: Point, d: Point) -> Point {
    let den = (a[0] - b[0]) * (c[1] - d[1]) - (a[1] - b[1]) * (c[0] - d[0]);
    let ab = a[0] * b[1] - a[1] * b[0];
    let cd = c[0] * d[1] - c[1] * d[0];
    [
        (ab * (c[0] - d[0]) - (a[0] - b[0]) * cd) / den,
        (ab * (c[1] - d[1]) - (a[1] - b[1]) * cd) / den,
    ]
}
fn at_x(a: Point, b: Point, x: f64) -> Point {
    intersection(a, b, [x, 0.], [x, 1000.])
}
fn relative_ccw(a: Point, b: Point, p: Point) -> i32 {
    let d = sub(b, a);
    let mut p = sub(p, a);
    let mut ccw = p[0] * d[1] - p[1] * d[0];
    if ccw == 0. {
        ccw = p[0] * d[0] + p[1] * d[1];
        if ccw > 0. {
            p = sub(p, d);
            ccw = p[0] * d[0] + p[1] * d[1];
            if ccw < 0. {
                ccw = 0.;
            }
        }
    }
    if ccw < 0. {
        -1
    } else if ccw > 0. {
        1
    } else {
        0
    }
}
fn scaled(il: f64, fraction: f64) -> f64 {
    (il * fraction).round_ties_even()
}
/// Raw finite cubic is the recomputed native candidate curve, not a persisted source curve.
pub fn define_lookup(curve: Curve, interline: f64) -> Result<LookupGeometry, Error> {
    if !interline.is_finite() || interline <= 0. {
        return Err(Error::InvalidInterline);
    }
    if !curve.bezier.iter().copied().all(finite) {
        return Err(Error::NonFiniteGeometry);
    }
    let [p1, c1, c2, p2] = curve.bezier;
    let dx = p2[0] - p1[0];
    let dy = p2[1] - p1[1];
    let horizontal = (dy / dx).abs() <= 0.5 || dx.abs() >= scaled(interline, 6.);
    let first = p1.map(f64::round_ties_even);
    let last = p2.map(f64::round_ties_even);
    let width = (last[0] - first[0]).abs();
    let vdir = relative_ccw(p1, p2, [(c1[0] + c2[0]) / 2., (c1[1] + c2[1]) / 2.]) as f64;
    let hdir = if last[0] > first[0] {
        1.
    } else if last[0] < first[0] {
        -1.
    } else {
        0.
    };
    let mid = [(first[0] + last[0]) / 2., (first[1] + last[1]) / 2.];
    let bis = if curve.above {
        bisector(p1, p2)
    } else {
        bisector(p2, p1)
    };
    // Point2D.distance uses sqrt(dx*dx+dy*dy); CubicUtil end vectors use Math.hypot.
    let bis_len = point_distance(bis[0], bis[1]);
    let bis_unit = [
        (bis[1][0] - bis[0][0]) / bis_len,
        (bis[1][1] - bis[0][1]) / bis_len,
    ];
    let ext = scaled(interline, 2.);
    let first_ext = extension(mid, first, ext);
    let last_ext = extension(mid, last, ext);
    let polygon = if horizontal {
        let depth = vdir * scaled(interline, 3.);
        let fb = [first_ext[0], first_ext[1] + depth];
        let lb = [last_ext[0], last_ext[1] + depth];
        let inner = scaled(interline, 0.5);
        if width > 2. * inner {
            let fi = extension(mid, first, -inner);
            let li = extension(mid, last, -inner);
            [
                [fi, first_ext, fb, at_x(fb, lb, fi[0])],
                [li, last_ext, lb, at_x(fb, lb, li[0])],
            ]
        } else {
            let mb = at_x(fb, lb, mid[0]);
            [[mid, first_ext, fb, mb], [mid, last_ext, lb, mb]]
        }
    } else {
        let depth = scaled(
            interline,
            if width <= scaled(interline, 1.5) {
                1.5
            } else {
                2.5
            },
        );
        let delta = mul(bis_unit, depth);
        let fb = add(first_ext, delta);
        let lb = add(last_ext, delta);
        let inner = scaled(interline, 1.5);
        if point_distance(first, last) > 2. * inner {
            let fi = extension(first_ext, first, inner);
            let li = extension(last_ext, last, inner);
            [
                [fi, first_ext, fb, add(fi, delta)],
                [li, last_ext, lb, add(li, delta)],
            ]
        } else {
            let b = if vdir == hdir {
                bisector(first, last)
            } else {
                bisector(last, first)
            };
            let bi = intersection(fb, lb, b[0], b[1]);
            [[mid, first_ext, fb, bi], [mid, last_ext, lb, bi]]
        }
    };
    let target_ext = scaled(interline, 0.5);
    let target = [
        add(p1, mul(unit(sub(p1, c1)), target_ext)),
        add(p2, mul(unit(sub(p2, c2)), target_ext)),
    ];
    // The source pruner discards area-construction failures. Keep typed failures visible here.
    if !polygon
        .iter()
        .flatten()
        .copied()
        .chain(target)
        .chain([bis_unit])
        .all(finite)
    {
        return Err(Error::DegenerateGeometry);
    }
    let area = [
        LinearArea::polygon(&polygon[0]).map_err(Error::Area)?,
        LinearArea::polygon(&polygon[1]).map_err(Error::Area)?,
    ];
    Ok(LookupGeometry {
        horizontal,
        endpoint: [p1, p2],
        target,
        bisector_unit: bis_unit,
        area,
    })
}
/// Implement inside the retained Graph/OwnedMusic boundary. Returned objects are live borrowed
/// owners. Serialized fixture/report IDs or recreated Head/Chord objects cannot implement admission.
pub trait ChordOwners {
    type Chord;
    type Head;
    fn chord_bounds(&self, chord: &Self::Chord) -> Result<Rect, String>;
    fn head_bounds(&self, head: &Self::Head) -> Result<Rect, String>;
    /// Source getNotes order (bottom-up with stable same-ordinate membership order).
    fn notes<'a>(&'a self, chord: &'a Self::Chord) -> Result<Vec<&'a Self::Head>, String>;
}
#[derive(Debug)]
pub struct HeadCandidate<'a, C, H> {
    pub chord: &'a C,
    pub head: &'a H,
    pub side: Side,
    pub chord_euclidean: f64,
}
fn check_rect(b: Rect) -> Result<(), Error> {
    if b[2] <= 0 || b[3] <= 0 || b.iter().any(|v| v.unsigned_abs() > 4_000_000) {
        Err(Error::InvalidBounds)
    } else {
        Ok(())
    }
}
fn center(b: Rect) -> Point {
    [(b[0] + b[2] / 2) as f64, (b[1] + b[3] / 2) as f64]
}
/// Source Line2D.ptSegDist uses its projection algorithm rather than distance to Head center.
fn segment_distance(a: Point, b: Point, point: Point) -> f64 {
    let d = sub(b, a);
    let mut p = sub(point, a);
    let dot = p[0] * d[0] + p[1] * d[1];
    let proj = if dot <= 0. {
        0.
    } else {
        p = sub(d, p);
        let reverse = p[0] * d[0] + p[1] * d[1];
        if reverse <= 0. {
            0.
        } else {
            reverse * reverse / sq(d)
        }
    };
    let distance = sq(p) - proj;
    (if distance < 0. { 0. } else { distance }).sqrt()
}
/// Return every qualified chord's best real Head. This deliberately does not choose a Slur.
/// Common-chord removal, chord ranking, mirror/orphan rules and clump selection must follow.
pub type LookupCandidates<'a, C, H> = Result<Vec<HeadCandidate<'a, C, H>>, Error>;
pub fn lookup_chords<'a, O: ChordOwners>(
    owners: &'a O,
    curve: Curve,
    geometry: &LookupGeometry,
    side: Side,
    chords: &[&'a O::Chord],
) -> LookupCandidates<'a, O::Chord, O::Head> {
    let index = side.index();
    let area = &geometry.area[index];
    let target = geometry.target[index];
    let end = geometry.endpoint[index];
    let mut found = Vec::new();
    for &chord in chords {
        let box_ = owners.chord_bounds(chord).map_err(Error::Owner)?;
        check_rect(box_)?;
        if !area
            .intersects_rect(box_.map(|v| v as f64))
            .map_err(Error::Area)?
        {
            continue;
        }
        let mut best = None;
        let mut best_dist = f64::MAX;
        for head in owners.notes(chord).map_err(Error::Owner)? {
            let b = owners.head_bounds(head).map_err(Error::Owner)?;
            check_rect(b)?;
            let c = center(b);
            if !geometry.horizontal && !area.contains(c).map_err(Error::Area)? {
                continue;
            }
            let refpoint = [c[0], (b[1] + if curve.above { b[3] - 1 } else { 0 }) as f64];
            let r = sub(refpoint, end);
            let bis = geometry.bisector_unit;
            if r[0] * bis[0] + r[1] * bis[1] <= 0. {
                continue;
            }
            let dist = sq(sub(c, target));
            if dist < best_dist {
                best_dist = dist;
                best = Some(head);
            }
        }
        if let Some(head) = best {
            let x = (box_[0] + box_[2] / 2) as f64;
            let chord_euclidean =
                segment_distance([x, box_[1] as f64], [x, (box_[1] + box_[3]) as f64], target);
            found.push(HeadCandidate {
                chord,
                head,
                side,
                chord_euclidean,
            });
        }
    }
    Ok(found)
}

/// Pair-selection access must use live Chord/Staff/Stem/Beam identities.
/// An unsupported orphan-owner boundary returns an error, never an invented legal orphan.
pub trait PairOwners: ChordOwners {
    type Staff;
    fn same_chord(&self, a: &Self::Chord, b: &Self::Chord) -> bool;
    fn chord_center_2d(&self, chord: &Self::Chord) -> Result<Point, String>;
    fn mirror<'a>(&'a self, chord: &'a Self::Chord) -> Result<Option<&'a Self::Chord>, String>;
    fn top_staff<'a>(&'a self, chord: &'a Self::Chord) -> Result<&'a Self::Staff, String>;
    fn same_staff(&self, a: &Self::Staff, b: &Self::Staff) -> bool;
    fn staff_id_compare(&self, a: &Self::Staff, b: &Self::Staff) -> i32;
    fn stem_direction(&self, chord: &Self::Chord) -> Result<i32, String>;
    fn beam_stem_relation_count(&self, chord: &Self::Chord) -> Result<usize, String>;
    fn can_be_orphan(&self, curve: Curve, side: Side) -> Result<bool, String>;
}
pub type SelectedPair<'a, 'o, C, H> = [Option<&'a HeadCandidate<'o, C, H>>; 2];
/// Selected references point to the same pending lookup candidates. A later relation-owning
/// wrapper must retain these candidates' one-time allocated relations through tie mirror switches.
/// Equal cross-Chord euclidean values preserve supplied enumeration; portable parity of the
/// original HashMap value order is unproved and must not be replaced with source-ID ordering.
pub type PairSelection<'a, 'o, C, H> = Result<Option<SelectedPair<'a, 'o, C, H>>, Error>;
pub fn select_pair<'a, 'o, O: PairOwners>(
    owners: &'o O,
    curve: Curve,
    geometry: &LookupGeometry,
    manual: bool,
    left: &'a [HeadCandidate<'o, O::Chord, O::Head>],
    right: &'a [HeadCandidate<'o, O::Chord, O::Head>],
) -> PairSelection<'a, 'o, O::Chord, O::Head> {
    let mut lists = [
        left.iter().collect::<Vec<_>>(),
        right.iter().collect::<Vec<_>>(),
    ];
    let common = left
        .iter()
        .filter(|l| right.iter().any(|r| owners.same_chord(l.chord, r.chord)))
        .collect::<Vec<_>>();
    for link in common {
        let center = owners.chord_center_2d(link.chord).map_err(Error::Owner)?;
        let remove = if point_distance(center, geometry.target[0])
            > point_distance(center, geometry.target[1])
        {
            0
        } else {
            1
        };
        lists[remove].retain(|r| !owners.same_chord(r.chord, link.chord));
    }
    let mut selected = [None, None];
    let mut mirrors = [None, None];
    for side in 0..2 {
        lists[side].sort_by(|a, b| a.chord_euclidean.total_cmp(&b.chord_euclidean));
        if let Some(&best) = lists[side].first() {
            let mirror = owners.mirror(best.chord).map_err(Error::Owner)?;
            if mirror.is_some_and(|m| lists[side].iter().any(|v| owners.same_chord(v.chord, m))) {
                mirrors[side] = Some(best);
            } else {
                selected[side] = Some(best);
            }
        }
    }
    for side in 0..2 {
        if let Some(link) = mirrors[side] {
            let mirror = owners
                .mirror(link.chord)
                .map_err(Error::Owner)?
                .ok_or_else(|| Error::Owner("slur_actual_mirror_changed".into()))?;
            let other = *lists[side]
                .iter()
                .find(|v| owners.same_chord(v.chord, mirror))
                .ok_or_else(|| Error::Owner("slur_mirror_candidate_required".into()))?;
            let keep = if let Some(opposite) = selected[1 - side] {
                let other_staff = owners.top_staff(opposite.chord).map_err(Error::Owner)?;
                let staff = owners.top_staff(link.chord).map_err(Error::Owner)?;
                let direction = owners.stem_direction(link.chord).map_err(Error::Owner)?;
                if !owners.same_staff(staff, other_staff) {
                    direction * owners.staff_id_compare(other_staff, staff) > 0
                } else {
                    direction
                        == owners
                            .stem_direction(opposite.chord)
                            .map_err(Error::Owner)?
                }
            } else {
                owners
                    .beam_stem_relation_count(link.chord)
                    .map_err(Error::Owner)?
                    <= owners
                        .beam_stem_relation_count(other.chord)
                        .map_err(Error::Owner)?
            };
            selected[side] = Some(if keep { link } else { other });
        }
    }
    if let (Some(left), Some(right)) = (selected[0], selected[1])
        && owners
            .mirror(left.chord)
            .map_err(Error::Owner)?
            .is_some_and(|m| owners.same_chord(m, right.chord))
    {
        return Ok(None);
    }
    if selected.iter().all(Option::is_none) {
        return Ok(None);
    }
    if !manual {
        for side in [Side::Left, Side::Right] {
            if selected[side.index()].is_none()
                && !owners.can_be_orphan(curve, side).map_err(Error::Owner)?
            {
                return Ok(None);
            }
        }
    }
    Ok(Some(selected))
}
