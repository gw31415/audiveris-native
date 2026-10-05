// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transaction boundary for explicitly selected stem parts/head requests.
//! No automatic StemBuilder selection, intrinsic grading, chords or epilog.
use crate::{
    beam_dispatch::Registry,
    head_stem_relation::{self, ConnectionInput, VerticalSide},
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_glyph,
};
use audiveris_staff_removal::GlyphPixels;
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub struct SelectedStem<'a> {
    pub local: usize,
    pub parts: &'a [GlyphPixels],
    pub grade: f64,
}
#[derive(Clone)]
pub struct HeadRequest {
    pub local_relation: usize,
    pub head: Key,
    pub stem_local: usize,
    pub point_size: u16,
    pub vertical: VerticalSide,
    pub profile: u8,
    /// Sheet-level learned means for this shape, never per-head seed offsets.
    pub learned_dx: [Option<f64>; 2],
}
/// Original HeadLinker.CLinker supplies its corner stump, not the full stem
/// union. A null stump uses the template reference point. The physical stump
/// is caller-selected; this boundary does not extract/select corner stumps.
pub struct StumpRequest<'a> {
    pub request: HeadRequest,
    pub stump: Option<&'a GlyphPixels>,
}
/// Complete replacement of one system's explicitly selected STEM boundary.
/// All physical acquisition, relation checks and SIG updates commit together.
/// A geometric rejection is recorded without adding an edge; malformed/stale
/// inputs leave both indexes unchanged. Caller supplies the certified bank,
/// staff point sizes and already assessed intrinsic stem grades.
pub fn register_selected(
    system: u64,
    interline: usize,
    bank: &Bank,
    registry: &mut Registry,
    graph: &mut Graph,
    stems: &[SelectedStem<'_>],
    requests: &[HeadRequest],
) -> Result<Value, String> {
    register_boundary(
        (system, interline, false),
        bank,
        registry,
        graph,
        stems,
        requests,
        None,
    )
}
/// Explicit original length-based support population on selected physical
/// stems. Main interline and small head shape determine consistency; this is
/// separate from automatic selection, intrinsic/contextual grades and epilog.
pub fn register_selected_with_consistency(
    system: u64,
    interline: usize,
    bank: &Bank,
    registry: &mut Registry,
    graph: &mut Graph,
    stems: &[SelectedStem<'_>],
    requests: &[HeadRequest],
) -> Result<Value, String> {
    register_boundary(
        (system, interline, true),
        bank,
        registry,
        graph,
        stems,
        requests,
        None,
    )
}
/// Explicit nullable corner-stump contact on selected stem unions, followed by
/// original measured physical length consistency. Existing single-part APIs
/// retain their certified contact behavior. Multiple-part head requests must
/// use this explicit boundary; inferring contact from union bounds hides gaps.
pub fn register_selected_with_stumps(
    system: u64,
    interline: usize,
    bank: &Bank,
    registry: &mut Registry,
    graph: &mut Graph,
    stems: &[SelectedStem<'_>],
    requests: &[StumpRequest<'_>],
) -> Result<Value, String> {
    if requests.len() > 1_000_000 {
        return Err("selected_stump_request_budget".into());
    }
    let mut direct = Vec::with_capacity(requests.len());
    let mut bounds = Vec::with_capacity(requests.len());
    for contact in requests {
        direct.push(contact.request.clone());
        bounds.push(if let Some(stump) = contact.stump {
            crate::beam_dispatch::centroid(stump)?;
            Some([
                stump.left,
                stump.top,
                i32::try_from(stump.width).map_err(|_| "invalid_selected_stump_width")?,
                i32::try_from(stump.height).map_err(|_| "invalid_selected_stump_height")?,
            ])
        } else {
            None
        });
    }
    register_boundary(
        (system, interline, true),
        bank,
        registry,
        graph,
        stems,
        &direct,
        Some(&bounds),
    )
}
fn register_boundary(
    scope: (u64, usize, bool),
    bank: &Bank,
    registry: &mut Registry,
    graph: &mut Graph,
    stems: &[SelectedStem<'_>],
    requests: &[HeadRequest],
    stump_bounds: Option<&[Option<[i32; 4]>]>,
) -> Result<Value, String> {
    let (system, interline, populate_consistency) = scope;
    if system == 0
        || !(1..=128).contains(&interline)
        || stems.len() > 100_000
        || requests.len() > 1_000_000
    {
        return Err("invalid_selected_stem_scope_or_budget".into());
    }
    let mut next_registry = registry.clone();
    let mut next_graph = graph.clone();
    let mut records = Vec::new();
    let mut by_local = BTreeMap::new();
    for stem in stems {
        if stem.local == 0 || by_local.contains_key(&stem.local) {
            return Err("duplicate_or_invalid_selected_stem_id".into());
        }
        let glyph_id = stem_glyph::acquire(&mut next_registry, stem.parts)?;
        let glyph = &next_registry.get(glyph_id)?.glyph;
        let mut record = stem_glyph::record(stem.local, glyph, stem.grade)?;
        record["glyph_id"] = json!(glyph_id);
        by_local.insert(stem.local, records.len());
        records.push(record);
    }
    let prior = graph.snapshot();
    let mut ids = BTreeSet::new();
    let mut attachments = Vec::new();
    let mut decisions = Vec::new();
    for (request_index, request) in requests.iter().enumerate() {
        if request.local_relation == 0
            || !ids.insert(request.local_relation)
            || request.head.system != system
            || request.head.phase != Phase::Head
        {
            return Err("invalid_selected_head_stem_request".into());
        }
        let id = graph.id(request.head).ok_or("missing_selected_stem_head")?;
        let head = prior["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .ok_or("removed_selected_stem_head")?;
        if head["kind"] != "HeadInter" || head["manual"] == true || head["is_manual"] == true {
            return Err("unsupported_selected_stem_head".into());
        }
        let stem_index = *by_local
            .get(&request.stem_local)
            .ok_or("missing_selected_stem")?;
        let stem = &records[stem_index];
        let contact = if let Some(bounds) = stump_bounds {
            bounds[request_index]
        } else {
            if stems[stem_index].parts.len() != 1 {
                return Err("multiple_stem_parts_require_explicit_corner_stump".into());
            }
            Some(
                serde_json::from_value(stem["glyph"]["bounds"].clone())
                    .map_err(|_| "invalid_selected_stem_glyph_bounds")?,
            )
        };
        let input = ConnectionInput {
            shape: head["shape"]
                .as_str()
                .ok_or("invalid_selected_stem_head_shape")?,
            bounds: serde_json::from_value(head["bounds"].clone())
                .map_err(|_| "invalid_selected_stem_head_bounds")?,
            stem: serde_json::from_value(stem["median"].clone())
                .map_err(|_| "invalid_selected_stem_median")?,
            glyph_bounds: contact,
            vertical: request.vertical,
            interline,
            profile: request.profile,
            learned_dx: request.learned_dx,
        };
        let relation = if populate_consistency {
            head_stem_relation::check_relation_with_consistency(bank, request.point_size, input)?
        } else {
            head_stem_relation::check_relation(bank, request.point_size, input)?
        };
        decisions.push(json!({"local_relation_id":request.local_relation,"head":request.head,"stem_local":request.stem_local,"relation":relation}));
        if stump_bounds.is_some() {
            decisions.last_mut().unwrap()["stump_bounds"] = json!(contact);
        }
        if let Some(relation) = relation {
            attachments.push(HeadStemAttachment {
                local: request.local_relation,
                head: request.head,
                stem_local: request.stem_local,
                relation,
            });
        }
    }
    next_graph.sync_stems(system, &records, &attachments)?;
    let mut report = json!({"selected_stem_boundary_complete":true,"automatic_stems_complete":false,"stems_complete":false,"heads_complete":false,"musicxml_complete":false,"system_id":system,"scope":"Explicit selected parts and already assessed stem grades; certified head bank/staff sizes; nonmanual geometry/registration only. No automatic STEMS, chords, cleaner or epilog.","physical_stems":records,"head_requests":decisions,"shared_sig":next_graph.snapshot()});
    if populate_consistency {
        report["length_consistency_populated"] = json!(true);
    }
    if stump_bounds.is_some() {
        report["corner_stumps_explicit"] = json!(true);
    }
    *registry = next_registry;
    *graph = next_graph;
    Ok(report)
}
