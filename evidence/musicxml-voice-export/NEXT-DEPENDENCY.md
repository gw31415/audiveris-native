# Next starting/Voice event connection

The actual image writer now consumes owned Voices and final stack BEGIN slots,
inserts backup at Voice boundaries and forward only for a positive clock gap,
and selects exact binary divisions from notes and movement durations. Four
original saved-object controls reach the actual PartwiseBuilder, including a
positive CONTINUE/1/8 gap. These are controlled solved object comparisons, not
new polyphonic image admission. Existing three image outputs remain the public
CLI examples; general starting/compound image and per-chord event guards remain.

Next extract the original processStartingChords loop as an owned processing unit:
first compound plus measure-rest chords sorted by center ordinate, create an
original Voice for every selected chord, then setAndPushTime(0). Original
processStartingChords does not skip an already voiced selected chord; unlike
mapChords.purgeRookiesSet it unconditionally constructs/reassigns. Compare that
constructor/setVoice behavior before changing the native unassigned-seed guard. Compare real original grouped/parallel
objects before connecting multiple starting chords. Then replace serial event
construction with actual assigned Voice/onset/duration events and check duration
for all ordinary Voices. Retain refusal for unresolved clocks and recognition
conditions until a same-input original polyphonic image/export result agrees.

Other Voice kinds/Merged, shared multi-part stacks, exact Rational/tuplets,
lyrics/cross-system matching and full cleaner remain ledger dependencies, not
omissions. Full595 is historical95e191b; all global flags remain false.
