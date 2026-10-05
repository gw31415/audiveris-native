# Atomic BEAMS prolog observation and glyph identity

## Result

The corrected development-only `BeamsFaithfulProbe` matches the unmodified original `BeamsStep.doit` whole final JSON on all 13 rasters. This includes Laura's two remaining `BEAM_SPOT`/`BLACK_HEAD_SPOT` glyphs at `[993,629,15,13]` and `[811,1055,15,13]`. Masks, raw extracted spots, and replayed BeamStructure diagnostics remain byte-for-JSON-value exact to the earlier preparation observer.

The corrected records are in `fixtures/official-beam-prolog/`; hashes, original command lines, logs, source snapshots and comparisons are in `evidence/beam-prolog-oracle/`. The old `fixtures/official-beams/` and `evidence/beam-preparation-oracle/` files are unchanged historical v15 evidence. Their limitation was real; it is not silently rewritten.

This validates the original observer, not native BeamInter construction or completion of BEAMS.

## Established cause

The pinned 5.11.0 JAR at upstream commit `9e1e55cd2746037d059345881c53e6a6754bffbd` has the following lifetime-sensitive sequence:

1. `BlackHeadSizer.closeBlackHead` creates a new closed glyph, calls `GlyphIndex.registerOriginal`, and returns its canonical object. `process` adds `BLACK_HEAD_SPOT` or `BLACK_STACK_SPOT` to that object. Its singles/stacks lists are temporary.
2. `GlyphIndex` stores weak references, including its content-to-original lookup. A closed glyph that differs by object identity from the original extracted spot can become collectible when sizing returns.
3. `SpotsBuilder.dispatchSheetSpots` registers each raw extracted spot, uses the returned canonical object, adds `BEAM_SPOT`, and strongly retains it in `SystemInfo.freeGlyphs`.
4. The BEAMS epilog removes only glyphs whose sole group is `BEAM_SPOT`. A reused head-tagged canonical glyph has two groups and remains.

The old explicit-boundary probe serialized all extracted spot geometries between steps 1 and 3. Those allocations provoked GC and removed otherwise-reusable head-tagged weak originals. It consequently dispatched fresh, untagged raw objects and removed them in the epilog.

## Causal controls

`evidence/beam-prolog-oracle/identity-controls/validation.json` summarizes 13 retained control runs, with original output, execution arguments and GC logs beside it. All control runs agree with original plain final output apart from the two grouped spots where noted.

- Immediate original `buildSpots` → `dispatchSheetSpots`: both survive
- Original `doProlog`, and plain original `doit`: both survive
- Spot serialization between build and dispatch: neither survives
- Forced `System.gc()` at the same boundary: neither survives
- Retaining the current original index objects while forcing GC: both survive
- Retaining the same objects while serializing spots: both survive
- Full old observer with temporary retention at that boundary: full final output becomes exact to plain

The weak-index inspection control sees 472 live entries before forced GC and 378 after it. Both target `BLACK_HEAD_SPOT` objects disappear. In ordered logs, the serialization control shows two collection events between its post-build and post-serialization markers; the immediate control has none between post-build and dispatch. This is direct evidence of the mechanism, not merely a suspected cause.

The control's historically named `gc_counts` field contains `System.nanoTime()` boundary markers, not GC counts. The bundled jlink runtime lacks `java.management`, so actual collection evidence comes from `-Xlog:gc` output. An attempted Epsilon-GC run exhausted its 512 MiB heap and is not used as evidence.

## Faithful observer boundary

`BeamsFaithfulProbe` calls the original `BeamsStep.doProlog` atomically. It never serializes glyphs, replays masks, collects the index, or forces GC between head sizing and dispatch. After original dispatch has established the system-owned glyph references, it captures:

- Head and music-font scale
- System x bounds and exact `Area` paths
- Dispatched canonical glyph geometry, groups, weights and centroids
- Currently live weak-index entries before and after prolog

Later mask diagnostics use original filtering/morphology methods on separate buffers. Raw spot diagnostics use `GlyphFactory.buildGlyphs`; they do not rerun `BlackHeadSizer` on the live sheet. The original per-system builders and original epilog then run with the original prolog context.

Index snapshots are explicitly labeled as currently live entries, not a complete registration stream. Closed objects that are not used by dispatch remain subject to the original weak-reference lifetime. Use the separate original BlackHeadSizer control when a complete closed-glyph stream is required. Canonical identity is bounds, orientation and exact run-table geometry; Java numeric IDs are deliberately excluded.

The native registry strongly owns registered objects through its phase. It models original immediate-dispatch content identity and group propagation; it does not simulate JVM collection schedules.

## Reproduction

The dependencies must already exist locally; this command does not request network bootstrap:

```sh
python3 scripts/linux_oracle.py --probe beams-faithful \
  --fixtures fixtures/official-grid/beams-*.png laura-gray \
  --output /path/to/new-evidence-directory
cargo test --locked --offline --test beam_observer_oracle
```

Every corrected full final is compared to the independently retained original plain-step archive. The regression test also verifies all legacy fixture hashes and the unchanged masks, extracted spots and structure diagnostics.
