#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent rectangles/rasters and numerical boundaries, never native output."""
import json,math,random,sys
from pathlib import Path
cases=[]
def add(name,**kw):
    c=dict(name=name,interline=16,typical_height=8,median=[20,40,84,40],height=8,rectangles=[[20,36,64,8]])
    c.update(kw);cases.append(c)
add('perfect-beam')
for top,bottom in [(False,False),(True,False),(False,True),(True,True)]:
    add(f'side-flags-{top}-{bottom}',top=top,bottom=bottom,rectangles=[[20,36,64,8],[20,34,64,2],[20,44,64,2]])
for il in [1,2,3,4,5,6,9,10,14,16,22]:add('interline-'+str(il),interline=il)
for height in [5.6-1e-10,5.6,5.6+1e-10,8,11.2-1e-10,11.2,11.2+1e-10]:
    add('height-'+repr(height),height=height,rectangles=[[20,30,64,20]],parameter_overrides={'maxBeltBlackRatio':1.0})
for hook in [False,True]:
    for width in [9.4999999999,9.5,9.5000000001,10.5,10.5000000001,14.4999999999,14.5,14.5000000001,15.5,63.5,64.5]:
        add(f'width-{hook}-{width}',median=[20,40,20+width,40],hook=hook,rectangles=[[20,36,math.ceil(width),8]])
for count in [279,280,281,300,399,400]:
    # 50 x 8 = 400 core points; exactly 70% at count 280.
    points=[[20+i%50,36+i//50,0] for i in range(count)]
    add('core-count-'+str(count),median=[20,40,70,40],rectangles=[],pixels=points)
# 64x8 core, outer 72x12 => belt 352; use 40x8 core and 48x12 outer => belt256.
for ratio in [.4-1e-10,.4,.4+1e-10]:
    # Override permits exact equality of independently fixed 88 / 352.
    add('belt-threshold-'+repr(ratio),rectangles=[[20,36,64,8],[16,34,72,1],[16,35,16,1]],parameter_overrides={'maxBeltBlackRatio':.25+(ratio-.4)})
for d in [-1,0,1e-8,.0000562341325,.001,.1,.5,1,2]:add('distance-'+str(d),distance_impact=d)
for shift in [-30,-4,0,100,150]:add('sheet-edge-'+str(shift),median=[shift,4,shift+64,4],rectangles=[[shift,0,64,8]])
add('gray-is-not-black',rectangles=[[20,36,64,8,1]])
add('subpixel-no-core-points',median=[20.1,40.2,20.4,40.2],height=.1,typical_height=.1,interline=1,rectangles=[])
for slope in [-1,-.7,-.33333333333333,-.125,0,.125,.33333333333333,.7,1]:
    for frac in [0,.1,.5,.9]:
        for flags in [(True,True),(False,False),(True,False),(False,True)]:
            add(f'slope-{slope}-{frac}-{flags}',median=[20+frac,40+frac,52+frac,40+frac+32*slope],rectangles=[[0,0,160,96]],top=flags[0],bottom=flags[1])
r=random.Random(925177)
for i in range(24):
    x=r.uniform(0,35);y=r.uniform(25,55);width=r.uniform(8,70);dy=r.uniform(-18,18)
    add('random-raster-'+str(i),median=[x,y,x+width,y+dy],height=r.uniform(5.6,11.2),rectangles=[[r.randrange(0,100),r.randrange(10,65),r.randrange(4,30),r.randrange(2,15)] for _ in range(30)],top=bool(i&1),bottom=bool(i&2))
# Dense sloped rasters generated independently by direct y-at-x inequalities.
# These exercise nontrivial accepted grades, not just rejected geometry.
for i in range(64):
    x=r.uniform(0,28);y=r.uniform(24,62);width=r.uniform(16,85);dy=r.uniform(-18,18)
    h=r.uniform(6.0,10.8);pixels=[]
    for xx in range(160):
        yy=y+(xx-x)*dy/width
        for yy_i in range(96):
            if x<=xx<x+width and yy-h/2<=yy_i<yy+h/2:
                # Deterministic occasional holes/noise retain nontrivial grades.
                if (xx*19+yy_i*23+i)%29!=0:pixels.append([xx,yy_i,0])
    add('dense-sloped-'+str(i),median=[x,y,x+width,y+dy],height=h,rectangles=[],pixels=pixels,top=bool(i&1),bottom=bool(i&2),distance_impact=.3+(i%8)*.1)
boundary=.1**4.25
for d in [math.nextafter(boundary,-math.inf),boundary,math.nextafter(boundary,math.inf),boundary*(1-1e-12),boundary*(1+1e-12)]:
    add('distance-grade-boundary-'+repr(d),distance_impact=d)
Path(sys.argv[1]).write_text(json.dumps(cases,indent=2)+'\n')
print(len(cases))
