# Native header time candidates

Pinned source: Audiveris 5.11.0, `9e1e55cd2746037d059345881c53e6a6754bffbd`.

`time_preparation` follows HeaderTimeColumn.allocateBuilder and HeaderTimeBuilder's fresh automatic search: the incoming header stop already includes key-column propagation, without adding one. It preserves the repeated-stop bottom-line calculation, sheet-clamped ROI, integer projection, all whitespace runs (including terminal runs), inclusive whitespace range edges, and integer half-height cropping. Sheet and staff scale parameters remain separate.

`time_candidates` extracts actual NO_STAFF components, filters only minimum weight, and follows stable reverse-weight seed/neighbor traversal. Whole adapters stop at the first non-whole ranked evaluation; half adapters continue through the top three and require original centroid pitch magnitude 1–3. Better-per-shape replacement is strictly greater, preserving ties. The native pinned neural classifier and feature extractor run without Java.

`time_registration` uses a hash-checked generic Bravura geometry bank, 21 shapes × 128 interlines. Single glyphs use rounded layout half-dimensions; composite numerator/denominator symbols use original integer dimensions and integer division. These are reusable font metrics, not recognition answers.

The candidate API is a stage boundary, not completed time recognition: it does not yet pair/filter candidates or select a system-wide time. It returns explicit incomplete flags. Existing interactive timeRange overrides and tablature are outside this fresh five-line path. The 4-million-pixel limit remains. More than eight retained time parts is explicitly unsupported rather than silently truncating the original search.

Tests compare original constructor snapshots and reached before-filter stages. A system may abort after its first failing staff; later unvisited builders have only constructor evidence. The integration comparison computes post-key boundaries through native key-column propagation and independently checks them against the original before entering the time stage. CLI per-staff candidate diagnostics intentionally visit every staff; system early-abort/selection remains a separate stage.

Source hashes are in `evidence/time-candidates-source.json`. Oracle execution, renderers and provenance remain development-only, through the Linux harness.
