# Linked explicit Tuplet on the real Der Winter page

Original Audiveris at `9e1e55cd2746037d059345881c53e6a6754bffbd` processes the
unchanged `fixtures/real-rest-page/der-winter-gray.png`. These four independent
runs are development evidence, not native recognition inputs.

- `factory-observed.json`: real SIX candidate `[372,1089,13,16]` is created and
  registered in its actual shared Stack with NULL Staff.
- `lookup-observed.json`: actual Chords total `3/8`; three outgoing-false links
  from Chords to Tuplet are returned in abscissa order.
- `stages-observed.json`: ordinary SYMBOLS → LINKS → RHYTHMS. Addition callbacks
  yield abnormal `[true,true,false]`. Weak contextual purge removes the same
  three relation objects in insertion order, then the same Tuplet vertex.
- `support-inputs-observed.json`: independent lookup observation including exact
  cached Chord intrinsic grades. These differ from rounded values in saved OMR.

The support sources, by Chord bounds, are `[277,1019,15,47]` at
`0.9402200761442747`, `[369,1020,15,45]` at `0.9516913164381212`, and
`[481,1025,15,46]` at `0.9439557546363567`. SIX uses target coefficient `2*0.17`,
with relation grade one. Original reverse-grade summation gives contribution
`0.9641948300543761` and contextual grade `0.22218618988685207`, below `0.5`.
Do not replace these cached intrinsic grades with current Head averages or
Chord contextual grades.

The listener runs after original SIG callbacks. Before the first edge-removal
event, Stack membership is empty and the Tuplet reports removed while it is
still registered in SIG. Its abnormal flag remains false. The last vertex event
removes it from SIG while the retained object still references the same SIG.
Linked Chord clocks/Measure references and Stack expected duration remain
unchanged across all seven events. Those ordinary clocks are NULL; this is not
proof for initialized-clock, manual, frozen, or rollback cases.

`REVIEW.json` preserves the initial 19 contract checks for factory/lookup/stages.
`PROVENANCE.json` verifies all four executed observation/source/image/console
hashes and distinguishes independent runs. Each original provenance is retained
byte-for-byte. `executed-sources/RealTupletSupportInputsImageProbe.java` is the
exact extra observer source; the common installed helpers and runner are
identified by hash rather than duplicated. Historical compile/run arguments in
`support-inputs-provenance.json` can be replayed with that archive path substituted
for `/tmp`. Compile-only signature stubs must never appear on runtime classpaths.

This false SIX disappears only after genuine positive linkage and contextual
purge. It does not prove surviving-positive Tuplet MusicXML quality or overall
recognition parity.
