import json,hashlib,pathlib
base=pathlib.Path('/tmp/native-slur-binary-support-v1'); ref=pathlib.Path('/tmp/native-slur-raw-gap-refinement-v2')
load=lambda p:json.loads(p.read_text())
rust=load(ref/'rust-observed.json'); rr={r['candidate_index']:r for r in rust}; java=load(base/'observed.json'); refined=load(ref/'observed.json'); py=load(ref/'refined-candidates.json')
fail=[]; checks=0

def eq(a,b,ctx):
 global checks
 checks+=1
 if a!=b:fail.append({'context':ctx,'java_or_recipe':a,'rust':b})

def supports(j,r,ctx):
 eq(j['curve_point_bounds'],r['curve_point_bounds'],ctx+':bounds')
 eq(j['source_containing_runs'],r['source_containing_runs'],ctx+':point_run_rows')
 eq(round(.75*j['curve_point_bounds'][2]),r['required_run_count'],ctx+':required_run_count')
 expected_count=len({(v['point'][0],v['start'],v['stop']) for v in j['source_containing_runs'] if v['accepted_run']})
 eq(expected_count,r['accepted_run_count'],ctx+':accepted_run_count')
 eq(j.get('retrieve_glyph_return')=='Glyph',r['accepted'],ctx+':return_presence')
 if j.get('retrieved_glyph'):
  for jk,rk in [('bounds','glyph_bounds'),('weight','weight'),('total_run_count','accepted_run_count')]:eq(j['retrieved_glyph'][jk],r[rk],ctx+':glyph_'+jk)
base_supports=0;gaps=0
for j in java['rows']:
 i=j['candidate_index'];r=rr[i];eq(j['math_accepted'],r['math_accepted'],f'{i}:baseline_math')
 if j['math_accepted']:
  base_supports+=1;supports(j,r['baseline'],f'{i}:baseline')
 if 'source_raw_BINARY_gap'in j:
  gaps+=1;s=j['source_raw_BINARY_gap'];g=r['refined']['gap']
  for k in ['adjacent','vector','largest_gap']:eq(s[k],g[k],f'{i}:gap_{k}')
  eq(s['gap_class']=='Horizontal',g['horizontal'],f'{i}:gap_axis')
  eq(s['largest_gap']<=java['max_gap_length'],g['accepted'],f'{i}:gap_accept')
  eq(java['max_gap_length'],g['maximum_gap'],f'{i}:gap_threshold')
for j in load(base/'gap-area-observed.json'):
 eq(j['black'],rr[j['candidate_index']]['refined']['gap']['raw_black_points'],f"{j['candidate_index']}:actual_Java_Area_raw_black_pixels")
for j in py['rows']:
 i=j['candidate_index'];r=rr[i]['refined']
 for k in ['points','added_actual_foreground_points']:eq(j[k],r[k],f'{i}:recipe_{k}')
 eq(j['raw_gap_foreground_pixels'],r['gap']['raw_black_points'],f'{i}:recipe_raw_gap_pixels')
ref_supports=0
for j in refined['rows']:
 i=j['candidate_index'];r=rr[i]['refined'];eq(j['math_accepted'],r['math_accepted'],f'{i}:refined_math')
 if j['math_accepted']:ref_supports+=1;supports(j,r['support'],f'{i}:refined')
paths=['/tmp/short_slur_raw_support.rs','/tmp/check_slur_raw_support_actual.rs',str(base/'observed.json'),str(base/'gap-area-observed.json'),str(ref/'observed.json'),str(ref/'refined-candidates.json'),str(ref/'rust-observed.json')]
result={'scope':'Same saved native candidate points and unchanged raw native BINARY. Expected gap/run observations are actual original Java; refinement point recipe is generic Python v2, explicitly not original Skeleton. No recognition rerun, obstacle owner test, Head link, graph insertion or cleaner lifecycle.', 'candidate_sequences':len(rust),'baseline_source_math_accepts':base_supports,'actual_Java_gap_cases':gaps,'actual_Java_Area_black_sets':19,'refined_math_cases':len(refined['rows']),'refined_source_math_accepts':ref_supports,'checks':checks,'failures':fail,'split_pair_support':[{ 'candidate_index':i,'baseline_accepted':rr[i]['baseline']['accepted'],'baseline_runs':rr[i]['baseline']['accepted_run_count'],'required_runs':rr[i]['baseline']['required_run_count'],'added_real_points':len(rr[i]['refined']['added_actual_foreground_points']),'refined_runs':rr[i]['refined']['support']['accepted_run_count'],'refined_accepted':rr[i]['refined']['support']['accepted'],'raw_gap':rr[i]['refined']['gap']['largest_gap']} for i in [136,137,141]],'hashes':{p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()for p in paths}}
(ref/'rust-comparison.json').write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps({k:v for k,v in result.items() if k!='hashes'},indent=2));assert not fail
