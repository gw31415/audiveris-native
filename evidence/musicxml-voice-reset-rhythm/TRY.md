# Genuine reset followed by actual selected stack rhythm

```sh
cargo test --offline --locked -j2 --lib reset_rhythm_tests
```

The existing parallel book is loaded independently for four original cases,
comparing20 method boundaries, including actual MeasureRhythm.process before
Stack slots and actual StackRhythm.process before system refinement. Empty
old Voice7/support2/fresh1 bodies, original slots and signed termination/excess
survive reset; two actual new Voices are constructed by the existing factory.
Seven CompoundSlots precede six fresh stack Slots, which reuse numbers without
reusing old objects. Cached expectedDuration, actual BeamGroup predecessor,
incoming tie, manual Next/preferred priority and target bodies are compared.

The initial native test failed at measure_processed: Chords had null Slot
references. Number-only identity observations also incorrectly treated rebuilt
stack Slots as old Slots. BEFORE-SLOT-REFERENCES.log records both failures.
The shared ordinary mapper now binds actual CompoundSlot identities to Chords;
timed stack construction binds fresh Slot identities. Graph staging clones
retain those references, and genuine Measure reset clears them. Candidate
renumbering retains constructor identity, including removed candidates.

OwnedMusic.reprocess_stack_rhythm uses cached expectedDuration, genuine stack/
Measure reset, the common starting/rookie/lifetime mapper, actual finalCheck,
stack Slots and actual-duration read. It stops before system refinement and
publishes no owner report. Its staging preserves actual Chord storage and
untouched Measure Slot storage; guards fail without changing actual Chords,
Graph/manual relations, live/retained Voice identities, clocks or layout.
The duplicate actual-member failure after staged reset is repaired and retried
against the same original full-stack state. Reset followed directly by system
refinement still refuses its null predecessor; no source or Voice ID is invented.

The same mapper remains connected to the existing image/selected export path.
The existing six manual HeadStem Link/Unlink Do/Undo/Redo operations now also
run this actual stack backend before selected export and compare clocks with
the original Rational7/64, underfilled Voice termination and divisions control.
Public automatic/correction admission and global completion flags stay guarded.
This ordinary compared backend does not claim general MeasureRhythm passes,
multipart stacks or controller/history/PAGE/project/persistence parity.

Reproduce the original with the pinned Linux oracle and existing PAGE book:

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/existing-parallel-beam.omr \
  --probe VoiceResetRhythmProbe --output /tmp/new-reset-rhythm-control
```

Original task/environment and unobtained ce785c9 remain untouched. No649 repeat,
archive transfer, new authentication/permissions or Mac/GHA is used.
