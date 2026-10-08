# Retained sheet Skew and nearest real beam

```sh
cargo test --offline --locked -j2 --lib beam_rest_nearest_tests
```

Measured SheetSkew now moves from native image Page to its actual item owner.
No mutable report can supply or replace it; missing/nonfinite measurements do
not silently become zero. Existing zero-skew image admission still checks its
actual retained Skew and report/provenance scope. The ordinary scope is not
expanded by geometric kernel controls at other slopes.

The explicit actual-measure nearest query stages lazy getters, validates actual
group membership and reads live beam medians/Containment order. It matches
pinned skewed vertical intersections, full-only eligibility, inclusive endpoints,
strict-distance member ties and null results. It does not add relations, assign
Voices, reset clocks or build owners from a report. Two original controls cover
432 queries. Distances agree within1e-10; selected real member labels are exact.
Actual source image geometry matches independent original fresh BEAMS; stored
book's one-decimal geometry has its own graph-copy comparison. Real source
median is never replaced to hide that persistence boundary.

Queries hold real old Voice/Slot clocks, termination/excess, expected/actual,
manual status, actual Chord and Slot references/storage, and Rest cache objects.
An altered report Skew cannot change the retained physical query. A later invalid
member median refuses the staged query after a valid preceding member and keeps
Graph, Chord/Slot storage, live/retired clocks, cache refs and report/layout.
Repair/retry uses the same actual Chords and no new Voice IDs.

Original reproduction (existing book; no archive creation):

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/existing-parallel-beam.omr \
  --probe BeamRestNearestProbe \
  --fresh-beams fixtures/official-beam-rest-nearest/original-fresh-beam.json \
  --output /tmp/new-beam-rest-nearest-control
```

Fresh geometry provenance is retained; do not repeat the earlier BEAMS runner
or create another archive to recover it. Its unexpected new auto-save was
removed; source book/task and unobtained ce785c9 were preserved. Full detection,
BeamRest.added and implicit-tuplet passes remain unclaimed and flags stay false.
See VALIDATION.json for current limited target/XML-XSD checks; no649 repetition,
new auth/permissions, Mac/GHA or environment/model/speed change.
