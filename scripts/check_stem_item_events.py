#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected physical source constructor/events/length CLI, not automatic image OMR."""
from pathlib import Path
import json,sys
from check_stem_length_map import check as prior_check
from check_cue_clef import read,sha
from stem_items_plan_request import make
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-item-events'
def near(a,b):
 if isinstance(b,float):assert isinstance(a,(int,float))and abs(a-b)<1e-12,(a,b)
 elif isinstance(b,dict):
  for k,v in b.items():near(a[k],v)
 elif isinstance(b,list):
  assert len(a)==len(b)
  for x,y in zip(a,b):near(x,y)
 else:assert a==b,(a,b)
def item(v):
 import math
 l=v['line'];x=math.floor(min(l[0],l[2]));y=math.floor(min(l[1],l[3]));return {**v,'gap':v['kind']=='GapItem','line_bounds':[x,y,math.ceil(max(l[0],l[2]))-x,math.ceil(max(l[1],l[3]))-y]}
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used'] is False and r['independent_compilations_and_runs']==2 and r['source_controls']==4432 and r['declared_line_boundary_controls']==18 and r['builders']==612 and r['original_constructors_sortItems_insertGapEvents_retrieveLengths_called'] is True
 assert r['original_sha256']==sha(F/'controls.json') and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemItemEventsProbe' and x['official_cli_step']=='HEADS'
  assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only' not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items())
  assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and n['line_absolute_tolerance']==1e-12 and n['actual_CLI_executed'] is True
 assert all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 for name in ('native-controls','cli-request','cli-report'):assert sha(F/(name+'.json.gz'))==n[name.replace('-','_')+'_gzip_sha256']
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');cli=read(F/'cli-report.json.gz');assert read(F/'cli-request.json.gz')==make(o);near(v['report'],cli)
 assert len(o['groups'])==612 and o['native_outputs_used'] is False
 cases=[c for g in o['groups']for c in g['cases']];assert len(cases)==len(cli['cases'])==4432
 truncated=gaps=0;modes=set();kinds=set()
 for c,d in zip(cases,cli['cases']):
  assert c['sorted_tokens']==d['events']['sorted_tokens'] and c['prefix_lengths']==d['prefix_lengths'] and c['lengths']==d['lengths'];near([item(x)for x in d['events']['items']],c['items'])
  count=sum(x['token']is not None for x in d['events']['items']);assert d['events']['truncated']==(count<len(c['sources']));truncated+=d['events']['truncated'];gaps+=sum(x['kind']=='GapItem'for x in d['events']['items']);modes.add(c['mode']);kinds.update(x['kind']for x in c['items'])
 boundaries=[c for g in o['groups']for c in g['declared_line_boundaries']];assert len(boundaries)==len(v['declared_line_boundaries'])==18
 for c,d in zip(boundaries,v['declared_line_boundaries']):near(d['items'],c['items']);assert d['prefix_lengths']==c['prefix_lengths'] and d['lengths']==c['lengths']
 assert {'actual','empty','reversed-suffix','duplicate-first-suffix'}==modes and {'HalfLinkerItem','LinkerItem','GlyphItem','GapItem'}==kinds
 assert cli['source_selection']=='caller_supplied_prefiltered_physical_glyphs_and_linkers' and all(cli[k]is False for k in ('registered_to_sig','epilog_complete','heads_complete','stems_complete')) and all(v[k]is False for k in ('automatic_image_sources_complete','page_cache_SIG_complete','epilog_complete','heads_complete','stems_complete'))
 return {'status':'passed','actual_original_builders':612,'selected_source_constructor_event_length_CLI_controls':4432,'declared_line_ULP_event_API_controls':18,'truncated_CLI_cases':truncated,'inserted_gap_entries':gaps,'physical_glyph_reference_beam_source_selection_supplied':True,'integer_bounds_item_tokens_lengths_maps_exact':True,'floating_line_absolute_tolerance':1e-12,'automatic_item_construction_sort_gap_length_CLI_connected':True,'automatic_image_source_selection_complete':False,'page_cache_SIG_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
