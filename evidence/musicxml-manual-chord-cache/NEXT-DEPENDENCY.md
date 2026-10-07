# Next dependency

Retained HeadChord cache now invalidates on each actual ChordStem added/removed
callback, keeps clocks/owners, and warms only through getters. Current stem is
live relation ownership; it is not cached. Callback entry can still read old
geometry after the edge changes, before invalidateCache. Snapshots do not warm.
The legacy data geometry projection is not the actual lazy cache.

Next connect outer manual HeadStem callback to nested old ChordStem removal,
new ChordStem addition, cache invalidation, then Head/Stem abnormal checks.
Synchronize the existing retained OwnedMusic Chord object using those actual
getters without constructing new Part/Measure/Voice or clearing clocks. Keep
public/generic manual admission guarded until real reprocess/output comparison.

Original saved control targets a two-flag Stem: 7/16 becomes7/64, then actual
PageRhythm reprocessing creates two Voices and PartwiseBuilder serializes both.
This is original declared correction evidence, not automatic image success or
native output acceptance. Native fine-duration/complete-measure guards stay.
Book/score reduction, PAGE/history, general cache/setters, multiple staff/shared
owners and complete cleaner/epilog remain pending, not omitted. Flags false.
Full595 remains historical95e191b.
