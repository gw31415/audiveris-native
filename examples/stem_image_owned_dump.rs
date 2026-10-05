// SPDX-License-Identifier: AGPL-3.0-or-later
//! Evidence for the actual in-process image owner; no declared parts or oracle input.
use audiveris_native::{GrayImage, page_runtime, stem_image_owned};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: stem_image_owned_dump IMAGE NEW_PROOF.json".into());
    }
    let image_bytes = fs::read(&args[1])?;
    let image = GrayImage::decode(&image_bytes)?;
    let upstream = page_runtime::analyze(&image, "page-glyphs", None, None)?;
    let registry = upstream
        .registry
        .as_ref()
        .ok_or("missing actual registry")?;
    let pointers: Vec<_> = (1..=registry.len())
        .map(|id| registry.get(id).map(|e| e.glyph.runs.as_ptr()))
        .collect::<Result<_, _>>()?;
    let snapshot = registry.snapshot();
    let private_graph_before = format!("{:?}", upstream.graph);
    let graph_before = upstream.graph.snapshot();
    let owned = stem_image_owned::continue_items(&image, upstream)?;
    let after = owned.prepared.registry.snapshot();
    let mut retained = 0;
    for (i, ptr) in pointers.iter().enumerate() {
        if owned.prepared.registry.get(i + 1)?.glyph.runs.as_ptr() != *ptr
            || after[i] != snapshot[i]
        {
            return Err("upstream physical storage changed".into());
        }
        retained += 1;
    }
    let private_graph_after = format!("{:?}", owned.graph);
    if private_graph_before != private_graph_after || graph_before != owned.graph.snapshot() {
        return Err("actual upstream Graph history changed".into());
    }
    let mut heads = serde_json::to_vec(&owned.upstream_report)?;
    heads.push(b'\n');
    let r = &owned.prepared.report;
    let proof = json!({"status":"passed","source_image_sha256":format!("{:x}",Sha256::digest(&image_bytes)),"entire_old_HEADS_report_sha256":format!("{:x}",Sha256::digest(&heads)),"actual_native_image_pipeline_executed":true,"serialized_Graph_reconstructed":false,"upstream_physical_run_allocations_retained":retained,"upstream_registry_entries_groups_and_IDs_exact":true,"final_registry_count":owned.prepared.registry.len(),"actual_private_Graph_history_retained":true,"private_Graph_debug_sha256":format!("{:x}",Sha256::digest(private_graph_before.as_bytes())),"initialized_cache_systems":owned.caches.keys().collect::<Vec<_>>(),"cache_entries_or_stem_SIG_vertices_created":false,"corner_count":r["corner_count"],"final_head_item_count":r["final_head_item_count"],"gap_item_count":r["gap_item_count"],"physical_endpoint_stem_glyph_grade_selection_complete":false,"page_cache_sig_complete":false,"cleaner_epilog_complete":false,"heads_complete":false,"stems_complete":false,"context":"Original XML-restored profile1/zero-skew Laura context; fresh original transient/full automatic STEMS unverified"});
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    serde_json::to_writer_pretty(&mut output, &proof)?;
    writeln!(output)?;
    Ok(())
}
