# Rebuilt next stack with null unselected source

```sh
cargo test --offline --locked -j2 --lib original_null_unselected_source
```

The new pinned original control compares three method boundaries after genuine
source Measure reset. Actual PageRhythm processes the next stack: four fresh
Voice bodies and four new timed Slots are constructed, target actual duration
is5/8, with termination-3/8 or-1/2. Chord onset/duration/end, exact cached Voice
references, new Slot identity, old target slots and manual relation identity
are observed. Refinement subsequently throws NullPointerException at the null
previous BeamGroup predecessor, after nullable incoming tie handling.

Production2a3bed1 already agrees. Native public Page processing fails atomically
without inventing the null source, skipping refinement or renumbering a report.
Pre-call Graph/actual Chord storage/layout, old live/retired owners and clocks,
manual relations/report, cached Slot identities and Slot storage survive.
Separate actual stack processing matches the original exception-state new
Voice/Chord/Slot clocks and actual5/8 before the system guard fails. This is not
Java exception rollback parity. Test observations now address the actual source
or target Measure so new target references are not mistaken for source bodies.

Seven relevant targets plus fmt/clippy pass. Production code is unchanged;
retained63+8 CLI XML/XSD and six-action Rational7/64 XML/XSD remain explicitly
at source2a3bed1, whose production code is identical. They are not described as
new executions. Original task/ce785c9 and prior original acquisitions remain
preserved; no649 repeat, duplicate archives, new auth/permissions or Mac/GHA.

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/existing-parallel-beam.omr \
  --probe VoicePageNullRhythmProbe --output /tmp/new-page-null-control
```
