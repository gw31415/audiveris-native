#!/usr/bin/env python3
"""Extract comparison frames; never native input or a reconstruction of owners."""
import argparse,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
x=json.loads(a.input.read_text())
def frame(f):
 s=f['state'];peers=[];chosen=None
 for m in s['measures']:
  voices=[]
  for v in m['voices']:
   voices.append({k:v[k] for k in ['id','termination','excess']})
   if v['object']==f['checked_voice_object']:chosen=(m['part'],v['id'])
  peers.append({'part':m['part'],'expected':m['shared_expected'],'actual':m['shared_actual'],'excess':m['shared_excess'],'abnormal':m['abnormal'],'voices':voices})
 return {'phase':'before' if 'BEFORE' in f['stage'] else 'after','checked_part':chosen[0],'checked_voice_id':chosen[1],'expected':s['expected'],'actual':s['actual'],'excess':s['excess'],'peers':peers}
rows=[]
for idx,replay in zip([1,3],x['replays'],strict=True):
 rows.append({'system':1,'stack_index':idx,'common_termination':replay['common_termination'],'frames':[frame(f) for f in replay['frames'] if 'VOICE_CHECK_DURATION' in f['stage']]})
c=next(c for c in x['declared_controls']['cases'] if c['name']=='positive-check-updates-one-shared-excess')
v={'scope':'Actual original saved post-PAGE owners; selected original per-Voice method replay, not fresh recognition. Original reload transient terminations begin null, native live RHYTHMS may already hold final values. Test evidence only, never owner reconstruction.','original_commit':x['source_commit'],'observed_sha256':hashlib.sha256(a.input.read_bytes()).hexdigest(),'book_sha256':'a5cf0b402687b9448f6a73aad2d82a93a39dcb9001a97c60a32a1909b58f3204','replays':rows,'declared_shared_growth':{'system':1,'stack_index':3,'before':frame(c['before']),'after':frame(c['after']),'actual_last_chord_offset':'1/2','actual_last_chord_duration':'1/8','initial_transient_values_not_normalized':True}}
# Each actual frame remains a single complete record; do not duplicate the full observer output.
def encode(v,n=0):
 if isinstance(v,dict):return '{\n'+',\n'.join('  '*(n+1)+json.dumps(k)+': '+encode(val,n+1) for k,val in v.items())+'\n'+'  '*n+'}'
 if isinstance(v,list):return '[\n'+',\n'.join('  '*(n+1)+json.dumps(val,separators=(',',':')) for val in v)+'\n'+'  '*n+']'
 return json.dumps(v)
s=encode(v)+'\n';assert json.loads(s)==v;a.output.write_text(s)
