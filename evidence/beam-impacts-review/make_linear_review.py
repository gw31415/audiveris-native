# SPDX-License-Identifier: AGPL-3.0-or-later
import json,math,random
from pathlib import Path
r=random.Random(0x5b314)
base=Path(__file__).parent
cases=[]
def add(name,left,right):
    queries=[[x/2,y/2] for y in range(-22,23) for x in range(-22,23)]
    for x,y in left+right:
        queries.extend([[xx,yy] for xx in [math.nextafter(x,-math.inf),x,math.nextafter(x,math.inf)] for yy in [math.nextafter(y,-math.inf),y,math.nextafter(y,math.inf)]])
    cases.append(dict(name=name,left=left,right=right,queries=queries))
rect=[[-8,-6],[8,-6],[8,6],[-8,6]]
add('same-area',rect,rect)
add('opposite-orientation',rect,rect[::-1])
add('touching-side',rect,[[8,-6],[10,-6],[10,6],[8,6]])
add('zero-area',[[0,0],[1,1],[2,2]],rect)
add('self-intersecting',[[-8,-8],[8,8],[-8,8],[8,-8]],[[0,-8],[8,0],[0,8],[-8,0]])
for i in range(240):
    left=[[r.randrange(-40,41)/4,r.randrange(-40,41)/4] for _ in range(r.randrange(3,10))]
    right=[[r.randrange(-40,41)/4,r.randrange(-40,41)/4] for _ in range(r.randrange(3,10))]
    add(f'random-{i}',left,right)
(base/'linear-input.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
