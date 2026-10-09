# Review: current null/header + short-rest changes against 362d129

Result: no concrete correctness blocker found in the reviewed production delta.
This is a source/code review, not a new build, test or image validation result.
No checkout edit or Git mutation was performed. Review snapshot diff:
`/tmp/review-null-header-shortrest-362d129.diff`; reviewed file hashes:
`/tmp/review-null-header-shortrest-362d129-sha256.json`.

## Confirmed decisions

- `src/head_musical_pitch.rs:36-70`: null Clef uses the static default G-clef
  numerical rules without registering or assigning any Clef identity. The new
  absolute-pitch API matches the separate original `ClefInter.absolutePitchOf`
  switch, including C-clef's `28 - roundedClefPitch - roundedHeadPitch` and the
  F-clef special 8va/8vb branches. It is correctly not reconstructed from
  noteStepOf/octaveOf; those original APIs use a different C-clef sign.
  Finite/range guards remain explicit, and percussion musical export is still
  guarded although its absolute-pitch API legitimately returns zero.
- `src/music_rookie_mapping.rs:466-507`: the owned Staff path preserves actual
  same-System/Staff clef lookup, now allowing a truly absent clef to supply
  `None` to the numerical API. It does not borrow the carried XML clef or a
  previous System's physical clef. Non-owned legacy input still uses its
  explicitly supplied Clef.
- `src/music_xml/note_pitch.rs:103-131,289-325`: nullable diatonic reads preserve
  `effective_clef_id = null`; no forged ID. `read_for_export` passes the carried
  logical key fifths to alteration, while ordinary `read` preserves the local
  physical key lookup. `Some(None)` is correctly distinguished from no override.
  Source `PartwiseBuilder.processNote` passes `current.keys[staffIndex]` fifths
  to `HeadInter.getAlteration`; the existing recursive tie path forwards that
  same fifths parameter as source does.
- `src/music_xml/recognized_export.rs:20-74`: the header-state transitions match
  `PartwiseBuilder.processKeys/processKey/processKeyVoid`: explicit same keys
  collapse globally; partial missing Staff keys leave prior exported state;
  a wholly void system-start measure cancels a prior nonzero key globally; an
  initial void emits neither a fabricated zero key nor a fabricated Clef.
  Clefs are compared semantically, not by source Inter ID.
- `src/music_xml/recognized_export.rs:79-159`: the guarded single-clef path uses
  actual Measure and Staff Arcs. Its deskewed x-offset is the same singleton
  Staff overload used by source ClefIterators; it guards clefs after the first
  Slot rather than moving them silently to the measure start. The new helper
  also covers rest-only measures without inventing a Head to obtain a header.
- Short-rest exact fractions, dot factors and Voice eligibility were reviewed
  independently in `/tmp/review-short-rest-duration-voice-362d129.md`. No bug was
  found; the exact duration reader supports shorter fractions while the legacy
  tick API retains its explicit finer-than-sixteenth error.

## Limits that remain explicit, not new findings

1. Physical Key/Clef lookup remains an as-built spatial projection of the live
   Graph, rather than the original ordered Measure membership registry. The
   new writer rejects multiple actual keys/clefs and timed clef changes; it
   does not prove Clef add/remove lifecycle. The existing note-reader/mapping
   lookups likewise are not general edited-score ownership parity. The current
   code and real-target observation accurately qualify that boundary.
2. The real DerWinter helper first compares actual nullable pitch and header
   state. Its internal rhythm/export continuation now encounters the explicit
   implicit-tuplet/conflicting-Slot-time guard. Preserving the failed attempted
   success evidence and checking complete retained-owner/clock rollback is
   appropriate; reaching that guard is not final MusicXML note equivalence.
   That next mapping dependency was intentionally not investigated here.
3. Header controls directly test the state contract, while the real source
   observation covers this page's partial-header/null-Clef case. They do not
   establish all-page, all-part or complete rhythm/export quality acceptance.
4. The full exact-input range, source arithmetic and explicit guards were read;
   no additional cargo, Java, image or XML/XSD run was performed for this review.
   Parent-owned completed validation remains the checkpoint evidence.

Pinned source reviewed at `9e1e55cd2746037d059345881c53e6a6754bffbd`:
ClefInter absolutePitchOf/noteStepOf/octaveOf; Measure getClefBefore,
getMeasureClefBefore, hasSameKeys; MeasureStack.getXOffset; PartwiseBuilder
processKey/processKeys/processKeyVoid, processNote and ClefIterators.push.
Existing Octave /tmp proposals and saved diagnostics were left unchanged.
