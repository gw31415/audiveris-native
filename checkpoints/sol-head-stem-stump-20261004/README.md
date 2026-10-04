# sol-head-stem-stump-20261004

Source `dcf334cbc06af596dc1a67face497594b29177d5`; tree `1cc91250a19498ed3f8e4836e6a6755ad544b1d4`; 6229 files.

Original HeadLinker.CLinker.checkStemRelation passes its nullable corner stump, not the assembled stem glyph. Native typed explicit-stump registration validates a caller-selected physical stump or declared null, checks two-part union glyph/StemInter geometry, relation gaps, physical length consistency/support and shared SIG callbacks atomically. Legacy single-part registered head-request behavior is preserved; unspecified contact on multiple-part requests now fails explicitly.6048 original explicit controls across12StemHeads/both sides and endpoints/seven vertical gaps/three horizontal offsets/physical or null stump/profiles0,1,2 match:4272 accepted/1776 rejected/3024 null. Whole-union contact incorrectly accepts1104 of those rejected requests; this substitution is prevented. Original independent source-backed runs reproduce every control byte. New matrix Bravura51pt/mainIL11 ordinary staff, explicit object controls on original Laura prefix, not automatic raster STEMS/page terminals. Prior shared IDs, phases, registry state/rollback, existing single-part support/357 dependency controls, all24 HEADS kernels/13sameLaura/three independent synthetic font pages/default352heads513exclusions are maintained. GlobalHEADS/STEMSfalse.

410 debug workspace tests pass. HEADS and full OMR remain unfinished.

Restore: `python checkpoints/sol-head-stem-stump-20261004/RESTORE.py --repository ARCHIVAL_CLONE --destination NEW_DIRECTORY`. Destination must not exist. No new credentials, downloads or toolchain setup.

See MANIFEST.json for complete hashes, validation and remaining work.
