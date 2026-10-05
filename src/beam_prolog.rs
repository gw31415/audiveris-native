// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native BEAMS prolog after pixel preparation: BlackHeadSizer, canonical glyph
// identity/group union and system dispatch. BeamInter creation remains downstream.
use crate::{
    beam_dispatch::{self, Geometry, Group, Registry, SystemSpots},
    beam_spots::PreparedSpots,
    black_head_sizer::{self, HeadGroup, SizingResult},
    scale::Scale,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub struct Prolog {
    pub geometry: Geometry,
    pub registry: Registry,
    pub systems: Vec<SystemSpots>,
    pub head_sizing: SizingResult,
}
impl Prolog {
    pub fn report(&self) -> Result<Value, String> {
        let sizing = &self.head_sizing;
        let closed=sizing.closed_candidates.iter().map(|c|json!({"source_index":c.source_index,"glyph":beam_dispatch::glyph_json(&c.glyph),"registered_id":c.registered_id,"accepted_after_close":c.accepted_after_close,"group":c.group})).collect::<Vec<_>>();
        let scale = json!({"black_head":sizing.black_head_scale.map(|s|json!({"widthMean":s.width_mean,"widthStd":s.width_std,"heightMean":s.height_mean,"heightStd":s.height_std})),"music_font":sizing.music_font_scale.as_ref().map(|s|json!({"name":s.name,"pointSize":s.point_size}))});
        Ok(
            json!({"stage":"BEAMS.native_prolog","beam_prolog_complete":sizing.music_font_unsupported.is_none(),"beams_complete":false,"musicxml_complete":false,"head_scale":scale,"head_sizing":{"parameters":sizing.parameters,"closed_candidates":closed,"singles":sizing.singles,"stacks":sizing.stacks,"measured_core":sizing.measured_core,"music_font_unsupported":sizing.music_font_unsupported},"systems":beam_dispatch::grouped_snapshot(&self.systems,&self.registry,Group::BeamSpot)?,"registry":self.registry.snapshot(),"free_glyphs":self.systems,"identity_scope":"native phase-owned canonical glyph identities, not Java numeric IDs or a JVM weak-index GC simulation; accepted seed identities/groups are retained","supported_scope":"fresh default five-line single-column native GRID; no drum/one-line or music-family overrides; current native resource bounds","pending":["BeamStructure and BeamInter construction","beam hooks/groups/extensions and multiple-measure rests","LEDGERS, HEADS, stem attachment, rhythm and MusicXML"]}),
        )
    }
}

/// Existing accepted seeds are already registered/free in original STEM_SEEDS.
/// Retain their group and insertion position if a beam spot reuses their pixels.
fn initialize_seeds(
    geometry: &Geometry,
    registry: &mut Registry,
    seeds: &Value,
) -> Result<Vec<SystemSpots>, String> {
    if seeds["stem_seeds_complete"] != true {
        return Err("incomplete_beam_prolog_seeds".into());
    }
    let rows = seeds["systems"]
        .as_array()
        .filter(|v| v.len() == geometry.systems.len())
        .ok_or("invalid_beam_prolog_seed_systems")?;
    let mut systems = Vec::new();
    let mut total = 0usize;
    for (row, system) in rows.iter().zip(&geometry.systems) {
        if row["system_id"] != system.id {
            return Err("inconsistent_beam_prolog_seed_system".into());
        }
        let glyphs = row["accepted_seeds"]
            .as_array()
            .ok_or("invalid_beam_prolog_accepted_seeds")?;
        total = total
            .checked_add(glyphs.len())
            .filter(|n| *n <= beam_dispatch::MAX_REGISTERED_GLYPHS)
            .ok_or("unsupported_beam_prolog_seed_count")?;
        let mut ids = Vec::new();
        let mut seen = BTreeSet::new();
        for raw in glyphs {
            let glyph = beam_dispatch::glyph_from_json(raw)?;
            let id = registry.register(&glyph)?;
            if !seen.insert(id) {
                return Err("duplicate_beam_prolog_seed".into());
            }
            registry.add_group(id, Group::VerticalSeed)?;
            ids.push(id);
        }
        systems.push(SystemSpots {
            system_id: system.id,
            glyph_ids: ids,
        });
    }
    Ok(systems)
}

pub fn build(
    prepared: &PreparedSpots,
    grid: &Value,
    seeds: &Value,
    scale: &Scale,
) -> Result<Prolog, String> {
    let geometry = Geometry::from_grid(
        prepared.binarized.width,
        prepared.binarized.height,
        grid,
        scale.interline.main,
    )?;
    if seeds["stem_scaling"]["stem_scale"]["max"].as_u64()
        != Some(prepared.parameters.max_stem as u64)
    {
        return Err("inconsistent_beam_prolog_stem_scale".into());
    }
    geometry.validate_point_work(
        prepared
            .glyphs
            .len()
            .checked_mul(2)
            .ok_or("unsupported_beam_prolog_query_work")?,
    )?;
    let mut registry = Registry::default();
    let mut systems = initialize_seeds(&geometry, &mut registry, seeds)?;
    let head_sizing = black_head_sizer::process(
        &prepared.glyphs,
        black_head_sizer::Parameters::from_interline(scale.interline.main)?,
        "Bravura",
        |glyph| geometry.register_closed(&mut registry, glyph),
    )?;
    for c in &head_sizing.closed_candidates {
        if let (Some(id), Some(group)) = (c.registered_id, c.group) {
            registry.add_group(
                id,
                match group {
                    HeadGroup::Single => Group::BlackHeadSpot,
                    HeadGroup::Stack => Group::BlackStackSpot,
                },
            )?;
        }
    }
    beam_dispatch::dispatch_into(&prepared.glyphs, &geometry, &mut registry, &mut systems)?;
    Ok(Prolog {
        geometry,
        registry,
        systems,
        head_sizing,
    })
}
