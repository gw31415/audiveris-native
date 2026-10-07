# Next dependency

Ordinary support addition now completes its original synchronous callback per
new live edge. Automatic Augmentation checks abnormal only; manual relation or
manual dot additionally recounts the actual chord. A manual relation can remove
its automatic dot inside edgeAdded; the original Part/Measure and nested
edgeRemoved/vertexRemoved lifecycle and LINK ledger order are retained. Reusing
an already-live support is not a new callback. DoubleDot addition checks only
its source, independently of chord counting.

Next: connect original correction-task ownership transitions around these
setters/relations, rather than relocate or rebuild Measure collections inside
simple setters. Pinned 5.11 EditingTask bytecode read (not a mutation-success comparison)
shows performDo: editor.finalDoit -> apply new links (discard failed entries)
-> remove old links -> InterIndex.publish. Undo: editor.undo -> restore old links
-> remove new links -> publish. Do not assume unlink-before-link. Follow original
InterEditor.finalDoit, Link.applyTo/removeFrom and controller impact/rhythm-cache
invalidation with actual saved owners before exposing edit commands.
Current raw setters do not implement complete correction commands. Multi-staff /
cross-Part/shared stacks, all FRAT classes, general Measure split/merge/delete,
shared-support image admission and complete cleaner/epilog remain partial.
Existing images and the original undotted quarter-rest refusal remain admission
boundaries. All completion flags stay false; full non-UI parity is the goal.
