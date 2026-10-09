# Actual Part Staff pause lookup and notation

Original AbstractPauseInter.lookupLink takes the actual Staff's Measure
abscissa and scans same-system HeadChords whose actual member Staffs contain
that Staff, with integer center between Measure left and pause center-right.
It chooses the rightmost by stable chord-bounds abscissa order. It does not
restrict lookup to the Part top Staff or restrict the candidate list to the
selected Measure when the pause right side extends beyond its boundary.

Support reads actual cached Chord confidence from membership invalidation;
later Head contextualization must not recompute it. Export the live
ChordPauseRelation on the first actual note of the Chord with placement from
integer pause/note centers. Preserve clocks, relation/manual identity and
actual Chord owners. Fresh unchanged original image stages contain15 SYMBOLS
pauses and3 surviving LINKS caesuras, including the lower piano Staff.

The focused test follows the same ordinary image preparation/recognition path
through SYMBOLS, LINKS, normal Page rhythm and actual Voice/BEGIN export.
It uses original observations only as expected values, checks cached-confidence
residuals separately from exact membership/relations, and does not claim numeric
grade equality or zero-rest/tie/grace coverage. Public multi-Part CLI admission
and missing Dynamics/Pedal still require connection before quality acceptance.

The first failed comparison exposed missing automatic manual=false metadata on
the Pause Inter/Relation; creation now records original defaults explicitly.
The second failure was an overstrict test of Vec allocation across normal
container refresh, which stages a collection while retaining actual registered
Chord identity. The contract comparison instead checks surviving canonical
Chord IDs/origins, member/Stem references, Measure membership and clocks, plus
unchanged Chord storage during the read-only writer. Both failed logs remain.
