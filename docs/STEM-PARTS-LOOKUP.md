# Explicit stem-parts lookup CLI

`audiveris-native stem-parts-lookup REQUEST.json REPORT.json` selects sections
and assembles raw physical glyphs using the original 5.11 lookup contract.
The input supplies the section catalog, lookup polygon, theoretical line and
optional stump. It does not derive those from a page image. This is an executable
entry point for the collector, while automatic page STEMS remains unfinished.

For example, save this as `request.json` and run the command with a new report:

```json
{
  "schema": "audiveris-native.stem-parts-lookup-request.v1",
  "vertical_sections": [{"id": 1, "vertical": true, "first_pos": 20, "runs": [[10, 5]]}],
  "horizontal_sections": [],
  "searches": [{
    "polygon": [[19, 0], [25, 0], [25, 100], [19, 100]],
    "stump": null,
    "parameters": {
      "direction": 1,
      "slope": 0,
      "max_stem_thickness": 5,
      "max_line_section_dx": 3.3,
      "last_head_y": null,
      "theoretical_line": [20, 0, 20, 100]
    }
  }]
}
```

Each run is `[start, length]`; `first_pos` advances across runs in the declared
orientation. IDs are positive and unique within each orientation. Each search
returns `vertical_ids`, `horizontal_ids` and `raw_glyphs`, preserving selection
order and physical bounds/orientation/runs. A stump uses the existing physical
glyph JSON (`bounds`, `orientation`, `runs`). Empty catalogs/searches are valid.

Reports use `audiveris-native.stem-parts-lookup-report.v1`, declare
`lookup_geometry: caller_supplied_linear_polygons`, and keep `registered_to_sig`,
`heads_complete` and `stems_complete` false. This command does not filter/register
chunks, create cached StemInter objects, attach head relations, update shared SIG
or perform cleaner/epilog. The normal page-glyphs report remains unchanged.

Limits: 8 MB request, 32 MB encoded report, 32,768 sections, 4,096 searches,
8 million catalog pixels, 200 million catalog-pixels times searches and 1 million
output runs. Zero skew and one linear polygon per search are supported. Section
centroid Java-int overflow and existing undefined/single-pixel factory fits
refuse explicitly. Curves, polygon holes and broader special-staff/scan parity
are not certified. Input/schema/late-search refusal produces no report, and an
existing output is never overwritten. Disk I/O failures are reported normally.

Verification: actual CLI execution reproduces all 2,343 saved independent original
Laura first-system controls, including distance/head-stop ULP boundaries and all
4,459 raw glyph entries. This is distinct from same-Laura font/size controls,
independent synthetic font pages and unverified published scans.
