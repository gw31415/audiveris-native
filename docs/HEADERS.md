# Native HEADERS, bounded fresh-input path

The native pipeline now connects GRID → clef registration → retained per-staff key processing → cross-staff key alignment/finalization → time candidate extraction → time filtering/system selection → final clef selection, interior-bar purge and header freezing.

The runtime is Rust. Original Audiveris 5.11.0 at `9e1e55cd2746037d059345881c53e6a6754bffbd` remains a development oracle, not a runtime wrapper. Generic Bravura metrics and the pinned classifier model are immutable native assets.

## Verification boundary

71 independent/base/damaged renderings are compared against live original `HeaderBuilder.processHeader` snapshots. Comparisons cover staff header boundaries, selected clef/key/time, full transient ranges, intrinsic/contextual grades, glyph pixels, font-normalized geometry, recursive freezing, surviving SIG node membership and relation semantics. Native IDs are local identifiers, canonicalized structurally for comparison; floating grades use absolute tolerance 1e-9. The time stage additionally has 37 original controlled cases for compatibility, ties, support, cleanup and incomplete-pair survivors.

The original CLI's persisted/reloaded snapshot is a separate check: it drops transient ranges and serializes grades to three decimals. Native precision is compared to the live snapshot, never to rounded reload grades. Existing earlier fixtures remain unchanged.

## Finalization details

- Recompute contextual clef grades from surviving key support before stable selection
- Preserve original competing-clef/exclusion order
- Purge barlines only for integer center strictly inside the header, retaining left-staff-end bars and start-position dummy bars
- Carry original left/right-end peak attributes from native GRID rather than infer them again
- Freeze selected headers and their live containment members; preserve detached header object references

## Scope and unfinished work

`headers_complete: true` describes this implemented stage only for inputs admitted by the bounded native path. It does not claim the entire Audiveris port is complete. General GRID branches remain incomplete; the current path is fresh automatic five-line monochrome input, single supported scale, maximum four million pixels. Time extraction explicitly rejects more than eight retained components instead of truncating them. The source-derived degenerate Gaussian case is also explicitly unsupported.

STEM_SEEDS is now connected as a separate native stage; see [its verification boundary](STEM-SEEDS.md). BEAMS, LEDGERS, automatic HEADS/stem attachment, rhythm, MusicXML, and bounded-memory 300 DPI PDF integration remain open. The CLI continues to expose these limits and separate incomplete downstream flags.
