# Native key/clef scoring and decision

`src/key_clef.rs` ports the pure numerical portion of Audiveris `KeyBuilder` at
commit `9e1e55cd2746037d059345881c53e6a6754bffbd`. It consumes explicit ordered
clef candidates and optional, already classified alterations. It does not run a
classifier, fill missing alterations, modify a graph, create a `KeyInter`, or
complete key/header recognition.

## Typed API and contract

- `pitches(ClefKind, KeyShape)` returns the exact seven-element `KeyInter` table
  for SHARP or FLAT and each of TREBLE, BASS, BARITONE, TENOR, ALTO,
  MEZZO_SOPRANO, and SOPRANO. Percussion has no pitch table
- `compute_pitched_grades(kind, shape, slices)` accepts ordered
  `Option<KeyAlter>` entries with `intrinsic_grade` and `measured_pitch`
- `compute_key_grade(alter_count, pitched_grades)` computes the inter-alter
  contextual average; its positive divisor is the actual number of non-null
  alterations, including present alterations with zero pitched grade
- `check_with_clefs(shape, clefs, slices)` accepts ordered `ClefCandidate`
  records with `kind` and `intrinsic_grade`. It returns per-candidate diagnostics,
  best-compatible and overall-winner indices, and `compatible_winner`

`compatible_winner: true` is only the decision that the compatible clef wins. It
does not report that downstream recognition has executed. These helpers do not
mutate their inputs or invoke Java/subprocesses. Callers own the candidate order
and any later graph operations.

The native adapter accepts at most seven slices and 64 clef candidates, finite
measured pitches, and finite grades in `[0,1]`. Invalid inputs return explicit
`KeyClefError` values. Intrinsic grades are already upstream `Inter.getGrade()`
values; the adapter does not multiply them by `Grades.intrinsicRatio` again.
An empty slice list returns zero alterations before looking up a pitch table;
direct nonempty percussion pitch scoring is an error, whereas clef selection
explicitly skips percussion pitch scoring.

## Source details retained

The pitch-error limit is `0.5`, `1.0`, `1.5`, or `2.0` for respectively one, two,
three, or four-plus total slices, including missing slices. Only a strictly
greater error rejects the candidate. Equality remains accepted with pitched
grade zero. On an excessive error the returned alter count is zero, while
earlier output-array writes are preserved and remaining entries stay zero.

Both `ClefKeyRelation` and `KeyAltersRelation` override the source coefficient
with 5. A fresh `Support` has grade 1, producing source ratio 6 and contribution
`partner * (ratio - 1)`. The contextual formula and nested addition order match
`GradeUtil` and `computeKeyGrade`; no reordered sum or extra minimum key-grade
rejection is introduced.

The first selection pass skips percussion and chooses the strictly greatest
positive compatible contextual score, starting at zero. The second pass runs
only when that candidate exists. Only that candidate receives its contextual
score; every other candidate, including percussion, competes with its intrinsic
grade. Both passes preserve the first candidate on equality. Duplicate kinds
remain distinct ordered candidates.

## Verification and its boundaries

The independent fixture contains 286 constructed controls covering 950 ordered
clef candidates, 4,000 pitched-array entries (including excluded-percussion
placeholders), 286 numeric key grades, and 286 numeric contextual grades. All 14
pitch tables are checked. Native comparison uses exact `f64` bits after JSON
round-trip, exact indices/counts/kinds, and explicit array-length checks before
pairwise comparisons. Eleven additional unit tests cover source rules, error
contracts, resource limits, missing slices, partial rejection, and tie order.

The original JAR directly supplies the pitch arrays, pitched grades, key grades,
and contextual grades. Of the decision controls, 230 capture the original
`checkWithClefs` final winner at `retainAll`; 56 observe its original `false`
return. The private best-compatible local is not directly observed:
best-compatible index/contextual score and acceptance on intercepted calls are
explicitly source-replayed diagnostics over original numeric outputs and the
observed final winner. Percussion zero arrays/counts in the fixture are marked
unscored observer placeholders. The native output represents that exclusion as
`pitched: None`.

This is constructed method-level parity, not image-level key recognition or a
completed `checkWithClefs` mutation path. The observer stops before missing-alter
filling, pitch application, key construction, and graph registration. See
[the oracle method and provenance](KEY-CLEF-ORACLE.md) for the exact construction
and observation limits.

Focused checks from the repository root:

```sh
cargo test --locked --lib key_clef::
cargo test --locked --test key_clef
cargo test --locked --release --lib key_clef::
cargo test --locked --release --test key_clef
cargo clippy --locked --test key_clef -- -D warnings
```

Evidence:

- [Preserved source hashes](../evidence/key-clef-source.json)
- [Independent original-JAR provenance](../evidence/key-clef-oracle-provenance.json)
- [Controlled fixture](../fixtures/key-clef/official-controlled.json)
- [Native fixture comparison](../tests/key_clef.rs)
- Native focused-check logs: `evidence/key-clef-native/`
