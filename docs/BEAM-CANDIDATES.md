# Native BEAMS initial candidates

This checkpoint translates Audiveris 5.11.0, pinned commit
`9e1e55cd2746037d059345881c53e6a6754bffbd`, through the initial
`BeamsBuilder.createBeams` boundary. It is a partial BEAMS stage.

## Usable CLI

```
audiveris-native beam-spots INPUT.png SPOTS.pgm REPORT.json
```

The grayscale PNG/PGM input is limited to four million pixels. Output files must
not already exist. `SPOTS.pgm` remains the prepared beam-spot mask. The report
now adds `beam_candidates`: ordered spot traces, assigned spot IDs, initial
beam/hook geometry and pixel glyphs, individual grade impacts, and exclusions.
The initial candidate snapshot keeps its own partial `beams_complete=false`.
The top-level `last_completed_substage` reports the furthest completed native
boundary: supported single-staff inputs now reach `BEAMS.EPILOG`; the candidate
marker remains the fallback when later processing cannot complete.
`last_completed_stage=STEM_SEEDS` and `musicxml_complete=false` remain intentional.
See [the current CLI contract](BEAMS-EPILOG.md). The stored prolog snapshot precedes
candidate registration and survives bounded candidate-stage errors.

No Java, subprocess, network, font rendering, or expected fixture is required at
runtime. `page-glyphs` and `staff-mask` append the same bounded reports while
retaining their original image output semantics.

## Original behavior retained

- Only BeamSpot-group glyphs are considered, sorted by original full ordinate
  (top, left, ID). Original default standard parameters are tried first; measured
  small-beam parameters are retried only on rejection
- Width, mean-thickness, centerline slope, border distance, resulting width and
  slope-difference checks precede side adjustment, middle extension and split
- First top and last bottom jitter contribute the original distance impact
- Eligible standard items can create both a hook and beam, with an Exclusion
  relation; inherited minimum grade is distinct from the good-beam threshold
- Pixel scoring uses exact normalized core and subtracted belt areas, integer
  sample coordinates, original clipped-foreground/unclipped-denominator policy,
  cached bounds rounding, and weighted geometric grades
- Retrieved glyphs use the original clipped rectangular buffer and vertical runs.
  They are not cropped to a connected-component bounding box. Registry insertion
  retains content identity and previously held group ownership
- Candidate IDs are native phase IDs. Canonical comparison omits incidental JVM
  allocation IDs but compares pixels, order, geometry, grades and relations

`src/beam_structure.rs`, `src/beam_impacts.rs` and the independently licensed
`audiveris-area` numerical routines provide the underlying algorithms. Their
separate notices and exact original controls are retained.

## Independent evidence

`BeamCandidatesProbe.java` runs the original private createBeams method after
an uninterrupted original prolog. It then continues the remaining original
BEAMS methods and verifies the resulting whole final against the plain original
observer on all 13 images. Native candidate output is never used to create the
oracle. Original fixture and source hashes are in `evidence/beam-candidates-oracle`.

The native image pipeline is compared against all 13 original initial outputs,
including Laura's two systems: 27 initial beam/hook inters and one exclusion.
Spot order, assigned glyph pixels and relation/inter cardinalities are exact;
finite geometry and grades use an absolute 1e-8 comparison threshold. Component
review additionally covers 483 glyph metrics, 1,002 stress geometries, 5,321
original Java hypot vectors, 307 item grading cases, 576 extra parallelogram
cases, and strict boundary/denominator/acceptance regressions.

## Explicit limits and remaining work

Page-level candidate work is conservatively capped at one billion units,
100,000 spots and 100,000 inters per system. Component-specific run, section,
line, sampling and curve-work limits also apply; failures are unsupported-input
errors, never claimed original rejections. Failed mutable phase contexts must be
discarded; the CLI retains the already serialized prolog.

This boundary does not implement removal/extension of assigned spots, late hook
discovery, beam groups, MultipleRests, LEDGERS, HEADS/stem attachment, rhythm,
MusicXML, or bounded-memory 300 DPI PDF input. It does not enable a smallBeams
configuration override or broaden the existing fresh single-column GRID scope.

## Cloud restart continuation: pre-group merge lifecycle (2026-10-03)

The recovered v17 source now retains typed candidate identity and supports an
already evaluated merge through `SystemCandidates::apply_merge`. Rejection is a
no-op. Acceptance registers the new glyph/inter, appends it in raw order, then
removes both inputs from active lookup and deletes their incident exclusions.
Removed inters remain in raw order; IDs are not reused. Glyph identities and
groups remain in the shared registry. Supply the same prolog registry that owns
the candidates. Mutation sampling is cumulatively capped at one billion units
per system, separately from initial candidate work.

`BeamLifecycleFixtureProbe` initializes a real original Book/SheetStub/Sheet via
LOAD/BINARY/SCALE and GRID. It records six original states for one standard-beam
accept/reject lifecycle, including registration order, exclusion deletion,
idempotent removal, glyph reuse and group retention. Native IDs are matched by
registration order, not JVM numeric counter values. The original graph and
registration/removal methods are unchanged; compiler-only Book/RunClass stubs
are absent from the runtime classpath. Evidence is in
`evidence/beam-lifecycle-checkpoint`.

This is the fresh candidate phase before beam groups, chords or ensembles. It
does not claim general graph lifecycle equivalence, small-beam lifecycle
coverage, eligibility lookup, or extension orchestration. Those remain separate
bounded tasks. The CLI still stops at initial candidates.

## Cloud restart continuation: one side-candidate lookup

`beam_side_lookup::get_side_beam` and `SystemCandidates::lookup_side` now port
the original `BeamsBuilder.getSideBeam` boundary. It queries the side area
against inter rectangles, skips removed inters and the target, filters slope
difference and endpoint-to-line distance, and stably orders by signed horizontal
gap. Default reach is four interlines, maximum vertical gap is round-to-even
0.25 interline, and maximum slope difference is 0.065. Original constant
overrides are not supported. Supplied reach includes zero and subpixel cases.
Diagnostic Java area attachments remain omitted as in the existing side geometry.

The original observer executes unchanged getSideBeam on 300 independent cases,
records original rectangle intersections, and obtains the full order by removing
each selected entry from a temporary input list and calling the method again.
It covers both sides, five interline scales, positive/negative slopes, removed
entries, reversed raw order, equal gaps and empty/reversed side areas. All
intersections, full ordered IDs and selected IDs match; 147 cases select a beam
and 153 return no candidate. No native outputs generate the expected results.
`LinearArea::intersects_rect` uses the retained OpenJDK line-crossing sources;
separate controls distinguish boundary contact and holes from overlap.

This adds no lookup loop, merge dispatch, or full extension orchestration.
The next checkpoint must run the complete workspace debug and release suites
before any further stage is implemented.
