#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
from pathlib import Path
import gzip,hashlib,json,subprocess,sys
from check_stem_image_head_items import compare as compare_items
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-image-head-expansion'
BASE='3d2482fe6e28a1912c3bc03dfbe53cd628092e47'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def key(c):return tuple(c['head_bounds'])+(c['shape'],c['side'],c['vertical'])
def same(a,b,path=''):
 if isinstance(a,dict):
  for k,v in a.items():same(v,b[k],path+'/'+k)
 elif isinstance(a,list):
  assert len(a)==len(b),(path,len(a),len(b))
  for i,(x,y) in enumerate(zip(a,b)):same(x,y,path+'/'+str(i))
 elif isinstance(a,float):assert abs(a-b)<=1e-12,(path,a,b)
 else:assert a==b,(path,a,b)
def compare(o,n):
 assert o['native_outputs_used'] is False
 assert o['head_corners']==n['corner_count']==1408 and o['accepted_creations']==n['accepted_creations']==409
 assert len(o['systems'])==len(n['systems'])==2
 for a,b in zip(o['systems'],n['systems']):
  assert a['system']==b['system'] and a['cache_size']==b['cache_size']
  assert len(a['cases'])==len(b['cases']);expected={key(c):c for c in a['cases']}
  assert len(expected)==len(b['cases'])
  for c in b['cases']:same(expected[key(c)],c)
 assert [s['cache_size'] for s in n['systems']]==[43,51]
 assert all(n[k] is False for k in ('automatic_raster_pipeline_complete','final_stem_glyph_selection_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
 compare_items(read(ROOT/'fixtures/official-stem-image-head-items/controls.json'),n['head_items_plan'])
 return {'status':'passed','head_corners':1408,'accepted_creations':409,'system_cache_sizes':[43,51],'parts_endpoints_relations_physical_stem_grades_and_cache_reuse_match':True,'XML_inspection_replay_before_original_linking_and_epilog':True,'full_original_image_scheduler_and_SIG_attachments_verified':False,'heads_complete':False,'stems_complete':False}
def historical():
 # Old reports keep their recorded source hashes. Verify those against the
 # published parent Git objects instead of silently rebasing their provenance.
 h=read(F/'historical-source-sha256.json');g=ROOT.parent.parent/'source-git'
 assert subprocess.check_output(['git','--git-dir='+str(g),'rev-parse',BASE+'^{tree}'],text=True).strip()=='be0e81f3e7738a7af45543b249040b50495dfcf8'
 checked=set();artifacts=0
 for name,digest in h.items():
  if name.startswith('fixtures/'):
   assert sha(ROOT/name)==digest,name;artifacts+=1
 for p in (ROOT/'fixtures').glob('*/native-reproduction.json'):
  d=read(p)
  for name,digest in d.get('native_source_sha256',{}).items():
   assert h[name]==digest,(str(p),name)
   if name not in checked:
    b=subprocess.check_output(['git','--git-dir='+str(g),'show',BASE+':'+name]);assert hashlib.sha256(b).hexdigest()==digest,name;checked.add(name)
 return {'status':'passed','historical_source_commit':BASE,'historical_source_tree':'be0e81f3e7738a7af45543b249040b50495dfcf8','unchanged_historical_fixture_files':artifacts,'old_native_source_hashes_verified_against_parent_Git_objects':len(checked),'old_reports_relabelled_as_current_source':False}
def check(path):
 o=read(F/'controls.json');r=read(F/'native-reproduction.json');n=read(F/'native-stems-report.json.gz');proof=read(F/'ownership-proof.json')
 for name,digest in r['current_production_source_sha256'].items():assert sha(ROOT/name)==digest,name
 assert sha(F/'native-stems-report.json.gz')==r['native_report_gzip_sha256']
 assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_report_raw_sha256']
 assert sha(path)==n['native_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
 assert r['actual_current_image_CLI_executed'] and r['HEADS_bytes_unchanged']
 assert proof['status']=='passed' and proof['full_private_Graph_unchanged'] and proof['all2335_preexisting_registry_run_allocations_retained'] and proof['same_actual_cache_entries_reused_with_white_image_and_changed_profile']
 p=[read(F/x) for x in ('controls-provenance.json','controls-certified-provenance.json')]
 for x in p:
  assert x['probe']=='StemImageHeadExpansionProbe' and x['release']=='5.11.0' and x['official_cli_step']=='HEADS'
  assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert 'compile-only' not in x['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in x['observer_source_sha256'].items())
  assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['result_sha256']==sha(F/'controls.json')
  assert x['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','dependency_jar_sha256','resource_sha256','compiler_executable_sha256','runtime_executable_sha256'):assert p[0][k]==p[1][k]
 result=compare(o,n);result['historical_source_evidence']=historical();result['original_update_line_boundary_conditions']=len(o['update_controls']);return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
