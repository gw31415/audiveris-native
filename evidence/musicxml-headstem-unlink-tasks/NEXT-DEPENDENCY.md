# Next dependency

Ordinary retained LinkTask and UnlinkTask do/undo/redo now preserve real relation
identity/manual status, canonical Stem/Chord, cached expectedDuration and exact
callback clocks. Common selected reprocessing matches original underfilled
Voices and export; notify_impact is still planned impact, not PAGE/persistence.

HeadStemRelation.preLink is the next actual controller dependency: original
chooses canonical head share/mirror, or moves head Containment to the target
Stem's existing/new chord, removing an empty old chord. These Addition/Removal/
Containment tasks require true typed Chord/Part/Measure/Voice member lifecycle,
not reconstruction from report or a synthetic direct setStem. Public admission
stays until corresponding original callback/rollback/export comparison agrees.
General owner/tuplet/signed/multiple-staff, full cleaner/epilog/project/history
remain pending, not omitted. Never normalize automatic13/16 or add ending padding.
All global completion flags false; full595 remains historical95e191b.

The subsequent preLink lifecycle saved controls are now implemented in four
small source checkpoints: existing target, new unplaced Chord, new-Chord SIG
callback ownership and last-Chord Voice retirement. Follow the latest
`../musicxml-headstem-prelink-voice/NEXT-DEPENDENCY.md`; full parity remains false.
