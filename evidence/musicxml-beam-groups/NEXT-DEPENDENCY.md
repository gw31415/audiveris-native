# Next original dependencies

The image owner now connects ordinary, distinct-glyph beam groups to native
MusicXML. `BeamLinker.VLinker.getCloserLimit` is connected at zero skew: actual
neighbor bounds, own-group exclusion, **AbstractBeamInter.isGood = 0.35**,
nonhook segment intersection, side-aligned exception and stable nearest
direction-facing border sort, returning the opposite border. Sixteen declared
numeric original controls cover positive shrinking; the one new full-image
comparison has two nonintersecting groups, so its four closer limits are null.
These are different validation scopes.

1. Hooks: `BeamLinker.linkSides` uses the ordinary link profile for hooks or
   competing hooks, retains linked-side state and removes a competing hook only
   when the concrete beam has both sides. Connect typed owned hook support in
   SIG/cache, `SigReducer.checkHooksHaveStem`, PageCleaner hook areas, and
   `PartwiseBuilder` forward/backward hook determined by beam center versus stem
   center. A partial beam is not interchangeable with an ordinary full beam.
2. General groups: glyph aliases/callback removal, cross-staff and cross-system
   ownership, multi-part surrounding areas and original orphan finalizer
   branches. Numeric closer-limit agreement does not establish those branches
   or end-to-end overlapping alien-group image parity.
3. More beam levels/flags, dots and tuplets: actual factories/lateChecks,
   `AbstractChordInter.getBeamsOrFlagsNumber`, Rational/rhythm dependencies and
   original export; retain full-measure safety checks while connecting them.
4. General reduction, correction logic accessible through CLI and complete
   cleaner/epilog. Original nonUI feature parity remains the final goal; no
   planned recognition/analysis/correction/export omissions.

The thin AA-off beam contour now admits restored fractional slopes only when
all four vertices normalize to the already certified rectangle. Slopes crossing
normalized pixel rows and wider strokes still refuse; this is not bounding-box
erasure for general sloping beams. No nonzero page skew/scan/other-font expansion
was performed. All overall completion flags remain false.
