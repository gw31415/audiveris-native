// SPDX-License-Identifier: AGPL-3.0-or-later
//! AbstractChordInter.setAndPushTime/pushTime: original sequence -> group ->
//! classified tie order, using actual chord ownership and live SIG relations.
//! Resolved normal-chord duration on the current exact grid only. Automatic
//! relation acquisition, measure-rest duration and general Rational are pending.
use crate::{music_measures::Measure, music_voices::Voice, shared_sig::Graph};
use serde_json::{Value, json};

fn chord(snapshot: &Value, id: usize) -> Result<&Value, String> {
    let n = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .ok_or("music_chord_time_live_chord_required")?;
    if !matches!(
        n["kind"].as_str(),
        Some("HeadChordInter" | "RestChordInter")
    ) {
        return Err("music_chord_time_ordinary_chord_required".into());
    }
    Ok(n)
}
fn same_measure(snapshot: &Value, a: usize, b: usize) -> Result<bool, String> {
    let identity = |id| -> Result<(u64, u64, u64), String> {
        let n = chord(snapshot, id)?;
        Ok((
            n["origin"]["system"]
                .as_u64()
                .ok_or("music_chord_time_system_required")?,
            n["part_id"]
                .as_u64()
                .ok_or("music_chord_time_part_required")?,
            n["measure_index"]
                .as_u64()
                .ok_or("music_chord_time_measure_required")?,
        ))
    };
    Ok(identity(a)? == identity(b)?)
}
fn set_time(graph: &mut Graph, id: usize, time: Option<u32>) -> Result<(), String> {
    let snapshot = graph.snapshot();
    let mut n = chord(&snapshot, id)?.clone();
    let key = serde_json::from_value(n["origin"].clone())
        .map_err(|_| "music_chord_time_origin_required")?;
    n["time_offset"] = json!(time.map(crate::music_time::format_ticks));
    graph.vertex(key, n, false)?;
    Ok(())
}
fn end_time(graph: &Graph, id: usize) -> Result<Option<u32>, String> {
    let snapshot = graph.snapshot();
    let n = chord(&snapshot, id)?;
    let (Some(time), Some(duration)) = (n["time_offset"].as_str(), n["duration"].as_str()) else {
        return Ok(None);
    };
    crate::music_time::parse_ticks(time)?
        .checked_add(crate::music_time::parse_ticks(duration)?)
        .map(Some)
        .ok_or_else(|| "music_time_tick_overflow".into())
}
enum Action {
    Push(usize),
    Sequence(usize),
    Group(usize),
    Tie(usize),
    Forward(usize, usize),
}
/// Original recursive traversal is represented by a stack to preserve call
/// order without unbounded Rust recursion. Cycles/unbounded paths explicitly
/// refuse atomically; they are not claimed as original successful scores.
pub(crate) fn set_and_push(
    graph: &mut Graph,
    voices: &[Voice],
    measures: &[Measure],
    id: usize,
    time: Option<u32>,
) -> Result<(), String> {
    let mut staged = graph.clone();
    set_time(&mut staged, id, time)?;
    let mut actions = if time.is_some() {
        vec![Action::Push(id)]
    } else {
        Vec::new()
    };
    let mut count = 0;
    while let Some(action) = actions.pop() {
        count += 1;
        if count > 4096 {
            return Err("music_chord_time_propagation_budget".into());
        }
        match action {
            Action::Push(id) => {
                actions.push(Action::Tie(id));
                actions.push(Action::Group(id));
                actions.push(Action::Sequence(id));
            }
            Action::Sequence(id) => {
                let snapshot = staged.snapshot();
                if let Some(edge) = snapshot["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["kind"] == "NextInVoiceRelation" && e["source"] == id)
                {
                    let target = edge["target"]
                        .as_u64()
                        .ok_or("music_chord_time_target_required")?
                        as usize;
                    actions.push(Action::Forward(id, target));
                }
            }
            Action::Tie(id) => {
                for target in crate::music_voices::following_tied_chords(&staged.snapshot(), id)?
                    .into_iter()
                    .rev()
                {
                    actions.push(Action::Forward(id, target));
                }
            }
            Action::Forward(source, target) => {
                if same_measure(&staged.snapshot(), source, target)? {
                    let end = end_time(&staged, source)?;
                    set_time(&mut staged, target, end)?;
                    // Original forwards then calls pushTime even for null.
                    actions.push(Action::Push(target));
                }
            }
            Action::Group(id) => {
                let snapshot = staged.snapshot();
                if let Some(gid) = crate::music_voices::chord_group(&snapshot, id)? {
                    let groups = crate::music_voices::beam_groups(&staged, voices, measures)?;
                    let group = groups
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|g| g["id"] == gid)
                        .ok_or("music_chord_time_group_required")?;
                    let members = group["chords"].as_array().unwrap();
                    let mut previous = None;
                    for row in members {
                        let member = row["id"]
                            .as_u64()
                            .ok_or("music_chord_time_member_required")?
                            as usize;
                        if same_measure(&snapshot, id, member)? {
                            if let Some(prev) = previous {
                                let end = end_time(&staged, prev)?;
                                set_time(&mut staged, member, end)?;
                            }
                            previous = Some(member);
                        }
                    }
                    let last = members
                        .last()
                        .ok_or("music_chord_time_nonempty_group_required")?["id"]
                        .as_u64()
                        .ok_or("music_chord_time_member_required")?
                        as usize;
                    if same_measure(&snapshot, id, last)? {
                        // Group's final chord continues sequence/ties, not group.
                        actions.push(Action::Tie(last));
                        actions.push(Action::Sequence(last));
                    }
                }
            }
        }
    }
    *graph = staged;
    Ok(())
}
