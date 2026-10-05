#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify actual synthetic cue CLEF state, with separate historical baselines."""
import gzip, hashlib, json, math, sys
from pathlib import Path
from check_grid_cue_completion import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-cue-clef'
def read(p): return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def near(a,b):
 if isinstance(b,dict):
  for k,v in b.items():
   if k!='id': near(a[k],v)
 elif isinstance(b,list):
  assert len(a)==len(b)
  for x,y in zip(a,b):near(x,y)
 elif isinstance(b,(int,float)) and not isinstance(b,bool):assert math.isfinite(a) and abs(a-b)<1e-9,(a,b)
 else: assert a==b,(a,b)
def check_cue(path):
 a=read(path)
 for n,key in [('candidates','clef_candidates'),('registration','clef_registration')]:near(a[key]['staffs'],read(F/(n+'.json'))['staffs'])
 assert len(a['clef_registration']['staffs'])==4 and a['clef_registration']['staffs'][0]['clef_stop']==119
 assert a['clef_registration']['staffs'][0]['inters'][0]['bounds']==[101,139,19,49]
 assert a['clef_registration']['headers_complete'] is False
 return {'status':'passed','scope':'One independent synthetic cue raster; candidate and registration observers only','staffs':4,'small_staff_clef_stop':119,'small_staff_clef_bounds':[101,139,19,49],'cue_HEADS_terminal_parity':False,'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['native_outputs_used'] is False and r['status']=='passed'
 assert r['input_sha256']==sha(ROOT/'fixtures/official-head-actual-pages/images/Bravura-cue.png')
 for row in r['cases']:
  n=row['name'];assert sha(F/(n+'.json'))==row['sha256'] and row['original_fields_byte_reproduced']
  a=read(F/(n+'-provenance.json'));b=read(F/(n+'-certified-provenance.json'))
  for p in [a,b]:
   assert p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and p['release']=='5.11.0'
   assert all(sha(ROOT/n)==h for n,h in p['observer_source_sha256'].items()) and 'compile-only' not in p['runtime_classpath']
   assert len(p['fixtures'])==1 and p['fixtures'][0]['input_sha256']==r['input_sha256'] and p['fixtures'][0]['ok'] and p['fixtures'][0]['exit_code']==0
  for k in ['observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256']:assert a[k]==b[k]
 p=read(F/'native-reproduction.json');assert p['status']=='passed' and all(sha(ROOT/n)==h for n,h in p['native_source_sha256'].items()) and sha(F/'native.json.gz')==p['native_report_gzip_sha256']
 return {'status':'passed','independent_synthetic_cue_CLEF':check_cue(F/'native.json.gz'),'current_cue_page_CLI_dependency':'unsupported_key_mixed_specific_interline','actual_published_scan_parity':False,'full24_HEADS_page_propagation':False,'automatic_STEMS_complete':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check_cue(Path(sys.argv[1])) if len(sys.argv)>2 else check(Path(sys.argv[1])),indent=2))
