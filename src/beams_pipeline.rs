// SPDX-License-Identifier: AGPL-3.0-or-later
// Existing native stages in original initial/extension/hook/group/rest order.
use crate::{
    GrayImage, beam_candidates::CandidateAnalysis, beam_prolog::Prolog, scale::Scale,
    shared_sig::Graph,
};
use serde_json::{Value, json};
pub fn advance(
    state: &mut CandidateAnalysis,
    prolog: &mut Prolog,
    residual: &GrayImage,
    binary: &GrayImage,
    grid: &Value,
    scale: &Scale,
    shared: &mut Graph,
) -> Result<Value, String> {
    let extension_scale = crate::beam_extend_to_beam::ExtensionScale {
        interline: scale.interline.main,
        standard_height: scale.beam.thickness as f64,
        small_height: scale.small_beam.map(|v| v.thickness as f64),
    };
    let mut reports = Vec::new();
    for owner in &mut prolog.systems {
        let system = state
            .system_mut(owner.system_id)
            .ok_or("missing_native_beams_system")?;
        let extension = crate::beam_extension_loop::extend_beams(
            system,
            residual,
            &mut prolog.registry,
            extension_scale,
            crate::beam_extend_in_parallel::ParallelOwner {
                free_glyphs: owner,
                geometry: &prolog.geometry,
            },
        )?;
        let hooks = crate::beam_late_hooks::build_hooks(
            system,
            residual,
            &mut prolog.registry,
            extension_scale,
            owner,
        )?;
        let mut groups = crate::beam_groups::populate_system(system, scale.interline.main)?;
        shared.sync_system(
            system,
            &groups,
            &crate::multiple_rest_registration::RestGraph::default(),
        )?;
        let rest_result = (|| {
            let mapping = crate::native_rest_mapping::SystemMapping::from_grid_with_main(
                grid,
                binary.width,
                binary.height,
                owner.system_id,
                scale.interline.main,
            )?;
            let mut positions = std::collections::BTreeMap::new();
            for beam in system.inters().iter().filter(|b| {
                !b.is_removed()
                    && b.kind == "BeamInter"
                    && b.bounds[2] >= (4. * scale.interline.main as f64).round_ties_even() as i32
            }) {
                positions.insert(beam.id(), mapping.position(beam)?);
            }
            let staffs = mapping
                .staffs
                .iter()
                .map(|s| (s.id, s.geometry(scale.line.main)))
                .collect::<Vec<_>>();
            crate::multiple_rest_pipeline::process_shared(
                system,
                &mut groups,
                &mut prolog.registry,
                crate::multiple_rest_pipeline::Context {
                    image: binary,
                    interline: scale.interline.main,
                    fore: scale.line.main,
                    sections: &mapping.sections,
                    staffs: &staffs,
                },
                |b| positions.get(&b.id()).copied().flatten(),
                shared,
            )
        })();
        let rests = match rest_result {
            Ok(rests) => {
                json!({"complete":true,"conversions":rests.rests.len(),"registered_serifs":rests.serifs.len(),"supported_scope":"declared native GRID original global neighbor areas, certified full-width or non-crossing straight/curved clipped system bounds, five-line staff membership selection; supplied vertical-column geometry retained; system-bounds lag intersections and empty/supplied straight-ledger pitches"})
            }
            Err(error) => json!({"complete":false,"error":error}),
        };
        reports.push(json!({"system_id":owner.system_id,"extensions":{"initial_ids":extension.initial_ids,"side_attempts":extension.sides.len(),"skipped_removed":extension.skipped_removed},"hooks":{"initial_ids":hooks.initial_ids,"attempts":hooks.attempts.len(),"appended":hooks.attempts.iter().filter(|a|a.appended.is_some()).count()},"groups":groups.groups.iter().map(|g|json!({"id":g.id,"members":g.members,"bounds":g.bounds})).collect::<Vec<_>>(),"supports":groups.supports,"rests":rests}));
    }
    let rests_complete = reports.iter().all(|v| v["rests"]["complete"] == true);
    let epilog = if rests_complete {
        let removed =
            crate::beam_dispatch::finish_beams_epilog(&mut prolog.systems, &prolog.registry)?;
        json!({"complete":true,"systems":removed.into_iter().map(|(id,glyphs)|json!({"system_id":id,"removed_free_glyph_ids":glyphs})).collect::<Vec<_>>(),"remaining_free_glyphs":prolog.systems,"registry_glyph_count":prolog.registry.len(),"graph_mutation":false})
    } else {
        json!({"complete":false,"reason":"incomplete_native_rest_boundary"})
    };
    Ok(
        json!({"stage":"BEAMS.native_through_multiple_rests","extensions_complete":true,"hooks_complete":true,"groups_complete":true,"multiple_rest_boundary_complete":rests_complete,"systems":reports,"epilog":epilog,"epilog_complete":rests_complete,"beams_complete":rests_complete,"musicxml_complete":false,"pending":["LEDGERS/HEADS/rhythm/MusicXML","curved clipped system bounds, curved ledger paths and native ledger discovery"]}),
    )
}
