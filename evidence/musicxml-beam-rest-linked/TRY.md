# Actual linked rest candidates before detection

```sh
cargo test --offline --locked -j2 --lib beam_rest_linked_tests
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/existing-parallel-beam.omr \
  --probe BeamRestLinkedProbe --output /tmp/new-beam-rest-linked-control
```

`OwnedMusic.interleaved_rest_candidates` reads actual measure Chords, lazy real
beam caches and live BeamGroup member getters. It computes relation-only
candidates without adding BeamRest edges, assigning Voices, resetting clocks or
using a report ownership plan. The staged graph commits only successful lazy
getter work. `rest_detection_complete` is false; this query is separate from
normal selected rhythm until original geometry and detection are compared.

Two original short/excess cases compare eight actual intermediate states:
incoming-before-outgoing enumeration, insertion order, direction symmetry,
layered Rest-only closure and cycles, indirect Head exclusion, exact relation
class filters, blacklist priority after closure, foreign-Measure/null-Voice
references and manual BeamRest's actual seed membership. Original Rest cache
objects remain empty and identical through manual addition, clear and genuine
Measure reset. Old Voice bodies, end/excess, expected/actual clocks, actual
Chord/Slot references and manual relation identity remain compared.

An absent actual typed Rest owner referenced late in closure is refused with
`music_rest_actual_chord_required`. Native rollback keeps Graph, physical Chords,
Slot storage, live/retired Voice clocks, cache objects and report/layout; putting
the actual Chord back permits retry. It does not invent an owner or Voice ID.
The first test run stopped on a test-only label lookup into unlabelled target
Heads; lookup now explicitly selects labelled actual Chords. Production guards
were not relaxed. Raw failed logs remain in this Cloud.

See VALIDATION.json for current targeted checks and XML/XSD evidence. Existing
selected CLI proof stays retained; no additional duplicate selected CLI profile,
historical649, archive, new authentication or other environment was used.
Original task/environment and unobtained ce785c9 remain untouched. All full-port
and public scope flags remain false/guarded.
