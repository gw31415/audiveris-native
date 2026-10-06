# Original head-cleaner lifecycle controls

Audiveris5.11.0 source `9e1e55cd2746037d059345881c53e6a6754bffbd`, jar SHA256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.
`scripts/java/HeadCleanerLifecycleProbe.java` extends the existing original
foundation observer and loads the same saved book for System/Staff. It clears
SIG and declares physical heads/stems and relations. No new image fixture.

Six successful controls compare first typed edge lookup, first/second selected
cleaner traversal or automatic StemsRetriever traversal, kept relation indexes,
endpoint abnormal/removed callbacks and physical glyph retention:

- Manual/frozen three stems, selected and automatic: only relation1 remains.
- Parallel high-ratio, selected distinct stems: `[1,2]` then `[2]` remain.
- Parallel low-ratio, selected distinct stems: `[1,2]` then `[1]` remain.
- Manual/frozen small canonical share, selected and automatic: both remain.

The original selects the first typed parallel relation, irrespective of grade.
Repeated selected cleanup is not necessarily idempotent. Manual/frozen flags do
not exempt these edge removals. Vertices and glyphs are not deleted.

Original automatic high-ratio parallel input threw `IllegalArgumentException:
no such edge in graph: 1.00~HeadStem` after duplicated stem references aliased a
removed edge. Observed once; not retried. Automatic low-ratio was not observed.
Both automatic parallel inputs remain excluded from successful controls; native
retains its automatic parallel refusal. See the small
[original limitation receipt](../../evidence/musicxml-head-cleaner-lifecycle/ORIGINAL-PARALLEL-LIMIT.json).

The actual-image owner test adds declared alternative edges to existing cached
stems. It validates retained cache/registry identities and atomic refusal, not
an automatically recognized multi-stem image or full original image-SIG parity.
Original800 decision controls and general-overlap image regressions stay separate.
Full cleaner/epilog and full nonUI parity remain incomplete.
