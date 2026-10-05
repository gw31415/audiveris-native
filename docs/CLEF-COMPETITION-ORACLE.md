# Constructed original clef-competition oracle

`scripts/java/ClefCompetitionControlProbe.java` exercises the unmodified original
`Staff.getCompetingClefs(int)` and `SIGraph.getExclusions(Inter)` on a constructed
original graph. It is development-only numerical/ordering evidence, not raster
recognition or end-to-end HEADERS parity.

## Construction

The observer uses the original no-argument `SIGraph` constructor and its actual
original `DirectedMultigraph` delegate. Original `ClefInter` and `KeyAlterInter`
constructors create detached vertices with explicit staff identity, bounds, and
IDs. Development-only `Unsafe.allocateInstance` creates incomplete `Staff` and
`SystemInfo` holders; their links and IDs are populated reflectively. Vertices and
exclusions are inserted directly into the original graph delegate, avoiding
unrelated sheet-index updates and inter-added callbacks. Retrieval, sorting,
incident-edge ordering, filtering and deduplication all run through original JAR
code, without replaying the selection algorithm.

Every attempted edge insertion records the original result in `edge_insertions`.
Only successful insertions appear in `effective_edges`, which is the graph input
for native comparison. Original `DirectedMultigraph` allows parallel edges but
rejects self-loops with `IllegalArgumentException: loops not allowed`. Rejected
attempts must not be treated as original graph edges.

## Fixture and semantics observed

`fixtures/clef-competition/official-controlled.json` contains 76 cases, including
60 deterministic varied graphs. Each case records insertion-ordered nodes and
edges, target staff ID, query x, original returned `expected_ids`, and incident
exclusion ordering at the selected anchor.

The controlled cases directly demonstrate:

- Stable bounds-left sorting and the last inserted vertex winning equal-left
  anchor ties
- Strict `left < x` for the anchor, including x exactly equal to tied left values
- Incident exclusions traversed incoming first, then outgoing, preserving each
  direction's insertion order despite globally interleaved edge insertion
- Parallel and reverse exclusions deduplicated by opposite vertex
- Foreign-staff and non-clef exclusion partners omitted
- Exclusion partners at or beyond x retained: x filtering applies only to the
  anchor
- Empty graph, only-foreign-clef graph, and no-left-anchor empty results

Nodes carry `is_clef`. A native helper whose inputs are specifically clef nodes
must filter out `false` entries while preserving the other nodes' relative
insertion order; edges to absent non-clef IDs are ignored as non-clef partners.

## Reproduction

```
python3 scripts/linux_oracle.py --probe clef-competition-control --output /tmp/new-clef-competition-evidence
```

The runner pins Audiveris 5.11.0 JAR SHA-256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`, source commit
`9e1e55cd2746037d059345881c53e6a6754bffbd`, the official Linux distribution and
compiler. It saves the exact compiled observer source snapshots and dependency
hashes, with compile-only ABI shims excluded from runtime classpaths.

- Observer: `scripts/java/ClefCompetitionControlProbe.java`
- Fixture: `fixtures/clef-competition/official-controlled.json`
- Compact provenance: `evidence/clef-competition-oracle-provenance.json`
- Raw receipt and source snapshots:
  `evidence/linux-oracle/clef-competition-control-20261002/`

Unlike the key-clef scoring probe's separately labelled method-local
`bestCompatibleClef` replay, this competition fixture's returned ordering is
entirely a direct original-method observation.
