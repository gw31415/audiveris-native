# Initial BEAMS candidate handoff

Current CLI contract: [BEAMS epilog](BEAMS-EPILOG.md). The stage reports below describe the named historical checkpoint; current supported single-staff inputs can reach BEAMS.EPILOG. Phase snapshots retain their earlier completion flags.

This is a source-development checkpoint, not a fully swept replacement binary.
The bundled executable remains the verified v16 BEAMS prolog binary at commit
23f4f8f94aa5d9945be0a7a7fceea53f174e5b71. The new source adds initial candidates.
Keep the distinction when reporting progress or running the bundled executable.

## Task and scope

Continue a pure native Rust CLI port of official Audiveris 5.11.0, pin
9e1e55cd2746037d059345881c53e6a6754bffbd. Java is a development oracle only.
Use the requested cloud environment. Do not use the user's Mac, publish a repo,
add authentication, or claim a complete OMR/PDF/MusicXML implementation.

A prior network fetch of the following source batch was cancelled. Do not retry
it or use another route: raw.githubusercontent.com/Audiveris/audiveris/...,
glyph/dynamic/{StickFactory,StraightFilament,Filament,SectionCompound,LinkedSection},
sheet/{PageCleaner,StaffManager}, check/{CheckSuite,Check,SuiteImpacts}, sig/GradeImpacts.
Use retained source, bytecode and evidence. Licenses/provenance must survive.

## Verified base

v16 has 212 passing debug tests and 212 release tests, fmt/clippy/release build,
and five CLI comparisons with an empty environment and PATH=/nonexistent.
Native processing reaches full supported HEADERS/STEM_SEEDS and BEAMS.PROLOG:
black-head/font sizing, exact glyph group identity and system dispatch. The
original Laura observer discrepancy was resolved by avoiding diagnostic-triggered
weak-reference garbage collection inside the original prolog.

## Source additions in this handoff

- src/beam_structure.rs: original sections/borders/line geometry, adjustment,
  split and jitter. 144 original raster calls, 279 direct controls, 27 method
  controls, 1,002 independent stress controls. fdlibm hypot fixed a real split
  branch divergence; 5,321 original Java vectors compare exactly
- src/beam_impacts.rs and audiveris-area/linear.rs: original core/belt masks,
  grading and acceptance. Exact normalized subtraction corrected one boundary
  pixel. Cached contour-order Rectangle2D bounds corrected seven boxes/three
  totals. 307 direct controls, 576 extra parallelograms/424,201 integer queries,
  three denominator/grade controls, eight NaN/zero-jitter controls pass
- src/beam_candidates.rs: original initial createBeams/checkBeamGlyph/
  createBeamInters and glyph retrieval, canonical registry, exclusions, standard
  then measured-small retry, bounded work, honest incomplete flags
- src/main.rs: appends candidate report to existing native CLI; preserves the
  serialized prolog on new-stage errors. Public analyze rejects incomplete
  prolog font state before mutation. CLI marker BEAMS.INITIAL_CANDIDATES is only
  for successful candidate output. Full last_completed_stage remains STEM_SEEDS
- scripts/java/BeamCandidatesProbe.java: calls original private createBeams
  after atomic prolog, then continues original downstream methods. All 13 whole
  original finals still exactly match the plain observer. Independent original
  fixtures are fixtures/official-beam-candidates; native expectations were never
  used to generate them

All 13 native image candidate comparisons passed again after the final Context
refactor and partial-prolog guard: 27 initial inters, one exclusion; exact spot order,
assigned glyphs/counts and canonical fields (finite values tolerance 1e-8).
483 original glyph metric controls passed. The final focused debug run passed
all seven tests, including the partial-prolog nonmutation guard and one actual
CLI case with PATH=/nonexistent and existing-output refusal. Latest checks are in
evidence/beam-candidates-checkpoint and component review directories. Do not
interpret focused component passes as a full workspace debug/release pass.

## Required next verification

1. Inspect current source commit and retained review evidence; address any open
   candidate-review issue before proceeding
2. Run cargo fmt --all -- --check; cargo clippy --workspace --locked --offline
   --all-targets -- -D warnings; debug/release workspace tests; release build
3. Run actual CLI beam-spots on empty, double, sloped, triple and Laura fixtures
   with env -i PATH=/nonexistent. Compare masks/prolog/initial candidates to the
   stored original oracle; verify refusal preserves existing output files
4. Save a fully verified checkpoint and freshly built binary under the existing
   Library identity. Preserve v15/v16 histories; don't replace historical labels
5. Only then continue original extensions/assigned-spot removal/late hooks,
   groups/MultipleRests. Full BEAMS remains incomplete until separately verified

## Reproduction and environment transfer

Cargo.lock and all source/assets/fixtures/evidence are in the archive. Normal
Cargo toolchains may be used; the dot workspace used Rust 1.98.1 and absolute
CARGO_HOME/CARGO_TARGET_DIR paths to prevent duplicate rebuilds. Its toolchain,
Cargo cache, official JAR and complete JDK runtime are outside the source tree
and are not implicitly transferred by a task prompt. Original development
commands/provenance retain their historic paths, which require remapping in the
consumer. Runtime code does not depend on those paths.

Official JAR SHA256:
9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5.
Do not invent its presence in a new executor. Check available files/tools and
report an actual missing-oracle blocker rather than using fixture data as the
implementation. Read-only stored fixture comparisons can run without Java.

The previous dot build workspace had 4.91 GB free after authorized removal only
of generated target/debug/incremental. Source/evidence/archives/release binary
and dependencies were preserved. There is no requirement to copy build caches.

## Honest unfinished work

No beam extension or late hook discovery, beam groups, MultipleRests, LEDGERS,
automatic HEADS/stem attachment, rhythm, MusicXML, or bounded-memory 300 DPI PDF
input. Current four-million-pixel and fresh single-column GRID limits remain.
Do not claim full BEAMS from exact initial-candidate comparisons alone.

## 2026-10-03 cloud restart lifecycle checkpoint

Starting from recovered v17 `18639b22cc33c1194d4be1ca4a661ce7bb5c1a90`, the
restart branch adds pure sideAreaOf, middleArea and mergeOf controls, typed
candidate identity, and fresh pre-group accepted-merge registration/removal.
The earlier minimal-Sheet observer setup failure is resolved by real official
Book/SheetStub/Sheet initialization. Six original lifecycle snapshots now match
native identity-normalized state, including rejection no-op and exclusions.
See `evidence/beam-lifecycle-checkpoint` and `docs/BEAM-CANDIDATES.md` for scope.
No eligibility lookup or extension orchestration has been implemented.
Next bounded step: one original-controlled eligibility lookup boundary, before
any whole-page extension loop. Full BEAMS/OMR and release verification remain
unfinished. The later 7cdd component was not retrieved or compared.

## 2026-10-03 side lookup continuation and next gate

One additional read-only boundary, getSideBeam, is now original-controlled:
300 cases match rectangle intersection, full gap ordering and chosen ID.
The prior six-state merge lifecycle controls remain unchanged and passing.
No full extension loop was added. Evidence: `evidence/beam-side-lookup-checkpoint`.

Before adding any further stage, create one complete workspace checkpoint:

1. Run `cargo +1.98.1 test --workspace --locked --offline -j 2` and save actual
   timing, per-target counts and logs.
2. Run `cargo +1.98.1 test --workspace --release --locked --offline -j 2` with
   the same evidence, then `cargo +1.98.1 build --release --locked --offline`.
3. Run fmt check and workspace/all-targets clippy; record the fresh binary hash
   separately from the historical bundled v16 binary.
4. If any check fails, fix/recheck that regression before any further feature.
   Save the source checkpoint and cumulative v17 delta to the same Library ID.

These full checks are planned, not claimed complete by the focused lookup tests.
