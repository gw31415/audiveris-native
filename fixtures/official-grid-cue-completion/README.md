# Independent synthetic cue page: observed GRID propagation

The independently rendered `official-head-actual-pages/images/Bravura-cue.png`
contains a seven-pixel cue staff and three eleven-pixel ordinary staffs. This is
an independent synthetic page in the original font, not a published score scan
and not one of the 13 comparisons made on the same Laura image.

`cue-final.json` observes unmodified Audiveris 5.11.0 GRID completion, persistent
staff state, physical glyphs, sections, systems/parts and GRID SIG. The derived
observer adds only actual staff scale/small flags. `cue-projection.json` observes
the separate initial StaffProjector stage. Each original observer was compiled
and executed twice, from original raster through original SCALE and GRID,
without native intermediate input; every output byte reproduced. Provenance
pins the original jar, sources, executable/classpath, dependencies and input.
Compile-only ABI shims are excluded from the original runtime classpath.

The native path uses the actual raster/scale/sections/factory, discarded-only
second pass, projection and original main-scale curve sampling. StaffProjector
keeps sheet margins/widths but uses staff-specific bar height, bar/gap thresholds
and chunk threshold. Small staff scale propagates through line completion and
persistent GRID IR; ordinary output bytes remain unchanged. The native snapshot
matches all four projection records, twenty physical lines, 32 glyphs, 85
horizontal / 483 vertical sections, four systems and parts, all observed GRID
SIG interpretations and relations, staff-mask SHA and distance SHA. Test-local
Java numeric IDs are mapped through exact physical glyph/interpretation identity.
The empty/no-second-pass and ratio boundary tests remain in `official-grid-second-pass`.
Unknown/inconsistent staff scales are rejected without partial candidates.

`cargo run --locked --offline --example grid_cue_dump -- INPUT.png NEW_OUTPUT.json`
is a development-only GRID observer. `grid-candidates` now accepts the bounded
five-line main/small layout and produces diagnostics. The page/HEADS CLI advances
through GRID but still refuses mixed specific interlines in KEY; it creates no
success outputs. A further original observation found small CLEF registration
also differs (native missing clef, original clef-stop 119). Do not simply remove
the KEY guard. Candidate classification, specific font bounds versus main-scale
centroid geometry, key preparation and later header phases need separate oracle
comparison before cue HEADS terminal parity (original 168 heads/198 exclusions).

No whole GRID/HEADS/STEMS completion claim, real scan claim, arbitrary special
staff claim, automatic STEMS or full 24-shape page propagation is made. All global
completion flags remain false. Previous kernel controls, same-Laura 13 controls,
three independent ordinary synthetic pages and the Laura default reports remain
separately certified historical evidence.
