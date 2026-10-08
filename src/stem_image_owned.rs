// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image HEADS -> items with its actual Graph and physical registry still owned.
//! Per-system caches are initialized; selected stems/relations/epilog remain pending.
use crate::{
    GrayImage, page_runtime, shared_sig::Graph, stem_cache_sig::Retriever, stem_image_head_items,
    stem_image_stumps::Prepared,
};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct OwnedItems {
    pub(crate) staffs: Vec<std::sync::Arc<crate::grid_ir::StaffGeometry>>,
    pub(crate) sheet_skew: crate::sheet_skew::SheetSkew,
    pub(crate) interline: usize,
    pub(crate) catalog: crate::head_staff_catalog::TemplateCatalog,
    pub upstream_report: Value,
    pub upstream_output_pixels: Vec<u8>,
    pub graph: Graph,
    pub caches: BTreeMap<u64, Retriever>,
    pub(crate) free_glyphs: Vec<crate::beam_dispatch::SystemSpots>,
    pub prepared: Prepared,
}
/// No Java, oracle, caller-defined items/endpoints or serialized Graph import.
pub fn prepare_items(image: &GrayImage) -> Result<OwnedItems, Box<dyn std::error::Error>> {
    continue_items(
        image,
        page_runtime::analyze(image, "page-glyphs", None, None)?,
    )
}
/// Takes the actual upstream owner by move. The report is a read-only view;
/// it cannot supply removed-edge identity history or replace the live Graph.
pub fn continue_items(
    image: &GrayImage,
    upstream: page_runtime::OwnedPage,
) -> Result<OwnedItems, Box<dyn std::error::Error>> {
    if !upstream.success
        || upstream.report["schema"] != "audiveris-native.experimental-page-glyphs.v1"
        || upstream.report["head_registration"]["head_registration_complete"] != true
    {
        return Err("complete owned image HEADS required".into());
    }
    let mut expected = upstream.report["head_registration"]["shared_sig"].clone();
    expected
        .as_object_mut()
        .ok_or("missing owned HEADS SIG")?
        .remove("covered_substages");
    if expected != upstream.graph.snapshot() {
        return Err("owned upstream Graph does not match image HEADS".into());
    }
    let catalog = crate::stem_image_head_plan::image_catalog(&upstream.report)?;
    let registry = upstream.registry.ok_or("missing owned HEADS registry")?;
    let prepared =
        stem_image_head_items::prepare_with_registry(image, &upstream.report, Some(registry))?;
    let mut caches = BTreeMap::new();
    for system in upstream.report["head_registration"]["systems"]
        .as_array()
        .ok_or("missing owned HEADS systems")?
    {
        let id = system["system_id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or("invalid owned HEADS system")?;
        if caches.insert(id, Retriever::new(id)?).is_some() {
            return Err("duplicate owned HEADS system".into());
        }
    }
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let interline = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code())?
        .interline
        .main;
    Ok(OwnedItems {
        staffs: upstream.staffs,
        sheet_skew: upstream.sheet_skew,
        interline,
        catalog,
        upstream_report: upstream.report,
        upstream_output_pixels: upstream.output_pixels,
        graph: upstream.graph,
        caches,
        free_glyphs: upstream.free_glyphs,
        prepared,
    })
}
