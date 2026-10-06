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

The next checkpoint connects original `StemsRetriever.finalizeStems/
checkHeadStems` on the MusicXML image owner after each system's head linking,
before all-systems beam-side finalization. The public diagnostic `continue_linking`
retains its saved `linkStems` observation boundary. `SIGraph.getPartitions(null,
allStems)` sorts stably by intrinsic grade and partitions explicit exclusions;
then the existing `HeadStemsCleaner.check` acts on each compatible ensemble.
Head order follows original `Inters.byAbscissa`. The sequence enumerator is
shared with contextual grading, whose focused-head stem-conflict hack remains
separate. No caller supplies a selected ensemble on the image route.

One declared original actual-SIG control has three physical stems and an
exclusion between alternatives: partitions `[1,0]`, `[0,2]` retain relations
`[1,2]`, remove relation0, retain all stems and refresh abnormal flags. Native
matches this control. It is not a full-image positive multi-stem cleanup claim.
The existing selected cleaner/canonical-sharing controls remain separate.
The image route verifies owned/cache identities through its post-reduction
admission; single-voice export still requires one retained stem per head.
The cleaner's bounded partition storage admits at most16 distinct stem partners;
parallel/manual contexts and complete original epilog remain pending.

Next implement cross-system orphan beam finalization using actual SystemManager
vertical neighbors, first/last staff lines, physical glyph equality and linked
counterparts, before contextualizing. General ensemble/shared/frozen callbacks
and all original consistency/overlap/correction passes also remain partial.

General groups/aliens/cross-staff, sloping/wider PageCleaner contours, other
beam levels/flags/dots/tuplets/polyphony and broader rhythm/correction/export
remain tracked. No original nonUI function is planned for omission. All global
completion flags remain false.

## Later head-cleaner lifecycle checkpoint

The earlier manual/parallel limitation above is historical: selected distinct
stem parallel first-edge behavior and manual/frozen head/edge cleanup now match
six original controls, with precise owned edge removal/endpoint abnormal
callbacks. Automatic parallel remains guarded following one observed original
deleted-edge-alias exception; the other automatic parallel input is unobserved.
The16-partner bound and full cleaner/epilog remain partial. Cross-system orphan
and ordinary foundation checks progressed in their later existing receipts;
this does not complete general ensemble/mirror/Part/lyrics/epilog ownership.
[Current unit](../musicxml-head-cleaner-lifecycle/VALIDATION.json).
