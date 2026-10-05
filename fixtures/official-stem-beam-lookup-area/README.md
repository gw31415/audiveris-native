# Beam lookup-area controls

Two independent original 5.11 compilations/runs invoke VLinker.buildLuArea on
12 legitimate Laura first-system beam builders after StemsRetriever.process.
72 controls cover both directions, declared surrounding part/system edges,
sloped limits and below/equal/above/zero gap ratios. The 72 original records
are byte-identical. Native coordinates, nonzero adjacent edges and theoretical
lines match exactly. Ring start/winding and exactly repeated consecutive
vertices (Java Area zero-length edges) are normalized; no coordinate rounding.

Native inputs declare the reference, beam border, surrounding parts and
profile-converted gap pixels. Part bottoms retain y+height-1; the initial
system maximum retains y+height. Native invalid/skewed, nonconvex and excessive
inputs refuse without mutation. The empty-neighborhood native boundary test
is separate from actual original-context comparison, not additional oracle
coverage.

This is a controlled comparison on Laura, not independent real-scan or
special-staff page validation. Automatic neighborhood selection, closer-limit
selection, reference/target discovery, chunk registry/filter composition,
item expansion, cache/head/SIG composition and epilog remain incomplete.
HEADS/STEMS flags remain false. The explicit-input CLI is unchanged; these
geometry APIs have not yet been composed into automatic page STEMS.

The actual observed beam profile is 1 and its converted gap is 13 pixels.
Other profile/size dependency kernels remain historical separate evidence;
this observer does not certify other profile/size whole-page behavior.
