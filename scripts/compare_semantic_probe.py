#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1];work=root/'evidence/generated/semantic-native';work.mkdir(parents=True,exist_ok=True)
results=[]
for name in ['laura-gray','synthetic-system-bars','cclef-pair','cclef-tail','cclef-too-wide','cclef-distant','right-extending','line-root','cclef-thick-second','cclef-connected','bracket-pair','bracket-middle','bracket-small-serif']:
 binary=work/(name+'.pgm');report=work/(name+'.json')
 for p in [binary,report]:p.unlink(missing_ok=True)
 r=subprocess.run([str(root/'target/debug/audiveris-native'),'grid-candidates',str(root/'fixtures/official-grid'/(name+'.png')),str(binary),str(report)],capture_output=True,text=True)
 a=json.loads(report.read_text());print(name,'exit',r.returncode,flush=True)
 assert r.returncode==0,(name,r.stderr,a.get('line_completion'))
 native=a['line_completion']['bar_filtering'];golden=json.loads((root/'fixtures/official-grid'/(name+'-semantic.json')).read_text())
 before=next(p for p in golden['phases'] if p['phase']=='refineRightEnds')['staffs'];after=golden['phases'][-1]['staffs']
 bkeys=[p['key'] for s in before for p in s['peaks']];akeys=[p['key'] for s in after for p in s['peaks']]
 for key,expected in [('before_cclef',bkeys),('remaining_peaks',akeys)]:assert native[key]==expected,(name,key,native[key],expected)
 expected_brackets=[{'peak':p['key'],'top':'BRACKET_TOP' in p['attributes'],'middle':'BRACKET_MIDDLE' in p['attributes'],'bottom':'BRACKET_BOTTOM' in p['attributes']} for staff in after for p in staff['peaks'] if 'BRACKET_' in p['attributes']]
 assert native['brackets']==expected_brackets,(name,'brackets',native['brackets'],expected_brackets)
 for ns in native['bracket_serifs']:
  actual=next(p for phase in golden['phases'] if phase['phase']=='detectBracketEnds' for staff in phase['staffs'] for p in staff['peaks'] if p['key']==ns['peak'])
  expected=next(s for s in actual['serifs'] if s['side']==ns['side'])
  assert ns['weight']==expected['weight'] and sorted(ns['section_keys'])==sorted(expected['section_keys']),(name,ns,expected)
 for s,t in zip(native['staffs'],after):
  for key in ['id','left','right']:assert s[key]==t[key],(name,key,s[key],t[key])
 results.append({'case':name,'native_exit':r.returncode,'before_cclef':bkeys,'remaining_peaks':akeys,'cclef_removed':[p for p in bkeys if p not in akeys],'endpoints_match':True,'brackets':native['brackets'],'serifs':native['bracket_serifs'],'removed_peaks':native['removed_peaks']})
 print('matched; C-clef removed:',results[-1]['cclef_removed'],flush=True)
(root/'evidence/grid-semantic-parity.json').write_text(json.dumps({'cases':results},indent=2)+'\n')
