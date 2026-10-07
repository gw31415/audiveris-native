> 最終目標はUI表示を除く原版の譜面認識・解析・補正・保存・MusicXML出力機能同等性です。本文の限定範囲は現在の小チェックポイントの受付/検証境界であり、最終スコープ削減ではありません。[全機能台帳と接続順](PORT-STATUS.md)。

# Native image → MusicXML

Latest image increment: [two-flag sixteenth chords](#two-flag-sixteenth-chords-2026-10-06); earlier [multi-head eighth beams](#multi-head-eighth-beams-2026-10-06); earlier [same-stem multi-head chords](#same-stem-multi-head-chords-2026-10-06); earlier [parallel half-note Voice](#parallel-half-note-voice-from-an-actual-image-2026-10-06); earlier [parallel flagged eighth Voice](#parallel-flagged-eighth-voice-from-an-actual-image-2026-10-06); earlier [parallel two-level sixteenth Voice](#parallel-two-level-sixteenth-voice-from-an-actual-image-2026-10-06); earlier [parallel beamed eighth Voice](#parallel-beamed-eighth-voice-from-an-actual-image-2026-10-06); earlier [known late Voice forward](#known-late-voice-forward-from-an-actual-image-2026-10-06); earlier  [parallel quarter Voices](#parallel-quarter-voices-from-an-actual-image-2026-10-06). Older processing stages:  [ordinary chord prolog/late](#ordinary-chord-compatibility-before-weak-purge), [ordered foundation checks/purge](#ordered-foundation-checks-and-per-check-purge), [fresh epilog seed lists](#fresh-epilog-seed-lists), [ordinary stem consistency/cache removal](#ordinary-stem-consistency-and-cached-removal) and [owned weak-stem purge](#owned-weak-stem-purge). The [two-system measured47pt input](#two-system-beams-with-measured-head-font-and-owned-epilog) retains20 original-matching events and one cross-system glyph-alias orphan deletion. Earlier sections retain their checkpoint-specific comparison boundaries. The full migration remains incomplete.

`audiveris-musicxml IMAGE OUTPUT.musicxml REPORT.json` connects the actual image-owned HEADS/STEMS candidates and caches, reduced SIG, native staff-barlines/chords, isolated SymbolsFilter/classifier and bounded ordinary-Voice duration/onset calculation to a native MusicXML 4.0 writer. It does not read original recognition XML or use a Java runtime.

The first verification checkpoint used three independent clean opaque PNG pages with Bravura, interline11, zero skew, ordinary five-line staff, treble clef, no key signature, explicit 4/4, one voice, and two/four complete measures, including two systems. Black/void heads produce quarter/half notes; the separate rest page contains quarter rests. Durations, rests, pitches, offsets and voices match original 5.11 XML-loaded HEADS followed by native original MEASURES/CHORDS/SYMBOLS/LINKS/RHYTHMS. This oracle boundary is distinct from original end-to-end transient recognition.

Native quarter, quarter-rest, half-note and four-measure outputs were also checked with MusicXML 4.0 XSD and original proxymusic reimport. A fixed development renderer reconstructed that first input exactly; it is not a general engraver. The CLI refuses missing recognized meter, incomplete/excess measures, ambiguous symbols and existing outputs. It does not guess omitted music.

The batch entry is `audiveris-musicxml --output-dir EXISTING_DIR [--part-name NAME] IMAGE...`. It writes one MusicXML/report pair per input, with preflight collision/no-overwrite checks. A later recognition failure stops the batch and retains individually reported successful pairs; failed writes remove that pair. Part name is metadata for one nontransposing melodic part, not instrument inference or recognition control. Three independent synthetic scores, including four measures and a system break, are verified; arbitrary real-score admission remains unverified. A central quarter-rest configuration refuses weak head optionals instead of emitting an invented note. Half rests are additionally verified at two controlled positions, including a page with provably unreachable orphan beam candidates (details below). PDF, scans/photos, nonzero skew, other fonts/scales, special staffs, full24 shapes, accidentals, other beam/flag configurations, dots/ties/slurs/tuplets, polyphony, pickups, multi-staff or transposing instruments remain outside this bounded path. Complete cleaner/epilog and full reduction parity are pending. All global completion flags remain false.

The accepted full542 regression belongs to source48e6177; changed-source validation is recorded by checkpoint. Existing test reduction audit is deferred until current implementation/verification/necessary fixes finish. No Actions dispatch/rerun is used.


Half-rest continuation (2026-10-05; historical early boundary): two controlled positions of the same clean
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


## Two sibling beams → seventeenth notes

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --bin audiveris-musicxml -j1
mkdir -p work/beam-subdivisions
./target/debug/audiveris-musicxml --output-dir work/beam-subdivisions \
  fixtures/official-image-musicxml-beam-subdivisions/beam-four-eighth-quarter.png \
  fixtures/official-image-musicxml-beam-subdivisions/beam-four-seventeenth-quarter.png
```

With a custom `CARGO_TARGET_DIR`, use that directory's `debug/audiveris-musicxml`.
Output names must be absent. Java is only an oracle/development dependency;
the native Linux CLI loads images and its own classifier/template artifacts.

The second input is ordinary Bravura44/interline11, treble/C-major4/4,
single voice, two complete measures. Four B4 seventeenths with two positive
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
seventeenth ticks and minimal1/2/4 MusicXML divisions preserve whole measures.
The legacy `duration_eighths` report field is now0.5 for a seventeenth.

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
Seventeen original numeric controls cover positive limits and boundaries. The new
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


## Forward/backward secondary beam hooks

The native image CLI now connects typed BeamHookInter builders and actual
cache/SIG/chord membership to sixteenth durations and forward/backward hook
marks. One original5.11-successful image has four sixteenths with a primary
begin/continue/continue/end beam and alternating secondary hooks, followed by
three quarters and a second measure of four quarters (11 events).

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --bin audiveris-musicxml -j1
mkdir -p work/beam-hooks
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-beam-hooks/beam-four-hooks.png \
  work/beam-hooks/score.musicxml work/beam-hooks/report.json
```

Use the configured Cargo target directory if your build sets CARGO_TARGET_DIR.
This Linux recognition/export CLI needs no Java runtime. The Java renderer and
observers are development-only original comparison tools.

`BeamLinker.linkSides` is connected with system link profile0 for hooks and
competing ordinary beams, maximum inspection profile4 for sides, and profile1
for intermediate stumps. Hooks try both sides and need only one linked side.
An excluded same-glyph hook is removed only when the ordinary beam links both
sides; failures do not discard already-created stems. Typed contextual supports,
`checkHooksHaveStem`, the certified thin rectangular PageCleaner contour and
BeamHeadCleaner operate on actual SIG objects. PartwiseBuilder's hook mark uses
the integer beam center relative to the actual stem center. Ordinary full beams
retain their two-chord/end checks; complete-measure rhythm guards remain.

The 11-event full-image comparison, the restored HEADS builder/SIG/raster
observations, and four declared competing-hook plus four endpoint controls are
separate evidence scopes. The full image has distinct glyphs, not a competing
beam/hook glyph. No full-SIG or general engraving parity is claimed. General
aliases, cross-staff/system/multiple-part groups, more than two beam levels,
other hook configurations, wider/general sloping eraser strokes, full reduction/
cleaner/epilog, nonzero skew/scans/other fonts/special staff remain pending.
All global completion flags remain false. See
[evidence](../evidence/musicxml-beam-hooks/VALIDATION.json) and
[next dependencies](../evidence/musicxml-beam-hooks/NEXT-DEPENDENCY.md).

## Same-glyph beam/hook competition and retained groups

A second hook page exercises four physical glyphs that original HEADS admits
as both an ordinary beam and a hook. It preserves the same 11 musical events
as the original: four B4 sixteenths with primary begin/continue/continue/end
and secondary forward/backward hooks, then the existing quarter notes.
Recognition and MusicXML already matched before this checkpoint. This change
corrects retained group ownership after competition/reduction: deleted beam
alternatives disappear from `members`, bounds are recomputed, and dying groups
are removed through the original ensemble callbacks. No input-specific branch
is used. Original full-image export, separate loaded-HEADS group observations,
and declared deletion/grade controls are distinguished in
[the dependency record](../evidence/musicxml-beam-epilog/NEXT-DEPENDENCY.md).

```sh
cargo build --offline --locked --bin audiveris-musicxml
mkdir -p work/beam-epilog-output
target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-beam-epilog/beam-competing-hooks.png \
  work/beam-epilog-output/score.musicxml \
  work/beam-epilog-output/report.json
```

Use new output filenames: the CLI refuses overwrites. With a configured Cargo
`target-dir`, use that directory instead of `target/debug`. The executable is
Linux-native and needs no Java runtime. Full cleaner/epilog, cross-system glyph
aliases and general engraving remain unfinished; no global completion flag is
set. This is an additional testable unit toward all original nonUI functions.

The MusicXML image route now executes the original automatic head-stem cleanup
order before beam-side support: build intrinsic-grade/exclusion ensembles from
its live SIG, then apply the existing cleaner. No selected parts/ensemble API
is required from CLI users. The same input/command above remains usable. One
three-stem/exclusion control is an original SIG comparison, distinct from the
full-image alias page. The diagnostic linking CLI keeps its earlier pre-cleaner
observation boundary so saved comparisons remain reproducible. Single-voice
export still requires one retained stem per head; general polyphony, parallel
relations, >16 cleaner partners, cross-system orphan finalization, wider/sloping
erasure and full reduction/epilog remain unfinished, not planned omissions.

## Ordered owned beam epilog across systems (intermediate68d96ff)

The image path now runs original ordered cross-system orphan checks on actual
GRID neighbors/staff curves and canonical physical glyph records, then per-system
beam-side support/contextualization. Eight original declared SIG cases agree;
this does not claim positive orphan removal on an automatic input image.
Existing hook/group/competition CLI examples remain the runnable examples.
At that intermediate source, the new original-successful two-system20-event input
stopped at the exact Bravura47pt template dependency. The next checkpoint below
connects it and verifies one positive image removal; original XML is never
substituted as native output.
Full epilog and all original nonUI functions remain pending, not omitted.

## Two-system beams with measured head font and owned epilog

This checkpoint accepts one original-successful clean Bravura input with measured
47pt heads (14x12px), interline11, zero skew, ordinary five-line treble/C-major4/4,
one voice, four complete measures over two systems. Each system starts with four
beamed eighths and two quarters, then four quarters. Native emits20 events matching
original5.11 full-image RHYTHMS/PAGE/export on pitches, durations, onsets, voices,
beam marks and beamed stem directions; both outputs pass MusicXML4 XSD.

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --bin audiveris-musicxml -j1
mkdir -p work/two-system-beams
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-cross-system/beam-two-systems.png \
  work/two-system-beams/score.musicxml work/two-system-beams/report.json
```

Use the configured Cargo target directory where applicable. Output files must
not already exist. Recognition/export needs no Java runtime and reads no original
MusicXML. The new bank contains exact original47pt ordinary oval templates only,
without font-size substitution/interpolation. Other measured sizes/shapes remain
bounded by their available exact assets and downstream dependencies.

All retained standard beams now follow actual original VLinker scheduling, even
when no heads/parts can be reached. The early horizontal-envelope removal shortcut
is retired. The same image's original HEADS-loaded process→ordered Finalizer
changes3 beams to2 by removing the stemless upper-system alias of the lower beam;
native removes the same physical glyph alias. Actual group removal/cache ownership
and per-system contextualization remain connected. This proves one positive
image case; the eight declared original SIG conditions are separate comparisons.

Complete cleaner/epilog/SIG and general cross-staff/system behavior, SmallBeam,
voices/parts, other rhythms/notation, nonzero skew, real scans, special staffs and
full24-shape page parity remain unfinished. No original nonUI function is planned
for omission. All global completion flags stay false. Earlier verification records
retain their own source commits.

[Validation](../evidence/musicxml-cross-system/IMAGE-VALIDATION.json) and
[next dependencies](../evidence/musicxml-cross-system/NEXT-DEPENDENCY.md).

Validation checkpoint: one whole-workspace run passed577 tests with zero
failures/ignored, fmt and workspace/all-target clippy passed at native source
`c6f03c5`. The original observer follows fresh-retriever-per-system StemsStep
epilog order and produces unchanged beam/glyph observations.

Independent existing clone fetched/fast-forwarded to `a5976c0`,
matched root/src/crates trees, built the Linux CLI offline/locked, and reproduced
20 original-matching events plus one actual orphan removal; both XMLs pass XSD.
[Readback receipt](../evidence/musicxml-cross-system/FINAL-READBACK.json).

## Fresh epilog seed lists

The two-system CLI command above now also retrieves each system's retained
VERTICAL_SEED free-glyph list and repeats the original connected-bar-area purge
before Finalizer. The report's linking stage exposes
`limited_beam_finalizer.systems[].vertical_seed_repurge`.
Only retriever candidates are filtered; registry/group/SIG/cache ownership stays
intact. This changes no accepted notation or input assumptions. Positive bar-area
evidence is from the existing Laura saved HEADS state (98→97 and202→200),
separate from the20-event image MusicXML regression. Only vertical medians with
a positive aligned gap are admitted. Angled/empty/reversed ribbons, multi-staff
MusicXML and full reduction/cleaner remain pending final targets.

## Ordinary stem consistency and cached removal

The existing two-system and competing-hook commands also run ordinary
`SigReducer.checkStems`: direction considers head and beam connection source
grades; headless stems are removed from the actual SIG and the same canonical
cache object; tail-side head links are cut with invading overlap exclusions.
Retained foundation checks repeat after structural changes before export, with
an explicit64-pass resource budget.
The report adds `limited_stem_finalizer.original_stem_consistency`.

This is generic connection logic, with no input-specific branch. Five declared
original SIG controls and an owned-cache head-loss condition validate internal
behavior separately from the existing image outputs. It does not add an accepted
notation/font/staff/rhythm category. Initial weak-stem purge, full original
reduction ordering/context, chord/frozen/late checks and lyrics InterCleaner
remain pending. See [next dependencies](../evidence/musicxml-seed-reduction/NEXT-DEPENDENCY.md).

## Owned weak-stem purge

The initial image-owned `SIGraph.deleteWeakInters` continuation now includes
weak stems. Selection is staged against the retained SIG; every affected
canonical cached stem is resolved and locked before SIG/cache removal commits.
The same cached object becomes removed, its physical glyph stays registered,
and re-registration cannot resurrect it. Missing ownership refuses before any
live mutation. This uses computed native contextual grades, without caller
selection IDs/grades, serialized Graph imports or image-specific branches.

The unmodified original deletes a declared stem at the existing original
createStem grade0.3013532310210354 and retains exact0.5 and frozen stems.
These three declared conditions and the actual-owner cache check are separate
from image→MusicXML comparisons. The current same-image20-event example remains
the input below; this stage does not claim a new image type or notation.

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-cross-system/beam-two-systems.png \
  /tmp/beam-two-systems.musicxml /tmp/beam-two-systems.json
```

Output paths must not exist. Historical full580 regression belongs to
source`c270552`. At the weak-cache checkpoint, per-check purge and chord
prolog/late were pending; their ordinary subset is connected in the sections
below. General overlap/broader chord/frozen reduction, lyrics/container
ownership and remaining cleaner/epilog are still pending.
Global completion flags remain false; all remaining nonUI functions remain
implementation targets. [Validation](../evidence/musicxml-seed-reduction/WEAK-PURGE-VALIDATION.json).

## Ordered foundation checks and per-check purge

The image CLI now runs the original ending-head/head/hook/beam/ledger/stem
consistency order, with contextualize/owned weak purge immediately after every
check. Modification-driven consistency passes precede exclusion reduction;
weak or exclusion removals start another epoch. The same registry, live SIG
and cached stems remain owned throughout. The historical diagnostic
`--head-foundations` boundary keeps its previous comparison semantics.

A declared three-inter original cascade matches actual AdapterForFoundations
and native retained-owner checks: three passes, modification counts2/1/0. This
is distinct from automatic raster comparison. The existing two-system20-event
input and command in [Owned weak-stem purge](#owned-weak-stem-purge) remain the
small runnable example; no new font/scan/staff/notation admission is claimed.

At this order-only checkpoint, chord prolog/late was pending; its ordinary head
subset is now connected below. Full overlap discovery, broader chord branches,
all support and frozen/merged conditions, full LINKS/InterCleaner/epilog and
nonUI feature parity remain pending.
Completion flags stay false. Latest full580 belongs to c270552; current affected
checks are recorded separately without claiming a new full-workspace run.

## Ordinary chord compatibility before weak purge

The image-owned foundation adapter now runs ordinary `analyzeChords` prolog
before its first weak purge and repeats it at the original late boundary,
followed by exclusion reduction/contextualize/purge. Good intrinsic stems add
same-duration HeadHead support, incompatible black/void-head duration exclusions
and exclusions for intersected unlinked heads (median intersection plus
inclusive0.02 box IOU). Contact insertion retains canonical native IDs and
refuses replacing support with exclusion or the reverse.

Two declared original SIG conditions compare created relationships and every
contextual grade; repeated analysis adds no duplicate relationship. This is
distinct from full-image chord recognition/export. Existing CLI commands and
inputs stay available; no general chord/polyphony/font/scan admission is claimed.
Small/percussion/stemless/cue size branches, general overlap discovery, full
LINKS/lyrics/Part cleaner and epilog remain implementation targets.

Current source validation: `52943de`, all583 workspace tests passed (interrupted
execution resumed only uncompleted binaries;176 binaries covered), doc tests,
fmt/clippy and independent-clone Linux CLI build. The reused
`fixtures/official-image-musicxml-cross-system/beam-two-systems.png` yields20
original-matching events and XSD-valid MusicXML; its native XML bytes remain
unchanged. [Compact receipt](../evidence/musicxml-ordered-foundations/FULL-VALIDATION.json).

```sh
cargo build --bin audiveris-musicxml
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/ordered-foundations.musicxml /tmp/ordered-foundations.json
```

The output paths must not already exist. This validates the retained two-system
example and generic ordinary foundation/chord processing, not general chord
export or full nonUI parity. No new image fixture or admission guard removal.


## Ordinary foundation overlap before chord prolog

`SigReducer.detectOverlaps` now runs on the retained foundation owner before
contextualization and chord prolog for ordinary black/void heads, stems, beams,
hooks and barlines. It keeps x order, the inclusive0.05 bounds IOU prefilter,
both directional `overlaps`, compatible beams, standard-head/stem exemption
and canonical exclusion/support refusal. Typed exclusions to retained GRID
barlines stay in the live SIG for reduction, contributing no Support to the
head/stem contextual grade; unsupported external supports still refuse.
Bounds alone never replace the actual
physical test. Head core uses original floor/ceil shrink; intersecting glyphs
use their foreground runs, including both horizontal/vertical orientation.
Glyphless straight Inter ribbons use the existing OpenJDK linear Area engine
with the original intersection classifier and strict boundary contact semantics.
GRID barlines resolve their existing GRID registry runs rather than copying them
into the HEADS/BEAMS registry or assuming their glyph is absent.

Eleven original declared SIG branch conditions are a kernel comparison, separate
from the retained actual image owner/cache/registry and existing MusicXML image
regressions. The CLI input and command remain:

```sh
cargo build --locked --bin audiveris-musicxml
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/foundation-overlap.musicxml /tmp/foundation-overlap.json
```

Output paths must not exist. No new image fixture, notation/font/scan admission
or caller-selected geometry API is added. The new affected checks and independent
clone CLI/XSD receipt are recorded in
[VALIDATION.json](../evidence/musicxml-foundation-overlap/VALIDATION.json).
Historical full583 belongs to source52943de; it is not a full-workspace run of
this changed source. Existing Laura352/513 and older report/source hashes remain
historical evidence unchanged.

The weaker-branch removal callback for a GRID barline still refuses pending its
full ownership lifecycle; this stage compares barline overlap discovery, not
all possible barline-removal outcomes.

General mirror/ensemble/header/all-class overlap, slur/fermata special geometry,
text ownership/grade adjustment and small/percussion/stemless/cue chord branches
remain partial or unconnected. Full support, Part/lyrics cleaner, LINKS slur/tie
epilog, nonzero skew/scans/special staffs and full24-shape pages remain final
implementation targets. All global completion flags stay false.

This unit passed37 affected tests, fmt and workspace/all-target clippy at source
`ea8c756`. The independent fetched clone built the Linux CLI and emitted the
original-matching20-event two-system page and11-event competing-hook page; both
native and saved original outputs pass MusicXML4 XSD. The competing-hook image
executes one new native overlap exclusion; the two-system page executes zero.
That positive native image condition is distinct from the declared original11
SIG-predicate/grade comparisons, not complete whole-image SIG equivalence.
[Independent readback](../evidence/musicxml-foundation-overlap/READBACK.json).

```sh
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-beam-epilog/beam-competing-hooks.png /tmp/foundation-overlap-hooks.musicxml /tmp/foundation-overlap-hooks.json
```

## Cleaner edge lifecycle on the retained image owner

The existing image CLI now applies original HeadStemsCleaner removals directly
to actual typed edges. First typed parallel-edge selection, manual/frozen edge
cleanup and head/stem abnormal callbacks are compared in six declared original
SIG controls. All vertices, unrelated edges, physical registry and canonical
cache objects remain retained. A declared alternative-edge test on the actual
image owner checks atomic refusal and cache identities; it does not claim an
automatically recognized positive multi-stem image. No notation admission change.

Original automatic high-ratio parallel input throws on a deleted-edge alias;
the native automatic parallel guard remains. Low-ratio automatic is unobserved.
Full cleaner/Part/lyrics/epilog and mirror/ensemble/special-head branches remain
pending. Global completion flags remain false. This source passed33 affected
tests, fmt and workspace/all-target clippy. Historical full583 is at52943de.
[Validation](../evidence/musicxml-head-cleaner-lifecycle/VALIDATION.json).

Reuse the existing CLI inputs with fresh output paths:

```sh
cargo build --locked --bin audiveris-musicxml
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/head-cleaner.musicxml /tmp/head-cleaner.json
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-beam-epilog/beam-competing-hooks.png /tmp/head-cleaner-hooks.musicxml /tmp/head-cleaner-hooks.json
```

Independent fetched-clone20/11-event output and MusicXML4 XSD checks are recorded
separately in [READBACK.json](../evidence/musicxml-head-cleaner-lifecycle/READBACK.json).

At source`e532ae2`, independent fetch/build produced20 and11 original-matching
events; native and saved original MusicXML pass MusicXML4 XSD. Both XML byte
hashes remain identical to the preceding successful checkpoint. Root/src/crates
trees match. The receipt-only commit does not change tested source.

## Original special-head contact predicates

Foundation overlap now applies the original `Shape.isSmallHead` distinction to
all recognized24 head shapes. It uses the same actual glyph/core geometry and
Support refusal: small heads can exclude an intersecting stem; standard heads
retain the original exemption. Six original declared SIG controls cover four
small oval shapes, cross and breve, including one existing HeadStem Support.
These are predicate comparisons, not successful special-head page MusicXML.
No new image fixture or notation admission. Later size/duration/owner branches
remain partial; all global completion flags stay false. Reuse the existing
20/11-event CLI commands above with new output paths.
[Validation](../evidence/musicxml-special-head-overlap/VALIDATION.json).

Source`3f76b49` passed12 affected tests and fmt/workspace-all-target clippy.
Independent fetched Linux CLI preserves original20+11 events, prior XML byte
hashes and native/original MusicXML4 XSD.
[Readback](../evidence/musicxml-special-head-overlap/READBACK.json).

## Typed mirror and ensemble ownership predicates

Original mirror head/chord/note exemptions and stemless HeadChord member
geometry now use actual typed MirrorRelation/Containment in the retained graph.
Both directional member predicates and Support refusal replace ensemble box-only
assumptions; a member gap is not foreground, and an ensemble excludes no own
member. Mirror is original Support with zero default grade coefficients.
Seven declared original SIG conditions are compared separately from actual
image owner/cache identity checks and existing31-event image MusicXML regressions.
No automatic raster mirror creation or full page-SIG parity is claimed.

Stem-bound chord bounds, all ensemble classes/removal callbacks, special-head
duration/size processing and full cleaner/epilog remain partial. All global
completion flags remain false. Existing CLI inputs/commands above stay valid;
no new page admission.
[Validation](../evidence/musicxml-mirror-ensemble-overlap/VALIDATION.json).

Source`00ee541` passed one complete workspace run:588 tests,0 failures/ignored,
176 binaries and7 empty doc-test groups; fmt/all-target clippy passed. The fetched
Linux CLI retains20+11 original-matching events, prior XML bytes and native/original
MusicXML4 XSD. [Readback](../evidence/musicxml-mirror-ensemble-overlap/READBACK.json).
Next size/duration fixtures contain original observations only; Rust not yet ported.

## Intrinsic head duration / beam-size consistency

Original `SigReducer.analyzeChords` now groups heads by intrinsic duration,
not shape name. Size exclusions precede same-duration support; actual BeamStem
edges supply standard/small beam mutual and head-size exclusions. Eight original
declared SIG conditions and24 Shape duration/size/order observations agree.
Existing ordinary prolog/late, atomic unknown/frozen refusal and production
special-head/small-beam builder guards remain. The observed original default
allowSmallHeadOnStandardBeam=false is supported; its nondefault setting remains.

This expands a generic original processing unit, not special-head raster-page
admission or full MusicXML/cleaner parity. Targeted regressions/fmt/clippy are used;
last full588 is source`00ee541`, before this change. The existing two-system image
and competing-hook CLI examples remain the image checks. Next: original
stem-bound HeadChord bounds/leading-note/tail and actual ensemble ownership.
[Validation](../evidence/musicxml-special-chord-consistency/VALIDATION.json).

Source`853b19d`:10 targeted checks passed, including Laura352 heads/513 exclusions;
independently fetched Linux build/20+11 image events/prior exact MusicXML bytes
and native-original MusicXML4 XSD passed. Root/src/crates trees agree.
[Readback](../evidence/musicxml-special-chord-consistency/READBACK.json).

## Stem-bound HeadChord geometry / live owner

Original `HeadChordInter.getBounds/getLeadingNote/computeLocations` now supplies
actual MEASURES/CHORDS and generic overlap ownership. Member bounds are united
and only the stem tail point is added; standard integer center, bottom-up members
and strict distance-tie order are retained. Four declared original controls match.
The former native chord envelope held heads alone. Beam ordering now uses the
original head location rather than the enlarged chord-box center; near-head rest
admission continues to check actual heads. Real Containment/ChordStem and canonical
cache identities are retained and validated before committing the continuation.

Empty-member original location getters throw; native does not invent a note
location. Stem-only bounds are compared separately. Complete ensemble grade /
addition-removal / Part-lyrics callbacks and full cleaner/epilog remain pending.
No full stage flag is enabled. Existing two-system/competing-hook CLI examples
remain usable;17 focused checks include the11-test MusicXML regression file.
[Validation](../evidence/musicxml-stem-chord-geometry/VALIDATION.json).

Source`593425b`:17 target regressions/fmt/clippy and independently fetched Linux
20+11 original-matching image events/prior XML bytes/MusicXML4 XSD passed.
Fetched root/src/crates trees agree; full588 remains earlier00ee541.
[Readback](../evidence/musicxml-stem-chord-geometry/READBACK.json).

## Actual HeadChord identity callbacks

Typed Containment/ChordStem additions/removals now invalidate the registered
HeadChord member order, original bounds/leading note/head-tail locations and
mean member best grade. Original setGrade leaves a computed contextual grade
intact. Six ordered original relation operations agree; malformed later edges
refuse atomically and repeated deletion retains original idempotence. Vertices,
physical glyphs and cached stem identities are preserved. Real MusicMeasures
construction invokes additions; uncached music vertex cleanup uses identity
removal callbacks. No public selected-member/grade recognition API is added.

Relation deletion leaves an empty chord with null bounds/grade, as original.
Cached extensive head/stem deletion, dying containers, RestChord/other ensembles,
Part/stack/lyrics and complete cleaner/epilog remain pending. The existing
image CLI examples remain the image comparison boundary. Six target checks/fmt/
clippy passed; full regression and fetched CLI evidence are recorded separately.
[Validation](../evidence/musicxml-chord-containment-lifecycle/VALIDATION.json).

Source`322135d` passed all590 tests (176 binaries+7 empty doc-test groups),0
failures/ignored, plus fmt/workspace-all-target-clippy. Independently fetched
Linux build/20+11 image events, prior XML byte hashes and native-original
MusicXML4 XSD passed; root/src/crates trees agree. All full-stage flags remain false.
[Full](../evidence/musicxml-chord-containment-lifecycle/FULL-VALIDATION.json),
[readback](../evidence/musicxml-chord-containment-lifecycle/READBACK.json).

Next original RestInter proximity5 conditions show the remaining head-only guard
difference at stem tail. Its standalone Rust prototype agrees, but CLI connection
is pending; the recorded4 RestChord ownership controls are original-only.
[Prototype boundary](../evidence/musicxml-rest-chord-admission/PROTOTYPE.json).

### Rest admission follows the actual chord envelope

`RestInter.createValid` now uses the actual chord bounds, including the stem tail,
rather than the old head-only union. Its half-open intersection and strict
center2D minimum-dx boundary are retained. The original five declared admission
conditions agree; the new native test first reproduced the tail mismatch before
the fix. This is connected to image rest admission, not a new classifier or
notation expansion. Targeted14 tests/fmt/all-target clippy pass; the last full590
run belongs to source`322135d`, before this change. Existing image MusicXML
regressions pass. RestChord vertex ownership, other ensembles and complete
cleaner/epilog remain pending.
[Validation](../evidence/musicxml-rest-chord-admission/VALIDATION.json).

### RestChord actual member and vertex lifecycle

`RestChordInter` now uses the same `AbstractChordInter` Containment invalidation
as head chords: bottom-up actual members, highest leading note, union bounds,
head/tail center and mean member best grade; existing contextual grade survives.
`AbstractInter.remove(true)` is connected for uncached Rest/RestChord ownership:
partial removal refreshes the surviving chord, last-member vertex deletion
deletes its dying chord, extensive chord removal deletes only unshared members.
Direct relation deletion remains distinct from vertex deletion. Stable vertex IDs,
glyphs and cached stems remain retained. This is the actual Symbols/Links graph
path, not a new caller-selected-ID API. Four original registered-SIG conditions
agree; existing weak-rest cleanup and image MusicXML regressions are checked
separately. Part/measure/stack/staff/lyrics callback bookkeeping, cached extensive
head/stem ownership, interleaved BeamRest and complete cleaner/epilog remain
pending. All global completion flags remain false.
[Validation](../evidence/musicxml-rest-chord-lifecycle/VALIDATION.json).

Existing runnable regression (choose unused output paths):

```sh
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/rest-owner.musicxml /tmp/rest-owner.json
```

This20-event image retains the earlier original MusicXML agreement. It does not
claim that all four declared ownership branches are generated by raster input.

### Pre-rhythm note/chord container synchronization

After LINKS reduction, native uses actual live `Containment`/`ChordStemRelation`
and tombstones to refresh its retained measure chord list, member metadata and
geometry. This follows `AbstractChordInter.invalidateCache/remove` and
`Measure.addInter/removeInter`; partial deletion no longer leaves copied bounds
or removed chord members. The stage report at `limited_links.containers` (CLI JSON:
`symbols.limited_links.containers`) also projects
original `AbstractNoteInter.added/remove` staff membership from the live SIG in
registration order. It is a derived projection, not an independent Rust Staff
object or complete editing implementation. Stale identity/geometry is refused
atomically. Live empty chords from direct relation editing remain pending.

Four saved original removal conditions now also match actual Staff notes and
Measure RestChord registration. The same-image weak-rest regression checks that
deleted IDs disappear from containers and cached stem objects remain identical.
Existing image MusicXML output is separately verified; there is no new notation
admission or new image fixture. Voice removal, lyrics, full Part/stack ownership
and cached extensive head/stem removal remain pending. All global completion
flags stay false.
[Validation](../evidence/musicxml-note-containers/VALIDATION.json).

Use unused output paths with the existing Linux CLI, for example:

```sh
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/note-containers.musicxml /tmp/note-containers.json
```

The complete `InterCleaner.purgeContainers` is separately pending: pinned Java
`Part.purgeContainers` specifically purges removed lyric lines, not these chord
collections. Lyrics recognition/ownership must be connected before claiming it.

### BeamHeadCleaner retains original contextual grades

Original `org.audiveris.omr.sig.BeamHeadCleaner.process` only removes temporary
BeamHead relations. Native's additional contextualization incorrectly changed
head grades after this ordered LINKS stage. The actual cleaner now removes only
validated edges, keeping vertices, intrinsic/contextual grades and identities.
A later explicitly ordered contextualization still works. One original declared
SIG lifecycle reproduces the old mismatch and verifies the fix/idempotence;
existing image MusicXML regressions are checked separately. This is not complete
post-LINKS raster SIG equality or full cleaner/epilog parity.
[Validation](../evidence/musicxml-beam-head-cleaner/VALIDATION.json).

Source`238c535` passed the complete593-test workspace run (176 binaries+7 empty
doc-test groups), failed0/ignored0, after the rest admission/ownership/containers
and cleaner group. Source was fixed during execution. Independent clone built
the native Linux CLI;20+11 events still match original/prior XML bytes and XSD.
Original full590 at`322135d` remains an earlier source-specific record.
[Full validation](../evidence/musicxml-beam-head-cleaner/FULL-VALIDATION.json),
[readback](../evidence/musicxml-beam-head-cleaner/READBACK.json).

The next post-chord stem-removal observation is original-only preparation:
`fixtures/official-stem-chord-geometry/post-stem-removal.json` shows a HeadChord
remaining a HeadChord without a stem, with refreshed head-only bounds/locations.
Native cached post-chord callback and class propagation are not implemented by
this observation. [Dependency](../evidence/musicxml-beam-head-cleaner/NEXT-DEPENDENCY.md).

### Cache-owned stem removal after CHORDS

Original `ChordStemRelation.removed` now invalidates the surviving HeadChord
inside the retained retriever's staged SIG transaction, before incident edges
are pruned. HeadChord and RestChord class is carried separately from the optional
stem. Pre-rhythm container refresh reads the actual typed ChordStem relation; a
stemless HeadChord keeps its head membership, head-only bounds and note locations
and stays in the measure's head-chord list.

The saved original one-condition post-stem observation matches native before/after
geometry, grades, members and live counts. Native uses an actual image-backed
registry glyph and canonical cache Arc; callback failure is atomic, repeated
removal is idempotent, and the physical registry is retained. This is a declared
lifecycle comparison without HeadStem relations, not automatic raster parity
of that branch. The original fixture's `native_connected:false` records its
earlier preparation checkpoint. Ordinary image/whole-breve guards are retained.
Voices, lyrics, complete Part cleanup and general epilog remain pending.

Existing executable example (use fresh output names):

```sh
cargo build --offline --locked --bin audiveris-musicxml -j1
audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png out.musicxml out.json
```

[Validation](../evidence/musicxml-post-chord-cache/VALIDATION.json).

### Extensive HeadChord removal preserves cached stems

Original `AbstractInter.remove(true)` now removes an actual HeadChord's
unshared Head members through the existing staged music-removal path. Members
contained by another live chord remain. A stem is linked by ChordStemRelation,
not Containment, so it stays registered in the same retriever/cache/physical
registry. Removed HeadStem relations update only surviving endpoints' abnormal
state, matching the original callback. Frozen-member refusal remains atomic.

Two declared original controls compare exclusive and shared Head membership,
remaining SIG relations/geometry/grades and pre-rhythm staff/measure container
projection. Native's stem grade is independently assessed from an actual image
glyph and preserved; the original control's declared intrinsic0.8 is not claimed
as builder parity. This does not certify raster generation of this deletion
branch, Voice ownership, lyrics or complete cleaner/epilog. Existing image CLI
examples remain available; no new image admission guard is lifted.

[Validation](../evidence/musicxml-head-chord-removal/VALIDATION.json).

### Retained single-voice chord ownership

The admitted ordinary one-staff/single-voice image path now retains Voice objects
with actual chord IDs, starting staff and owning measure. `Voice.addChord`
deduplication/center ordering and original first-staff HIGH voice ID generation
feed event/SIG voice fields and the MusicXML writer; the writer no longer supplies
its own fixed voice label. CLI JSON adds `voices` for inspection.

Original `Measure.removeVoiceChord`/`Voice.removeChord` is consumed by actual
container refresh: last-chord removal deletes that voice, while another chord
keeps the same voice. Two original stored-voice observations extend the existing
head-chord ownership regression; no new image/condition matrix is added. This
cleanup is a declared original/native retained-owner comparison. There is still
no post-rhythm CLI correction endpoint; general polyphonic matching, slots,
merged parts, beam/tie voice propagation, lyrics and complete epilog are pending.

Existing command/input examples above continue to apply. No image admission
guard is removed; existing image MusicXML comparison is distinct from these
controlled ownership operations.

[Validation](../evidence/musicxml-voice-chord-ownership/VALIDATION.json).

Full native workspace validation at source95e191b:595 passed,0 failed/ignored,176 executables and7 empty doc-test groups. A transport disconnect interrupted the initial run after136 executables; only40 remaining executables were resumed from the unchanged Cargo inventory (the interrupted4-test binary was repeated). [Full validation](../evidence/musicxml-voice-chord-ownership/FULL-VALIDATION.json). This is separate from full593 at source238c535.


## Solved chord clocks → retained stack slots (2026-10-06)

`StackRhythm.generateStackSlots` now gathers actual normal-Voice chords by their
resolved time, sorts slot incomings by center ordinate, assigns the real chord's
slot back-reference and writes `BEGIN`/`CONTINUE` entries using original
`Voice.completeSlotTable`. A chord ending exactly at the next slot is no longer
active there; missing chord offset marks the measure abnormal. The image CLI
calls this after timing its admitted single voice. JSON exposes
`measures[].slots` (actual incoming chord IDs) and `voices[].slots` (actual
chord ID/status); MusicXML examples and output bytes remain unchanged.

The existing original saved page is compared with the native image path. A
second declared Voice on the same original page checks continuation/equal-end/
missing-offset behavior separately: it does **not** establish automatic
polyphony matching. The existing one-part/one-staff admission still maps one
Measure to one stack. General shared multi-part stacks, geometric slots/
SlotsRetriever/ChordsMapper, whole-measure-rest Voice, Rational/tuplets,
lyrics/automatic ties/correction and complete epilog remain pending. All global
completion flags stay false. No extra image fixture or image-specific branch.

Run the existing example with new output paths:

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-cross-system/beam-two-systems.png \
  stack-slots.musicxml stack-slots.report.json
```

[Original/targeted validation](../evidence/musicxml-stack-slots/VALIDATION.json).
Full595 remains the result at source95e191b, not a new full run.


## Slot-based Voice/stack duration (2026-10-06)

`Voice.getDuration` reads the maximum end of its `BEGIN` slots; it is distinct
from the last-chord `end_time`. `MeasureStack.getSlotsDuration` reads all actual
slot incomings, skipping unknown slot times. `StackRhythm.readStackActualDuration`
retains zero for no Voices, the slot duration when positive, otherwise the
expected duration. These original computations now feed the admitted image
CLI's retained stack timing and consistency check; `voices[].duration` is added,
and `measures[].actual_duration` comes from this stored state. Existing example
commands/output bytes stay valid.

Five original saved/declaration states compare normal duration, a non-last
chord's maximum, unknown slot time, Voice-without-slots and no Voices. The
no-slot fallback does not implement measure-rest recognition/Voice construction.
Shared multi-part stacks, general Rational/voice matching/whole-rest Voice,
automatic slur/tie/lyrics and complete cleaner/epilog remain pending. All global
completion flags remain false. [Validation](../evidence/musicxml-slot-duration/VALIDATION.json).
Full595 remains at95e191b; this stage uses targeted tests.


## Live chord time propagation before slots (2026-10-06)

Original `AbstractChordInter.setAndPushTime/pushTimeViaVoiceSequence/
pushTimeViaGroup/pushTimeViaTie` now traverses actual `NextInVoiceRelation`,
selected beam-group chords and already-classified `SlurHeadRelation` ties, in
original sequence→group→tie order and within the exact owning Measure. The
native image CLI consumes this before final stack slots; propagated chord
clocks must agree with its admitted single-voice event clocks. Existing input
examples/output bytes stay valid; no image coordinates or event list supply
the result.

Original comparison covers one saved group and separate declared sequence,
classified tie, cross-measure and null-seed controls. Existing sequential
single-voice seeding and resolved exact-grid duration still apply. This does
not acquire automatic NextInVoice/SlurHead relations, classify/export ties,
solve general slots/polyphony or implement whole-rest/special-chord branches.
Full Rational and original warning/caught-exception paths beyond typed resolved
normal chords remain pending. Cyclic/unbounded propagation explicitly refuses
atomically with a budget, instead of claiming an original successful score.
All global completion flags remain false.
[Validation](../evidence/musicxml-chord-time/VALIDATION.json).
Full595 remains at95e191b; this stage uses targeted tests.


## Default whole-measure rest → MusicXML (2026-10-06)

One new independent Bravura44/interline11/zero-skew ordinary G-clef/C-major/4/4
single-voice image contains four quarters then one whole-measure rest. Original
5.11 exports all five events. Native previously recognized `WHOLE_REST` and
refused its unported meaning; it now connects `SystemInfo.isMeasureRestShape`,
`Voice.createMeasureRestVoice`, the no-slot stack duration and original
`PartwiseBuilder` rest export branch. The actual rest chord is kept separately
from normal Voice chords, with null intrinsic duration/end/termination and no
slots. MusicXML uses stack duration, `<rest measure="yes"/>` and omits note type.
Pitch/rest/type/Voice/duration/onset, actual rest bounds and stored Voice/stack
state are compared with the same original image; all XML engraving/layout
metadata parity is not claimed. Native does not read the original XML.

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-measure-rest/quarter-whole-rest.png \
  quarter-whole-rest.musicxml quarter-whole-rest.report.json
```

Output paths must not exist. The default is `partialWholeRests=false`.
Original shape classification under both switch values is a separate8-condition
kernel comparison, not two full image runs. Exposing processing switches,
`partialWholeRests=true` image processing, LONG/BREVE rest images, mixed
measure-rest/normal polyphony, all-rest/no-head pages, generic shared stacks/
slot-voice matching, lyrics/automatic slur-tie, correction and full cleaner/
epilog remain pending. Earlier limited successful examples remain supported;
original-failing central quarter-rest and half-rest boundary inputs retain
refusal. All global completion flags stay false.
[Validation](../evidence/musicxml-measure-rest/VALIDATION.json).
Full595 is the earlier95e191b result, not a new full run.


Slot geometry and read-only selection (2026-10-06): `Slot.computeXOffset` now
uses integer centers of actual incoming chords, their mean, and original ties-to-even
rounding of the stack-relative abscissa. Slot reports expose `x_offset` in pixels.
`MeasureStack.getClosestSlot` preserves time order for equidistant candidates;
a whole-measure-rest stack with no slots returns null. Original saved state for
20 slots in the existing two-system image and within-staff point queries match.
The separate whole-rest image covers the empty-stack boundary.

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  --slot-at 1 311 \
  fixtures/official-image-musicxml-cross-system/beam-two-systems.png \
  /tmp/slot-score.musicxml /tmp/slot-score.report.json
```

Output paths must not exist. The prefix also works before `--output-dir` and is
applied to each independent score. `slot_query` contains the selected actual slot,
its time and live incoming chord IDs; MusicXML and recognition remain unchanged.
Measure number is the one-based global report number. X is an image pixel
coordinate; the admitted single-staff measure explicitly fixes the reference
staff (`getXOffset(point, staff)`). This is a CLI wrapper for the original selection
logic, not an editing operation. General point-to-staff selection, multi-staff
interpolation, shared stacks, geometric `SlotsRetriever`/`ChordsMapper` Voice
assignment and correction persistence remain pending. No global completion flag
is advanced. Original scalar/saved-state comparisons and current automated
single-voice image admission are separate validation boundaries.


Rhythm reset ownership (2026-10-06): image rhythm now calls original
`Measure.resetRhythm` / `AbstractChordInter.resetTiming` before each current
measure pass. Only that measure's Voices, abnormal flag and real chord
slot/voice/time references are reset. Intrinsic chord duration, stack
slots/expected/actual timing, relations, other measures and physical registry
remain intact, as in the original methods. Final stack collection replaces
its slot list later in the original order. Reset/repeat on original saved
normal6 chords and whole-measure-rest1 chord is separate from automatic
correction recognition. The existing whole-rest and `--slot-at` examples
remain the runnable CLI checks. General MeasureRhythm narrow/wide slot
retrieval/matching, SmallChord, shared stacks and correction reprocessing
entry/persistence are still pending; no admission guard was removed.


Actual chord stem direction (2026-10-06): the original
`HeadChordInter.getStemDir` now reads actual integer head/tail locations.
It reports -1/up, +1/down, zero/no stem (or coincident locations). This is a
prerequisite of `SlotsRetriever` adjacency; general narrow/wide slot and Voice
matching is still pending. `PartwiseBuilder` stem semantics now apply to every
currently admitted stemmed note, including quarter/half notes without beam/flag.
The current one-head admission makes tail-versus-leading-head comparison equal
to the original per-note comparison; equality exports down when a stem exists.
Rest/stemless notes do not emit a stem. Actual geometry supplies direction,
while the existing live beam HeadStem identity guard is retained.

The prior whole-rest CLI example now emits four original-matching `<stem>up</stem>`
tags followed by the stemless measure rest. `chord_stem_direction` exposes the
original getter code in each event. Original saved real24-chord getters and
four declared up/down/leading-distance/stemless controls are recorded separately
from current image recognition. Existing MusicXML16 plus getter/Voice2 targeted
tests passed; subsequent test-only getter assertions reran2 without export or
geometry code changes. This is not a new full595 run. The new XML adds ordinary
note stem metadata; old report hashes remain evidence for their original source.
Full compound/mirror/special-head per-note engraving and stem default-y metadata
remain partial, alongside general Voice matching/shared stacks/lyrics/automatic
slur-tie/correction/full cleaner-epilog. All global completion flags remain false.


All-rest original export boundary (2026-10-06): one two-measure input with only
whole rests and no heads reaches original full batch export, but original5.11
emits `<divisions>0</divisions>` and fails MusicXML4 XSD. No additional original
run or position search was made. Existing native code already recognizes two
whole rests (four quarter beats each) and emits valid XML with divisions1; this
is a native observation, not a successful original MusicXML equality claim.
The input and both outputs are retained at
[evidence/musicxml-all-measure-rest](../evidence/musicxml-all-measure-rest/ORIGINAL-LIMIT.json).
Full all-rest original export parity remains unconfirmed. Next connected work is
[geometric narrow/wide slots and real Voice matching](../evidence/musicxml-all-measure-rest/NEXT-DEPENDENCY.md),
not more invalid-original export retries.

### Preliminary narrow/wide MeasureSlots (2026-10-06)

The image continuation now invokes original `SlotsRetriever` before assigning
Voice/time. It uses the real owned chords, member/stem geometry and typed SIG:
SeparateTime/NextInVoice, explicit SameTime, beam groups, mirrors, root-stem
alignment, SameTime intermediate extension, adjacency, close and distant pairs,
in that original order. Every candidate must be EQUAL or CLOSE to **all** current
slot members. Narrow/wide thresholds retain original scale rounding. Default
whole-measure rests are excluded; their dedicated Voice remains separate.

`preliminary_measure_slots` in the existing JSON report contains narrow/wide
candidate slots and oriented relationships using actual chord IDs. For example:

```sh
cargo build --offline --locked -j1 --bin audiveris-musicxml
/path/to/target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-cross-system/beam-two-systems.png \
  /tmp/measure-slots.musicxml /tmp/measure-slots.json
```

Use the existing low-debug build environment when working in the Cloud task.
The current single-voice path now takes its candidate order from narrow slots;
a slot with multiple incoming chords still refuses until real ChordsMapper
assignment is connected. This does not enable polyphony or expand image admission.
These candidates precede rhythm and differ from final timed stack `slots`.

Saved original successful image states (24 standard chords across six measures,
including an empty whole-rest candidate list) and five declared controls compare
relationship priority and partitions. Saved-state/method controls are separate
from same-image native recognition and successful original MusicXML comparison.
The production adapter is currently ordinary chords, no genuine augmentation
dots, zero skew and one staff; it uses existing live beam getters. Generic
relationship branches do not prove automatic mirror/root/cue acquisition.
Compound slots, SlotMapper/ChordsMapper/VoiceDistance, shared multi-part stacks,
full Rational/tuplets, lyrics, automatic slur/tie and correction/cleaner/epilog
remain pending. All global flags remain false; the full595 result belongs to
`95e191b`, not this source. No nonUI logic is planned for omission.

### Compound slot membership (2026-10-06)

The continuation now connects `MeasureRhythm.buildCompoundSlots` and
`Slot.CompoundSlot.chordsOf/getNarrowSlot`: narrow/wide chord-set intersections
partition narrow members, chord centers determine stable ordinate order, and
lookup takes the first containing narrow member. The original final-range cursor
behavior is preserved, including the independently declared overlapping-wide
control; that control is not a successful image/Voice assignment claim.
`preliminary_measure_slots[].compounds` exposes actual chord/narrow-slot identities
and lookup. The CLI example above is unchanged.

The current single-voice continuation now takes its candidate sequence from these
compound members. Multiple chords in a compound still require general
`SlotMapper/ChordsMapper/VoiceDistance`; they refuse instead of being serialized
as the same voice. No input image scope was enlarged. Normal saved pages, an empty
measure-rest candidate list, two narrow members, ordinate order/lookup, the
original overlapping cursor and empty input are checked separately. Original
transient candidate-slot setter identity/timing and iterative mapping remain
partial; final timed stack slot ownership is retained. Next is actual starting
Voice construction and zero-time propagation before rookie matching. Full595
remains the old `95e191b` result and all global flags remain false.

### Starting Voice and original zero-time propagation (2026-10-06)

The image path resolves current admitted intrinsic chord durations from actual
shape/flag/beam links **before** starting rhythm. It then follows original
`MeasureRhythm.processStartingChords`: default measure rests plus the first
compound's chords in center-ordinate order -> starting Voice -> real setVoice
beam/tie propagation -> `setAndPushTime(0)`. The current one-voice admission
requires one starting chord. Subsequent chords enter the Voice through actual
assignment; the Voice no longer prepopulates every chord in the measure.

`preliminary_measure_slots[].starting_state` records this real intermediate
state. On the existing two-system beam example, the first four eighth chords
have Voice1 and offsets 0,1/8,1/4,3/8; the following two quarter chords initially
have no Voice/time. The dedicated whole-rest Voice has no ordinary chord list,
unknown intrinsic duration and starting time0. Both intermediate states match
unmodified original getters from the same saved successful inputs. The existing
CLI commands/images and successful MusicXML remain valid.

Previously propagated times are checked against the admitted event clock,
not overwritten silently. Current normal duration support/guards remain bounded
(no genuine dots/tuplets/special mirrored durations, at most two ordinary beams
or one flag); this reorganizes existing recognition logic rather than expanding
image admission. Later rookie/active mapping still uses the bounded single-voice
continuation; original general SlotMapper/ChordsMapper/VoiceDistance, multi-Voice
starting IDs/kinds, shared stacks, correction and complete epilog remain pending.
Full595 remains tied to `95e191b`; current validation is targeted MusicXML, not a
new whole-workspace result. All global completion flags remain false.

### Original rookie-to-active Voice mapping (2026-10-06)

Later unassigned chords now follow original `SlotMapper.readSlotTimes`,
`purgeRookiesSet`, the admitted `retrieveActives` branch, separated
`VoiceDistance`, `ChordsMapper.processNext/process` and `InjectionSolver`.
Costs use real owned Voice/staff/member/stem and absolute diatonic pitch, not
image identity or an event-counter choice. Explicit NextInVoice preassignment
precedes black-list rejection; the exact small injection search retains original
strict-improvement/tie order and the 60 no-link cost. A later rookie enters the
chosen actual Voice and receives the active chord's end time through real
`setAndPushTime`; the existing serial event clock is now a consistency check.

`preliminary_measure_slots[].mapping_steps` exposes known times, unassigned
rookies, eligible active chords, original costs/pairs and the resulting clocks.
The unchanged examples above still run without Java. Two saved successful
original pages provide 24 observed compound steps and 13 mappings. Three solver
controls and two constraint-list controls are method comparisons, **not** image
polyphony or automatic acquisition of new constraints.

This connects only the existing one-incoming/single-Voice image path. Both
comparison books use original `implicitTuplets=false`. Generic extinction,
retry/blacklisting, new-Voice time inference, `VoiceDistance.Merged`, implicit
shrinking, multi-note/cross-staff mapping and general polyphonic export are
pending; unsupported cases refuse instead of inventing a time/Voice. Exhaustive
search has an explicit visit budget and refuses exhaustion rather than returning
a suboptimal result. No input admission or global completion flag changed.
The original invalid-XSD all-rest page remains a separate negative boundary.
Full595 continues to refer only to `95e191b`; current checks are targeted.

### Measure-local Voice extinction and explicit sequence exception (2026-10-06)

After each admitted compound is mapped, the image path now follows original
`MeasureRhythm.purgeExtinctVoices`: unknown first-member time makes no decision;
unknown last-chord time marks the real measure abnormal; a normal Voice whose
last end is strictly earlier than this slot expires unless its same-measure
NextInVoice sequence reaches the current or a later candidate slot. Comparison
uses actual candidate abscissae, not numeric slot IDs. Default whole-rest Voices
are skipped. Extinct identity is held per real measure and used by the next
`SlotMapper.retrieveActives`: only an explicit NextInVoice pair to a current
unassigned rookie keeps an extinct last chord eligible.

The existing CLI command and image examples are unchanged.
`preliminary_measure_slots[].extinction_steps` records current position,
first-member time, extinct Voice IDs and actual abnormal status. The two saved
successful original pages match at all24 normal compounds; **these images have
no extinction**, so they do not demonstrate automatic multi-Voice gaps.
Seven declared original-object controls separately check a gap, equal-end
boundary, a sequence through an earlier chord to a later one, a sequence ending
before the current slot, the pre-extinct explicit-rookie exception, and unknown
last/first clocks. The controls do not enable polyphonic image/export admission.

For the currently guarded one-chord narrow member, its known clock equals that
actual chord's clock. General narrow-slot setter identity/retry, multiple
members/Voices, implicit tuplets, `Merged`, shared stacks and correction remain
pending. Invalid cyclic sequences explicitly refuse atomically rather than hang;
no claim is made that the original accepts them. No image scope or global
completion flag changed; full595 is still the old `95e191b` result. The original
all-rest invalid-XSD page stays a separate negative boundary.

### Retained transient narrow-slot clocks (2026-10-06)

The current mapper now holds real, initially unknown narrow-slot clocks separately
from chord clocks and final stack slots. They are owned by the current measure;
compound views addressing the same narrow ID share that clock. Original
`mapChords.readSlotTimes` sets a single known narrow time before purging beam/tie
rookies. Original `checkSlotTime`'s single inferred-time branch sets an unknown
narrow clock from the selected active end **before** `applyMapping` assigns the
Voice and pushes chord time. An empty time set does not reset a retained clock.
Conflicting or unsupported inference still refuses, without a guessed minimum.

`purgeExtinctVoices` now reads the retained first-member slot clock, replacing
the previous admitted one-chord proxy. `preliminary_measure_slots[].candidate_time_steps`
shows narrow getter values before/after each mapper. Original same-image saved
getters match24 compounds/six measures: starting beam chord times can already be
known while narrow slot times are still null; after mapping, narrow values become
0, 1/8, etc. The default whole-rest measure has no narrow-clock entries. The
existing image CLI commands and output examples are unchanged.

This is the existing one-incoming/non-implicit-tuplet path. General multi-time
`analyzeTimes`, blacklisting/retries, new-Voice/sibling-time inference, full
Rational/tuplets, shared stacks and actual candidate chord-slot reference/setter
identity are not complete. No new image admission, no global true flag, no new
whole-workspace claim; full595 remains `95e191b`. Previously saved method-control
and XSD-negative evidence remain distinct from successful image MusicXML.

### Known-time mapping retries (2026-10-06)

The live mapper now follows nonimplicit `mapRookies` rejection/reselection before
assigning a real Voice/time. With a known narrow time, an incompatible active end
is blacklisted and another injection is solved; a known-slot chord ending beyond
expected duration is rejected unless explicit NextInVoice pins it. A repeatedly
forced contradictory Next pair stops without applying its mapping. Exhausted
choices remain unassigned; the CLI still refuses pending new-Voice inference.
`preliminary_measure_slots[].mapping_validation_steps` exposes the resulting
black list and whether a proposal was applied; existing CLI commands are unchanged.

Five original saved-object controls compare changed selection, exhaustion,
late ending, explicit late-end exception and forced time conflict. These are not
image polyphony or successful overfull MusicXML. Normal image admission remains
one incoming/one Voice/nonimplicit binary rhythm. Multiple inferred times,
`analyzeTimes`, new Voices, full Rational/tuplets and shared stacks are still
pending, with all global flags false. Full595 remains the old `95e191b` result.

### Conflicting inferred narrow times (2026-10-06)

Nonimplicit `MeasureRhythm.inferSlotTimes/checkSlotTime/analyzeTimes` now shares
the live proposal validation path. When mapped candidates infer different times
for one previously unknown narrow member, the earliest time is retained, later
choices are blacklisted, and selection retries before Voice/time assignment.
It does not force a conflicting chord into a Voice. Exhausted rookies stay
unassigned; pending new-Voice inference still causes the current CLI to refuse.

Two controls built with original saved chord/Voice objects compare equal inferred
ends and conflicting ends, including proposal cost, blacklist order, remaining
rookies and retained slot clock. These are method controls, not automatic image
polyphony. Existing CLI commands and the three 36-event image examples above
remain the usable scope; their MusicXML must remain unchanged. Implicit tuplets,
unknown-clock abnormal propagation, general new Voices and shared stacks are
still partial. All global completion flags remain false; full595 is the historical
`95e191b` result, not a rerun of this source.

### New Voice for an unmapped chord with a known time (2026-10-06)

The live mapper now follows `SlotMapper.createNewVoices` for an unassigned chord
whose own time is known: `Voice`/`Measure.generateVoiceId` selects the first unused
HIGH ID, and the same retained constructor/`setVoice` path owns the actual chord
in SIG. Already-voiced candidates create no additional Voice. The known chord
time is preserved. Positive native ownership is checked on the image-derived
Graph against two declared original-object controls, not only a serialized
proposal. No new recognition input or polyphonic MusicXML success is claimed.

Existing CLI commands and all three 36-event examples remain unchanged. The
current single-Voice writer explicitly refuses a new Voice rather than overwriting
its owner with the initial Voice ID. EQUAL/closest-sibling time inference,
previous-slot merge, general Voice export, other Voice kinds, tuplets/shared stacks
and complete correction/cleaner remain dependencies. All global flags stay false;
full595 still refers to the historical `95e191b` run.

### Unknown active clock and abnormal state (2026-10-06)

Original `inferSlotTimes/applyMapping` can assign an existing Voice while leaving
a chord time unknown and marking its measure abnormal. The native mapper now
preserves that state instead of discarding the original behavior. If the narrow
time is already known, an unknown active end stops before applying the mapping;
remaining unknown-time new-Voice inference stays explicitly pending.

Two original method controls compare both branches, including Voice membership,
null clocks, unassigned rookies and abnormal state on an image-derived native
Graph. These are negative/partial timing states, not successful MusicXML pages.
The writer refuses an unresolved actual chord clock rather than synthesizing a
serial onset. The three successful 36-event CLI examples remain unchanged; no
new image acceptance is claimed. Sibling/previous-slot inference, general Voice
export, full Rational/tuplets and shared stacks still need connection. Global
completion flags remain false; full595 remains the old `95e191b` result.

### Sibling time selection for a new Voice (2026-10-06)

`createNewVoices` now creates the actual retained Voice before reading a chord's
clock, as in Java. An unknown clock uses original `getOrderedSiblings`: EQUAL
relations take precedence, ties preserve compound order, other siblings are
ordered by horizontal distance, and unknown sibling clocks are skipped. The
first known sibling clock propagates through the existing actual SIG time path.
Two original saved-object controls compare farther EQUAL selection and skipping
unknown EQUAL candidates, including ordered siblings, Voice members and times.
The same native creation method is called by the image mapper; these positive
multi-chord controls are not automatic polyphonic image/export success.

Existing one-incoming image guards and the three 36-event CLI commands remain.
An isolated unknown-time chord still refuses pending previous-slot merge; the
single-Voice writer still refuses other Voice owners and unknown actual clocks.
Current CLI examples do not exercise the positive sibling branch. General
compound/candidate-slot identity, previous-slot merge, multiple-Voice export,
exact Rational/tuplets and shared stacks remain dependencies. No global flag is
true and full595 remains historical `95e191b`.

### Retained candidate-slot references and positions (2026-10-06)

The native mapper and Voice extinction now share transient chord-to-slot bindings
and constructor-cached positions. A later constructor rebinds a shared chord;
changing an earlier slot's member list does not recompute its position or rebind
that chord. Removing a slot from an ordered view does not erase its retained
reference. Stable constructor keys are separate from mutable slot numbers and
from narrow clocks/final timed stack-slot IDs. The position adapter remains the
existing zero-skew one-staff translation.

Two original actual-object constructor/list-mutation controls compare bindings,
cached positions and `Slot.compareTo` results on an image-derived native Graph.
These are getter/reference controls, not full `mergeWithPreviousSlot` or image
polyphony. Existing CLI commands and three 36-event examples remain unchanged.
Ordered-slot membership/removal/renumbering, general compound Voice mapping/export,
exact Rational/tuplets and shared stacks still need connection. All global flags
remain false and full595 still refers to old `95e191b`.

### Previous-slot merge and mutable candidate traversal (2026-10-06)

When a newly owned Voice has an isolated chord with no clock, native now follows
`MeasureRhythm.SlotMapper.mergeWithPreviousSlot`: it joins the previous compound
only if a previous EQUAL chord has a known clock. It propagates that actual time,
removes the first equal compound from ordered candidates and renumbers later ones.
Original `Slot.equals` uses the cached abscissa, not the numeric ID or identity;
`List.remove/contains` retain that behavior even for distinct equal-position objects.
The removed object remains addressable; the rookie's constructor reference and
cached abscissa do not move to the previous slot. Narrow member lists stay separate.
The image mapper now walks this mutable order before writing events, skipping
extinction on removed slots and continuing at the same index after removal.

Four original saved-object controls compare ordered membership/IDs, retained
references/positions, actual Voice members and clocks. The positive comparison
calls original `createNewVoices`; two negatives and an equal-position boundary
compare `mergeWithPreviousSlot` with
an already declared new Voice. This is not new automatic polyphonic image/export
admission. Existing CLI examples remain the three 36-event inputs, with their
commands above; no new image or switch is required. Unknown nonmatching clocks
still refuse in the bounded writer. General compound/multiple-Voice export,
exact Rational/tuplets, Merged/shared stacks and the remaining nonUI functionality
stay in the port ledger. All completion flags remain false; full595 belongs to
old `95e191b`, not this stage.

### Grouped chord clocks per narrow member (2026-10-06)

The image mapper's initial known-clock read now follows original
`MeasureRhythm.SlotMapper.mapChords/readSlotTimes` for each narrow member: one
known value sets that member's clock, an empty read retains its prior clock, and
multiple known values within one member report inconsistency without setting a
clock. Different members may have different clocks. This nonimplicit pre-mapping
phase has no mapping pair to blacklist. Three original `mapChords` controls
with already-owned actual chords isolate this phase and compare the native
owned Graph/narrow clock results. They do not exercise active matching/export.

The bounded image mapper still rejects general multi-incoming compounds and
grouped conflicts; it does not infer a success from the false consistency result.
The three 36-event CLI examples and commands above remain unchanged. The next
dependency is general compound incoming mapping and multiple-Voice traversal,
then exact Rational/implicit tuplets and shared stacks. All global flags remain
false, no original nonUI omission is planned, and full595 remains old `95e191b`.

### Multiple rookie assignment on owned Voices (2026-10-06)

The existing actual Graph mapper now consumes a whole compound's incoming
ordinary chords, rather than one rookie: original separated distance/injection
order, member-specific clock checks, blacklist/retry, accepted Voice/time
assignment and leftover new Voice creation share the same ownership path. Three
original saved-object `SlotMapper.mapChords` controls compare agreeing ends,
conflicting ends in one member, and distinct ends in separate members. Native
checks the retained candidate, actual measure/chord ownership and member coverage.

These controls reuse the existing two-system image's recognized Graph but declare
the compound members; they are not a new polyphonic image example. Current CLI
commands and three 36-event outputs stay unchanged. Image starting-parallel and
multi-incoming admission and multiple-Voice export still refuse; the next step
is solved Voice stack-slot traversal and original backup/forward handling. Known
grouped conflicts, exact Rational/implicit tuplets, Merged/shared stacks and the
remaining nonUI functionality stay incomplete. No original logic omission is
planned. Global flags remain false; full595 belongs to old `95e191b`.

### Owned Voice/stack traversal in the MusicXML writer (2026-10-06)

The public Linux CLI writer now follows `PartwiseBuilder.processMeasure` over
retained Voices and final stack slots, emitting notes only at BEGIN entries.
At a Voice boundary it backs up the actual preceding cursor; before a later
chord it inserts a forward with that Voice number. Movement durations join note
durations in selecting exact binary divisions, so a 1/8 gap between quarter
notes is preserved. Measure-rest Voice uses its dedicated chord and actual stack
duration. Chord and event ownership are checked before export; unresolved clocks
and duplicate/missing events refuse.

Four original saved-object controls reach actual `PartwiseBuilder.build`: the
three compound cases plus an overlapping 1/8-offset case with positive CONTINUE
entries. Native compares final slot tables, exact note/backup/forward order,
normalized durations and normalized XML body bytes. Wrapped normalized bodies
also pass MusicXML4 XSD. This does not claim full original engraving equality or
a new automatically recognized polyphonic page. Existing three 36-event CLI
examples and commands above remain the image scope.

To run the owned control comparison after cloning, use
`cargo test --offline --locked --lib multiple_rookies_retain_original_assignment_retry_and_voice_ownership`
(or omit `--offline` when dependencies are not yet available). The next image
dependency is original multi-starting-chord ownership, followed by actual
multi-Voice event/onset construction and all-Voice duration checks. Starting,
compound image admission and serial event guards remain until those dependencies
and a same-image original comparison agree. Exact Rational/tuplets, Merged/shared
stacks, lyrics and remaining nonUI features stay partial. All flags stay false;
full595 remains historical `95e191b`.

### Selected starting-chord constructor loop (2026-10-06)

The image CLI now calls the owned original `processStartingChords` constructor
loop after selecting measure rests/first compound and applying its one-start
admission. The loop sorts actual bounds by center ordinate, creates a retained
Voice for every selected seed and calls `setAndPushTime(0)`. Original starting
construction is unconditional; it is different from later rookie purging.
Two original actual-object controls compare parallel starts and an already-owned
start. The latter retains an old member record in the original raw state; native
keeps that state for parity but refuses its inconsistent export ownership.
Rookie new-Voice creation still refuses an already assigned seed.

Run the new controlled comparison with
`cargo test --offline --locked --lib starting_voice_constructors_preserve_original_parallel_order_and_reassignment`
(or omit `--offline` before dependencies are available). Existing image commands
and the three 36-event outputs remain the public CLI examples. These raw controls
do not add a polyphonic page or validate multiple starts with beam/tie propagation.
Next connect event assembly to actual mapped Voice/onset/duration and all-Voice
duration checks, then a same-image original parallel-Voice result before opening
image admission. General Rational/tuplets/Merged/shared stacks/lyrics/full cleaner
and the remaining nonUI ledger stay partial. All flags remain false; full595
still refers to historical `95e191b`.

### All-Voice measure final check (2026-10-06)

The image CLI now runs original `MeasureRhythm.finalCheck` after final stack
slots, checking actual standard chord Voice/time and calling `checkDuration`
for every retained Voice when expected duration is known. Five sequential
original saved-object observations compare both terminations, later-only excess,
persistent excess, a missing nonlast clock and unknown expected duration.
Underflow alone is accepted by this original phase; the current complete-measure
image guard remains separately enforced. These controls do not add an automatic
polyphonic image. The existing three CLI examples/36 events remain the image
comparison scope and retain their XML bytes.

Run `cargo test --offline --locked --lib multiple_rookies_retain_original_assignment_retry_and_voice_ownership`
for the owned mapping/writer/final-check comparison. Next bind event metadata to
actual Graph Voice/onset/duration before a same-image parallel-Voice validation.
All remaining nonUI features are still in the ledger, without planned omissions;
global flags remain false and full595 refers only to historical `95e191b`.

### Actual chord timing and Voice event binding (2026-10-06)

Image-generated pitch/header/beam metadata now receives Voice, onset and duration
from its actual retained SIG chord after final slots/stack duration, following
`PartwiseBuilder.processMeasure/processNote`. Event assembly no longer reassigns
Voice or overwrites a mapper clock. Normal duration is the actual chord getter;
measure-rest duration is the computed stack duration. Nonmember events and
unknown clocks refuse without changing the event.

The same binder now feeds the existing four original saved-object compound
mapping/Voice writer comparisons, including a new Voice and a fractional forward.
Run the owned comparison command above. Existing three image CLI examples are
retained; their 36 events/previous XML bytes/XSD are checked on independent Git
retrieval. No new automatic polyphonic image is claimed: compound/starting/
one-member/one-Voice/serial-clock/complete-measure image guards remain until
a same-input original full recognition/export verifies their dependencies.
General Rational/tuplets/Merged/shared stacks and the rest of the nonUI ledger
remain partial. Full595 is historical `95e191b`, not a new run.

### Parallel quarter Voices from an actual image (2026-10-06)

The existing original Bravura/staff renderer supplies one independent two-measure
4/4 image with distinct upper/lower quarter-note heads and stems. Unmodified
Audiveris5.11 full image recognition/export succeeds. Native Linux CLI now uses
actual compound mapping and ordinate-ordered starting Voice constructors, reads
Voice/onset/duration from its actual Graph and writes16 matching notes, with
a four-quarter backup between Voices in each measure. Native/original XSD,
musical event order and actual chord/slot/Voice geometry/timing agree; original
engraving/whole XML byte equality is not claimed. Original nonpersisted Voice
termination is recomputed by finalCheck after loading the recognized saved state.

Try this example with no Java runtime:

```sh
mkdir -p work/cli
cargo run --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-parallel-voices/parallel-quarters.png \
  work/cli/parallel-quarters.musicxml work/cli/parallel-quarters.json
```

The image admission cursor and horizontal overlap check are per actual Voice;
the final check visits all retained Voices. Each Voice must still be complete
and have exact known clocks. Omitting the final lower Voice quarter in the renderer
keeps the upper Voice complete but must refuse the incomplete lower Voice; this
is a native admission check, not a claim that the modified image fails in Java.
Run `cargo test --offline --locked --test musicxml_limited actual_parallel_voices_preserve_original_events_and_check_each_voice_duration`
(or omit `--offline` if dependencies are not cached).

Parallel admission currently covers ordinary one-member quarter head chords in
4/4 on the existing normal one-staff/nonmerged Bravura44/47 path. Only this one
parallel image is verified. Other parallel durations, beam/flag/rest/gap/pickup
and tuplet families, multi-member chords, other Voice kinds/Merged/shared stacks
and remaining nonUI features need their dependencies/comparisons. The existing
three36-event examples retain their XML bytes. No final completion flags are
raised; full595 still belongs only to historical `95e191b`.

### Known late Voice forward from an actual image (2026-10-06)

A second small parallel-quarter input omits the first lower quarter at rendering
time. Unmodified original5.11 full image/export succeeds with15 notes and a
quarter-duration `forward` for Voice2 in measure1. Native creates that Voice
through the existing mapper, consumes its known1/4 onset and emits the matching
forward. No rest or clock is invented. Each Voice cursor now advances to a known
onset as in `PartwiseBuilder`; backwards clocks, horizontal overlap, unknown
clocks and incomplete Voice ends still refuse.

Use the same command as above with `parallel-late-voice.png` as input and distinct
output names. The existing parallel test now covers both positive images and
the incomplete-lower-Voice boundary; no new test function was added. The raw
original saved-state comparison still explicitly recomputes transient finalCheck
after reload, distinct from the original full image/XML evidence.

Parallel family remains ordinary one-member quarter heads in4/4. Only these two
parallel images are verified; beam/flag/rest/other-duration parallel families,
general rhythm/gap repair, tuplets/Merged/shared stacks/multi-member chords and
all remaining nonUI functions remain partial. The prior36 events retain XML
bytes and the earlier16-event parallel image remains matching. All global flags
stay false. Full595 is the old `95e191b` result.

## Parallel beamed eighth Voice from an actual image (2026-10-06)

The same two-measure Bravura/ordinary-staff4/4 image succeeds in unmodified
original5.11 and the native Linux CLI: upper Voice has four beamed eighths and
two quarters in measure1; lower Voice has four quarters. Measure2 has parallel
quarters. All18 notes, original begin/continue/continue/end, Voice-major backups,
actual chord/slot/Voice ownership, onset/duration and duration checks agree.
Native computes from pixels and its owned SIG; original saved-state getters and
full original image/export are separate evidence, not runtime input.

This connects existing AbstractChordInter.getDuration, setVoice/getBeamGroup and
setAndPushTime/pushTimeViaGroup, MeasureRhythm compound/rookie mapping and
PartwiseBuilder Voice traversal for positive parallel single-level beam heads.
Other parallel flags/higher beam levels/rests/meters, multi-member head chords,
shared/Merged stacks, general Rational/tuplets and remaining nonUI functions are
still partial. Unknown/backwards/overlapping time and incomplete Voices continue
to refuse. Previous67-event inputs are retained; completion flags remainfalse.

```
mkdir -p work/cli
cargo run --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-parallel-voices/parallel-beam.png \
  work/cli/parallel-beam.musicxml work/cli/parallel-beam.json
```

Output paths must be new. Java is needed only for the development oracle, not
for this native CLI. See [validation](../evidence/musicxml-parallel-beam/VALIDATION.json).

## Parallel two-level sixteenth Voice from an actual image (2026-10-06)

One further original-successful image has four upper beamed sixteenths followed
by three quarters against four lower quarters; measure2 retains parallel
quarters. Native19 notes, both beam levels, divisions4, actual Voice/chord/group
membership, slot clocks, duration/finalCheck and normalized MusicXML agree with
original5.11. This joins the same original owned group/duration/compound-mapping
path; no image-specific branch or guessed time is introduced.

```
mkdir -p work/cli
cargo run --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-parallel-voices/parallel-sixteenth.png \
  work/cli/parallel-sixteenth.musicxml work/cli/parallel-sixteenth.json
```

Parallel one/two-level ordinary beams in4/4 are now admitted. Parallel flags,
rests, half/whole-note heads, other meters, shared/Merged stacks, chord ensembles
and tuplets still need their own dependencies and image comparisons. These are
remaining work toward full nonUI parity, not planned omissions. Previous outputs
and all incomplete/unknown/overlap guards remain; global completion staysfalse.
See [validation](../evidence/musicxml-parallel-sixteenth/VALIDATION.json).

## Parallel flagged eighth Voice from an actual image (2026-10-06)

A third new original-successful image has two flagged upper eighths plus three
quarters against four lower quarters, then parallel quarters in measure2.
Native17 notes, exact durations/onsets/Voice/stem, backups and no beam tags match
original5.11 MusicXML. Actual chord/slot/Voice membership and original
HeadChordInter.getFlagsNumber/getBeamsOrFlagsNumber/intrinsic duration getters
agree. This is full same-image recognition/export evidence plus separately
identified saved-owner getter evidence; declared mapper controls remain separate.

The adapter now retains the already resolved actual beam/flag count beside the
intrinsic duration/type, so parallel admission consumes the same original
getDurationSansDotOrTuplet calculation as clock propagation. Ordinary quarter,
single-flag eighth and one/two-beam eighth/sixteenth heads in4/4 are admitted.
No coordinates, fixture identity, new Voice guesses or original XML are used.
The existing erased-flag refusal and incomplete lower-Voice refusal remain.

```
mkdir -p work/cli
cargo run --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-parallel-voices/parallel-flag.png \
  work/cli/parallel-flag.musicxml work/cli/parallel-flag.json
```

Only development oracle tools require Java. Use new output paths. Parallel
half/whole-note heads, mixed rests, higher flags/beam levels, other parallel
meters, multi-member chords, shared/Merged stacks, general Rational/tuplets,
lyrics/slur/tie and full cleaner/correction/export remain partial or unconnected.
These stay in the complete nonUI migration plan; all completion flags remainfalse.
See [validation](../evidence/musicxml-parallel-flag/VALIDATION.json).

## Parallel half-note Voice from an actual image (2026-10-06)

An original-successful two-measure image has two upper void-head half notes
against four lower quarters, then parallel quarters in measure2. Native14 notes
match original5.11 pitch/Voice/duration/onset/stem/backups and XSD. Original
actual chord/slot/Voice ownership, standard-chord intrinsic duration getters,
stack duration and all-Voice finalCheck agree. Group shortening is zero for
these plain half heads; their original shape-based half duration now joins the
same active-end/compound mapping and owned event binding used by other heads.
No fixture identity, coordinate or clock guess is used in native code.

```
mkdir -p work/cli
cargo run --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-parallel-voices/parallel-half.png \
  work/cli/parallel-half.musicxml work/cli/parallel-half.json
```

Nine retained CLI inputs now cover135 original-matching musical events. The four
new parallel images add68 notes: single-level eighth18, two-level sixteenth19,
single-flag eighth17 and half14; earlier67 XML bytes remain unchanged. This is
bounded clean synthetic Bravura/ordinary-staff/treble/C-major/4/4 comparison,
with two ordinary Voices. Original declared mapper controls and saved recognized
owner getters are identified separately from full same-image original exports.
General shared/Merged stacks, mixed rests, other parallel meters, multi-member
chords, whole/stemless/special heads, higher flags/beam levels, general Rational/
tuplets, lyrics/slur/tie and full cleaner/correction/export remain partial. No
original nonUI feature is planned for omission; all completion flags remainfalse.
See [validation](../evidence/musicxml-parallel-half/VALIDATION.json).

## Same-stem multi-head chords (2026-10-06)

```sh
cargo build --offline --locked --bin audiveris-musicxml
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-chord-members/chord-members.png \
  /tmp/chord-members.musicxml /tmp/chord-members.json
```

Use unused output paths. The native Linux CLI requires no Java runtime. Four
two-note quarter chords and four single notes produce12 events, with B4 before
F5 inside each first-measure chord. The second head emits `<chord/>` and shares
its actual chord's Voice/onset/duration; each measure advances four quarters,
not one quarter per head. No original MusicXML is read by the CLI. Original
full-image export, original saved ownership/getters, and native output/XSD are
compared separately. Normal Bravura/interline11/zero-skew/treble/C-major/4/4
input is validated; mixed mirrored/special heads and unverified chord beam/flag
combinations remain pending. All global completion flags remain false.

## Multi-head eighth beams (2026-10-06)

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-chord-members/chord-beam.png \
  /tmp/chord-beam.musicxml /tmp/chord-beam.json
```

This two-measure input produces14 notes. Each of the four beamed two-head
chords has one eighth-note duration, two member events at the same onset, and
original begin/continue/continue/end beam marks on **both** members. The actual
BeamGroup owns four chords, not eight note events; each ordinary Voice still
ends at one complete4/4 measure. Original full-image export/XML/XSD agrees;
reloaded original actual group/shortening/slot/member/Voice getters are separate
checks. The report's `events[].chord` derives from live ensemble order, agreeing
with `<chord/>`; it is not a recognition hint or ownership mutation.

Ordinary uniform black-head quarter chords and single-level beam chord combinations
are image-verified. Mixed mirrored void/black heads, shared/stemless/special
heads, other chord-flag configurations, shared stacks and nonbinary tuplets
still need their original dependencies and input comparisons. Global flags
remain false; prior successful CLI inputs stay supported.

## Two-flag sixteenth chords (2026-10-06)

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-chord-members/chord-flag2.png \
  /tmp/chord-flag2.musicxml /tmp/chord-flag2.json
```

Four two-head FLAG2 chords, three quarter notes and four next-measure quarters
produce15 native events. The flags are real classified glyphs with actual
FlagStem ownership and value2, not two guessed flag relations. Original
`HeadChordInter.getFlagsNumber` sums those values; each chord lasts1/16 and
advances once. `<chord/>` marks the second head; divisions4 expresses exact
durations. Original full-image pitch/type/Voice/onset/duration/stem/chord
stream and native XML/XSD match. A separately read original owned SIG control
checks multi-head Stem direction, first-valid flag lookup, grade and endpoint.

FLAG1 success and erased-flag refusal remain tested. The positive image check
here uses upward FLAG2; other multi-head flag combinations/positive downward
FLAG2 images, higher flags, mixed flag+beam and mixed mirror/special members
remain unverified or guarded. Other original nonUI features stay in the ledger.
Global completion flags remain false.

## True first augmentation dot (2026-10-06)

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-augmentation/augmentation.png \
  /tmp/augmentation.musicxml /tmp/augmentation.json
```

The first measure contains an actually recognized dotted B4 quarter (3/8),
a flagged D5 eighth and two quarters; the second contains four quarters.
The eight events match the unmodified original full-image MusicXML. The dot
retains its registered glyph and actual head relation; chord duration is
resolved before Voice clocks. A same-glyph CAESURA alternative is registered,
contextualized and purged by the original weak threshold. A surviving pause
still refuses. Separate saved original owner/getter and declared pause
controls are distinguished from full-image recognition.

At this first-dot checkpoint only ordinary unshared first-dot augmentation
was enabled. The following checkpoint adds double dots; shared/mirrored dot selection, positive rest/pause/repeat/staccato,
unsupported supported conflicts and durations finer than sixteenths retain
explicit guards. All other pending original nonUI features remain in the
ledger. Global completion flags remain false.

## Double augmentation dots (2026-10-06)

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-augmentation/double-augmentation.png \
  /tmp/double-augmentation.musicxml /tmp/double-augmentation.json
```

This eight-event input has a double-dotted B4 quarter (7/16), a FLAG2 D5
sixteenth (1/16) and ordinary quarters. Real second-dot→first-dot→head ownership
supplies the chord's dot count and 7/4 intrinsic multiplier. Voice advances
once, and Partwise writes two `<dot/>` elements. Original full-image musical
content is separate from saved original dot/clock/getter and declared weak
CAESURA/empty unlinked FLAT controls. Glyphs classified as weak unlinked flats
are registered and purged by the original threshold; positive or surviving
accidentals still refuse until their head/pitch/notation path is connected.

Ordinary unshared first/double dots are enabled only with exactly representable
sixteenth-tick durations. Shared/mirrored/rest/repeat/staccato dots, competing
first/second same-glyph interpretations, tuplets and full cleanup remain
partial. Erasing the real second dot refuses the incomplete measure; no
missing duration is invented. Global completion flags remain false.

## Inconsistent chord augmentation cleanup (2026-10-07)

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-augmentation/inconsistent-augmentation.png \
  /tmp/inconsistent-augmentation.musicxml /tmp/inconsistent-augmentation.json
```

A single dot attaches to one of two actual heads in the first chord. Original
`countDots` rounds the mean `[1,0]` to zero and removes the automatic dot.
Native admits this exact removed glyph using its owned cleanup receipt, then
exports12 undotted quarter-note events matching the original full-image output.
Separate original temporary-dot controls verify automatic deletion and manual
retention; frozen ownership loss still refuses. No absent candidate is guessed
away, and the first/double-dot successful examples remain supported.

General rest/shared/mirrored/competing dot interpretations, positive
accidentals/pauses, special heads, shared stacks, tuplets, lyrics/slur/tie and
complete cleaner/epilog remain partial; none are removed from the final nonUI
parity scope. Global completion flags remain false.

The follow-up `rest-augmentation-unconfirmed.png` is a negative comparison,
not an enabled dotted-rest example. Original emits no dots and numbers its
7/8 first measure0; native refuses earlier at an unconnected foundation
removal. Its XSD validity is not recognition success. See
`evidence/musicxml-rest-augmentation-limit/ORIGINAL-LIMIT.json`; the positive
rest augmentation path remains pending without changing image admission.

## Owned rest augmentation (2026-10-07)

```sh
./target/debug/audiveris-musicxml \
  fixtures/official-image-musicxml-augmentation/half-rest-augmentation.png \
  /tmp/half-rest-augmentation.musicxml /tmp/half-rest-augmentation.json
```

This separate original-successful image contains B4 quarter + dotted half rest
in measure1, then D5/B4 half notes in measure2. Native recognizes4 events and
writes a real RestInter-owned dot, half-rest type and three-quarter-note
duration. Original full-image music/XML/XSD and saved RestChord/Voice/slot
getters are verified separately. Rest lookup uses original bounds, center-right,
height margin and gap grades, with no image-specific coordinates in code.
Incoming AugmentationRelation has target coefficient zero: it cannot rescue
weak rests. Multiple/shared dot interpretations and orphan lifecycle remain
pending. The earlier negative quarter-rest image still refuses; neither its
original undotted result nor XSD validity is positive rest-dot evidence.
All overall completion flags stayfalse, and broader nonUI functions remain on
the full parity ledger.

Rest-loss orphan consistency (2026-10-07): the same successful half-rest example
is retained. A separate original declared-grade control verifies weak-rest and
chord deletion, surviving dot abnormal callback, then orphan-dot deletion with
physical glyph retention. Native connects these owners before dot counting.
No weak-rest image success is claimed from that control, and missing beats or
unknown removed classifier candidates remain refused. Competing/shared supports
and the complete LINK/cleaner/epilog sequence are still partial.

Rest/second-dot supported competition: original rest and DoubleDot relation
selection is connected to source callbacks and the following target checks.
Five declared original controls verify larger-grade/first-tie/all-zero choices
and synchronous single-RestChord dot counts. This is not a new successful
image set; the existing half-rest and note/double-dot examples are retained.
Creation/admission guards for shared support, competing heads/seconds and unknown
removed glyphs remain. Full LINK/cleaner/epilog and post-rhythm editing staypartial.

Actual double-dot chain detachment is verified separately from image export:
removing its real head support immediately recounts the single ordinary owned
HeadChord; the next target check removes both dots in order and retains glyphs.
Post-Voice mutation still requires rhythm reset. Original lazy Part getter
caches change whether the removed second dot remains in its Measure collection;
that cached ownership is observed, not yet matched by native. Competing-head
selection, multi-member callbacks and complete cleaner/epilog remain pending.
Evidence: `evidence/musicxml-dot-chain-detach/VALIDATION.json`.

Head support competition now preserves original owned-chord/top-down member
order, stable upper/lower-dot preference and integer-pitch parity of every
supported head (including ties-even rounding). Six declared original controls
verify it independently from image recognition; stronger relation grade is not
the head criterion. Existing CLI examples remain, while multiple/shared support
creation/admission, multi-member callbacks, direct vertex-removal ownership and
complete LINK/cleaner/epilog remain partial. Evidence:
`evidence/musicxml-head-dot-competition/VALIDATION.json`.

Dot vertex removal now recounts surviving actual ordinary note owners. The
same callback handles multi-head mean counts and reentrant inconsistent automatic
dot deletion; manual dots stay. Original actual first-dot chain mutation and
two declared automatic/manual two-member controls are separate from full-image
success. Existing CLI examples/admission guards remain. Part getter/Measure
cached ownership, shared mixed-owner intermediate ordering, discarded-candidate
closure receipts, post-rhythm edits and complete cleaner/epilog remain partial.
Evidence: `evidence/musicxml-dot-vertex-lifecycle/VALIDATION.json`.

Original mixed-owner dot callback order now matches observed intermediate states:
a deleting dot remains SIG-registered until its copied incoming/outgoing edges
have invoked synchronous callbacks; nested automatic deletion occurs before
later owner callbacks. Head-first/rest-first/manual controls and one actual
double-dot chain compare all callback entry/return states. These controls do not
admit shared-support images. Part/Measure caches and LINK candidate closure for
all nested deletions remain partial. Evidence:
`evidence/musicxml-dot-callback-order/VALIDATION.json`.

LINK dot candidate receipts now retain every actual reentrant vertex deletion,
once in unregister order, with original owner/member-count proof only for
inconsistent automatic countDots cleanup. Weak/orphan deletion alone does not
permit a previously unhandled dot image. An original weak-dot control confirms
that its callback also deletes a strong automatic dot; contextualization now
precedes weak deletion to prevent stale snapshot resurrection. Mixed-owner and
weak-cascade controls are separate from the existing five full-image CLI
examples. Part/Measure cached membership and shared-support image admission
remain unconnected. Evidence:
`evidence/musicxml-dot-removal-receipts/VALIDATION.json`.

Ordinary dot ownership now preserves lazy Part caching and actual Measure
membership across SIG lifecycle callbacks. First-dot registration precedes
staff/support assignment; second dots inherit Part through DoubleDot without a
staff. The image CLI populates the dot subset of PageRhythm FRAT ownership before
rhythm reset. Removal updates its containing Measure before SIG edge callbacks.
Actual saved cold/warm controls and original same-glyph factory-order replay
compare these states separately from existing image recognition/export. Full
multi-staff/cross-Part stacks, other FRAT classes, shared-support image admission
and complete cleaner/epilog remain partial. CLI reports expose the retained
state at `shared_sig.augmentation_dot_ownership`. Evidence:
`evidence/musicxml-dot-part-ownership/VALIDATION.json`.

Same retained-dot re-registration now preserves its contextual grade when the
intrinsic grade is unchanged, matching original same-Inter add/remove. Saved
cold/warm controls verify Part persistence and ordered Measure membership through
second-before-first re-addition; original scalar stack boundaries verify
left-exclusive/right-inclusive selection. These controls do not broaden image
admission or demonstrate full correction/editing lifecycle. Existing image CLI
examples and the negative quarter-rest boundary remain the regression scope.
Evidence: `evidence/musicxml-dot-owner-readd/VALIDATION.json`.

To inspect ordinary automatic dot ownership with the existing Linux source CLI:

```sh
mkdir -p work
cargo run --locked --offline --bin audiveris-musicxml -- fixtures/official-image-musicxml-augmentation/double-augmentation.png work/dot-owner.musicxml work/dot-owner.json
```

Use fresh output names. The JSON's `shared_sig.augmentation_dot_ownership`
contains actual Measure dot IDs and cached Part IDs; the XML remains generated
from the recognized image. This is the existing verified double-dot example,
not a new image acceptance case.

Ordinary Head/Rest/dot owner edits now follow original AbstractInter setters:
explicit Part assignment/reset, staff, intrinsic grade and bounds preserve
unrelated cached Part/context and actual Measure membership. DotFactory's image
path uses the staff setter after SIG registration and before support. Actual
saved-chain setter stages and callback entry/return states are a control, not
new image admission. Moving bounds alone does not perform a complete correction
task or repair old Measure membership. Multi-staff/shared support image admission,
general edit tasks, support re-add ordering and complete cleaner/epilog remain
partial. Evidence: `evidence/musicxml-dot-owner-edit/VALIDATION.json`.

Retained support remove/re-add now keeps stable relation identity separately
from live insertion order. Original first-supported Part selection (including
null without fallback), incoming-before-outgoing callback copy and every owner
count/cache/Measure callback stage match one declared shared-support control on
actual saved HeadChord owners. Already-live support replay does not reorder.
Existing image CLI admission is unchanged; this is not a shared-support image
success. Manual support-added callbacks, general edit tasks, multi-staff and
complete cleaner/epilog remain partial. Evidence:
`evidence/musicxml-dot-support-readd/VALIDATION.json`.

Ordinary newly-added Augmentation/DoubleDot support now completes its original
synchronous callback before another relation is inserted. Augmentation checks
source abnormal then recounts its actual chord if dot or relation is manual;
DoubleDot checks its source only. Three original declared controls distinguish
automatic cache retention, manual-dot count1 and manual-relation nested automatic
dot deletion. Callback entry/return compares Part/Measure, grade/context/bounds,
manual state and owner counts; actual nested deletion reaches LINK receipts once.
This is not automatic shared-support image recognition or a complete correction
command. Existing five-image CLI examples and quarter-rest refusal remain;
Part/Measure editing tasks, multi-staff and full cleaner/epilog remain partial.
Evidence: `evidence/musicxml-dot-support-added/VALIDATION.json`.

Retained ordinary-dot correction task model is available through the Rust library
(`shared_sig::DefaultEditingTask`), under saved owner control only. It is not a
CLI correction command: preEdit/controller impacts/history/general editors remain
pending. Existing runnable image commands and their admission are unchanged.
See [task control](../evidence/musicxml-editing-task/VALIDATION.json).

`shared_sig::EditingSequence` adds retained task do/undo order and explicit impact
notifications. It schedules required RHYTHMS/PAGE work after publication; it does
not yet execute those steps or expose a correction CLI command.

Later retained-owner checkpoint supersedes the earlier notification-only boundary:
`Export::apply_editing_sequence` runs a bounded actual Stack/Measure pass on edited
native owners using retained expectedDuration, then HIGH Voice purge/sort/rename
and native export. The image-entry ownership guard remains unchanged; serialized
reports cannot establish edited-owner provenance. The reprocessing entry does not
call `populateFRATs` or LINKS and does not rebuild dot Part/Measure membership.
Failure restores Graph/rhythm/report and does not advance the task sequence.

Run the same ordinary double-dot input through the new raw bounds-only command:

```sh
cargo build --offline --locked -j1 --bin audiveris-musicxml
# ID72 is the first native dot in this saved input; generic code selects no fixed ID.
/path/to/audiveris-musicxml --edit-dot 72 3 0 \
  fixtures/official-image-musicxml-augmentation/double-augmentation.png \
  /existing/output/dot-edited.musicxml /existing/output/dot-edited.report.json
```

Use the native live `AugmentationDotInter` ID from an ordinary recognition report;
IDs belong to that exact recognized input. Output paths must not exist. The command
accepts exactly one image. It moves the retained dot via raw EditingTask; it does
not search for replacement support links. Library sequences additionally support
undo/redo. `editing_reprocess` separates planned RHYTHMS/PAGE impact from actually
processed owned stacks and the bounded Voice prefix. Full controller/PageStep,
support-changing timed edits, cross-stack ownership editing, linked Voice ID
refinement, shared/multi-staff stacks and general corrections remain pending.
An ordinary edited dot reprocess matches original saved-owner clocks/slots/Voice
while retaining other measures/cache/glyph; it is not evidence of automatic
image correction or broader image admission. All completion flags stay false.

The later Voice refinement unit connects existing actual NextInVoice references
across adjacent owned Measures and applies preferred-ID collisions last, matching
original Voices.refineSystem/Measure.swapVoiceId. It preserves the retained Voice
objects/Chord members and rebinds event IDs. This is declared original/native owner
comparison, not image discovery of such links or a new preference editing command.
The same `--edit-dot` example remains usable; tie/cross-Measure beam refinement,
timed support/general edits and PageStep remain explicitly pending.

Retained-owner refinement now also follows original classified incoming tie and
previous-Measure BeamGroup predecessor references before NextInVoice/preferred ID.
Existing `--edit-dot` bounds-only example is unchanged; automatic tie/cross-bar
beam discovery and support-changing corrections are not newly admitted. See
[owner ordering comparison](../evidence/musicxml-voice-tie-beam/VALIDATION.json).

Retained Voice ID changes preserve null Chord Voice references left by partial
rhythm instead of rebuilding ownership. Controlled fallback tie priority matches
original; this adds no null-reference image acceptance or new correction option.
The existing automatic examples and bounds-only `--edit-dot` command are retained.

BeamGroup/NextInVoice refinement requires a current actual Chord Voice reference.
It rejects a null reference even if retained membership still contains the Chord;
original throws in those controls. Native rollback prevents partially refined
output. This preserves admitted examples and adds no new automatic image scope.

Original timed first-Chord-null Voice refinement is now controlled as a separate
owner lifecycle. It does not grant MusicXML event ownership: event binding still
rejects null, and automatic image/`--edit-dot` admission remains unchanged.
Timed support edits are still guarded; do not clear clocks ahead of callbacks.

Timed ordinary support callbacks now update live duration/endTime while keeping
Part/Measure/Voice/slot/time. This is a saved-owner control, not a new image or
CLI correction admission. `--edit-dot` bounds-only example remains unchanged;
Export timed support guard is still retained at this checkpoint.

### Explicit ordinary support transfer (current checkpoint)

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  --move-dot-support 72 6 \
  fixtures/official-image-musicxml-augmentation/double-augmentation.png \
  support-moved.musicxml support-moved.report.json
```

IDs refer to this input's **live recognized** dot72 and head6; other inputs must
use their own report/retained identities. This deliberately changes support from
B4 to F5 after recognition; it is a user correction, not automatic interpretation
of that image. The resulting8-note stream is compared with the original's same
explicit task/reprocess/PartwiseBuilder operation. XML/XSD is checked separately.
Only one ordinary black single-head support transfer inside the same retained
stack is admitted; final shared support, rests/void/multi-head/DoubleDot tasks and
cross-stack editing stay guarded. Tasks add new links before removing old ones;
undo restores unlinks before removing new links. Added callback counts only
manual relation/dot, removed callback counts a live target. Original nonmanual
undo underfills13/16; native refuses a full-measure export and restores all state.
A genuinely manual support created by transfer permits subsequent do/undo/redo
through the library's retained sequence. Full controller/preEdit/history/PageStep,
automatic target discovery and project persistence remain partial; flags false.
Existing `--edit-dot` and unedited image examples are retained.

### Explicit relation-task support correction

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  --link-dot-support 72 6 manual \
  fixtures/official-image-musicxml-augmentation/double-augmentation.png \
  relation-support.musicxml relation-support.report.json
```

This uses the image's live IDs through **UnlinkTask then LinkTask**: remove old,
add the supplied relation, publish source, then process selected actual stacks.
Undo reverses execution, while impact visits source/target in original task order.
Unlike `--move-dot-support`'s EditingTask, this does not edit bounds or detach glyphs.
`manual|automatic` is an explicit relation status, never silently promoted.
On this input `automatic` leaves an original13/16 measure; native refuses with no
output pair. Automatic old-support undo is likewise refused transactionally.
Genuine manual support do/undo/redo matches original owned rhythm/export.
The positive8-note result is a **declared correction**, not automatic recognition
of a new image or automatic correction target discovery. Original saved-owner
controls, original real RhythmsStep processing and native XML/XSD checks are
separate. Same-stack single black HeadChord guard, special/shared/cross-stack and
full controller/PageStep limitations remain; nonUI parity is still incomplete.

Retained relation-task replay now preserves actual object identity and edited
relation grade/status: adding the same already live object emits no callback but
still publishes source. Removing an absent object is an original exception;
native refuses before impact/publication rather than silently accepting it.
The same CLI example is retained. Grade setter and task-replay checks are owned
controls; they do not add automatic correction discovery or general setter CLI.

### One mixed bounds/support correction sequence

```sh
cargo run --offline --locked --bin audiveris-musicxml -- \
  --link-dot-support 72 6 manual --edit-dot 72 3 0 \
  fixtures/official-image-musicxml-augmentation/double-augmentation.png \
  mixed-correction.musicxml mixed-correction.report.json
```

For the same retained dot this runs bounds EditingTask, UnlinkTask, LinkTask,
then one selected-stack reprocess in a single rollback boundary. It keeps two
source publications before impact; this is not two separately committed edits.
The input/IDs above are an example; typed live ownership and stack checks remain
mandatory. Combining distinct dots or the other editor-support task family is
refused. Original CtrlTask RHYTHMS dispatch and native event/XML are compared as
an explicit correction. Automatic undo remains13/16 and cannot be exported as a
full measure. Full controller/PAGE/history and actual Book/SheetStub dirty-state
persistence are partial: planned `mark_sheet_modified` records only the direct
controller DO request, not all dirty changes caused by original callbacks.

### Editing dirty-state observation

The existing mixed correction command also reports `shared_sig.editing_book`
and each publication/impact's `actual_dirty_state`. These come from the retained
Book/SheetStub editing owner, not planned impact flags. Bounds/publication alone
can remain clean; a changed dot abnormal callback dirties the owner before impact.
Controller DO marks it even below the first impacted step. UNDO/REDO can dirty
through callbacks without a direct DO request. Sheet flag false does not clear
Book flags. Writing MusicXML does not imply score reduction or project storage.

Tracking begins after initial recognition/rhythm on the current single-image
owner; imported project flags, all dirty sources, multi-sheet/repository state
and store/load remain partial. No new image or correction scope is admitted.

Dirty tracking also covers the retained cleaner Head/Stem abnormal callbacks and
automatic relation re-add. Original same-object duplicate does not invoke an
added callback. These are owned controls; CLI image admission is unchanged.
Manual HeadStem changes need actual ChordStem/cache integration and are guarded,
not approximated by the automatic callback. Original manual ownership can change
without an abnormal or dirty change; controller DO remains a separate cause.

Impact's legacy `latest_completed_step` field is the caller-supplied planning
horizon here, not observed native SheetStub.doneSteps. `planned_steps` may include
PAGE, while `controller_impact_complete` remains false and only the separately
reported retained RHYTHMS work has executed. No PageStep completion or project
persistence is inferred from this notification or the dirty booleans.

Retained HeadChord entries also expose `retained_chord_cache`. This is lazy
memory state: a ChordStem callback clears geometry/beams, whereas readonly report
creation does not warm it. Existing CLI options and admitted images stay the
same. Manual HeadStem/ChordStem corrections are not yet admitted by the export
path; original saved-owner/reprocess observation is separate from native success.

The lower-level `OwnedMusic.apply_manual_head_stem_callback` now updates the
actual retained ordinary Chord owner/cache after the compared nested callbacks.
It is an explicit correction stage with supplied relation properties, not a
MusicXML CLI option: publication, selected-stack reprocess and transactional
export validation remain guarded. Existing CLI examples and input scope stay
unchanged. An original7/64/underfilled parallel-Voice correction is tracked as
a remaining dependency rather than silently rounded or completed.

The retained ordinary `Graph.chord_duration` / `chord_end_time` getters now
read exact fractions such as7/64. Getter success is separate from MusicXML
admission: current rhythmic clocks/export continue to reject finer-than-
sixteenth durations. Manual HeadStem correction remains a lower-level owned
control, with no new CLI switch or automatic image reception enabled.

The retained edit rollback checkpoint also covers actual Chord stem/geometry/
members. The compared manual owner control reaches selected rhythm processing
but refuses7/64 and restores its old owner/XML. It does not add a manual
HeadStem CLI option; original Rational/underfilled-Voice processing remains a
required dependency before that admission.
