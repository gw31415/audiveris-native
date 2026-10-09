import pathlib,json,hashlib,math
base=pathlib.Path('/tmp/native-slur-math-original-v1');j=json.loads((base/'observed.json').read_text())['rows'];r=json.loads((base/'rust-observed.json').read_text());e=json.loads((base/'rust-extraction.json').read_text());p=json.load(open('/tmp/native-short-slur-exploration.json'))['candidates']
maxima={};states=[];presence=[];mismatches=[]
def compare(a,b,key,i):
 if isinstance(a,dict):
  for k in a:compare(a[k],b[k],key+'.'+k,i)
 elif isinstance(a,list):
  assert len(a)==len(b)
  for aa,bb in zip(a,b):compare(aa,bb,key,i)
 elif isinstance(a,(int,float)) and not isinstance(a,bool):
  assert isinstance(b,(int,float)),(i,key,a,b)
  delta=abs(a-b)
  if key not in maxima or delta>maxima[key]['delta']:maxima[key]={'delta':delta,'candidate_index':i}
 elif a!=b:states.append([i,key,a,b])
for a,b in zip(j,r):
 i=a['candidate_index'];assert i==b['candidate_index']
 if a['math_accepted_before_glyph_and_linkage']!=b['math_accepted']:mismatches.append(i)
 for jk,rk in [('rough_before_public_distance','rough'),('fitted_before_public_distance','fitted'),('selected_before_impacts','selected_before_impacts'),('selected_after_impacts','selected_after_impacts'),('bezier','bezier')]:
  if (a.get(jk)is not None)!=(b.get(rk)is not None):presence.append([i,jk])
  if a.get(jk)is not None and b.get(rk)is not None:compare(a[jk],b[rk],rk,i)
 if (a.get('impacts')is not None)!=(b.get('impacts')is not None):presence.append([i,'impacts'])
 if a.get('impacts')and b.get('impacts'):compare([x['clamped_impact']for x in a['impacts']],b['impacts'],'impacts',i);compare(a['grade'],b['grade'],'grade',i)
extraction_equal=len(e['candidates'])==len(p)and all(a['component_indices']==b['component_indices']and a['points']==b['points']and a['weight']==b['weight']for a,b in zip(e['candidates'],p))
report={'scope':'Independent actual original Java is expected output. Standalone Rust proposal uses JAMA matrix operation order and bundled FdLibm hypot transcription. Numeric comparison on supplied153real candidate sequences plus4controls only; no complete Slur recognition claim.','rows':len(j),'final_decision_mismatches':mismatches,'nullable_presence_mismatches':presence,'discrete_or_nonfinite_state_mismatches':states,'max_absolute_deltas':maxima,'tolerance_all_reported_finite_scalars':1e-12,'all_finite_deltas_within_tolerance':all(x['delta']<=1e-12 for x in maxima.values()),'extraction_component_sets_points_weights_match_all153':extraction_equal,'rust_extraction_counts':{k:v for k,v in e.items()if k!='candidates'},'selected_before_circle_centers_radii_distances_exact_on_observed_values':all(maxima['selected_before_impacts.'+k]['delta']==0 for k in ['center','radius','distance']), 'circle_constructor_fitted_distances_exact_on_observed_values':maxima['fitted.distance']['delta']==0}
(base/'rust-comparison.json').write_text(json.dumps(report,indent=2)+'\n');assert not(mismatches or presence or states)and extraction_equal and report['all_finite_deltas_within_tolerance'];print(json.dumps({k:v for k,v in report.items()if k!='max_absolute_deltas'},indent=2));print('Nonzero maxima:',{k:v for k,v in maxima.items()if v['delta']})
