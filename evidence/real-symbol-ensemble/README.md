# Actual Chord ensemble visits during SYMBOLS cleaning

Original SymbolsCleaner partitions live SIG vertices, keeping frozen entries
before ensemble membership checks. Nonfrozen ensemble members are visited via
their parent, except beams. Strong parents are visited before weak parents; an
actual Chord visits its notes in stable bottom-up order. BeamGroup has no raster
visitor operation. Head and Stem thresholds are0.6 and0.7; other supported
entries use original good grade0.4.

Native now reads the actual live Containment edges and Chord member geometry.
It does not recover ensembles from report arrays. The unchanged full image has
67 strong HeadChords containing94 Heads, including one individual Head below
0.6. That Head is erased through its strong parent, without optional glyphs,
matching the original observation. All94 direct Head strength decisions and
all67 actual member orders match; the graph and retained owners remain intact.

An exploratory exact best-grade assertion failed. Grade numerical equality is
not claimed by this cleaner contract; numerical residuals are recorded apart
from the matching strength decisions and remain a recognition-quality dependency.
Weak standalone optional glyph collection is still explicit pending work when
an input requires it. The source full-cleaned-mask hash is retained, but this
unit does not certify equality of the entire native cleaner mask or real-score
MusicXML. Synthetic XML regressions remain separate from quality acceptance.
