# Native key column

`key_column::analyze(no_staff, grid, clef_registration, sheet_interline)` processes fresh native GRID staff inputs through `KeyBuilder.process`, retaining mutable interpretations across cross-staff work. It does not replay oracle recognition outputs. Every raster, glyph, classifier score, candidate, interpretation, relation, and mutation is produced natively.

## Order and contracts

- FLAT precedes SHARP, including the existing staff-local clef/key feedback.
- Builders are staff-ID ordered; system parts come from native `grid.systems[].parts`, not inferred from adjacency.
- Multi-staff systems gather offsets from every slice of each best local key. Original Gaussian EM uses per-index means, sigma 1, uniform mixtures, up to ten iterations, and mixture convergence below 1e-10. Rounding is half-even.
- Replication preserves exact nearest/tie bounds, absolute local/source alignment checks, current ROI insertion order, pitch windows and ink thresholds. It distinguishes already-consistent, no-clef, no-replication, shrink and system destruction.
- Old ensemble removal is extensive: old contained alterations can become removed while slice references remain. Cached key contextual grades are not silently recomputed after membership changes. Newly replicated keys are not contextualized simply by adding their clef relation.
- Finalization retries missing alters without neighbor cropping, adjusts assigned pitch while preserving measured pitch, freezes ROI alterations, records starts and header key range, and removes the competing shape.

`systems[].final_state` is the original pre-propagation column state. `returned_key_column_offset` is the maximum staff-relative key stop. `after_system_key_stop` applies the original HeaderBuilder setter to every system staff only when that offset is positive. IDs are local per-staff diagnostic identities, not Java/SIG numeric IDs. `frozen_key_alter_ids` excludes pre-existing GRID objects.

## Evidence

51 independent original rasters include 15 two-staff renderings. Native final state, key members, ranges, pitches, grades, frozen alters, relations and returned offsets match the hook-free-validated original column observer. The matching original time-column entry snapshots independently verify HeaderBuilder propagation. Thirteen finite controlled original EM cases match means, sigmas, mixture weights (1e-12) and exact rounded offsets; twenty original Gaussian probability controls cover the 1e-10 boundary and adjacent floating values.

See `evidence/key-column/native-validation.json`, `evidence/key-column-source.json` and `evidence/key-column-oracle-validation.json`. Ordinary integer state is exact; existing native classifier floating differences are compared within 1e-9. Recognition is bounded to the fresh five-line, single-scale GRID domain already supported upstream. Seven slices and all upstream image/graph/classifier budgets remain enforced. Degenerate EM posterior underflow is explicitly unsupported; the original instead propagates NaN and casts offsets to zero. Manual/existing interpretations and tablature are unsupported.

This finishes the bounded key-column stage. Time signatures and final header clef selection/freezing remain separate stages; `headers_complete` stays false.
