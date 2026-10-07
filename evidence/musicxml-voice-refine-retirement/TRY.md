# Actual Voice retained through refineSystem purge

```sh
cargo test --offline --locked -j2 --lib music_voices::lifecycle_tests
```

Pinned original `9e1e55cd2746037d059345881c53e6a6754bffbd` loads the real timed
saved double-augmentation owner. Two declared last timeOffsets1/8 and5/4 produce
original termination-5/8 and+1/2/excess1/2 via actual Voice.checkDuration.
Chord0 is declared manual. Voice.resetChords empties only its actual list; old
slot entries, actual Chord references, expected1, actual1 and timing stay intact.
Real Voices.refineSystem removes that Voice from Measure.voices without erasing
it. Repeated purge, re-add of the same Voice object and re-purge retain its
identity. A constructor creates a distinct new Voice1, leaving old slot entries
and clocks with old1. Sixteen original intermediate states across two cases are
compared against actual native objects, not owner reconstruction from reports.

Native previously lost the old object after purge; that failure was reproduced
before the fix. A common Graph owner now receives the actual moved Voice, binds
cached Chords by their existing identity and reuses the same retained entry after
re-add/re-purge. Empty-Chord removal uses that common owner too. Original and
native relation/manual state stays unchanged within each implementation; the
full original and native SIG graphs are not claimed equal. Bounds/image Stem
extent are outside this control and the known five-pixel difference remains.

A later invalid actual Voice in the next Measure makes refinement fail after
the first purge has run in staging. Graph/actual Voices/identity, real Chords,
expected/slot/termination/excess/manual state remain at the starting values;
repair and retry succeed. This is native atomic policy, not original Java UI
exception rollback parity. Genuine Measure/stack reset observations are included
as independent original evidence but remain a next native unit, not a claim
of current reset/history or CLI admission completion.

Reproduce the original controls in a fresh output directory:

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/double-augmentation.omr \
  --output /tmp/new-original-voice-lifecycle
```
