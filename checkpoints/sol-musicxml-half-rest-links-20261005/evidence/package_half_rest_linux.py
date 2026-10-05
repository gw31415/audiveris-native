from pathlib import Path
import tarfile,io,json,hashlib,subprocess,gzip
W=Path('/workspace/work/musicxml-stems44');S=Path('/workspace/onboarding/artifacts/audiveris-source');C=Path('/workspace/audiveris-native/checkpoints/sol-musicxml-half-rest-links-20261005');O=Path('/codex/browser/projectless/0001-continue-audiveris-heads-port-audiveris-rust-cli/output');exe=Path('/workspace/onboarding/target/release/audiveris-musicxml');proof=json.loads((W/'HALFREST-RELEASE-XSD-ORACLE-REIMPORT-VERIFIED.json').read_text());full=json.loads((W/'HALFREST-FULL-WORKSPACE-VERIFIED.json').read_text());assert full['status']=='passed' and full['source_commit']==proof['source_commit'];sha=lambda b:hashlib.sha256(b).hexdigest()
readme=f'''Audiveris native limited Linux CLI
Source: {proof['source_commit']}
License: AGPL-3.0-or-later; LICENSE included. Corresponding source is recovered
from private gw31415/audiveris-native checkpoint sol-musicxml-half-rest-links-20261005
with its RESTORE.py and full SOURCE-SHA256.json. No original Java runtime required.

Linux x86_64 ELF64, dynamic libgcc_s/libm/libc, symbols require GLIBC_2.35.
Tested only in the existing Linux Cloud environment, not every Linux distribution.
Binary SHA256: {proof['binary_SHA256']}
Build: CARGO_INCREMENTAL=0 cargo build --release --offline --locked --bin audiveris-musicxml --jobs 1
Full workspace tests: {full['tests_passed']} passed on this source; fmt/clippy passed.

Extract, then run from this directory:
  sha256sum -c SHA256SUMS
  mkdir out
  ./audiveris-musicxml --output-dir out --part-name Flute examples/quarter-rest.png examples/half-two-measures.png examples/quarter-half-break.png examples/half-rest-y241.png
  ./audiveris-musicxml examples/half-rest-y239.png half-rest.musicxml half-rest.report.json
Outputs must be new; output directory must exist. A later batch refusal keeps
earlier successful pairs. Part name is metadata, not instrument recognition.

限定対応: 不透明PNG/PGM、400万画素以下、Bravura/interline11、傾き0、通常五線、
ト音記号/C major/明示4拍子、単一非移調part/単声、完全小節の四分・二分音符と休符。
3独立合成譜面24イベントと、同じ二分休符譜面の2位置条件10イベントを区別して検証。
原版との比較境界はXML保存HEADS以後。native CLI自身は画像から認識し、原版XMLは使わない。
二分休符のbeam処理はstump/seedなし・到達可能headなしの限定条件。
弱い休符を除去して小節が不足すれば拒否する。音符や休符を補造しない。
未対応: 任意実スキャン/写真/PDF、別フォント/サイズ/斜め/特殊staff、全24形ページ、
実際のbeam/flag八分音符、臨時記号/付点/tie/slur/tuplet、多声/弱起/複数staff/移調。
限定cleaner/epilog確認は完全STEMS/LINKS移植を意味しない。全体完了フラグはfalse。
This is a bounded prototype CLI, not practical completion for arbitrary scores.
'''.encode()
files={'audiveris-musicxml':exe.read_bytes(),'README.txt':readme,'LICENSE':(S/'LICENSE').read_bytes(),'SOURCE.json':json.dumps({'source_commit':proof['source_commit'],'source_tree':full['source_tree'],'binary_SHA256':proof['binary_SHA256'],'full_tests_passed':full['tests_passed'],'heads_complete':False,'stems_complete':False,'musicxml_complete':False,'epilog_complete':False},indent=2).encode()+b'\n'}
for case in proof['cases']:
 n=case['case'];files['examples/'+n+'.png']=(W/'half-rest-release-outputs/inputs'/(n+'.png')).read_bytes();files['expected/'+n+'.musicxml']=(W/'half-rest-release-outputs'/(n+'.musicxml')).read_bytes()
files['SHA256SUMS']=(''.join(sha(b)+'  '+n+'\n' for n,b in sorted(files.items()))).encode()
p=C/'audiveris-musicxml-linux-x86_64.tar.gz'
with p.open('wb') as raw:
 with gzip.GzipFile(filename='',mode='wb',fileobj=raw,mtime=0) as gz:
  with tarfile.open(fileobj=gz,mode='w') as tar:
   for n,b in sorted(files.items()):
    info=tarfile.TarInfo(n);info.size=len(b);info.mode=0o755 if n=='audiveris-musicxml' else 0o644;info.mtime=0;tar.addfile(info,io.BytesIO(b))
with tarfile.open(p,'r:gz') as tar:
 for n,b in files.items():assert tar.extractfile(n).read()==b
# User-facing outputs share verified archive bytes; this is a deliverable, not a restore copy.
# The required projectless output path is denied on this Cloud executor.
# Do not escalate or change permissions; deliver the private Git artifact.
if O.is_dir():
 (O/p.name).write_bytes(p.read_bytes());(O/'README.txt').write_bytes(readme)
r={'status':'passed','source_commit':proof['source_commit'],'archive':str(p),'archive_SHA256':sha(p.read_bytes()),'archive_bytes':p.stat().st_size,'binary_SHA256':proof['binary_SHA256'],'members_SHA256':{n:sha(b)for n,b in files.items()},'packaged_binary_readback_verified':True,'fresh_remote_binary_restoration_not_yet_verified':True,'Linux_scope':'x86_64 dynamic GLIBC_2.35/libgcc_s/libm/libc; tested Cloud host only'};(W/'HALFREST-LINUX-PACKAGE-VERIFIED.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items()if k!='members_SHA256'}))
