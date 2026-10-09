#!/usr/bin/env python3
"""Read-only evidence planning. Write projections/metadata only beside this script in /tmp."""
from pathlib import Path
from collections import Counter
import hashlib,json,copy
HERE=Path(__file__).resolve().parent
assert str(HERE).startswith('/tmp/')
ROOT=Path('/workspace/audiveris-native')
PREVIOUS=Path('/tmp/slur-checkpoint-evidence-plan-v1')
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def meta(path):return {'path':str(path),'sha256':digest(path),'bytes':path.stat().st_size}
def write(name,value):
 p=HERE/name;p.write_text(json.dumps(value,indent=2)+'\n');return p
old=json.loads((PREVIOUS/'copy-plan.json').read_text());entries=copy.deepcopy(old['entries'])
for e in entries:
 assert digest(Path(e['path']))==e['sha256'],('historical plan input changed',e['path'])
 if e['destination_relative']=='prepare.py':e['destination_relative']='provenance/prepare-plan-v1.py'
references=[]
def add(path,destination,purpose,action='copy'):
 path=Path(path);entry=dict(meta(path),destination_relative=destination,action=action,purpose=purpose)
 assert all(e['destination_relative']!=destination for e in entries),destination
 entries.append(entry)
 return entry
# Adopted fixture and test sources already exist in checkout; verify instead of duplicate.
for origin,dest in [
 ('/tmp/slur-kernel-test-adoption-fit-v1/source-observations.json','fixtures/official-short-slur-fit/source-observations.json'),
 ('/tmp/slur-kernel-test-adoption-fit-v1/SOURCE.json','fixtures/official-short-slur-fit/SOURCE.json'),
 ('/tmp/slur-kernel-test-adoption-fit-v1/fit_source_observations.rs','src/music_slurs/fit_source_observations.rs'),
 ('/tmp/slur-gap-obstacle-adapter-v1/source-fixture.json','fixtures/official-slur-gap-obstacles/observed.json')]:
 assert digest(Path(origin))==digest(ROOT/dest),dest
 add(origin,dest,'Already adopted exact fixture/test source; verify existing checkout bytes only.','verify_existing')
for origin,dest,purpose in [
 ('slur-kernel-test-adoption-fit-v1/README.md','source/adopted-fit/README.md','Historical fixture proposal scope; later adopted execution is separately recorded.'),
 ('slur-kernel-test-adoption-fit-v1/HASHES.json','source/adopted-fit/HASHES.json','Frozen source fixture/projection/test manifest; referenced adopted files are not duplicated.'),
 ('slur-kernel-test-adoption-fit-v1/project_existing_observation.py','source/adopted-fit/project_existing_observation.py','Reproducible bounded fixture projection, never a source/native execution.'),
 ('slur-adopted-fit-v1/PROVENANCE.json','native/adopted-fit/PROVENANCE.json','Actual adopted157row test binary/source hashes and qualified pass.'),
 ('slur-adopted-fit-v1/RESULT.json','native/adopted-fit/RESULT.json','Actual targeted fit invocation and successful exit; not image lifecycle.'),
 ('slur-checkpoint-evidence-plan-v1/retained-full-artifacts.json','provenance/retained-full-artifacts-v1.json','Exact larger historical original references; no large artifact copied.'),
 ('slur-checkpoint-evidence-plan-v1/README.md','provenance/plan-v1-README.md','Preserve historical kernel count and mutable rlib qualification.'),
 ('slur-lifecycle-adoption-notes-v2.json','ADOPTION-NOTES-v2.json','Current root adoption and failures; explicitly incomplete and qualityfalse.')]:add(Path('/tmp')/origin,dest,purpose)
# Source review of the cache representation adapter; actual code/test are already adopted.
for name in ['README.md','PROVENANCE.json','MANIFEST.json','READ-BASES.json']:
 add(Path('/tmp/slur-live-chord-getter-review-v1')/name,'source/live-chord-getter/'+name,'Pinned source/getter review and exact historical hashes; no duplicate test patch or native implementation.')
for name in ['test.patch','test-addition.rs']:
 references.append(meta(Path('/tmp/slur-live-chord-getter-review-v1')/name))
retry=Path('/tmp/slur-lifecycle-v3')
for name in ['CHECKS.json','FAST-RESULT.json']:
 if (retry/name).exists():
  add(retry/name,'native/lifecycle-v3-pending/'+name,'Actual fresh check/scalar record only; v3 fullimage retry remains pending and separate.')
# Original detached filter and4 actual original threshold observations.
base=Path('/tmp/slur-gap-obstacle-original-v1')
for name in ['README.md','PROVENANCE.json','MANIFEST.json','SOURCE-HASHES.json','observed.json','inputs.tsv','SlurGapObstacleProbe.java','run.py','native-retained-diagnostic.json']:
 add(base/name,'source/gap-obstacles/'+name,'Detached original filter15cases+4Head threshold controls; no original Skeleton/cleanup lifecycle proof.')
add('/tmp/slur-gap-obstacle-adapter-v1/README.md','source/gap-obstacles/ADAPTER-README.md','Adapter interpretation and scope; implementation/test code is already in checkout.')
# Preserve original raw failed RESULTS locally, publish bounded projections without their repeated whole-repo maps.
for version in [1,2]:
 base=Path(f'/tmp/slur-lifecycle-v{version}')
 result=json.loads((base/'RESULT.json').read_text());assert result['exit_code']==101
 compact={k:v for k,v in result.items()if k!='source_sha256'}
 source_map=result['source_sha256'];compact['raw_RESULT']=meta(base/'RESULT.json')
 compact['source_sha256_total_files']=len(source_map)
 compact['source_sha256_map_canonical_sha256']=hashlib.sha256(json.dumps(source_map,sort_keys=True,separators=(',',':')).encode()).hexdigest()
 compact['selected_Slur_source_sha256']={k:v for k,v in source_map.items()if k.startswith(('src/music_slurs','src/shared_sig/slur'))}
 compact['projection_qualification']='All run result fields retained; repeated whole-checkout source map replaced by count/hash and selectedSlur subset. Full raw RESULT remains untouched locally.'
 compact['success_claimed']=False
 fp=write(f'lifecycle-v{version}-RESULT-compact.json',compact)
 add(fp,f'native/lifecycle-v{version}/RESULT-compact.json','Failed actual image+rational batch; exit101 and source_stable preserved, full raw hash referenced.')
 references.extend([meta(base/'RESULT.json'),meta(base/'START.json')])
 for name in ['PARTIAL-VALIDATION.json','CHECKS.json','FAST-RESULT.json']:
  add(base/name,f'native/lifecycle-v{version}/{name}','Exact historical partial result: focused passes do not imply DerWinter lifecycle success.')
 for name in ['RESULTS.json']:
  add(Path(f'/tmp/slur-lifecycle-v{version}-cli-regression')/name,f'native/lifecycle-v{version}/CLI-{name}','Existing63+8event XML byte/XSD pass for this recorded version, not Slur success.')
# Fresh actual native gap diary is small enough to retain once, with its full owner/glyph observations.
diary=Path('/tmp/slur-lifecycle-v2/gap-decisions.json');data=json.loads(diary.read_text())
add(diary,'native/lifecycle-v2/gap-decisions.json','Exact19actual native prefilter decisions from failed v2; owning addresses local to this run, no successfulSlur lifecycle implied.')
summary={'input':meta(diary),'rows':len(data['rows']),'allowed':dict(Counter(str(r['allowed'])for r in data['rows'])),'errors':[r['error']for r in data['rows']if r['error']is not None],'obstacle_decisions':dict(Counter(o['decision']for r in data['rows']for o in r['obstacles'])),'qualification':'Native observations only.4candidate obstacles across19rows; acceptance of a gap does not establish selectedSlur/Head/callback/cleanup ownership.'}
add(write('gap-diary-summary.json',summary),'native/lifecycle-v2/gap-diary-summary.json','Small index of exact retained v2 diary; not a replay or a recognition result.')
# Freeze source manifest and adopted checkout verification without rereading mutable build output.
verified=[]
for e in entries:
 if e['action']=='verify_existing':
  target=ROOT/e['destination_relative'];assert digest(target)==e['sha256']
  verified.append({'path':e['destination_relative'],'sha256':digest(target),'matches_recorded_source':True,'copied':False})
for rel in ['src/music_slurs/fit.rs','src/shared_sig/slur_obstacles.rs','src/music_slurs/extension_boxes.rs','src/music_slurs/image_tests.rs','src/shared_sig/chord_cache.rs','src/shared_sig/slur_owners.rs','src/music_slurs/boundary_tests.rs']:
 verified.append({'path':rel,'sha256':digest(ROOT/rel),'current_checkout_reference_only':True,'copied':False})
add(write('adopted-checkout-verification.json',verified),'provenance/adopted-checkout-verification.json','Existingfixture/testcode verified, current production/diary source hashes referenced without duplication.')
add(write('retained-run-records.json',references),'provenance/retained-run-records.json','Raw failed result/start records retained locally by exact hash; no repeated whole-repo maps.')
old_manifest=PREVIOUS/'original-manifest-verification.json'
qualification={'scope':'Historical original-manifest verification remains unchanged; no replacement of original hashes with rebuilt outputs.','record':meta(old_manifest),'mutable_rlib_mismatches':json.loads(old_manifest.read_text())[1:3],'policy':'Old mutable target/debug/deps .rlib hashes are run provenance. Their later mismatch does not establish corruption of original source observations, nor availability of the old executable. No current build product is copied or re-labelled as historical.'}
add(write('mutable-build-qualification.json',qualification),'provenance/mutable-build-qualification.json','Explicitly retain bothhistorical rlib mismatch records; no new kernel validation claim.')
# The package itself does not decide when parent can save a completed checkpoint.
status={'schema':'audiveris-native.slur-lifecycle-checkpoint-preparation.v2','source_pin':old['source_pin'],'quality_accepted':False,'Slur_lifecycle_native_source_comparison_completed':False,'writer_applied':False,'new_Slur_XML_success_claimed':False,'successful_individual_evidence':['10v1focusedtargets','157adoptedfit observations','1v2obstacle target covering15originalfilter rows+4thresholds','AnLaura andexact7/64byte/XSD in each failed imagebatch','63+8CLIbyte/XSD eachrecordedversion'],'failed_runs':{'v1':'DerWinter slur_non_crossable_intersection_comparison_pending','v2':'DerWinter retained_chord_cache_owner_required'},'pending':['v3live-versus-retainedChord adapter scope/source review and any copied scalar record are separate from image success','v3retry result and same-target lifecycle/callback/Part/cleanup/rollback comparison','Export remains intentionally withheld until separatewriter slice','Publicfactory admission and representative image quality acceptance'],'source_scope_qualification':'Original callback proof resumedCURVES from unchanged savedCHORDS Book; upstream owners unmarshalled, clocksNULL, no fresh fullimage and no new originalMusicXML export.','packaging_operations':'Only read existing artifacts and wrote /tmp projections/plan; no checkoutcopy/build/image/Git.'}
add(write('STATUS-PENDING.json',status),'provenance/PREPARATION-STATUS.json','Checkpoint prep is not a success declaration; parent must add freshv3 outcome before finalizing evidence.')
for name,dest in [('prepare.py','provenance/prepare-plan-v2.py'),('README.md','provenance/PREPARATION.md')]:
 if (HERE/name).exists():add(HERE/name,dest,'Plan reproduction/instructions; no automatic source execution or checkout writes.')
plan={'source_pin':old['source_pin'],'destination_root':'evidence/real-slur-lifecycle','destination_policy':'copy destination_relative is beneath destination_root; verify_existing destination_relative is relative to repository root. This script copies nothing into checkout.','previous_plan':meta(PREVIOUS/'copy-plan.json'),'copy_count':sum(e['action']=='copy'for e in entries),'copy_bytes':sum(e['bytes']for e in entries if e['action']=='copy'),'verify_existing_count':sum(e['action']=='verify_existing'for e in entries),'entries':entries,'status':'incomplete; v3retry pending','raw_logs_Books_JARs_binaries_included':False}
write('copy-plan.json',plan)
print(json.dumps({k:plan[k]for k in ['copy_count','copy_bytes','verify_existing_count','status']}))
