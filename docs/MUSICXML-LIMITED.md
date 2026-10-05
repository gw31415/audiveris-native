> 最終目標はUI表示を除く原版の譜面認識・解析・補正・保存・MusicXML出力機能同等性です。本文の限定範囲は現在の小チェックポイントの受付/検証境界であり、最終スコープ削減ではありません。[全機能台帳と接続順](PORT-STATUS.md)。

# Native image → MusicXML

`audiveris-musicxml IMAGE OUTPUT.musicxml REPORT.json` connects the actual image-owned HEADS/STEMS candidates and caches, reduced SIG, native staff-barlines/chords, isolated SymbolsFilter/classifier and bounded monophonic duration/onset calculation to a native MusicXML 4.0 writer. It does not read original recognition XML or use a Java runtime.

Verified inputs so far are three independent clean opaque PNG pages with Bravura, interline11, zero skew, ordinary five-line staff, treble clef, no key signature, explicit 4/4, one voice, and two/four complete measures, including two systems. Black/void heads produce quarter/half notes; the separate rest page contains quarter rests. Durations, rests, pitches, offsets and voices match original 5.11 XML-loaded HEADS followed by native original MEASURES/CHORDS/SYMBOLS/LINKS/RHYTHMS. This oracle boundary is distinct from original end-to-end transient recognition.

Native quarter, quarter-rest, half-note and four-measure outputs were also checked with MusicXML 4.0 XSD and original proxymusic reimport. A fixed development renderer reconstructed that first input exactly; it is not a general engraver. The CLI refuses missing recognized meter, incomplete/excess measures, ambiguous symbols and existing outputs. It does not guess omitted music.

The batch entry is `audiveris-musicxml --output-dir EXISTING_DIR [--part-name NAME] IMAGE...`. It writes one MusicXML/report pair per input, with preflight collision/no-overwrite checks. A later recognition failure stops the batch and retains individually reported successful pairs; failed writes remove that pair. Part name is metadata for one nontransposing melodic part, not instrument inference or recognition control. Three independent synthetic scores, including four measures and a system break, are verified; arbitrary real-score admission remains unverified. A central quarter-rest configuration refuses weak head optionals instead of emitting an invented note. Half rests are additionally verified at two controlled positions, including a page with provably unreachable orphan beam candidates (details below). PDF, scans/photos, nonzero skew, other fonts/scales, special staffs, full24 shapes, accidentals, shorter beamed/flagged notes, dots/ties/slurs/tuplets, polyphony, pickups, multi-staff or transposing instruments remain outside this bounded path. Complete cleaner/epilog and full reduction parity are pending. All global completion flags remain false.

The accepted full542 regression belongs to source48e6177; changed-source validation is recorded by checkpoint. Existing test reduction audit is deferred until current implementation/verification/necessary fixes finish. No Actions dispatch/rerun is used.


Half-rest continuation (2026-10-05): two controlled positions of the same clean
Bravura score contain a half rest, two quarter notes and two half notes over two
4/4 measures. Original5.11 XML-loaded HEADS onward retains both half rests after
LINKS/RHYTHMS. Position241 produces beam/hook candidates whose eight original
VLinkers reach zero heads. Native admission uses a larger horizontal envelope
than the original lookup areas, including all final BLinker reference points,
head bounds and corner references. Only zero-skew candidates with no stump/seed
and no potentially reachable head are removed by the original orphan checks;
empty beam groups and incident relations become tombstones. Positive beam
linking remains unsupported and a beamed-note page is refused.

Position240 is a negative boundary: original creates a half rest with grade
0.48341279568379303, then removes it at LINKS. Native isolated contextualization
uses the same weak threshold0.5, removes the actual RestInter and its empty chord,
and refuses the now incomplete measure. No rest is fabricated. The canonical
stem caches and physical registry remain owned. For the admitted one-stem/head,
no-live-beam subset, the existing HeadStemsCleaner is called and cached identity
is checked, followed by contextualization as in StemsStep.doEpilog. LINKS has no
lyrics, slurs, tremolos or BeamHead relations in this subset, so their cleaner and
epilog loops are empty; free-glyph deletion is false by original default. This
is not a port of those omitted features or complete STEMS/LINKS epilog.

These are controlled synthetic-page comparisons, separate from the three
independent synthetic scores/24 events already verified. Arbitrary scans,
other fonts, skew, special staffs and full24shape pages remain outside scope.
All global completion flags remain false. Existing test reduction is deferred.


First lower ledger continuation (2026-10-05): one additional independent clean
Bravura input has eight quarter notes (C4 E4 G4 B4 | B4 D5 F5 D5), two complete
4/4 measures, and one concrete C4 ledger. The unmodified original 5.11 image
pipeline through RHYTHMS/PAGE/export emits these eight notes. Native recognizes
the same image without importing original recognition results or XML. This is
one additional bounded image comparison, not full ledger/scan admission.

The port now retains ledgers under the original SIGraph.deleteWeakInters
exemption, erases their canonical horizontal foreground runs under
PageCleaner.visitLedger, and applies Glyphs.buildLinks/GlyphCluster.decompose
for isolated/two-part components. The two C4 residues are evaluated as each
single part and their union, in original order. Original BasicClassifier
returns DOT_set for the union; DotFactory.checkDistanceToConcreteLine rejects
it because it touches a concrete ledger. That kernel comparison uses the native
actual residue runs on the original classes; it is distinct from the original
independent successful full-image MusicXML result. Native uses the already
ported ClefInter diatonic arithmetic and admits C4 only with the owned first
lower ledger. Other ledger pitches, clusters above two parts, compound musical
Inter conflict reduction and genuine dots/augmentation remain explicitly pending.
The central-quarter-rest negative case still refuses; original misrecognizes
that same input, so it is not treated as a native-only defect.

Run the saved input with the current CLI (output paths must not exist):

```sh
audiveris-musicxml fixtures/official-image-musicxml-ledger/ledger-quarter.png ledger-quarter.musicxml ledger-quarter.json
```

The prior full544 regression is evidence for source a2bf13ca8da7afe0b9e41da491190558898f4b34.
This change uses targeted MusicXML/cluster/foundation checks; a new full-workspace
run is not claimed. All global completion flags remain false. Broader ledger
coverage or genuine dot/compound symbol support requires separate scope review.


Compound rest / SIG exclusion continuation (2026-10-05): one saved successful
quarter-rest image has a single one-pixel horizontal ink interruption at the
first rest. No position search or extra font/scale/scan admission was performed.
Original 5.11's unmodified image pipeline through RHYTHMS/PAGE/export still emits
B4 rest E4 G4 | B4 rest F5 D5 (eight quarter events). Native now registers the
actual two-part union, canonical part IDs and RestInter/chord in the retained
registry and SIG, then emits the same eight events from the image.

Original order is SymbolsBuilder.evaluateGlyph (same-glyph exclusions),
ChordsBuilder.buildRestChords, InterFactory.lateChecks, then LinksStep /
SigReducer.reduceLinks: physical overlap detection, contextualization and weak
purge before SIGraph.reduceExclusions. There is no reduceSymbols entry point;
that name in the prior checkpoint ledger was an error, now corrected here.
This bounded implementation covers unsupported-support-free RestInter and
TimeWholeInter competition only, actual intersecting glyph foreground with
minimum bounds IoU 0.05, strongest exclusion first and source retention on equal
grades. Removed rest members empty their actual containing chords while IDs,
incident tombstones, physical registry and head/stem caches remain retained.
Support/frozen/cross-kind conflicts and clusters above two parts still refuse.

On this page the two fragments individually yield no accepted interpretation;
the union is QUARTER_REST. There is one compound rest and no competing surviving
symbol branch. Nonzero conflicts are therefore verified separately with original
SigReducer.detectOverlaps/SIGraph.reduceExclusions explicit-grade controls on
that actual compound and its source part (stronger source, stronger target, tie).
The original phase callback reloads saved SYMBOLS XML (rounded grades) before
calling actual LinksStep; this boundary is distinct from the independent original
whole-image MusicXML export. The source comparison fixture labels both boundaries.
True augmentation dots are not a dependency of this input; their lateDotChecks,
links/duration/export remain unported. General compound-symbol/music support and
arbitrary scans remain unfinished. All global completion flags stay false.


Frozen HEADER key continuation (2026-10-05): the saved G-major input below
is recognized by both the unmodified original 5.11 image pipeline and native
as B4 G4 E4 G4 | B4 D5 F#5 D5. Native imports its own frozen HEADER KeyInter,
actual KeyAlter glyph, Containment and ClefKeyRelation into the owned SIG;
it does not import oracle notes or XML. The 105 `KeyInter.getAlterFor`
controls (15 fifths x seven steps) are kernel evidence, distinct from this
single whole-image comparison. This is not validation of 15 whole-page keys.

Original `SigReducer.checkLedgers` removes a sharp-derived orphan ledger.
Its saved STEMS XML reload and `reduceFoundations` observation are separate
from the full-image export. Native follows the concrete-head/next-ledger
predicate, retains physical glyphs and identity tombstones, and preserves C4.
Shared-ledger/multi-staff reduction is pending. Original XML-reloaded ledger
bounds derive from the glyph and differ from the native candidate envelope;
actual glyph runs, index, unsupported predicate and removal agree.

```sh
cargo build --locked --bin audiveris-musicxml
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-header/g-major-header-quarter.png g-major.musicxml g-major.json
```

Current admission still requires Bravura44pt/interline11, ordinary five-line
G-clef, single staff/voice and complete 4/4 measures. C major examples continue
to work. HEADER TimePair import, other clefs/meters, later SYMBOLS key, cancels,
local accidentals/ties and general key changes remain pending; all original
functions remain the final target. The wide-gap sharp image where original
HEADER has no key is not counted as a successful HEADER comparison.
