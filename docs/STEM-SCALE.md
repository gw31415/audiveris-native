# Native stem scale

`stem_scale::analyze(no_staff, &Scale)` returns a typed, serializable analysis containing stem main/max width, horizontal-run histogram, accepted-run area, ordered peaks and the selected peak, thresholds and fallback status. It finishes stem scaling only; `stem_seeds_complete` remains false.

## Actual pinned-runtime behavior

Original StemScaler obtains NO_STAFF and paints bar/header/staff-exterior erasure onto `ByteProcessor.getBufferedImage()`. In the pinned ImageJ 1.54p runtime that image is detached. The returned ByteProcessor is unchanged. All eleven original raster observations confirm identical before/focused/after byte hashes, and an independent 5×4 original control confirms that painting every BufferedImage pixel white does not change the ByteProcessor bytes. Native execution therefore reads the whole immutable NO_STAFF image, including remaining bar/header ink. It does not substitute the erasure described by source comments.

## Computation

- Scan horizontal zero-valued foreground runs over the entire binary image.
- Increment one histogram bin per complete run when its length is at most the sheet's main interline. Longer runs are omitted, never split.
- Histogram domain is `[0, interline]`; bin zero remains zero. `area` is the number of accepted runs, not black pixels.
- Quorum is half-even-rounded `0.1 × area`; derivative threshold is half-even-rounded `0.05 × area`; min value is 1 and peak-extension gain is 0.1.
- Original HiLo hysteresis, first maximum, stable ties, descending peak order and unflushed terminal state are preserved. The first surviving peak supplies main/max widths.
- With no surviving peak, default ratio 1.0 gives native `Scale.line.main` and `Scale.line.max`, matching the cached original Scale getters.

A local bounded HiLo translation is necessary here because the existing key-projection helper deliberately rejects values above 32767, while original StemScaler bins are Java ints. This implementation accepts larger bins if total area fits a signed Java int.

## Evidence and bounds

The full native PNG→GRID→HEADERS→stem-scale chain matches all eleven independent original fixtures exactly: 182 histogram bins and 25,461 accepted runs, all chosen peaks/scales, and 7,164,856 input pixels across the corpus. Sixteen additional direct original HistoKeeper/computeStem controls independently verify fallback, terminal Hi/Lo states, half-even thresholds, zero derivative, equal peak order and bins above 32767 (up to 50000, area 125001). Seven focused unit tests also exercise input validation and overflow rejection; these safety-guard expectations remain source/native-only tests. See `evidence/stem-scale/native-validation.json`, `evidence/stem-scale-source.json`, and `evidence/stem-seeds-oracle-validation.json`.

Bounds remain binary 0/255 NO_STAFF, at most 4,000,000 pixels per image, main interline 1–128, valid positive foreground scales, and signed-int histogram area without overflow. Invalid inputs return errors before pixel indexing or unchecked histogram accumulation. No image mutation, Java execution, network access, font loading or oracle lookup occurs in this stage.

The API models a fresh automatic StemScaler with default constants. Respecting an existing user-assigned stem scale belongs to StemSeedsStep orchestration. Vertical candidates, StickFactory and StemChecker are now connected separately in the [native STEM_SEEDS pipeline](STEM-SEEDS.md).
