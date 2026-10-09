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

def observe(path):
 root=E.parse(path).getroot();out={'notes':[],'measures':[],'motions':[],'directions':[],'voice_groups':[],'core_notes':[],'core_voice_groups':[],'staccato_relations':[]}
 for pi,p in enumerate(root.findall('part'),1):
  div=None;staves=1;meter=None;keys={};clefs={}
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
     out['notes'].append(row);out['core_notes'].append(row[:-1]);out['staccato_relations'].extend(row[:-1]+(a,) for a in arts if a[0]=='staccato');groups.setdefault(row[8],[]).append(row[:8]+row[9:])
     if x.find('chord') is None and not grace:cursor+=duration
   out['measures'].append((pi,mi,meter,staves,tuple(keys.get(s,0) for s in range(1,staves+1)),tuple(clefs.get(s) for s in range(1,staves+1))))
   out['voice_groups'].extend((pi,mi,tuple(sorted(v,key=str))) for v in groups.values());out['core_voice_groups'].extend((pi,mi,tuple(sorted((n[:-1] for n in v),key=str))) for v in groups.values())
 return out
def main():
 import argparse
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--native',type=Path,required=True);p.add_argument('--original',type=Path,required=True);p.add_argument('--truth',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
 native=args.native;original=args.original
 a=observe(native);e=observe(original);metrics={}
 for category in a:
  x=Counter(a[category]);y=Counter(e[category]);metrics[category]={'native':len(a[category]),'original':len(e[category]),'multiset_equal':x==y,'missing':list((y-x).elements()),'extra':list((x-y).elements())}
 v={'categories':metrics,'native_truth_events':compare(args.truth,native),'note_voice_identifiers_exact_on_this_page':metrics['core_notes']['multiset_equal'],'zero_rest_tie_grace_image_does_not_prove_those_categories':True,'numeric_grade_parity_claimed':False,'quality_accepted':False}
 args.output.write_text(json.dumps(v,indent=2)+'\n')
 print(json.dumps({k:{f:q[f] for f in ('native','original','multiset_equal')} for k,q in metrics.items()}))

if __name__=='__main__':main()
