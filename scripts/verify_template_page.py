#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Full-raster comparison against a hash-pinned existing official distribution."""
import hashlib, json, os, struct, subprocess, sys, time
from pathlib import Path
root=Path(__file__).resolve().parents[1]
contents=Path(sys.argv[1]).resolve()
javac=Path(sys.argv[2]).resolve()
work=root/'evidence/generated/template-page'; work.mkdir(parents=True,exist_ok=True)
jar=contents/'app/audiveris.jar'
jar_hash=hashlib.sha256(jar.read_bytes()).hexdigest()
assert jar_hash=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
subprocess.run([str(javac),'-d',str(work),str(root/'scripts/java/TemplatePageProbe.java')],check=True)
summary={'upstream_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':jar_hash,'oracle':'unchanged official VerticalFilter, ChamferDistance.Short, TemplateFactory and Template; independent original raster input','automatic_mask_selection':False,'cases':[]}
for name in ['laura-gray','synthetic-system-bars']:
 source=root/'fixtures/official-grid'/f'{name}.png'
 w,h=struct.unpack('>II',source.read_bytes()[16:24])
 # Dense reproducible page coverage plus clipping/no-reference cases. Not note locations.
 coords=[(x,y) for y in range(0,h,max(1,h//12)) for x in range(0,w,max(1,w//12))]+[(-1000,-1000),(w-1,h-1),(-5,4)]
 probes=[{'shape':shape,'point_size':51,'x':x,'y':y,'anchor':anchor} for shape in ['BREVE','WHOLE_NOTE','NOTEHEAD_VOID','NOTEHEAD_BLACK'] for anchor in [None,'MIDDLE_LEFT'] for x,y in coords]
 assert len(probes)<=4096
 request=work/f'{name}-probes.json';request.write_text(json.dumps(probes))
 tsv=work/f'{name}-probes.tsv';tsv.write_text('\n'.join(f"{p['shape']}\t{p['x']}\t{p['y']}\t{p['anchor'] or 'null'}" for p in probes))
 for masked in [False,True]:
  case=f'{name}-'+('mask' if masked else 'raw')
  native=work/f'{case}-native.json'; native.unlink(missing_ok=True)
  expected=work/f'{case}-java.json';expected.unlink(missing_ok=True)
  cmd=[str(root/'target/debug/audiveris-native'),'template-probes',str(source),str(request),str(native)]
  if masked:
   mask=work/f'{name}-mask.pgm';mask.write_bytes(f'P5\n{w} {h}\n255\n'.encode()+bytes(0 if i%997==0 or i//w==h//2 else 255 for i in range(w*h)))
   cmd.append(str(mask))
  subprocess.run(cmd,check=True)
  command=[str(contents/'runtime/Contents/Home/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-cp',str(work)+os.pathsep+str(contents/'app/*'),'TemplatePageProbe',str(source),str(tsv),str(expected),str(masked).lower()]
  start=time.monotonic()
  with (work/f'{case}.log').open('w') as log:
   run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=120)
  assert run.returncode==0,(case,run.returncode)
  a=json.loads(native.read_text());b=json.loads(expected.read_text())
  for k in ['width','height','binary_sha256','distance_sha256_be_i16']:assert a[k]==b[k],(case,k,a[k],b[k])
  assert len(a['results'])==len(b['results'])==len(probes)
  for i,(x,y) in enumerate(zip(a['results'],b['results'])):
   for k in ['score','hole_white_ratio']:assert x[k]==y[k],(case,i,k,x[k],y[k])
  summary['cases'].append({'case':case,'pixels_compared':w*h,'probe_count':len(probes),'scores_exact':True,'distance_sha256_be_i16':a['distance_sha256_be_i16'],'binary_sha256':a['binary_sha256'],'input_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'distance_overflowed':a['distance_overflowed'],'java_exit':run.returncode,'java_seconds':round(time.monotonic()-start,3),'java_argv':command})
  # Small frozen regression fixture: independent expected scores and hashes, no huge distance dump.
  fixture=root/'fixtures/template-page';fixture.mkdir(exist_ok=True)
  (fixture/f'{case}.json').write_text(json.dumps({'input':name,'masked':masked,'probes':probes,'expected':b},separators=(',',':'))+'\n')
  print(case,'exact',len(probes),'scores and',w*h,'distance values',flush=True)
(root/'evidence/template-page-parity.json').write_text(json.dumps(summary,indent=2)+'\n')
