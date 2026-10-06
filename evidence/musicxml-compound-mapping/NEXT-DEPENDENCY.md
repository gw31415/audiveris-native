# Next compound Voice/export dependency

Separated full SlotMapper.mapChords now applies multiple incoming ordinary chords
to actual owned Voices with per-member clocks. Three original saved-object controls
compare measured distances/tie order, nonimplicit blacklist/reselection, actual
Voice membership/new Voice, chord times and retained candidate references. This
is not automatic polyphonic image admission; starting-parallel and writer guards
remain. Known grouped conflicts and implicit-tuplet analyze/shrink remain separate.

Next join the solved owned Voices to original StackRhythm.generateStackSlots and
PartwiseBuilder.processMeasure voice-by-voice BEGIN traversal: preserve CONTINUE
entries, insert backup from the actual previous cursor and forward only for a
gap. A new Voice must never be overwritten or silently serialized. Original
measure-rest Voices use their dedicated chord and stack actual duration. The
current public image guard should only change once starting-chord assignment,
actual multi-Voice event/export integration and a same-image original result agree.
Exact Rational/tuplets, Merged/shared stacks, lyrics and complete cleaner stay in
the nonUI ledger. Existing image commands and 36 events stay unchanged; all595 is
historical 95e191b, all completion flags remain false.
