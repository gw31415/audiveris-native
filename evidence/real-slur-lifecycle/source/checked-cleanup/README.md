# Historical unmatched candidates after owned Slur cleanup

`test-only.patch` changes only the existing real-page CHECKED observation helper. It does not change recognition, evaluation ranks, checked predicates, factory admission, coordinates or masks used by production.

The two historical native FERMATA_BELOW candidates each consisted of two actual native connected glyphs. `prepare.py` reads the unchanged saved native classifier/component dump, joins each old candidate's part_glyph_ids to its unique single-glyph trial and then the corresponding component bounds/runs. It verifies the recorded weights77 and81. The literal run masks are confined to the Rust observation test; no source glyph or expected native output becomes a recognition input. `PROVENANCE.json` records exact dump, fixture and freshv3lifecycle marker hashes.

The freshv3 accepted Slur registry masks overlap71/77 and70/81 old candidate pixels, respectively. Per-component overlaps are62+9 and59+11. Residual old pixels6 and11 are not described as erased. The current post-SYMBOLS classifier dump was not saved before the old assertion failed; the new test must still prove both old candidate geometries are absent, not optionally forgive them.

The new assertion requires a unique actual same-System accepted Slur for each old mask, actual registry glyph/body correspondence, original raw black pixels, every old component intersecting the accepted glyph, retained Part/System/Slur membership and exact relation Arcs. It then requires all of that accepted Slur's ink to be absent from actual Symbol-group registry glyphs after cleanup. Group::Symbol is populated only by the ordinary post-CURVES symbol phase. These are native ownership/ink observations; existing earlier lifecycle assertions already prove acceptedSlur-only pixel erasure and callbacks.

All10 original CHECKED rejects,77 source factory arrivals,4 accepted-limit controls and the requirement for no extra surviving family evaluations remain strict. The old two unmatched rows remain preserved as historical observations in the emitted report, now accompanied by actual cleanup evidence instead of a false `preserved` claim. The same existing DerWinter target supplies execution; no new image or scalar target is added.

Preparation performed no checkout edit, build, image execution, Java run or Git action. The patch has not been executed by this agent; parent must format/compile and run the coordinated v4 target. No SlurXML or quality success is claimed.
