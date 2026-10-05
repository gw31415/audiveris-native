#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Independent remote clone -> streamed full source Git reconstruction.
# Avoids a duplicate source worktree and does not borrow local Git objects.
from pathlib import Path
import subprocess,hashlib,json,tarfile,io,base64,os,sys
W=Path('/workspace/work/musicxml-ledger-single');R=Path('/workspace/audiveris-native');D=W/'fresh-main.git';G=W/'fresh-source.git';stage='sol-musicxml-ledger-cluster-20261005';sha=lambda b:hashlib.sha256(b).hexdigest()
def run(g,*args,input=None):return subprocess.check_output(['git','--git-dir='+str(g),'-c','gc.auto=0',*args],input=input,stderr=subprocess.PIPE)
def archive(path):return run(D,'show','HEAD:'+str(path))
assert not D.exists() and not G.exists()
remote=subprocess.check_output(['git','-C',str(R),'remote','get-url','origin'],text=True).strip()
p=subprocess.run(['git','clone','--bare','--depth','1','--single-branch','--branch','main',remote,str(D)],capture_output=True);assert p.returncode==0,'fresh ordinary connection clone failed; details held without credentials'
assert not (D/'objects/info/alternates').exists()
head=run(D,'rev-parse','HEAD').decode().strip();assert head==subprocess.check_output(['git','-C',str(R),'rev-parse','HEAD'],text=True).strip();tree=run(D,'rev-parse','HEAD^{tree}').decode().strip();assert tree==subprocess.check_output(['git','-C',str(R),'rev-parse','HEAD^{tree}'],text=True).strip()
rows=run(D,'ls-tree','-rz','HEAD').split(b'\0');cat=subprocess.Popen(['git','--git-dir='+str(D),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
count=0
for row in rows:
 if not row:continue
 meta,n=row.split(b'\t',1);oid=meta.split()[2];cat.stdin.write(oid+b'\n');cat.stdin.flush();header=cat.stdout.readline().split();b=cat.stdout.read(int(header[2]));assert cat.stdout.read(1)==b'\n';assert sha(b)==sha((R/os.fsdecode(n)).read_bytes());count+=1
cat.stdin.close();assert cat.wait()==0
print('Fresh archive all blobs SHA256/tree matched:',count,flush=True)
m=json.loads(archive('checkpoints/'+stage+'/MANIFEST.json'));base=json.loads(archive('checkpoints/sol-snapshot-loop-20261003/MANIFEST.json'));assert sha(archive(m['all_source_sha256']['path']))==m['all_source_sha256']['sha256']
subprocess.run(['git','init','--bare',str(G)],check=True,capture_output=True)
outer=archive(base['base_outer_archive_path']);assert sha(outer)==base['base_outer_archive_sha256']
with tarfile.open(fileobj=io.BytesIO(outer),mode='r:gz') as t:inner=t.extractfile('verified-source-8ebbf976-20261003/source.tar.xz').read()
assert sha(inner)==base['base_source_archive_sha256'];entries=[]
with tarfile.open(fileobj=io.BytesIO(inner),mode='r:xz') as t:
 for member in t:
  if not member.isfile():continue
  assert member.name.startswith('audiveris-source/');name=member.name[len('audiveris-source/'):];assert '..' not in Path(name).parts
  b=t.extractfile(member).read();oid=run(G,'hash-object','-w','--stdin',input=b).decode().strip();mode='100755' if member.mode&0o111 else '100644';entries.append(f'{mode} {oid}\t{name}\n')
run(G,'update-index','--index-info',input=''.join(entries).encode());assert run(G,'write-tree').decode().strip()==base['base_source_tree']
del outer,inner,entries
snapshot=archive(base['archive']);assert sha(snapshot)==base['archive_sha256']
with tarfile.open(fileobj=io.BytesIO(snapshot),mode='r:gz') as t:
 preimage=t.extractfile('sol-snapshot-loop-20261003/BASE-COMMIT.txt').read();history=t.extractfile('sol-snapshot-loop-20261003/history.bundle').read()
assert run(G,'hash-object','-t','commit','-w','--stdin',input=preimage).decode().strip()==base['reconstructed_local_base_commit'];run(G,'update-ref','HEAD',base['reconstructed_local_base_commit'])
bundle=W/'restore-one.bundle';bundle.write_bytes(history);run(G,'bundle','verify',str(bundle));run(G,'fetch','--no-tags',str(bundle),base['local_implementation_commit']);run(G,'update-ref','HEAD',base['local_implementation_commit']);run(G,'read-tree','HEAD');assert run(G,'write-tree').decode().strip()==base['local_implementation_tree']
manifest={}
for row in rows:
 if not row:continue
 _,n=row.split(b'\t',1);n=os.fsdecode(n)
 if n.startswith('checkpoints/') and n.endswith('/MANIFEST.json'):
  item=json.loads(archive(n))
  if item.get('patch') or item.get('bundle'):manifest[item['local_implementation_commit']]=item
chain=[];commit=m['local_implementation_commit']
while commit!=base['local_implementation_commit']:
 item=manifest[commit];assert item not in chain;chain.append(item);commit=item.get('parent_implementation_commit') or item['bundle']['prerequisite']
for i,item in enumerate(reversed(chain)):
 if item.get('bundle'):
  d=item['bundle'];encoded=archive(d['path']);assert sha(encoded)==d['base64_sha256'];b=base64.b64decode(encoded);assert sha(b)==d['decoded_sha256'] and len(b)==d['decoded_bytes'];bundle.write_bytes(b);run(G,'bundle','verify',str(bundle));run(G,'fetch','--no-tags',str(bundle),item['local_implementation_commit']);run(G,'update-ref','HEAD',item['local_implementation_commit']);run(G,'read-tree','HEAD')
 else:
  d=item['patch'];patch=archive(d['path']);assert sha(patch)==d['sha256'] and len(patch)==d['bytes'];run(G,'apply','--cached','--check','-',input=patch);run(G,'apply','--cached','-',input=patch);assert run(G,'write-tree').decode().strip()==item['local_implementation_tree'];preimage=archive(item['implementation_commit_preimage']);oid=run(G,'hash-object','-t','commit','-w','--stdin',input=preimage).decode().strip();assert oid==item['local_implementation_commit'];run(G,'update-ref','HEAD',oid)
 assert run(G,'write-tree').decode().strip()==item['local_implementation_tree']
 if (i+1)%20==0:print('Historical source chain verified:',i+1,'/',len(chain),flush=True)
assert not (G/'objects/info/alternates').exists();assert run(G,'rev-parse','HEAD').decode().strip()==m['local_implementation_commit'];source_tree=run(G,'write-tree').decode().strip();assert source_tree==m['local_implementation_tree']
hashes=json.loads(archive(m['all_source_sha256']['path']));source_rows=run(G,'ls-tree','-rz','HEAD').split(b'\0');cat=subprocess.Popen(['git','--git-dir='+str(G),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE);seen=set()
for row in source_rows:
 if not row:continue
 meta,n=row.split(b'\t',1);n=os.fsdecode(n);oid=meta.split()[2];cat.stdin.write(oid+b'\n');cat.stdin.flush();header=cat.stdout.readline().split();b=cat.stdout.read(int(header[2]));assert cat.stdout.read(1)==b'\n';assert sha(b)==hashes[n],n;seen.add(n)
cat.stdin.close();assert cat.wait()==0;assert seen==set(hashes)
package=archive(m['linux_cli']['path']);assert sha(package)==m['linux_cli']['sha256'];exe=W/'fresh-audiveris-musicxml'
with tarfile.open(fileobj=io.BytesIO(package),mode='r:gz') as t:
 members={member.name:t.extractfile(member).read()for member in t if member.isfile()};lines=members['SHA256SUMS'].decode().splitlines()
 for line in lines:
  h,n=line.split('  ',1);assert sha(members[n])==h
 assert sha(members['audiveris-musicxml'])==m['linux_cli']['executable_SHA256'];exe.write_bytes(members['audiveris-musicxml']);exe.chmod(0o755)
 inp=W/'fresh-ledger-quarter.png';inp.write_bytes(members['examples/ledger-quarter.png']);xml=W/'fresh-ledger-quarter.musicxml';report=W/'fresh-ledger-quarter.report.json';p=subprocess.run([str(exe),str(inp),str(xml),str(report)],capture_output=True);assert p.returncode==0;assert xml.read_bytes()==members['expected/ledger-quarter.musicxml'];events=json.loads(report.read_text())['events'];assert len(events)==8
proof={'status':'passed','remote_main':head,'archive_tree':tree,'all_fresh_archive_blob_SHA256_verified':count,'source_commit':m['local_implementation_commit'],'source_tree':source_tree,'all_fresh_restored_source_blob_SHA256_verified':len(seen),'historical_delta_chain_verified':len(chain),'restore_mode':'independent full source Git objects/index reconstruction; no duplicate source worktree','source_worktree_materialization_not_performed':True,'local_object_borrowing_or_alternates':False,'fresh_package_executable_SHA256':sha(members['audiveris-musicxml']),'fresh_package_CLI_image_exit_code':0,'fresh_package_expected_MusicXML_byte_match':True,'native_events':len(events),'Library_attempted':False}
(W/'FRESH-GIT-SOURCE-CLI-VERIFIED.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof),flush=True);bundle.unlink()
