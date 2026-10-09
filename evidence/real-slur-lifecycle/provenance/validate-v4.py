from pathlib import Path
import hashlib,json,shutil
root=Path('/workspace/audiveris-native');run_dir=Path('/tmp/slur-lifecycle-v4');e=root/'evidence/real-slur-lifecycle'
def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
run=read(run_dir/'RESULT.json');assert run['exit_code']==0 and run['source_stable']
assert '1 passed; 0 failed' in (run_dir/'console.log').read_text()
assert all(sha(root/n)==h for n,h in run['source_sha256'].items())
assert read(run_dir/'FAST-RESULT.json')['exit_code']==0
assert all(x['exit_code']==0 for x in read(run_dir/'CHECKS.json'))
assert read('/tmp/slur-lifecycle-v1/PARTIAL-VALIDATION.json')['fast_passed']==10
assert all(x['exit_code']==0 for x in read('/tmp/slur-adopted-fit-v1/RESULT.json'))
assert read('/tmp/slur-lifecycle-v3/FAST-RESULT.json')['exit_code']==0
prior=read('/tmp/slur-lifecycle-v2/PARTIAL-VALIDATION.json')
assert prior['image_rational_passed']==2 and prior['image_rational_failed']==1
assert all(x['bytes_identical'] and x['XSD']=='passed' for x in prior['XML'].values())
for n in ['real-slur-native-lifecycle-v2.json','real-slur-native-export-withheld-v1.json','real-slur-gap-native-decisions.json']:
 assert (Path('/tmp')/n).stat().st_mtime>=run['start_time'],n
lifecycle=read('/tmp/real-slur-native-lifecycle-v2.json');withheld=read('/tmp/real-slur-native-export-withheld-v1.json')
checked_path=Path('/tmp/real-der-winter-native-checked-rejections.json');assert checked_path.stat().st_mtime>=run['start_time']
checked=read(checked_path);removed=checked['unmatched_CURVES_candidates_removed_by_actual_Slur_cleanup']
assert len(checked['rejected'])==10 and len(checked['protected_factory_arrivals'])==77 and len(checked['accepted_limit_controls'])==4
assert [(x['historical_native_weight'],x['erased_old_pixels']) for x in removed]==[(77,71),(81,70)]
assert all(x['old_candidate_absent'] and x['accepted_Slur_ink_absent_from_post_cleanup_Symbol_glyphs'] for x in removed)
assert len(lifecycle['slurs'])==6 and all(s['runtime_contracts_match'] for s in lifecycle['slurs'])
assert lifecycle['late_publication_refusal_verified'] and lifecycle['same_prepared_stage_published_after_refusal']
for k in ['previous_XML_preserved','same_actual_Slur_and_relation_owners_after_refusal','ordinary_LINKS_rhythm_PAGE_succeeded','export_preserved_graph_and_clocks','writer_still_required']:assert withheld[k]
assert not withheld['new_XML_written'] and not withheld['quality_accepted'] and withheld['retained_slurs']==6
cli=read('/tmp/slur-lifecycle-v4-cli-regression/RESULTS.json');assert cli['automatic_events']==63 and cli['declared_correction']['events']==8
assert all(x['previous_XML_bytes']=='identical' and x['XSD']=='passed' for x in [*cli['automatic_inputs'],cli['declared_correction']])
assert read('/tmp/slur-lifecycle-v4-build.json')['exit_code']==0
observations={'lifecycle':lifecycle,'withheld':withheld}
copies={
 'native/lifecycle-v4/RESULT.json':run_dir/'RESULT.json',
 'native/lifecycle-v4/CHECKS.json':run_dir/'CHECKS.json',
 'native/lifecycle-v4/FAST-RESULT.json':run_dir/'FAST-RESULT.json',
 'native/lifecycle-v4/BUILD.json':Path('/tmp/slur-lifecycle-v4-build.json'),
 'native/lifecycle-v4/CLI-RESULTS.json':Path('/tmp/slur-lifecycle-v4-cli-regression/RESULTS.json'),
 'native/lifecycle-v4/lifecycle.json':Path('/tmp/real-slur-native-lifecycle-v2.json'),
 'native/lifecycle-v4/export-withheld.json':Path('/tmp/real-slur-native-export-withheld-v1.json'),
 'native/lifecycle-v4/gap-decisions.json':Path('/tmp/real-slur-gap-native-decisions.json'),
 'native/lifecycle-v4/checked-cleanup.json':checked_path,
 'provenance/run-lifecycle-v4.py':Path('/tmp/run-slur-lifecycle-v4.py'),
 'provenance/build-regression-v4.py':Path('/tmp/slur-lifecycle-build-regression-v4.py')}
for n,p in copies.items():
 dst=e/n;assert not dst.exists(),str(dst);dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst)
v={
 'unique_targets_passed':17,'fast_unique_targets':14,
 'target_provenance':{'v1':{'fast_passed':10},'adopted_fit':{'passed':1,'original_cases':157},'v2':{'fast_passed':1,'original_filter_cases':15,'actual_Head_threshold_controls':4,'AnLaura_and_rational_passed':2},'v3':{'new_getter_contract_passed':1,'DerWinter_failed_after_six_Slur_lifecycle':True},'v4':{'batch_Stem_owner_contract_passed':1,'previously_failing_DerWinter_passed':1}},
 'current_source_stable':True,'current_binary_sha256':run['binary_sha256'],
 'Slur_lifecycle':{'actual_slurs':6,'same_selected_relation_Arcs':12,'per_Slur_callbacks_match_source':True,'actual_System_Part_Staff_Head_Chord_Measure_paths_validated':True,'Part_collection_before_cached_Part':True,'cleanup_only_actual_accepted_glyph_pixels':True,'same_prepared_stage_after_late_refusal':True,'raw_eligible':lifecycle['raw_eligible'],'clumps':lifecycle['clumps']},
 'original_scope':'Fresh CURVES resumed from unchanged persisted CHORDS Book; upstream owners reloaded, callback clocks null. Native extraction uses retained components/column centers/raw gap support, not original Skeleton/trunk provenance.',
 'clock_scope':'Image callback clocks remain null. Declared actual Voice/Slot/Chord 7/64 and expected3/4 cache controls plus exact7/64 XML are separate evidence; no active-clock original image callback claim.',
 'DerWinter_export':withheld,
 'CHECKED_after_actual_Slur_cleanup':{'original_rejected':10,'original_factory_arrivals':77,'accepted_limit_controls':4,'old_unmatched_candidates_removed':removed},
 'retained_full_frontier':{'path':'/tmp/real-rest-native-internal-rhythm-export.json','sha256':sha('/tmp/real-rest-native-internal-rhythm-export.json'),'bytes':Path('/tmp/real-rest-native-internal-rhythm-export.json').stat().st_size,'copied':False},
 'retained_XML_regressions':{'AnLaura_and_exact7_64':prior['XML'],'provenance':'Successful v2 executions retained; v3 adds the Slur geometry adapter; v4 batches unchanged full Stem cache certification and updates the obsolete pre-Slur-cleanup assertion. Neither successful image nor rational target was repeated.'},
 'CLI_63_plus_8':{'bytes_identical':True,'XSD':'passed','production_binary_sha256':read('/tmp/slur-lifecycle-v4-build.json')['binary_sha256']},
 'initial_failures':[{'run':'preflight','failure':'Ten clippy issues before image execution; corrected without changing JAMA operation order.'},{'run':'v1','failure':'Conservative noncrossable guard rejected DerWinter; replaced with original Area/extension-box/line filtering and contextual threshold0.4.'},{'run':'v2','failure':'retained_chord_cache_owner_required; existing recognition geometry now read directly without allocating or warming an editing cache.'},{'run':'v3','failure':'Six Slur lifecycle comparisons passed; old CHECKED assertion still required two unmatched candidates removed by actual Slur cleanup.'}],
 'public_representative_CLI_rerun':False,'quality_accepted':False,
 'next':'Recognized MusicXML Slur writer; then ordinary Flag/factory/common SymbolsLinker reduction and representative public acceptance.'}
(e/'VALIDATION.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps({'unique_targets_passed':17,'slurs':6,'relations':12,'new_DerWinter_XML':False,'source_stable':True,'quality_accepted':False}))
