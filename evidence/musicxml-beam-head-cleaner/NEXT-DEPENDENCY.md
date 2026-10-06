# Remaining original cleaner ownership dependencies

Pinned original5.11.0 source9e1e55cd2746037d059345881c53e6a6754bffbd.
The final goal includes all original nonUI logic; these are pending dependencies,
not planned omissions. No global completion flag changes.

- Cached Head/Stem extensive member deletion must be staged with actual
  retriever/cache and typed HeadChord/Mirror ownership, preserving registry and
  physical glyph identities. Rest-only extensive callbacks are not this stage.
- AbstractChord.remove -> Measure.removeInter also removes Voice ownership.
  Current synchronized containers are before RHYTHM; voice/chord correction and
  measure reassignment need real stored Voice/Part/stack ownership.
- InterCleaner.purgeContainers delegates to Part.purgeContainers, which removes
  deleted LyricLineInter from Part.lyrics (Part.java1185-1202). It is not generic
  chord or glyph cleanup. Lyrics/Sentence recognition, Part ownership and CLI
  correction logic must precede claiming full InterCleaner.
- RestChord.getBeams derives/sorts actual BeamRestRelation beams by tail distance.
  BeamRest support/context/checks are not connected; rest symbol reduction
  continues to refuse unsupported supports. No invented interleaved-rest output.
- Remaining full-class overlap, special-head raster branches, frozen editing,
  slur/fermata/text and general mirror/ensemble cleanup remain partial.

Existing image CLI inputs retain MusicXML event/beam/XSD regressions; declared
SIG controls remain separate from automatic raster generation of those branches.
