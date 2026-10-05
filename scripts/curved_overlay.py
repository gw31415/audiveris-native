#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Separate sampled-curve debug layer over unchanged embedded fixture pixels."""
import json,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];out=ROOT/'evidence/visual'
for name in ['straight','sloped','curved','broken','two-staves','laura-180dpi']:
 original=ET.parse(out/f'{name}-comparison.svg').getroot();uri=original.find('{http://www.w3.org/2000/svg}image').attrib['href']
 r=json.loads((ROOT/'evidence/generated/java-curved'/f'{name}.json').read_text());w,h=r['width'],r['height'];d=r['filaments'];keep=set(d['line_filtering']['retained_ids'])
 p=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{w*2+48}" height="{h+90}"><rect width="100%" height="100%" fill="white"/><style>text{{font:14px Arial,sans-serif}}</style>',f'<text x="12" y="20">{name}: original / merged curves. {len(d["filaments"])} filaments, {len(keep)} after line filters.</text>','<text x="12" y="42">Cyan: retained; red: rejected; magenta: combs. Sampled curves only. NOT Staff detections.</text>']
 for x in [12,w+36]:p.append(f'<image x="{x}" y="66" width="{w}" height="{h}" href="{uri}"/>')
 p.append(f'<g transform="translate({w+36},66)">')
 for f in d['filaments']:
  if f['sample_y'] is None:continue
  color='#00a7c4' if f['id'] in keep else '#e24b35';points=' '.join(f'{x:.5f},{y:.5f}' for x,y in f['sample_y']);p.append(f'<polyline points="{points}" fill="none" stroke="{color}" stroke-width="1.8"><title>filament {f["id"]}; sections {f["section_ids"]}</title></polyline>')
  for x,y in f['points']:p.append(f'<circle cx="{x:.5f}" cy="{y:.5f}" r="1.5" fill="{color}"/>')
 for c in d['combs']:p.append(f'<path d="M {c["x"]} {c["ys"][0]} V {c["ys"][-1]}" fill="none" stroke="#bc1676" stroke-opacity=".3" stroke-width=".7"/>')
 p.append('</g></svg>');path=out/f'{name}-curved-merged.svg';path.write_text('\n'.join(p));ET.parse(path)
print('Six sampled merged-curve overlays written and XML-validated; original PNG unchanged')
