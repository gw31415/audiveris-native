# Original retained Header subset for mixed MeasureFiller

Java revision: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

`an-laura-observed.json` is extracted without rerunning Java from the existing original `/tmp/measure-filler-clef-key-book-v1/observed.json` (SHA recorded inside). Source StaffHeader Clef/Key IDs choose the expected header subset; source actual Containment/member/Measure records provide descriptors. The separately observed nonheader Clef is outside this prerequisite. The extraction never supplies a native object, registration order or recognition decision.

The original book was saved at SYMBOLS. Loading invokes `Measure.afterReload`, which already fills existing headers. Its explicit two original MeasureFiller calls therefore establish filter/encounter-order/idempotence and final owner references, not fresh first-add callback observations. Source saved pre-reload XML independently retains no header Measure membership. Native first-add observations remain native, compared to the original filtered encounter order and final collections with this qualification.

Original source scheduling is `HeaderBuilder` complete System Clef column → Key column → Time column; LINKS begins with mixed MeasureFiller. This unit corrects the native C/K subsequence using actual registration, never by sorting Filler input. Full Time/global SIG ordering and lazy Java Part cache timing are not asserted.
