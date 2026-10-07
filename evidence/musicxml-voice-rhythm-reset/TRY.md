# Actual Voice owners survive legitimate rhythm reset

```sh
cargo test --offline --locked -j2 --lib music_voices::lifecycle_tests
```

The already acquired independent original VoiceLifecycle control uses pinned
Java9e1e55cd2746037d059345881c53e6a6754bffbd. Its saved24 observations across
two cases are now compared against native actual owners. The previous16 purge,
re-add and number-reuse observations are unchanged; eight further boundaries
cover real Measure.resetRhythm, repeat, MeasureStack.resetRhythm and a distinct
rebuilt Voice1. No original evidence was regenerated for this native unit.

Original Measure reset clears its live Voice registration and Chord Voice/Slot/
time references. It leaves retired actual Voice members/slot entries/termination/
excess and cached duration intact. Stack slots, actual1 and excess1/2 survive
Measure reset, then the separate stack reset clears them; expected1 and manual
Chord0 survive both. The old four-slot Voice and the zero-slot reused Voice keep
separate identities after both leave registration. Rebuilt Voice1 has a third
identity. Actual typed Chords and Measure slot storage survive Measure reset.
Relation identity/manual observations remain unchanged within each implementation;
full original/native SIG graph equality is not claimed.

Source448aca1 reproduced loss of the reused actual Voice after Measure reset.
The common actual owner now retains each live Voice before legitimate Chord
references are cleared. A later invalid Chord identity fails after staged owner
retirement and earlier Chord resets; actual Graph, live/retired Voice bodies and
identities, typed Chords, slots, expected/termination/excess/manual fields remain
unchanged. Repair/retry matches the original Measure-reset boundary. This is
native atomic failure policy, not original Java exception/controller rollback
parity. General history and Part/Measure ownership remain pending.

No report-based ownership reconstruction or callback-before clock erasure is
introduced. Existing CLI admission stays guarded and all completion flagsfalse.
The original task/environment and unobtained ce785c9 are untouched; historical649
is not rerun. See VALIDATION.json and the preceding original fixture provenance.
