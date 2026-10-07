# Next dependency

The corrected retained-owner path now executes bounded stack rhythm and HIGH
Voice purge/sort/rename -> previous-Measure NextInVoice -> preferred-ID collision
swaps, preserving owned Chords and output binding. Original declared parallel
owner control is separate from existing automatic images and raw bounds CLI.

Next original Voices.refineSystem first resolves getTiedId, then the first
Chord's BeamGroup(previous-Measure) predecessor, then NextInVoice and preferred
ID. Existing classified SlurHead/tie and actual BeamGroup getters supply parts
of those dependencies; automatic CURVES/tie generation, cross-Measure beam image
admission and general Voice kinds remain partial. Observe actual owner references
before connecting the next branch; keep exact previous-Measure and callback order.

Timed support edit callbacks also remain guarded: they currently cannot recount
a timed native Chord before legitimate reprocessing. Do not reset clocks before
the original EditingTask callback, falsify a report or loosen this guard. The next
lifecycle must preserve actual Part/Measure/cache/Voice while invalidating original
duration semantics in the proper task order. Full controller PageStep/history/
persistence, shared stacks/multi-staff/tuplets/cleaner remain final-parity work.

No new full suite, broad image admission or backup duplication. Current-source
readback and historical595@95e191b remain distinct; all completion flags false.
