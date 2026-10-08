# Actual Measure reset and pre-refinement rhythm rebuild

Pinned Java9e1e55cd2746037d059345881c53e6a6754bffbd reads the already acquired
parallel-beam PAGE book used by the retired BeamGroup control. Its exact input,
JAR, observer/helper hashes and commands are recorded in provenance.json.
Neither native outputs nor ce785c9 are original inputs; the existing book and
previous committed controls are preserved.

Four independently loaded cases compare20 method boundaries. Short and excess
cases retain the actual emptied old Voice7, support Voice2, distinct fresh
Voice1, six old slots, termination-5/8 or+1/2/excess1/2, and manual Head/tie/
BeamStem/Next relations. Genuine Measure reset clears Chord Voice/Slot/time
references while retained bodies and stack clocks survive. Stack reset then
clears stack slots/actual/excess while preserving the same expected object.
Actual MeasureRhythm.process constructs two new real Voice objects and seven
CompoundSlot references, before any stack slots or system refinement. Actual
StackRhythm.generateStackSlots merges equal onsets into six fresh stack Slots
and fills new Voice tables; readStackActualDuration gives1. The fresh slots
reuse old numbers but are different objects. BeamGroup and incoming tie/Next
still reference the actual source and target Chords throughout. Final system
refinement follows the original tie → beam → Next → preferred order.

Two further cases call actual StackRhythm.process directly after genuine
Measure reset. Their pre-refinement state equals the corresponding decomposed
stack-slot/read-duration state. No artificial Voice numbering or report owner
reconstruction is performed. General implicit tuplets/interleaved-rest passes,
multipart stacks, automatic cross-bar acquisition and controller/history are
outside this comparison and remain pending.

The executed observer for this acquisition is preserved in native Git8ae5b66
at scripts/java/VoiceResetRhythmProbe.java; its SHA matches provenance.json.
Later extraction of common preparation supports the PageRhythm control without
replacing these observations or their recorded executed-source hashes.
