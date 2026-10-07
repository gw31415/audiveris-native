# Next dependency

Retained automatic HeadStem added callbacks now check Head then Stem and mark the
actual editing stub only on changed abnormal values. Re-add sees the other live
relations; same-object duplicate is not an added callback. Initial recognition
batch assembly remains separate; no new image or manual correction admission.

The original manual branch is now observed on actual saved HeadChord/Stem/
Measure/Voice owners. Outer HeadStem added enters with old ChordStem; nested
ChordStem removed callbacks see null stem; nested added callbacks see the new
stem. Both invalidate HeadChord cache before final Head/Stem abnormal checks.
Part/Measure/Voice/time remain; all abnormal values and Book/Stub dirty can stay
false despite the ownership mutation. Controller DO would independently dirty.

Next connect these actual identity-edge mutations, HeadChord lazy cache/getters
and retained OwnedMusic Chord stem/geometry/Voice projection through the proper
editing/reprocess transaction. Do not write a report to reconstruct ownership
or clear clocks before callback. Preserve manual status and original ordered
nested callbacks. Keep the retained-manual guard until owned comparison and
reprocessing/export validation exist. This is missing dependency, not an
original failure, and not native manual-image success.

Book ScoreReduction/refineScore, repository/multi-sheet/project persistence,
controller PAGE/history, shared/special support and full cleaner/epilog remain
pending, not omitted. Flags false; full595 remains historical95e191b.
