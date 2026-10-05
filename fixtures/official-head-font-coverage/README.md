# Exact font / point-size HEADS kernel coverage

Unmodified official Audiveris 5.11.0 (9e1e55cd2746037d059345881c53e6a6754bffbd),
existing Linux release runtime. Observers read no Rust output. Jar, individual
font resources, runtime/compiler and observers are hash recorded in provenance.
`HeadFontBankOracle` exports exact original TemplateFactory keypoints/bounds and
anchors. `HeadFontCoverageProbe` calls original Scanner evaluation/hole conversion,
weak-stemless and constructor grade gates, followed by HeadInter.retrieveGlyph.

Bravura, Leland and FinaleJazz; exact 32, 40, 51 and 64pt; BREVE, WHOLE_NOTE,
NOTEHEAD_VOID and NOTEHEAD_BLACK. There are 192 binary PNG renders: clean, erased
right half, clipped upper-left edge and empty. Both black-spot relevant and hollow
range modes yield 384 comparisons; physical foreground glyphs yield 192. Central
veto and unavailable family/shape/size errors remain explicit. No interpolation or
fallback font is used. Selection of one catalog must be carried from matching to
retrieval by its caller; these APIs do not infer fonts from pixels.

The historical builtin Bravura51 bank and its calibration/assets remain unchanged.
The additional catalogs are independently hash-pinned, bounded and selected through
Bank::load_coverage. Other font_version labels identify the resource bundled in
5.11.0, not a guessed upstream font release version. Production Rust calls no Java,
font rasterizer, network or AI provider.

This is **kernel coverage**, not full page parity for these fonts/sizes. The CLI
orchestration still admits the certified Bravura51 ordinary five-line/ledger path.
Special staff scheduling/drum catalogs, cue-page state/size propagation, and full
pages for other fonts require further original-oracle verification. HEADS completion
stays false. Laura's full shared SIG, staff retirement and earlier reports are
checked separately. Reproduce using the existing Linux dependencies only:

    python3 scripts/build_head_font_coverage.py --oracle-home EXISTING_ORACLE --output NEW_DIRECTORY
    python3 scripts/linux_oracle.py --oracle-home EXISTING_ORACLE --probe head-font-coverage --fixtures laura-gray --output NEW_DIRECTORY
