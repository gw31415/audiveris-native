# Explicit Tuplet observations on AnLaura

These are development-only observations of original Audiveris commit
`9e1e55cd2746037d059345881c53e6a6754bffbd` processing the unchanged
`fixtures/official-grid/laura-gray.png`. Native recognition does not read these
files. No native comparison or image-quality acceptance is claimed here.

| File | Independent original run |
| --- | --- |
| `factory-observed.json` | Ordinary CURVES, original SYMBOLS prolog, then per-system original buildSymbols → buildRestChords → lateChecks. Delegating factory observes checked Tuplet evaluations. |
| `stages-observed.json` | Ordinary SYMBOLS → LINKS → RHYTHMS on the same real book within this run. Holds actual Tuplet and Chord references through removal. |
| `lookup-observed.json` | Fresh ordinary SYMBOLS, MeasureFiller, the seven preceding SymbolsLinker methods, original lookupLinks, private collector replay, then original linkTuplets. Remaining LINKS epilog and RHYTHMS are outside this run. |

Two checked `TUPLET_THREE` candidates reach the factory. The first intersects
two actual Chords, is created without a Staff, and is registered in the actual
Stack as abnormal and nonmanual. The second has no nearby Chord and returns
null without changing graph object identities. The generated positive candidate
survives SYMBOLS. In ordinary LINKS, its collector reaches total `3/8`, matching
the expected total, but the integer Tuplet center X=802 is outside the first/last
Chord tail range 804–823. Lookup returns no links; `linkStackTuplets` removes the
nonmanual orphan. It does not reappear in RHYTHMS. Actual Chord references,
clock fields, and Stack expected duration are retained through removal.

`REVIEW.json` retains the ten original contract-check results and their source
interpretation. Original per-run provenance files are copied byte-for-byte.
`PROVENANCE.json` verifies observation, observer/helper, input, and console
hashes; identifies the installed Java hierarchy; and records its compile-only
check. It does not rewrite historical execution claims.

## Observation limits

- Factory, stages, and lookup are independent fresh runs. Identity labels and
  numeric IDs must not be compared across runs as object identity.
- The GraphListener was registered after the original SIG listener. Its events
  observe **after** the original callback; no before-callback interception is
  claimed.
- The collector trace replays original private methods against the same actual
  objects. Its returned links are not applied; original linkTuplets recomputes
  its own links afterward.
- This candidate never receives a ChordTupletRelation. Linked-positive Tuplets,
  manual relation preservation, and `TUPLET_SIX` remain untested.
- Tuplet-quality parity and native XML support are not established by this
  original-only orphan example.

## Source preservation and compilation

The installed `scripts/java/RealTupletProbeSupport.java` contains only the eleven
unchanged common utility methods and per-JVM identity/candidate storage from the
executed Rest helper. It does not duplicate the Rest observer. The installed
factory changes only its superclass to this support. Stages and Lookup are
byte-identical to their executed sources. This consolidated hierarchy was
compiled with javac 21 and the pre-existing compile-only signatures; it was not
re-executed against Audiveris for these saved observations.

For exact historical replay, unchanged Rest, Stages, Lookup and Ledger helpers
are referenced at their repository paths and hashes in `PROVENANCE.json`.
The historical factory differs only in its superclass. The manifest records the
single byte substitution that reproduces its executed SHA256 in a temporary
copy; this reconstruction was verified against the retained executed source.
No duplicate source archive is stored. Runtime classpaths must contain original
Audiveris jars and compiled observer classes, **never** signature stubs.
The installed `scripts/run_real_tuplet_control.py` supports `--probe factory`,
`--probe stages`, and `--probe lookup`. `--compile-only` validates the whole
hierarchy without starting recognition. RUNNER-COMPILE-PROVENANCE.json and
RUNNER-COMPILE-CHECKS.json retain nine successful checks. The consolidated
hierarchy has not replaced or repeated the saved original observations.

Example: `python3 scripts/run_real_tuplet_control.py --oracle-home
/workspace/native-build-tools/oracle --image fixtures/official-grid/laura-gray.png
--output /tmp/real-tuplet-new-run --probe factory` (one shell command).
