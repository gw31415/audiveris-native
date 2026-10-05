# Constructed key/clef scoring oracle

This is a development-only independent numerical observer of the unmodified
Audiveris 5.11.0 Linux JAR. It is **not image-level parity** and adds no Java
runtime dependency to the native executable.

## Inputs and directly observed outputs

`scripts/java/KeyClefControlProbe.java` creates detached original `KeyRoi`,
`KeySlice`, `KeyAlterInter`, and `ClefInter` instances. It supplies explicit
intrinsic grades and measured pitches; it does not classify glyphs. Test-only
`Unsafe.allocateInstance` creates incomplete `KeyBuilder`, `ShapeBuilder`,
`Parameters`, and `Staff` holders, followed by reflection to populate only the
fields needed by the original methods. Pitch limits and support ratios are read
from the original JAR's constants and relation objects, rather than copied from
the native implementation.

For each candidate the observer directly invokes:

- `KeyInter.getPitches`, for all seven pitched clef kinds and both key shapes
- `KeyBuilder.ShapeBuilder.computePitchedGrades`
- `KeyBuilder.ShapeBuilder.computeKeyGrade`, only when original `alterCount > 0`
- `GradeUtil.contributionOf` and `GradeUtil.contextual`

`pitched_grades` preserves the original partially written array after an early
rejection. A rejected candidate has `alter_count: 0`, even when preceding slices
were non-null and received grades. Missing slices contribute zero; the key grade
divisor uses actual original `alterCount`, while the pitch limit depends on total
slice count. Percussion is explicitly marked `percussion_excluded: true`; its
zero array/count are observer placeholders, and the original pitch method is
never called for it.

## Original final-winner observation and explicit limits

The observer also invokes the original private `checkWithClefs`. The containing
clef list is a recording `ArrayList`. Its overridden `retainAll` captures the
singleton passed by the **original selection code**, then throws a dedicated
observer exception before `fillMissingAlters`, `applyPitchImpact`,
`createKeyInter`, or graph edge registration. The captured final winner is stored
both as `expected.best_clef_index` and
`observation.original_selected_clef_index`.

All detached inter objects are marked `removed` before the call. This is an
explicit test construction: the flag is not read by either scoring method or
selection loop, but makes calls to `remove` return without requiring a real
staff/graph. When no positive compatible candidate exists, the original method
finishes with `false` and stuffs its constructed ROI; that boolean is stored in
`observation.check_with_clefs_return`. Intercepted calls have `null` there and
`stopped_at_original_retain_all: true`; the observer does not claim they completed.

`bestCompatibleClef` is a method-local variable, inaccessible to ordinary
reflection. Therefore `best_compatible_index` and its contextual grade are
**source-replayed strict-order selection over directly observed JAR scores**.
`key_accepted` on intercepted calls is also a source-replayed comparison between
that index and the independently observed final winner. Those fields are
explicitly listed in fixture `source_replayed_fields`. This limitation does not
apply to the numerical grades or the captured final winner.

## Coverage

The persisted fixture contains 286 constructed cases:

- All seven pitched-clef arrays, SHARP and FLAT, lengths 1 through 7
- Both signs of exact pitch thresholds and one representable `double` immediately
  below/above each threshold, for lengths 1, 2, 3, 4, 5, and 7
- Empty/all-missing slices, sparse slices with actual-count denominators, total
  slice-count pitch windows, and early-rejection partial arrays
- Percussion exclusion and a percussion candidate winning by intrinsic grade
- No candidate/no compatible candidate, boosted compatible vs stronger
  incompatible candidate, incompatible winner, duplicate-compatible ties, and
  final exact ties in both input orders
- Zero alter grade, zero clef grade, accepted exact threshold with zero pitched
  grade, and 80 seeded varied cases with reordered candidates and missing alters

The original constants observed here are pitch limits `0.5` and `2.0`, with source
ratios `6.0` for both `KeyAltersRelation` and `ClefKeyRelation` (coefficient 5 plus
one). No extra minimum key-grade rejection is introduced: the original source
contains only a TODO for that check.

## Reproduction and evidence

Run from the repository root after bootstrapping the official Linux oracle:

```
python3 scripts/linux_oracle.py --probe key-clef-control --output /tmp/new-key-clef-evidence
```

The harness verifies original JAR SHA-256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`, records compiler,
distribution, dependency and observer hashes, and isolates configuration/cache
paths. Compile-only ABI shims are absent from the runtime classpath.

- Fixture: `fixtures/key-clef/official-controlled.json`
- Observer: `scripts/java/KeyClefControlProbe.java`
- Compact provenance: `evidence/key-clef-oracle-provenance.json`
- Raw runner evidence: `evidence/linux-oracle/key-clef-control-20261002/`
- Pinned source commit: `9e1e55cd2746037d059345881c53e6a6754bffbd`

The native fixture test is responsible for strict input/output length checks,
exact integer/index comparisons, and exact `f64` bit comparisons after JSON
round-trip. It must not relabel the source-replayed local fields as direct JAR
local-variable observation, nor promote this controlled numerical coverage to
full HEADERS, classification, missing-alter filling, or graph-registration parity.
