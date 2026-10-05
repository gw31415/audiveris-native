# Native black-head sizing

Pinned Audiveris 5.11.0: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

`src/black_head_sizer.rs` translates original `BlackHeadSizer` from the locally
cached official JAR bytecode. Cached `MusicFont.java` supplies the original
point-size interpolation. Java is used only to generate independent development
oracles. Native recognition does not launch Java, load expected results, or call
any network service.

## Behavior

- Scale fractions with original half-even rounding, including interline-squared
  weight thresholds and unrounded 0.9-interline closing diameter
- Inclusive width/weight/height checks before and after original morphology
- Exact local glyph buffer, float-radius morphology, threshold 110 and vertical
  four-connected glyph extraction; zero/multiple closing components are discarded
- New closed glyph objects with original coordinates; passed spot pixels/order
  remain untouched
- Caller-supplied registration before the second check, even when that check
  rejects the candidate; no registration outside eligible systems still permits
  local tagging and measurement
- BLACK_HEAD_SPOT and BLACK_STACK_SPOT classification with original inclusive
  thresholds, including the untagged gap between single and stack ranges
- Quorum 20, stable width sorting, central `[n/4, 3n/4)` subset for both width
  and height, and original sample-standard-deviation arithmetic
- Bravura music-font point-size interpolation using generic pinned font metrics

The registration callback does not replace glyph pixels. It returns the canonical
identity so the caller can union tags on that object while retaining the original
input references. Repeated inputs count repeatedly in the local measurement
list, as in the original. `None` for a resulting scale below quorum means no
update, not clearing a pre-existing sheet scale.

## Independent validation

`tests/black_head_sizer.rs` compares:

- All 128 interline parameter sets, including half-even boundaries
- 728 synthetic original `checkSpot`, `closeBlackHead` and actual `process`
  controls; registration precedes the second check; both tags, disappeared and
  multiple-component closings, the untagged gap and post-close rejection occur
- Eleven original `measureSingles` lists at/below/above quorum, nonmultiple-of-four
  subset sizes, tied widths, height variation and unbiased standard deviations
- 2,048 original music-font interpolations spanning widths 0.125 through 256
- Black-head and music-font scales for all thirteen retained official sheet-spot
  lists, including Laura

The synthetic observer deliberately runs original `closeBlackHead` separately
before `process` to retain the canonical registered object for inspection. It
clears any previous groups on that retained object before each process control so
unrelated earlier controls cannot contaminate tagging observations. These are
labelled method-boundary controls; the sheet-level original full-step snapshots
provide the independent final-scale cross-check.

Full control provenance is in `fixtures/black-head-provenance.json`, including
JAR/font/dependency/source hashes and exact compile/runtime commands. Reproduce
using existing local official Linux dependencies:

```sh
python3 scripts/linux_black_head_oracle.py --probe black-head-control --fixtures stem-seeds-clean
python3 scripts/linux_black_head_oracle.py --probe black-head-font-metrics
cargo test --locked --test black_head_sizer
```

## Native bounds and remaining scope

Interline 1..128, at most 100,000 input spots, one million aggregate input runs,
and one billion aggregate morphology probes are explicit native safeguards,
not claimed original limits. Parameters/glyphs/aggregate work are validated
before the registration callback runs. The glyph area limit remains four million
pixels. Unsupported fonts or font widths retain a successful measured black-head
scale and expose an explicit missing-font-scale reason.

`assets/black-head-font-metrics.json` stores generic `NOTEHEAD_BLACK` TextLayout
widths for every integer Bravura point size 0..1024. These are transparent font
metrics from the pinned OTF, not lookup recognition of test images. The asset has
source/font hashes and uses the existing Bravura OFL attribution.

This module alone does not implement system dispatch, beam structures or
BeamInter creation, automatic HEADS/stem attachment, rhythm, or MusicXML. Those
stages must not be inferred from successful head-size measurement.


The full native [BEAMS prolog adapter](BEAM-PROLOG.md) now connects the registration callback and system dispatch. Direct measurement calls also enforce the one-million-run aggregate bound before inspecting repeated glyph references.
