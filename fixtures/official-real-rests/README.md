# Actual Rest factories and later RestChord allocation

`image-observed.json` records the original pinned Java executable on the unchanged real Der Winter PNG in `fixtures/real-rest-page`. `provenance.json` gives the source commit, executable, observer, input and output hashes. The observer now lives at `scripts/java/RealRestLifecycleImageProbe.java`; it delegates every factory call to original `InterFactory.create`.

The source begins at ordinary CURVES, which includes CHORDS, then runs original SymbolsStep.doProlog. Each system executes original SymbolsBuilder.buildSymbols, ChordsBuilder.buildRestChords, then InterFactory.lateChecks. Scheduling is serial for observations; within-system order is unchanged. IdentityHashMap labels refer to actual Rest, Chord, Staff, Part, Measure, Stack and Containment references. No native output supplies any source ownership.

There are 12 Rest factory results, each registered with its actual Staff and no Chord. The later ChordsBuilder pass adds 12 real Chords and Containment edges with null Voice, Slot and time offset. Staff 3 has two Rests and Staff 6 has one; these lower Staves share their Part's Measure with the upper Staff. The same objects and relations survive lateChecks. The separately recorded final original export has nine Rests: three weak candidates are removed later. Twelve SYMBOLS candidates must not be compared directly with nine final MusicXML events.

No factory NULL return or center/centroid Staff disagreement occurred on this page. The factory's nullable Part.getMeasureAt and the independent Chord center-to-Stack lookup remain separate source contracts. A native Chord without a Measure currently retains an explicit admission error because its retained owner type requires a Measure index.

An earlier `/tmp/real-rest-lifecycle-original-v1` trial began at MEASURES and lacked CHORDS/CURVES. That trial is invalid for factory parity and is neither substituted for nor deleted by this evidence. The valid v2 trial and hashes are preserved unchanged here.
