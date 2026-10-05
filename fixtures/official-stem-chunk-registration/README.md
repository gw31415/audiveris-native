# Raw chunk registration prefix

Two independent original 5.11 compilations/runs use raw StickFactory chunks
from 612 legitimate first-system Laura builders after StemsRetriever.process.
All 1,364 raw physical glyphs equal the prior saved original actual-builder
lookup controls. Each builder has four registration controls: fresh input,
duplicate canonical existing-part exclusion, all-part exclusion, and repeated
raw alias. There are 2,448 cases / 6,025 raw entries, including 43 empty builders.
Original results are byte-identical after JVM IDs are normalized to canonical
object identity tokens in encounter order.

Original GlyphIndex is legitimately initialized against the actual sheet,
fresh per builder; canonical objects are held strongly throughout the control.
The observer executes the registerOriginal/removeAll prefix from lookupChunks,
not the whole method. Registered IDs, retained order, alias reuse and registry
counts match native results exactly. All excluded chunks stay registered.

The native API accepts raw physical chunks and existing IDs belonging to the
same shared native glyph registry. It registers all raw input before removing
every excluded occurrence; registry groups/identity remain stable. Empty
input, late invalid geometry/identity and work/cardinality budgets are tested.
Invalid-input refusals preserve native registry state; this refusal behavior
is a native boundary contract, not extra original-context coverage. Detached
or foreign-index existing parts are unsupported.

This is the registration prefix with declared raw chunks, not automatic
geometry-to-parts/registry integration. Later head/alignment/stump filters,
directional sorting, parts/items/gap/length expansion, cache/head/SIG composition
and epilog remain unfinished. Whole page shared-index IDs, weak-GC behavior,
nonzero skew, 24-shape full-page propagation, scanned scores and special-staff
full-page parity are not certified. HEADS/STEMS flags remain false; existing
page and explicit collector CLI reports are unchanged.
