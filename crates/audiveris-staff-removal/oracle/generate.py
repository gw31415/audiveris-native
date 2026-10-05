#!/usr/bin/env python3
"""Official 5.11.0 classes receive PNGs; no Rust output is used as oracle input."""
from pathlib import Path
import json,subprocess,hashlib
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT.parent/'full-java-oracle/distribution/Audiveris/Audiveris.app/Contents'
fixtures=ROOT/'fixtures';records=[]
patterns={'diagonal':[[0,255],[255,0]],'bridge':[[0,255,0],[0,0,0]],'empty':[[255]*5 for _ in range(3)],'singleton':[[0]],'order':[[255,0,255,255],[0,255,255,0],[0,0,255,0]],'split-merge':[[0,0,0,0,0],[0,255,0,255,0],[0,0,0,0,0]]}
for name,rows in patterns.items():
    im=Image.new('L',(len(rows[0]),len(rows)));im.putdata([v for row in rows for v in row]);im.save(fixtures/(name+'.png'))
cases=list(patterns)+['official-no-staff']
for name in cases:
 for orientation in ['HORIZONTAL','VERTICAL']:
    src=fixtures/(name+'.png');out=fixtures/(name+'-'+orientation+'.json')
    args=[str(APP/'runtime/Contents/Home/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true', '-cp',str(ROOT/'oracle/classes')+':'+str(APP/'app/*'),'GlyphOracle',str(src),orientation,'17','-9',str(out)]
    proc=subprocess.run(args,capture_output=True,text=True,timeout=60)
    (ROOT/'oracle'/(name+'-'+orientation+'.log')).write_text(proc.stdout+proc.stderr)
    if proc.returncode:raise RuntimeError((name,proc.returncode,proc.stderr))
    data=json.loads(out.read_text());records.append(dict(input=src.name,orientation=orientation,origin=[17,-9],glyph_count=len(data),input_sha256=hashlib.sha256(src.read_bytes()).hexdigest(),output=out.name,output_sha256=hashlib.sha256(out.read_bytes()).hexdigest(),argv=args,exit=proc.returncode))
(ROOT/'oracle/golden-manifest.json').write_text(json.dumps(dict(upstream_commit='9e1e55cd2746037d059345881c53e6a6754bffbd',cases=records),indent=2)+'\n')
print(json.dumps([{k:r[k] for k in ['input','orientation','glyph_count']} for r in records],indent=2))
