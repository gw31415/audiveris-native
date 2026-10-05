#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Diagnostic original-image + seed/comb overlay; never modifies the source image."""
import base64,html,json,struct,zlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];out=ROOT/'evidence/visual';out.mkdir(exist_ok=True)
def png_gray(w,h,p):
 def chunk(t,b):return struct.pack('>I',len(b))+t+b+struct.pack('>I',zlib.crc32(t+b))
 return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,0,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(b'\0'+p[y*w:(y+1)*w] for y in range(h)),9))+chunk(b'IEND',b'')
for name in ['straight','sloped','curved','broken','two-staves','laura-180dpi']:
 report=json.loads((ROOT/'evidence/generated/java-filaments'/f'{name}.json').read_text());w,h=report['width'],report['height'];d=report['filament_seeds']
 if name=='laura-180dpi':image=(ROOT/'fixtures/rendered/laura-180dpi.png').read_bytes()
 else:
  raw=(ROOT/'fixtures/sections'/f'{name}.pgm').read_bytes();pixels=raw.split(b'\n',3)[3];image=png_gray(w,h,pixels)
 uri='data:image/png;base64,'+base64.b64encode(image).decode()
 parts=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{w*2+48}" height="{h+90}" viewBox="0 0 {w*2+48} {h+90}"><rect width="100%" height="100%" fill="white"/><style>text{{font-family:Arial,sans-serif;font-size:14px}}</style>',f'<text x="12" y="20">{html.escape(name)} — original (left) / native debug overlay (right)</text>',f'<text x="12" y="42">Cyan: rectangular seeds; orange: other seeds; magenta: combs. Java matched. NOT detected staves.</text>']
 for x in [12,w+36]:parts.append(f'<image x="{x}" y="66" width="{w}" height="{h}" href="{uri}"/>')
 parts.append(f'<g transform="translate({w+36},66)">')
 for seed in d['seeds']:
  x,y,bw,bh=seed['bounds'];color='#00a7c4' if seed['comb_eligible'] else '#ed9200';parts.append(f'<rect x="{x}" y="{y}" width="{bw}" height="{bh}" fill="none" stroke="{color}" stroke-width="1.1"><title>section {seed["section_id"]}</title></rect>')
 for comb in d['combs']:
  parts.append(f'<path d="M {comb["x"]} {comb["ys"][0]} V {comb["ys"][-1]}" fill="none" stroke="#d00072" stroke-opacity=".32" stroke-width=".7"/>')
 parts.append('</g></svg>');(out/f'{name}-comparison.svg').write_text('\n'.join(parts))
print('Wrote six original/overlay SVG comparisons; inputs unchanged')
