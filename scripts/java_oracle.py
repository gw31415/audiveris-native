#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compile unmodified pinned Java algorithm classes with minimal infrastructure adapters.
No Maven, GUI, Java installation, or production dependency. JDK path supplied by caller.
"""
import hashlib,json,math,os,shutil,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PIN='9e1e55cd2746037d059345881c53e6a6754bffbd'
head=subprocess.check_output(['git','-C',str(ROOT/'upstream'),'rev-parse','HEAD'],text=True).strip()
if head != PIN: raise SystemExit('upstream checkout does not match pinned commit')
BASE=ROOT/'upstream/app/src/main/java'
WORK=ROOT/'evidence/generated/java-oracle'
WORK.mkdir(parents=True,exist_ok=True)
def write(name,text):
    p=WORK/(name+'.java'); p.parent.mkdir(parents=True,exist_ok=True); p.write_text(text)
originals=['image/AdaptiveFilter','image/RandomFilter','image/VerticalFilter','image/PixelSource','image/PixelFilter','image/SourceWrapper','run/RunsRetriever']
for relative in originals:
    source=BASE/('org/audiveris/omr/'+relative+'.java')
    target=WORK/('org/audiveris/omr/'+relative+'.java'); target.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(source,target)
# Only peripheral services are substituted. Population/getContext is deliberately unavailable:
# production isFore uses tile's population variance; getContext uses sample variance upstream.
stubs={
'net/jcip/annotations/ThreadSafe': 'package net.jcip.annotations; public @interface ThreadSafe {}',
'net/jcip/annotations/NotThreadSafe': 'package net.jcip.annotations; public @interface NotThreadSafe {}',
'ij/process/ByteProcessor': '''package ij.process; public class ByteProcessor { private final int w,h; private final int[] p; public ByteProcessor(int w,int h){this.w=w;this.h=h;p=new int[w*h];} public int getWidth(){return w;} public int getHeight(){return h;} public int get(int x,int y){return p[y*w+x];} public void set(int x,int y,int v){p[y*w+x]=v;} }''',
'org/slf4j/Logger': '''package org.slf4j; public interface Logger { default void warn(String s,Object... v){} default void error(String s,Object... v){} }''',
'org/slf4j/LoggerFactory': '''package org.slf4j; public class LoggerFactory { public static Logger getLogger(Class<?> c){return new Logger(){};} }''',
'org/audiveris/omr/constant/ConstantSet': '''package org.audiveris.omr.constant; public class ConstantSet {}''',
'org/audiveris/omr/constant/Constant': '''package org.audiveris.omr.constant; public class Constant { public static class Integer { private final int v; public Integer(String u,int v,String d){this.v=v;} public int getValue(){return v;} } }''',
'org/audiveris/omr/math/Population': '''package org.audiveris.omr.math; public class Population { public void includeValue(int v){throw new UnsupportedOperationException();} public int getCardinality(){throw new UnsupportedOperationException();} public double getMeanValue(){throw new UnsupportedOperationException();} public double getStandardDeviation(){throw new UnsupportedOperationException();} }''',
'org/audiveris/omr/run/Run': '''package org.audiveris.omr.run; public record Run(int start,int length) {}''',
'org/audiveris/omr/run/Orientation': '''package org.audiveris.omr.run; import java.awt.Rectangle; public enum Orientation { HORIZONTAL,VERTICAL; public Rectangle oriented(Rectangle r){return this==HORIZONTAL?r:new Rectangle(r.y,r.x,r.height,r.width);} }''',
'org/audiveris/omr/util/Concurrency': '''package org.audiveris.omr.util; public interface Concurrency { boolean isThreadSafe(); }''',
'org/audiveris/omr/util/OmrExecutors': '''package org.audiveris.omr.util; import java.util.concurrent.*; public class OmrExecutors { public static class Flag { public boolean getValue(){return false;} } public static final Flag defaultParallelism=new Flag(); public static ExecutorService getHighExecutor(){throw new UnsupportedOperationException();} }''',
'org/audiveris/omr/step/ProcessingCancellationException': '''package org.audiveris.omr.step; public class ProcessingCancellationException extends RuntimeException { public ProcessingCancellationException(Throwable e){super(e);} }'''
}
for k,v in stubs.items(): write(k,v)
write('org/audiveris/omr/image/Oracle','''package org.audiveris.omr.image;
import ij.process.ByteProcessor;
import org.audiveris.omr.run.*;
import java.awt.Rectangle;
import java.util.*;
public class Oracle extends RandomFilter {
 Oracle(ByteProcessor p){super(p,0.7,0.9);}
 public static void main(String[] args) throws Exception {
  Scanner s=new Scanner(System.in); int w=s.nextInt(),h=s.nextInt();
  ByteProcessor p=new ByteProcessor(w,h);
  for(int y=0;y<h;y++) for(int x=0;x<w;x++) p.set(x,y,s.nextInt());
  Oracle f=new Oracle(p); ByteProcessor b=f.filteredImage();
  VerticalFilter vf=new VerticalFilter(p,0.7,0.9); ByteProcessor vb=vf.filteredImage();
  for(int y=0;y<h;y++) for(int x=0;x<w;x++){
   if(b.get(x,y)!=vb.get(x,y))throw new AssertionError("Random/Vertical mismatch");
   double mean=f.tile.getMean(x,y),sq=f.sqrTile.getMean(x,y),sd=Math.sqrt(Math.abs(sq-mean*mean));
   System.out.println("P "+x+" "+y+" "+b.get(x,y)+" "+mean+" "+sd+" "+(0.7*mean+0.9*sd));
  }
  for(Orientation o:Orientation.values()) {
   new RunsRetriever(o,new RunsRetriever.RunAdapter(){
    public boolean isThreadSafe(){return false;}
    public boolean isFore(int c,int pos){return (o==Orientation.HORIZONTAL?b.get(c,pos):b.get(pos,c))==0;}
    public boolean foreRun(int c,int pos,int n){return true;}
    public void endPosition(int pos,List<Run> runs){for(Run r:runs)System.out.println("R "+o+" "+pos+" "+r.start()+" "+r.length());}
   }).retrieveRuns(new Rectangle(0,0,w,h));
  }
 }
}''')
jdk=Path(sys.argv[1]).resolve()
sources=list(WORK.rglob('*.java'))
subprocess.run([str(jdk/'bin/javac'),'-d',str(WORK/'classes')]+[str(s) for s in sources],check=True)
summary={'upstream_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','oracle':'unmodified AdaptiveFilter, RandomFilter, VerticalFilter, RunsRetriever; test-only infrastructure adapters','fixtures':[],'source_sha256':{r:hashlib.sha256((BASE/('org/audiveris/omr/'+r+'.java')).read_bytes()).hexdigest() for r in originals}}
for fixture in sorted((ROOT/'fixtures').glob('*.pgm')):
    raw=fixture.read_bytes(); magic,dim,maxval,pixels=raw.split(b'\n',3); w,h=map(int,dim.split()); assert magic==b'P5' and maxval==b'255'
    out=WORK/(fixture.stem+'.pgm'); report=WORK/(fixture.stem+'.json')
    for f in [out,report]: f.unlink(missing_ok=True)
    subprocess.run([str(ROOT/'target/debug/audiveris-native'),'binarize',str(fixture),str(out),str(report)],check=True)
    rust=json.loads(report.read_text()); binary=out.read_bytes().split(b'\n',3)[3]
    oracle=subprocess.check_output([str(jdk/'bin/java'),'-Djava.awt.headless=true','-cp',str(WORK/'classes'),'org.audiveris.omr.image.Oracle'],input=(f'{w} {h}\n'+' '.join(map(str,pixels))).encode()).decode()
    java_runs={'HORIZONTAL':[[] for _ in range(h)],'VERTICAL':[[] for _ in range(w)]}; contexts={}; count=0
    for line in oracle.splitlines():
        t=line.split()
        if t[0]=='P':
            x,y,v=map(int,t[1:4]); assert binary[y*w+x]==v,(fixture,x,y,'binary mismatch'); contexts[x,y]=list(map(float,t[4:])); count+=1
        else:
            pos,start,length=map(int,t[2:]); java_runs[t[1]][pos].append([start,length])
    error=0.0
    for c in rust['probes']:
        for got,expected in zip([c['mean'],c['std_dev'],c['threshold']],contexts[c['x'],c['y']]):
            error=max(error,abs(got-expected)); assert math.isclose(got,expected,rel_tol=1e-13,abs_tol=1e-12)
    assert rust['horizontal_runs']==java_runs['HORIZONTAL']; assert rust['vertical_runs']==java_runs['VERTICAL']
    summary['fixtures'].append({'file':fixture.name,'pixels_compared':count,'binary_mismatches':0,'horizontal_runs_match':True,'vertical_runs_match':True,'max_probe_abs_error':error,'binary_sha256':hashlib.sha256(binary).hexdigest(),'input_sha256':hashlib.sha256(raw).hexdigest()})
(ROOT/'evidence/java-parity.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
