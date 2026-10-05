#!/usr/bin/env python3
# SPDX-License-Identifier: CC0-1.0
from pathlib import Path
import json,hashlib,random
root=Path(__file__).resolve().parents[1]/'fixtures/sections';root.mkdir(exist_ok=True);manifest=[]
for name in ['straight','sloped','curved','noisy','two-staves','broken','crossings','blank']:
 w,h=320,320;p=bytearray([255])*(w*h)
 if name!='blank':
  for top in ([24,170] if name=='two-staves' else [60]):
   for line in range(5):
    for x in range(10,310):
     if name=='broken' and (90<=x<102 or 170<=x<177):continue
     shift=(x-10)//25 if name=='sloped' else ((x-160)**2//2500 if name=='curved' else 0)
     for thick in range(2):p[(top+line*16+shift+thick)*w+x]=0
 if name=='noisy':
  rng=random.Random(20261002)
  for _ in range(300):p[rng.randrange(w*h)]=0
 if name=='crossings':
  for x in [60,160,260]:
   for y in range(40,148):
    for t in range(2):p[y*w+x+t]=0
 b=f'P5\n{w} {h}\n255\n'.encode()+p;(root/(name+'.pgm')).write_bytes(b)
 manifest.append({'file':name+'.pgm','sha256':hashlib.sha256(b).hexdigest(),'license':'CC0-1.0','case':name})
(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
