# Original non-crossable filter controls

Pinned source: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

The final observation has 15 actual `CurvesBuilder.filterReachableArcs` outcomes, each on one reachable Arc with detached original Sheet/System/Inter/SlurInfo/Arc owners and an explicitly supplied Skeleton non-crossable list. All agree with independently recorded source extension-rectangle, Rectangle.intersectsLine, Area.intersects and AreaUtil.intersection decisions. Four actual HeadInter.isContextuallyGood controls confirm the 0.4 threshold.

Two bounded Java invocations were used: first the source geometry calls, then the same 15 cases extended to invoke the actual private filter and Head grade controls. Neither loaded/stored a Book, ran recognition, changed original files or configuration, or ran native code. Runtime preferences/cache/home were isolated under this directory. `PROVENANCE.json` describes the final invocation. The final expected output is `observed.json`; expectations were not produced by Python or Rust.

Five cases borrow geometry from earlier saved native component endpoints and a later native before-rhythm Graph snapshot. `native-retained-diagnostic.json` names and hashes these assets. They are diagnostics and detached test inputs only; they neither describe the current failing live Graph nor supply production ownership. The remaining ten cases are generic geometry controls. Source IDs/candidate numbers only identify observation rows.

Source boundaries: CurvesBuilder.erasedInters, getExtensionBox, getLineArea, filterReachableArcs; Skeleton.CurvesCleaner.eraseShapes; PageCleaner.canHide; AbstractInter.isContextuallyGood/getGoodGrade; AreaUtil.horizontalParallelogram/intersection. The probe bypasses constructors where needed and supplies erased owners explicitly, so it does not prove Skeleton construction/erasure, all-reachable arc discovery, callbacks or recognition quality.
