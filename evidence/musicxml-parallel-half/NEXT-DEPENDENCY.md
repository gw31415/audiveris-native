# Remaining original rhythm/ensemble dependencies

Actual ordinary parallel quarter/half/flag-eighth and one-two-level beam Voices
now reach MusicXML with original ownership, clocks and duration checks. Next
independent units should connect multi-member chord event/export ownership,
whole/stemless/special head intrinsic duration, or mixed-rest/other-meter Voice
mapping after a successful original input. Shared/Merged stacks need actual
multi-staff/part and stack ownership; general tuplets need nonbinary Rational and
recognition/linkage before admission. Implicit tuplets/pickup, lyrics/slur/tie,
full cleaner/epilog, correction/project/export and other original nonUI features
remain in the complete migration plan. Kernel/object comparisons cannot stand
in for actual image success; no image-specific fallback or guessed clock.

## Concrete next disconnection found in saved sources

Original AbstractChordInter.getMembers uses Inters.byReverseCenterOrdinate;
getNotes returns that bottom-up order. PartwiseBuilder.processChord processes
all those notes, and processNote emits chord indication after the first. Native
music_xml currently guards members.len()!=1 before slot mapping, in duration
resolution and in the event loop; music_voice_export.write_measure selects one
event per chord ID and treats that chord ID as the emission key. These must be
changed together for one original-successful actual multi-head chord image:
member-owned note events in original order, one Voice/clock advance per chord,
first note normal/subsequent notes <chord/>, and actual member-note uniqueness.
The existing bind_event already checks real membership and mapped ownership;
retain it and the family/clock/incomplete guards. This is a source dependency
audit only, not implemented or verified multi-head image admission. Shared stack
and nonbinary tuplets cannot be claimed on the ordinary current input evidence.
