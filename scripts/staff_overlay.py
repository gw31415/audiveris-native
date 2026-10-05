#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,xml.etree.ElementTree as ET
from pathlib import Path
root=Path(__file__).resolve().parents[1];out=root/'evidence/visual'
uri=ET.parse(out/'laura-180dpi-comparison.svg').getroot().find('{http://www.w3.org/2000/svg}image').attrib['href']
r=json.loads((root/'evidence/generated/java-clusters/laura-180dpi.json').read_text());w,h=r['width'],r['height'];staffs=r['staff_candidates']['staffs']
s=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{w*2+48}" height="{h+90}"><rect width="100%" height="100%" fill="white"/><style>text{{font:18px Arial,sans-serif}}</style>', '<text x="12" y="24">An Laura: original / initial native Staff geometry (six staffs; systems not yet computed)</text>', '<text x="12" y="50">163 lines → 121 after network → 8 clusters → 6 five-line survivors → 6 initial Staffs. Polyline debug view.</text>']
for x in [12,w+36]:s.append(f'<image x="{x}" y="72" width="{w}" height="{h}" href="{uri}"/>')
s.append(f'<g transform="translate({w+36},72)">')
for i,staff in enumerate(staffs):
 color=['#007aa3','#d73e28','#8a42bb','#25853a','#c36600','#cf287e'][i%6]
 for line in staff['lines']:
  points=' '.join(f'{x:.5f},{y:.5f}'for x,y in line['points']);s.append(f'<polyline points="{points}" fill="none" stroke="{color}" stroke-width="2.5"/>')
 x,y=staff['lines'][0]['points'][0];s.append(f'<text x="{x}" y="{y-10}" fill="{color}">Staff {staff["id"]}</text>')
s.append('</g></svg>');p=out/'laura-initial-staffs.svg';p.write_text('\n'.join(s));ET.parse(p);print(p)
