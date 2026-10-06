# Original group lifecycle and STEMS continuation

This unit connects `AbstractInter.remove`/`Containment.removed` and
`BeamGroupInter.invalidateCache/getBounds` to the actual native SIG. Member
removal preserves surviving IDs/order, updates their owning group bounds and
removes a group when its last member dies. Extensive group deletion removes
members without another live ensemble owner. The connected weak-foundation
purge and reducer replay use the same callbacks; partially retained groups no
longer need an all-or-none admission guard. Glyph registration/cache objects
are retained. The shared-ensemble branch still needs an independent oracle
control; it is not a full-image claim.

`StemsRetriever.Finalizer.boostBeamSides` uses the overridden
`AbstractBeamInter.isGood`: intrinsic grade >= `Grades.goodBeamGrade` (0.35),
not general contextual grade. Two original boundary controls and a native
owned-side subset check this distinction. Original controls have five side
supports; the native boundary test intentionally constructs one side/head,
not a whole-SIG/count comparison.

One additional full image has four physical beam/hook aliases at HEADS. It
already emitted the original's 11 events before this change. The new fix is
its retained SIG group: four removed ordinary beam alternatives no longer
remain in `members`. After reduction/BeamHeadCleaner, the original and native
both retain one primary beam and four hooks, with matching member kinds,
bounds and group bounds. Native IDs are not JVM IDs. Three separate original
owned-group deletion controls cover partial/last/extensive deletion. They are
not alternative full-image exports or a general ensemble proof.

Next connect original `StemsRetriever.finalizeStems/checkHeadStems` at the end
of each system's linking: `SIGraph.getPartitions(null, allStems)` orders by
intrinsic grade and partitions only explicit exclusions; then the existing
`HeadStemsCleaner.check` acts on each compatible ensemble. This differs from
the focused-head contextual-grade partition hack. The current image finalizer
still admits only one stem per head. Follow with cross-system orphan beam
finalization using actual SystemManager vertical neighbors, first/last staff
lines, physical glyph equality and linked counterparts, before contextualizing.

General groups/aliens/cross-staff, sloping/wider PageCleaner contours, other
beam levels/flags/dots/tuplets/polyphony and broader rhythm/correction/export
remain tracked. No original nonUI function is planned for omission. All global
completion flags remain false.
