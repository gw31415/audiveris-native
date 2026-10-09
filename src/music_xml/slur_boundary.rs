// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{music_slurs::State, shared_sig::Graph};

pub(crate) fn require_no_slurs(graph: &Graph, state: Option<&State>) -> Result<(), String> {
    // These output paths have no Slur visits. Either retained owners or live
    // SIG material must refuse before an event stream can silently omit them.
    if state.is_some_and(|state| {
        !state.slurs.is_empty() || state.parts.iter().any(|part| !part.slurs.is_empty())
    }) || graph.has_live_slur_material()
    {
        return Err("musicxml_slur_writer_required_for_retained_or_live_Slur".into());
    }
    Ok(())
}
