#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import gzip, hashlib, json, sys
from pathlib import Path
from check_stem_image_stumps import check as prior_check, compare as stump_compare
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-image-sources'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p): return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def key(c): return (tuple(c['head_bounds']),c['shape'],c['side'],c['vertical'])
def compare(o,n):
    assert o['native_outputs_used'] is False and o['original_phase']=='inspectStems_before_linkStems_and_finalizeStems'
    assert o['corners']==n['corner_count']==1408
    assert o['selected_seed_occurrences']==n['selected_builder_seed_occurrences']==248
    assert o['aligned_seed_occurrences']==n['aligned_builder_seed_occurrences']==246
    assert len(o['systems'])==len(n['systems'])==2
    reg=n['stump_plan']['physical_registry']
    count=removed=raw=aligned=0
    for a,b in zip(o['systems'],n['systems']):
        assert all(a[f]==b[f] for f in ('system','min_seed_contribution','max_line_seed_dx','max_alignment_dx','max_alignment_dy'))
        assert (a['min_seed_contribution'],a['max_line_seed_dx'],a['max_alignment_dx'],a['max_alignment_dy'])==(6,1.65,1.65,44.)
        expected={key(c):c for c in a['cases']}
        assert len(expected)==len(a['cases'])==len(b['cases'])
        for c in b['cases']:
            e=expected[key(c['head'])]
            assert all(e[f]==c[f] for f in ('theoretical_line','y_range','stump','selected_seeds','aligned_seeds'))
            assert sorted(x['coordinates'] for x in e['area_path'] if x['type'] in (0,1))==sorted(c['polygon'])
            assert all(x['type'] in (0,1,4) for x in e['area_path'])
            for ids,glyphs in [('selected_seed_ids','selected_seeds'),('aligned_seed_ids','aligned_seeds')]:
                assert len(c[ids])==len(c[glyphs])
                assert [reg[id-1]['glyph'] for id in c[ids]]==c[glyphs]
            assert set(c['aligned_seed_ids'])<=set(c['selected_seed_ids'])
            for id in c['alignment_removed_ids']: assert 1<=id<=len(reg)
            removed+=len(c['alignment_removed_ids'])
            raw+=len(c['selected_seed_ids']);aligned+=len(c['aligned_seed_ids']);count+=1
    assert (count,raw,aligned,removed)==(1408,248,246,2)
    assert len(o['seed_source_controls'])==12
    assert n['image_builder_seed_selection_complete'] is True and n['image_seed_alignment_complete'] is True
    assert all(n[k] is False for k in ('other_head_linker_selection_complete','beam_sibling_linker_selection_complete','final_stem_glyph_selection_complete','automatic_raster_pipeline_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
    return {'status':'passed','actual_image_corner_seed_inputs_exact':count,'initial_selected_seed_occurrences':raw,'post_alignment_seed_occurrences':aligned,'alignment_removed_occurrences':removed,'ordered_full_physical_seed_lists_exact':True,'builder_reference_to_target_line_and_corner_contribution_range_exact':True,'original_seed_source_condition_controls':12,'other_head_beam_linkers_final_glyph_cache_SIG_epilog_complete':False,'heads_complete':False,'stems_complete':False}
def check(path):
    prior=prior_check(path)
    o=read(F/'controls.json');proof=read(F/'source-backed-reproduction.json')
    assert proof['status']=='passed' and proof['native_outputs_used'] is False and proof['independent_original_runs']==2 and proof['original_sha256']==sha(F/'controls.json')
    ps=[read(F/f) for f in ('controls-provenance.json','controls-certified-provenance.json')]
    for p in ps:
        assert p['probe']=='StemImageSourcesProbe' and p['official_cli_step']=='HEADS' and p['release']=='5.11.0'
        assert p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
        assert len(p['fixtures'])==1
        f=p['fixtures'][0];assert f['ok'] and f['exit_code']==0 and f['result_sha256']==sha(F/'controls.json') and f['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'): assert ps[0][k]==ps[1][k]
    r=read(F/'native-reproduction.json');n=read(F/'native-stems-report.json.gz')
    assert r['status']=='passed' and r['actual_image_CLI_executed'] is True and all(sha(ROOT/k)==v for k,v in r['native_source_sha256'].items())
    assert sha(F/'native-stems-report.json.gz')==r['native_stems_report_gzip_sha256']
    assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_stems_report_raw_sha256']
    assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    assert n['native_HEADS_report_sha256']==r['entire_old_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
    page=read(path);stump_compare(read(ROOT/'fixtures/official-stem-image-stumps/controls.json'),n['stump_plan'],page)
    result=compare(o,n)
    result.update(original_XML_loaded_HEADS_context=True,original_phase_is_inspection_before_linking_and_epilog=True,original_fresh_transient_lifecycle_newly_verified=False,nonzero_skew_scans_special_staff_full24_pages_verified=False,old_HEADS_and_three_prior_CLI_modes_byte_identical=True,historical_prior_checkpoint_contracts=prior)
    return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
