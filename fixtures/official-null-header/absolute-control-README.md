# Direct original absolute-pitch controls

`absolute-control-original.json` contains 268 fresh calls to the pinned original
JAR. These are declared scalar controls, not recognition results. The helper
creates original ghost `ClefInter` objects (or passes null), declares their pitch,
and obtains the integer head pitch from an original `HeadInter.getIntegerPitch`.
It then calls `ClefInter.absolutePitchOf` directly. Independently recorded
`noteStepOf` and `octaveOf` results never supply the absolute-pitch answer.

The controls cover null, G/F/C/percussion clefs, small and octave G/F clefs,
positive and negative half-even head/clef pitches, and adjacent floating-point
values around ±0.5 and ±1.5. The 228 shape/half-even cases, 8 clef-pitch boundary
cases and 32 head-pitch boundary cases all completed successfully. The original
run took 1.24 seconds and did not run an image pipeline or native code.

At `C_CLEF`, clef pitch 4 and head pitch 0, the original directly returns absolute
pitch **24**, but separately returns **G4** for musical step/octave. Reconstructing
absolute pitch from G4 would return 32 and is not faithful to the original API
used by `VoiceDistance`. Its source formula is `28 - rint(clef.pitch) - headPitch`.
At null clef and head pitch 0, the static G/TREBLE fallback returns 34 (B4),
without adding any clef to a Staff or graph. Exact ±0.5 head pitches round to 0;
their immediately adjacent values preserve the original half-even boundary.

`absolute-control-PROVENANCE.json` records the actual compile/run arguments,
exit status, helper, class, JAR, runtime and source hashes. The output SHA-256 is
`f5da91c5c4366a314306678c8ac9710608f3553b5d9cad443b5224f53f30d8da`.
The fixture supplies next-unit assertions only. Nullable native pitch/mapping/
export changes remain unimplemented in this unit, and no recognition-quality
conclusion follows from these controls.

To reproduce using the existing tools, use a fresh private directory (the paths
below are illustrative; the provenance records the original executed paths):

```sh
probe_dir=$(mktemp -d /tmp/clef-absolute-control.XXXXXX)
mkdir -p "$probe_dir/classes" "$probe_dir/tmp" "$probe_dir/prefs" "$probe_dir/home"
/workspace/native-build-tools/oracle/jdk-21.0.12.1+1/bin/javac --release 21 \
  -d "$probe_dir/classes" fixtures/official-null-header/absolute-control-probe.java
/workspace/native-build-tools/oracle/official/opt/audiveris/lib/runtime/bin/java \
  -Djava.awt.headless=true -Djava.io.tmpdir="$probe_dir/tmp" \
  -Djava.util.prefs.userRoot="$probe_dir/prefs" -Duser.home="$probe_dir/home" \
  -cp "/workspace/native-build-tools/oracle/official/opt/audiveris/lib/app/*:$probe_dir/classes" \
  ClefAbsoluteControls "$probe_dir/observed.json"
```
