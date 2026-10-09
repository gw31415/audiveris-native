# Real Arpeggiato factory lookup

The source factory tests the original candidate against the then-live, same-System
HeadChord list. A full Chord rectangle includes its stem, so full-box overlap
alone was an incorrect public CLI admission gate. Native now follows that broad
lookup with actual head-only geometry from live Containment members. It uses the
source profile fallback, ties-to-even distance rounding, vertical overlap grade
and first-maximum selection. This lookup neither creates nor mutates owners.

The unchanged Der Winter candidate at [805,803,1,16] is the real nullable case:
47 current System1 HeadChords, lookup [806,803,28,16], one full-box hit with actual
head bounds [807,758,14,12], y overlap -33. Source factory and separate lookup
return null. Native observes at the checked factory boundary before reduction,
keeps the entire graph equal, and verifies actual System/Part/Measure pointers.
Original evidence enters assertions only after native recognition.

A positive lookup still stops at musicxml_arpeggiato_chord_link_pending. Its
construction, callbacks and notation have not been certified. The source caches
its sorted HeadChord list when InterFactory is constructed; the connected native
SYMBOLS factories do not add/move HeadChords within that interval. A future
factory that does so must preserve the source list lifetime. Arbitrary equal-x
native/original encounter-order parity remains unproven for positive ties.

The historical admission helper remains conservative; ordinary recognized image
admission relies on the actual earlier factory check. Numerical boundary tests
are source-contract controls, not extra original image observations. This unit
is recognition plumbing, not a score-quality equivalence claim. See VALIDATION
and NEXT-DEPENDENCY for the observed public CLI result and remaining work.
