# Actual existing-Tuplet shrink with retained candidate owners

Pinned original7 real-book controls compare whole/excess/exact7/64 starts, actual
stopChord, unmatched sync (source defaults first), null first clock and a later
real Stack Slot instead of CompoundSlot. The original private method receives an
explicit actual TupletGenerator constructed on the real Measure, while the ordinary
processing switch remains unchanged. Original SlotsRetriever/buildCompoundSlots
construct the actual narrow/compound owners before shrink. These are declared
resource controls, not ordinary font/switch binding or full rhythm admission.

Native binds candidates from existing actual Chords/Measure using the previously
compared narrow/wide retrieval and compound constructors. It retains each actual
compound Arc and narrow clock owner. Source compares supplied Voice actual order,
first time.equals(sync), default-first when none matches, stop list index, empty
range bypass, actual BeamGroup extension/first-pivot split, existing Tuplet reuse,
then previous.getEndTime -> setAndPushTime -> current CompoundSlot's first-containing
narrow clock update, and final optional stopChord push. All successful source calls
have zero new Graph callbacks and preserve manual Tuplet/relation objects. Wrong
explicit native font scale is bypassed by the same existing-object generation path.

Actual old Stack Slot and new candidate Slot can reuse numbers. The native test
observer now holds baseline Slot Arcs and compares object identity; number equality
cannot prove getSlot returns the old object. Old Voice/Stack Slot storage/end/excess/
expected and report remain while physical Chord clocks and narrow clocks change.
The observation correction does not rebuild any owner from source output.

Source null first clock throws before generation. On later Stack Slot cast failure,
source has already pushed real Chord clocks and assigned earlier narrow clocks
1/12 and1/6 before ClassCastException. Native outer operation restores Graph/cache/
actual container storage/physical Chord clocks/candidate clocks/manual identities;
it does not claim original direct Java rollback. Rebinding the genuine actual
candidate object permits retry from that preserved pre-call state.

Fixture53,975 bytes losslessly interns10 frames/one old relation set and whole-
inflates to original output. Source outputs supply evidence only. Fresh allocation
callbacks, nullable Voice/stop fallback, inspect eligibility, merge/second pass and
ordinary resource binding remain next dependencies. Full/public false. See
VALIDATION.json.
