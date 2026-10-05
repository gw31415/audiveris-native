# Native key alteration components and empty-slice fallback

The runtime now extends key preparation into actual glyph extraction and classifier evaluation. `key_components` records the first connected-component pass; `key_fallback` records the later empty-slice extraction pass. These outputs are provisional key-alteration candidates, not selected key signatures or complete HEADERS.

## Source-faithful operations

- Extract vertical connected components from native NO_STAFF pixels in the geometric key area
- Filter tiny/right-edge parts, retain at most eight parts, and build the official chamfer-distance adjacency graph at a 1.5 staff-interline gap
- Enumerate connected subsets with weight/size pruning before constructing compound pixel sets; use source-equivalent component/neighbor ordering
- Apply exact width, height, weight, centroid-to-slice and embraced-peak gates
- Compute the existing native 110-feature vector, evaluate the pinned model, keep acceptable top-three SHARP/FLAT evaluations, and greedily remove overlapping-part candidates by grade
- Assign provisional alterations to slices, with intrinsic grade 0.8 times raw evaluation
- Compute sharp pitch from rounded glyph centroid; compute flat measured pitch from the original mass/area mixture (mass offset 0.65, pinned Bravura area offset 1.0559375), then ties-to-even assigned pitch
- For empty slices, crop their own pixels, erase overlapping assigned neighbor runs, retry with the original lower 0.01 intrinsic-grade threshold, and retain unresolved nulls

No Java, image-specific recognition answer, oracle output, font rasterizer or new neural model is used by the Rust runtime. Exact-grade ambiguous overlap/slice ties outside validated ordering fail explicitly rather than silently selecting a possibly different glyph.

## Independent comparisons

The first component pass matches original-method snapshots on 26 rasters, including damaged/speck variants of independent Bravura inputs. Eight cases require multipart combinations and six remove overlapping classifier candidates. These perturbations and their exact input hashes are recorded, not inferred from native outputs.

Hard fallback matches 22 original-image cases with 11 ordered attempts: nine recovered alterations and two unresolved nulls. The test compares actual final glyph geometry, score, pitch and every attempt's raw/cropped pixel hash. A separately labeled controlled overlap checks the original ByteProcessor/BufferedImage/RunTable erasure path: the same 240 pixels are removed, and all 4,972 cropped bytes match exactly.

The new pure clef/key scorer separately matches 286 original-method controls and all 14 pitch tables. This numerical decision stage is not yet a claim of live SIG mutation or complete key selection. Competing-clef graph order is kept explicit because registration order is not necessarily the order used by Staff.getCompetingClefs.

## Remaining work

Clef-conditioned pitch-window replacement, applying pitch penalties to live alterations, KeyInter ensemble construction/contextualization, FLAT-before-SHARP feedback, cross-staff key consistency, and final key/header assignment remain separate work. Time signatures and later STEM_SEEDS/BEAMS/LEDGERS, automatic heads, rhythm, MusicXML and bounded 300 DPI PDF remain unfinished.

The existing 4M-pixel cap remains. ROI/part/combination budgets and unsupported input failures are intentional; successful stage comparisons do not imply unrestricted full-score recognition.
