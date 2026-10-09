#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Observe original ledger memberships from unchanged image through fresh HEADS."""
import argparse,os,json,subprocess
from pathlib import Path
import linux_oracle as oracle
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--oracle-home',type=Path,required=True);p.add_argument('--image',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--probe',choices=('RealLedgerMembershipImageProbe','RealLedgerReductionImageProbe','RealMeasuresImageProbe','RealPartMeasuresImageProbe','RealHeadChordsImageProbe','RealStemGeometryImageProbe','RealSymbolHeaderImageProbe','RealSymbolAreaImageProbe','RealSymbolCleanerImageProbe','RealSymbolCandidatesImageProbe','RealSymbolHeadPixelsImageProbe','RealSymbolClusterOrderImageProbe','RealSymbolAltersImageProbe','RealDotStackImageProbe','RealDotGeometryImageProbe','RealDotsFactoryImageProbe','RealRepeatDotLookupImageProbe','RealSupportedAlterImageProbe','RealContextProjectionImageProbe','RealRecognitionContainersImageProbe','RealChordDurationImageProbe','RealVoiceFamilyImageProbe','RealVoiceConstructionImageProbe','RealVoiceDistanceImageProbe','RealFlagCompetitionStagesImageProbe','RealFlagOverlapOrderImageProbe','RealFlagCompetitionManualImageProbe','RealFlagRestWeakImageProbe','RealFlagRestManualImageProbe','RealTimeLateImageProbe','RealTimeColumnFailureImageProbe'),default='RealLedgerMembershipImageProbe');a=p.parse_args();out=a.output.resolve();out.mkdir()
for name in ('compile-only','classes','tmp','cache','home','prefs','config','data'):(out/name).mkdir()
lib=a.oracle_home.resolve()/'official/opt/audiveris/lib';jar=lib/'app/audiveris.jar';oracle.checked(jar,oracle.JAR_SHA256);javac=a.oracle_home.resolve()/oracle.JDK_DIR/'bin/javac';source=oracle.ROOT/('scripts/java/'+a.probe+'.java');stub=out/'compile-only';classes=out/'classes'
(stub/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n');(stub/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(stub),str(stub/'Book.java'),str(stub/'RunClass.java')],[str(javac),'--release','21','-cp',str(stub),'-d',str(classes),str(source)]]
if a.probe != 'RealLedgerMembershipImageProbe':commands[-1].append(str(oracle.ROOT/'scripts/java/RealLedgerMembershipImageProbe.java'))
if a.probe == 'RealSymbolClusterOrderImageProbe':commands[-1].append(str(oracle.ROOT/'scripts/java/RealSymbolCandidatesImageProbe.java'))
with (out/'compile.log').open('w') as log:
 for cmd in commands:subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,check=True)
cmd=[str(lib/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.output='+str(out/'observed.json'),'-cp',str(lib/'app/*')+os.pathsep+str(classes),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run',a.probe,'-output',str(out),'--',str(a.image.resolve())]
env=dict(os.environ)
for key,name in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[key]=str(out/name)
with (out/'console.log').open('w') as log:result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=120)
v=dict(upstream_commit=oracle.UPSTREAM_COMMIT,jar_sha256=oracle.sha(jar),observer_sha256=oracle.sha(source),image_file_sha256=oracle.sha(a.image),compile_commands=commands,compile_only_signatures_on_runtime_classpath=False,run_argv=cmd,exit_code=result.returncode,console_sha256=oracle.sha(out/'console.log'),native_outputs_used=False)
if a.probe != 'RealLedgerMembershipImageProbe':v['image_helper_sha256']=oracle.sha(oracle.ROOT/'scripts/java/RealLedgerMembershipImageProbe.java')
if (out/'observed.json').is_file():v['observed_sha256']=oracle.sha(out/'observed.json')
(out/'provenance.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(dict(output=str(out),exit_code=result.returncode)),flush=True)
raise SystemExit(result.returncode if result.returncode else (0 if (out/'observed.json').is_file() else 1))
