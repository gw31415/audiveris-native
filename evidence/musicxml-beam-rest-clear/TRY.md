# Pre-reset clear on actual owners

```sh
cargo test --offline --locked -j2 --lib beam_rest_clear_tests
```

The native actual-measure clear backend reads its typed Chords and cached real
beam getters, collects actual BeamGroup members, removes only automatic
BeamRest registrations and performs the original beam-abnormal callback. It does
not reset any clocks, create Voices or read an ownership plan from a report.
RestChord beam-list objects are retained independently from rhythm clocks.
Actual note/chord cache invalidation clears their cache; legitimate timing reset
preserves it. This explicit clear pass remains separate from normal rhythm until
original detection is compared. All public/global completion flags stay false.

Six original cases/30 boundaries compare source/unselected RestChord membership,
manual relation registration identity, repeated clear, exact old Voice objects,
slot lists, termination/excess, expected/actual/excess clocks, actual Chord clocks
and Slot references, cached beam-list objects, legitimate Measure reset and
explicit Rest cache invalidation. The native clear transaction also rejects a
later invalid typed endpoint after staging a preceding removal, preserving all
actual Chord fields/storage, actual Slot references/storage, live/retired Voice
objects and clocks, old beam caches and report/layout. Repair/retry succeeds.
Invalid measure requests mutate nothing. Original null/unassigned rest Voice is
preserved, not silently skipped into rhythm or supplied with a new Voice ID.

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/existing-parallel-beam.omr \
  --probe BeamRestClearProbe --output /tmp/new-beam-rest-clear-control
```

Current ordinary default/selected CLI remains guarded; clear is not exposed as a
serialized correction/owner-input option. No original task/environment mutation,
ce785c9 acquisition/regeneration,649 repeat, new auth/permissions, Mac/GHA or
model/speed/environment change occurred.
