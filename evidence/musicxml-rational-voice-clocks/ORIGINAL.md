# Original dependencies

Audiveris5.11 source `9e1e55cd2746037d059345881c53e6a6754bffbd`:

- `math/Rational`: reduced arithmetic, compareTo, minus, gcd.
- `sheet/rhythm/Voice`: completeSlotTable, getDuration, checkDuration.
- `sheet/rhythm/StackRhythm`: generateStackSlots/readStackActualDuration.
- `score/Page`: computeDurationDivisor over standard Chords across the page,
  then simpleDurationOf(QUARTER).
- `score/PartwiseBuilder`: processMeasure Voice/BEGIN traversal with actual
  duration/onset, backup/forward; ending-forward code is disabled in original.

Source URL prefix:
https://github.com/Audiveris/audiveris/tree/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/

`scripts/java/RationalVoiceClockProbe.java` runs real official5.11 objects from
`double-augmentation.omr`, with a declared manual alternate HeadStem addition
and actual PageRhythm.reprocessStack. It observes real Slot/Voice/cache timing
and original Partwise stream. Native scheduling is explicitly declared in the
control; automatic mapper success is not inferred from this comparison.
Native checked nonnegative-u32 reduced fractions and signed termination are
still a bounded subset of general original signed Rational/tuplet processing.
