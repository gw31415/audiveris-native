# Instant staccato on held Staff neighborhoods and real Chords

Original DotFactory first runs repeat interpretation, then staccato, before its
late augmentation/DoubleDot checks. This unit connects staccato at InterFactory
candidate encounter, rather than rejecting through a coarse single-Staff rectangle.
Repeat interpretation still has an explicit boundary guard and needs its original
instant lookup/order comparison next. Full DotFactory parity is not claimed.

Articulation lookup uses actual surrounding Staff areas and first/last curves;
between Staffs it searches both cores. Directional marcato/staccatissimo restricts
above/below, minimum y gap and profile0/1 ranges precede weighted grade, and minimum
vertical gap wins in stable Chord encounter order. Actual Chord/Part/Measure and
clock references are held, not reconstructed from report Staff coordinates.

On the original unchanged image,97 accepted classifier Dot candidates yield15
positive staccato lookups. Sixteen actual pre-late articulations (15 STACCATO and
one STACCATISSIMO_BELOW) supply constructor/add/link/each removal controls. Their
source shapes and grades are controlled factory inputs, not a full classifier
parity claim. The complete original image XML has8 surviving staccato marks;
weak reduction/export and independent notation quality remain unfinished.

Typed ChordArticulation support has zero source coefficient and target2.5. Added
articulation is abnormal before its relation; relation add/removal only refreshes
the surviving articulation. Invalid typed support stages rollback. Existing
Head/Chord graph metadata, relations and clocks stay intact. An internal DotFactory
interpretation is not returned into the same-glyph InterFactory Exclusion list;
allocate Symbol keys from all retained identities so another interpretation or a
later creation cannot replace it or reuse a retired key.

The first focused real-image test attempt was stopped because its preservation
loop repeatedly serialized the same full snapshot. The comparison now captures
one snapshot and indexes all original IDs; comparison and rollback coverage stay
unchanged. Successful unique tests,63+correction8/exact7/64 XML bytes/XSD and actual
CLI frontier are recorded in VALIDATION.json. Quality acceptance stays false.
