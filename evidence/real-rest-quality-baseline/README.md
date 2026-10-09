# Original baseline on the Der Winter rest page

Unmodified original Audiveris at
`9e1e55cd2746037d059345881c53e6a6754bffbd` successfully exported the exact saved
[`der-winter-gray.png`](../../fixtures/real-rest-page/der-winter-gray.png), SHA-256
`10f61bf0d44be190df7177b57c1cd26a31bfbcc68d9ad8f85a76813d51ae59e0`.
The historical run exited 0 in 10.04 seconds. `ORIGINAL-RUN.json` and
`ORIGINAL-CONSOLE.log` preserve its command, JAR and output evidence.
`der-winter-original.musicxml` is byte-for-byte the `der-winter-gray.xml` member of
that run's MXL, with SHA-256
`e04333e06b7c58181824602b4b57367d5c2aad2b2b7811707fa2dc3f3e26c7fb`.
No original recognition or rendering was repeated for this evidence packaging.

The independent CC0 score contains 124 events: 115 pitched notes and nine rests.
The original export also contains 124 events, in two parts with nine measures
each. Counts alone do not establish correctness. Exact global staff, ordinal
measure, rational onset, MIDI pitch or rest, duration and grace comparison finds:

- 102 matching events; precision, recall and F1 each `0.8225806451612904`.
- 22 missing truth events and 22 extra original events, all on global staff 3.
  Two differences concern timing/duration in ordinal measure 3; twenty concern
  pitch in ordinal measures 6–9. These are 22 unmatched pairs, not a claim of
  44 independent mistakes.
- All nine rest events match exactly, including their staff, measure, onset and
  duration. Meter comparisons match as well.

`ORIGINAL-EVENT-QUALITY.json` lists every difference. Its result was independently
recomputed from the unchanged score and copied XML and found equal to the
previous result. This metric does not yet score voice continuity, chord grouping
or notation relationships; for example, the export contains 12 slur elements but
their correctness has not been evaluated here. `BASELINE.json` records these
limits and the exact rest events. Ordinal measures in the comparison are based on
score order, not the MusicXML measure-number strings.

The complete provenance for the older Linux MuseScore 3.2.3 renderer, unchanged
source score and white-background normalization is in
[`fixtures/real-rest-page`](../../fixtures/real-rest-page/README.md).
The source score and original XML are development comparison data only. They
must not supply notes to native recognition.

This unit establishes a real-rest acceptance input and the original's known
errors. It does not assert native quality parity. Next, the native public image
command must recognize these same PNG pixels, then compare pitch, duration,
rests, chords, voices, meter/measures and required notation relationships with
both the original and independent truth, followed by XSD and reader validation.

The first native public command on the same image exited1 after75.57 seconds at
`unsupported_beam_lookup_area_nonconvex`; no XML or report was written.
`NATIVE-FIRST-RESULT.json` and `NATIVE-FIRST-CLI.log` retain that failure. The next
recognition dependency is original Beam VLinker lookup-area geometry and its
non-convex path consumers, before later header/rest/pickup/slur acceptance.
