Direct unchanged Audiveris 5.11 HeadStemRelation dependency kernels, using
existing Linux oracle/JAR with default constants and no native inputs.

264 portion controls cover six head heights, normal/slanted/short/degenerate/
reversed stem endpoints, exact midpoint and endpoint margins plus adjacent
values. 20 consistency controls cover ordinary/small heads, default neutral
length2.8, exact/adjacent lengths and zero; original zero-length small consistency
is positive Infinity, explicitly encoded as a string only in this observer JSON.
The native floating-point API preserves that original result and requires the
future caller to handle it before JSON serialization. 70 invading controls cover
signed horizontal/vertical interline-fraction gaps and inclusive dx<=0.05,dy<=0
boundaries. Exact gap limits are certified for profiles0,1,2; further profiles
fail explicitly in the native API rather than implying wider coverage.

    python scripts/export_head_stem_kernel_oracle.py \
      --oracle-home /workspace/onboarding/linux-oracle --output NEW_DIRECTORY

No downloads or toolchain setup. Reflection avoids linking JDK21 compilation to
the original JDK25 class-file version; execution still calls the unchanged JAR
using its existing runtime. Provenance pins the original JAR, runtime, compiler,
observer and controls. A second source-backed run reproduced all kernel fields.

The Rust public module provides gap_limits, stem_portion, consistency and
is_invading, with explicit finite/geometry budgets and no SIG mutation. It does
not select/assemble stem glyphs, compute full relation grade, attach stems,
contextualize HeadStemRelation support, clean head stems or run complete STEMS.
It is not wired into the default CLI, whose certified outputs remain unchanged.
Whole HEADS and STEMS are unfinished; actual other-font/special-staff page parity
and remaining HEADS shape catalogs are still required.
