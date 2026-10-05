> 最終目標はUI表示を除く原版の譜面認識・解析・補正・保存・MusicXML出力機能同等性です。本文の限定範囲は現在の小チェックポイントの受付/検証境界であり、最終スコープ削減ではありません。[全機能台帳と接続順](PORT-STATUS.md)。

# Native image → MusicXML

`audiveris-musicxml IMAGE OUTPUT.musicxml REPORT.json` connects the actual image-owned HEADS/STEMS candidates and caches, reduced SIG, native staff-barlines/chords, isolated SymbolsFilter/classifier and bounded monophonic duration/onset calculation to a native MusicXML 4.0 writer. It does not read original recognition XML or use a Java runtime.

Verified inputs so far are three independent clean opaque PNG pages with Bravura, interline11, zero skew, ordinary five-line staff, treble clef, no key signature, explicit 4/4, one voice, and two/four complete measures, including two systems. Black/void heads produce quarter/half notes; the separate rest page contains quarter rests. Durations, rests, pitches, offsets and voices match original 5.11 XML-loaded HEADS followed by native original MEASURES/CHORDS/SYMBOLS/LINKS/RHYTHMS. This oracle boundary is distinct from original end-to-end transient recognition.

Native quarter, quarter-rest, half-note and four-measure outputs were also checked with MusicXML 4.0 XSD and original proxymusic reimport. A fixed development renderer reconstructed that first input exactly; it is not a general engraver. The CLI refuses missing recognized meter, incomplete/excess measures, ambiguous symbols and existing outputs. It does not guess omitted music.

The batch entry is `audiveris-musicxml --output-dir EXISTING_DIR [--part-name NAME] IMAGE...`. It writes one MusicXML/report pair per input, with preflight collision/no-overwrite checks. A later recognition failure stops the batch and retains individually reported successful pairs; failed writes remove that pair. Part name is metadata for one nontransposing melodic part, not instrument inference or recognition control. Three independent synthetic scores, including four measures and a system break, are verified; arbitrary real-score admission remains unverified. A central quarter-rest configuration refuses weak head optionals instead of emitting an invented note. Half rests are additionally verified at two controlled positions, including a page with provably unreachable orphan beam candidates (details below). PDF, scans/photos, nonzero skew, other fonts/scales, special staffs, full24 shapes, accidentals, general beamed/flagged short notes, dots/ties/slurs/tuplets, polyphony, pickups, multi-staff or transposing instruments remain outside this bounded path. Complete cleaner/epilog and full reduction parity are pending. All global completion flags remain false.

The accepted full542 regression belongs to source48e6177; changed-source validation is recorded by checkpoint. Existing test reduction audit is deferred until current implementation/verification/necessary fixes finish. No Actions dispatch/rerun is used.


Half-rest continuation (2026-10-05): two controlled positions of the same clean
Bravura score contain a half rest, two quarter notes and two half notes over two
4/4 measures. Original5.11 XML-loaded HEADS onward retains both half rests after
LINKS/RHYTHMS. Position241 produces beam/hook candidates whose eight original
VLinkers reach zero heads. Native admission uses a larger horizontal envelope
than the original lookup areas, including all final BLinker reference points,
head bounds and corner references. Only zero-skew candidates with no stump/seed
and no potentially reachable head are removed by the original orphan checks;
empty beam groups and incident relations become tombstones. That orphan removal remains separate from the subsequently connected positive
single-beam path below. The old beamed-note negative has only three quarter
beats in its first4/4 measure and still refuses after correct eighth recognition.

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


Frozen HEADER TimePair continuation: one controlled variant moves the two time
numbers into the original header first-space range (x165, without position
search). Original 5.11 independently exports the same eight G-major notes and
recognizes a frozen 4/4 TimePair at [166,221,19,45]. Native previously refused
`music_frozen_time_header_import_pending`; it now registers the actual top/bottom
number glyphs, Containment and TimeTopBottomRelation in the retained owner,
erases those canonical glyphs, reads the pair numerator/denominator, and emits
the same eight notes/F# and 4/4 attributes. No synthetic pair glyph or default
meter is introduced. This is one controlled image variant, separate from the
105 key kernels and first x185 G-major key image.

```sh
cargo build --locked --bin audiveris-musicxml
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-header-time/g-major-header-time-quarter.png header-time.musicxml header-time.json
```

Current selected-pair import is limited to frozen 4/4 with two TIME_FOUR members.
Header TimeWhole/common/cut, other meters/clefs and mid-score changes remain
unconnected. They remain on the final original-function parity target.
The full552 regression/fmt/clippy belongs to migration source2395352 / main80340be;
this following TimePair change uses targeted MusicXML/header checks and a fresh
normal checkout. A new full554 regression is not claimed. All global completion
flags remain false; test deletion remains deferred.

## Recognized 3/4 header → MusicXML

The fixed 4/4 restriction is removed. A selected frozen TimePair and its actual
TOP/BOTTOM number shapes, values, relations and physical glyphs now determine
the meter; no default meter or synthetic glyph is inserted. Following original
`PageRhythm.populateTimeSignatures` / `TimeRational.getValue`, measure duration
is numerator/denominator in whole-note units, inherited when no new time is
present. The strict complete-measure check uses that duration instead of four
quarters. The current voice still supports quarter/half notes/rests only, so
a measure duration requiring fractional-quarter units refuses; meter changes
also refuse until time-range/export propagation is connected.

One new original-rendered Bravura44, treble, G-major, ordinary five-line, zero-skew
3/4 image contains two complete measures and six quarter notes:
B4 G4 E4 | B4 F♯5 D5. Unmodified original5.11 full image → RHYTHMS/PAGE/export
and native image → MusicXML agree on all six events and the 3/4 meter. Separately,
saved original OMR TimePair/number glyphs, three-decimal grades, measure durations
and offsets are compared against native owned state. Original XML is never fed
to the native runtime. Other integer-quarter meters use generalized propagation
but are not claimed as whole-image verified. At this 3/4 checkpoint Header TimeWhole/common/cut was pending; the next
section records COMMON_TIME connection. Other clefs, fractional-quarter
rhythms and time changes remain pending.

```sh
cargo build --locked --bin audiveris-musicxml -j 1
# Choose fresh output paths; Java is not required.
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-header-meters/g-major-header-three-four.png \
  three-four.musicxml three-four.report.json
```

The input/expected observations are in `fixtures/official-image-musicxml-header-meters`.
Original export/OMR/provenance and a small validation summary are in
`evidence/header-meter-meaning`. Preservation uses normal source commits and
Git SHA/tree readback; the historical encoded backup chain is not resumed.
Global HEADS/STEMS/MusicXML/epilog completion flags remain false.

3/4 checkpoint149b059 validation: MusicXML integration 7 tests (including both 4/4 and 3/4 plus the underfilled 3/4 boundary), HEADER 2 tests, and TimeColumn 1 test/37 original controls pass. Workspace all-targets clippy with warnings denied, fmt, Linux CLI and local MusicXML4 XSD pass. No new full regression was run; the last full552 result remains attached to source239535 before the TimePair/meter additions.

## Recognized COMMON_TIME header → MusicXML

One further original-rendered G-major image has two 4/4 measures and eight
quarter notes B4 G4 E4 G4 | B4 D5 F♯5 D5, with a recognized frozen COMMON_TIME
header. Unmodified original5.11 and native full-image exports agree on these
notes and `<time symbol="common"><beats>4</beats><beat-type>4</beat-type></time>`.
Saved OMR time glyph/grade and native owned registry/SIG comparison are separate.

The existing original TimeWhole shape mapping now feeds the header import.
PageCleaner erases the actual physical glyph; header TimeWhole is excluded
from overlap candidates under the original `SigReducer.reduce` header filter.
No synthetic number pair is created. `PartwiseBuilder.processTime` supplies
the common/cut MusicXML symbol attribute. CUT and other TimeWhole forms reuse
this mapping but lack whole-image verification; they are not claimed verified
by the COMMON_TIME input. No default meter, incomplete-measure padding, meter
change or fractional-quarter rhythm has been invented.

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-header-common/g-major-header-common.png \
  header-common.musicxml header-common.report.json
```

Build as above. Original OMR/export/provenance and the small validation summary
are in `evidence/header-common-meaning`. The CLI remains a partial port; the
full original feature ledger, including non-treble clefs, accidentals, rhythmic
relations, polyphony, multi-staff, correction/project support, scans/fonts/skew
and complete cleaner/epilog/export, remains the target. No original function
is planned for omission. All global completion flags remain false.

COMMON_TIME checkpoint validation: one complete workspace regression after the coherent HEADER changes passes all554 tests, zero failures. Fmt, workspace all-targets clippy with warnings denied, Linux native CLI, normalized original MusicXML comparison and local MusicXML4 XSD pass. Prior checkpoints keep their historical source/result boundaries; old recognition reports are unchanged.


## Recognized bass/alto header → MusicXML

The same image-owned `ClefInter` now supplies both `ClefInter.noteStepOf/octaveOf`
and `PartwiseBuilder.buildClef` semantics. The writer selects G/F/C, computes
`line = 3 - rint(pitch / 2)` and emits octave changes for 8va/8vb shapes. It
compares sign/line/octave-change, rather than physical Inter identity, before
writing a new clef. Repeated unchanged headers do not duplicate attributes.
A different clef inside a measure still refuses until ordered clef events are
connected; measure-start changes have writer support, not a new image-parity claim.
Percussion requires unpitched note mapping and still refuses. Missing headers
never create default notes or clefs.

Two development inputs use Bravura51 rendering / recognized44pt, interline11,
ordinary five-line staff, zero skew, C major, explicit4/4, one voice and two
complete measures of quarter notes. F clef is placed on the fourth line and C
clef on the middle line using original font baselines; there is no coordinate
search. Unmodified original5.11 processes each *same image* through RHYTHMS and
export; native does not load its XML or Java. Native output matches:

- Bass: `D3 B2 G2 B2 | D3 F3 A3 F3`, `<sign>F</sign><line>4</line>`.
- Alto: `C4 A3 F3 A3 | C4 E4 G4 E4`, `<sign>C</sign><line>3</line>`.

The alto image exposed a genuine connection gap: a left border residue yielded
an ARPEGGIATO evaluation. Original `InterFactory.create` calls
`ArpeggiatoInter.createValidAdded`, which returns null without a possible chord
link. Native now applies the empty-search branch using the original profile1
right-side lookup rectangle (`rint(2.5 * interline)`, half-open intersections).
It uses all retained head-chord bounds conservatively. Any possible chord link
still refuses; positive arpeggiation, relation grades and export remain pending.
The original null/edge/one-pixel-overlap controls are separate kernels on the
actual residue and saved RHYTHMS chord context, not extra successful images.

```sh
cargo build --locked --bin audiveris-musicxml -j1
mkdir clef-output
./target/debug/audiveris-musicxml --output-dir clef-output   fixtures/official-image-musicxml-clefs/bass-header-quarter.png   fixtures/official-image-musicxml-clefs/alto-header-quarter.png
```

Output paths must not already exist. Each input produces a `.musicxml` and a
`.report.json`. With `CARGO_TARGET_DIR`, use that directory's `debug/` binary.
Product execution requires neither Java nor original recognized output.

Original buildClef controls cover nine pitched shapes and four C-clef positions
(12 cases), separately from the preexisting180 pitch controls. Small/octave
clefs, tenor/soprano images, percussion, general ledgers, short notes/beams,
mid-measure changes, scans/skew/other fonts/staffs and complete rhythm/score
semantics are not claimed as newly image-verified. The previous full554 belongs
to51f8d44; this change uses affected MusicXML and clef tests, not a repeated
full-workspace run. All global completion flags remain false. These are staged
boundaries in the full nonUI parity goal, never planned feature omissions.


## Single-flag eighth notes → MusicXML

`FlagInter` is now created from classified physical glyphs using original
`AbstractFlagInter.lookupLink/createValidAdded`: first eligible stem in x order,
profile1 lookup/foot area, intrinsic profile0 gap impacts, head/stem polarity
and extension point. The retained stem/cache identity is used by
`FlagStemRelation`; no stem or duration is supplied by a fixture. Original
`SIGraph.computeContextualGrade` includes both HeadStem and FlagStem support
before weak flag purge. Containment/ChordStem identity relations add no support.

`AbstractChordInter.getDurationSansDotOrTuplet` divides the head duration by2
for an actual attached flag. Single-voice offsets and measure sums now use
exact eighth-of-whole integer ticks. MusicXML uses divisions2 when eighths are
present and keeps divisions1 on existing quarter/half inputs. Under/overfilled
measures still refuse. Finer fractions, dots and tuplets are not guessed.

One clean Bravura51 render / recognized44pt, interline11, zero-skew, ordinary
five-line treble/C-major/4/4 two-measure image is verified through unmodified
original5.11 full-image RHYTHMS/export and native recognition. It contains
`B4(1/8) B4(1/8) G4(1/4) E4(1/4) G4(1/4) | B4 D5 F5 D5`.
Native matches9 notes, offsets, two flags' physical glyphs, saved three-decimal
flag/support grades and `<type>eighth</type><stem>up</stem>`. This input has no
beams in either original or native output; it is **not positive beam parity**.
The separate original lookup/grade kernels cover boundary contact,1px overlap,
wrong vertical position and signed gap limits. Removing the two actual flag
wing rasters produces five quarters and refuses, proving that duration does
not come from stored note coordinates or the expected output list.

```sh
cargo build --locked --bin audiveris-musicxml -j1
./target/debug/audiveris-musicxml   fixtures/official-image-musicxml-flags/flag-eighth-quarter.png   flag-eighth-quarter.musicxml flag-eighth-quarter.report.json
```

Use `CARGO_TARGET_DIR/debug/` when the target directory is overridden. Output
paths must not already exist. No Java/original recognized XML is needed at runtime.

Historical125e928 boundary, closed for the single-beam case below: positive beam-origin builder/items/cache scheduling and beam
finalization, positive down/small/multiple flags in full images, supported-symbol
conflicts, finer rhythmic units, tuplets/dots/polyphony/changes/general original
parity. At that checkpoint a potentially reachable beam refused. The full original nonUI
parity goal is unchanged; HEADS/STEMS/MusicXML/epilog/automatic flags remainfalse.

This flag/support/fractional-rhythm checkpoint passed one full workspace run:
558 tests,0 failures/ignored, plus fmt, all-target clippy with `-D warnings`,
the direct Linux CLI and MusicXML4 XSD. Earlier checkpoint counts retain their
own source identity. [Validation summary](../evidence/musicxml-flag-eighth/VALIDATION.json).

The next positive beam input is retained at
`fixtures/official-image-musicxml-beams/beam-eighth-quarter.png`. Original5.11
successfully exports two eighths with beam1 begin/end and seven quarters.
At the historical125e928 checkpoint native refused
`music_beam_with_stump_or_seed_pending`, with no output pair written. The
following section records the newly connected successful native image path. [Concrete remaining dependencies](../evidence/musicxml-beam-eighth/NEXT-DEPENDENCY.md).


## Single standard beam eighth notes → MusicXML

The saved positive image now runs without Java:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --bin audiveris-musicxml -j1
mkdir -p work/beam-example
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-beams/beam-eighth-quarter.png \
  work/beam-example/score.musicxml work/beam-example/report.json
```

Use absent output filenames; the CLI refuses overwrite. A custom target directory
changes the executable path, not the input or recognition settings.

This is the same Bravura44/interline11, ordinary five-line treble/C-major4/4,
single-voice, two-complete-measure image that original5.11 exports successfully.
First measure: B4 eighth, B4 eighth, G4 quarter, E4 quarter, G4 quarter.
Second: B4, D5, F5, D5 quarters. All nine pitches/durations/onsets and the two
`beam number="1"` begin/end values match; divisions2 and MusicXML4 XSD pass.
The existing quarter/rest/half/ledger/key/meter/clef/flag examples remain usable.
The incomplete old beamed-note negative now reaches the correct three-beat
measure-duration refusal rather than being stopped before beam recognition.

Original dependencies connected: `BeamLinker.VLinker.buildLuArea/filterHeads`
and automatic `StemBuilder.filter/lookupChunks/items`, `VLinker.expand/link`
with side profile4/artificial-grade fallback, beam-first head scheduling and
actual canonical cache/SIG attachments; `Finalizer.boostBeamSides` temporary
supports; beam-aware contextualization/retained-owner head competition,
`SigReducer.checkBeamsHaveBothStems`, bounded `PageCleaner.processArea`,
`BeamHeadCleaner.process`; `AbstractChordInter.getBeams/getDurationSansDotOrTuplet`
and `PartwiseBuilder` beam ordering and numbering. Membership comes from actual
BeamStem/ChordStem links, not fixture coordinates or output reuse.

The original HEADS reflection builder/reduction and declared artificial-stem
controls are separate from the successful original full-image RHYTHMS/export.
Native preserves raw prior beam metadata, while lookup/eraser geometry follows
the existing one-decimal XML-restored context. Raw beam bounds and final original
stem-glyph trimming are not whole BEAMS/STEMS/SIG equality. Reduced head bounds
match; raw versus saved-three-decimal grades retain small differences.

Historical4a87a20 admission (extended below): one standard beam per system, one surrounding part,
no sibling/alien/cross-system shared-glyph beam, zero skew and uniform ordinary
staff. The eraser currently certifies horizontal restored integer borders and
Java2D's normalized one-pixel contour. Siblings/multiple beam levels/hooks,
closer limits, cross-staff/system, sloping/fractional/wider eraser strokes,
mirrored/void beamed heads and broader cleaner/reduction/rhythm remain pending.
At4a87a20, `continue` was implemented by ordered owned chord membership but had no whole-image
comparison yet. The validated image has two side stumps and no intermediate
stump; the original side-first/failed-side scheduling branch must be connected
and compared before claiming intermediate-stump image support. Nonzero skew, real scans, other fonts/special staffs/full24
pages and32sources remain managed limitations, not planned omissions.
All global completion flags stay false. Releases/Library/GHA are unchanged.

This checkpoint covers all563 unique workspace tests using the passed fixed-production
prefix and the corrected boundary/unrun targets/crates/doc-tests. Two stopped runs
and their fixes are recorded separately; this is not an uninterrupted full-suite
run. Cargo test-list coverage found no missing or extra tests; fmt and all-target
clippy passed. An independently fetched clean clone built the Linux CLI and
repeated the nine-event comparison/XSD check. [Validation](../evidence/musicxml-beam-eighth/VALIDATION.json).


## Intermediate beam stumps and continue

The same native Linux command now accepts
`fixtures/official-image-musicxml-beam-subdivisions/beam-four-eighth-quarter.png`.
Four B4 eighths, G4/E4 quarters and the second measure's four quarters match
unmodified original5.11: ten events, pitches, exact onsets/durations, and
beam1 `begin / continue / continue / end`. MusicXML4 XSD passes.

`BeamLinker.linkSides/linkStumps` now schedules LEFT, RIGHT, then intermediate
stumps, retaining the original failed-side stop and all vertical candidates.
Head-origin checked BeamStem supports reach the actual owned SIG/cache.
Horizontal fractional borders use Java2D's quarter-pixel normalization; eight
declared numeric raster boundary controls and the original image beam's6.1px
height are checked separately from full-image export. Slopes/wider strokes,
sibling/multiple beams, hooks/alien limits and general cleaner remain pending.
No image coordinates or oracle outputs are recognition inputs. Global completion
flags remain false; original nonUI full-function parity remains the final goal.


## Two sibling beams → sixteenth notes

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --bin audiveris-musicxml -j1
mkdir -p work/beam-subdivisions
./target/debug/audiveris-musicxml --output-dir work/beam-subdivisions \
  fixtures/official-image-musicxml-beam-subdivisions/beam-four-eighth-quarter.png \
  fixtures/official-image-musicxml-beam-subdivisions/beam-four-sixteenth-quarter.png
```

With a custom `CARGO_TARGET_DIR`, use that directory's `debug/audiveris-musicxml`.
Output names must be absent. Java is only an oracle/development dependency;
the native Linux CLI loads images and its own classifier/template artifacts.

The second input is ordinary Bravura44/interline11, treble/C-major4/4,
single voice, two complete measures. Four B4 sixteenths with two positive
beam levels, then G4/E4/G4 quarters; second measure B4/D5/F5/D5 quarters.
Unmodified original5.11 exports all11 events. Native pitches, exact durations,
onsets and both numbered beam sequences match; divisions4 and MusicXML4 XSD
pass. This adds an input, not a fixture-specific recognition branch.

Original dependencies: `VLinker.filterBeams/filterHeads` supplies sibling full
LinkerItems and the last group border before heads; stable descending beam width,
all `linkSides` before all `linkStumps`; `linkSiblings` applies the base relation
grade to the same actual stem, with the shorter-beam head-side test and linked
BLinker state; `BeamBeamRelation` default grade1/coefficient3 contributes to
context with typed beam endpoints. `AbstractChordInter.getBeams` orders from
tail toward head; connected beam count supplies binary duration, then exact
sixteenth ticks and minimal1/2/4 MusicXML divisions preserve whole measures.
The legacy `duration_eighths` report field is now0.5 for a sixteenth.

Separate original HEADS inspection checks all8 sibling builders' items/lines/
glyphs/length maps; actual process/finalizer observations check four shared
stem grades and8 BeamStem support ratios. These are not full original/native
SIG identity or final glyph-trimming equality. Saved prior reports retain
their historical source identity.

Current bounded positive image admission: one ordinary physical beam group per
system, one surrounding part, zero skew, horizontally normalized thin eraser;
up to two complete standard beam levels for export. General builder scheduling
and membership do not depend on those images' coordinates. Hooks/competing
hooks, alien closer limits/separate groups, cross-system glyph aliases,
cross-staff parts, slopes/wider strokes, more levels/multiple flags, dotted/
tuplet/polyphonic rhythm and full reduction/cleaner/epilog remain unconnected or
unverified. Nonzero skew, real scans, other fonts/special staff/full24shape
pages/32sources remain limitations. All original nonUI functions stay on the
implementation ledger; no global completion flag is set.


Original `Inters.byReverseWidth` uses actual integer bounds widths. A final
small fidelity correction keeps same-width insertion order even when fractional
median lengths differ; three original numeric BeamInter controls certify the
boundary separately from the image comparisons. Full565 regression belongs
to321abfc; the final comparator change is checked with affected targets, not
reported as another whole-workspace run.

The independent clean clone fetched9b14a4e, matched its Git tree, built the
Linux CLI with offline locked Cargo, and repeated both inputs. Direct emitted
XML pitches/durations/onsets/beam1/2 match original5.11 (10/11 events); both
MusicXML4 XSD checks pass. This readback is distinct from the full565 run on
321abfc and the five affected target checks after the bounds-width correction.
See [validation summary](../evidence/musicxml-beam-subdivisions/SIXTEENTH-VALIDATION.json).
## Separate ordinary beam groups → MusicXML

Current additional runnable unit: one clean Bravura44/interline11, ordinary
five-line treble/C-major/4/4 single-voice image has two separate pairs of eighth
notes and two quarters in measure1, followed by four quarters in measure2.
Native recognizes the input and emits ten events, with independent
`begin/end`, `begin/end` beam marks. It does not import original XML or oracle
recognition results. Original5.11 full-image RHYTHMS/PAGE/export agrees on pitch,
duration, onset, voice, beam marks and the beamed stem directions; native and
original output validate against MusicXML4 XSD. Broader engraving, including
original explicit quarter-note stem directives, is not claimed identical.

```sh
cargo build --locked --bin audiveris-musicxml -j1
mkdir -p work/beam-groups
./target/debug/audiveris-musicxml --output-dir work/beam-groups \
  fixtures/official-image-musicxml-beam-groups/beam-two-groups.png
```

The CLI requires no Java runtime. With `CARGO_TARGET_DIR` set, use its debug
binary path instead of `./target/debug`. Output files must not already exist.
For low-space builds, retain the established `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_DEV_DEBUG=0` settings; sources/inputs/evidence are not disposable
build caches.

The generic original closer-limit algorithm now runs on actual owned beam
bounds/grade/group geometry before seeds, heads and chunks are selected.
Sixteen original numeric controls cover positive limits and boundaries. The new
image has nonintersecting groups and null limits; its four HEADS-loaded
builders/items/lengths/selected glyphs are a separate comparison. Original
AA-off fill/stroke rows verify the fractional contours needed on this image.
General sloping contours, hooks/competing hooks, aliases/cross-staff ownership,
more beam levels/flags/dots/tuplets and full rhythm/reduction/cleaner/epilog are
still pending. No final nonUI original feature is planned for omission.

This stage uses meaningful affected regressions and fmt/clippy. The previous
full565 result belongs to source321abfc; no new full-workspace result is claimed.
[Verification summary](../evidence/musicxml-beam-groups/VALIDATION.json) and
[next dependencies](../evidence/musicxml-beam-groups/NEXT-DEPENDENCY.md).
