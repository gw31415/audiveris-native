Original Audiveris 5.11 complete HEADS controls on the same Laura raster.

After unmodified original default GRID/HEADERS/STEM_SEEDS/BEAMS/LEDGERS,
HeadsCatalogPageProbe changes only HEADS family/stored point size and optionally
first-staff specific interline7/main11. It then executes original HeadsStep
prolog, NoteHeadsBuilder.buildHeads and HeadsStep epilog. Twelve controls cover
Bravura/Leland/FinaleJazz × 32/40/51/64; the thirteenth uses Bravura51 with the
first staff derived point32. These do not certify pages actually rendered in
other fonts, cue sizes, drums, one-line staffs or tablature. HEADS remains false.

The independent Java observer never reads native outputs. Original fixture
hashes, invocation arguments, observer/JAR pins are in cases/provenance. A fresh
source-backed original cue invocation reproduced every observer result field.

    python scripts/linux_oracle.py --oracle-home /workspace/onboarding/linux-oracle \
      --probe head-page-catalog --head-family Bravura --head-point-size 51 \
      --head-cue-first-staff --fixtures laura-gray --output NEW_DIRECTORY

The native development control configures HEADS only after default upstream:

    audiveris-native heads-catalog-control INPUT OUTPUT.pgm REPORT.json Leland 51

The optional final `cue` requests the controlled first-staff interline7. Exact
assets must exist for every derived point size. Unsupported catalogs, dropped
required metadata, stale staff sizes and inconsistent dependencies fail before
physical glyph or shared SIG transactions commit. The existing page-glyphs
command and legacy Bravura51 API retain byte-identical Laura output.

native-controls/*.json.gz are explicitly native CLI outputs, separate from the
original oracle fixtures. native-comparisons.json binds their uncompressed
hashes. check_head_page_catalog rechecks ordered physical heads/glyphs, shape,
bounds, staff/pitch, grades/DX, ordered exclusions/staff notes, small-beam
contextualization, seed epilog and preservation of prior shared identities and
relations. Intermediate original performance or duplicate-removal counters are
not observed by this probe and are not claimed as compared.

Pending: actual other-font/cue/special-staff full-page upstream and HEADS parity;
15 additional small/hollow/nonoval templates, custom drum/template overrides,
undeclared fonts/point sizes; then stem attachment, rhythm and MusicXML.
