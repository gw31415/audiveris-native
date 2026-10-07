# Stack reprocessing original control

Pinned 5.11 source `9e1e55cd2746037d059345881c53e6a6754bffbd`.
The probe extends the existing retained-dot editing helper and loads the saved
original double-augmentation book with actual Stack/Measure/Chord/Voice/SIG/glyph
owners. Part getters are warmed explicitly before the bounds-only raw edit.
DefaultEditor drawing construction is excluded as in the earlier headless
control; original compiled model/task methods run.

One run invokes actual PageRhythm.reprocessStack. A separately loaded original
owner replays its original stack reset, MeasureRhythm.process, stack slots/actual
and Voices.refineSystem methods to expose intermediate states. Replay is labelled
separately and equals the real complete call. Neither calls populateFRATs.
Only stack reset/order is integrated natively at this checkpoint. Complete edited
owner reprocessing and correction CLI are not claimed. Existing images reused.
