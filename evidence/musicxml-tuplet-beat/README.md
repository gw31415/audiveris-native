# Actual Voice beat groups before Tuplet force generation

Original private TupletGenerator.buildGroups/checkGroups compares96 retained
Voice controls: actual ordered Chords, original clocks including a backwards last
offset, all-zero, exact3/8 thresholds, large gaps and first/later null. Explicit
beats1/4,1/2,1/8 and0 isolate this private boundary. Raw beat is3/2normal beat;
nextTime starts ZERO and increments once per matching Chord, with no loop to fill
a time gap. Null time fails at the current source Chord rather than being skipped.
Zero beat is a declared private-method control, not a time-signature admission.
General signed negative clocks remain separately pending.

A second actual BeamGroup/Beam/BeamStem relation set is declared on real retained
Chord stems. Source checkGroups returns the first different actual group using the
first retained Beam of each Chord. Four HALF controls fall back once to QUARTER;
other beat values retain their groups. The96 controls include32 original null
failures. Final groups, initial first pivot, fallback and unchanged real clocks/
manual/relation/Voice/Slot ownership match; no fresh force allocation is claimed
by these private planning controls. No Voice ID or owner is synthesized.

Native query preserves actual current/retained Voice Arc identity and its Chord
order. It reads each Chord clock before any later invalid owner is encountered;
an unknown Voice Arc is rejected. Composite failure restores staged cached Beam
reads/Graph. Physical Chord/Slot storage and held reports remain unchanged.
The late-invalid-owner ordering check is native transaction/admission evidence;
it does not claim an equivalent Java object corruption operation.

Source2,634,298 byte output losslessly interns22 frames/one original relation set
into109,528 bytes, whole-inflation equality verified. The first provenance writer
referenced an unavailable optional BeamGroup source file after verification; the
corrected record hashes only actually read primary files, with fixture bytes
unchanged. Source observer and runtime/book hashes are retained. No native output
supplies fixtures; no whole649/profile/archive or original120 generation repeat.
See VALIDATION.json. Actual signature/font binding, automatic force generation,
force eligibility/inspect/merge/second pass and full nonUI completion remain.
