# Initial BEAMS candidate boundary review

## Outcome

The existing two focused original-parity tests passed in this independent review:

- `original_glyph_metrics_match_all_thirteen_raster_diagnostics`: 483 original glyph metric rows
- `original_beam_spots_match_complete_native_pipeline`: all 13 original image pipelines, including initial candidates, order, assigned spots, geometry, impacts, glyph pixels and relations

Command: `cargo test --locked --offline -j 2 --test beam_candidates --test beam_preparation original_ -- --nocapture`

Environment: dot cloud checkout, PATH `/workspace/shared/audiveris-cloud-continuation/toolchain/bin`, CARGO_HOME `/workspace/shared/audiveris-cloud-continuation/cargo-home`, CARGO_TARGET_DIR `/workspace/shared/audiveris-cloud-continuation/target`.

Exit code 0; 2 tests passed. Other tests were filtered, including the singleton geometry boundary and CLI test. This was not a full test/clippy run. The command began before the final handoff edits; later edits need rerunning in the destination task.

## Finding reported to implementation owner

At review time, `beam_prolog::build` can return an `Ok(Prolog)` whose `head_sizing.music_font_unsupported` is present. `Prolog::report` then reports `beam_prolog_complete=false`, but main.rs kept `Some(context)` and ran `beam_candidates::analyze` regardless. Candidates could consequently report complete and advance last_completed_substage despite incomplete prolog. Gate candidates on completed prolog (and preferably enforce the same prerequisite in the public analyze function). The existing 13 source fixtures do not cover this branch. The owner was notified; no production edits were made by this reviewer.

## Source equivalence checked

- Original `buildBeams` selects BEAM_SPOT only and applies `Glyphs.byFullOrdinate` (top, left, registered ID). Native order agrees and every selected fixture order matches.
- `createBeams` retries measured small-beam parameters only if the standard check returns a reason. Both checks use cue=false. Native orchestration matches. Configuration-forced smallBeams remains explicitly unsupported.
- `checkBeamGlyph` checks width, mean thickness, centerline slope, global border distance, resulting width and slope divergence in that order, then adjustSides/extendMiddleLines/splitLines. Native order and constants match locally retained bytecode.
- `createBeamInters` uses first-top/last-bottom jitter, attempts eligible standard hooks before beams, suppresses hook creation for small parameters and adds an overlap Exclusion when both hook and beam are accepted. Native code uses the actual inherited minimum acceptance grade (0.8*0.1), not the separate good-beam threshold.
- `retrieveGlyph` clips original inter area bounds to the sheet rectangle, preserves that entire rectangle as the run-table dimensions, samples black pixels contained by the inter, creates vertical runs and registers the original glyph. Native code preserves rectangular padding rather than tight-cropping pixels. `AbstractHorizontalInter.contains` confirms area containment for a fresh inter with no glyph.
- Registry registration is global/phase-owned and content-keyed; no group mutation or new free-glyph system membership is added by candidate registration. Original beam registration likewise adds the SIG vertex, retrieves/registers a glyph, then sets it on the inter. Native output uses inline canonical glyph content, not a durable glyph-ID reference for future mutable graph operations.
- Glyph.computeLine traverses canonical runs in position order and pixels in descending along-run order, adding absolute origin coordinates. Native geometry follows this and the original BasicLine.toCenterLine endpoint shifts.

## Bounds and failure review

- Input image binary/size validation, 100,000 selected spots, conservative aggregate work, per-system inter count, registry aggregate runs, structure and sampling bounds are explicit native restrictions.
- Preflight selection validates IDs and estimates work before registration. Later errors can leave the in-memory phase registry partially changed. The API explicitly requires discarding the failed context; main.rs saves a serialized prolog before candidate mutation and returns an incomplete candidate report. That snapshot-preserving structure is correct.
- No additional focused retry, registry-identity, memory-stress or error-path controls were added or run before handoff. No memory high-water measurement was performed. Do not infer bounded-memory production-readiness from the successful fixtures.
- Native limit failures deliberately propagate as unsupported-input errors rather than emulate Java's catch/log/continue behavior. Original per-item beam exception handling can preserve a hook and continue after an exception; parity is only claimed inside native supported bounds.

## Evidence

Pinned upstream commit: `9e1e55cd2746037d059345881c53e6a6754bffbd`.
Local official JAR SHA-256 verified: `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.

Existing original `BeamCandidatesProbe.java` independently calls original private createBeams after atomic prolog, then continues the remaining original BEAMS methods. Existing evidence reports full-original continuation exact on all 13 fixture results; this review inspected those records but did not rerun the Java whole-pipeline observer.

`Glyph.javap.txt` and `AbstractHorizontalInter.javap.txt` in this directory were generated with the locally cached JDK javap against that verified JAR. No network/source fetch, user Mac access, publication or auth change occurred.

Full beam extensions, late hooks, groups, MultipleRests and downstream recognition remain unfinished. Review stopped at the requested WIP handoff boundary; no later-stage work began.
