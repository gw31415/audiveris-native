# Captured group detection

Original BeamGroupInter.detectInterleavedRests is now an explicit native operation
on existing Chord/Measure/Staff/Curve owners. Each group captures its real Chords,
Separate blacklist, transitive Next-only whitelist and actual Measure insertion
ordered plain candidates before its own white additions. Each adjacent captured
pair adds strict-between white rests to the nearest full Beam, or keeps null,
then performs original plain competition. Later groups collect after earlier
changes. No candidate report constructs owners and no callback resets clocks.

The pinned Java probe compares ten independent short/excess cases: incoming and
transitive white before plain, black winning a white conflict, strict tail-x
endpoints, null nearest, and a captured real RestChord with old manual relation.
A later GraphListener observes fresh added/removed AFTER original SigListener;
native entry additionally compares held original clocks and initial dy/portion.
Actual old Voice/Slot, expectedDuration/end/excess, physical Chord/Slot storage,
Staff/Curve refs and cached Rest beam lists survive the complete detection.

Comparison found two previously uncovered contracts. Beam/group sorting must use
held HeadChord bounds even when its live SIG geometry differs. A RestChord's
zero-length head/tail intersection can be NaN; original PointUtil.rounded's
Math.rint-to-int narrowing gives zero. Neither control coerces the recorded
native automatic Stem5px difference. Late plain member failure after earlier
white addition discards staged changes and callback queue; repair retries on the
same physical owners with original manual identities and unused registrations.

Raw original attempts remain separate: first compiler import omission, then
successful v2. Original book/JAR unchanged, no new archives/whole-profile/full649
runs. Fixture clock interning is lossless and records both raw and compact hashes.
Full MeasureRhythm preparation still requires implicit tuplets BEFORE clear/detect
and legitimate Measure reset, and later implicit generation/merge. Full nonUI and
public completion flags remain false. See VALIDATION.json and fixture provenance.
