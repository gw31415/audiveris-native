# Final GRID work inventory and native IR contract

Source pin: Audiveris 5.11.0 `9e1e55cd2746037d059345881c53e6a6754bffbd`.
Target in this checkpoint: ordinary five-line, single-interline, single-column layouts. Full GRID remains false until both geometry and interpretation stages are complete.

| Source methods | Work boundary |
|---|---|
| BarsRetriever.buildColumns / detectStartColumns / refineRightEnds | Existing geometry port; other filters must operate on the same peak graph |
| purgePartialColumns / purgeTooLeft / purgeLeftPeaks / purgeUnalignedBars / purgeExtendingPeaks / purgeCClefs | Implement peak mutation and preserve edge/column consistency |
| detectBracePortions / buildBraces / purgeLeftOfBraces / verifyLinesRoot | Brace and starting-line semantics; not implied by ordinary bar support |
| detectBracketEnds / detectBracketMiddles | Bracket detection; explicit pending boundary |
| groupBarPeaks / partitionWidths / createInters / recordBars | Thin/thick bar interpretations with source pixels and Staff references |
| createConnectionInters / extendConnection / groupBarlines | Geometrically justified connectors and SIG relations; no invented links |
| createGroups / createParts / contextualize | Instrument grouping and contextual grades; preserve pending status where unsupported |
| Staff.simplifyLines / StaffFilament.toStaffLine / StaffLine.simplifyPoints | Persist actual final lines, Y+0.5, adaptive subdivision, thickness and glyph reference |
| GlyphIndex.registerOriginal / Glyph.equals | Deduplicate exact origin, orientation, dimensions and run table; native IDs are not Java IDs |
| StaffLineCleaner.process / LagManager.rebuildHLag / dispatchRuns / buildHorizontalLag | Rebuild horizontal sections from exact residual; retain original vertical lag |
| SystemInfo.updateCoordinates / SystemManager.computeSystemArea / dispatchHorizontalSections / dispatchVerticalSections | Place sections by their integer centroid in actual system areas, allowing multiple memberships |
| StaffManager.computeStaffArea / SystemManager.checkIndentations / allocatePages | Staff area and page/movement topology require separate coverage |

## Stable downstream IR rules

- IR schema is versioned. `grid_complete`, per-stage status and unsupported reasons are data, not inferred from output presence.
- `system_id` and `staff_id` refer to actual detected topology. `staff_ids` is ordered. A staff's line objects reference registered glyph IDs containing actual pixels.
- Glyph IDs are deterministic native registry identities. Equality requires equal integer left/top, orientation, width/height and canonical runs. Never copy Java numeric IDs or create semantic equivalence merely because numbers line up. Independent comparison maps entities by content.
- Rebuilt horizontal and retained vertical section IDs have separate namespaces. Pre-removal horizontal section IDs cannot be reused as if they named post-removal sections.
- A section may belong to more than one system when the areas overlap. Membership uses the source centroid/area rule, not nearest-staff guesses.
- Raw four-connected components and their isolated candidate scores remain components, not notes. They are separate from bar/brace/bracket interpretations and from recognized musical entities.
- Edges must carry the source geometric/recognition evidence and endpoints. Pending interpretations, parts and contextual grades stay absent or explicitly pending; they are not synthesized for MusicXML convenience.

This document is updated with verified implementation coverage before the checkpoint is saved.

## Implemented and independently verified in this checkpoint

- Persistent StaffLine point conversion, thickness, simplification and glyph association. AnLaura's first 29 lines simplify to 2 points; the last keeps 17. All floating points/thickness match the official in-memory GRID observation within 1e-9.
- Exact-pixel registry covers the implemented regular-bar and staff-line glyphs. AnLaura registers 48 native originals corresponding bijectively by content to 48 official originals; 2 additional official brace glyphs remain outside this implementation. Shared bar glyphs across staff interpretations actually reuse one registry entry.
- `rebuildHLag`, short-vertical-run dispatch, horizontal section construction and `updateCoordinates` are ported. Original vertical sections are retained. Every bound, first position, run, integer centroid and ID matches: 6,162 horizontal and 3,673 vertical sections on AnLaura.
- Single-column system area boundaries use the adjacent system's outer staff spline with 0.9-interline margin, extended horizontally at endpoint Y, as upstream. Actual Bezier X is solved for area containment; this deliberately differs from GeoPath's endpoint-normalized interpolation. All system membership IDs match: system 1 has H=1,645/V=1,390; system 2 H=4,552/V=2,346. Overlap is real and retained (H=35/V=63).
- Ordinary-bar branch includes partial-column / too-left / left / unaligned / extending purges, whole-column deletion, width grouping/partition, `createInters`, `createConnectionInters`, `groupBarlines`, `recordBars`, and the non-excluding BarConnection support-grade formula. Native IDs are mapped by geometry and glyph content during comparison, never by Java ID value.
- The AnLaura ordinary-bar subset has 27 BarlineInter + 9 BarConnectorInter; every shape, staff association, bound, intrinsic/contextual grade and all 30 relation endpoints/kinds match official final GRID. The synthetic fixture has 7 inters and all 3 relations matching. Every registered glyph matches its corresponding official origin/box/orientation/runs.

## Explicit remaining work

The inventory is not a claim that every listed method is complete. Brace detection/creation, bracket detection, line-root checks, C-clef peak rejection, groups/parts, freeze propagation, and general SIG exclusion-aware contextualization remain pending. End geometry is currently computed before the added semantic purges; layouts whose final staff end changes because of those purges need the stage order unified. The two fixtures do not exercise that difference. Staff-area construction, indentation/page allocation and multi-column layouts remain pending. `grid_complete:false` remains mandatory, and no fabricated brace, part, page or semantic relation is emitted.

The next MusicXML-facing consumer can rely on versioned line pixels/geometry, regular-bar subset evidence, rebuilt sections and verified system memberships. It must inspect pending stages and must not treat this IR as a complete musical score.

`scripts/official_final_probe.py` invokes the original `GridStep.doit` inside the official jar after SCALE, then reads in-memory results. It does not reload rounded XML points or feed native intermediate data to Java. Goldens retain the full reference graph, including unmatched braces and actual parts, so missing native scope remains visible.

HEADS-stage integration candidate received from the independent task: local `task-6/audiveris-template-match`, commit `593d1c5d4cddfed4085371ef6a25447151a923d0`. Retain as a later dependency; not integrated into this GRID checkpoint. Its fixed-font template/distance-score core does not itself provide arbitrary-size font rendering, search or mask selection.

The subsequent [brace/part checkpoint](BRACES-PARTS.md) supersedes the earlier missing-brace comparison: all 38 AnLaura final interpretations, 30 relations, 50 glyphs, both 2-part systems and logical page descriptors now match. Its remaining-scope list is authoritative; full GRID is still incomplete.

## Subsequent semantic/mask checkpoint

[GRID-SEMANTICS.md](GRID-SEMANTICS.md) supersedes the earlier pending bracket,
C-clef and endpoint-order statements for its explicitly supported cases. It also
adds the automatic GRID staff mask. Full GRID/HEADS and MusicXML remain incomplete.
