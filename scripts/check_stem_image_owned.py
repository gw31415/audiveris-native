#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
from pathlib import Path
import hashlib,json,sys
from check_stem_image_head_items import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-image-owned'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(path):
    prior=prior_check(path)
    n=json.loads((F/'native-ownership-proof.json').read_text())
    reproduction=json.loads((F/'native-reproduction.json').read_text())
    assert reproduction['status']=='passed' and reproduction['actual_in_process_image_CLI_executed']
    for name,digest in reproduction['native_source_sha256'].items():assert sha(ROOT/name)==digest,name
    assert reproduction['ownership_proof_sha256']==sha(F/'native-ownership-proof.json')
    assert n['status']=='passed' and n['actual_native_image_pipeline_executed']
    assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    assert n['entire_old_HEADS_report_sha256']==sha(path)=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
    assert not n['serialized_Graph_reconstructed']
    assert n['upstream_physical_run_allocations_retained']==1220 and n['final_registry_count']==2335
    assert n['upstream_registry_entries_groups_and_IDs_exact'] and n['actual_private_Graph_history_retained']
    assert n['initialized_cache_systems']==[1,2]
    assert (n['corner_count'],n['final_head_item_count'],n['gap_item_count'])==(1408,3076,92)
    for k in ('cache_entries_or_stem_SIG_vertices_created','physical_endpoint_stem_glyph_grade_selection_complete','page_cache_sig_complete','cleaner_epilog_complete','heads_complete','stems_complete'):assert n[k] is False
    for name in ('original-first-provenance.json','original-second-provenance.json'):
        p=json.loads((F/name).read_text())
        assert p['release']=='5.11.0' and p['probe']=='StemImageHeadItemsProbe' and p['official_cli_step']=='HEADS'
        assert p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
        assert p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert 'compile-only' not in p['runtime_classpath']
        assert all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
        assert len(p['fixtures'])==1 and p['fixtures'][0]['ok'] and p['fixtures'][0]['exit_code']==0
        assert p['fixtures'][0]['result_sha256']==sha(ROOT/'fixtures/official-stem-image-head-items/controls.json')
        assert p['fixtures'][0]['input_sha256']==n['source_image_sha256']
    assert reproduction['ten_prior_CLI_mode_bytes_unchanged']
    return {'status':'passed','actual_upstream_run_storage_and_private_Graph_history_retained':True,
        'upstream_physical_entries':1220,'final_physical_entries':2335,'initialized_cache_systems':[1,2],
        'original_XML_loaded_inspection_replay':True,'actual_original_transient_STEMS_newly_certified':False,
        'automatic_final_stem_glyph_cache_SIG_cleaner_epilog_complete':False,
        'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
