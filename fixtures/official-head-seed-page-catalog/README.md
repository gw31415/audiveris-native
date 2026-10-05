Original 5.11 seed catalog propagation controls on the same Laura raster after
unmodified default Bravura GRID, HEADERS, STEM_SEEDS, BEAMS and LEDGERS.

The test observer sets only the original sheet's HEADS MusicFamily parameter and
MusicFontScale after those upstream steps. Twelve controls cover Bravura, Leland,
FinaleJazz at 32/40/51/64 stored points. One extra Bravura51 control changes only
the original first staff's specific interline to 7, with main interline 11 and
staff curves/previous seeds/ledgers retained. Its template sizes are 32 for the
first staff and 51 for the other five. These are controlled catalog/staff-state
experiments, not recognition of pages actually rendered in other fonts or cue
staff sizes. HEADS remains false.

The original Scanner.eval raw and vetoed windows, seed slices, pivots, signed
ledger ordering and distance/mask hashes cover 14764 seed/pitch pairs. The raw
controls temporarily clear the observer's competitor/bar lists, invoke the same
original eval, and restore the lists in finally; diagnostic eval counts include
those extra raw calls and are not production counters. Original output's historic
scope text is retained separately from the explicit controlled scope.

native-before-seeds.json contains previously verified native upstream inputs for
the Rust test, with the original CLI SHA256 and source commit. The independent
Java oracle never reads that file or any native output. provenance.json records
all 13 successful original invocations, observer and JAR pins. A fresh source-
backed invocation via linux_oracle.py reproduced every cue control field.

    python scripts/linux_oracle.py --oracle-home /workspace/onboarding/linux-oracle \
      --probe head-seed-page-catalog --head-family Leland --head-point-size 40 \
      --fixtures laura-gray --output NEW_DIRECTORY

For the cue control use Bravura, 51 and --head-cue-first-staff. No bootstrap or new
runtime dependencies. Family/size/cue options are accepted only by the two controlled page catalog probes.

The opt-in native analyze_catalog seed API preflights the exact oval assets for
each staff before preparing pixels. It carries family/stored point metadata and
the required-metadata marker through signed ledgers and seed veto. Stale point
bindings, unknown metadata fields, absent sizes and dropped required metadata
fail explicitly. The legacy default analyze entry point retains its old certified
scope. Untouched CLI reports remain byte-identical. The subsequent official-head-page-catalog stage propagates this contract through
range/aggregate/live conflicts/glyph/HeadInter/SIG and epilog. They still do not
declare complete HEADS.

Pending: full other-font, actual cue/one-line/drum/tab page upstream and HEADS verification;
15 unmatched small/hollow/nonoval templates and custom drum/template overrides;
then stem attachment, rhythm and MusicXML.
