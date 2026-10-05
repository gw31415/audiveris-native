// SPDX-License-Identifier: AGPL-3.0-or-later
// Integrated MultipleRestsBuilder boundary; supplied native staff/lag geometry.
use crate::{
    GrayImage,
    beam_candidates::{CandidateInter, SystemCandidates},
    beam_dispatch::Registry,
    beam_groups::GroupPass,
    multiple_rest_admission::StaffPosition,
    multiple_rest_registration::RestGraph,
    sections::Section,
    serif_peaks::{Detector, StaffGeometry},
};
pub struct Context<'a> {
    pub image: &'a GrayImage,
    pub interline: usize,
    pub fore: usize,
    /// Original vLag then hLag order, already restricted to system bounds.
    pub sections: &'a [Section],
    pub staffs: &'a [(u64, StaffGeometry<'a>)],
}
/// Discover all beams, register all rests/serifs, then remove converted beams.
/// Staff association/pitch measurement is supplied; detectors are cached by
/// staff identity and created only after admission requests a side search.
/// The returned graph has phase-local identities; shared GRID/SIG integration
/// and caller rollback after registration resource errors are not implemented.
pub fn process(
    state: &mut SystemCandidates,
    groups: &mut GroupPass,
    registry: &mut Registry,
    context: Context<'_>,
    staff: impl FnMut(&CandidateInter) -> Option<StaffPosition>,
) -> Result<RestGraph, String> {
    let mut profiles = std::collections::BTreeMap::new();
    for &(id, geometry) in context.staffs {
        if id == 0 || profiles.insert(id, geometry).is_some() {
            return Err("duplicate_rest_staff_geometry".into());
        }
    }
    let mut detectors = std::collections::BTreeMap::new();
    let plans = crate::multiple_rest_detection::discover_system(
        state,
        context.interline,
        context.fore,
        staff,
        |id, q| {
            if let std::collections::btree_map::Entry::Vacant(entry) = detectors.entry(id) {
                let geometry = *profiles.get(&id).ok_or("missing_rest_staff_geometry")?;
                entry.insert(Detector::new(context.image, geometry)?);
            }
            detectors
                .get_mut(&id)
                .unwrap()
                .detect(q.x, q.left, q.added_chunk)
        },
    )?;
    let mut graph = RestGraph::default();
    if !plans.is_empty() {
        graph.register(&plans, context.sections, context.interline, registry)?;
        graph.finish_cleanup(state, groups)?;
    }
    Ok(graph)
}

/// Fresh pre-rhythm pass with a shared graph observer. Imports current beam and
/// group identities before conversion, then synchronizes registration/cleanup.
/// Existing GRID vertices and relations are retained by the adapter.
pub fn process_shared(
    state: &mut SystemCandidates,
    groups: &mut GroupPass,
    registry: &mut Registry,
    context: Context<'_>,
    staff: impl FnMut(&CandidateInter) -> Option<StaffPosition>,
    shared: &mut crate::shared_sig::Graph,
) -> Result<RestGraph, String> {
    shared.sync_system(state, groups, &RestGraph::default())?;
    let rests = process(state, groups, registry, context, staff)?;
    shared.sync_system(state, groups, &rests)?;
    Ok(rests)
}
