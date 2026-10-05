# Whole score-page rasters in actual official fonts

Seven new complete synthetic page images are independently drawn from original Audiveris 5.11 font resources. They are not changes to Laura's HEADS parameters and are not scans of published scores. All font-specific headers and head symbols are drawn in the named font. The renderer uses original TemplateSymbol for heads, original MusicFont TextLayout for headers and explicit black staff/stem lines. A second render reproduces all seven PNG file hashes. Renderer/JAR/compiler/runtime/font pins are saved.

Three ordinary two-staff, five-line pages reproduce original complete HEADS physical glyphs, ordered exclusions, staff notes, small-beam contexts, prior SIG preservation and head-seed epilog:

| Page | Heads | HEADS exclusions | Match |
|---|---:|---:|---|
| Bravura-five | 64 | 82 | exact terminal contract |
| Leland-five | 60 | 68 | exact terminal contract |
| FinaleJazz-five | 58 | 130 | exact terminal contract |

The original observer runs unmodified original GRID/HEADERS/STEM_SEEDS/BEAMS/LEDGERS with default upstream font behavior, then selects the named HEADS catalog at 51pt; the Rust development control follows that same contract. This verifies HEADS terminals on new actual-font rasters, not all intermediate upstream metrics or globally selected font behavior through complete OMR. Existing thirteen same-Laura HEADS-only controls are preserved separately.

Four target pages remain outside native full-page coverage. Original explicit processing switches are set before GRID and actual staff line count/interline/drum/tab/small state is exported. These are not compared with default native settings as equivalent configurations:

| Page/configuration | Original heads/exclusions | Native dependency |
|---|---:|---|
| Bravura-cue (one 7px small staff + three 11px main staffs) | 168/198 | GRID clusters second pass for multiple interlines |
| Bravura-one-drum (oneLineStaves + drumNotation + fiveLineStaves) | 91/286 | One-line GRID construction/switch propagation; drum HEADS catalog propagation |
| Bravura-five-drum (drumNotation + fiveLineStaves) | 117/329 | HEADERS percussion state/switch propagation; drum HEADS catalog propagation |
| Bravura-six-tab (sixStringTablatures + fiveLineStaves) | 0/0 | Six-line tablature GRID construction and HEADS skip |

The six-line drawing has a G clef because original Shape has no TAB_CLEF glyph. With the declared switches the original recognizes a six-line TabStaff and no heads; the accompanying five-line drawing is not retained by original GRID in this fixture. This is an observed tablature skip target, not mixed tab/ordinary staff coverage. Initial default-switch special-page matches were deliberately excluded from the parity receipts: original also ignored/reinterpreted special drawings under those defaults. Initial equal-count cue raster was rejected at SCALE by both implementations; the saved dominant-main cue page is accepted by original but exposes the native second-pass dependency.

The CLI now propagates an explicit unsupported staff-clustering dependency before later GRID processing; unsupported cue outputs are not written. Integration tests execute all three ordinary font pages and compare against independent original observations; the cue test checks main/small scales, earliest blocked cluster stage and both CLI modes without success artifacts. HEADS/STEMS/full OMR completion remains false. Further sizes, real scans, globally changed upstream fonts, custom drum maps, arbitrary symbols/notation, and special-staff native whole-page parity remain pending.

Later cue GRID dependency progress is recorded in `../official-grid-second-pass` and `../official-grid-cue-completion`. The native gap labels and comparisons in this fixture set are historical records from its original checkpoint; cue HEADS remains unverified, but the current GRID diagnostic CLI now handles its bounded 7/11 layout and the page CLI reaches the explicit mixed KEY guard. This does not relabel generated pages as real scans.
