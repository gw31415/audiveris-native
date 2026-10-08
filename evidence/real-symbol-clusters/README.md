# Original capped SYMBOLS candidate generation on the actual score

Use the retained actual System areas to dispatch physical centroids, register all
extracted glyphs with their SYMBOL group, and build the original0.8-interline
links. Connected components use the original maximum7-part heavier selection.
Recursive decomposition preserves heavier-seed order, member/neighbor encounter
order, sibling considered-set growth, the0.04 classifier weight threshold, width/
height pruning and the left-of-System height exception. Actual SmallChord bounds
supply the1-interline right-side fine box and0.006 area threshold; the current real
image has no SmallChords/fine boxes. Unsupported musical factories stay guarded.

Original JGraphT1.5.2 ConnectivityInspector accumulates components in HashSet.
Glyph.hashCode includes RunTable.hashCode, which includes the Orientation enum's
Object identity hash. Thus component order is not a stable musical contract.
A second fresh original image observation, with identical physical glyphs and
176 components, changed53 selected-part orders and25 trial orders but changed zero
candidate multisets: both runs evaluate788 compounds. Keep this qualification
instead of embedding a JVM identity hash or per-image ordering into recognition.
Native components use deterministic breadth-first encounter order. The actual
image test compares selected physical sets and candidate multisets; it separately
feeds the observed part order into the same native recursion and compares all
member order, compound runs/bounds/weight fingerprints with original evaluation.
This control reorders only kernel inputs, not retained Graph/Staff/Chord ownership.

The first attempt assumed original component order was BFS and failed at System1
cluster1; that assumption was corrected using the actual JGraphT runtime, not by
claiming the failed order comparison passed. See ORIGINAL-ORDER-QUALIFICATION.json
and VALIDATION.json. Historical arbitrary global ordering is not quality parity.

Existing owners/Graph/manual relations/callback clocks are unchanged by candidate
construction. Native image recognition/XML and independent musical acceptance
remain unfinished. Retain63+correction8/exact7/64 XML bytes/XSD and normal saves.
