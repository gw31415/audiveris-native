# Work-only translation experiment, not production/native certification.
import json,math,hashlib
from pathlib import Path
w=Path('/workspace/work/stem-cleaner');p=w/'parts-original-trial/laura-gray/result.json';data=json.loads(p.read_text())
def centroid(g):
 b=g['bounds'];sx=sy=weight=0
 for position,start,length in g['runs']:
  across=position*length;along=length*(2*start+length-1)/2;weight+=length
  if g['orientation']=='VERTICAL':sx+=across;sy+=along
  else:sx+=along;sy+=across
 return b[0]+sx/weight,b[1]+sy/weight
mismatches=[];dropped=0
for i,c in enumerate(data['cases']):
 assert c['slope']==0.0
 glyphs={p['id']:p['glyph'] for p in c['parts']}
 if c['stump'] is not None:glyphs[0]=c['stump']
 working=sorted(c['input_order'],key=lambda k:glyphs[k]['bounds'][1] if c['direction']>0 else -(glyphs[k]['bounds'][1]+glyphs[k]['bounds'][3]-1))
 if c['stump'] is not None:
  if 0 in working:working.remove(0)
  working.insert(0,0)
 removed=[];j=0
 while j<len(working)-1:
  a,b=working[j:j+2];xa,ya=centroid(glyphs[a]);xb,yb=centroid(glyphs[b]);aligned=abs(yb-ya)>c['max_alignment_dy'] or abs(xb-xa)<=c['max_alignment_dx']
  if not aligned:
   drop=a if glyphs[a]['bounds'][3]<glyphs[b]['bounds'][3] else b
   removed.append(drop);working.remove(drop);j-=1
  j+=1
 retained=[k for k in c['input_order'] if k not in removed]
 if retained!=c['retained_order'] or removed!=c['removed_order']:mismatches.append({'case':i,'actual_retained':retained,'expected_retained':c['retained_order'],'actual_removed':removed,'expected_removed':c['removed_order']})
 dropped+=len(removed)
r={'status':'passed' if not mismatches else 'failed','cases':len(data['cases']),'removed_entries':dropped,'mismatches':mismatches[:12],'original_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'scope':'Work-only Python translation experiment against one original actual CLinker/filterUnaligned trial, measured slope0,dx1.65/dy44; not Rust implementation or independent original certification'}
(w/'parts-alignment-work-trial.json').write_text(json.dumps(r,indent=2)+'\n');print(r)
