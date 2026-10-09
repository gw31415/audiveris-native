// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected native Time owners, retained at the shared-SIG registration boundary.
use super::RegisteredHeader;
use crate::{
    beam_dispatch::{self, Registry},
    native_headers::{
        NativeHeaderInter, NativeHeaderRelation, NativeStaffHeader, NativeSystemHeaders,
    },
    shared_sig::{Graph, Key, Phase},
};
use serde_json::json;
use std::{collections::BTreeSet, sync::Arc};

fn require_time_relations(
    system: &NativeSystemHeaders,
    source: &Arc<NativeHeaderInter>,
    allowed: &[Arc<NativeHeaderRelation>],
) -> Result<(), String> {
    for relation in system.relations() {
        let touches_selected = std::iter::once(source)
            .chain(source.members())
            .any(|owner| {
                Arc::ptr_eq(owner, relation.source()) || Arc::ptr_eq(owner, relation.target())
            });
        if touches_selected && !allowed.iter().any(|held| Arc::ptr_eq(held, relation)) {
            return Err("music_header_Time_incident_relation_pending".into());
        }
    }
    Ok(())
}

pub(crate) fn register_time(
    graph: &mut Graph,
    registry: &mut Registry,
    header: &NativeStaffHeader,
    system: &NativeSystemHeaders,
) -> Result<Option<RegisteredHeader>, String> {
    let Some(source) = header.time() else {
        return Ok(None);
    };
    if !system.staffs().iter().any(|h| std::ptr::eq(h, header))
        || header.staff().system != system.system().id
    {
        return Err("music_header_actual_Time_owners_required".into());
    }
    let valid_owner = |node: &crate::native_headers::NativeHeaderInter| {
        node.in_final_sig()
            && node.body()["staff_id"] == header.staff().id
            && node.body()["frozen"] == true
            && node.body()["removed"] != true
    };
    if !valid_owner(source) {
        return Err("music_header_actual_frozen_Time_required".into());
    }
    let body = source.body();
    let key = Key {
        system: system.system().id,
        phase: Phase::MusicTime,
        local: usize::try_from(source.id()).map_err(|_| "music_header_Time_identity_overflow")?,
    };
    if graph.id(key).is_some() {
        return Err("music_header_fresh_Time_binding_required".into());
    }
    let mut next = graph.clone();
    let mut physical = registry.clone();
    if body["kind"] == "TimeWholeInter" {
        if !source.members().is_empty() {
            return Err("music_header_TimeWhole_members_pending".into());
        }
        let shape = body["shape"].as_str().ok_or("music_time_shape_required")?;
        let (numerator, denominator) = crate::time_column::whole_time(shape)?;
        let value = match shape {
            "COMMON_TIME" | "CUT_TIME" => shape.to_owned(),
            _ => format!("{numerator}/{denominator}"),
        };
        if body["numerator"] != numerator
            || body["denominator"] != denominator
            || body["value"] != value
        {
            return Err("music_selected_time_whole_value_required".into());
        }
        require_time_relations(system, source, &[])?;
        let mut data = body.clone();
        data["source_header_id"] = json!(source.id());
        data["glyph_id"] =
            json!(physical.register(&beam_dispatch::glyph_from_json(&data["glyph"])?)?);
        let id = next.vertex(key, data, false)?;
        next.bind_native_header_inter(key, header, source.clone())?;
        registry.commit_extension(physical)?;
        *graph = next;
        return Ok(Some(RegisteredHeader {
            key,
            projection: json!({"id":id,"origin":key,"source_header_id":source.id(),
                "staff_id":header.staff().id,"numerator":numerator,"denominator":denominator,"shape":shape}),
        }));
    }
    if body["kind"] != "TimePairInter" || !body["glyph"].is_null() {
        return Err("music_header_actual_TimePair_required".into());
    }
    let numerator = body["numerator"]
        .as_i64()
        .ok_or("music_time_numerator_required")?;
    let denominator = body["denominator"]
        .as_i64()
        .ok_or("music_time_denominator_required")?;
    if !crate::time_column::supported(numerator, denominator)
        || body["value"] != format!("{numerator}/{denominator}")
    {
        return Err("music_selected_time_pair_value_required".into());
    }
    let members = source.members();
    if members.len() != 2 {
        return Err("music_header_actual_TimePair_members_required".into());
    }
    let mut identities = BTreeSet::from([source.id()]);
    let mut containment = Vec::new();
    for (member, (side, value)) in members
        .iter()
        .zip([("TOP", numerator), ("BOTTOM", denominator)])
    {
        let data = member.body();
        if !valid_owner(member)
            || data["kind"] != "TimeNumberInter"
            || data["side"] != side
            || data["value"] != value
            || crate::time_column::time_number(data["shape"].as_str().unwrap_or(""))? != value
            || !member.members().is_empty()
            || !identities.insert(member.id())
        {
            return Err("music_header_actual_TimeNumber_required".into());
        }
        let edges = system
            .relations()
            .iter()
            .filter(|r| {
                r.body()["kind"] == "Containment"
                    && Arc::ptr_eq(r.source(), source)
                    && Arc::ptr_eq(r.target(), member)
            })
            .collect::<Vec<_>>();
        if edges.len() != 1 {
            return Err("music_header_actual_Time_member_relation_required".into());
        }
        containment.push(edges[0].clone());
    }
    let supports = system
        .relations()
        .iter()
        .filter(|r| {
            r.body()["kind"] == "TimeTopBottomRelation"
                && Arc::ptr_eq(r.source(), &members[0])
                && Arc::ptr_eq(r.target(), &members[1])
        })
        .collect::<Vec<_>>();
    if supports.len() != 1 {
        return Err("music_header_actual_Time_number_support_required".into());
    }
    require_time_relations(
        system,
        source,
        &[
            supports[0].clone(),
            containment[0].clone(),
            containment[1].clone(),
        ],
    )?;
    let mut member_keys = Vec::new();
    let mut rows = Vec::new();
    for member in members {
        let member_key = Key {
            local: usize::try_from(member.id())
                .map_err(|_| "music_header_TimeNumber_identity_overflow")?,
            ..key
        };
        if next.id(member_key).is_some() {
            return Err("music_header_fresh_TimeNumber_binding_required".into());
        }
        let mut data = member.body().clone();
        data["source_header_id"] = json!(member.id());
        data["glyph_id"] =
            json!(physical.register(&beam_dispatch::glyph_from_json(&data["glyph"])?)?);
        next.vertex(member_key, data, false)?;
        next.bind_native_header_inter(member_key, header, member.clone())?;
        rows.push(next.retained_inter(member_key)?.0.clone());
        member_keys.push(member_key);
    }
    // TimeBuilder creates number support before TimeColumn creates the ensemble.
    next.register_native_header_relation(
        member_keys[0],
        member_keys[1],
        supports[0].clone(),
        Phase::MusicTime,
    )?;
    let mut data = body.clone();
    data["source_header_id"] = json!(source.id());
    data["member_ids"] = json!(member_keys.iter().map(|k| next.id(*k)).collect::<Vec<_>>());
    data["members"] = json!(rows);
    let id = next.vertex(key, data, false)?;
    next.bind_native_header_inter(key, header, source.clone())?;
    for (member, relation) in member_keys.into_iter().zip(containment) {
        next.register_native_header_relation(key, member, relation, Phase::MusicTime)?;
    }
    registry.commit_extension(physical)?;
    *graph = next;
    Ok(Some(RegisteredHeader {
        key,
        projection: json!({"id":id,"origin":key,"source_header_id":source.id(),
            "staff_id":header.staff().id,"numerator":numerator,"denominator":denominator,
            "member_ids":rows.iter().map(|row|&row["id"]).collect::<Vec<_>>()}),
    }))
}
