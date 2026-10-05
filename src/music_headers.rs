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
