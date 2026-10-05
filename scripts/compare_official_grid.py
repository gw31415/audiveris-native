#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,subprocess,hashlib
from pathlib import Path
root=Path(__file__).resolve().parents[1];work=root/'evidence/generated/official-grid';work.mkdir(exist_ok=True,parents=True)
golden=json.loads((root/'fixtures/official-grid/golden.json').read_text());reports=[]
for name in ['laura-gray','synthetic-system-bars','synthetic-two-staves']:
 binary=work/(name+'.pgm');report=work/(name+'.json')
 for p in [binary,report]:p.unlink(missing_ok=True)
 r=subprocess.run([str(root/'target/debug/audiveris-native'),'grid-candidates',str(root/'fixtures/official-grid'/(name+'.png')),str(binary),str(report)],capture_output=True,text=True);assert r.returncode==(2 if name=='synthetic-two-staves'else 0),r.stderr
 native=json.loads(report.read_text());reference={x['target']:x for x in golden['cases']if x['case']==name};pixels=binary.read_bytes().split(b'\n',3)[3];sha=hashlib.sha256(pixels).hexdigest();assert sha==reference['BINARY']['output_rasters']['BINARY.png']['pixel_sha256']
 sc=native['scale']['selected'];refsc=reference['SCALE']['scale']
 for k in ['line','interline']:assert sc[k]==[int(refsc[k][v])for v in ['min','main','max']]
 assert sc['beam']['thickness']==int(refsc['beam']['main-thickness'])
 before=json.loads((root/'fixtures/official-grid'/(name+'-projection.json')).read_text());staffs=native['staff_candidates']['staffs'];assert len(staffs)==len(before)
 for a,b in zip(staffs,before):
  assert a['id']==b['staff_id']and a['left']==b['left']and a['right']==b['right']
  assert len(a['lines'])==len(b['lines'])
  for x,y in zip(a['lines'],b['lines']):assert x['points']==y['points'],(name,'initial curve mismatch')
 projections=native['bar_candidates']['staff_projections']
 for a,b in zip(projections,before):
  for k in ['projection','derivative_threshold','blank_threshold','lines_threshold','chunk_threshold','blanks','ending_blanks','peaks']:
   if a[k]!=b[k]:
    print(name,a['staff_id'],k,'MISMATCH',str(a[k])[:100],str(b[k])[:100]);raise AssertionError(k)
  for x,y in zip(a['line_thicknesses'],b['lines']):assert abs(x-y['thickness'])<1e-12
 def compare(a,b):
  if isinstance(b,dict):
   for k,v in b.items():compare(a[k],v)
  elif isinstance(b,list):
   assert len(a)==len(b)
   for x,y in zip(a,b):compare(x,y)
  elif isinstance(b,float):assert abs(a-b)<=1e-9+1e-12*abs(b)
  else:assert a==b
 sticks=json.loads((root/'fixtures/official-grid'/(name+'-sticks.json')).read_text());compare(native['bar_sticks']['staffs'],sticks)
 graph=json.loads((root/'fixtures/official-grid'/(name+'-graph.json')).read_text());compare(native['system_candidates'],graph)
 final=reference['GRID'];delta=[]
 for a,b in zip(staffs,final['staves']):
  delta.append({'staff':a['id'],'left_delta':a['left']-float(b['attributes']['left']),'right_delta':a['right']-float(b['attributes']['right']),'native_points':[len(x['points'])for x in a['lines']],'official_final_points':[len(x['points'])for x in b['lines']]})
 result={'case':name,'binary_sha256':sha,'foreground':pixels.count(0),'binary_exact':True,'scale_exact':True,'initial_staff_geometry_exact':True,'projection_and_peaks_exact':True,'raw_peak_counts':[len(x['peaks'])for x in projections],'official_grid_success':final['success'],'official_systems':final['system_count'],'native_grid_complete':False,'native_initial_systems':native['system_candidates'].get('systems',[]),'native_exit':r.returncode,'bar_stick_membership_and_geometry_match':True,'graph_stages_match':True,'final_geometry_delta':delta};print(result);reports.append(result)
(root/'evidence/official-grid-comparison.json').write_text(json.dumps(reports,indent=2)+'\n')
