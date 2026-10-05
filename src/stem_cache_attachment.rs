// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit head/stump relations on an already registered, retriever-owned
//! cached stem. Automatic items/selection, cleaner and epilog remain separate.
use crate::{
    beam_dispatch::{self, Registry},
    head_stem_relation::{self, ConnectionInput},
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_attachment::HeadRequest,
    stem_cache_sig::{CachedStem, Retriever},
};
use audiveris_staff_removal::GlyphPixels;
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use std::sync::Arc;
fn owned(
    retriever: &mut Retriever,
    registry: &Registry,
    graph: &Graph,
    stem: &Arc<CachedStem>,
) -> Result<(usize, u64, Value), String> {
    let state = stem.snapshot()?;
    if state["in_sig"] != true || state["removed"] == true {
        return Err("live_registered_cached_stem_required".into());
    }
    // register on an already registered object is a read-only ownership, registry,
    // cache and SIG consistency check. It does not allocate or rebuild any glyph.
    let mut checked = graph.clone();
    let id = retriever.register(registry, &mut checked, stem)?;
    if state["id"] != id || checked.snapshot() != graph.snapshot() {
        return Err("stale_cached_attachment_SIG".into());
    }
    let snapshot = graph.snapshot();
    let record = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == id)
        .ok_or("missing_cached_attachment_stem")?;
    let key: Key = serde_json::from_value(record["origin"].clone())
        .map_err(|_| "invalid_cached_attachment_origin")?;
    if key.phase != Phase::Stem || key.local != stem.glyph_id() {
        return Err("invalid_cached_attachment_origin".into());
    }
    Ok((id, key.system, snapshot))
}
fn boundary(
    snapshot: &Value,
    system: u64,
) -> Result<(Vec<Value>, Vec<HeadStemAttachment>), String> {
    let inters = snapshot["inters"]
        .as_array()
        .ok_or("invalid_cached_attachment_snapshot")?;
    let mut records = Vec::new();
    for v in inters {
        let key: Key = serde_json::from_value(v["origin"].clone())
            .map_err(|_| "invalid_cached_attachment_origin")?;
        if key.system == system && key.phase == Phase::Stem {
            let mut r = v.clone();
            r["id"] = json!(key.local);
            records.push(r);
        }
    }
    let mut attachments = Vec::new();
    for e in snapshot["relations"].as_array().unwrap() {
        if e["kind"] != "HeadStemRelation" || e["system_id"] != system {
            continue;
        }
        let source = inters
            .iter()
            .find(|v| v["id"] == e["source"])
            .ok_or("missing_cached_attachment_head")?;
        let target = inters
            .iter()
            .find(|v| v["id"] == e["target"])
            .ok_or("missing_cached_attachment_stem")?;
        let head: Key = serde_json::from_value(source["origin"].clone())
            .map_err(|_| "invalid_cached_attachment_origin")?;
        let stem: Key = serde_json::from_value(target["origin"].clone())
            .map_err(|_| "invalid_cached_attachment_origin")?;
        attachments.push(HeadStemAttachment {
            local: e["local_relation_id"]
                .as_u64()
                .ok_or("invalid_cached_attachment_relation_id")? as usize,
            head,
            stem_local: stem.local,
            relation: e.clone(),
        });
    }
    Ok((records, attachments))
}
pub struct Contact<'a> {
    pub request: &'a HeadRequest,
    pub stump: Option<&'a GlyphPixels>,
    pub interline: usize,
    pub bank: &'a Bank,
}
/// Reject malformed/foreign/stale requests without changing registry/cache/SIG.
/// Geometric rejection adds no relation; success keeps existing STEMS and prior
/// phases and computes original measured physical-length consistency.
pub fn connect(
    retriever: &mut Retriever,
    registry: &Registry,
    graph: &mut Graph,
    stem: &Arc<CachedStem>,
    contact: Contact<'_>,
) -> Result<Option<Value>, String> {
    let (id, system, snapshot) = owned(retriever, registry, graph, stem)?;
    let request = contact.request;
    if request.local_relation == 0
        || request.head.system != system
        || request.head.phase != Phase::Head
        || request.stem_local != stem.glyph_id()
    {
        return Err("invalid_cached_head_request".into());
    }
    let head_id = graph
        .id(request.head)
        .ok_or("missing_cached_attachment_head")?;
    let head = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == head_id)
        .ok_or("removed_cached_attachment_head")?;
    if head["kind"] != "HeadInter" || head["manual"] == true || head["is_manual"] == true {
        return Err("unsupported_cached_attachment_head".into());
    }
    let (records, mut attachments) = boundary(&snapshot, system)?;
    if attachments
        .iter()
        .any(|e| e.local == request.local_relation)
    {
        return Err("duplicate_cached_attachment_relation".into());
    }
    let record = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == id)
        .unwrap();
    let bounds = contact
        .stump
        .map(|g| {
            beam_dispatch::centroid(g)?;
            Ok::<[i32; 4], String>([
                g.left,
                g.top,
                i32::try_from(g.width).map_err(|_| "invalid_cached_stump_width")?,
                i32::try_from(g.height).map_err(|_| "invalid_cached_stump_height")?,
            ])
        })
        .transpose()?;
    let relation = head_stem_relation::check_relation_with_consistency(
        contact.bank,
        request.point_size,
        ConnectionInput {
            shape: head["shape"].as_str().ok_or("invalid_cached_head_shape")?,
            bounds: serde_json::from_value(head["bounds"].clone())
                .map_err(|_| "invalid_cached_head_bounds")?,
            stem: serde_json::from_value(record["median"].clone())
                .map_err(|_| "invalid_cached_stem_median")?,
            glyph_bounds: bounds,
            vertical: request.vertical,
            interline: contact.interline,
            profile: request.profile,
            learned_dx: request.learned_dx,
        },
    )?;
    if let Some(r) = relation.as_ref() {
        attachments.push(HeadStemAttachment {
            local: request.local_relation,
            head: request.head,
            stem_local: request.stem_local,
            relation: r.clone(),
        });
        let mut next = graph.clone();
        next.sync_stems(system, &records, &attachments)?;
        *graph = next;
    }
    Ok(relation)
}
/// Explicit original edge-removal callback boundary, preserving the registered
/// cached stem and every unrelated relation/phase. Unknown/foreign edges refuse.
pub fn disconnect(
    retriever: &mut Retriever,
    registry: &Registry,
    graph: &mut Graph,
    stem: &Arc<CachedStem>,
    local: usize,
) -> Result<(), String> {
    let (_, system, snapshot) = owned(retriever, registry, graph, stem)?;
    let (records, mut attachments) = boundary(&snapshot, system)?;
    let i = attachments
        .iter()
        .position(|e| e.local == local && e.stem_local == stem.glyph_id())
        .ok_or("missing_owned_cached_relation")?;
    attachments.remove(i);
    let mut next = graph.clone();
    next.sync_stems(system, &records, &attachments)?;
    *graph = next;
    Ok(())
}
