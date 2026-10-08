# Next recognition dependency

The unchanged real page CLI v17 now reaches
`music_dot_multistaff_stack_filter_pending` (170.86s, no XML).
Read original SystemInfo.getStackAt, MeasureStack.addInter/removeInter and
DotFactory lateChecks before extending the ordinary binding. Use the held
Part/Staff/MeasureLayout/Stack Arcs and per-Staff abscissae; a shared system Stack
must choose the Measure belonging to the inter's actual Part. Preserve null Part,
lazy cache and ordered callback behavior. Do not rebuild ownership from reports
or clear clocks before callbacks. Keep real Chord/expectedDuration/manual relation
identity in rollback.

Then continue actual Staff-curve dot lookup, supported accidental context/reduction,
remaining required symbol relationships, shared rhythm/voices and MusicXML output.
Factory controls are not full classifier parity. Original mirrored AlterHead
cases and independent real-rest scores still need comparison. The image-quality
acceptance stays false until actual native image exports and musical comparisons
against both original and independent truth meet the revised goal.

Keep focused tests and existing63+correction8/exact7/64 XML bytes/XSD, normal main
save/readback and original task/work189/unobtained ce785c9. Excluded editor/API/
conversion work and full649 reruns are not new acceptance requirements.
