#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent binary scenes and supplied beam pairs; no native results are read."""
import copy
import json
import sys
from pathlib import Path

cases = []
for h1, h2, small in [(8, 8, None), (8, 8, 4), (4, 4, 4), (6, 6, 4),
                      (4, 8, 4), (8, 4, 4), (4, 4, None)]:
    for slope in [0, 0.2, -0.2]:
        for gap in [0, 4, 30]:
            for pattern in ["stripe", "gapped", "white"]:
                c = {"interline": 16, "standard_height": 8, "small_height": small,
                     "width": 128, "height": 96, "pattern": pattern, "gap": gap,
                     "first": {"median": [10, 40, 30, 40 + 20*slope], "height": h1,
                               "distance_impact": 1, "small_class": h1 == 4},
                     "second": {"median": [30+gap, 40+(20+gap)*slope, 60+gap, 40+(50+gap)*slope],
                                "height": h2, "distance_impact": 0.8, "small_class": h2 == 4},
                     "black_pixels": []}
                # Scene stripe height is supplied independently of merged geometry.
                stripe_height = {4: 4, 6: 6, 8: 8}[h1] if h1 == h2 else 6
                if pattern != "white":
                    for x in range(10, 60+gap):
                        if pattern == "gapped" and 30 <= x < 30+gap:
                            continue
                        center = 40 + (x-10)*slope
                        for y in range(96):
                            if center-stripe_height/2 <= y < center+stripe_height/2:
                                c["black_pixels"].append([x, y])
                cases.append(c)

# Grade rejection and first-input parameter selection are separate controls.
for distance in [0, 0.01, 0.5, 1]:
    c = copy.deepcopy(cases[0]); c["pattern"] = "distance_control"
    c["first"]["distance_impact"] = distance
    c["second"]["distance_impact"] = distance
    cases.append(c)
for original in list(cases):
    if original["pattern"] == "stripe" and original["gap"] == 4:
        c = copy.deepcopy(original); c["first"], c["second"] = c["second"], c["first"]
        c["pattern"] = "swapped"; cases.append(c)
c = copy.deepcopy(cases[0]); c["pattern"] = "black_core_and_belt"
c["black_pixels"] = [[x,y] for x in range(4,68) for y in range(30,50)]
cases.append(c)
Path(sys.argv[1]).write_text(json.dumps(cases, separators=(",", ":"))+"\n")
print("independent merge inputs="+str(len(cases)))
