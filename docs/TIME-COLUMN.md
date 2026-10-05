# Native time-column selection

`time_column::analyze(no_staff, grid, clef_registration, key_column, sheet_interline)` consumes native post-key boundaries and returns the time-column state before final header clef selection, interior-bar purge and header freezing.

## Algorithms retained

Each staff builder is allocated first; candidate finding/filtering then advances in staff order and aborts the column immediately on an empty filtered staff. Header adapters are cleaned on this failure, with original list references preserved. Filter creates supported num/den relations within the inclusive center-distance threshold and incompatible exclusions for unsupported nearby pairs. Orphan halves are removed; contextual grades use ordered intrinsic partner contributions. Conflict resolution uses actual glyph pixels when available, with exact weak/equal-versus-strong comparisons. The source retains `wholes` and `nums` against the filtered set but intentionally does not retain `dens`.

Whole values and every supported num/den pair populate system value vectors. Pair intrinsic grade is the mean of its members' contextual grades. Whole-before-pair strict comparison preserves first ties. Values must be present on every standard staff; their system grades are intrinsic means. The source's two HashMaps use deterministic TimeValue/TimeRational hashes: this bounded native implementation reproduces non-tree bucket/insertion traversal, including resize. COMMON_TIME and CUT_TIME remain distinct from numeric fractions despite shared rational hashes.

Graph mutation preserves incomplete live pairs, extensive removal, cached grades, stale builder references, and exact containment listener effects. The original number-loop `instanceof TimePairInter` condition is not silently corrected. Cleanup and discardNeighbors are distinct phases. External GRID/clef/key removals are retained for final HeaderBuilder work.

## State contract

Each system has `after_retrieve_time` and `after_system_time_stop`. Both contain staff ranges/stops and header pointers, the surviving SIG interpretations/relations, and removed existing-object origins. IDs are system-global native diagnostic identities. `origin` maps objects back to GRID IDs, per-staff KeyColumn IDs, computed dummy header bars, or new time interpretations. Clefs, keys, alterations and times retain staff IDs; connectors/brace may have null staff IDs. Header key/time snapshots remain available even if detached from the live graph.

A subtle original distinction is preserved: timeRange.stop is assigned the symbol's exclusive right edge, whereas Staff.getTimeStop returns the selected interpretation's inclusive right edge once header.time exists. System propagation uses that getter value.

## Evidence and limits

71 original rasters (72 systems/97 staffs) pass native final time state and full graph comparisons; 15 staff times are selected in 11 systems. The cut-time raster genuinely removes an existing GRID bar. Thirty-seven independent original controls each execute separate vector and consistency passes: inclusive alignment boundaries, supported/unsupported rationals, orphans, partner support, exact/ULP ties, insertion-order variants, whole-versus-pair, incomplete-pair survivors, strong/equal/weak neighbors, missing staff and incompatible columns.

See `evidence/time-column/native-validation.json`, `evidence/time-column-source.json`, and the original observer provenance. All geometry/state decisions are exact; raster numeric comparisons use 1e-9 for native classifier roundoff and controls use 1e-14. This stage does not read oracle outputs at runtime. Generic font metrics and classifier weights remain separately pinned resources.

Support remains fresh automatic five-line single-scale native GRID with Bravura. Tablature/manual/existing signatures, larger/hash-treeified value maps, expanded same-side exclusion partitions, and intersecting non-glyph curved-area neighbors are explicitly unsupported rather than silently approximated. Upstream image/graph/classifier bounds still apply. Final header completion is a separate stage and `headers_complete` stays false here.
