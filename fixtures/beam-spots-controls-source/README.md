# Independent BEAMS pixel controls

These fixtures contain original Java results, never native outputs. Production
Rust and cargo tests only read the frozen JSON; they do not invoke Java.

- `../beam-spots-controls.json`: decoded retained `BeamFiltersControlProbe`
  results (32 declared patterns, 224 closes). Each decoded mask was checked
  against the source SHA-256 before being retained. The original local evidence
  is `beam-research/filter-controls/stem-seeds-clean/result.json`; the source and
  result digests are in `provenance.json`.
- `../beam-spots-controls-additional.json`: `BeamPixelReviewProbe.java` invokes
  the unmodified pinned JAR directly, with no recognition algorithm copied into
  the observer. Seed `0x514f7a13` gives 24 arbitrary grayscale and binary images;
  ten radii include negative, zero, exact/next-float support boundaries and masks
  larger than the input. This also checks 120 inclusive length filters, original
  vertical glyph ordering/runs, Gaussian kernel bits and 14 ImageJ ROIs. Six
  original StaffLine geometries add 42 integer yAt samples including half-even
  ties, extrapolation, linear/quadratic/cubic interpolation for header ROIs.
- Two fresh isolated original runs produced identical additional JSON bytes.

## Numerical and boundary conclusions

The native default Gaussian kernel matches the original float bits; it rounds
to bytes between horizontal and vertical passes. Median boundary pixels remain
unchanged. StaffLine constructors add 0.5 to each y; the fixture records
constructor inputs separately and feeds original stored getPoints() values into
the native header adapter. All 42 integer samples match, including ties and
extrapolation. Circular support uses `x*x+y*y < radius*radius+1`; radius `-0.1`
therefore has a five-pixel cross, while exactly zero has only the center.
All 464 original morphology results match two complete native extrema passes,
including degenerate dimensions and structuring elements larger than images.

The native implementation deliberately rejects signed endpoint overflow:
`right == i32::MAX` or `bottom == i32::MAX`. Original ImageJ throws for the former
but silently does nothing for the latter. The test retains that distinction;
it does not label the stricter native rejection as original equivalence.

## Reproduce the additional independent original controls

Use only the already-local, hash-verified development dependencies. From the
repository, make a fresh output directory and use the exact JAR/runtime pins in
`provenance.json` and `docs/LINUX-ORACLE.md`:

```sh
OUT=$(mktemp -d ../beam-research/pixel-review-repeat.XXXXXX)
mkdir -p "$OUT"/classes "$OUT"/home "$OUT"/config "$OUT"/data "$OUT"/cache "$OUT"/prefs
../linux-oracle/jdk-21.0.12.1+1/bin/javac --release 21 -d "$OUT/classes" \
  fixtures/beam-spots-controls-source/BeamPixelReviewProbe.java
HOME="$OUT/home" XDG_CONFIG_HOME="$OUT/config" XDG_DATA_HOME="$OUT/data" \
XDG_CACHE_HOME="$OUT/cache" \
../linux-oracle/official/opt/audiveris/lib/runtime/bin/java \
  -Xmx512m -XX:ActiveProcessorCount=2 -Djava.awt.headless=true \
  -Duser.home="$OUT/home" -Djava.util.prefs.userRoot="$OUT/prefs" \
  -cp "../linux-oracle/official/opt/audiveris/lib/app/*:$OUT/classes" \
  BeamPixelReviewProbe "$OUT/result.json" > "$OUT/console.log" 2>&1
cmp fixtures/beam-spots-controls-additional.json "$OUT/result.json"
```

Full page recognition, whole-system header selection, head sizing, glyph
registration/dispatch and BeamInter construction are separate validation scopes.
The original controls here prove only the explicitly exercised pixel/glyph
operations and native fail-closed bounds.
