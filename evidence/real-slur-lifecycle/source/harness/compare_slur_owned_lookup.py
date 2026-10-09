import json,pathlib,math,hashlib
p=pathlib.Path('/tmp/slur-owned-lookup-original-v2');j=json.loads((p/'observed.json').read_text());r=json.loads((p/'rust-observed.json').read_text());fail=[];checks=0;maxerr=0.;scalar_deltas=[]
def eq(a,b,path):
 global checks,maxerr
 checks+=1
 if isinstance(a,(float,int))and not isinstance(a,bool)and isinstance(b,(float,int))and not isinstance(b,bool):
  err=abs(a-b);maxerr=max(maxerr,err)
  if err:scalar_deltas.append({'path':path,'java':a,'rust':b,'absolute_error':err})
  if err>1e-10:fail.append({'path':path,'java':a,'rust':b})
 elif isinstance(a,list)and isinstance(b,list):
  if len(a)!=len(b):fail.append({'path':path,'java_length':len(a),'rust_length':len(b)})
  for i,(x,y)in enumerate(zip(a,b)):eq(x,y,path+f'[{i}]')
 elif a!=b:fail.append({'path':path,'java':a,'rust':b})
for a,b in zip(j['rows'],r):
 label=a['case'];eq(a['case'],b['case'],label)
 pair = a.get('selected_pair')
 actual = b.get('selected_pair')
 eq(pair is None,actual is None,label+'.selected_pair_presence')
 if pair is not None and actual is not None:
  for side in range(2):
   x=pair[side]['link'];y=actual[side]
   eq(x is None,y is None,label+f'.pair{side}.presence')
   if x is not None and y is not None:
    for k in ['chord','head','euclidean']:eq(x[k],y[k],label+f'.pair{side}.{k}')
 for key in ['horizontal','bisector_unit','targets']:eq(a[key],b[key],label+'.'+key)
 for side in range(2):
  aa=a['areas'][side];bb=b['areas'][side];eq(aa['bounds'],bb['bounds'],label+f'.area{side}.bounds')
  for i,(q,w)in enumerate(zip(aa['queries'],bb['queries'])):
   for k in ['point','contains','unit_rect_intersects']:eq(q[k],w[k],label+f'.area{side}.query{i}.{k}')
  eq(len(aa['queries']),len(bb['queries']),label+f'.area{side}.query_count')
  x=a['lookups'][side];y=b['lookups'][side];eq(len(x),len(y),label+f'.lookup{side}.count')
  for q,w in zip(x,y):
   for k in ['chord','head','euclidean','head_distance_squared']:eq(q[k],w[k],label+f'.lookup{side}.{k}')
refs=json.loads(pathlib.Path('/tmp/der-winter-slur-owned-persisted-v1/observed.json').read_text())['slurs'];refs={int(s['slur_id']):s for s in refs};same=[]
for row in j['rows']:
 if row['kind'].startswith('saved_real'):
  ref=refs[row['slur_id']];expected=[int(e['head_ref'].split('inter')[1])for e in ref['endpoints']];actual=[a['link']['head']for a in row['selected_pair']];same.append({'slur':row['slur_id'],'source_live_lookup_heads':actual,'persisted_heads':expected,'match':actual==expected})
result={'scope':'Original Java expected observations on saved rounded curves/real source owners and separate detached source control owners. Rust has borrowed test-fixture views; this is geometry/selection-kernel comparison, not native recognition or same-runtime object proof. Rust pair selection is exercised for nonmirror cases, empty selection, equal-target common Chord and manual orphan bypass. Positive mirror/native orphan/clump/tie/edge insertion cases remain unexercised.','cases':len(r),'saved_real_slurs':len(same),'control_cases':len(r)-len(same),'area_queries':sum(len(a['queries'])for r in j['rows']for a in r['areas']),'checks':checks,'finite_tolerance':1e-10,'maximum_scalar_absolute_error':maxerr,'scalar_nonzero_deltas':scalar_deltas,'failures':fail,'source_live_relookup_matches_persisted_heads':same}
(p/'comparison.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['scalar_nonzero_deltas','source_live_relookup_matches_persisted_heads']},indent=2));assert not fail;assert all(r['match']for r in same)
