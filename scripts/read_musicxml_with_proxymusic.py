#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Read emitted MusicXML using the existing pinned original ProxyMusic tools."""
import argparse,hashlib,json,subprocess
from pathlib import Path
import linux_oracle as oracle
PROXY_SHA='933fe64e7136b0d98a186b61fc139fc9e1ee98b64662b60c9cf9af37b5f8f107'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--oracle-home',type=Path,required=True);p.add_argument('--xml',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 out=a.output.resolve();out.mkdir();classes=out/'classes';classes.mkdir();lib=a.oracle_home.resolve()/'official/opt/audiveris/lib';app=lib/'app';oracle.checked(app/'audiveris.jar',oracle.JAR_SHA256);oracle.checked(app/'proxymusic-4.0.3.jar',PROXY_SHA)
 source=oracle.ROOT/'scripts/java/ReadMusicXml.java';javac=a.oracle_home.resolve()/oracle.JDK_DIR/'bin/javac';compile=[str(javac),'--release','21','-cp',str(app/'*'),'-d',str(classes),str(source)]
 with (out/'compile.log').open('w') as f:subprocess.run(compile,stdout=f,stderr=subprocess.STDOUT,check=True,timeout=60)
 run=[str(lib/'runtime/bin/java'),'-cp',str(app/'*')+':'+str(classes),'ReadMusicXml',str(a.xml.resolve())]
 with (out/'read.json').open('w') as f, (out/'read.log').open('w') as log:r=subprocess.run(run,stdout=f,stderr=log,timeout=60)
 v={'reader_source_sha256':oracle.sha(source),'proxymusic_jar_sha256':PROXY_SHA,'existing_original_jar_sha256':oracle.JAR_SHA256,'input_musicxml_sha256':oracle.sha(a.xml),'compile_argv':compile,'run_argv':run,'exit_code':r.returncode,'DTD_and_external_entities_disabled':True,'no_recognition_or_native_inputs_created':True}
 (out/'PROVENANCE.json').write_text(json.dumps(v,indent=2)+'\n')
 if r.returncode:raise SystemExit(r.returncode)
 print((out/'read.json').read_text().strip())
if __name__=='__main__':main()
