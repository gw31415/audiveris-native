// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest::{measure_groups, validate_measure_chord};
use crate::{
    music_measures::{Chord, ChordKind, OwnedMusic},
    shared_sig::{Graph, Key},
};
use serde_json::{Value, json};

struct RestPolygon {
    group: Key,
    polygon: [[i32; 2]; 4],
    left_bounds: [i32; 4],
    right_bounds: [i32; 4],
    main_median: Option<[f64; 4]>,
}
struct RestCandidate {
    rest: Key,
    center: [i32; 2],
    nearest: (Key, f64),
}
fn point(p: Option<[i64; 2]>) -> Result<[i32; 2], String> {
    p.ok_or("music_rest_actual_chord_geometry_required")?
        .map(i32::try_from)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map(|v| [v[0], v[1]])
        .map_err(|_| "music_rest_actual_chord_geometry_required".into())
}
fn rectangle(p: [i64; 4]) -> Result<[i32; 4], String> {
    let v = p
        .map(i32::try_from)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "music_rest_actual_chord_geometry_required")?;
    Ok([v[0], v[1], v[2], v[3]])
}
fn intersection(m: [f64; 4], h: [i32; 2], t: [i32; 2]) -> Result<[i32; 2], String> {
    let [x1, y1, x2, y2] = m;
    let ([x3, y3], [x4, y4]) = (h.map(f64::from), t.map(f64::from));
    let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let (a, b) = (x1 * y2 - y1 * x2, x3 * y4 - y3 * x4);
    let p = [
        (a * (x3 - x4) - (x1 - x2) * b) / den,
        (a * (y3 - y4) - (y1 - y2) * b) / den,
    ];
    if p.iter().any(|v| !v.is_finite() || v.abs() > 16_000_000.) {
        return Err("music_rest_actual_chord_geometry_required".into());
    }
    Ok(p.map(|v| v.round_ties_even() as i32))
}
fn overlap(a: [i32; 4], b: [i32; 4]) -> bool {
    i64::from(a[0]) + i64::from(a[2]) > i64::from(b[0])
        && i64::from(b[0]) + i64::from(b[2]) > i64::from(a[0])
}
impl OwnedMusic {
    /// Original lookup geometry on an explicit real group/Chord pair. This
    /// query retains clock/cache owners; full clear/detect preparation remains
    /// required before the ordinary selected rhythm path can call lookup.
    pub fn interleaved_rest_lookup_geometry(
        &mut self,
        number: usize,
        group: Key,
        left: Key,
        right: Key,
        candidates: &[Key],
    ) -> Result<Value, String> {
        let mut graph = self.owner.graph.clone();
        let query = self.prepare_rest_lookup(&mut graph, number, group, left, right, candidates)?;
        self.owner.graph = graph;
        Ok(query)
    }
    fn prepare_rest_polygon(
        &self,
        graph: &mut Graph,
        number: usize,
        group: Key,
        left: Key,
        right: Key,
    ) -> Result<RestPolygon, String> {
        let mi = number
            .checked_sub(1)
            .filter(|mi| *mi < self.measures.len())
            .ok_or("music_rest_actual_measure_required")?;
        let groups = measure_groups(graph, &self.measures[mi], &self.chords, mi)?;
        let gid = graph.id(group).ok_or("music_rest_actual_group_required")?;
        if !groups.contains(&gid) {
            return Err("music_rest_actual_measure_group_required".into());
        }
        let actual = |key| -> Result<&Chord, String> {
            let ch = self
                .chords
                .iter()
                .find(|c| c.key == key)
                .ok_or("music_rest_actual_chord_required")?;
            let measure = self
                .measures
                .get(ch.measure)
                .ok_or("music_rest_actual_measure_required")?;
            validate_measure_chord(graph, measure, ch)?;
            if measure.system != group.system {
                return Err("music_rest_actual_chord_required".into());
            }
            Ok(ch)
        };
        let l = actual(left)?;
        let r = actual(right)?;
        let li = l.measure;
        let ri = r.measure;
        let lr = chord_geometry(graph, l)?;
        let rr = chord_geometry(graph, r)?;
        let lb = rectangle(
            lr.bounds
                .ok_or("music_rest_actual_chord_geometry_required")?,
        )?;
        let rb = rectangle(
            rr.bounds
                .ok_or("music_rest_actual_chord_geometry_required")?,
        )?;
        let (mut lh, lt, mut rh, rt) = (
            point(lr.head)?,
            point(lr.tail)?,
            point(rr.head)?,
            point(rr.tail)?,
        );
        let ld = (i64::from(lh[1]) - i64::from(lt[1])).signum();
        let rd = (i64::from(rh[1]) - i64::from(rt[1])).signum();
        let median = graph.main_rest_median(group)?;
        let staff = |mi: usize| {
            self.owner
                .staffs
                .iter()
                .find(|s| s.id == self.measures[mi].staff && s.system == self.measures[mi].system)
                .ok_or("music_actual_staff_owner_required")
        };
        let ls = staff(li)?;
        let rs = staff(ri)?;
        let polygon = if ld == rd {
            if std::sync::Arc::ptr_eq(ls, rs) {
                let (ml, mr) = (ls.middle_at(lh[0])?, ls.middle_at(rh[0])?);
                if ld * (i64::from(ml) - i64::from(lh[1])) > 0 {
                    lh[1] = ml;
                }
                if rd * (i64::from(mr) - i64::from(rh[1])) > 0 {
                    rh[1] = mr;
                }
            }
            [rh, rt, lt, lh]
        } else {
            let (mut uh, ut, mut dh, dt, us) = (
                if ld < 0 { lh } else { rh },
                if ld < 0 { lt } else { rt },
                if ld > 0 { lh } else { rh },
                if ld > 0 { lt } else { rt },
                if ld < 0 { ls } else { rs },
            );
            let mid_x = i32::try_from((i64::from(lh[0]) + i64::from(rh[0])) / 2)
                .map_err(|_| "music_rest_actual_chord_geometry_required")?;
            let mid = us.middle_at(mid_x)?;
            if uh[1] > mid {
                uh[1] = mid;
                // With zero left direction both original variables alias the
                // same right Point; extending one must also extend the other.
                if ld == 0 {
                    dh = uh;
                }
            }
            let m = median.ok_or("music_rest_actual_beam_median_required")?;
            let um = intersection(m, uh, ut)?;
            let dm = intersection(m, dh, dt)?;
            let last_y = dm[1]
                .checked_sub(
                    um[1]
                        .checked_sub(uh[1])
                        .ok_or("music_rest_actual_chord_geometry_required")?,
                )
                .ok_or("music_rest_actual_chord_geometry_required")?;
            [uh, um, dm, [dm[0], last_y]]
        };
        Ok(RestPolygon {
            group,
            polygon,
            left_bounds: lb,
            right_bounds: rb,
            main_median: median,
        })
    }
    fn prepare_rest_lookup(
        &self,
        graph: &mut Graph,
        number: usize,
        group: Key,
        left: Key,
        right: Key,
        candidates: &[Key],
    ) -> Result<Value, String> {
        let area = self.prepare_rest_polygon(graph, number, group, left, right)?;
        let mut rows = Vec::new();
        for &key in candidates {
            let (candidate, intersects, x_overlap, id) = self.rest_candidate(graph, &area, key)?;
            rows.push(json!({"chord_id":id,"intersects":intersects,"x_overlap":x_overlap,"nearest_beam_id":candidate.as_ref().map(|c|graph.id(c.nearest.0).unwrap()),"eligible":candidate.is_some()}));
        }
        Ok(
            json!({"polygon":area.polygon,"main_median":area.main_median,"candidates":rows,"rest_detection_complete":false}),
        )
    }
    fn rest_candidate(
        &self,
        graph: &Graph,
        area: &RestPolygon,
        key: Key,
    ) -> Result<(Option<RestCandidate>, bool, bool, usize), String> {
        let chord = self
            .chords
            .iter()
            .find(|c| c.key == key)
            .ok_or("music_rest_actual_chord_required")?;
        validate_measure_chord(
            graph,
            self.measures
                .get(chord.measure)
                .ok_or("music_rest_actual_measure_required")?,
            chord,
        )?;
        if chord.kind != ChordKind::Rest || key.system != area.group.system {
            return Err("music_rest_actual_rest_chord_required".into());
        }
        let bounds = rectangle(chord.bounds)?;
        let intersects = audiveris_area::polygon_intersects_rect(&area.polygon, bounds)?;
        let x_overlap = overlap(bounds, area.left_bounds) || overlap(bounds, area.right_bounds);
        let candidate = if intersects && !x_overlap {
            let (rest, center) = actual_rest_note(graph, chord)?;
            graph
                .nearest_rest_beam(area.group, center, self.owner.sheet_skew)?
                .map(|nearest| RestCandidate {
                    rest,
                    center,
                    nearest,
                })
        } else {
            None
        };
        Ok((candidate, intersects, x_overlap, chord.id))
    }
    /// Original plain lookup on an explicit adjacent Chord pair and actual rest
    /// candidates. Complete clear/detect and tuplet preparation remain pending.
    pub fn lookup_interleaved_rests(
        &mut self,
        number: usize,
        group: Key,
        left: Key,
        right: Key,
        candidates: &[Key],
        white: &[Key],
    ) -> Result<Vec<usize>, String> {
        let mut graph = self.owner.graph.clone();
        let area = self.prepare_rest_polygon(&mut graph, number, group, left, right)?;
        for &key in white {
            let chord = self
                .chords
                .iter()
                .find(|c| c.key == key && c.kind == ChordKind::Rest)
                .ok_or("music_rest_actual_rest_chord_required")?;
            validate_measure_chord(
                &graph,
                self.measures
                    .get(chord.measure)
                    .ok_or("music_rest_actual_measure_required")?,
                chord,
            )?;
            if key.system != group.system {
                return Err("music_rest_actual_rest_chord_required".into());
            }
        }
        let mut added = Vec::new();
        let mut seen = Vec::new();
        'candidate: for &key in candidates {
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            let Some(candidate) = self.rest_candidate(&graph, &area, key)?.0 else {
                continue;
            };
            let (beam, distance) = candidate.nearest;
            for (eid, _, rest) in graph.beam_rest_relations(beam)? {
                if graph.rest_contains_candidate_vertical(
                    rest,
                    candidate.center,
                    self.owner.sheet_skew,
                )? {
                    let owners = self
                        .chords
                        .iter()
                        .filter(|ch| {
                            ch.kind == ChordKind::Rest
                                && ch.members.iter().any(|n| {
                                    serde_json::from_value::<Key>(n["origin"].clone()).ok()
                                        == Some(rest)
                                })
                        })
                        .collect::<Vec<_>>();
                    if owners.len() != 1 {
                        return Err("music_rest_actual_chord_required".into());
                    }
                    let sibling = owners[0];
                    validate_measure_chord(
                        &graph,
                        self.measures
                            .get(sibling.measure)
                            .ok_or("music_rest_actual_measure_required")?,
                        sibling,
                    )?;
                    actual_rest_note(&graph, sibling)?;
                    if white.contains(&sibling.key)
                        || distance > f64::from(graph.beam_rest_distance(eid)?)
                    {
                        continue 'candidate;
                    }
                    graph.remove_lookup_beam_rest(eid)?;
                }
            }
            // A later competitor rejection keeps earlier removals in this same
            // lookup. Only execution errors discard the whole staged operation.
            for (eid, _, _) in graph.beam_rest_relations(candidate.rest)? {
                if distance > f64::from(graph.beam_rest_distance(eid)?) {
                    continue 'candidate;
                }
                graph.remove_lookup_beam_rest(eid)?;
            }
            added.push(graph.add_owned_beam_rest(
                beam,
                candidate.rest,
                false,
                self.owner.sheet_skew,
                self.owner.interline,
            )?);
        }
        self.owner.graph = graph;
        Ok(added)
    }
}
fn chord_geometry(
    graph: &mut Graph,
    chord: &Chord,
) -> Result<crate::shared_sig::ChordRead, String> {
    if chord.kind == ChordKind::Head {
        return graph.retained_chord_getters(chord.key);
    }
    let (_, center) = actual_rest_note(graph, chord)?;
    Ok(crate::shared_sig::ChordRead {
        stem: None,
        bounds: Some(chord.bounds),
        head: Some(center.map(i64::from)),
        tail: Some(center.map(i64::from)),
    })
}

pub(super) fn actual_rest_note(graph: &Graph, chord: &Chord) -> Result<(Key, [i32; 2]), String> {
    let snapshot = graph.snapshot();
    let mut members = Vec::new();
    for member in &chord.members {
        let key: Key = serde_json::from_value(member["origin"].clone())
            .map_err(|_| "music_rest_actual_note_required")?;
        let id = graph.id(key).ok_or("music_rest_actual_note_required")?;
        let n = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("music_rest_actual_note_required")?;
        if n["kind"] != "RestInter"
            || member["id"] != id
            || !snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "Containment" && e["source"] == chord.id && e["target"] == id)
        {
            return Err("music_rest_actual_note_required".into());
        }
        let b: [i32; 4] = serde_json::from_value(n["bounds"].clone())
            .map_err(|_| "music_rest_actual_note_bounds_required")?;
        let center = [
            b[0].checked_add(b[2] / 2)
                .ok_or("music_rest_actual_note_bounds_required")?,
            b[1].checked_add(b[3] / 2)
                .ok_or("music_rest_actual_note_bounds_required")?,
        ];
        members.push((key, center));
    }
    members.sort_by_key(|(_, p)| std::cmp::Reverse(p[1]));
    members
        .first()
        .copied()
        .ok_or("music_rest_actual_note_required".into())
}
