# Next dependency

Retained EditingSequence now executes the actual DefaultEditingTasks in original
UITaskList do/redo order and reverse undo order, including each task's publication
before an impact notification. Dot InterTask class selects RHYTHMS first, not
LINKS from a nested relation. Inclusive planned range ends at the latest completed
step; mark_sheet_modified is a DO-only notification intent. reprocessed_steps
remains empty and controller_impact_complete false. No SheetStub persistence or
actual step execution is claimed.

Next: actual RhythmsStep.impact selects each retained edited Inter's current center
in its actual SystemInfo stack, deduplicates stacks in encounter order, and calls
PageRhythm.reprocessStack (StackRhythm.process with retained expected duration,
then Voices.refineSystem). PageStep follows with voice refinement and measure
fixing for AugmentationDotInter. Observe real owned stack/cache invalidation and
reprocessStack before connecting native execution; do not rebuild ownership in
simple setters or infer full image correction from a saved control.

General preEdit/history/editors/ensemble, owner relocation, multiple staff/Part,
shared stacks/support image admission, full FRAT/cleaner/epilog and CLI correction
commands remain partial and part of the non-UI parity goal. Existing 5 images47
and the negative quarter rest stay admission boundaries. Completion flags false.
