#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run independent original BlackHeadSizer/font controls from local pinned JAR.

Examples (the local official dependencies must already exist):
  python3 scripts/linux_black_head_oracle.py --probe black-head-control --fixtures stem-seeds-clean
  python3 scripts/linux_black_head_oracle.py --probe black-head-font-metrics

Reuses the verified Linux development-only harness and never changes production
runtime inputs. Does not fetch source files; omit --bootstrap to use local assets.
"""
import linux_oracle

linux_oracle.PROBES.update({
    "black-head-control": ("BlackHeadSizerControlProbe", "SCALE"),
    "black-head-font-metrics": ("BlackHeadFontMetricsProbe", None),
})

if __name__ == "__main__":
    linux_oracle.main()
