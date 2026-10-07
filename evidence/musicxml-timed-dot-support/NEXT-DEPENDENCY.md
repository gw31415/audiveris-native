# Next dependency

Timed ordinary countDots now refreshes its live duration projection without
clearing existing Part/Measure/Voice/slot/time. Shared live duration code feeds
both callback projection and legitimate StackRhythm processing; Graph exposes
uncached ordinary chord_duration/chord_end_time getters.

Next connect only the compared same-stack ordinary AugmentationRelation task
path to retained Export reprocessing. Original adds links before deleting old
support, and undo restores old support before removing the new relation. Added
callbacks recount only for manual relation/dot; removed callbacks recount any
live target. Compare all intermediate owners/clocks, real cached expectedDuration
reprocessing and rollback: original nonmanual undo leaves actual13/16 (not a
successful full-measure export), while manual original support yields full undo.
Do not repopulate FRATs or reset time before callback, infer automatic shared-
support discovery, or bypass provenance/actual membership guards.

DoubleDot task reprocessing, tuplets/special heads, shared stacks/multi-staff,
general controller/PageStep/history/cleaner/epilog stay nonUI parity dependencies.
Historical595@95e191b remains the last reported full run; flags remain false.
