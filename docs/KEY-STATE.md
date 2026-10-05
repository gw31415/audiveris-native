# Native staff-local key process state

`key_state` ports the automatic staff-local `KeyBuilder.process` sequence from
Audiveris source commit `9e1e55cd2746037d059345881c53e6a6754bffbd`. It stops before
`KeyColumn` cross-staff alignment and finalization. Runtime remains native Rust.

## State and ordering

The module consumes native NO_STAFF pixels, canonical GRID geometry and native
registered-clef candidates. It runs FLAT before SHARP and records three snapshots:

- `after_FLAT.process`
- `after_SHARP.process`
- `after_selectBestClef`

Each snapshot includes the ordered shared clef worklist (including repeated
references), remaining live clefs, both shape ranges/ROIs, retained raw evaluation
and glyph, assigned alterations, optional key ensemble and ordered relations.
Native inter IDs are local to each staff snapshot; they are not Java IDs.

Implemented source behavior includes:

- Strict original competing-clef ordering, scoring and irreversible loser removal
- Hard empty-slice extraction followed by clef-conditioned missing/weak-alter
  retries using `key_components::fill_missing_alters`
- Retained raw evaluation/glyph on non-improving re-extraction, same-glyph alter
  retention and replacement identity handling
- The original pitch-window integer cast, integer half-height and ties-even y
- A single retry-trigger array before filling, then a fresh post-fill pitch-grade
  calculation. Partial array writes survive a later pitch rejection
- Pairwise alteration supports, key-to-member containment and clef-to-key support
- Initial key intrinsic grade from the mean of member best grades, not the
  optimistic simulated key grade used during clef choice. Fresh member contextual
  grades remain null until an explicit upstream operation computes them
- Single-item trailing-space rejection, removing that shape's alterations/key
  while retaining earlier clef elimination
- Conditional final staff-local clef contextualization: a singleton shared list
  skips refresh; a duplicated or multi-candidate list is rebuilt and evaluated

No header key assignment, final key stop, cross-staff replication, time signature,
final HeaderBuilder clef selection or freezing is performed. The output retains
`key_column_finalization_complete:false`, `key_recognition_complete:false` and
`headers_complete:false`.

## Independent evidence

`tests/key_state.rs` compares the complete native image pipeline against 36
original-JAR process fixtures. Three ordered stages per staff are compared for
clefs, ranges, slice state, key membership/grades/bounds and relations. Graph IDs
are mapped by live semantic identity for comparison; their ordering and endpoints
are otherwise preserved.

The fixtures include original music, empty/noise images, disconnected/damaged
alterations, difficult empty slices, two genuine pitch-window recoveries and both
flat/sharp one-item trailing-space failures. The recoveries restore the missing
first flat; the trailing failures leave no key ensemble or slices for that shape.

Integer, count, ordering, nullable-state and boolean fields compare exactly.
End-to-end floating comparisons use an explicit absolute tolerance of 1e-9;
the measured maximum difference is 9.44e-16 across 1,958 compared floating fields.
This is distinct from the pure numerical `key_clef` oracle, whose supplied-input
grade comparisons are bit-exact. The live tests do not claim original Java ID
equality or exhaustive graph/recognition parity.

Five source-derived state tests separately cover original first-space behavior,
singleton versus duplicate clef refresh, actual ensemble mean/member ordering,
destruction with retained clef decisions, and partial post-fill grade writes.

Source hashes are retained in `evidence/key-state-source.json`. Original process
generation and hook/no-hook validation are recorded by
`evidence/key-process-oracle-validation.json`. Focused native logs and numeric
comparison counts are under `evidence/key-state/`. Generated JVM classes/caches
are not runtime inputs or required durable evidence.

```sh
cargo test --offline --locked --lib key_state::tests
cargo test --offline --locked --test key_state
cargo test --offline --locked --release --lib key_state::tests
cargo test --offline --locked --release --test key_state
cargo clippy --offline --locked --lib --test key_state -- -D warnings
```

## Support contract

This stage uses fresh automatic header candidates from the existing bounded
native pipeline. Manual/existing key material, outside-header clefs, ledger-based
pitch remapping and unrelated support types are not included. Existing image,
GRID, single-scale, candidate, projection, slicing and classification bounds still
apply. A failure returns no partial staff-process result. No image/PDF-to-MusicXML
completion is implied.
