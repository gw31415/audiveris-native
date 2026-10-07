# Next dependency

Unshared head/rest augmentation now reaches native Voice/Partwise output.
Original SigReducer.LinksAdapter.checkConsistencies calls checkAugmentationDots,
checkAugmentedDots, checkAugmentedHeads and checkAugmentedRests around owned
contextualization/purge. In particular, rest removal can leave an actual strong
orphan dot. Observe original relation.removed/checkAbnormal and first/second-dot
cleanup before connecting that branch; do not treat it as a guessed undotted
image. Shared/competing augmentations, repeat/staccato, special heads, shared
stacks, tuplets, lyrics/slur/tie and full cleaner/epilog remain partial.
The former negative quarter-rest image remains a single negative comparison.
