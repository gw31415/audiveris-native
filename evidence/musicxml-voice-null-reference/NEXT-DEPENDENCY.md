# Next dependency

Voice ID changes preserve an actual null Chord.voice even while Voice.chords
retains it. getTiedId skips the null source, tries the next live tie and stops at
the first nonnull Voice. Declared partial-rhythm controls are not image acceptance.

NextInVoice and BeamGroup predecessor branches differ: the original dereferences
Chord.getVoice().getId without the tie getter's null skip. Observe the original
NextInVoice-null boundary and resolve actual current Chord references for these
branches, not retained membership alone or a previously repeated numeric ID.
Keep native failure transactional and separate original exception from success.

Timed support edits remain guarded until original callback-time duration semantics
and retained reprocessing are connected with clocks intact. No populateFRATs in
reprocessStack, no report provenance bypass. CURVES/general Voice/shared stacks/
multi-staff/PageStep/complete cleaner and epilog remain final-parity dependencies.
All completion flags false; historical595 at95e191b is not a current full rerun.
