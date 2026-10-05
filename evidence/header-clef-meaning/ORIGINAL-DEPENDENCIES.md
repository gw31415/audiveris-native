Original5.11 source: 9e1e55cd2746037d059345881c53e6a6754bffbd.

- HeadersBuilder / ClefInter: already recognized frozen actual native header -> owned MusicHeader SIG identity. No oracle state imported by product.
- Measure.getClefBefore / AbstractNoteInter / ClefInter.noteStepOf, octaveOf: actual header pitch and actual head pitch, Java ties-even rounding and integer division. Existing180 original pitch controls retained.
- PartwiseBuilder.buildClef: G/F/C sign, 3-rint(pitch/2) line, octave-change +/-1 for 8VA/8VB. Private original method invoked normally through reflection for12 shape/position controls; no exported note list used by native.
- PartwiseBuilder.areEqual/processClef/isNewClef: compare semantic attributes, not physical IDs. Writer supports measure-start attributes; mid-measure ordered events remain pending.
- SymbolsFilter/PageCleaner: native alto header starts82; physical left border is[80,220,2,46] outside erased header. Original same image also has header start82 and no real retained left bar (dummy bar at82).
- InterFactory.create(ARPEGGIATO) -> ArpeggiatoInter.createValidAdded -> lookupLink: right-side lookup starts glyph.x+width; width Scale.toPixels(ChordArpeggiatoRelation.getXGapMaximum(profile)). Original saved system profile1 gives2.5*11 ->28 pixels. AWT half-open intersects excludes exact boundary contact. If no chord intersects, factory returns null. Native uses all retained head-chord bounds conservatively and only connects this null branch; a possible link refuses rather than claiming positive arpeggiato support.

Bass/alto same-image full recognition/export comparisons are distinct from12 buildClef kernels and3 lookup controls in saved RHYTHMS chord context. Small/8va/8vb/other C-position images, positive arpeggiation, percussion, general ledger/music/scan conditions remain unverified. Product CLI uses neither Java nor original output. Final original nonUI target is unchanged.
