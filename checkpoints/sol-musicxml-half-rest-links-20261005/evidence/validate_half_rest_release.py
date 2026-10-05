from pathlib import Path
import subprocess,json,hashlib,os,time
from lxml import etree
W=Path('/workspace/work/musicxml-stems44');S=Path('/workspace/onboarding/artifacts/audiveris-source');binary=Path('/workspace/onboarding/target/release/audiveris-musicxml');out=W/'half-rest-release-outputs';out.mkdir();inp=out/'inputs';inp.mkdir();cases=[]
names=['quarter-rest','half-two-measures','quarter-half-break','half-rest-y239','half-rest-y241']
for name in names:
 f=S/('fixtures/official-image-musicxml-half-rest' if name.startswith('half-rest-y') else 'fixtures/official-image-musicxml-limited')/(name+'.png');(inp/f.name).write_bytes(f.read_bytes())
args=[str(binary),'--output-dir',str(out),'--part-name','Flute',*[str(inp/(n+'.png')) for n in names]]
trace=out/'native-runtime.trace';start=time.monotonic();p=subprocess.run(['strace','-f','-e','trace=execve,connect','-o',str(trace),*args],cwd=out,capture_output=True);(out/'console.log').write_bytes(p.stdout+p.stderr);assert p.returncode==0,p.stderr;elapsed=time.monotonic()-start
tr=trace.read_text();assert tr.count('execve(')==1 and 'connect(' not in tr
schema=etree.XMLSchema(etree.parse(str(W/'MusicXML-schema/musicxml.xsd')))
prior=json.loads((W/'BATCH-CURRENT-XSD-ORACLE-REIMPORT-VERIFIED.json').read_text())['cases'][0]['reimport_argv'];oldroot=str(W/'reimport-current-quarter-rest')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for name in names:
 xml=out/(name+'.musicxml');report=out/(name+'.report.json');d=json.loads(report.read_text());schema.assertValid(etree.parse(str(xml)));events=d['events'];assert d['heads_complete']==False and d['stems_complete']==False and d['musicxml_complete']==False and d['epilog_complete']==False
 f=S/('fixtures/official-image-musicxml-half-rest' if name.startswith('half-rest-y') else 'fixtures/official-image-musicxml-limited')/(name+'-observed.json');oracle=json.loads(f.read_text())
 for sys in oracle['systems']:
  for n in sys['notes']:
   e=next(e for e in events if e['system_id']==sys['system'] and e['bounds']==n['bounds'] and e['shape']==n['shape'])
   for k in ['rest','step','octave','duration','time_offset','voice','dots','measure_index']:assert e[k]==n[k],(name,k,e[k],n[k])
 dest=W/('half-rest-final-reimport-'+name);dest.mkdir()
 for sub in ['home','tmp','prefs','config','data']:(dest/sub).mkdir()
 argv=[a.replace(oldroot,str(dest)) for a in prior];argv[-2]=str(xml);argv[-1]=str(dest)
 for i,a in enumerate(argv):
  if a.startswith('-Dorg.bytedeco.javacpp.cachedir='):argv[i]='-Dorg.bytedeco.javacpp.cachedir='+str(W/'reimport-current-quarter-rest/cache')
 env=os.environ.copy();env.update({'XDG_CONFIG_HOME':str(dest/'config'),'XDG_DATA_HOME':str(dest/'data'),'XDG_CACHE_HOME':str(W/'reimport-current-quarter-rest/cache')})
 p=subprocess.run(argv,env=env,capture_output=True,timeout=60);(dest/'console.log').write_bytes(p.stdout+p.stderr);assert p.returncode==0,p.stderr
 re=json.loads((dest/'result.json').read_text())['events'];assert len(re)==len(events)
 for e,r in zip(events,re):
  for k in ['rest','step','octave','duration_quarters']:assert e[k]==r[k],(name,k)
  assert e['measure_number']==r['measure'] and str(e['voice'])==r['voice']
  assert r['offset_quarters']==sum(e0['duration_quarters'] for e0 in events[:events.index(e)] if e0['measure_number']==e['measure_number'])
 cases.append({'case':name,'events':len(events),'measures':len(d['measures']),'XSD':'passed','original_XML_loaded_rhythm_fields':'matched','original_proxymusic_reimport':'matched','input_SHA256':sha(inp/(name+'.png')),'MusicXML_SHA256':sha(xml),'report_SHA256':sha(report),'reimport_result_SHA256':sha(dest/'result.json'),'reimport_argv':argv,'reference_render_not_general_engraver':True})
head=subprocess.check_output(['git','--git-dir=/workspace/onboarding/source-git','rev-parse','HEAD'],text=True).strip()
proof={'status':'passed','source_commit':head,'binary_SHA256':sha(binary),'binary_bytes':binary.stat().st_size,'argv':args,'cwd':str(out),'native_runtime_execve_count':1,'native_runtime_connect_count':0,'Java_and_original_recognition_XML_required_for_native_CLI':False,'native_batch_seconds':elapsed,'prior_independent_synthetic_scores_events':24,'additional_controlled_half_rest_position_pages_events':10,'trace_SHA256':sha(trace),'cases':cases,'full_practical_scope_complete':False}
(W/'HALFREST-RELEASE-XSD-ORACLE-REIMPORT-VERIFIED.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps({k:v for k,v in proof.items() if k not in ['cases','argv']}))
