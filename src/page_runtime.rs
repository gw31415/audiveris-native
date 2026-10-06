// SPDX-License-Identifier: AGPL-3.0-or-later
//! Existing image phases retained as one owned page, without report rehydration.
use crate as audiveris_native;
use crate::{GrayImage, scale, sections, vertical};
use serde_json::Value;

pub struct OwnedPage {
    pub report: Value,
    pub output_pixels: Vec<u8>,
    pub graph: crate::shared_sig::Graph,
    pub registry: Option<crate::beam_dispatch::Registry>,
    pub success: bool,
}
// MusicFont.getHeadPointSize falls back to four times staff interline when
// BlackHeadSizer did not establish a measured font scale. The available bank
// covers ordinary 11px/44pt and the existing 7px cue staff only.
fn unmeasured_catalog(
    grid: &Value,
    main_interline: usize,
) -> Option<crate::head_staff_catalog::TemplateCatalog> {
    let staffs = grid["staffs"].as_array()?;
    let cue = staffs.iter().any(|s| s["small"] == true);
    let ordinary = !staffs.is_empty()
        && staffs.iter().all(|s| {
            s["specific_interline"] == 11
                && s["lines"].as_array().is_some_and(|v| v.len() == 5)
                && s["drum"] != true
                && s["tablature"] != true
        });
    (main_interline == 11 && (cue || ordinary)).then(|| {
        crate::head_staff_catalog::TemplateCatalog {
            family: "Bravura".into(),
            stored_point_size: 44,
        }
    })
}
/// Runs the unchanged native upstream kernels. The graph includes private
/// removed-edge keys and reserved identities omitted by its report snapshot.
pub fn analyze(
    image: &GrayImage,
    mode: &str,
    model: Option<&audiveris_classifier::Model>,
    declared: Option<(crate::head_staff_catalog::TemplateCatalog, bool)>,
) -> Result<OwnedPage, Box<dyn std::error::Error>> {
    if !matches!(mode, "page-glyphs" | "staff-mask" | "beam-spots") {
        return Err("unsupported owned page mode".into());
    }
    if image.width == 0
        || image.height == 0
        || image
            .width
            .checked_mul(image.height)
            .is_none_or(|n| n != image.pixels.len() || n > crate::MAX_PIXELS)
    {
        return Err("invalid owned page image".into());
    }
    let binary = vertical::binarize(image, |_, _, _| {});
    let sc = scale::analyze(&binary).result.map_err(|e| e.code())?;
    let prep = sections::prepare(&binary, &sc)?;
    let factory = audiveris_native::filament_merge::analyze(image.width, &sc, &prep.horizontal);
    let staff =
        audiveris_native::clusters::analyze_two_pass(image.width, &sc, &prep.horizontal, &factory);
    if staff["status"] == "unsupported" {
        return Err(staff["error"]
            .as_str()
            .unwrap_or("unsupported_staff_layout")
            .into());
    }
    let projection =
        audiveris_native::staff_projection::analyze(&binary, &sc, &prep.horizontal, &staff);
    let sticks = audiveris_native::bar_sticks::analyze(&binary, &sc, &prep, &projection);
    let slope = factory["line_filtering"]["global_slope"]
        .as_f64()
        .unwrap_or(0.);
    let systems = audiveris_native::system_graph::analyze(&binary, &sc, &staff, &sticks, slope);
    let completion = audiveris_native::line_completion::analyze(
        &binary,
        &sc,
        &prep,
        &staff,
        &projection,
        &sticks,
        &systems,
        slope,
    );
    if completion["status"] != "partial_line_completion" {
        return Err(completion["error"]
            .as_str()
            .unwrap_or("line completion unavailable")
            .to_string()
            .into());
    }
    let (residual, result) = audiveris_native::page_glyphs::extract(
        &binary,
        &completion,
        sc.interline.main as u32,
        model,
    )?;
    let mut grid_ir =
        audiveris_native::grid_ir::build(&residual, &sc, &prep, &completion, &systems, &sticks)?;
    let header_preparation = audiveris_native::header_preparation::from_grid(&grid_ir)?;
    let clef_candidates = audiveris_native::clef_candidates::analyze(
        &residual,
        &grid_ir,
        &header_preparation,
        sc.interline.main,
    )?;
    let clef_registration = audiveris_native::clef_registration::analyze(
        &clef_candidates,
        &grid_ir,
        &header_preparation,
        sc.interline.main,
    )?;
    let key_preparation = audiveris_native::key_preparation::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        sc.interline.main,
    )?;
    let key_peaks = audiveris_native::key_peaks::analyze(&key_preparation, sc.interline.main)?;
    let key_slices = audiveris_native::key_slices::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        sc.interline.main,
    )?;
    let key_components = audiveris_native::key_components::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        sc.interline.main,
    )?;
    let key_fallback = audiveris_native::key_components::analyze_with_empty_slice_fallback(
        &residual,
        &grid_ir,
        &clef_registration,
        sc.interline.main,
    )?;
    let key_process = audiveris_native::key_state::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        sc.interline.main,
    )?;
    let key_column = audiveris_native::key_column::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        sc.interline.main,
    )?;
    let time_candidates = audiveris_native::time_candidates::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        &key_column,
        sc.interline.main,
    )?;
    let time_column = audiveris_native::time_column::analyze(
        &residual,
        &grid_ir,
        &clef_registration,
        &key_column,
        sc.interline.main,
    )?;
    let headers =
        audiveris_native::header_finalization::analyze(&grid_ir, &time_column, sc.interline.main)?;
    let stem_seeds =
        audiveris_native::stem_seeds::analyze(&residual, &grid_ir, &headers, &sc, slope)?;
    let (beam_images, beam_preparation) = match audiveris_native::beam_preparation::analyze(
        &residual,
        &grid_ir,
        &headers,
        &stem_seeds,
        &sc,
    ) {
        Ok((images, report)) => (Some(images), report),
        Err(error) if mode == "beam-spots" => return Err(error.into()),
        Err(error) => {
            eprintln!(
                "BEAMS spot preparation unavailable: {error}; earlier native stages retained"
            );
            (
                None,
                serde_json::json!({"stage":"BEAMS.native_spot_preparation","pixel_preparation_complete":false,"beams_complete":false,"error":error}),
            )
        }
    };
    let (mut beam_context, beam_prolog) = if let Some(images) = &beam_images {
        match audiveris_native::beam_prolog::build(images, &grid_ir, &stem_seeds, &sc)
            .and_then(|prolog| prolog.report().map(|report| (prolog, report)))
        {
            Ok((context, report)) => (Some(context), report),
            Err(error) => {
                eprintln!("BEAMS prolog unavailable: {error}; beam spot pixels retained");
                (
                    None,
                    serde_json::json!({"stage":"BEAMS.native_prolog","beam_prolog_complete":false,"beams_complete":false,"error":error}),
                )
            }
        }
    } else {
        (
            None,
            serde_json::json!({"stage":"BEAMS.native_prolog","beam_prolog_complete":false,"beams_complete":false,"error":"incomplete_beam_spot_dependency"}),
        )
    };
    // Preserve the completed prolog snapshot before candidate registration mutates
    // the phase context. A bounded-stage failure must not erase prior results.
    let (mut candidate_state, beam_candidates) = if let Some(context) = &mut beam_context {
        match audiveris_native::beam_candidates::analyze_typed(&residual, context, &sc) {
            Ok(state) => {
                let report = state.report();
                (Some(state), report)
            }
            Err(error) => {
                eprintln!(
                    "BEAMS initial candidates unavailable: {error}; completed prolog retained"
                );
                (
                    None,
                    serde_json::json!({"stage":"BEAMS.native_initial_candidates","initial_beam_candidates_complete":false,"beams_complete":false,"musicxml_complete":false,"error":error}),
                )
            }
        }
    } else {
        (
            None,
            serde_json::json!({"stage":"BEAMS.native_initial_candidates","initial_beam_candidates_complete":false,"beams_complete":false,"musicxml_complete":false,"error":"incomplete_beam_prolog_dependency"}),
        )
    };
    let mut common_graph = audiveris_native::shared_sig::Graph::default();
    common_graph.import_grid(&grid_ir)?;
    let later_beams = if let (Some(state), Some(context)) =
        (&mut candidate_state, &mut beam_context)
    {
        for system in state.systems() {
            common_graph.sync_candidates(system)?;
        }
        match audiveris_native::beams_pipeline::advance(
            state,
            context,
            &residual,
            &binary,
            &grid_ir,
            &sc,
            &mut common_graph,
        ) {
            Ok(report) => report,
            Err(error) => {
                serde_json::json!({"stage":"BEAMS.native_through_multiple_rests","multiple_rest_boundary_complete":false,"beams_complete":false,"musicxml_complete":false,"error":error})
            }
        }
    } else {
        serde_json::json!({"stage":"BEAMS.native_through_multiple_rests","multiple_rest_boundary_complete":false,"error":"incomplete_beam_candidates","beams_complete":false,"musicxml_complete":false})
    };
    let ledger_candidates = if later_beams["epilog_complete"] == true {
        if let Some(state) = &candidate_state {
            let systems = state
                .systems()
                .iter()
                .map(|s| {
                    (
                        s.system_id(),
                        audiveris_native::ledger_beam_purge::from_native(s),
                    )
                })
                .collect::<Vec<_>>();
            match audiveris_native::ledger_beam_purge::candidate_prefix(
                &residual,
                &grid_ir,
                sc.interline.main,
                sc.line.main,
                &systems,
            ) {
                Ok(report) => report,
                Err(error) => {
                    serde_json::json!({"stage":"LEDGERS.native_candidate_prefix","candidate_prefix_complete":false,"ledgers_complete":false,"error":error})
                }
            }
        } else {
            serde_json::json!({"candidate_prefix_complete":false,"ledgers_complete":false,"error":"missing_beam_state"})
        }
    } else {
        serde_json::json!({"candidate_prefix_complete":false,"ledgers_complete":false,"error":"incomplete_beams_epilog_dependency"})
    };
    let ledger_registration = if ledger_candidates["candidate_prefix_complete"] == true {
        if let Some(context) = &mut beam_context {
            let mut systems = Vec::new();
            let mut error = None;
            for row in ledger_candidates["systems"].as_array().unwrap() {
                let id = row["system_id"].as_u64().unwrap();
                let outcome = (|| {
                    let staffs =
                        audiveris_native::ledger_registration::staffs_from_grid(&grid_ir, id)?;
                    let report = audiveris_native::ledger_registration::build(
                        &residual,
                        row["candidates"].as_array().unwrap(),
                        &staffs,
                        sc.interline.main,
                        sc.line.main,
                        &mut context.registry,
                    )?;
                    common_graph.sync_ledgers(id, report["inters"].as_array().unwrap())?;
                    Ok::<_, String>(serde_json::json!({"system_id":id,"registration":report}))
                })();
                match outcome {
                    Ok(report) => systems.push(report),
                    Err(e) => {
                        error = Some(e);
                        break;
                    }
                }
            }
            serde_json::json!({"stage":"LEDGERS.native_builder","builder_complete":error.is_none(),"ledger_registration_complete":error.is_none(),"post_analysis_complete":false,"ledgers_complete":false,"systems":systems,"error":error,"glyph_registry":context.registry.snapshot()})
        } else {
            serde_json::json!({"builder_complete":false,"ledgers_complete":false,"error":"missing_native_registry"})
        }
    } else {
        serde_json::json!({"builder_complete":false,"ledgers_complete":false,"error":"incomplete_ledger_candidate_dependency"})
    };
    let ledger_post_analysis = if ledger_registration["builder_complete"] == true {
        if let Some(context) = &mut beam_context {
            let specs = ledger_registration["systems"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    let id = s["system_id"].as_u64().unwrap();
                    Ok::<_, String>((
                        id,
                        audiveris_native::ledger_registration::staffs_from_grid(&grid_ir, id)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>();
            match specs.and_then(|specs| {
                audiveris_native::ledger_post_analysis::process(
                    &residual,
                    ledger_registration["systems"].as_array().unwrap(),
                    &specs,
                    ledger_candidates["systems"].as_array().unwrap(),
                    sc.interline.main,
                    sc.line.main,
                    &mut context.registry,
                )
            }) {
                Ok(mut report) => {
                    for system in report["systems"].as_array().unwrap() {
                        common_graph.sync_ledgers(
                            system["system_id"].as_u64().unwrap(),
                            system["registration"]["inters"].as_array().unwrap(),
                        )?;
                    }
                    report["glyph_registry"] = context.registry.snapshot();
                    report
                }
                Err(error) => {
                    serde_json::json!({"post_analysis_complete":false,"ledgers_complete":false,"error":error})
                }
            }
        } else {
            serde_json::json!({"post_analysis_complete":false,"ledgers_complete":false,"error":"missing_native_registry"})
        }
    } else {
        serde_json::json!({"post_analysis_complete":false,"ledgers_complete":false,"error":"incomplete_ledger_builder_dependency"})
    };
    if declared.as_ref().is_some_and(|(_, cue)| *cue) {
        let first = grid_ir["staffs"]
            .as_array_mut()
            .and_then(|a| a.first_mut())
            .ok_or("missing_controlled_head_staff")?;
        first["specific_interline"] = serde_json::json!(7);
    }
    // Original unmeasured page: default Bravura uses 4*specific interline.
    // Only the observed ordinary 11 -> 44 and cue 7/11 -> 28/44 banks exist.
    // Other unmeasured sizes retain their explicit HEADS dependency boundary.
    let automatic_catalog =
        if declared.is_none() && beam_prolog["head_scale"]["music_font"].is_null() {
            unmeasured_catalog(&grid_ir, sc.interline.main)
        } else if declared.is_none() {
            // Carry the measured original head font into the exact available
            // catalog. Keep the certified legacy Bravura51 observation boundary.
            crate::head_staff_catalog::TemplateCatalog::from_head_scale(&beam_prolog["head_scale"])
                .ok()
                .filter(|c| c.family != "Bravura" || c.stored_point_size != 51)
        } else {
            None
        };
    let head_scale = if let Some((catalog, _)) = &declared {
        serde_json::json!({"music_font":{"name":catalog.family,"pointSize":catalog.stored_point_size}})
    } else if let Some(catalog) = &automatic_catalog {
        serde_json::json!({"music_font":{"name":catalog.family,"pointSize":catalog.stored_point_size}})
    } else {
        beam_prolog["head_scale"].clone()
    };
    let seed_analyzer = if declared.is_some() || automatic_catalog.is_some() {
        audiveris_native::head_seed_search::analyze_catalog
    } else {
        audiveris_native::head_seed_search::analyze
    };
    let head_seed_search = if ledger_post_analysis["ledgers_complete"] == true {
        match seed_analyzer(
            &binary,
            &grid_ir,
            &stem_seeds,
            &ledger_post_analysis,
            &head_scale,
            stem_seeds["stem_scaling"]["stem_scale"]["max"]
                .as_u64()
                .unwrap_or(0) as usize,
            sc.interline.main,
        ) {
            Ok(report) => report,
            Err(error) => {
                serde_json::json!({"seed_template_search_complete":false,"heads_complete":false,"error":error})
            }
        }
    } else {
        serde_json::json!({"seed_template_search_complete":false,"heads_complete":false,"error":"incomplete_ledgers_dependency"})
    };
    let head_seed_veto =
        match audiveris_native::head_seed_veto::analyze(audiveris_native::head_seed_veto::Input {
            binary: &binary,
            grid: &grid_ir,
            seeds: &stem_seeds,
            ledgers: &ledger_post_analysis,
            shared_sig: &common_graph.snapshot(),
            raw_search: &head_seed_search,
            main_interline: sc.interline.main,
            max_stem: stem_seeds["stem_scaling"]["stem_scale"]["max"]
                .as_u64()
                .unwrap_or(0) as usize,
        }) {
            Ok(report) => report,
            Err(error) => {
                serde_json::json!({"seed_veto_complete":false,"heads_complete":false,"error":error})
            }
        };
    let head_range_search = if let (Some(images), Some(context)) = (&beam_images, &beam_context) {
        match audiveris_native::head_range_search::analyze(
            audiveris_native::head_range_search::Input {
                binary: &binary,
                head_spots: &images.head_spots,
                grid: &grid_ir,
                headers: &headers,
                seeds: &stem_seeds,
                ledgers: &ledger_post_analysis,
                seed_veto: &head_seed_veto,
                shared_sig: &common_graph.snapshot(),
                geometry: &context.geometry,
                main_interline: sc.interline.main,
                max_stem: stem_seeds["stem_scaling"]["stem_scale"]["max"]
                    .as_u64()
                    .unwrap_or(0) as usize,
            },
        ) {
            Ok(report) => report,
            Err(error) => serde_json::json!({"range_black_void_prefix_complete":false,
                "range_search_complete":false,"heads_complete":false,"error":error}),
        }
    } else {
        serde_json::json!({"range_black_void_prefix_complete":false,"range_search_complete":false,
            "heads_complete":false,"error":"incomplete_head_spots_dependency"})
    };
    let head_range_catalog = if let (Some(images), Some(context)) = (&beam_images, &beam_context) {
        match audiveris_native::head_range_search::analyze_catalog(
            audiveris_native::head_range_search::Input {
                binary: &binary,
                head_spots: &images.head_spots,
                grid: &grid_ir,
                headers: &headers,
                seeds: &stem_seeds,
                ledgers: &ledger_post_analysis,
                seed_veto: &head_seed_veto,
                shared_sig: &common_graph.snapshot(),
                geometry: &context.geometry,
                main_interline: sc.interline.main,
                max_stem: stem_seeds["stem_scaling"]["stem_scale"]["max"]
                    .as_u64()
                    .unwrap_or(0) as usize,
            },
        ) {
            Ok(report) => report,
            Err(error) => serde_json::json!({"range_catalog_candidates_complete":false,
                "range_search_complete":false,"heads_complete":false,"error":error}),
        }
    } else {
        serde_json::json!({"range_catalog_candidates_complete":false,"range_search_complete":false,
            "heads_complete":false,"error":"incomplete_head_spots_dependency"})
    };
    let head_candidate_filter = match audiveris_native::head_candidate_filter::analyze(
        &head_range_catalog,
        sc.interline.main,
    ) {
        Ok(report) => report,
        Err(error) => {
            serde_json::json!({"range_aggregation_complete":false,"heads_complete":false,"error":error})
        }
    };
    let head_glyph_retrieval = if let Some(context) = &mut beam_context {
        match audiveris_native::head_seed_search::prepare(
            &binary,
            &grid_ir,
            &stem_seeds,
            &ledger_post_analysis,
        )
        .and_then(|(table, _)| {
            audiveris_native::head_glyph_retrieval::analyze(
                audiveris_native::head_glyph_retrieval::Input {
                    binary: &binary,
                    grid: &grid_ir,
                    seed_veto: &head_seed_veto,
                    catalog: &head_range_catalog,
                    main_interline: sc.interline.main,
                    table: &table,
                    shared_sig: &common_graph.snapshot(),
                    max_stem: stem_seeds["stem_scaling"]["stem_scale"]["max"]
                        .as_u64()
                        .unwrap_or(0) as usize,
                },
                &mut context.registry,
            )
        }) {
            Ok(report) => report,
            Err(error) => {
                serde_json::json!({"head_glyph_retrieval_complete":false,"heads_complete":false,"error":error})
            }
        }
    } else {
        serde_json::json!({"head_glyph_retrieval_complete":false,"heads_complete":false,"error":"missing_head_glyph_registry"})
    };
    let mut shared_sig = common_graph.snapshot();
    shared_sig["covered_substages"] = serde_json::json!(["GRID"]);
    if beam_candidates["initial_beam_candidates_complete"] == true {
        shared_sig["covered_substages"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("BEAMS.INITIAL_CANDIDATES"));
    }
    if later_beams["extensions_complete"] == true {
        shared_sig["covered_substages"]
            .as_array_mut()
            .unwrap()
            .extend([
                serde_json::json!("BEAMS.EXTENSIONS"),
                serde_json::json!("BEAMS.LATE_HOOKS"),
                serde_json::json!("BEAMS.GROUPS"),
            ]);
    }
    if later_beams["multiple_rest_boundary_complete"] == true {
        shared_sig["covered_substages"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("BEAMS.MULTIPLE_RESTS"));
    }
    if later_beams["epilog_complete"] == true {
        shared_sig["covered_substages"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("BEAMS.EPILOG"));
    }
    if ledger_registration["builder_complete"] == true {
        shared_sig["covered_substages"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("LEDGERS.BUILDER"));
    }
    if ledger_post_analysis["post_analysis_complete"] == true {
        shared_sig["covered_substages"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("LEDGERS.EPILOG"));
    }
    let head_registration = if let Some(context) = &mut beam_context {
        match audiveris_native::head_seed_search::prepare(
            &binary,
            &grid_ir,
            &stem_seeds,
            &ledger_post_analysis,
        )
        .and_then(|(table, _)| {
            audiveris_native::head_glyph_retrieval::analyze_for_registration(
                audiveris_native::head_glyph_retrieval::Input {
                    binary: &binary,
                    grid: &grid_ir,
                    seed_veto: &head_seed_veto,
                    catalog: &head_range_catalog,
                    main_interline: sc.interline.main,
                    table: &table,
                    shared_sig: &common_graph.snapshot(),
                    max_stem: stem_seeds["stem_scaling"]["stem_scale"]["max"]
                        .as_u64()
                        .unwrap_or(0) as usize,
                },
                &mut context.registry,
            )
        })
        .and_then(|live| {
            let mut report = audiveris_native::head_registration::analyze(
                &live,
                &mut common_graph,
                sc.interline.main,
            )?;
            report["staff_dependency_complete"] = live["staff_dependency_complete"].clone();
            report["live_glyph_retrieval"] = live;
            report["shared_sig"]["covered_substages"] = shared_sig["covered_substages"].clone();
            report["shared_sig"]["covered_substages"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!("HEADS.REGISTRATION_AND_EPILOG"));
            Ok(report)
        }) {
            Ok(report) => report,
            Err(error) => {
                serde_json::json!({"head_registration_complete":false,"staff_dependency_complete":false,"heads_complete":false,"error":error})
            }
        }
    } else {
        serde_json::json!({"head_registration_complete":false,"staff_dependency_complete":false,"heads_complete":false,"error":"missing_head_glyph_registry"})
    };
    let last_completed_substage = if ledger_post_analysis["post_analysis_complete"] == true {
        "LEDGERS.EPILOG"
    } else if ledger_registration["builder_complete"] == true {
        "LEDGERS.BUILDER"
    } else if ledger_candidates["candidate_prefix_complete"] == true {
        "LEDGERS.CANDIDATE_PREFIX"
    } else if later_beams["epilog_complete"] == true {
        "BEAMS.EPILOG"
    } else if later_beams["multiple_rest_boundary_complete"] == true {
        "BEAMS.MULTIPLE_RESTS"
    } else if later_beams["groups_complete"] == true {
        "BEAMS.GROUPS"
    } else if beam_candidates["initial_beam_candidates_complete"] == true {
        "BEAMS.INITIAL_CANDIDATES"
    } else if beam_prolog["beam_prolog_complete"] == true {
        "BEAMS.PROLOG"
    } else if beam_images.is_some() {
        "BEAMS.SPOT_PREPARATION"
    } else {
        "STEM_SEEDS"
    };
    let (staff_mask, note_preparation) = audiveris_native::note_mask::from_grid(&binary, &grid_ir)?;
    let output_pixels = if mode == "staff-mask" {
        staff_mask.pgm()
    } else if mode == "beam-spots" {
        beam_images
            .as_ref()
            .ok_or("missing_beam_spot_output")?
            .binarized
            .pgm()
    } else {
        residual.pgm()
    };
    // Preserve the original HEAD_SPOTS release boundary.
    drop(beam_images);
    let mut page_report = serde_json::json!({"schema":if mode=="staff-mask"{"audiveris-native.experimental-staff-mask.v1"}else if mode=="beam-spots"{"audiveris-native.experimental-beam-spots.v1"}else{"audiveris-native.experimental-page-glyphs.v1"},"line_completion":completion,"page_glyphs":result,"grid_ir":grid_ir,"note_preparation":note_preparation,"header_preparation":header_preparation,"clef_candidates":clef_candidates,"clef_registration":clef_registration,"key_preparation":key_preparation,"key_peaks":key_peaks,"key_slices":key_slices,"key_components":key_components,"key_fallback":key_fallback,"key_process":key_process,"key_column":key_column,"time_candidates":time_candidates,"time_column":time_column,"headers":headers,"stem_seeds":stem_seeds,"beam_preparation":beam_preparation,"beam_prolog":beam_prolog,"beam_candidates":beam_candidates,"shared_sig":shared_sig,"later_beams":later_beams,"ledger_candidates":ledger_candidates,"ledger_registration":ledger_registration,"ledger_post_analysis":ledger_post_analysis,"head_seed_search":head_seed_search,"head_seed_veto":head_seed_veto,"head_range_search":head_range_search,"head_range_catalog":head_range_catalog,"head_candidate_filter":head_candidate_filter,"head_glyph_retrieval":head_glyph_retrieval,"head_registration":head_registration,"last_verified_heads_substage":if head_registration["head_registration_complete"]==true && head_registration["staff_dependency_complete"]==true{"HEADS.REGISTRATION_AND_EPILOG"}else{"HEADS.PARTIAL"},"last_completed_stage":if ledger_post_analysis["ledgers_complete"]==true{"LEDGERS"}else if later_beams["beams_complete"]==true{"BEAMS"}else{"STEM_SEEDS"},"last_completed_substage":last_completed_substage,"musicxml_complete":false});
    if let Some((catalog, cue)) = &declared {
        page_report["head_catalog_control"] = serde_json::json!({"family":catalog.family,"stored_point_size":catalog.stored_point_size,"cue_first_staff":cue,"scope":"HEADS-only catalog/spacing control after default native upstream phases; actual other-font/special-staff page coverage remains uncertified."});
    }
    if let Some(catalog) = &automatic_catalog {
        let cue = grid_ir["staffs"]
            .as_array()
            .is_some_and(|staffs| staffs.iter().any(|s| s["small"] == true));
        page_report["head_template_selection"] = if !beam_prolog["head_scale"]["music_font"]
            .is_null()
        {
            serde_json::json!({"source":"original_measured_head_scale","family":catalog.family,"stored_point_size":catalog.stored_point_size,"original_music_font_scale_present":true,"scope":"exact bounded template catalog at the measured original head size; unsupported staff sizes still refuse; broad page parity unclaimed"})
        } else if cue {
            serde_json::json!({"source":"original_unmeasured_default_cue","family":catalog.family,"stored_point_size":catalog.stored_point_size,"original_music_font_scale_present":false,"scope":"exact Bravura oval templates at staff28/sheet44 on the observed five-line 7/11 cue page; other sizes/families and full24 page propagation remain uncertified"})
        } else {
            serde_json::json!({"source":"original_unmeasured_default_ordinary","family":catalog.family,"stored_point_size":catalog.stored_point_size,"original_music_font_scale_present":false,"scope":"ordinary uniform five-line interline11, default Bravura44 oval templates when no font scale was measured; final selection, rhythm and MusicXML remain unfinished"})
        };
    }
    let success = declared.is_none()
        || head_registration["head_registration_complete"] == true
            && head_registration["staff_dependency_complete"] == true;
    Ok(OwnedPage {
        report: page_report,
        output_pixels,
        graph: common_graph,
        registry: beam_context.map(|p| p.registry),
        success,
    })
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn ordinary_unmeasured_default_and_existing_cue_use_known_banks() {
        let ordinary = json!({"staffs":[{"specific_interline":11,"lines":[0,1,2,3,4]}]});
        assert_eq!(
            unmeasured_catalog(&ordinary, 11).unwrap().stored_point_size,
            44
        );
        let cue = json!({"staffs":[{"specific_interline":7,"small":true}]});
        assert_eq!(
            unmeasured_catalog(&cue, 11)
                .unwrap()
                .staff_point_size(7, 11)
                .unwrap(),
            28
        );
    }
    #[test]
    fn unmeasured_empty_unknown_size_and_special_staff_remain_unsupported() {
        assert!(unmeasured_catalog(&json!({"staffs":[]}), 11).is_none());
        for row in [
            json!({"specific_interline":12,"lines":[0,1,2,3,4]}),
            json!({"specific_interline":11,"lines":[0]}),
            json!({"specific_interline":11,"lines":[0,1,2,3,4],"drum":true}),
            json!({"specific_interline":11,"lines":[0,1,2,3,4],"tablature":true}),
        ] {
            assert!(unmeasured_catalog(&json!({"staffs":[row]}), 11).is_none());
        }
        assert!(
            unmeasured_catalog(
                &json!({"staffs":[{"specific_interline":11,"lines":[0,1,2,3,4]}]}),
                12
            )
            .is_none()
        );
    }
}
