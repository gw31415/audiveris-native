# Original HEADS registration and epilog

Pinned unmodified Audiveris 5.11.0 at 9e1e55cd2746037d059345881c53e6a6754bffbd,
Linux, runs GRID through LEDGERS, HeadsStep.doProlog, NoteHeadsBuilder.buildHeads
and HeadsStep.doEpilog. The observer never reads native output. Laura ends with
352 heads, 22 duplicate removals, 513 head overlap exclusions, staff note counts
44/58/48/56/112/34, three contextualized small beam hooks and the original head
seed scale. It loses no heads or beams during Laura small-beam purging.

`purge-controls.json` observes original private purge and tally methods on 40
constructor-only controls: empty/singleton, three duplicates, lower/higher grade,
EPSILON boundary and adjacent double, two-sided seed transfer, no seed, weak
heads, unlike shapes/bounds, same/different staff, pitch distance, horizontal
ratios 0.2/0.8, area ratio 0.25, separated/touching rectangles. Their glyphs are
null; Laura additionally exercises actual canonical foreground glyph identity.

`epilog-controls.json` contains seven original beam contextual-grade cases
including exclusions and opposite directed supports, nine small-beam cases
including grade equality, geometric edge contact, slanted area, minimum width
and sequential removals, and four tally cases with 0/9/10 samples and removal.

AbstractNoteInter.added updates Staff.notes when added to SIG; its LinkedHashSet
retains registration order. The later sorted addNote pass does not reorder it.
Removed duplicates are tombstones in the native shared Graph, removed tally
entries are cleared, and all relation endpoints belong to live vertices. Replays
preserve native IDs; JVM IDs are used only to normalize oracle comparisons.
The new head_registration.shared_sig is the continuation of the same common
Graph; the old shared_sig report remains the pre-HEADS diagnostic snapshot.

The ordinary Bravura 51pt/five-line staff and ledger dependency subset is
certified here. HEADS completion remains false for the overall port until its
broader font/size/staff catalogs and pipeline coverage are verified. No native
persistent HEAD_SPOTS image is stored; transient raster state is released by CLI
once the output pixels have been prepared. MusicXML remains unfinished.
