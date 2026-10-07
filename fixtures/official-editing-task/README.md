# Retained EditingTask control

Pinned Audiveris 5.11 source: `9e1e55cd2746037d059345881c53e6a6754bffbd`.
`EditingTaskProbe.java` extends the existing DotPartOwnershipProbe and callback
probe helpers. It loads the saved original double-augmentation book, with real
SIG, chords, Measure, glyph index, Link and EditingTask instances.

DefaultEditor construction needs GUI zoom initialization in this headless
runtime. The probe installs its original model fields without that drawing
constructor and invokes the original compiled do/undo methods. This is a saved
owner control, not GUI rendering or automatic correction discovery. Actual
InterService publication delivery is observed after draining the Swing queue.

Do/undo/redo compares callback entry and return, owner counts, lazy Part,
staff/Measure membership, bounds, grade/context, glyph and attachments. One
same-class endpoint duplicate is rejected; one fresh support is added before the
old support is removed. Undo restores the original relation instance before
removing the successful new link. Physical glyph registration survives all edits.

The native regression begins with its actual image-owned Graph and physical glyph
registry, then explicitly installs the saved cold cache/staff/grade/attachment
control state. It does not count this declared edit as new full-image success.
Original controller impact/reprocessing, general editors and ensemble
invalidation are separate pending dependencies.

`EditingImpactProbe.java` observes two original tasks' real publication order and
bounds, CtrlTask.firstImpactedStep and epilog below RHYTHMS (latest HEADS). Actual
through-PAGE reprocessing is not observed in this fixture; its inclusive schedule
is separated from executed impacts. Native modified flag is notification intent,
not yet persisted SheetStub mutation.
