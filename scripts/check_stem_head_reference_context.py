#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,sys,gzip,hashlib
from pathlib import Path
from check_stem_head_neighborhood import check as prior_check
from check_stem_cache_head import read,sha
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-head-reference-context'
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used']is False and r['independent_compilations_and_runs']==2 and r['XML_adapter_controls']==394 and r['actual_loaded_original_head_references']==1408 and r['original_Double1Adapter_and_actual_HeadInter_getStemReferencePoint_called']is True and r['native_Laura_image_report_USED_ONLY_as_native_reference_composition_input']is True and r['transient_native_reference_differences']==708 and r['fresh_original_transient_raw_tally_lifecycle_observed']is False and r['complete_image_STEMS']is False
 assert r['original_sha256']==sha(F/'controls.json')and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemHeadReferenceContextProbe' and x['official_cli_step']=='HEADS' and x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only'not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items());assert len(x['fixtures'])==1 and x['fixtures'][0]['ok']and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed'and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items());assert sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256']and sha(F/'native-laura-report.json.gz')==n['native_Laura_report_gzip_sha256'];assert n['native_Laura_image_sha256']==r['input_sha256'];data=gzip.decompress((F/'native-laura-report.json.gz').read_bytes());assert hashlib.sha256(data).hexdigest()==n['native_Laura_report_raw_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');assert o['native_outputs_used']is False and o['rounding_mode']=='HALF_EVEN'and o['maximum_fraction_digits']==1 and o['minimum_fraction_digits']==0 and o['grouping_used']is False and o['loaded_HEADS_scale_before']==o['loaded_HEADS_scale_after_reference_reads']==v['loaded_context_DX']
 assert len(o['XML_adapter_controls'])==len(v['XML_adapter_controls'])==394
 for a,b in zip(o['XML_adapter_controls'],v['XML_adapter_controls']):assert all(a[k]==b[k]for k in ('xml','restored','restored_bits'))
 assert len(o['head_references'])==len(v['head_references'])==1408 and all(a['point']==b['point']for a,b in zip(o['head_references'],v['head_references']))
 assert sum(c['point']!=c['transient_point']for c in v['head_references'])==v['transient_reference_differences']==708 and v['explicit_XML_context']is True
 assert n['raw_seed_scale_report_unchanged']is True and n['old_raw_reference_API_unchanged']is True and all(v[k]is False and n[k]is False for k in ('automatic_raster_target_pipeline_complete','page_cache_SIG_complete','epilog_complete','heads_complete','stems_complete'))
 return {'status':'passed','original_XML_DX_adapter_NULL_signed_zero_ULP_fraction_controls':394,'actual_loaded_original_head_references_composed_with_native_Laura_image_report':1408,'transient_vs_explicit_restored_reference_differences':708,'explicit_XML_context_API':True,'raw_seed_scale_and_reference_API_preserved':True,'fresh_original_transient_tally_lifecycle_observed':False,'automatic_raster_target_pipeline_complete':False,'page_cache_SIG_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
