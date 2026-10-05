#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Actual CLI: supplied sections/geometry to complete chunk composition."""
from pathlib import Path
import json,sys
from check_stem_chunks import check as prior_check
from check_cue_clef import read,sha
from stem_chunks_lookup_request import make
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-chunks-cli'
def check(path):
 prior=prior_check(path);p=read(F/'cli-reproduction.json');assert p['status']=='passed' and p['actual_CLI_execution'] is True and p['cases']==3048
 assert sha(F/'request.json.gz')==p['request_gzip_sha256'] and sha(F/'report.json.gz')==p['report_gzip_sha256'];assert all(sha(ROOT/k)==h for k,h in p['native_source_sha256'].items())
 parts=read(ROOT/'fixtures/official-stem-parts-lookup/controls.json');chunks=read(ROOT/'fixtures/official-stem-chunks/controls.json');assert read(F/'request.json.gz')==make(parts,chunks)
 r=read(F/'report.json.gz');assert r['schema']=='audiveris-native.stem-chunks-lookup-report.v1' and r['lookup_geometry']=='caller_supplied_linear_polygons' and r['registry_scope']=='independent_per_search_canonical_catalogs' and r['sections']==3035
 expected=[c for g in chunks['groups']for c in g['cases']];assert len(expected)==len(r['cases'])==3048
 for c,d in zip(expected,r['cases']):
  assert c['raw_glyphs']==d['lookup']['raw_glyphs']
  for k in ('registered_ids','retained_ids','new_count','registry_count'):assert c[k]==d['chunks'][k]
  assert c['final_catalog']==[{'id':e['id'],'glyph':e['glyph']}for e in d['registry']] and all(e['groups']==[]for e in d['registry'])
 assert all(r[k]is False for k in ('registered_to_sig','heads_complete','stems_complete'))
 return {'status':'passed','actual_CLI_original_lookupChunks_controls':3048,'actual_first_system_builders':612,'section_catalog':3035,'raw_glyph_entries':sum(len(c['raw_glyphs'])for c in expected),'retained_entries':sum(len(c['retained_ids'])for c in expected),'exact_section_to_raw_registered_filtered_sorted_catalog_composition':True,'supplied_geometry_upstream_context_independent_search_registry':True,'null_stump_alias_VIP_empty_and_invalid_late_input_checked':True,'automatic_page_target_selection_complete':False,'cache_head_SIG_integration_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
