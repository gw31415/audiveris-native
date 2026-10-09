#!/usr/bin/env python3
"""Project the original actual mixed-start observation into a bounded test contract."""
from pathlib import Path
import argparse, hashlib, json
p=argparse.ArgumentParser()
p.add_argument('--input', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a=p.parse_args()
input_path, output_path = a.input, a.output
raw=json.loads(input_path.read_text());r=raw['mixed_starts'][0]
assert len(raw['mixed_starts'])==1 and r['vertex_identity_unchanged'] and r['relation_identity_manual_endpoints_unchanged']
assert r['measure_rest_chord_ids']==[]
def sort(v):return sorted(v,key=lambda x:json.dumps(x,sort_keys=True,separators=(',',':')))
def note(n):return {k:n[k] for k in ['kind','shape','bounds','manual','staff']}
def geo(c):return {'kind':c['kind'],'bounds':c['bounds'],'staff':c['staff'],'notes':sort([note(n)for n in c['notes']])}
byid={c['id']:c for c in r['before']['chords']}
def state(s):
 return {'expected':s['expected'],'abnormal':s['abnormal'],'chords':sort([dict(geo(c),manual=c['manual'],voice=c['voice'],slot=c['slot'],time_offset=c['time_offset'],duration=c['duration'])for c in s['chords']]),'voices':[{'id':v['id'],'kind':v['kind'],'starting_staff':v['starting_staff'],'is_measure_rest':v['is_measure_rest'],'chords':[geo(byid[i])for i in v['chord_ids']],'termination':v['termination'],'excess':v['excess']}for v in s['voices']]}
checks={'vertex_identity_unchanged':r['vertex_identity_unchanged'],'relation_identity_manual_endpoints_unchanged':r['relation_identity_manual_endpoints_unchanged']}
for a,b in zip(r['before']['chords'],r['after']['chords']):
 for k in ['id','object','staff_object','slot_object','notes']:assert a[k]==b[k]
 assert a['actual_measure_same'] and a['actual_part_same'] and b['actual_measure_same'] and b['actual_part_same']
 for c in [a,b]:
  if c['voice'] is not None:
   v=next(v for v in r['after']['voices']if v['id']==c['voice']); assert c['voice_object']==v['object'] and c['object'] in v['chord_objects']
checks['actual_chord_staff_slot_measure_part_and_member_identities_retained']=True
checks['actual_chord_voice_reference_equals_live_voice_object']=True
fixture={'scope':raw['scope'],'implicit_tuplets':raw['implicit_tuplets'],'system':r['system'],'part':r['part'],'physical_measure_index':r['physical_measure_index'],'printed_stack_id':r['printed_stack_id'],'measure_rest_chords':[],'starting_chords':[geo(byid[i]) for i in r['starting_chord_ids']],'source_identity_checks':checks,'before':state(r['before']),'after':state(r['after']),'raw_observation_sha256':hashlib.sha256(input_path.read_bytes()).hexdigest()}
output_path.write_text(json.dumps(fixture,indent=2)+'\n')
