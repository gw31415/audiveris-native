# Next dependency

Retained Voice object keys now survive last-Chord retirement, Chord undo and native
failure rollback. Next compare Voice reassignment when the original musical voice
number is reused: cached old Voice identity must stay distinct from the new Voice,
and explicit setVoice/justAssignVoice must update the actual binding. Integrate
normal and editing HeadChord ownership across real Part/Measure/Voice callbacks
and ordinary reprocessing with independent original intermediate-state controls.
Then add canonical head duplication/mirror AdditionTask and pair.source replacement,
InterController conflicting HeadStem relations and epilog before export admission.

Preserve relation/manual identity, nullable geometry, expectedDuration and exact7/64
with original short-Voice termination/divisions. No report-based ownership rebuild
or callback-before clock clearing,13/16 normalization or ending padding. Native
source Stem bounds are still five pixels longer into the head; saved bounds are
explicit controls, not new image parity. Full nonUI scope and all flags stay false.
Original task/environment and unpushed local ce785c9 were not changed/regenerated.
