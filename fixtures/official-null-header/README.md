# Original empty-header owner observation

Pinned original `9e1e55cd2746037d059345881c53e6a6754bffbd` processes the unchanged Der Winter opaque PNG through ordinary HEADERS and CHORDS in memory. `RealNullHeaderImageProbe` retains the original Staff/Part/Header references across those stages and checks each HeadChord against its actual Part, Measure and Stack. Original observations are test assertions only; native recognition starts from the PNG.

System 2 Staff 6 has header start 97 / stop 157 and null clef/key/time, with no header inters. Its 18 HeadChords contain 20 HeadInter members, all in actual Part 2 Measure/Stack owners. No ClefInter is synthesized and no previous-system clef is borrowed.

At CHORDS, `Measure.getClefBefore` is null for all staffs because the recognized header symbols have not yet been dispatched into measure containers. This fixture certifies the empty header and chord ownership; later note-pitch/default-clef and MusicXML behavior remain separate dependencies. It makes no full recognition or quality-parity claim.

`PROVENANCE.json` records the exact run/compile commands, original JAR/runtime, source and helper hashes. The private `/tmp/run-real-null-header.py` harness compiles only Book/RunClass signatures, excludes those signatures from the runtime classpath, and runs the saved helper using private tmp/cache/home/config/data/preferences paths. The helper has no native-output argument. `original-console.log` preserves the successful original run.

The focused native owner test passed once in 246.998 seconds including compilation. It matches all 18 null-header Staff chords and 20 heads against original bounds/shape/pitch/manual/Measure/Stack, retains actual Staff/Part and cached stem allocations, preserves existing relation records, and leaves new rhythm clocks unset. `NATIVE-RESULT.json`, `native-test.log` and `native-observed.json` preserve that result. No MusicXML or full-image quality parity is claimed.
