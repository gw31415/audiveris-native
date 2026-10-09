#!/usr/bin/env python3
"""Package recorded observations only; never build, test, recognize, or change Git."""
from pathlib import Path
import argparse, datetime, hashlib, json, shutil
p=argparse.ArgumentParser();p.add_argument('--inputs',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
I=json.loads(a.inputs.read_text());D=a.output.resolve();D.mkdir(parents=True,exist_ok=True)
if D==a.inputs.resolve().parent and a.inputs.name in ['VALIDATION.json','MANIFEST.json']:raise SystemExit('input conflicts with generated names')
def path(k):return Path(I[k])
def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def ref(f):
 f=Path(f);return {'path':str(f),'exists':f.is_file(),**({'bytes':f.stat().st_size,'sha256':sha(f)}if f.is_file()else{})}
def load(f):return json.loads(Path(f).read_text())
def save(name,v):(D/name).write_text(json.dumps(v,indent=2)+'\n')
def copy(k,name):
 f=path(k)
 if f.exists():shutil.copyfile(f,D/name)
 return ref(f)
start=load(path('start'));copy('start','image-target-START.json');copy('scalar','scalar-RESULT.json')
scalar=load(path('scalar'));assert scalar['exit_code']==0 and scalar['unique_targets']==4
scalar_matches=all(start['source_sha256'].get(k)==v for k,v in scalar['source_sha256'].items())
primary={'artifact':ref(path('primary')),'status':'pending','unique_targets':start['unique_targets'],'overall_passed':None}
if path('primary').exists():
 r=load(path('primary'));copy('primary','image-target-RESULT.json')
 primary.update({'status':'finished','exit_code':r['exit_code'],'elapsed_seconds':r['elapsed_seconds'],'source_stable':r['source_stable'],'overall_passed':r['exit_code']==0})
copy('checks','initial-checks.json');copy('clippy_log','initial-clippy.log');checks=load(path('checks'))
clippy=next(r for r in checks if r['check']=='clippy');log=path('clippy_log').read_text()
assert clippy['exit_code']==101 and log.count('error: using `chunks_exact` with a constant chunk size')==3
build=load(path('build'));assert build['exit_code']==0;copy('build','build-RESULT.json')
cli=load(path('cli'));assert cli['automatic_events']==63 and cli['declared_correction']['events']==8
assert all(r['XSD']=='passed'and r['previous_XML_bytes']=='identical'for r in cli['automatic_inputs'])
assert cli['declared_correction']['XSD']=='passed'and cli['declared_correction']['previous_XML_bytes']=='identical'
copy('cli','CLI-REGRESSION.json')
post={'status':'pending','qualification':'Three cfg(test) chunks_exact lints need the prepared test-only fix and a recorded successful follow-up check. Original failure is retained.'}
if I.get('post_fix_checks')and path('post_fix_checks').exists():
 results=load(path('post_fix_checks'));copy('post_fix_checks','post-fix-checks.json')
 post={'status':'finished','all_passed':all(r['exit_code']==0 for r in results),'artifact':ref(path('post_fix_checks'))}
markers={}
for key,name,expected_rejects,expected_protected in [('an_laura','AnLaura',5,15),('der_winter','DerWinter',10,77)]:
 f=path(key);fresh=f.exists()and f.stat().st_mtime>=start['start_time']
 row={'raw':ref(f),'fresh_after_recorded_START':fresh,'status':'pending'}
 if fresh:
  v=load(f);assert len(v['rejected'])==expected_rejects and len(v['protected_factory_arrivals'])==expected_protected
  assert v['graph_unchanged_during_assertion']and not v['factory_lifecycles_admitted_by_this_unit']
  row.update({'status':'observed','rejected':expected_rejects,'protected_factory_arrivals':expected_protected,'rank_limit_controls':len(v['accepted_limit_controls']),'unmatched_CURVES_candidates':len(v['unmatched_CURVES_candidates_preserved']),'full_test_pass_claimed_from_marker_alone':False})
  copy(key,name+'-CHECKED-observed.json')
 markers[name]=row
traces={}
for key,name in [('duration_trace','PAGE-DURATION-TRACE'),('shared_growth','PAGE-SHARED-GROWTH')]:
 f=path(key);fresh=f.exists()and f.stat().st_mtime>=start['start_time']
 row={'raw':ref(f),'fresh_after_recorded_START':fresh,'status':'pending','comparison_complete':False}
 if fresh:
  v=load(f);row.update({'status':'observed','comparison_complete':v.get('comparison_complete',False),'frames':len(v.get('frames',[]))})
  # Frames are already bounded reads of the two selected actual Stacks. Do not copy PAGE_result or any full Graph/report.
  compact={k:x for k,x in v.items()if k!='PAGE_result'};compact['raw']=ref(f)
  save(name+'.json',compact)
 traces[name]=row
source_refs=[ref(x)for x in I['source_references']]
save('SOURCE-REFERENCES.json',{'original_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','files':source_refs,'direct_checked_rejections':15,'ACCENT_exact_original_live_gap_and_failure_string_observed':False,'PAGE_original_scope':'Selected original methods replayed on saved post-PAGE actual owners, not fresh image or whole PAGE. Restored original null priors remain distinct from native already-computed prior clocks.','declared_shared_growth_scope':'Only original last-excess-Voice before/after is directly observed; preceding zero-termination native checks preserving1/64 are source-derived controls.'})
validation={'generated_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Unsaved CHECKED rejection behavior plus cfg(test) per-Voice PAGE observations. No positive factory admission.','scalar':{'passed':4,'exit_code':0,'source_stable':scalar['source_stable'],'relevant_sources_match_current_START':scalar_matches,'binary_sha256':scalar['binary_sha256']},'same_image_targets':primary,'fresh_checked_markers':markers,'PAGE_trace_markers':traces,'initial_fmt':'passed','initial_clippy':{'exit_code':101,'errors':3,'lint':'clippy::chunks_exact_to_as_chunks','file':'src/music_xml/page_measures/trace_tests.rs','lines':[219,219,394],'test_only':True,'hidden_or_relabelled_pass':False},'post_fix_checks':post,'build':{'exit_code':0},'CLI':{'automatic_images':6,'automatic_events':63,'declared_correction_events':8,'XML_bytes':'identical','XSD':'passed'},'current_new_real_XML_validation':'pending unless separately supplied below','additional_results':{},'public_factory_admission_checked':False,'quality_accepted':False,'production_or_images_executed_by_packager':False}
for key,f in I.get('additional_results',{}).items():
 if Path(f).exists():
  value=load(f);save('additional-'+key+'.json',value);validation['additional_results'][key]=ref(f)
validation['checkpoint_checks_complete']=primary.get('overall_passed')is True and post.get('all_passed')is True and all(x['comparison_complete']for x in traces.values())
save('VALIDATION.json',validation)
status='pending'if primary['status']=='pending'else('passed'if primary.get('overall_passed')else'failed')
trace_status='completed at both recorded boundaries'if all(x['comparison_complete']for x in traces.values())else'pending; no completed native per-Voice comparison is claimed yet'
(D/'README.md').write_text(f'''# CHECKED rejection and actual PAGE per-Voice checkpoint draft

Current same-image target status: **{status}**. Four scalar contracts passed, both fresh CHECKED markers are observed, and CLI63+8 byte/XSD regression plus the CLI build passed. The initial clippy run **failed with three chunks_exact_to_as_chunks errors in cfg(test) trace_tests.rs**. Its full log and exit101 remain preserved. Follow-up lint status is {post['status']}; no final validation success is inferred from a prepared fix. VALIDATION.json records the exact current evidence.

The production unit rejects15 original-backed physical clef/marker/repeat evaluations before accepted-rank counting and applies the separately qualified ACCENT StaffGap predicate. Native AnLaura retains15 original factory arrivals and Der Winter77, comparing each complete CHECKED array/order/grade. Four third-ranked Fermatas remain count-limit exclusions with no checker failure. Two unmatched native Fermata/Slur-component trials remain explicit CURVES work. The native ACCENT gap is189.87888124913025 against maximum88; its source exact live gap/failure string is still unobserved, so StaffGap remains source/geometry-qualified inference. Factory allocation, raw Staff/null behavior and callback/manual/relation states are not replaced with a null allowlist. Positive factories and public admission remain guarded.

The PAGE addition is cfg(test) observation of the existing ordinary production loop. It holds actual Stack/Measure/Voice/Slot Arcs and graph-backed Chord→Voice ownership, records each check and immediate peer excess publication, compares completed checked-Voice results while leaving other Voices unchanged, and retains live prior clocks. Native per-Voice evidence is currently {trace_status}. Original source is selected-method replay on a saved post-PAGE Book, not fresh first-pass recognition or whole PAGE; restored source null terminations are reported separately from native post-RHYTHMS values.

The declared shared-growth control changes only held canonical Stack and both actual peer Measure excess values to1/64, retaining the positive Voice's1/8 excess and its existing Chord clock/duration. Ordinary PAGE should immediately publish1/8 to both peers at that Voice check. The last Voice pair is directly compared to original; earlier zero-delta checks retaining1/64 are source-derived controls. The helper restores held typed Voice/Measure/Stack timings, generations, Slots, PAGE metadata and certificates before result validation/export, including panic restoration before resume. It does not reconstruct owners from reports. Proposed behavior alone is not runtime proof; the fresh completion markers determine evidence status.

The prior saved PAGE checkpoint matched all18 measure identities,124 notes/27 Voice groups and truth pickup27/27. That prior result is context, not reclassified as current-run final XML verification while these targets remain pending. Remaining quality gaps are the initial extra p/sound56 and missing6Slurs; truth event102/124 and rests9/9 are the original-quality baseline, not a claim of accepted public recognition.

This bounded package keeps raw observation hashes and compact results; it does not duplicate large Graph reports, raw original lifecycle archives or full truth comparisons. Source fixtures already present in the checkout are referenced. The packaging helper requires explicit inputs/output and only reads saved evidence; it performs no tests, builds, images or Git operations.
''')
(D/'PORT-STATUS-PARAGRAPH.md').write_text(f'''CHECKEDの原版棄却15件とACCENT StaffGapを実評価受付前へ接続し、fresh観測でAnLaura15/DerWinter77のfactory到達評価・順位を保持した。4境界契約、既存63＋補正8のXMLバイト/XSD、buildは成功。実PAGEのVoice単位・peer excess公開を同じ実画像ownerで観測するcfg(test)比較を追加し、3対象の最終状態は{status}、両trace完了は{all(x['comparison_complete']for x in traces.values())}。初回clippyはtest内chunks_exact lint3件で失敗し、修正後checkは{post['status']}として明示する。正のfactory/公開受付・品質受入は未完。原版reload nullとnative既存時計を区別し、次はtyped HeaderClef/MeasureFiller・cross-family overlapの冒頭p・Slur依存。\n''')
(D/'NEXT-DEPENDENCY.md').write_text('''# Next dependencies for image-quality acceptance

1. Finish the currently running same-image targets, require fresh complete ordinary and declared PAGE trace markers, apply the prepared test-only chunks_exact lint fix, and record successful follow-up fmt/clippy. Retain the initial failure. Verify current XML/XSD/exact7/64 and relevant real-page outputs before normal main save/readback; do not rerun images merely to relabel a test-only lint fix.
2. Continue the already observed positive factory lifecycle: real G-clef/Fermata/Octave/Marker objects, raw/null Staff behavior, actual relations and ordered callbacks. CHECKED success is not factory admission, and eventual weak removal is not a nullable-factory result.
3. Connect typed HeaderClef membership and recognition-required MeasureFiller on actual Staff/Part/Measure owners before their positive factories depend on them. Preserve absence/null semantics; no fake Clef, borrowed wrong-system pitch or report-based ownership reconstruction.
4. Implement common cross-family overlap/exclusion and first-purge scheduling with actual partner geometry/identities, and resolve the observed extra initial p/sound56. Its source boundary must be proved rather than deleting by fixture coordinates.
5. Connect retained native Slur components/curves to real Head links, cleanup and export. Six original/truth slurs and twelve endpoints remain absent, and two native Fermata component candidates remain unexplained by original target evaluation evidence. No coordinate-based null exemptions.

Goal remains representative score image→Audiveris-equivalent MusicXML quality, with source equivalence and independent truth accuracy kept separate. Preserve actual owners, exact clocks/expectedDuration, relation/manual identity and transactional rollback. Use bounded relevant targets and existing63+8/exact7/64 regressions; no general editing/API-completeness expansion, historical649-test rerun, new auth/environment, Mac/GHA or duplicated archives.
''')
if I.get('oneoff_runner')and path('oneoff_runner').exists():(D/'archive-oneoff-runner.txt').write_text('Historical executed runner only. Do not execute against recorded output paths.\n\n'+path('oneoff_runner').read_text())
shutil.copyfile(a.inputs,D/'inputs.json') if a.inputs.resolve()!=D/'inputs.json'else None
shutil.copyfile(Path(__file__),D/'package_evidence.py') if Path(__file__).resolve()!=D/'package_evidence.py'else None
manifest={'generated_utc':validation['generated_utc'],'files':{str(f.relative_to(D)):{'bytes':f.stat().st_size,'sha256':sha(f)}for f in sorted(D.iterdir())if f.is_file()and f.name!='MANIFEST.json'}}
size=sum(x['bytes']for x in manifest['files'].values());assert size<=150_000,('bounded package too large',size)
save('MANIFEST.json',manifest)
print(json.dumps({'output':str(D),'bytes_excluding_manifest':size,'same_image_targets':status,'trace_comparisons_complete':all(x['comparison_complete']for x in traces.values()),'post_fix_checks':post['status']}))
