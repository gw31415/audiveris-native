# Next real-image recognition units

1. An Laura public v4 stops at TUPLET_THREE. Compare original Tuplet ShapeChecker
   (including pitch admission), nullable createValid over actual system Chords,
   actual Stack registration and LINKS cleanup where needed. Do not whitelist
   tuplets or infer factory-null from final XML having no tuplets. Connect the
   verified path, then public Page/export and full semantic/truth/XSD/reader checks.
2. Der Winter v2 passes the former nonconvex Beam path and stops in
   music_measures::continue_measures at music_native_clef_required. Original second
   system bottom Staff has no clef/key header. Compare source's null-clef behavior
   and actual Part/Staff ownership without inventing a Clef object/report owner.
   Preserve the independent truth distinction: original has20 later bass pitch
   errors from fallback clef, so blindly matching wrong pitches is not a quality goal.
3. Rest factory currently tests Measure.staff (top Staff) and can skip lower
   Staffs. Source uses glyph integer centroid→actual Staff→actual Part→nullable
   Measure, with same-Staff HeadChord proximity. All12 original SYMBOLS rest
   candidates precede a separate buildRestChords stage; final XML has9 rests.
   Fresh source lifecycle observations are prepared in /tmp but are not yet a
   native acceptance proof. Implement and compare this necessary recognition
   unit when reached, then pickup/slur/voice/quality categories.

Precision debt: ordinary Beam planning still uses explicit XML-style rounding
in stem_image_head_plan before target/seed/stump plans, and a separate raster
rounding in music_beams. Graph retains raw actual values. Do not mix a raw final
border with rounded preceding choices. Investigate effects on required image
quality; numerical all-API parity is not the goal.

Keep small normal main commit/push/readback, actual owners/manual/relation
identity/clocks/rollback,63+8/exact7/64 regressions. No full649 repeat, Mac/GHA,
new auth/settings/permissions, different environment, or replacing original
task/unobtained ce785c9. Continue image quality work after this checkpoint.
