import pathlib,subprocess,os,json,hashlib
root=pathlib.Path('/workspace/native-build-tools/oracle');out=pathlib.Path('/tmp/native-slur-math-original-v1');out.mkdir()
for n in ['classes','tmp','cache','home','prefs','config','data']:(out/n).mkdir()
lib=root/'official/opt/audiveris/lib';jar=lib/'app/audiveris.jar';source=pathlib.Path('/tmp/NativeSlurCandidateMathProbe.java');book=pathlib.Path('/tmp/real-rest-original-v1/der-winter-gray.omr');inp=pathlib.Path('/tmp/native-short-slur-exploration.json')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(jar)=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
d=json.loads(inp.read_text());rows=[(i,c['kind'],c['points']) for i,c in enumerate(d['candidates'])]
rows += [(153,'control_horizontal',[[x,0] for x in range(16)]),(154,'control_diagonal',[[x,x] for x in range(16)]),(155,'control_duplicate',[[0,0],[0,0],[0,0]]),(156,'control_shallow_pixel_bump',[[x,1 if x==10 else 0] for x in range(21)])]
seq=out/'sequences.tsv';seq.write_text(''.join(str(i)+'\t'+kind+'\t'+';'.join(f'{x},{y}' for x,y in pts)+'\n' for i,kind,pts in rows))
compile=[str(root/'jdk-21.0.12.1+1/bin/javac'),'--release','21','-d',str(out/'classes'),str(source)]
with (out/'compile.log').open('w') as f:subprocess.run(compile,stdout=f,stderr=subprocess.STDOUT,check=True)
cmd=[str(lib/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-cp',str(lib/'app/*')+os.pathsep+str(out/'classes'),'NativeSlurCandidateMathProbe',str(book),str(seq),str(out/'observed.json')]
env=dict(os.environ)
for k,n in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[k]=str(out/n)
with (out/'console.log').open('w') as f:r=subprocess.run(cmd,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=60)
p={'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':sha(jar),'observer_sha256':sha(source),'runner_sha256':sha(pathlib.Path(__file__)),'original_book_sha256':sha(book),'native_exploration_sha256':sha(inp),'sequences_sha256':sha(seq),'compile_argv':compile,'run_argv':cmd,'exit_code':r.returncode,'native_sequence_count':153,'controls_count':4,'image_recognition_invoked':False,'ordinary_lifecycle_claimed':False}
if (out/'observed.json').exists():p['observed_sha256']=sha(out/'observed.json')
(out/'provenance.json').write_text(json.dumps(p,indent=2)+'\n');print(str(out),r.returncode)
raise SystemExit(r.returncode)
