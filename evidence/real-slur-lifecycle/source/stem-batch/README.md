# One full Stem-cache certification per Slur owner borrow

The read-only v3 runtime audit found no unbounded loop in short-Slur fit/raw support, lookup, pair selection or clump selection. Component union parents strictly decrease. LinearArea admits at most 128 edges, has 1024-link/row/cycle budgets and advances list indices. System Band rectangle extrema are capped at four steps; cubic ordinate solving uses bounded 256-step refinement/fallback. Raw scans advance within retained image dimensions. These bounds rule out an obvious static nontermination path; they are not a sampled runtime profile.

The likely finite hotspot is `NativeOwners::borrow -> Retriever::registered_stem -> Retriever::existing`. Before each individual Stem Arc lookup, `existing` clones the full Graph snapshot, indexes its vertices, copies all system Stem records, validates every cached Stem against them, reconstructs removed Stem glyph geometry, and collects all HeadStem attachments. This work repeats for each Chord in each clump's System owner view. The diagnosis is based on the actual call graph; no profiler was installed or run.

Root's v3 eventually completed after 624.812 seconds, with source stable. The fresh Slur lifecycle marker passed six Slurs, twelve endpoints, twelve raw-eligible candidates, ten clumps and sixteen pending relations. The same image target later failed an old CHECKED expectation that still included two candidate glyphs now removed by Slur cleanup. Overall v3 was therefore not a passing image target. The elapsed time covers the whole target and is not a Slur-only benchmark.

## Narrow production proposal

`batch.patch` adds `Retriever::registered_stems(graph, locals)` and uses it only in `NativeOwners::borrow`. Existing singular callers remain unchanged.

The batch invokes the unmodified `existing(graph)` once before resolving requested locals. Therefore every unrequested invalid cache entry is still checked, including removed/resurrected ownership and live geometry/grade/glyph consistency. Each requested local then uses the same missing/inactive checks and clones the same canonical Arc. Request order and duplicates are preserved. No report supplies ownership. The caller has already obtained typed actual ChordStem keys whose System is checked against each Chord; all keys belong to the selected System owner view.

`NativeOwners::borrow` performs one batch for its stem-bearing reads, consumes the returned Arcs in matching order, and preserves its later actual Chord/Stem identity check. A view with no Stem requests skips the batch, matching the previous absence of individual calls. No cached geometry, clocks, relation callbacks, manual flags, registry entries or source selection criteria are changed.

For a state simultaneously invalid in Stem-cache and Part/Staff ownership, error precedence can move earlier because batch certification precedes per-Chord Part/Staff checks. The invalid state is still refused before publication. No successful-state behavior change is intended. This is not a persistent validation cache: every new owner borrow performs the full certification again, and no stale certificate survives a graph mutation.

## One existing-fixture factory control

`test.patch` adds `stem_cache_sig::batch_tests::registered_stem_batch_preserves_order_identity_and_rejects_unrequested_stale_cache`.

The control uses case 0 from the existing official Stem-cache fixture and a translated second physical glyph through actual `create_selected` and `register` calls. It checks `[second, first, second]` output order and pointer identity; missing requests; an unrequested live Stem with changed grade rejected identically to the old singular getter; actual removal followed by acceptance of the survivor and refusal of the removed requested owner; and unchanged Graph, Registry and cached object state for getter calls. This is a small declared factory/ownership control, not recognition or a new Java observation.

## Provenance and execution

This child wrote only this `/tmp` proposal. No source checkout edits, build, native recognition, Java run, Git operation, process restart or configuration change were performed. Root adopted the production batch after v3 ended; test adoption/execution and the required next DerWinter continuation must be recorded by root separately. `READ-BASES.json` preserves production proposal bases; `TEST-BASE.json` preserves the test-delta base after root adoption. A later faster elapsed time cannot alone prove equivalence: actual lifecycle, cleanup, CHECKED outputs and source comparisons remain required.
