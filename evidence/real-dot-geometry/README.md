# Dot geometry on retained curved Staff and concrete Ledger maps

Original DotFactory checks the glyph center2D against actual Staff.pitchPositionOf.
Within five-line Staff height it compares the nearest even pitch; outside, it
chooses a concrete Ledger from that Staff map using strict search-box intersection
and integer center ties, then measures center2D distance. Null concrete Ledger
means sufficient distance. Retained removed aliases cannot be silently discarded
by a live-SIG scan. The threshold is0.3 of the Staff-specific interline.

Native uses the held System/Staff/area/virtual-curve/map objects. The real candidate
System is recorded at construction, instead of guessing across all report Staffs.
Actual PartLayout.measure_at dispatch also retains the original MeasureLayout Arc.
A missing actual Staff/Measure is explicit; geometry cannot manufacture ownership.
Dot additions and late checks remain staged without clearing existing clocks.

The original788 real compound candidates yield736 accepted and52 rejected by the
geometry check. These are hypothetical-dot geometry checks of all candidates,
not788 recognized dots. Three declared glyph shifts near each of15 actual Ledgers
provide45 positive concrete-ledger lookups (15 accepted distance checks). Twelve
actual Staff/Part Measure dispatch controls use the independently observed shared
Stack fixture. This does not claim complete instant/late DotFactory relationship
order, cross-measure/shared supports or PageRhythm.filter completion.

The source candidate order changes with Java HashSet identity; comparisons use
canonical glyph JSON SHA256 and candidate membership. The compact fixture derives
trial fingerprints from sorted-key JSON with compact separators, retaining bounds,
weight, Staff, pitch, selected Ledger and distance result. Raw observer output SHA
and compact derived SHA are both preserved in provenance. No source observations
are inputs to the production native CLI.

Focused tests and63+correction8/exact7/64 XML bytes/XSD are recorded in VALIDATION.
Actual native real-image MusicXML and quality acceptance remain open.
