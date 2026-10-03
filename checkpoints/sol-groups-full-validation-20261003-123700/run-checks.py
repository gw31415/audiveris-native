import os,pathlib,subprocess,json,datetime,time,re,signal
r=pathlib.Path('/workspace/audiveris-groups-release-check-20261003'); cache=pathlib.Path('/workspace/audiveris-debug-check-20261003'); source=pathlib.Path('/workspace/audiveris-savedlatest-20261003'); prefix=cache/'rust-1.98.1'; expected='b19d73263e6da118b5c3acbece8d5e90f9ea8d83';started=time.monotonic();env=os.environ.copy();env['PATH']=str(prefix/'bin')+':'+env['PATH'];env['CARGO_HOME']=str(cache/'cargo-home');env['CARGO_TARGET_DIR']=str(cache/'target');env['RUSTC']=str(prefix/'bin/rustc');env['RUSTDOC']=str(prefix/'bin/rustdoc');env['CARGO_BUILD_JOBS']='2';env['CARGO_TERM_COLOR']='never'
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(*args):return subprocess.check_output(['git','-C',str(source),*args]).decode().strip()
def tree():
 ie=env.copy();ie['GIT_INDEX_FILE']=str(r/'verification.index'); subprocess.run(['git','-C',str(source),'read-tree','HEAD'],env=ie,check=True); subprocess.run(['git','-C',str(source),'add','-u'],env=ie,check=True,stdout=subprocess.DEVNULL)
 return subprocess.check_output(['git','-C',str(source),'write-tree'],env=ie).decode().strip()
assert git('rev-parse','HEAD')=='6eeebeadab1048d62aa46300d38efeae86421181'
state={'source_commit':git('rev-parse','HEAD'),'source_checkpoint_commit':'dfc92629df63532d8726935e5f4f5ca784a62204','status':'running','runner_pid':os.getpid(),'started_utc':now(),'source_root':str(source),'source_expected_tree':expected,'source_tree_before':tree(),'source_clean_before':not git('status','--porcelain'),'checks':[],'release_checks':'release tests and release workspace build only','implementation_changes':'none','command_budget_seconds':480}
assert state['source_tree_before']==expected and state['source_clean_before']
def save():state['elapsed_seconds']=round(time.monotonic()-started,2);(r/'state.json').write_text(json.dumps(state,indent=2)+'\n')
for binary in ['rustc','cargo']:
 state[binary]=subprocess.check_output([str(prefix/'bin'/binary),'--version'],env=env).decode().strip()
save()
commands=[('release-tests',['cargo','test','--workspace','--release','--locked','--offline','-j','2']),('release-build',['cargo','build','--workspace','--release','--locked','--offline','-j','2'])]
try:
 for name,command in commands:
  remaining=480-(time.monotonic()-started)
  if remaining<=0:state['status']='timed_out';break
  step={'name':name,'command':command,'status':'running','started_utc':now(),'log':str(r/(name+'.log'))};state['checks'].append(step)
  with open(step['log'],'w') as log:
   p=subprocess.Popen(command,cwd=source,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True);step['pid']=p.pid;save();tick=time.monotonic()
   try:code=p.wait(timeout=remaining)
   except subprocess.TimeoutExpired:
    os.killpg(p.pid,signal.SIGTERM)
    try:p.wait(timeout=10)
    except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
    step['status']='timed_out';step['exit_code']=p.returncode;state['status']='timed_out';step['finished_utc']=now();step['seconds']=round(time.monotonic()-tick,2);save();break
  step['exit_code']=code;step['status']='passed' if code==0 else 'failed';step['finished_utc']=now();step['seconds']=round(time.monotonic()-tick,2)
  if name=='release-tests':
   text=pathlib.Path(step['log']).read_text();summaries=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);step['test_summary']={'suites':len(summaries),'passed':sum(int(x[1]) for x in summaries),'failed':sum(int(x[2]) for x in summaries),'ignored':sum(int(x[3]) for x in summaries)};step['failed_tests']=re.findall(r'^test (.*?) \.\.\. FAILED$',text,re.M)
  save()
 else:state['status']='passed' if all(x['status']=='passed' for x in state['checks']) else 'failed'
except Exception as e:state['status']='runner_error';state['error']=repr(e)
finally:
 state['source_tree_after']=tree();state['source_clean_after']=not git('status','--porcelain');state['source_unchanged']=state['source_tree_after']==expected and state['source_clean_after'];state['finished_utc']=now();save();(r/'COMPLETION-RECEIPT.json').write_text(json.dumps(state,indent=2)+'\n');print(json.dumps(state,indent=2),flush=True)
