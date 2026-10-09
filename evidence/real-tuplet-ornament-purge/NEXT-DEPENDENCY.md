# Next recognition dependencies

The goal remains PNG score-image recognition to original-quality MusicXML, not
the historical all-non-UI-API port. No public real-page XML is accepted yet.

1. Der Winter ordinary CLI now reaches `musicxml_arpeggiato_chord_link_pending`.
   Fresh original factory observation proves the one candidate at
   `[805,803,1,16]` is null: the same-System Chord's stem intersects the lookup
   rectangle, but actual head-only bounds do not. Replace the conservative full
   Chord-box admission with actual same-System HeadChord/member lookup at the
   factory boundary. Preserve positive creation as a frontier until compared;
   do not infer a null factory from a Chord disappearing in later LINKS.
2. AnLaura's next checked labels are QUINDICESIMA, G_CLEF, CODA, F_CLEF,
   F_CLEF_SMALL, FERMATA_BELOW and OTTAVA. Fresh source comparison matches all20
   raw native candidates. Source checks reject5 before factory (CODA AboveStaff;
   F/F-small NotWithinStaffHeight). The other15 are real positives:12 OctaveShift,
   one G Clef and2 Fermata. They must not enter a default-null allowlist.
   Ordinary SymbolsLinker removes the2 unlinked Fermatas. Three OctaveShift
   candidates propose two sides to the same actual Chord; source Link.applyTo
   retains one LEFT relation each, clearing abnormal. Original first weak purge
   then removes all13 remaining candidates before exclusion selection. Clef's
   real Part/Measure membership disappears before its first graph-removal
   callback. Source contexts are intrinsic and below0.5, but factory-time grade
   deletion would skip required intermediate ownership/relations.
3. Connect prepared nullable physical-clef numerical pitch and direct absolute
   pitch separately from logical exported key/clef state. Use
   `fixtures/official-null-header/NEXT-PITCH-DEPENDENCY.md` and268 direct original
   controls. Keep physical null IDs, original C-clef absolute formula and carried
   XML key state; do not create a Clef or reconstruct owners from reports.
4. Needed Slur recognition remains unconnected. Actual Der Winter cleaned
   components contain three almost-whole slurs and three split pairs. A generic
   component/fragment exploration yields14 geometric candidates including the6
   expected sets plus alternatives/noise. It does not use original geometry for
   selection. Require original-BINARY run/gap checks, actual Head/Chord endpoint
   linkage and comparison of source math before native acceptance/export. Pure
   fit counts are not musical-quality proof.
5. Run ordinary public image CLI after each certified connection. Compare its
   full MusicXML semantics to the original and independently compare events,
   rests, complete Chords, voice partitions/continuity, meter/measures and needed
   relations to the CC0 scores. Validate XSD and independent reader. Preserve
   unscored/ambiguous categories rather than declaring equivalence from absence.

Keep small normal main commit/push/readback checkpoints and actual owners,
manual/relation identities, callbacks and exact clocks/rollback. Continue the
63+correction8 and exact7/64 XML/XSD regression when affected. No full649 repeat,
new auth/permissions/environment, Mac/GHA or giant transfers. Original task,
work189 and the separately unobtained ce785c9 remain preserved.
