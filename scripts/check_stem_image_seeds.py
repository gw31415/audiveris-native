#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import gzip,hashlib,json,sys
from pathlib import Path
from check_stem_image_head_plan import check as prior_check, compare as target_compare
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-image-seeds'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def compare(o,n):
    assert o['native_outputs_used'] is False and o['unordered_neighbor_and_seed_catalogs_canonicalized'] is True and o['candidate_selection_order_preserved'] is True
    assert o['corners']==n['corner_count']==1408 and len(o['systems'])==len(n['systems'])==2
    registry=n['physical_registry'];glyph=lambda id:registry[id-1]['glyph'];canon=lambda g:json.dumps(g,sort_keys=True,separators=(',',':'));ordered=lambda v:sorted(map(canon,v));key=lambda v:(tuple(v['head_bounds']),v['shape'],v['side'],v['vertical'])
    count=selected=purged=areas=0
    for a,b in zip(o['systems'],n['systems']):
        assert a['system']==b['system']
        assert all(a[k]==b[k] for k in ('max_head_in_dx','max_head_out_dx','max_head_seed_dy','min_head_stump_dy'))
        assert ordered(a['no_stem_areas'])==ordered(b['no_stem_areas'])
        assert ordered(a['surviving_seeds'])==ordered(glyph(id) for id in b['surviving_seed_ids'])
        purged+=len(b['purged_seed_ids']);areas+=len(b['no_stem_areas'])
        e={key(c):c for c in a['cases']};assert len(e)==len(a['cases'])==len(b['cases'])
        for c in b['cases']:
            x=e[key(c['head'])];assert x['reference']==c['head']['reference']
            assert ordered(x['neighbors'])==ordered(glyph(id) for id in c['neighbor_seed_ids'])
            v=[{k:t[k] for k in ('glyph','gap','stands_out','eligible')} for t in c['candidates']];assert x['candidates']==v
            assert x['selected_seed']==c['selected_seed']
            assert c['section_fallback_required']==(c['selected_seed_id'] is None)
            if c['selected_seed_id'] is not None:
                assert glyph(c['selected_seed_id'])==x['actual_stump']==c['selected_seed'];selected+=1
            count+=1
    assert (count,selected,purged,areas)==(1408,577,3,27)
    assert o['selected_seed_corners']==n['selected_seed_corners']==577 and n['section_fallback_corners']==831
    unique={c['selected_seed_id'] for sys in n['systems'] for c in sys['cases'] if c['selected_seed_id'] is not None}
    original_unique={canon(c['selected_seed']) for sys in o['systems'] for c in sys['cases'] if c['selected_seed'] is not None}
    assert len(unique)==len(original_unique)==100
    assert len(o['seed_boundaries'])==9 and len(o['bar_overlap_boundaries'])==3
    assert n['upstream_registry_preserved'] is True and n['image_seed_branch_complete'] is True
    assert all(n[k] is False for k in ('automatic_stump_selection_complete','automatic_raster_pipeline_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
    return {'status':'passed','image_derived_corners':count,'selected_physical_seed_glyph_corners':selected,'distinct_selected_physical_seed_glyphs':100,'section_fallback_corners':831,'native_SIG_derived_no_stem_areas':areas,'no_stem_seed_exclusions':purged,'condition_specified_original_boundary_controls':12,'neighbor_sets_exact':True,'candidate_order_gap_eligibility_and_physical_selection_exact':True,'automatic_stump_pipeline_complete':False,'page_cache_SIG_complete':False,'epilog_complete':False,'heads_complete':False,'stems_complete':False}
def check(path):
    prior=prior_check(path);o=read(F/'controls.json');proof=read(F/'source-backed-reproduction.json');assert proof['status']=='passed' and proof['independent_original_runs']==2 and proof['native_outputs_used'] is False and proof['original_sha256']==sha(F/'controls.json')
    for file in ('controls-provenance.json','controls-certified-provenance.json'):
        p=read(F/file);assert p['probe']=='StemImageSeedsProbe' and p['official_cli_step']=='HEADS' and p['release']=='5.11.0' and p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
        assert len(p['fixtures'])==1 and p['fixtures'][0]['ok'] and p['fixtures'][0]['exit_code']==0 and p['fixtures'][0]['result_sha256']==proof['original_sha256'] and p['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    n=read(F/'native-stems-report.json.gz');r=read(F/'native-reproduction.json');assert r['status']=='passed' and r['actual_image_CLI_executed'] is True and all(sha(ROOT/k)==v for k,v in r['native_source_sha256'].items())
    assert sha(F/'native-stems-report.json.gz')==r['native_stems_report_gzip_sha256'] and hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_stems_report_raw_sha256']
    assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png') and n['native_HEADS_report_sha256']==r['entire_old_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
    page=read(path);assert n['physical_registry']==page['head_registration']['live_glyph_retrieval']['registry']
    target_compare(read(ROOT/'fixtures/official-stem-image-head-plan/controls.json'),n['head_plan'])
    result=compare(o,n);result['prior_image_entry_report_unchanged']=True;result['original_loaded_context']=True;result['original_transient_context_newly_verified']=False;result['nonzero_skew_scans_special_staff_full24_pages_verified']=False;result['historical_prior_checkpoint_contracts']=prior
    return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
