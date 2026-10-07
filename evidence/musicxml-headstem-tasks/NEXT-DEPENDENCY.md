# Next dependency

Actual ordinary manual HeadStem LinkTask do/undo/redo now uses retained Chord
ownership and canonical Stem identity, real callback/publication order and
controller DO dirty notification. Planned impact is distinct from actual selected
rhythm/refine/render, which is compared separately in test-only observed scope.
Original undo removes HeadStem but retains the new ChordStem and7/64. Do not
retarget old Stem, clear callback clocks, pad underflow or promote manual status.

Next observe an actual UnlinkTask over the same retained manual relation before
connecting its own do/undo/redo and identity/publication sequence. General
controller preLink, other relation kinds, valid public correction admission,
shared/multiple-staff owners, signed clocks/tuplets and complete cleaner/epilog/
project/history remain pending, not omitted. Public fine and incomplete Voice
admission guards remain. Full595 is historical95e191b; all global flags false.
