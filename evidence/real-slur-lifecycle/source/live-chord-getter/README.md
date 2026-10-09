# Actual HeadChord geometry at the Slur lookup boundary

The v2 DerWinter image continuation reached the newly compared obstacle filter, then refused Slur owner lookup with `retained_chord_cache_owner_required`. The no-cache recognition representation already owns Chord geometry. Allocating every retained editing cache and warming beams is not a prerequisite for reading that representation.

## Reviewed production change

`Graph::retained_or_live_chord_getters` validates a live typed HeadChord vertex. With a retained cache it calls the existing `read_retained_chord`, keeping old cached geometry until the original invalidation boundary. Without a cache it gets the Stem through actual incident `ChordStemRelation` and validates the endpoint type/System, then decodes the actual vertex's nullable bounds/head/tail. It neither allocates a cache nor reads reports, projected source ownership, Voice IDs, beams, or clocks. `NativeOwners::borrow` uses this sibling of the existing `retained_or_live_chord_bounds`.

The authoritative no-cache fields originate in `music_measures::build`: geometry from actual Heads and Stem is installed in the actual Graph vertex before the report is projected. `Graph::invalidate_note_chords` updates these same fields from actual live Containment/ChordStem edges and only resets `chord_cache` if one already exists. The fallback does not reconstruct owners from its output. The existing caller additionally checks the returned Stem key against the retained actual Chord owner.

## Pinned original contracts

Source revision: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

- `HeadChordInter.computeLocations` (137): a leading Head determines head/tail. Stemless head and tail are the Head center. A Stem determines tail x and top/bottom y, with head y from the leading Head.
- `HeadChordInter.getBounds` (279): a non-null bounds cache is retained. Otherwise union actual Head bounds, obtain actual Stem, and add the tail point. This is Java Rectangle.add(Point), without an extra pixel at the maximum coordinate.
- `HeadChordInter.getStem` (498): obtain the first actual ChordStemRelation endpoint; Stem may temporarily be null during construction.
- `AbstractChordInter.getHeadLocation` (699) and `getTailLocation` (919): compute only when the corresponding cache is null. They do not clear a previously cached value in anticipation of a callback.
- `AbstractChordInter.invalidateCache` (1069): this callback explicitly clears beams/bounds/head/tail and recomputes grade. Its null-Measure branch may request bounds while finding the Stack. Reading before this callback must preserve the old cache.
- `SlurLinker.lookup` (390): obtain each actual Chord's bounds, then test the lookup Area and select an actual Head. An eager all-Chord beam-cache warmup is not part of this sequence.

This is a representation adapter for the already-computed native recognition geometry. It does not claim native cache allocation timing is identical to all Java lazy fields, nor broaden admission for missing Head locations, orphan Slurs, unproved obstacles or ties.

## One bounded scalar target

`music_slurs::boundary_tests::slur_chord_getters_preserve_cold_geometry_warm_cache_and_actual_clock_owners`

`test.patch` adds to the existing declared owner fixture; it does not run recognition. It creates a real Graph ChordStem relation and retains the fixture's actual Chord, Voice, Slot, Measure, Staff/Part/System, Slur and relation owners. A Head box `[30,120,6,5]` and Stem box `[34,90,2,34]` give the source-defined bounds `[30,90,6,35]`, head `[35,122]`, tail `[35,90]`.

Observed contract: a cold getter returns those values without introducing a retained cache or changing any owner/clock observation; actual Slot offset is `7/64`, expected Measure duration `3/4`, and the actual Chord end `23/64`. The warm-state control declares old geometry while the actual edge already references the Stem, then requires it to survive the getter unchanged. This is an explicitly declared intermediate state, not a new source callback observation. The malformed-Stem endpoint control is in the warm branch only and must refuse without mutation; cold success still validates the actual live typed Stem in production.

The test uses the existing actual Voice/Slot fixture rather than inserting fake timing fields as a substitute for owners. Its old-cache setup intentionally calls `bind_recognition_chord_caches` in test preparation only. The production fallback does not call that eager binder.

## Validation boundary

This child performed read-only source review and prepared the test delta in `/tmp`. No child build, test, image run, Git operation or checkout edit was executed. Root adopted and formatted the delta and started `/tmp/run-slur-lifecycle-v3.py`; its outcome must be read separately. There is no Slur lifecycle success claim here.

The preceding v2 failure remains at `/tmp/slur-lifecycle-v2/PARTIAL-VALIDATION.json`. Its gap decisions were preserved at `/tmp/slur-lifecycle-v2/gap-decisions.json` before retry (67,396 bytes; SHA256 `3e316c3cbc9b43be5a1dff9e6ea2821234b47f244c1156e48504f26fbf29b4cf`). v2 AnLaura and exact 7/64 XML byte/XSD results passed; DerWinter did not complete. The recognition lifecycle, subsequent writer, and actual image-quality acceptance remain distinct frontiers.
