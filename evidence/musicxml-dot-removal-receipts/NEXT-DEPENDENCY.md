# Next dependency

LINK now consumes actual dot vertex deletions, including synchronous countDots
reentry before and during its consistency loop, once and in unregister order.
Only a countDots inconsistent-nonmanual-zero removal carries its observed
owner/member-count candidate proof. Weak/orphan removal is recorded separately;
absence alone does not grant MusicXML admission. Contextualization finishes
before selected weak deletions, so a stale snapshot cannot restore a recursively
removed strong dot.

Next independent dependency: original AugmentationDotInter.getPart lazy cache
and remove -> MeasureStack.removeInter -> Part/Measure membership lifecycle.
Compare cold/warmed getters on the existing real double-dot chain, including
membership before SIG edge callbacks. Native has not connected that ownership
cache. Do not infer Measure removal from vertex disappearance or admit shared
support images from declared mixed-owner tests. General contextual support,
re-added relation order, special heads and post-rhythm edit/reset still need
original observations. Complete LINK/cleaner/epilog, shared stacks/tuplets and
lyrics/slur/tie remain partial; global completion flags remain false.

Pinned Java ownership order for the next unit:
- DotFactory first-dot creation calls SIG.addVertex before setStaff and before
  applying support links. Second-dot creation leaves staff unset and applies
  DoubleDot only after vertex addition. AugmentationDotInter.added attempts
  stack.addInter immediately; do not assume the final support existed then.
- PageRhythm.populateFRATs sorts flags/rest chords/augmentation dots/tuplets by
  abscissa, filters each stack, then calls stack.addInter. That getter can warm
  dot Part ownership before rhythm. MeasureFiller handles Clef/Key only.
- MeasureStack.filter uses staff or nearby-staff Measure ownership and precise
  left-exclusive/right-inclusive bounds, not just nearest dot/note coordinates.
- remove queries the center stack and lazy/cached Part before AbstractInter
  sets its removed flag and starts SIG relation callbacks. A Measure augDots
  collection is separate from live vertices. Existing cold/warm observations
  are saved-page mutation controls, not proof of that automatic image lifecycle.
