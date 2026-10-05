// SPDX-License-Identifier: AGPL-3.0-or-later
//! Preserve native frozen header keys and their actual glyph/ensemble identities.
//! Key-only HeadInter.getAlteration fallback; accidental/tie paths remain guarded.
use crate::{
    beam_dispatch::{self, Registry},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(crate) fn import_key(
    graph: &mut Graph,
    registry: &mut Registry,
    header: &Value,
    clef_key: Key,
    source_relations: &[Value],
) -> Result<Value, String> {
    let original = &header["key"];
    if original.is_null() {
        return Ok(Value::Null);
    }
    let frozen = |n: &Value| -> Result<usize, String> {
        if n["frozen"] != true || n["removed"] == true {
            return Err("music_frozen_header_key_required".into());
        }
        n["id"]
            .as_u64()
            .filter(|v| *v > 0)
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("music_header_source_id_required".into())
    };
    let source_id = frozen(original)?;
    let fifths = original["fifths"]
        .as_i64()
        .filter(|v| (-7..=7).contains(v) && *v != 0)
        .ok_or("music_header_natural_cancel_or_invalid_fifths_pending")?;
    let members = original["members"]
        .as_array()
        .filter(|m| m.len() == fifths.unsigned_abs() as usize)
        .ok_or("music_header_key_members_required")?;
    if original["kind"] != "KeyInter" || !original["glyph"].is_null() {
        return Err("music_header_key_ensemble_required".into());
    }
    let shape = if fifths > 0 { "SHARP" } else { "FLAT" };
    let mut ids = BTreeSet::from([source_id]);
    for member in members {
        let id = frozen(member)?;
        if member["kind"] != "KeyAlterInter" || member["shape"] != shape || !ids.insert(id) {
            return Err("music_header_key_member_identity_or_shape_required".into());
        }
    }
    let source_clef = header["clef"]["id"]
        .as_u64()
        .ok_or("music_header_clef_source_required")?;
    let support = source_relations
        .iter()
        .filter(|r| {
            r["kind"] == "ClefKeyRelation" && r["source"] == source_clef && r["target"] == source_id
        })
        .collect::<Vec<_>>();
    if support.len() != 1 || graph.id(clef_key).is_none() {
        return Err("music_header_selected_clef_key_support_required".into());
    }
    for member in members {
        if source_relations
            .iter()
            .filter(|r| {
                r["kind"] == "Containment"
                    && r["source"] == source_id
                    && r["target"] == member["id"]
            })
            .count()
            != 1
        {
            return Err("music_header_key_containment_required".into());
        }
    }
    let mut next = graph.clone();
    let mut physical = registry.clone();
    let key = Key {
        system: clef_key.system,
        phase: Phase::MusicKey,
        local: source_id,
    };
    let mut retained = Vec::new();
    let mut relations = Vec::new();
    for member in members {
        let local = frozen(member)?;
        let glyph = beam_dispatch::glyph_from_json(&member["glyph"])?;
        let glyph_id = physical.register(&glyph)?;
        let member_key = Key { local, ..key };
        let mut data = member.clone();
        data["source_header_id"] = data["id"].clone();
        data["staff_id"] = header["staff_id"].clone();
        data["glyph_id"] = json!(glyph_id);
        let id = next.vertex(member_key, data, false)?;
        retained.push(
            next.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .unwrap()
                .clone(),
        );
        relations.push((key, member_key, json!({"kind":"Containment"})));
    }
    let mut data = original.clone();
    data["source_header_id"] = data["id"].clone();
    data["staff_id"] = header["staff_id"].clone();
    data["member_ids"] = json!(retained.iter().map(|n| &n["id"]).collect::<Vec<_>>());
    data["members"] = json!(retained);
    let id = next.vertex(key, data, false)?;
    relations.push((clef_key, key, support[0].clone()));
    next.relations(key.system, Phase::MusicKey, relations)?;
    // Append only: existing glyph allocations/cache aliases are preserved.
    registry.commit_extension(physical)?;
    *graph = next;
    Ok(json!({"id":id,"origin":key,"source_header_id":source_id,
        "staff_id":header["staff_id"],"fifths":fifths,
        "member_ids":retained.iter().map(|n| &n["id"]).collect::<Vec<_>>()}))
}

/// StaffHeader.freeze already selected the actual pair and froze its members.
/// Retain the selected TimePair and its actual numeric members atomically.
/// A selected TimeWhole retains its own physical glyph instead of number members.
pub(crate) fn import_time(
    graph: &mut Graph,
    registry: &mut Registry,
    header: &Value,
    system: u64,
    source_relations: &[Value],
) -> Result<Value, String> {
    let pair = &header["time"];
    if pair.is_null() {
        return Ok(Value::Null);
    }
    if pair["kind"] == "TimeWholeInter" {
        return import_whole_time(graph, registry, header, system);
    }
    if pair["kind"] != "TimePairInter" || pair["frozen"] != true || !pair["glyph"].is_null() {
        return Err("music_other_or_unfrozen_header_time_import_pending".into());
    }
    let numerator = pair["numerator"]
        .as_i64()
        .ok_or("music_time_numerator_required")?;
    let denominator = pair["denominator"]
        .as_i64()
        .ok_or("music_time_denominator_required")?;
    if !crate::time_column::supported(numerator, denominator)
        || pair["value"] != format!("{numerator}/{denominator}")
    {
        return Err("music_selected_time_pair_value_required".into());
    }
    let identity = |n: &Value| -> Result<usize, String> {
        if n["frozen"] != true || n["removed"] == true {
            return Err("music_frozen_header_time_required".into());
        }
        n["id"]
            .as_u64()
            .filter(|v| *v > 0)
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("music_time_source_id_required".into())
    };
    let source_id = identity(pair)?;
    let members = pair["members"]
        .as_array()
        .filter(|m| m.len() == 2)
        .ok_or("music_time_pair_members_required")?;
    let top = identity(&members[0])?;
    let bottom = identity(&members[1])?;
    if source_id == top || source_id == bottom || top == bottom {
        return Err("music_time_duplicate_source_identity".into());
    }
    for (n, (side, value)) in members
        .iter()
        .zip([("TOP", numerator), ("BOTTOM", denominator)])
    {
        if n["kind"] != "TimeNumberInter"
            || n["side"] != side
            || n["value"] != value
            || crate::time_column::time_number(n["shape"].as_str().unwrap_or(""))? != value
        {
            return Err("music_selected_time_pair_number_required".into());
        }
        if source_relations
            .iter()
            .filter(|rel| {
                rel["kind"] == "Containment"
                    && rel["source"] == source_id
                    && rel["target"] == n["id"]
            })
            .count()
            != 1
        {
            return Err("music_time_containment_required".into());
        }
    }
    let support = source_relations
        .iter()
        .filter(|rel| {
            rel["kind"] == "TimeTopBottomRelation"
                && rel["source"] == top
                && rel["target"] == bottom
        })
        .collect::<Vec<_>>();
    if support.len() != 1 {
        return Err("music_time_selected_number_support_required".into());
    }
    let mut next = graph.clone();
    let mut physical = registry.clone();
    let key = Key {
        system,
        phase: Phase::MusicTime,
        local: source_id,
    };
    let mut retained = Vec::new();
    let mut relations = Vec::new();
    for member in members {
        let member_key = Key {
            local: identity(member)?,
            ..key
        };
        let mut data = member.clone();
        data["source_header_id"] = data["id"].clone();
        data["staff_id"] = header["staff_id"].clone();
        data["glyph_id"] =
            json!(physical.register(&beam_dispatch::glyph_from_json(&member["glyph"])?)?);
        let id = next.vertex(member_key, data, false)?;
        retained.push(
            next.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .unwrap()
                .clone(),
        );
        relations.push((key, member_key, json!({"kind":"Containment"})));
    }
    let mut data = pair.clone();
    data["source_header_id"] = data["id"].clone();
    data["staff_id"] = header["staff_id"].clone();
    data["member_ids"] = json!(retained.iter().map(|n| &n["id"]).collect::<Vec<_>>());
    data["members"] = json!(retained);
    let id = next.vertex(key, data, false)?;
    relations.push((
        Key { local: top, ..key },
        Key {
            local: bottom,
            ..key
        },
        support[0].clone(),
    ));
    next.relations(system, Phase::MusicTime, relations)?;
    registry.commit_extension(physical)?;
    *graph = next;
    Ok(
        json!({"id":id,"origin":key,"source_header_id":source_id,"staff_id":header["staff_id"],"numerator":numerator,"denominator":denominator,"member_ids":retained.iter().map(|n|&n["id"]).collect::<Vec<_>>()}),
    )
}

fn import_whole_time(
    graph: &mut Graph,
    registry: &mut Registry,
    header: &Value,
    system: u64,
) -> Result<Value, String> {
    let time = &header["time"];
    if time["frozen"] != true || time["removed"] == true {
        return Err("music_frozen_header_time_required".into());
    }
    let shape = time["shape"].as_str().ok_or("music_time_shape_required")?;
    let (numerator, denominator) = crate::time_column::whole_time(shape)?;
    let label = match shape {
        "COMMON_TIME" | "CUT_TIME" => shape.to_owned(),
        _ => format!("{numerator}/{denominator}"),
    };
    if time["numerator"] != numerator
        || time["denominator"] != denominator
        || time["value"] != label
    {
        return Err("music_selected_time_whole_value_required".into());
    }
    let source_id = time["id"]
        .as_u64()
        .filter(|id| *id > 0)
        .and_then(|id| usize::try_from(id).ok())
        .ok_or("music_time_source_id_required")?;
    let key = Key {
        system,
        phase: Phase::MusicTime,
        local: source_id,
    };
    let mut next = graph.clone();
    let mut physical = registry.clone();
    let mut data = time.clone();
    data["source_header_id"] = time["id"].clone();
    data["staff_id"] = header["staff_id"].clone();
    data["glyph_id"] = json!(physical.register(&beam_dispatch::glyph_from_json(&time["glyph"])?)?);
    let id = next.vertex(key, data, false)?;
    registry.commit_extension(physical)?;
    *graph = next;
    Ok(
        json!({"id":id,"origin":key,"source_header_id":source_id,"staff_id":header["staff_id"],"numerator":numerator,"denominator":denominator,"shape":shape}),
    )
}

/// Exact KeyInter.getAlterFor order. HeadInter first consults accidentals/ties;
/// the current admitted image path rejects those unconnected symbol relations.
pub(crate) fn key_alter(step: &str, fifths: Option<i64>) -> Result<i32, String> {
    if !["A", "B", "C", "D", "E", "F", "G"].contains(&step) {
        return Err("musicxml_diatonic_step_required".into());
    }
    let fifths = fifths.unwrap_or(0);
    if !(-7..=7).contains(&fifths) {
        return Err("musicxml_key_fifths_out_of_range".into());
    }
    let order = if fifths > 0 {
        ["F", "C", "G", "D", "A", "E", "B"]
    } else {
        ["B", "E", "A", "D", "G", "C", "F"]
    };
    Ok(if order[..fifths.unsigned_abs() as usize].contains(&step) {
        if fifths > 0 { 1 } else { -1 }
    } else {
        0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_key_alter_controls_and_invalid_late_header_are_atomic() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-header/observed.json"
        ))
        .unwrap();
        for c in o["key_alter_kernel"]["cases"].as_array().unwrap() {
            assert_eq!(
                key_alter(c["step"].as_str().unwrap(), c["fifths"].as_i64()).unwrap(),
                c["alter"].as_i64().unwrap() as i32
            );
        }
        assert_eq!(key_alter("F", None).unwrap(), 0);
        assert!(key_alter("X", Some(1)).is_err());
        assert!(key_alter("F", Some(8)).is_err());
        let member = o["original_nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "KeyAlterInter")
            .unwrap();
        let mut h = json!({"staff_id":1,"clef":{"id":15},"key":{"id":19,"kind":"KeyInter","fifths":1,"frozen":true,"glyph":null,"members":[{"id":18,"kind":"KeyAlterInter","shape":"SHARP","frozen":true,"glyph":member["glyph"]}]}});
        let mut g = Graph::default();
        let k = Key {
            system: 1,
            phase: Phase::MusicHeader,
            local: 1,
        };
        g.vertex(k, json!({"kind":"ClefInter","frozen":true}), false)
            .unwrap();
        let mut r = Registry::default();
        let before = g.snapshot();
        let physical = r.snapshot();
        let rel = vec![
            json!({"kind":"Containment","source":19,"target":18}),
            json!({"kind":"ClefKeyRelation","source":15,"target":19}),
        ];
        h["key"]["members"][0]["glyph"]["runs"] = json!([[0, 0, 999]]);
        assert!(import_key(&mut g, &mut r, &h, k, &rel).is_err());
        assert_eq!(g.snapshot(), before);
        assert_eq!(r.snapshot(), physical);
        h["key"]["frozen"] = json!(false);
        assert!(import_key(&mut g, &mut r, &h, k, &rel).is_err());
        assert_eq!(g.snapshot(), before);
        assert_eq!(r.snapshot(), physical);
        assert!(
            import_key(&mut g, &mut r, &json!({"key":null}), k, &[])
                .unwrap()
                .is_null()
        );
    }
}

#[cfg(test)]
mod time_import_tests {
    use super::*;
    #[test]
    fn time_import_errors_preserve_graph_and_registry() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-header-time/observed.json"
        ))
        .unwrap();
        let mut pair = oracle["original_nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "TimePairInter")
            .unwrap()
            .clone();
        pair["id"] = pair["original_id"].clone();
        let mut members = oracle["original_nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "TimeNumberInter")
            .cloned()
            .collect::<Vec<_>>();
        members.sort_by_key(|n| if n["side"] == "TOP" { 0 } else { 1 });
        for n in &mut members {
            n["id"] = n["original_id"].clone();
        }
        let relations = vec![
            json!({"kind":"Containment","source":pair["id"],"target":members[0]["id"]}),
            json!({"kind":"Containment","source":pair["id"],"target":members[1]["id"]}),
            json!({"kind":"TimeTopBottomRelation","source":members[0]["id"],"target":members[1]["id"]}),
        ];
        pair["members"] = json!(members);
        let mut h = json!({"staff_id":1,"time":pair});
        let mut graph = Graph::default();
        let mut registry = Registry::default();
        let before = graph.snapshot();
        let physical = registry.snapshot();
        // A selected pair and its physical numbers must agree; reject the
        // inconsistent value before registering any member or relation.
        h["time"]["members"][0]["value"] = json!(3);
        assert!(import_time(&mut graph, &mut registry, &h, 1, &relations).is_err());
        assert_eq!(before, graph.snapshot());
        assert_eq!(physical, registry.snapshot());
        h["time"]["members"][0]["value"] = json!(4);
        h["time"]["members"][1]["glyph"]["runs"] = json!([[0, 0, 999]]);
        assert!(import_time(&mut graph, &mut registry, &h, 1, &relations).is_err());
        assert_eq!(before, graph.snapshot());
        assert_eq!(physical, registry.snapshot());
        let whole: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-header-common/observed.json"
        ))
        .unwrap();
        h["time"] = whole["original_nodes"][0].clone();
        h["time"]["id"] = h["time"]["original_id"].clone();
        h["time"]["glyph"]["runs"] = json!([[0, 0, 999]]);
        assert!(import_time(&mut graph, &mut registry, &h, 1, &[]).is_err());
        assert_eq!(before, graph.snapshot());
        assert_eq!(physical, registry.snapshot());
        h["time"] = whole["original_nodes"][0].clone();
        h["time"]["id"] = h["time"]["original_id"].clone();
        h["time"]["numerator"] = json!(3);
        assert!(import_time(&mut graph, &mut registry, &h, 1, &[]).is_err());
        assert_eq!(before, graph.snapshot());
        assert_eq!(physical, registry.snapshot());
        h["time"] = Value::Null;
        assert!(
            import_time(&mut graph, &mut registry, &h, 1, &[])
                .unwrap()
                .is_null()
        );
        assert_eq!(before, graph.snapshot());
        assert_eq!(physical, registry.snapshot());
    }
}
