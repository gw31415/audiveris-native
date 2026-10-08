# Actual nullable Voice Chord lists and real measure-rest construction

Pinned original two real-book short/excess controls observe direct Voice.removeChord
until the allocated list is empty: getDurationSansTuplet returns ZERO, the Voice
remains registered and all physical Chord backrefs/Slot/termination/excess survive.
Voice.resetChords instead sets its private list null. getChords still returns [],
but direct sans-Tuplet duration throws NullPointerException and find's first Chord
access throws IndexOutOfBoundsException. Native retains this actual nullable state,
returns typed errors, and never purges or invents a Voice for these calls.

Original direct addChord restores the same actual list membership, stably ordered
by integer center x, without rewriting Chord.voice or clocks. Native direct APIs
use current/retained object Arc identity, not a musical ID; ordinary setVoice
initializes that same field. Normal Measure.removeVoiceChord's purge remains its
separate existing lifecycle. Source-declared input controls do not rebuild owners
from any report.

Actual WHOLE_REST/RestInter and RestChord are registered with the real Staff/Measure,
then original createMeasureRestVoice and addVoice produce musical ID3 with its own
rest Chord reference and a null normal list. Both before/after resetChords it returns
null duration and bypasses find before missing expected/invalid beat/font reads.
Native constructs the same real RestChord/Voice, preserving source identity and
normal voices' slots/termination/excess/manual status. General processing switch
partialWholeRests and automatic special-rest image admission remain pending.

Source fixture26,142 bytes losslessly interns6 frames/one relation set; full
inflation equals the original output. Old relation rows are labelled independently
in Java/native, and native Graph equality separately checks unchanged relations
at each nonediting boundary. No source exception is silently converted to ZERO.
Native find rollback retains the nullable owner state, actual containers and clocks;
this does not assert original Java exception rollback. See VALIDATION.json.
