Exact seven-shape seed matching windows from the original Audiveris 5.11 Linux JAR.

Bravura, Leland and FinaleJazz at 32/40/51/64 points; VOID, BLACK, BLACK_SMALL,
CROSS, DIAMOND_FILLED, TRIANGLE_DOWN_FILLED and CIRCLE_X. The observer independently
renders 336 clean/half-erased/clipped/empty images, all byte-identical to the
already pinned official font/drum fixtures. Those existing images are referenced
by relative path rather than duplicated. Both LEFT_STEM and RIGHT_STEM windows
use original Scanner.eval, strict location improvement, central abandonment,
BLACK-only hole reassignment, and actual original frozen-bar Area intersection.
There are 2016 windows, including central and noncentral veto controls.

The original JAR/upstream/font pins and observer source snapshots are recorded in
provenance.json. Reproduce offline using the existing Linux oracle, with no
bootstrap, download or provider:

    python scripts/linux_oracle.py --oracle-home /workspace/onboarding/linux-oracle \
      --probe head-seed-coverage --fixtures laura-gray --output NEW_DIRECTORY

The native search_selected_with_veto API preflights the exact declared stem shape
and asset set before veto side effects; duplicate/unported/stemless selections
fail explicitly, empty selections are no-ops with valid search windows, and veto
errors propagate. Caller shape order is retained. The legacy seed search APIs,
ordinary builtin51 catalog and full Laura reports are unchanged.

This fixture certifies matching kernels, not the full lookupSeeds method. Physical
seed slicing/pivot selection, live staff catalog propagation, glyph registration,
epilog, additional hollow/small variants and other-font/special-staff full pages
still require integration verification. heads_complete remains false.
