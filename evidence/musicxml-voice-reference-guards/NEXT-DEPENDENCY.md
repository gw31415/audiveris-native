# Next dependency

Original retained HIGH order tie -> first BeamGroup predecessor -> NextInVoice ->
preferred ID is connected. Actual Chord Voice reference is now resolved after
each preceding swap; null is skipped only for ties. Declared original exceptions
for BeamGroup/Next are failure evidence, not automatic recognition/export success.
Native failure rolls back staging; Java partial mutation is not claimed identical.

The existing first-Chord ownership guard still requires a nonnull reference.
Observe original Voice.getFirstChord/rename behavior for that distinct partial
rhythm state before broadening it. General VoiceKind offsets, shared stacks/Part,
refinePage/refineScore adapters and automatic CURVES/tie export remain partial.

For timed support EditingTask, the source explicitly says getDurationSansTuplet
is NOT cached: countDots changes dotsNumber and duration is recomputed by the
getter, while existing Chord Voice/time/Slot survive until Stack/Measure reset.
Native duration is a derived projection. Observe timed original callback entry/
return, nested removals and dynamic duration/endTime before implementing the
corresponding native live duration refresh. Do not erase clocks before callback.
Native resolve_durations already reads live dots_number in legitimate bounded
reprocessing; initial SYMBOLS report is not authority for edited ownership.

Keep require_bounds_only_reprocess and music_dot_rhythm_reset_required until
those callback dependencies and an actual owned do/undo/reprocess are compared.
reprocessStack does not call populateFRATs. General support edits/controller/
PageStep/history/complete cleaner/epilog remain final nonUI parity dependencies.
Historical595@95e191b is not a current full run. All completion flags false.
