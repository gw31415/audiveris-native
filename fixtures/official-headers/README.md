# Official Linux HEADERS snapshots

These 16 JSON files are independent **official Audiveris** snapshots after its
complete HEADERS step. They are not native output and do not assert that the
Rust port has completed HEADERS.

Inputs with the same basename live in `fixtures/official-grid/`. The six
`rendered-header-*` inputs are synthetic score artwork from
`scripts/java/MakeHeaderFixtures.java`, using the pinned official Bravura font.
Existing natural/clef fixtures are unchanged. Exact input/result hashes and
original CLI receipts are retained in `evidence/linux-oracle-validation.json`
and `evidence/linux-oracle/`.

The pinned official JAR is SHA-256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`, release 5.11.0,
source reference `9e1e55cd2746037d059345881c53e6a6754bffbd`.

The observer reads original selected clef/key/time objects, their ranges,
glyphs and the resulting graph. `official_headers_complete` refers only to
the official run, which explicitly verifies the HEADERS step was marked done.
Shape may be null for compound key/time objects; key fifths and time numerator/
denominator are supplied separately. Java IDs are local to each original run.

Reproduce with `python3 scripts/linux_oracle.py --probe headers` or the
explicit six `rendered-header-*` PNG paths. See `docs/LINUX-ORACLE.md`.
