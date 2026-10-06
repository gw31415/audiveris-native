# Original BeamHeadCleaner grade lifecycle

Original5.11.0 source9e1e55cd2746037d059345881c53e6a6754bffbd.
BeamHeadCleanerProbe calls original SIG contextualization, then the actual
BeamHeadCleaner.process twice on one declared two-vertex support condition.
It removes the actual edge while preserving intrinsic/contextual grades and
vertices. This is separate from complete original full-image post-LINKS SIG
comparison. The old native extra contextualization dropped head ctx from
0.5098039215686275 to0.4; the new regression reproduced that mismatch.
No new raster page or symbol admission.
