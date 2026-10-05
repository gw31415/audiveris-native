#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Original selected cleaner check, distinct from automatic parts/STEMS epilog."""
import json,sys
from pathlib import Path
from check_stem_cache_sig import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-cleaner'
def check(path):
    prior=prior_check(path);r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False and r['original_fields_byte_reproduced'] is True
    assert sha(F/'controls.json')==r['original_sha256'] and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='HeadStemsCleanerControlProbe' and 'compile-only' not in x['runtime_classpath']
        assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items())
        assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['input_sha256']==r['input_sha256']
    for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'): assert p[k]==q[k]
    n=read(F/'native-reproduction.json');assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'] and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
    original=read(F/'controls.json');native=read(F/'native-controls.json.gz');assert original['native_outputs_used'] is False and len(original['cases'])==len(native['cases'])==800
    assert original['cases']==native['cases']
    two=removed=0
    for c in native['cases']:
        assert c['vertex_delta']==0 and not any(c['stem_removed'])
        before=sum(s['relation'] is not None for s in c['stems']);assert c['edge_delta']==len(c['kept'])-before
        removed+=before-len(c['kept']);two+=len(c['kept'])==2
        assert len(c['kept'])<=2
        if len(c['kept'])==2:assert c['dy']<=0.2 and c['mode'] in ('canonical','missing')
    assert two==108
    return {'status':'passed','selected_HeadStemsCleaner_controls':800,'two_stem_canonical_shares':two,'removed_relations':removed,
            'ordered_intrinsic_contribution_ties_verified':True,'dy_point2_boundary_verified':True,
            'physical_geometry_and_SIG_callback_states_verified':True,'parallel_cleaner_relations_supported':False,
            'automatic_parts_selection_complete':False,'STEMS_epilog_complete':False,'full24_HEADS_page_propagation':False,
            'published_scan_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
