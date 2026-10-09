#!/usr/bin/env python3
"""Read saved pinned Book+MusicXML, preserving actual serialized relation correspondence."""
import argparse,hashlib,json,xml.etree.ElementTree as E
from collections import defaultdict
from pathlib import Path
from zipfile import ZipFile
p=argparse.ArgumentParser();p.add_argument('book',type=Path);p.add_argument('xml',type=Path);p.add_argument('output',type=Path);p.add_argument('--fresh-curves',type=Path);a=p.parse_args()
with ZipFile(a.book) as z:
 member='sheet#1/sheet#1.xml';raw=z.read(member);sheet=E.fromstring(raw)
xroot=E.fromstring(a.xml.read_bytes());xmlparts={x.get('id'):x for x in xroot.findall('part')}
physical=defaultdict(int);endpoints=[];slurs={};counts={'book_notes':0,'xml_notes':0};roundtrip_differences=[]
def bounds(n):return [int(n.find('bounds').get(k))for k in ('x','y','w','h')]
for sy in sheet.findall('.//system'):
 system=int(sy.get('id'));nodes={int(x.get('id')):x for x in sy.findall('./sig/inters/*')}
 if not nodes:
  nodes={int(x.get('id')):x for x in sy.iter() if x.get('id') and x.tag in ('head','head-chord','rest','rest-chord','slur')}
 members=defaultdict(list);heads=defaultdict(list)
 for rel in sy.findall('.//relation'):
  if rel.find('containment') is not None:members[int(rel.get('source'))].append(int(rel.get('target')))
  sh=rel.find('slur-head')
  if sh is not None:heads[int(rel.get('target'))].append((int(rel.get('source')),sh.get('side')))
 for part in sy.findall('part'):
  pid=int(part.get('id'));staffs=part.findall('staff');staff_ids=[int(s.get('id'))for s in staffs]
  owned_slurs={int(s)for s in (part.findtext('slurs')or'').split()}
  for measure in part.findall('measure'):
   physical[pid]+=1;mi=physical[pid];xm=xmlparts[f'P{pid}'].findall('measure')[mi-1];xmlnotes=xm.findall('note')
   actual=[]
   for voice in measure.findall('voice'):
    vid=int(voice.get('id'));chords=[]
    whole=voice.findtext('whole-chord')
    if whole:chords=[int(whole)]
    else:
     for entry in voice.findall('slots/entry'):
      value=entry.find('value')
      if value.get('status')=='BEGIN':chords.append(int(value.get('chord')))
    for chord in chords:
     notes=sorted(members[chord],key=lambda n:-(bounds(nodes[n])[1]+bounds(nodes[n])[3]//2))
     actual.extend((vid,chord,n)for n in notes)
   assert len(actual)==len(xmlnotes),(pid,mi,len(actual),len(xmlnotes))
   counts['book_notes']+=len(actual);counts['xml_notes']+=len(xmlnotes)
   for index,((voice,chord,nid),note) in enumerate(zip(actual,xmlnotes)):
    head=nodes[nid];assert int(note.findtext('voice'))==voice
    expected_staff=staff_ids.index(int(head.get('staff')))+1
    assert int(note.findtext('staff')or 1)==expected_staff
    marks=note.findall('notations/slur');links=heads[nid]
    assert len(marks)==len(links),(pid,mi,nid,len(marks),len(links))
    for (sid,side),mark in zip(links,marks):
     assert sid in owned_slurs
     slur=nodes[sid];assert slur.tag=='slur' and slur.get('tie') not in ('true','1')
     assert mark.get('type')==('start'if side=='LEFT'else'stop')
     curve=[float(slur.find('curve').get(k))for k in ('x1','y1','ctrlx1','ctrly1','ctrlx2','ctrly2','x2','y2')]
     if sid not in slurs:slurs[sid]={'source_id':sid,'system':system,'part':pid,'above':slur.get('above')=='true','persisted_rounded_curve':curve,'bounds':bounds(slur),'endpoints':[]}
     row={'system':system,'part':pid,'physical_measure':mi,'measure_score_id':xm.get('number'),'note_index':index,'voice':voice,'staff_number':expected_staff,'staff_id':int(head.get('staff')),'head_bounds':bounds(head),'chord_bounds':bounds(nodes[chord]),'side':side,'source_slur_id':sid,'source_head_id':nid,'source_chord_id':chord,'attributes':dict(mark.attrib),'pitch':{x.tag:x.text for x in note.find('pitch')}if note.find('pitch')is not None else None}
     endpoints.append(row);slurs[sid]['endpoints'].append(row)
     x,y=curve[:2]if side=='LEFT'else curve[6:];cx,cy=curve[2:4]if side=='LEFT'else curve[4:6]
     staff=staffs[expected_staff-1];pts=staff.find('lines/line').findall('point');assert len(pts)==2
     (x0,y0),(x1,y1)=[(float(p.get('x')),float(p.get('y')))for p in pts];top=y0+(x-x0)*(y1-y0)/(x1-x0)
     il=int(sheet.find('scale/interline').get('main'));t=lambda v:str(round(10.0*v/il))
     row['saved_geometry_control']={'interline':il,'head_left':bounds(head)[0],'staff_top_at_end':top,'curve':curve,'above':slur.get('above')=='true','staff_line_points':[[x0,y0],[x1,y1]]}
     derived={'default-x':t(x-bounds(head)[0]),'default-y':t(top-y),'bezier-x':t(cx-x),'bezier-y':t(y-cy)}
     for k,v in derived.items():
      if mark.get(k)!=v:roundtrip_differences.append({'source_slur_id':sid,'side':side,'attribute':k,'saved_XML':mark.get(k),'rounded_Book_derivation':v})
assert len(slurs)==6 and len(endpoints)==12 and counts=={'book_notes':124,'xml_notes':124}
assert all(len(s['endpoints'])==2 and len({e['staff_id']for e in s['endpoints']})==1 for s in slurs.values())
result={'schema':'audiveris-native.original-slur-export.v1','original_revision':'9e1e55cd2746037d059345881c53e6a6754bffbd','book_sha256':hashlib.sha256(a.book.read_bytes()).hexdigest(),'book_member':member,'book_member_sha256':hashlib.sha256(raw).hexdigest(),'XML_sha256':hashlib.sha256(a.xml.read_bytes()).hexdigest(),'native_inputs_used':False,'counts':counts,'slurs':list(slurs.values()),'endpoints_in_export_order':endpoints,'rounded_Book_geometry_differences':roundtrip_differences,'qualification':'Persisted original owner/relationship and exported notation observations, not fresh runtime callbacks. Curve serialization is rounded; original XML attributes remain independent output evidence, not native input.'}
if a.fresh_curves:
 fresh=json.loads(a.fresh_curves.read_text());assert fresh['before_slurs']==[] and fresh['observer_errors']==[]
 derived_differences=[]
 for slur in slurs.values():
  expected=sorted((e['side'],tuple(e['head_bounds']))for e in slur['endpoints'])
  matches=[f for f in fresh['after_slurs'] if f['system']==slur['system'] and f['part_id_raw']==slur['part'] and sorted((e['side'],tuple(e['target']['bounds_raw']))for e in f['live_head_relations'])==expected]
  assert len(matches)==1,(slur['source_id'],len(matches));f=matches[0]
  assert f['in_actual_SIG'] and not f['removed'] and not f['tie'] and not f['manual'] and not f['abnormal']
  slur['fresh_CURVES_curve']=f['curve']
  for row in slur['endpoints']:
   control=dict(row['saved_geometry_control']);control['curve']=[v for point in f['curve'] for v in point]
   curve=control['curve'];x,y=curve[:2]if row['side']=='LEFT'else curve[6:];cx,cy=curve[2:4]if row['side']=='LEFT'else curve[4:6]
   (x0,y0),(x1,y1)=control['staff_line_points'];control['staff_top_at_end']=y0+(x-x0)*(y1-y0)/(x1-x0)
   t=lambda v:str(round(10.0*v/control['interline']))
   for k,v in {'default-x':t(x-control['head_left']),'default-y':t(control['staff_top_at_end']-y),'bezier-x':t(cx-x),'bezier-y':t(y-cy)}.items():
    if row['attributes'].get(k)!=v:derived_differences.append({'source_slur_id':slur['source_id'],'side':row['side'],'attribute':k,'saved_XML':row['attributes'].get(k),'fresh_CURVES_derivation':v})
 result['fresh_CURVES_reference']={'sha256':hashlib.sha256(a.fresh_curves.read_bytes()).hexdigest(),'scope':fresh['scope'],'joined_by':'System, Part and both side-labelled full Head bounds; no cross-run ID joins','qualification':'Independent fresh CURVES construction from original CHORDS Book; saved XML is from earlier complete original image run. No fresh PartwiseBuilder invocation. Above/below comes from the independently matched saved Book.'}
 result['fresh_CURVES_geometry_differences']=derived_differences
result['slurs']=[dict(slur,endpoints=[{k:row[k]for k in ('side','head_bounds','staff_id','source_head_id','source_chord_id')}for row in slur['endpoints']])for slur in slurs.values()]
a.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'slurs':6,'endpoints':12,'notes_compared':counts,'rounded_Book_geometry_differences':roundtrip_differences,'fresh_CURVES_geometry_differences':result.get('fresh_CURVES_geometry_differences')},indent=2))
