# Next original SlotMapper unit

Known single narrow times now have their own retained getter/setter; the selected
active end is stored before Voice/time apply. Extinction reads that first-member
clock instead of reconstructing it from the sole chord. Current image admission
and all global false flags are unchanged.

Next is original mapRookies validation/order: inferSlotTimes/checkSlotTime,
blacklisting/retry for conflicting inferred time/end beyond expected, then
createNewVoices/getOrderedSiblings/equal-priority and previous-slot merge.
Required dependencies include exact Rational and implicit tuplets; full branches
must not be replaced by closest-chord or counter-based heuristics. Actual
candidate chord-slot reference/setter identity remains partial before final
stack slot construction. Merged/multi-Voice/shared stacks still follow.

Current original comparisons are successful saved same-image slot clocks and
current image->MusicXML. Positive polyphonic gaps remain declared method controls
only. Original all-rest XSD failure is negative evidence, not a parity success.
No original nonUI logic is planned for omission; lyrics/automatic slur-tie,
correction and complete cleaner/epilog remain partial.
