#!/usr/bin/env python3
# SPDX-License-Identifier: CC0-1.0
# Original test artwork, generated for this project; no third-party score engraving.
from pathlib import Path
root=Path(__file__).resolve().parents[1]/'fixtures'
def save(name,w,h,p):
    (root/name).write_bytes(f'P5\n{w} {h}\n255\n'.encode()+bytes(p))
w,h=57,43
save('edge-noise.pgm',w,h,[(i*73+i//7*17)%256 for i in range(w*h)])
w,h=256,96
p=[225+(x*25//w) for y in range(h) for x in range(w)]
def black(x,y):
    if 0<=x<w and 0<=y<h: p[y*w+x]=20
for y in [28,36,44,52,60]:
    for x in range(8,248): black(x,y)
# Original seven-note exercise; filled heads, stems, ledger and bar lines.
for x,y in [(30,68),(60,64),(90,60),(120,56),(150,60),(180,64),(210,68)]:
    for yy in range(y-3,y+4):
        for xx in range(x-5,x+6):
            if (xx-x)**2/25+(yy-y)**2/9<=1: black(xx,yy)
    for yy in range(y-25,y+1): black(x+5,yy)
    if y>=68:
        for xx in range(x-8,x+9): black(xx,68)
for y in range(28,61): black(242,y)
save('original-exercise.pgm',w,h,p)

# Identical grayscale PNG raster, encoded without any external renderer.
import struct,zlib
def chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
(root/'original-exercise.png').write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,0,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(b'\0'+bytes(p[y*w:(y+1)*w]) for y in range(h))))+chunk(b'IEND',b''))
