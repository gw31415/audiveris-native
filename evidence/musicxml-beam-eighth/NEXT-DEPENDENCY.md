# Positive beam comparison: historical preparation and current connection

`fixtures/official-image-musicxml-beams/beam-eighth-quarter.png` is one fixed
Bravura44/interline11, ordinary five-line treble/C-major4/4 two-measure input.
The same two eighths and seven quarters as the flag input are joined by a
concrete beam instead of flags. No coordinate sweep was performed.

Unmodified original5.11 completes RHYTHMS/export on this input. The first
two B4 notes have duration1/divisions2 and beam number1 `begin`/`end`.
At125e928 native recognized the beam and two physical side stumps, then refused with
`music_beam_with_stump_or_seed_pending`. Neither native output file is created.
This evidence is **preparation/comparison, not a completed beam export**.

The dependency chain below was closed for one standard beam per system in the current checkpoint; see VALIDATION.json and docs/MUSICXML-LIMITED.md. Sibling/alien/shared-system and general parity remain pending. The recorded preparation state is historical.

Historical125e928 cut list, following original execution order (closed for the
validated two-side-stump image; hooks and general branches remain pending):

1. `BeamLinker.VLinker.buildGeometry/filterHeads/inspect` → automatic beam-origin
   seeds, head corners, sections/chunks and StemBuilder items. Existing native
   beam catalogs and generic `start_is_beam` filter/item kernels are usable
   parts; the connected image builder is currently head-origin only.
2. `StemsRetriever.linkStems` → beam `linkSides` before `linkStumps`, before head
   phase1/2. `VLinker.expand/link` must create/reuse actual cached stem glyphs,
   validate `BeamStemRelation.checkLink`, and mark/apply HeadStem links. The
   original side profile4/artificial-stem fallback is also still pending;
   substituting head-only linking or removing guards is not the algorithm.
3. Actual BeamStem support contextualization, weak/reduction checks,
   `SigReducer.checkBeamsHaveBothStems/checkHooksHaveStem` and
   `StemsRetriever.finalizeBeams`; preserve owned groups,
   cache identity and removal callbacks. Current beam-free reduction/finalizer
   cannot silently accept live beams.
4. Owned beam → chord membership under `ChordsBuilder/AbstractChordInter`,
   duration reduction and `PartwiseBuilder` beam ordering/number/begin/end.
   Reuse exact eighth ticks and divisions from the flag stage, without
   deriving duration from the fixture's positions.

Original saved beam median `[288,209,410,209]`, height6 and intrinsic grade0.565
are separately inspectable in the `.omr`. Native detection has the same median,
height and rounded grade, but its enclosing SIG bounds differ from the saved
final original bounds. Whole BEAMS/STEMS/SIG equality is not claimed. Sibling
beams/hooks, multiple head/stem ownership and general cleaner/rhythm still
require their corresponding original dependencies; none are planned omissions.

## Next original dependencies

1. Preserve `StemsRetriever.linkStems` / `BeamLinker.linkSides` LEFT/RIGHT side
   passes before `linkStumps`, including skipping intermediate stump linking
   after a failed side. The current original-success image has two side stumps
   only; a positive intermediate/`continue` image remains unverified.
2. Connect existing sibling/closer-limit catalogs to actual cached stems and
   owned SIG, then original longest-beam-first order and competing-hook cleanup.
   `SigReducer.checkHooksHaveStem` and `PartwiseBuilder` hook/beam ordering and
   finer duration divisions remain necessary before multiple levels/hooks.
3. Extend original beam-area stroked-raster erasure, cross-system shared-glyph
   finalization, mirrors/void heads and broader consistency/cleaner callbacks.
   Retain admission refusals until each dependency is connected and compared.

These are staged dependencies, not planned omissions from original nonUI parity.
The same-image nine-event native export is now confirmed; details, partitioned
563-test coverage and independent-clone build/XSD are in VALIDATION.json.
