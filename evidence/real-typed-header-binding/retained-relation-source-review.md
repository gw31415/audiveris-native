# Retained relation predicate for A's image ownership observer

The failing assumption is concrete: selected Key members retain **KeyAltersRelation** edges to each other. Both endpoints are in `hold()`'s selected C/K/member list, but these edges are not imported by the narrow C/K registration adapter. The assertion at current `src/music_headers_owned/image_tests.rs:91–96` therefore rejects valid header state.

Pinned source `fixtures/key-preparation-source/KeyBuilder.java:1745–1754` creates pairwise KeyAltersRelation among the accepted live KeyAlter objects before `KeyInter.createAdded`. `fixtures/key-clef-source/KeyAltersRelation.java:30–37` defines this member-to-member support. The existing later `KeyInter.buildKey` branch independently does the same at `KeyInter.java:1302–1307`.

Read-only existing fixture inspection of `fixtures/official-grid/laura-gray-time-column.json` shows **18 KeyAltersRelation per System**, all `KeyAlterInter → KeyAlterInter`, in both `after_retrieve_time` and `after_system_time_stop`. Each System also has 12 `KeyInter → KeyAlterInter` Containment and 3 `ClefInter → KeyInter` supports. NoExclusion in that sample is `BarlineInter ↔ BarConnectorInter`, so it does not explain the selected-endpoint assertion failure. The three-sharp control has 3 KeyAltersRelation and the four-flat control has 6, as the source pairwise loop requires. This inspection did not rerun images or original code.

Current `src/header_finalization.rs:342–351` retains the upstream relation vector. Its removals (`:32–37`, called for losing Clefs at `:408` and purged Barlines at `:445`) remove only incident edges of removed vertices. Freeze/member serialization (`:446–481`) does not remove KeyAltersRelation, and `RawFinalizedSystem` returns the retained vector at `:516–519`. The typed carrier must retain these exact relation Arcs, encounter order, endpoints and manual bodies.

The exact **Graph-imported subset** is, for each actual selected Staff header with actual Clef C and Key K:

- `kind == Containment`, source Arc pointer-equal to K, target Arc pointer-equal to one of K's selected member Arcs;
- `kind == ClefKeyRelation`, source Arc pointer-equal to C and target Arc pointer-equal to K.

Check those bound edges with `Graph::assert_native_header_relation`. Separately retain and compare **every** `NativeSystemHeaders::relations()` Arc and canonical endpoint Arc across continuation, including KeyAltersRelation and unrelated live graph relations. Do not claim these extra relations were imported, do not mutate/drop them to satisfy the observer, and do not expand the production importer solely to fix this A test. A kind-only filter is weaker than the actual selected-root/member predicate above.

No repository changes, build, original probe, native image, or Git operation were performed.
