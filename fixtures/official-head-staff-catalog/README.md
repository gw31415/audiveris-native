# Original staff scanning and pitch-dependent catalog contracts

Pinned, unmodified official 5.11.0 Linux runtime. The observer reads no Rust output.
The 384 cases instantiate original Staff/OneLineStaff, StaffLineAdapter and Scanner
in the order from NoteHeadsBuilder.processStaff. They observe original isOpen,
yOffsets, and all/stem/hollow shape enum sets. Main interlines 8/11/16, specific
staff interlines 7/11 and all 16 combinations of smallHeads, drumNotation,
oneLineStaves and crossHeads are included. Scopes are one-line automatic drum,
five-line pitched, five-line percussion clef and four-line percussion with no
default map. These are constructor contracts, not complete page lookup traces.

The 75 extra pitches -12..12 include missing mappings and drum boundaries.
Native constant metadata preserves all 24 original head shape enum names and
ordering; the 17 default drum pitch motif entries come from the hash-recorded
official res/drum-set.xml. Processing switches filter shapes exactly as ShapeSet
and Scanner.buildShapeList. No unknown pitch falls back to oval heads. A caller
still needs exact templates before matching each selected shape.

Original buildHeads is also called with a test subclass overriding virtual
isTablature to true, for four/six-line tablature controls: no SIG vertex change.
The compile-only Staff ABI shim is excluded from the runtime classpath. Native
planning skips tablature before font/scale lookup; one-line staff plans contain
three pitches and disallow ledgers.

The native certified page path now passes sheet main interline into staff/ledger
search offsets instead of staff spacing. Page orchestration explicitly rejects
unpropagated non-51 staff catalogs before preparing pixels; it does not silently
claim a 51pt result for a small staff requiring 32pt. The separate font/size kernel
catalog from the previous checkpoint remains available. Laura's existing reports
and shared SIG are unchanged.

Full-page one-line/drum/tab coverage, custom user drum-set.xml, additional special
shape matching/glyph/HeadInter contracts and size/catalog propagation through seed
veto and registration remain unfinished. HEADS completion is false. Initial
observer attempts with incorrect reflected API/member calls were discarded; only
the successful observer and independent final fixture are used in tests.
