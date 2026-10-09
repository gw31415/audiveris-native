from pathlib import Path
import json,subprocess,hashlib
out=Path('/tmp/slur-native-clump-original-v1');source=json.loads((out/'observed.json').read_text());groups=json.loads((out/'native-groups.json').read_text());cases=[]
for g in source['groups']:
 for ai,a in enumerate(g['attempts']):cases.append({'case':f'group_{g["group"]}_attempt_{ai}','input':[x['entry'] for x in a['inputs'] if x['entry'] is not None],'selected':a['selected']})
cases+=source['controls']
lines=[]
for c in cases:
 rows=[]
 for e in c['input']:
  f=[e['candidate'],str(e['point_count'])]
  for l in e['links']:f+=['-','-'] if l is None else [str(l['chord']),repr(l['euclidean'])]
  rows.append(','.join(f))
 lines.append(c['case']+'\t'+(';'.join(rows) or '-'))
(out/'selection-inputs.tsv').write_text('\n'.join(lines)+'\n')
(out/'component-inputs.tsv').write_text(''.join(str(r['case'])+'\t'+','.join(map(str,r['component_indices']))+'\n' for r in groups['rows']))
rustc='/workspace/native-build-tools/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin/rustc'
cmd=[rustc,'--edition','2024','/tmp/check_slur_clump_selection.rs','-o','/tmp/check_slur_clump_selection']
subprocess.run(cmd,check=True);subprocess.run(['/tmp/check_slur_clump_selection',str(out/'selection-inputs.tsv'),str(out/'component-inputs.tsv'),str(out/'rust-observed.tsv')],check=True)
actual=[line.split('\t') for line in (out/'rust-observed.tsv').read_text().splitlines()];diff=[];n=0;delta=0.;lookup={r[0]:r for r in actual if r[0]!='GROUP'}
for c in cases:
 r=lookup[c['case']];e=c['selected'];name=e['candidate'] if e else '-';n+=1
 if r[1]!=name:diff.append([c['case'],'selected',name,r[1]])
 if e:
  d=abs(float(r[2])-e['weighted_distance']);delta=max(delta,d);n+=1
  if d>1e-14:diff.append([c['case'],'weight',e['weighted_distance'],r[2]])
expected_groups=[[str(x) for x in g['cases']] for g in groups['groups']];actual_groups=[r[1].split(',') for r in actual if r[0]=='GROUP'];n+=len(expected_groups)
if expected_groups!=actual_groups:diff.append(['groups',expected_groups,actual_groups])
result={'scope':'Identical original-computed lookup entries compared to Rust source clump selection, plus actual native component-sharing grouping. Original actual ClumpLinker.selectSlur result is expected for 10 native groups;8 controlled original private selectAmong/getNonOrphans cases are separate. No native Head owner integration, original Skeleton grouping provenance, Slur lifecycle or quality claim.','cases':len(cases),'native_groups':len(source['groups']),'original_controls':len(source['controls']),'comparisons':n,'mismatch_count':len(diff),'max_weighted_delta':delta,'mismatches':diff,'rust_local_controls':['distinct equal-valued component objects do not share identity','transitive shared ownership preserves insertion order']}
(out/'comparison.json').write_text(json.dumps(result,indent=2)+'\n')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
(out/'RUST-PROVENANCE.json').write_text(json.dumps({'compile_argv':cmd,'module_sha256':sha('/tmp/slur_clump_selection_proposal.rs'),'harness_sha256':sha('/tmp/check_slur_clump_selection.rs'),'comparison_script_sha256':sha(__file__)},indent=2)+'\n')
print(json.dumps(result,indent=2))
