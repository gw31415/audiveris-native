# Next dependency

Existing-target HeadStemRelation.preLink now produces retained Containment
UnlinkTask, empty-Chord RemovalTask, Containment LinkTask and the declared final
manual HeadStem LinkTask. Actual Chord/Measure membership and Voice member lists
move without rebuilding owners from reports or clearing clocks before callbacks.
RemovalTask undo replays the same surviving relation objects; it does not restore
old Voice membership. Failure rollback covers real Chords, Measures, Voices,
expectedDuration, Graph identities and task receipts.

Next is original new-target AdditionTask and ChordStem/Containment lifecycle:
new HeadChord has no Part/Measure during initial addition, joins its stack when
Containment supplies the first real head, retains lazy owner references on undo,
and can be registered into its Measure earlier on redo. Add canonical head
sharing/mirror tasks after the unplaced/retained Chord owner is represented.
Then compare InterController conflicting-relation tasks, genuine reprocessing
and export before widening CLI admission.

The saved-owner comparison declares saved Stem bounds and three-decimal grades
on actual native objects. Native's automatic source Stem extends five pixels
farther into the head; physical extent parity remains a separate unresolved
dependency, and this control is not new automatic image success.

Keep public correction guards and all global completion flags false. Preserve
Rational7/64, underfilled Voice termination/divisions and all remaining nonUI
functions. Neither automatic13/16 normalization nor ending padding is allowed.
