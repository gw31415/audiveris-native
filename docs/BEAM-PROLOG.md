# Native BEAMS prolog

Pinned Audiveris 5.11.0: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

`beam-spots INPUT SPOTS.pgm REPORT.json` now adds original black-head sizing,
canonical glyph group propagation and system dispatch after the verified pixel
preparation. The image remains the beam-spot threshold mask. The new
`beam_prolog` JSON section contains head/font scale, closed candidates, grouped
system spots and phase-owned native identities.

`last_completed_stage` remains `STEM_SEEDS`; the successful substage is now
`BEAMS.PROLOG`. `beams_complete` and `musicxml_complete` remain false. No BeamInter,
beam-to-stem relation, note pitch/duration, rhythm or MusicXML is invented.

## Exact work and evidence

- Original 128 parameter sets, 728 head check/closing/process controls,
  11 size-measurement lists, and 2,048 generic Bravura font interpolations
- All 13 complete native image pipelines match head/font scales and system spot
  pixels, order, weights, centroids and group sets against atomic original prolog
- Laura has 409 raw prepared spots; 139 and 268 are dispatched to its two systems,
  including the same two dual `BEAM_SPOT` / `BLACK_HEAD_SPOT` canonical objects
- 49,248 direct original integer point queries cover straight, sloped, half-pixel,
  quadratic-extremum, cubic-knot and non-monotone-x boundary paths
- Canonical identity retains origin, dimensions, orientation and exact runs;
  existing accepted seed identities/groups and free-list positions are preserved
- Epilog's separately tested rule removes only free glyphs whose sole group is
  BEAM_SPOT; it is not prematurely run by the native prolog

[Black-head sizing](BLACK-HEAD-SIZING.md) documents the original thresholds,
registration-before-second-check, stable central-half measurements and generic
font-table derivation. [Observer identity](BEAM-OBSERVER-IDENTITY.md) records the
causal GC investigation and the corrected 13 original full-final comparisons.
Those original finals validate observation fidelity, not native BeamInter code.

## Containment and identity

Java Area containment uses ray crossings; equality at a sloped boundary cannot
be replaced with a north-inclusive/south-exclusive y interval. The native caller
assembles original extended/reversed Bezier boundaries and uses the separately
licensed `audiveris-area` curve library. Its retained OpenJDK notices and
GPL-2.0-only with Classpath exception are independent from the Audiveris AGPL
modules. There is no Java runtime dependency.

The native registry owns registered objects during its phase. Its numeric IDs
are native-only. It preserves the immediate original head-sizing-to-dispatch
identity/group behavior; it does not imitate nondeterministic JVM weak-reference
collection or claim its full registry snapshot equals a live weak-index snapshot.

## Bounds and fallback

Current input restrictions remain 8-bit grayscale PNG/PGM, four million pixels,
32 MB, fresh default five-line single-column native GRID, default music family
Bravura and no drum/one-line/smallBeams override. Glyphs/runs/curves/system
references have explicit budgets. Head operations preflight one million total
runs and one billion morphology samples. Dispatch preflights 100 million
conservative curve-query work units, with early duplicate membership rejection.
These are native safeguards, not original Audiveris limits.

A prolog failure retains successful spot pixels and an explicit error with the
last completed substage. Unsupported font sizing preserves a successful measured
black-head scale with a missing-font reason. Neither path silently claims full
prolog completion. Existing output paths are still refused.

BeamStructure/BeamInter construction, hooks, extensions, beam groups,
multiple-measure rests, LEDGERS, automatic HEADS/stem attachment, rhythm,
MusicXML and bounded-memory 300 DPI PDF remain unfinished.
