# Original Tuplet FontSymbol dimensions

These11,663 bytes are immutable dimension resources measured from the untouched
pinned Audiveris JAR for THREE/SIX, all six original MusicFamily enum values and
base interlines1..128. They retain actual backup-family selection (JazzPerc uses
FinaleJazz for tuplets). No font file is copied, Java is not used by native runtime,
and no dimensions are scaled/interpolated from calibrated note-head templates.
Native loading pins the asset SHA256 and explicitly rejects unsupported family/
interline rather than silently choosing a substitute. General native font layout,
unbounded size and actual SheetStub family ownership remain separate dependencies.

Original FontSymbol.getDimension applies Math.rint to ShapeSymbol TextLayout
bounds, with OmrFont base point size4*interline. These are measured font resources,
not a reimplementation of Java TextLayout or complete symbol/font/UI port.
[FontSymbol at the pinned source](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/ui/symbol/FontSymbol.java)
[ShapeSymbol at the pinned source](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/ui/symbol/ShapeSymbol.java)

provenance.json records actual original compile/run, JAR and six font hashes,
observer/reflection helper hashes, raw measurement output and asset hash. The
probe was originally run from/tmp; its identical source is saved under scripts/
java/TupletFontProbe.java. scripts/run_tuplet_font_control.py provides the same
measurement in a fresh isolated output directory without Book recognition/archive.
The resource generator only rearranged original integer dimension rows. Bounds
are independently checked by real original generateOneTuplet observations in
fixtures/official-tuplet-placement; no native outputs generated the assets.
