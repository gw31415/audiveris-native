# SPDX-License-Identifier: AGPL-3.0-or-later
# Synthetic pixels only; expected geometry comes exclusively from the original JAR.
import argparse, json, random
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument("output",type=Path,help="New, nonexisting destination directory")
output=parser.parse_args().output
output.mkdir(parents=True,exist_ok=False)
cases=[]
def add(name,w,h,pixels,**kw):
 rows=[]
 for x in range(w):
  ys=[y for y in range(h) if (x,y) in pixels]
  at=0
  while at<len(ys):
   start=ys[at];end=start+1;at+=1
   while at<len(ys) and ys[at]==end:end+=1;at+=1
   rows.append([x,start,end-start])
 if not rows:return
 origin=kw.pop('origin',(100,200))
 cases.append(dict(name=name,glyph=dict(bounds=[*origin,w,h],orientation='VERTICAL',runs=rows),**kw))
def rect(name,w,h,**kw):add(name,w,h,{(x,y) for x in range(w) for y in range(h)},**kw)
for w in [1,2,3,11,12,13,16,17,31,32,33,34,80]:
 for h in [1,7,8,12,15,16,17,24]:rect(f'rect_w{w}_h{h}',w,h)
for il in [3,10,17,20,64]:
 for w in [12,30,100]:rect(f'interline_{il}_width_{w}',w,12,interline=il,typical_height=5.5)
for s in [-1.5,-1.0,-0.3,-0.2999,0.0,0.3,0.3001,1.0,1.5]:
 for noise in [0,1,2]:
  w=70;h=140;pix=set()
  for x in range(w):
   y=round(60+s*(x-35));th=8+((x%3)-1)*noise
   pix.update((x,z) for z in range(y,y+th) if 0<=z<h)
  add(f'slope_{s}_noise_{noise}',w,h,pix)
for count in [2,3,4]:
 for gap in [1,3,6,10]:
  w=100;h=count*(8+gap);pix=set()
  for k in range(count):
   left=12*k;right=w-8*k
   for x in range(left,right):pix.update((x,y) for y in range(k*(8+gap),k*(8+gap)+8))
  for y in range(h):pix.add((50,y))
  add(f'stacked_{count}_gap_{gap}',w,h,pix)
for gap in [7,8,9,10,16]:
 for height in [1,2,8]:
  pix={(x,y) for x in range(100) for y in range(8) if x<40 or x>=40+gap}
  pix.update((x,y) for x in range(100) for y in range(height))
  add(f'item_gap_{gap}_bridge_height_{height}',100,8,pix)
# Both sides of the strict width and inclusive ordinate-gap comparisons.
for cutoff in [11.999999,12,12.000001]:
 rect(f'min_hook_{cutoff}',13,8,parameter_overrides={'minHookWidthLow':cutoff})
for origin in [(0,0),(8000,8000),(16000,16000)]:
 pix={(x,y) for x in range(90) for y in range(x//7,x//7+8+(x%11==0))}
 add(f'translated_{origin[0]}',90,24,pix,origin=origin)
rng=random.Random(5110)
for i in range(100):
 w=rng.randrange(16,160);h=90;pix=set()
 levels=rng.randrange(1,5)
 for k in range(levels):
  sl=rng.choice([-0.4,-0.2,0,.1,.3]);left=rng.randrange(0,min(w-1,15));right=rng.randrange(max(left+1,w-20),w+1)
  for x in range(left,right):
   start=int(12+k*14+sl*(x-w/2));height=rng.randrange(6,10)
   for y in range(start,start+height):
    if 0<=y<h:pix.add((x,y))
 if i%2==0:
  for y in range(5,min(h,levels*20)):pix.add((w//2,y))
 add(f'varied_{i:03}',w,h,pix)
json.dump(cases,(output/'controls-input.json').open('w'),separators=(',',':'))
print(len(cases),'cases')

methods=[]
def method(name,lines,h=8,overrides=None):
 c=dict(name=name,glyph=dict(bounds=[100,200,120,h],orientation='VERTICAL',runs=[[x,0,h] for x in range(120)]),initial_lines=[dict(median=m,height=height) for m,height in lines])
 if overrides:c['parameter_overrides']=overrides
 methods.append(c)
method('slopes_hook_exact_ignored',[([100,202,133,204],8),([100,210,132,218],8),([100,220,134,226],8)])
method('slopes_hook_above_included',[([100,202,133,204],8),([100,210,132.000001,218],8),([100,220,134,226],8)])
method('slopes_consecutive_long_only',[([100,201,190,210],8),([100,215,190,233],8),([100,242,190,269],8)])
for eps in [-1e-7,0,1e-7]:
 method(f'adjust_left_{eps}',[([116+eps,204,180,207],8)])
 method(f'adjust_right_{eps}',[([150,204,203+eps,207],8)])
method('extend_two_unchanged',[([100,202,200,209],8),([140,220,219,217],8)])
method('extend_three_all',[([100,201,200,205],8),([130,219,170,216],8),([150,230,219,235],8)])
for h in [12,15,16,17,24]:
 for lh in [15.999999,16,24]:method(f'split_massheight_{h}_lineheight_{lh}',[([100,200+h/2,219,200+h/2],lh)],h=h)
method('jitter_empty_trim',[([100,204,219,204],8)],overrides={'cornerMargin':60})
json.dump(methods,(output/'methods-input.json').open('w'),separators=(',',':'))
print(len(methods),'method cases')
