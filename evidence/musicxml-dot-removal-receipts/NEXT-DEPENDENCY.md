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
