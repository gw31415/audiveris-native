# Original StemBuilder section lookup and raw assembly

Scope: one actual Laura first system after original 5.11 HEADS and legitimate
StemsRetriever.process initialization. The observer reuses the 612 real builders
and temporarily changes selected distance/head-stop limits, restoring them in
finally blocks. It supplies their original lookup polygons, stumps and theoretical
lines to Rust. It does not certify native Head/Beam lookup-area construction,
other systems/pages, nonzero skew, complete automatic parts or final STEMS/SIG.

Two independent saved original compilations/runs have byte-identical results:
612 actual controls, 565 distances times nextDown/equal/nextUp, and 12 head stops
times nextDown/equal/nextUp, totaling 2343 controls on 1390 vertical and 1645
horizontal sections. Native section centroids, selected IDs/order and all 4459
raw physical glyph entries match. Actual controls contain 1364 raw glyphs.
These are separate from the 13 same-Laura font/size controls, three independent
synthetic font pages and published real scans (still unverified).

The observer captures selected horizontal IDs before StickFactory mutates its
input list ordering. It compares both original minCoreLength=0 and =1; every
positive-run case has identical raw physical glyphs. Rust reuses the certified
factory with minCoreLength=1. This is not chunk filtering or GlyphIndex/SIG
registration. The API preserves caller inputs and per-orientation section IDs.

Boundary policy: finite bounded input, zero skew and one linear polygon only;
Java int-overflow section centroids, undefined/single-pixel factory fits, curves,
multiple polygon rings and broader context/registration remain unsupported.
The native helper validates every original section centroid, while two native
tests cover original controls, empty input and invalid/overflow/identity refusal.
HEADS/STEMS completion remains false.
