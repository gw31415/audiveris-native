# SPDX-License-Identifier: AGPL-3.0-or-later
import json,math,random
from pathlib import Path
r=random.Random(0x4b514)
cases=[]
for i in range(512):
    x=r.randrange(-32,65)/4;y=r.randrange(-48,97)/4;w=r.randrange(4,193)/4
    if i%4==0:x=math.nextafter(x,-math.inf)
    if i%4==1:y=math.nextafter(y,math.inf)
    dy=r.randrange(-32,33)/4;h=r.randrange(4,49)/4
    cases.append(dict(name=f'random-beam-{i}',median=[x,y,x+w,y+dy],height=h,dx=r.randrange(0,9),top=r.randrange(0,6) if i&1 else 0,bottom=r.randrange(0,6) if i&2 else 0))
for i in range(64):
    x=r.choice([-8.,0.,8.]);y=r.choice([-8.,0.,8.]);x2=x+r.choice([8.,16.,32.]);y2=y+r.choice([-7.,-2.,0.,3.,7.]);h=r.choice([1.,2.,3.,8.])
    y=math.nextafter(y,r.choice([-math.inf,math.inf]));y2=math.nextafter(y2,r.choice([-math.inf,math.inf]));h=math.nextafter(h,r.choice([-math.inf,math.inf]))
    cases.append(dict(name=f'ulp-beam-{i}',median=[x,y,x2,y2],height=h,dx=4,top=2 if i&1 else 0,bottom=2 if i&2 else 0))
Path(__file__).with_name('beam-geometry-input.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
