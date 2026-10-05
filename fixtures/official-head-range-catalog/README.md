# Original 5.11 default range catalog controls

`laura.json` observes unmodified Audiveris 5.11.0, commit
`9e1e55cd2746037d059345881c53e6a6754bffbd`, on Linux after fresh
GRID/HEADERS/STEM_SEEDS/BEAMS/LEDGERS and HeadSpotsBuilder. The original
scanner exposes BREVE, WHOLE_NOTE, NOTEHEAD_VOID, NOTEHEAD_BLACK in enum
order. Without a relevant black spot, only NOTEHEAD_BLACK is omitted.
All 118 scanners have 2,631 scored proposals: 2,224 pass the range gates
(including three whole-note candidates), and 407 are weak stemless proposals
(59 breve, 348 whole). These are observations before aggregateMatches,
filterSeedConflicts, glyph retrieval, or SIG insertion.

`controls.json` and the PNG files are independent original TemplateSymbol
renders at Bravura 51pt, thresholded using the original template builder's
alpha/red threshold of 140. Placement uses original Template.getOffset,
including its left anchor rounding. Clean, half-erased, clipped-edge and
empty images are evaluated by original ChamferDistance and Scanner.eval
with explicitly empty bar/competitor lists. Both black-relevance states
are exercised. Native outputs are never supplied to the observer.

Score-only controls invoke original isWeakStemLessHead and createInter
(the latter constructs an unregistered temporary HeadInter). Score bits
preserve adjacent f64 values around the stemless and minimum-inter grade
boundaries. The original impact container clamps negative impacts to zero.
The default stemless boost is zero; minimum contextual grade is 0.5 and
minimum inter grade is 0.8 * 0.1. Equal grades are accepted, subject to
actual floating point rounding. Constructor eligibility controls beyond
the range score ceiling (0.4) test that gate independently.

The native `head_range_catalog` report preserves source shape, selected
shape, source score, intrinsic grade, slim bounds and scanner staff/pitch.
`matches` and `rejected` retain traversal order within each collection.
Eligibility metadata describes a future staff boost without applying it
to the intrinsic grade twice. No glyph ID, free glyph, HeadInter or SIG
vertex is allocated. The existing `head_range_search` black/void prefix
remains unchanged. Aggregation and seed conflicts must precede retrieval
and registration in later stages; `range_search_complete` and
`heads_complete` remain false.

Regenerate with the existing pinned Linux oracle:

```
python scripts/linux_oracle.py --oracle-home ORACLE_HOME \
  --probe head-range-catalog --fixtures laura-gray --output NEW_DIRECTORY
```

Use `tests/head_range_catalog.rs` and `scripts/check_head_range_catalog.py`
to compare the native implementation. Small, cross, drum, other fonts and
other point sizes remain unsupported.
