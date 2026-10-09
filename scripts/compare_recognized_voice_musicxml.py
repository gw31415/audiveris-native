#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Development comparison of actual image Voice/BEGIN MusicXML; no recognition input."""
import json,hashlib
from pathlib import Path
from fractions import Fraction
from collections import Counter
import xml.etree.ElementTree as E
import sys
from compare_image_score_events import compare

def _number(value):
 try:return str(Fraction(value))
 except (ValueError,ZeroDivisionError):return value

def _sound_value(name,value):
 if name in ('damper-pedal','soft-pedal','sostenuto-pedal'):
  return {'yes':'100','no':'0'}.get(value,_number(value))
 if name in ('tempo','dynamics','divisions','pan','elevation','fine'):return _number(value)
 return value

def _sound_tree(element):
 return (element.tag,tuple(sorted((k,_sound_value(k,v)) for k,v in element.attrib.items() if element.tag!='sound' or k!='id')),(element.text or '').strip(),tuple(_sound_tree(c) for c in element if c.tag!='offset'))

def _direction_sounds(direction,part,measure,cursor,divisions):
 offset=direction.find('offset')
 base=cursor
 # A direction offset changes playback only with sound="yes"; a sound offset overrides it.
 if offset is not None and offset.attrib.get('sound','no')=='yes':base+=Fraction(offset.text)/(4*divisions)
 for sound in direction.findall('sound'):
  own=sound.findtext('offset')
  at=base if own is None else cursor+Fraction(own)/(4*divisions)
  yield (part,measure,str(at),int(direction.findtext('staff','1')),direction.findtext('voice'),_sound_tree(sound))

def _relation_attributes(element):
 return tuple((name,element.attrib[name]) for name in ('time-only','placement','orientation','line-type') if name in element.attrib)

def _relation_key(kind,element,endpoint):
 if kind=='slur':return ('number',element.attrib.get('number','1'))
 if kind=='notation_tie' and 'number' in element.attrib:return ('number',element.attrib['number'])
 pitch=endpoint[4]
 semitone=None if pitch is None else 12*pitch[2]+{'C':0,'D':2,'E':4,'F':5,'G':7,'A':9,'B':11}[pitch[0]]+pitch[1]
 return ('pitch-voice',semitone,endpoint[8],element.attrib.get('time-only'))

def _relations(events,out):
 for kind,items in events.items():
  active={}
  for _,_,_,_,key,endpoint,action,attrs in sorted(items,key=lambda x:x[:4]):
   token=(endpoint,attrs)
   if action=='start':
    if key in active:
     previous=active.pop(key)
     out['relation_issues'].append((kind,'overlapping_start',previous[0],token))
    active[key]=[token,[]]
   elif action in ('stop','continue'):
    if key not in active:
     out['relation_issues'].append((kind,'unmatched_'+action,token))
    elif action=='continue':active[key][1].append(token)
    else:
     start,continuations=active.pop(key)
     out[kind+'_relations'].append((start,token,tuple(continuations)))
   elif kind=='notation_tie' and action=='let-ring':out[kind+'_relations'].append((token,None,()))
   else:out['relation_issues'].append((kind,'unsupported_type',action,token))
  for start,continuations in active.values():out['relation_issues'].append((kind,'unclosed_start',start,tuple(continuations)))

def observe(path):
 root=E.parse(path).getroot();out={'notes':[],'measures':[],'motions':[],'directions':[],'voice_groups':[],'core_notes':[],'core_voice_groups':[],'staccato_relations':[],'measure_identity':[],'direction_sounds':[],'sound_tie_endpoints':[],'notation_tie_endpoints':[],'slur_endpoints':[],'sound_tie_relations':[],'notation_tie_relations':[],'slur_relations':[],'relation_issues':[]}
 for pi,p in enumerate(root.findall('part'),1):
  div=None;staves=1;meter=None;keys={};clefs={};relation_events={k:[] for k in ('sound_tie','notation_tie','slur')};note_index=0;occurrences=Counter()
  for mi,m in enumerate(p.findall('measure'),1):
   cursor=Fraction(0);last=None;groups={}
   for x in m:
    if x.tag=='attributes':
     if x.findtext('divisions') is not None:div=int(x.findtext('divisions'))
     if x.findtext('staves') is not None:staves=int(x.findtext('staves'))
     for k in x.findall('key'):
      for s in ([int(k.attrib['number'])] if 'number' in k.attrib else range(1,staves+1)):keys[s]=int(k.findtext('fifths','0'))
     for c in x.findall('clef'):clefs[int(c.attrib.get('number','1'))]=(c.findtext('sign'),int(c.findtext('line')),int(c.findtext('clef-octave-change','0')))
     for t in x.findall('time'):meter=(int(t.findtext('beats')),int(t.findtext('beat-type')))
    elif x.tag in ('forward','backup'):
     delta=Fraction(int(x.findtext('duration')),4*div);out['motions'].append((pi,mi,x.tag,str(delta),x.findtext('voice'),x.findtext('staff')));cursor+=delta if x.tag=='forward' else -delta
    elif x.tag=='direction':
     out['direction_sounds'].extend(_direction_sounds(x,pi,mi,cursor,div))
     for dt in x.findall('direction-type'):
      for d in dt:
       if d.tag=='dynamics':signature=('dynamics',tuple(a.tag for a in d))
       elif d.tag=='pedal':signature=('pedal',d.attrib.get('type'),d.attrib.get('line'))
       else:signature=(d.tag,)
       out['directions'].append((pi,mi,str(cursor),int(x.findtext('staff','1')),x.attrib.get('placement'),signature))
    elif x.tag=='note':
     if x.find('chord') is None:last=cursor
     grace=x.find('grace') is not None;duration=Fraction(0) if grace else Fraction(int(x.findtext('duration')),4*div)
     pitch=x.find('pitch');pitch=None if pitch is None else (pitch.findtext('step'),int(pitch.findtext('alter','0')),int(pitch.findtext('octave')))
     arts=tuple((a.tag,a.attrib.get('placement'),a.attrib.get('type', 'up') if a.tag=='strong-accent' else None) for a in x.findall('notations/articulations/*'))
     row=(pi,mi,int(x.findtext('staff','1')),str(last),pitch,x.find('rest') is not None,str(duration),grace,x.findtext('voice','1'),x.find('chord') is not None,x.findtext('type'),len(x.findall('dot')),x.findtext('accidental'),x.findtext('stem'),tuple((int(b.attrib.get('number','1')),b.text) for b in x.findall('beam')),arts)
     base_endpoint=row[:9];occurrences[base_endpoint]+=1;endpoint=base_endpoint+(occurrences[base_endpoint],);note_index+=1
     for kind,selector in (('sound_tie','tie'),('notation_tie','notations/tied'),('slur','notations/slur')):
      for relation_index,relation in enumerate(x.findall(selector)):
       action=relation.attrib.get('type');attrs=_relation_attributes(relation)
       out[kind+'_endpoints'].append((endpoint,action,attrs))
       relation_events[kind].append((mi,last,note_index,relation_index,_relation_key(kind,relation,endpoint),endpoint,action,attrs))
     out['notes'].append(row);out['core_notes'].append(row[:-1]);out['staccato_relations'].extend(row[:-1]+(a,) for a in arts if a[0]=='staccato');groups.setdefault(row[8],[]).append(row[:8]+row[9:])
     if x.find('chord') is None and not grace:cursor+=duration
   out['measure_identity'].append((pi,mi,m.attrib.get('number'),m.attrib.get('implicit','no')))
   out['measures'].append((pi,mi,meter,staves,tuple(keys.get(s,0) for s in range(1,staves+1)),tuple(clefs.get(s) for s in range(1,staves+1))))
   out['voice_groups'].extend((pi,mi,tuple(sorted(v,key=str))) for v in groups.values());out['core_voice_groups'].extend((pi,mi,tuple(sorted((n[:-1] for n in v),key=str))) for v in groups.values())
  _relations(relation_events,out)
 return out
def main():
 import argparse
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--native',type=Path,required=True);p.add_argument('--original',type=Path,required=True);p.add_argument('--truth',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
 native=args.native;original=args.original
 a=observe(native);e=observe(original);metrics={}
 for category in a:
  x=Counter(a[category]);y=Counter(e[category]);metrics[category]={'native':len(a[category]),'original':len(e[category]),'multiset_equal':x==y,'missing':list((y-x).elements()),'extra':list((x-y).elements()),'qualification':('no_pairing_issues' if category=='relation_issues' else 'both_empty_not_exercised') if not x and not y else 'observed_comparison'}
 coverage={}
 for label,counts in {
  'rests':(sum(n[5] for n in a['notes']),sum(n[5] for n in e['notes'])),
  'graces':(sum(n[7] for n in a['notes']),sum(n[7] for n in e['notes'])),
  'implicit_measures':(sum(m[3]=='yes' for m in a['measure_identity']),sum(m[3]=='yes' for m in e['measure_identity'])),
  'sound_ties':(len(a['sound_tie_endpoints']),len(e['sound_tie_endpoints'])),
  'notation_ties':(len(a['notation_tie_endpoints']),len(e['notation_tie_endpoints'])),
  'slurs':(len(a['slur_endpoints']),len(e['slur_endpoints'])),
 }.items():coverage[label]={'native':counts[0],'original':counts[1],'qualification':'both_empty_not_exercised' if not any(counts) else 'observed_comparison'}
 v={'categories':metrics,'recognition_feature_coverage':coverage,'relation_pairing_valid':not a['relation_issues'] and not e['relation_issues'],'relation_comparison_scope':'Musical note endpoints retain part, measure order, staff, onset, spelled pitch, duration, grace and voice. Number labels resolve links but are not compared as identities. Placement, orientation and line-type are compared; engraving coordinates are excluded. Unnumbered ties resolve by sounding pitch and voice; ambiguous or unpaired endpoints are reported as relation_issues.','native_truth_events':compare(args.truth,native),'note_voice_identifiers_exact_on_this_page':metrics['core_notes']['multiset_equal'],'zero_rest_tie_grace_image_does_not_prove_those_categories':True,'numeric_grade_parity_claimed':False,'quality_accepted':False}
 args.output.write_text(json.dumps(v,indent=2)+'\n')
 print(json.dumps({k:{f:q[f] for f in ('native','original','multiset_equal')} for k,q in metrics.items()}))

if __name__=='__main__':main()
