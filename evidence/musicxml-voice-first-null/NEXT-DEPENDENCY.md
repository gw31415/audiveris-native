# Next dependency

Retained timed HIGH Voices now follow original null first-Chord semantics as
well as classified tie -> previous-Measure BeamGroup -> NextInVoice -> preferred
ID. Actual membership, not a reconstructed Chord.voice, supplies getFirstChord.
Native event binding still rejects a null reference; this control is not new
image/export admission. Original beam/Next null exceptions remain failure only.

Next observe original timed ordinary augmentation support do/undo callbacks,
actual Part/Measure membership, Voice/slot/time before callback and after impact,
then PageRhythm.reprocessStack with retained expectedDuration and no populateFRATs.
getDurationSansTuplet is explicitly uncached in original; countDots modifies
its live inputs, so duration/endTime may change while old clock/Voice persists.
Native derived duration projection must follow this without prematurely clearing
clocks, rebuilding Part/Measure in a setter or falsifying the initial report.
resolve_durations already recomputes live dots_number in legitimate reprocessing.
Keep timed support/require_bounds_only_reprocess guards until that lifecycle and
actual owned callback/reprocess/rollback are positively compared.

General VoiceKind offsets, shared stacks/multi-staff/Part, refinePage/refineScore
SlurAdapters, automatic CURVES/tie MusicXML and complete controller/PageStep/
history/cleaner/epilog remain nonUI parity work; no planned original omissions.
Historical595@95e191b is not a current full run. All completion flags stay false.
