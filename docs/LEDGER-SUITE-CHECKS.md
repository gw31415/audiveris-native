# Native ledger suite checks

Ports the original `LedgersBuilder.LedgerSuite`, `Check` and `CheckSuite` rules
at Audiveris `9e1e55cd2746037d059345881c53e6a6754bffbd`, from the authorized
JAR SHA-256 `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.

`ledger_checks::check` accepts a native filament description, the NO_STAFF
raster, main/specific interlines, foreground thickness, target ordinate and
the original wide-previous-ledger condition. It evaluates the seven original
checks in order: minimum thickness, maximum thickness, minimum length,
convexity of both ends, straightness, and left/right target pitch offsets.
Length/pitch use the staff-specific interline; minimum thickness/straightness
use the large interline and maximum thickness uses foreground line thickness.
Wide previous ledgers change length thresholds from 1/1.5 to 1.4/2 interlines.

Weights are 0.5, 0, 4, 2, 1, 0.5, 0.5. A zero maximum-thickness impact still
vetoes the grade despite its zero weight. Other positive weighted impacts
form the original weighted geometric mean multiplied by intrinsic ratio 0.8.
The original minimum grade is 0.1 times 0.8, with inclusive acceptance at the
threshold. Native checking reuses the already-verified stem Check interpolation
without changing its behavior.

The independent observer constructs original LedgersBuilder/LedgerSuite and
StickContext on a real GRID sheet; original StraightFilaments use declared
horizontal sections. The NO_STAFF raster is explicitly white except the listed
black neighbors above zero, one or both ends. Target ordinate and the previous
ledger width condition are declared inputs. No native outputs drive the oracle.
The 1,728 controls cover four interlines (including odd rounding), both length
threshold sets, core/minimum-length boundaries, foreground thickness checks,
convex/concave ends and pitch-margin values just below, at and immediately beyond
0.35 interline. All seven raw values, thresholds, weights, impacts, final grades
and decisions match: 302 accepted and 1,426 rejected, including 576 thickness
veto controls. Original candidate controls and 356 original stem-check contexts
remain passing.

An initial expanded observer included a rectangle taller than its width, so
the original fit became vertical and rejected horizontal endpoint access. The
observer now explicitly supplies horizontal filaments; the failed observer log
and stale-fixture count failure are retained in checkpoint evidence.

This stage supplies the original acceptance calculation; it does not register
LedgerInters. Native GRID ledger section dispatch, good full beam selection,
signed staff-index/reference lookup, original lookupLine scan and reduction,
shared graph/glyph registration and staff ledger maps remain pending. No
positive real-score rest conversion or full OMR/MusicXML completion is claimed.
Full suites are unverified at this source; the prior 321 debug/321 release
baseline on `7bc4213cea7ed27ee8355e319447dda3b9b7074f` is retained.
