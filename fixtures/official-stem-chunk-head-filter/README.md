# Original HEAD chunk pixel filter

Two independent original 5.11 runs invoke filterHeadParts on 600 legitimate
Laura first-system Head CLinker builders. 6,600 controls compare actual chunks,
VIP=false/true with remaining weight 14/15/16, duplicate canonical entries,
the final physical head row, the following nonoverlapping row and empty input.
Original results are byte-identical with JVM object IDs normalized to canonical
tokens. Native remaining weights and retained token orders match exactly.

The official method removes a chunk only when remaining weight is less than
15 AND the head is VIP. This unusual branch is preserved. Physical glyph
pixels/holes matter; later bbox-based GlyphItem filtering is a separate step.
GeoUtil.yOverlap counts one shared pixel row as positive overlap. Initial
boundary-test failure and its correction are recorded in checkpoint evidence;
the corrected observer calls original GeoUtil directly for diagnostics.

Native membership searches validated sorted head runs instead of allocating
all head pixels. Registry, cache, head and SIG state remain unchanged, including
candidate IDs removed from the returned list. Invalid identity/glyph and
cardinality budgets refuse explicitly. These native refusal tests are separate
from the original context controls.

Full lookupChunks registration/head/alignment/stump/sort composition, automatic
head/beam target/neighborhood selection, parts/items/gaps/length maps,
cache/head/SIG integration and epilog are incomplete. These are controlled
Laura comparisons, not independent scanned or special-staff page validation.
Nonzero skew and 24-shape full-page propagation remain unverified/unsupported
as previously recorded. HEADS/STEMS completion flags remain false; the existing
page and collector CLI reports are preserved.
