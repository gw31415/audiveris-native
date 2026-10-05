#!/usr/bin/env python3
# SPDX-License-Identifier: CC0-1.0
# Original deterministic geometry, no fonts or copied music.
from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parents[1]/'fixtures/scale'
root.mkdir(exist_ok=True)
manifest=[]
def save(name,w,h,p,case):
    b=f'P5\n{w} {h}\n255\n'.encode()+bytes(p);(root/(name+'.pgm')).write_bytes(b)
    manifest.append(dict(file=name+'.pgm',width=w,height=h,sha256=hashlib.sha256(b).hexdigest(),case=case,license='CC0-1.0'))
def fixture(name,w=320,h=320,groups=(),boxes=(),case=''):
    p=[255]*(w*h)
    for x0,x1,y,interline,thickness in groups:
        for line in range(5):
            for yy in range(y+line*interline,y+line*interline+thickness):
                for xx in range(x0,x1): p[yy*w+xx]=0
    for x0,x1,y0,y1 in boxes:
        for yy in range(y0,y1):
            for xx in range(x0,x1):p[yy*w+xx]=0
    save(name,w,h,p,case)
fixture('clean-16',groups=[(10,310,32,16,2)],case='staff line 2 / interline 16; guessed beam')
fixture('two-populations',groups=[(10,310,16,16,2),(10,280,160,24,2)],case='two interline populations ratio 1.5')
fixture('nearby-merge',groups=[(10,310,16,16,2),(10,280,150,18,2)],case='nearby peaks must merge at ratio <1.2')
fixture('distant-discard',h=600,groups=[(10,310,16,16,2),(10,280,180,40,2)],case='secondary ratio >1.9 discarded')
fixture('dual-line',groups=[(10,160,30,16,2),(160,310,30,16,4)],case='two significant black peaks merge')
fixture('significant-beam',groups=[(10,310,30,16,2)],boxes=[(10,310,180,191),(10,310,230,241),(10,310,280,291)],case='significant black beam peak > half interline')
fixture('measured-beam',groups=[(10,310,30,16,2)],boxes=[(10,100,180,190)],case='beam local maximum reaches quorum')
fixture('blank',case='no regular lines')
fixture('single-line',boxes=[(10,310,30,32)],case='no combo population')
fixture('too-low',groups=[(10,310,32,8,1)],case='interline below 11')
fixture('too-high',h=700,groups=[(10,310,32,110,3)],case='interline above 100')
fixture('sparse',w=1000,h=1000,groups=[(10,20,32,16,2)],case='combo exists but black ratio <.001')
fixture('long-blacks',h=600,groups=[(10,310,20,16,2)],boxes=[(10,80,200,380),(10,80,400,580)],case='out-of-range combo additions ignored, not clamped')
fixture('all-black',w=80,h=100,boxes=[(0,80,0,100)],case='oversize black run excluded; no combo')
fixture('no-black-peak',groups=[(10,310,8,64,26)],case='black mode at histogram right boundary: no falling derivative/no significant black peak')
fixture('rounding-white',h=322,groups=[(10,310,30,16,2)],case='Math.rint half-even maxWhite: 80.5 to 80')
fixture('dominant-black-inverted-beam',w=1000,h=1000,groups=[(10,50,30,16,2)],boxes=[(50,990,300,370)],case='dominant isolated black length can invert signed Java beam search bounds')
fixture('two-beams',h=600,groups=[(10,310,16,16,2),(10,280,160,24,2)],boxes=[(10,110,400,412),(10,100,480,496)],case='two interlines with two local beam maxima')
(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
