#!/usr/bin/env python3
# SPDX-License-Identifier: CC0-1.0
"""Synthetic geometry, not real music events or labeled recognition data."""
import struct,zlib
from pathlib import Path
root=Path(__file__).resolve().parents[1]/'fixtures/official-grid'
def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
for name,extra in [('cclef-pair',0),('cclef-tail',1),('cclef-too-wide',2),('cclef-distant',3),('right-extending',4),('line-root',5),('cclef-thick-second',6),('cclef-connected',7),('bracket-pair',8),('bracket-middle',9),('bracket-small-serif',10)]:
 w,h=960,(570 if extra==9 else 420);p=bytearray([255])*(w*h)
 def rect(x,y,ww,hh):
  for yy in range(y,y+hh):p[yy*w+x:yy*w+x+ww]=b'\x00'*ww
 for y in range(120,185,16):
  if extra==5:rect(100,y,19,2);rect(122,y,739,2)
  else:rect(100,y,761,2)
 for x in [400,700,858]:rect(x,120,2,66)
 if extra<4 or extra==6:
  x=145 if extra==3 else 112
  rect(x,120,6,66);rect(x+11,120,6 if extra==6 else 5 if extra==2 else 2,66)
  if extra==1:rect(x+22,126,2,54)
 elif extra==4:rect(870,95,5,115)
 elif extra==5:rect(120,120,2,66)
 elif extra in [7,8,9,10]:
  for y in range(250,315,16):rect(100,y,761,2)
  for x in [400,700,858]:rect(x,120,2,196)
  if extra==7:
   rect(112,120,6,196);rect(123,120,2,196)
  else:
   bottom=446 if extra==9 else 316
   if extra==9:
    for y in range(380,445,16):rect(100,y,761,2)
    for x in [400,700,858]:rect(x,120,2,326)
   rect(100,120,2,bottom-120);rect(84,110,6,bottom-100)
   if extra==10:rect(90,110,2,2);rect(90,bottom+6,2,2)
   else:rect(90,110,23,4);rect(90,bottom+6,23,4)
 # Small isolated marks stabilize line/beam scale; no candidate labels supplied.
 for x in [250,300,500]:rect(x,150,12,3)
 raw=b''.join(b'\0'+p[y*w:(y+1)*w] for y in range(h))
 (root/(name+'.png')).write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,0,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b''))
