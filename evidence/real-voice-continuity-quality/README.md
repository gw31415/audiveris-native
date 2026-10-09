# Across-measure single-stream quality observation

This adds a deliberately limited across-bar musical comparison to the independent structure comparator. It compares the last complete chord/rest group of one measure with the first complete group of the immediately following measure on the same staff, only when both measures have one nonoverlapping local stream in both scores. Voice names may change freely at the barline. No reused-number or pitch-distance heuristic supplies a voice identity.

The two existing independent MuseScore pages have one voice per staff/measure, zero across-measure slurs and zero ties. The original XML has no across-measure slurs or ties either. Consequently these inputs cannot establish general polyphonic voice identity across a boundary. Ambiguous boundaries remain explicitly unscored; they are not silently counted as correct or incorrect continuity. The previous within-measure and full event categories continue to expose their notes and voice partitions.

| Page | Staff boundaries | Jointly scored | Exact boundary pairs | Unscored |
|---|---:|---:|---:|---:|
| AnLaura | 9 | 4 | 3 | 5 |
| Der Winter | 24 | 24 | 19 | 0 |

AnLaura's unscored boundaries are staff 2 measures 1→2, 2→3, 3→4 and staff 3 measures 1→2, 2→3: at least one original XML measure contains multiple local voices. The one scored mismatch is staff 1 measures 3→4, where the last note's recognized MIDI pitch is 70 versus independent 68. Der Winter's five mismatches are all staff 3: measures 3→4 has the prior endpoint at onset 1/2 rather than 3/8; measures 5→6, 6→7, 7→8 and 8→9 retain the known pitch errors. These endpoint mismatches do not by themselves mean that voice identity changed.

`across_measure_voice_coverage.observations` retains every boundary, local stream/group counts, exclusion reasons, available endpoints and whether it was scored. Empty measures are not bridged. A single reused voice label containing overlapping notes is excluded. Both-empty metrics have null precision/recall/F1 and `both_empty_not_exercised` qualification.

18 Python contracts passed, including 5 new tests for independent per-measure renumbering, polyphonic label reuse without proof, overlapping groups under one label, empty-measure gaps, and wrong pitch versus a retained rest endpoint. The original 15 comparison categories are byte-equivalent as JSON values on both pages. Tests also assert real coverage, available relation counts and observed mismatches.

`RESULT.json` records exact commands, exit status, source/script hashes and metrics. No Rust/Git writes, image recognition reruns, external network or environment changes were used. New data derives solely from existing truth scores and original XML. `quality_accepted` remains false.
