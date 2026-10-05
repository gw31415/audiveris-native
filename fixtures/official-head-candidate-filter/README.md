# Original aggregateMatches and filterSeedConflicts

Fresh pinned Audiveris 5.11 on Linux, upstream
9e1e55cd2746037d059345881c53e6a6754bffbd. The observer reads no native outputs.
All 118 original range scanners retain the certified 2,224 accepted proposals
and 407 weak stemless rejections. Calling original aggregateMatches selects
454 representatives. The original non-head competitor slice leaves these
unchanged in this pre-seed-registration observation.

Twenty-one declared controls call original methods on unregistered HeadInter
objects: empty input, duplicates, stable grade ties, shape/ordinate-independent
aggregation, fixed representatives, inclusive dx/half centers, positive overlap,
IoU = 0.1, edge contact/separation, adjacent score-grade bits and multiple parallel
competitors. Competitors are in original abscissa order. Native invalid-input
controls test explicit rejection without mutation.

The original retrieval step replaces slim template bounds with actual glyph
bounds. Live seed conflict filtering therefore awaits seed glyph retrieval;
its kernel is independently certified, but the CLI flag stays false. No HeadInter
or SIG vertex is registered by this native stage. Intrinsic grades and source
candidate order remain unchanged. The original per-scanner aggregation order
must precede seed conflict filtering and range glyph retrieval.
