# Next nonimplicit SlotMapper dependency

Known narrow time validation now rejects/reselects before Voice/time apply,
preserving explicit Next duration exemption and stopping a repeatedly forced
conflicting pair. The image path remains guarded one incoming/one Voice.

Next: inferSlotTimes/checkSlotTime multiple inferred end values -> original
nonimplicit analyzeTimes -> blacklist mappings to later times, retain earliest
narrow time, retry before apply. This is the original minimum-time rule, not a
new closest-chord heuristic. Compare declared original actual-object conditions
separately from the existing 36-event image CLI regression.

Unknown active clocks/abnormal handling, createNewVoices/sibling/EQUAL/previous
slot merge, implicit-tuplet shrink/exact Rational, Merged/shared stacks and actual
candidate chord-slot reference identity remain partial. No planned nonUI omission;
all global flags false, all-rest original XSD rejection remains negative evidence.
