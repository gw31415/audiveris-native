// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected frozen Clef/Key source ownership at the MEASURES registration boundary.
use crate::{
    beam_dispatch::{self, Registry},
    native_headers::{NativeStaffHeader, NativeSystemHeaders},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, sync::Arc};

pub(crate) struct RegisteredHeader {
    pub(crate) key: Key,
    pub(crate) projection: Value,
}
pub(crate) fn register_clef(
    graph: &mut Graph,
    header: &NativeStaffHeader,
) -> Result<Option<RegisteredHeader>, String> {
    let Some(source) = header.clef() else {
        return Ok(None);
    };
    let staff = header.staff();
    if !source.in_final_sig()
        || source.body()["staff_id"] != staff.id
        || source.body()["kind"] != "ClefInter"
        || source.body()["shape"].as_str().is_none()
        || source.body()["frozen"] != true
        || source.body()["removed"] == true
    {
        return Err("music_native_selected_frozen_Clef_required".into());
    }
    let key = Key {
        system: staff.system,
        phase: Phase::MusicHeader,
        local: usize::try_from(staff.id).map_err(|_| "music_header_Staff_identity_overflow")?,
    };
    let mut next = graph.clone();
    if next.id(key).is_some() {
        return Err("music_header_fresh_Clef_binding_required".into());
    }
    let mut body = source.body().clone();
    body["source_header_id"] = json!(source.id());
    // The source already owns this nonnull Staff; retain its raw field.
    // A lazy null Staff requires its own source getter boundary.
    body["header_stop"] = json!(header.stop());
    let id = next.vertex(key, body, false)?;
    next.bind_native_header_inter(key, header, source.clone())?;
    // Original header Clef.added occurred before Measure creation. No late replay.
    *graph = next;
    Ok(Some(RegisteredHeader {
        key,
        projection: json!({"id":id,"origin":key,"staff_id":staff.id}),
    }))
}

pub(crate) fn register_key(
    graph: &mut Graph,
    registry: &mut Registry,
    header: &NativeStaffHeader,
    system: &NativeSystemHeaders,
    clef_key: Key,
) -> Result<Option<RegisteredHeader>, String> {
    let Some(source) = header.key() else {
        return Ok(None);
    };
    let clef = header
        .clef()
        .ok_or("music_header_key_without_actual_Clef_pending")?;
    if !Arc::ptr_eq(graph.native_header_source(clef_key)?, clef)
        || !system.staffs().iter().any(|h| std::ptr::eq(h, header))
        || clef_key.system != system.system().id
    {
        return Err("music_header_actual_Clef_Key_owners_required".into());
    }
    let body = source.body();
    let fifths = body["fifths"]
        .as_i64()
        .filter(|f| (-7..=7).contains(f) && *f != 0)
        .ok_or("music_header_natural_cancel_or_invalid_fifths_pending")?;
    let members = source.members();
    if !source.in_final_sig()
        || body["staff_id"] != header.staff().id
        || body["kind"] != "KeyInter"
        || body["frozen"] != true
        || body["removed"] == true
        || !body["glyph"].is_null()
        || members.len() != fifths.unsigned_abs() as usize
    {
        return Err("music_header_actual_frozen_Key_members_required".into());
    }
    let support = system
        .relations()
        .iter()
        .filter(|r| {
            r.body()["kind"] == "ClefKeyRelation"
                && Arc::ptr_eq(r.source(), clef)
                && Arc::ptr_eq(r.target(), source)
        })
        .collect::<Vec<_>>();
    if support.len() != 1 {
        return Err("music_header_actual_Clef_Key_support_required".into());
    }
    let shape = if fifths > 0 { "SHARP" } else { "FLAT" };
    let mut ids = BTreeSet::from([source.id()]);
    let mut containment = Vec::new();
    for member in members {
        let b = member.body();
        if !member.in_final_sig()
            || b["staff_id"] != header.staff().id
            || b["kind"] != "KeyAlterInter"
            || b["shape"] != shape
            || b["frozen"] != true
            || b["removed"] == true
            || !ids.insert(member.id())
        {
            return Err("music_header_actual_KeyAlter_required".into());
        }
        let edge = system
            .relations()
            .iter()
            .filter(|r| {
                r.body()["kind"] == "Containment"
                    && Arc::ptr_eq(r.source(), source)
                    && Arc::ptr_eq(r.target(), member)
            })
            .collect::<Vec<_>>();
        if edge.len() != 1 {
            return Err("music_header_actual_Key_member_relation_required".into());
        }
        containment.push(edge[0].clone());
    }
    let mut next = graph.clone();
    let mut physical = registry.clone();
    let key = Key {
        system: clef_key.system,
        phase: Phase::MusicKey,
        local: usize::try_from(source.id()).map_err(|_| "music_header_Key_identity_overflow")?,
    };
    if next.id(key).is_some() {
        return Err("music_header_fresh_Key_binding_required".into());
    }
    let mut registered = Vec::new();
    let mut rows = Vec::new();
    for member in members {
        let member_key = Key {
            local: usize::try_from(member.id())
                .map_err(|_| "music_header_KeyAlter_identity_overflow")?,
            ..key
        };
        if next.id(member_key).is_some() {
            return Err("music_header_fresh_KeyAlter_binding_required".into());
        }
        let mut data = member.body().clone();
        data["source_header_id"] = json!(member.id());
        data["glyph_id"] =
            json!(physical.register(&beam_dispatch::glyph_from_json(&data["glyph"])?)?);
        next.vertex(member_key, data, false)?;
        next.bind_native_header_inter(member_key, header, member.clone())?;
        rows.push(next.retained_inter(member_key)?.0.clone());
        registered.push(member_key);
    }
    // KeyBuilder inserts actual sibling supports before creating the Key ensemble.
    for relation in system
        .relations()
        .iter()
        .filter(|relation| relation.body()["kind"] == "KeyAltersRelation")
    {
        let left = members
            .iter()
            .position(|member| Arc::ptr_eq(member, relation.source()));
        let right = members
            .iter()
            .position(|member| Arc::ptr_eq(member, relation.target()));
        match (left, right) {
            (Some(left), Some(right)) if left != right => {
                next.register_native_header_relation(
                    registered[left],
                    registered[right],
                    relation.clone(),
                    Phase::MusicKey,
                )?;
            }
            (None, None) => {}
            _ => return Err("music_header_actual_KeyAlter_sibling_support_required".into()),
        }
    }
    let mut data = body.clone();
    data["source_header_id"] = json!(source.id());
    data["member_ids"] = json!(registered.iter().map(|k| next.id(*k)).collect::<Vec<_>>());
    data["members"] = json!(rows);
    let id = next.vertex(key, data, false)?;
    next.bind_native_header_inter(key, header, source.clone())?;
    for (member, relation) in registered.into_iter().zip(containment) {
        next.register_native_header_relation(key, member, relation, Phase::MusicKey)?;
    }
    next.register_native_header_relation(clef_key, key, support[0].clone(), Phase::MusicKey)?;
    // All actual owner/relation bindings validate before physical publication.
    registry.commit_extension(physical)?;
    *graph = next;
    Ok(Some(RegisteredHeader {
        key,
        projection: json!({"id":id,"origin":key,
        "source_header_id":source.id(),"staff_id":header.staff().id,"fifths":fifths,
        "member_ids":rows.iter().map(|n|&n["id"]).collect::<Vec<_>>()}),
    }))
}

#[cfg(test)]
pub(crate) mod image_tests;
#[cfg(test)]
pub(crate) mod tests;
