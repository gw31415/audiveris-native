# Original next-unit RestChord ownership (native not implemented yet)

Original5.11.0 source9e1e55cd2746037d059345881c53e6a6754bffbd.
RestChordLifecycleProbe observes4 actual registered rest/chord SIG conditions:
partial/last-member/extensive-chord/shared-member removal, actual mean member
best grade/union bounds/head-tail positions and remaining live relations.
All original repeated deletions are idempotent. The observer compares canonical
JSON, avoiding Java arrays' reference-based Map.equals; the corrected observation
is saved. Native RestChord invalidation/dying ownership is pending, distinct from
existing empty-rest-chord cleanup after symbol reduction. Full Part/stack/lyrics
and cached head/stem extensive ownership remain separately pending.
