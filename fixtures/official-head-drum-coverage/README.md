# Matched quarter-head range / physical glyph / SIG contracts

Original pinned Audiveris 5.11.0 on the existing Linux runtime; no Rust output is
read by observers. New quarter shapes are NOTEHEAD_BLACK_SMALL, NOTEHEAD_CROSS,
NOTEHEAD_DIAMOND_FILLED, NOTEHEAD_TRIANGLE_DOWN_FILLED and NOTEHEAD_CIRCLE_X.
Three fonts (Bravura/Leland/FinaleJazz), exact 32/40/51/64pt, four variants
(clean, right-half-erased, upper-left-clipped, empty) provide 240 binary images.
Original Scanner.eval/constructor grade gates give 480 explicitly selected range
comparisons (including empty hollow selection), and HeadInter.retrieveGlyph gives
240 physical foreground glyph comparisons. Cross boost uses the original default
0.0. Custom constant changes are outside this pinned catalog.

For every render, original HeadInter.isSameAs/overlaps are called on a same-bounds
head with a null-glyph twin. Original SIG.addVertex adds both to the same real
system graph and staff notes in insertion order; remove retires both and restores
staff-note and graph counts. Native tests check these shape/geometry contracts and
shared Graph insertion order, preservation of prior GRID vertices/relations and
HEAD tombstones. These are constructor/lifecycle controls, not complete drum page
recognition or the final staff-purge/beam/tally loop on a special-staff image.

A separate exact, bounded, hash-pinned bank contains all nine matched shapes
(ordinary four plus these five) at the declared sizes: 108 assets. Missing size,
family and unported head shapes fail explicitly. The historical builtin and first
ordinary coverage banks remain byte-identical; duplicated ordinary templates are
also compared as keypoints/bounds. No synthesized raster or size interpolation is
used. Original small-head TemplateSymbol scales its layout internally: direct
layoutShapeByCode may be null, and the exporter records that auxiliary field as
null while retaining the actual original generated bounds/keypoints.

Native candidates_for_shapes requires the caller's original enum-ordered staff
catalog. Empty selection does not create candidates; duplicate and unported shape
sets fail before veto calls. Existing ALL_SHAPES remains the ordinary four so
Laura does not change. The generic HeadInter same/overlap/purge validator now
admits these five exact shapes. Full page orchestration remains the certified
Bravura51 ordinary five-line/ledger path; size and font propagation through seed
veto and physical registration and upstream special staff detection are unfinished.
Original processing switches can select 24 ordinary heads; the 15 unmatched
small/hollow nonoval variants are not covered by this nine-shape bank. Custom drum
mapping and other fonts/sizes also remain outside coverage. HEADS stays false.

Reproduce with existing dependencies, without bootstrap:

    python3 scripts/build_head_drum_coverage.py --oracle-home EXISTING_ORACLE --output NEW_DIRECTORY
    python3 scripts/linux_oracle.py --oracle-home EXISTING_ORACLE --probe head-drum-coverage --fixtures laura-gray --output NEW_DIRECTORY
