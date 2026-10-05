# PDF adapter integration acceptance — pending, separate commit

The parent reported the independent `audiveris-input` crate at `f164c777ec10eef338fd71414f18e5ced2e0948f`. It uses an existing external Poppler 26.09.0 executable, with page/DPI/CropBox/rotation and pixel SHA metadata. It is **not integrated** into this repository yet. Production core remains separate from rendering.

Current root decoder cap is 4,000,000 pixels. A4 at 300 DPI is approximately 8.7 million pixels, so the current cap cannot accept ordinary 300 DPI A4 PDF pages. The 180 DPI real fixture does not validate 300 DPI support. Do not silently lower DPI or remove the cap with unbounded whole-image allocation.

Before an independent integration commit:

- Preserve 300 DPI or explicit user-chosen resolution; inspect page geometry and decoded dimensions before allocation. Retain renderer/version/arguments, page box, rotation, dimensions and pixel hash.
- Establish an explicit per-page pixel/byte budget, a bounded working-memory design, and decoder limits. The VerticalFilter ring reduces integral scratch; decoded input and binary image storage still scale with page area.
- Measure worst-density run/section counts, merge work, JSON output volume, actual peak process memory and elapsed time on representative and adversarial 300 DPI pages. The filament stage currently rejects more than 5,000 horizontal sections, independently of the earlier GRID caps.
- Process multiple pages sequentially, release page allocations, preserve page-specific diagnostics, and test failure/output behavior without silently skipping pages.
- Keep Poppler an explicit renderer option, not a PDFBox compatibility claim. Parent's preliminary PDFBox 3.0.4 comparison reported 201×101 versus Poppler 202×102 for fractional dimensions, gray differences of 1 and diagonal-edge differences up to 255; this was not the official dependency version.
- Parent subsequently reports official PDFBox 3.0.6 reference commit `0fa8176e6e75e748073fe1a640b40ec8958f04e1`: 14 cases and 7 antialias repeats; RGB gray difference 1, black diagonal counts 15 versus 46, fractional size 201×101 versus Poppler 202×102, crop/rotation agreement. This is reported evidence, not yet locally integrated or accepted. Review its fixtures before integration. Mac RSS observation is a measurement, not a hard process-memory guarantee.

No new renderer installation, authentication or external publication is authorized or needed here. These acceptance requirements do not block the current pure image/GRID work.
