# Der Winter real score image

`der-winter-gray.png` is the complete first and only page of Corona Schröter's
*Der Winter*, from the retained CC0 OpenScore/Lieder transcription
[`lc6053913.mscx`](../openscore-lieder/lc6053913.mscx). The unchanged score has
three staves, nine measures per staff, 115 pitched notes and nine rests.
The source and its [CC0 license](../openscore-lieder/LICENSE.txt) remain in their
original fixture directory. The image inherits that CC0 source provenance.

The actual renderer was **Linux MuseScore 3.2.3**, Debian package
`3.2.3+dfsg2-19`, extracted into the existing build-tools directory with its
recorded dependencies. The score's embedded authoring version is 3.4.2; that is
not the renderer version. The score was neither edited nor saved by the renderer.
This is a development fixture preparation dependency, not part of the native
image recognition command.

The successful historical render used `-s -m -c <private-prefs> -r 180 -o
<output>/der-winter.png`. It produced an RGBA PNG with black RGB under transparent
pixels. Alpha compositing over opaque white, then Pillow `convert('L')`, produced
the saved grayscale PNG without crop, resize or notation changes:

- Dimensions: 1489 × 2104; PNG mode L; no ancillary metadata.
- PNG SHA-256: `10f61bf0d44be190df7177b57c1cd26a31bfbcc68d9ad8f85a76813d51ae59e0`.
- Opaque pixel SHA-256: `d2b9a3209383e23e3a226081a9f996f9115506bd5dcb2e85ca55a2ea559ebf23`.

`PROVENANCE.json` records source, renderer, raw-image and normalized-image hashes.
`HISTORICAL-RENDER.json` preserves the existing successful command result;
`RENDERER-PACKAGES.json` preserves package download hashes. Historical environment
values in `PROVENANCE.json` were supplied by the recorded session, not inferred
from the historical result file. `NORMALIZATION-CHECK.json` is a fresh, successful
normalization-only check of the existing raw RGBA output: the PNG bytes match.

With the existing extracted renderer and Pillow available, the fixture script
uses private temporary config/cache/runtime directories and does not install
packages or modify persistent settings:

```sh
python3 scripts/prepare_real_rest_fixture.py --output /tmp/der-winter-fixture
```

The output directory must be new. `--source` and `--renderer-home` can select an
explicit score or existing tool directory. To check the recorded normalization
without rerendering:

```sh
python3 scripts/prepare_real_rest_fixture.py \
  --raw-png /tmp/real-rest-score-render-v1/der-winter-1.png \
  --output /tmp/der-winter-normalization
```

The script's normalization path was exercised; its fresh renderer invocation was
not repeated in this unit. It checks the reference pixel hash for the retained
score and records the PNG byte hash separately, since compression-library changes
can preserve pixels while changing PNG bytes.

The original Audiveris baseline is in
[`evidence/real-rest-quality-baseline`](../../evidence/real-rest-quality-baseline/README.md).
Neither source notation nor original MusicXML is a native recognition input.
Native parity on this image remains unverified.
