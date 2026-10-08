# Nullable actual Voice and shrink range boundaries

Nine pinned original real-book controls compare null Voice/null stop, live stop
owner, a directly declared null stop Voice field, first stop (empty range), reversed
range, stop outside the selected Voice, reset-null versus allocated-empty chord
lists and a purged actual Voice still referenced by the stop Chord. Candidate owners
are retrieved through original SlotsRetriever/buildCompoundSlots before calling
private shrink with an explicitly constructed original TupletGenerator; ordinary
processing switches and resource binding remain unchanged.

A supplied actual Voice wins. With no supplied Voice, source follows stop.getVoice,
including a retired actual object, and returns unchanged when both are null. Native
uses actual Graph Voice Arc references across current/retained owners, never a
numeric substitute. Source getChords exposes both null reset and allocated-empty
lists as empty here. A stop absent from the selected list uses the full range, then
receives the final push; first stop returns before generation. A matched start after
the stop index throws IllegalArgumentException before clock/container changes.

Native compares every old/new clock/Voice/StackSlot/expected/end/excess/manual frame
and actual narrow clocks. Manual ChordTuplet registration identity remains held.
Every source control has zero new Graph callbacks; existing Tuplets bypass wrong
explicit font resources. Outer native rollback preserves actual state on invalid
range.45,795-byte compact fixture losslessly inflates18 states/six distinct frames.

Remaining dependencies: original inspect signed eligibility, merge removal and
second rhythm pass, actual ordinary map candidate/font/switch/signature binding,
full nonUI ledger. Full/public false. Source controls are evidence, not owner input.
