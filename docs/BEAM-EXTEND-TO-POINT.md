# Native point-extension prerequisite

`SystemCandidates::extend_to_point` ports the original private `BeamsBuilder.extendToPoint` at Audiveris commit `9e1e55cd2746037d059345881c53e6a6754bffbd`. It is the accepted replacement boundary called by `extendToStem`; stem seed selection and the extension loop are still pending.

The port preserves LEFT/RIGHT signed gaps (reject <= 1), the original side parallelogram, full-area versus clipped-image foreground counts, the 0.6 foreground gate, selection of standard/small parameters by the original beam height (ties use standard), the original distance impact, grade evaluation, register/append-before-remove ordering, and incident exclusion removal. Rejection preserves the typed state and shared registry.

Original `dcmpg` followed by `ifge` accepts a NaN ratio when the sampled area has no integer points. The 1.0001-pixel controls caught this edge case; native accepts an empty mask before the ordinary grade evaluation, matching the original method. The first failed check is retained outside the committed source at `/workspace/audiveris-sol/point-initial-failure`.

38 independently generated controls invoke the unmodified original private method in a real Book/Sheet/Grid/SIG with the pinned official JAR. 17 accept. Controls cover both sides, inward/zero/one/fractional gaps, absent gap foreground, distance impacts, standard/small parameter selection and ties, candidate lifecycle, unrelated peer retention, and rejection state. The observer seeds registration on an all-black image, then restores the independent supplied image before the original extension invocation. Native fixtures are not used to manufacture original outputs.

Native finite geometry and existing resource limits apply, including a 100,000,000 bounding-box-cell mask budget. VIP diagnostic attachments are omitted. This API is not wired into the CLI or full extension loop yet. No claim is made for full BEAMS, hooks, groups, notes, rhythm, or MusicXML. The prior prefix still stops at the unimplemented stem branch; no false stub was introduced.
