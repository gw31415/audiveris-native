# Whole-mask observation correction

The historical image-observed.json/provenance used ByteProcessor(Image) by
reflection. Original SymbolsFilter uses ByteProcessor(BufferedImage), which
shares the TYPE_BYTE_GRAY raster. The Image overload left this observation's
array entirely zero:3,132,856 zero bytes. Its pixel hash26435b8f... is not a valid
original cleaned image and must not support native mask or musical quality claims.
The historical partition/grades/ensemble observations happened before erasure and
remain identical to the corrected observation; optional glyphs are zero in both.

The corrected observer explicitly selects BufferedImage. The corrected source
mask hash is80cd3d702ca8b68c6ac3123f1e1cb88dcbe5eec1b61f8aa729897cd5c9f1ae79.
Original foreground82,837 pixels; current native foreground82,728. Native erases
109 additional pixels, all near Heads. Source Head templates are51pt, while native
uses its retained catalog. Integer pointSize/4 before margin rounding versus
floating division is a suspected remaining difference
is pending; whole native mask equality is not certified. Historical raw observation
and source commit remain preserved and qualified, rather than rewritten as success.
