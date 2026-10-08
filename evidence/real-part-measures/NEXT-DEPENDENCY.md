# Next dependency — piano Heads and Chords in the same actual Measure

The retained piano Part now has actual ordered PartBarlines, per-Staff Measure
abscissae and common Stack references before Chord dispatch. Full-image library
comparison should reach music_multistaff_chord_dispatch_pending. Keep that frontier
separate from full native image CLI export success.

Original ChordsBuilder shares one stemChords list across a Part's Staff sequence,
then dispatches by actual Part.getMeasureAt(center), falling back to headLocation.
Compare same-image original ChordsBuilder actual members/stem/Part/Measure and
Staff references. Connect per-Staff headers and Heads to the common piano Measure
without inventing a Part/Voice or splitting cross-Staff stems. Measure/Chord refresh,
SYMBOLS/rest/dot ownership and Voice/output binding must use actual constituent
Staffs and per-Staff boundaries. Preserve selected rollback, manual/relation
identity and callback clocks.

Then continue the actual image meter/voices/rhythm/output frontiers and independent
real score quality comparison. Shared geometry Stacks are established here; their
common rhythm/Slot clocks must also be connected before multi-Part export admission.
Original genuine shared/stemless Heads, courtesy cleanup and missing-column repair
remain explicit if required. Keep the revised image-quality goal; unrelated APIs,
editing/UI/undo/project conversion are not remaining work. Normal main saves are
interim. Preserve original task/environment/work189 and unobtained ce785c9.
