#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Unmodified Java ScaleBuilder + HiLoPeakFinder/IntegerFunction; infrastructure-only stubs.
Requires existing java_oracle.py scaffold (run that script once first), and a supplied JDK.
"""
import hashlib,json,shutil,struct,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];PIN='9e1e55cd2746037d059345881c53e6a6754bffbd'
assert subprocess.check_output(['git','-C',str(ROOT/'upstream'),'rev-parse','HEAD'],text=True).strip()==PIN
BASE=ROOT/'upstream/app/src/main/java';WORK=ROOT/'evidence/generated/java-scale'
WORK.mkdir(parents=True,exist_ok=True)
for p in (ROOT/'evidence/generated/java-oracle').rglob('*.java'):
    q=WORK/p.relative_to(ROOT/'evidence/generated/java-oracle');q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q)
def write(name,text):
    p=WORK/(name+'.java');p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text)
originals=['sheet/ScaleBuilder','math/HiLoPeakFinder','math/IntegerFunction','math/Range']
for r in originals:
    p=BASE/('org/audiveris/omr/'+r+'.java');q=WORK/('org/audiveris/omr/'+r+'.java');q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q)
stubs={
'org/slf4j/Logger':'''package org.slf4j; public interface Logger { default void warn(String s,Object... v){} default void error(String s,Object... v){} default void debug(String s,Object... v){} default void info(String s,Object... v){} default boolean isDebugEnabled(){return false;} }''',
'org/audiveris/omr/constant/Constant':'''package org.audiveris.omr.constant; public class Constant { public static class Integer {private final int v;public Integer(String u,int v,String d){this.v=v;}public int getValue(){return v;}} public static class Ratio {private final double v;public Ratio(double v,String d){this.v=v;}public double getValue(){return v;}} }''',
'org/audiveris/omr/util/Navigable':'''package org.audiveris.omr.util; public @interface Navigable {boolean value();}''',
'org/audiveris/omr/util/ChartPlotter':'''package org.audiveris.omr.util; public class ChartPlotter { public ChartPlotter(String... x){} public void add(Object... x){throw new UnsupportedOperationException();}public void display(Object... x){throw new UnsupportedOperationException();}}''',
'org/audiveris/omr/ui/Colors':'''package org.audiveris.omr.ui; public class Colors {public static final Object CHART_PEAK=null, CHART_HILO=null, CHART_VALUE=null, CHART_DERIVATIVE=null, CHART_ZERO=null;}''',
'org/jfree/data/xy/XYSeries':'''package org.jfree.data.xy; public class XYSeries {public XYSeries(Object... x){throw new UnsupportedOperationException();}public void add(Object... x){throw new UnsupportedOperationException();}}''',
'javax/xml/bind/annotation/XmlAccessType':'''package javax.xml.bind.annotation; public enum XmlAccessType {NONE;}''',
'javax/xml/bind/annotation/XmlAccessorType':'''package javax.xml.bind.annotation; public @interface XmlAccessorType {XmlAccessType value();}''',
'javax/xml/bind/annotation/XmlAttribute':'''package javax.xml.bind.annotation; public @interface XmlAttribute {String name();}''',
'javax/xml/bind/annotation/XmlRootElement':'''package javax.xml.bind.annotation; public @interface XmlRootElement {String name();}''',
'org/audiveris/omr/step/StepException':'''package org.audiveris.omr.step; public class StepException extends Exception {public StepException(String s){super(s);}}''',
'org/audiveris/omr/run/Run':'''package org.audiveris.omr.run; public record Run(int start,int length) {public int getStart(){return start;}public int getLength(){return length;}}''',
'org/audiveris/omr/run/RunTable':'''package org.audiveris.omr.run; import java.util.*; import java.awt.Rectangle;import ij.process.ByteProcessor;
public class RunTable {final int w,h;final List<List<Run>> rows=new ArrayList<>();
 public RunTable(ByteProcessor b){w=b.getWidth();h=b.getHeight();new RunsRetriever(Orientation.VERTICAL,new RunsRetriever.RunAdapter(){public boolean isThreadSafe(){return false;}public boolean foreRun(int c,int p,int n){return true;}public boolean isFore(int c,int p){return b.get(p,c)==0;}public void endPosition(int p,List<Run> r){rows.add(r);}}).retrieveRuns(new Rectangle(0,0,w,h));}
 public int getWidth(){return w;}public int getHeight(){return h;}public Iterator<Run> iterator(int x){return rows.get(x).iterator();}}
''',
'org/audiveris/omr/sheet/Picture':'''package org.audiveris.omr.sheet;import org.audiveris.omr.run.RunTable; public class Picture {public enum TableKey {BINARY;}final RunTable r;public Picture(RunTable r){this.r=r;}public RunTable getVerticalTable(TableKey k){return r;}}''',
'org/audiveris/omr/sheet/SheetStub':'''package org.audiveris.omr.sheet;import org.audiveris.omr.step.StepException;public class SheetStub {public Integer getInterlineSpecification(){return null;}public Integer getBeamSpecification(){return null;}public void invalidate(){}public void decideOnRemoval(String msg,boolean b)throws StepException{throw new StepException(msg);}}''',
'org/audiveris/omr/sheet/Sheet':'''package org.audiveris.omr.sheet;import org.audiveris.omr.run.RunTable;public class Sheet {final RunTable r;public Sheet(RunTable r){this.r=r;}public String getId(){return "oracle";}public SheetStub getStub(){return new SheetStub();}public Scale getScale(){return null;}public Picture getPicture(){return new Picture(r);}public int getWidth(){return r.getWidth();}}''',
'org/audiveris/omr/sheet/Scale':'''package org.audiveris.omr.sheet;import org.audiveris.omr.math.Range;
public class Scale {public enum Info{BLACK,COMBO;}public final InterlineScale interline,smallInterline;public final LineScale line;public final BeamScale beam,smallBeam;
public Scale(InterlineScale i,LineScale l,BeamScale b,InterlineScale si,BeamScale sb){interline=i;line=l;beam=b;smallInterline=si;smallBeam=sb;}public String toString(Info i){return "oracle";}
public static class LineScale extends Range {public LineScale(Range r){super(r.min,r.main,r.max);}}
public static class InterlineScale extends Range {public InterlineScale(Range r){super(r.min,r.main,r.max);}public InterlineScale(int a,int b,int c){super(a,b,c);}}
public static class BeamScale {public final int thickness;public final boolean extrapolated;public BeamScale(int t,boolean e){thickness=t;extrapolated=e;}}}
'''
}
for k,v in stubs.items():write(k,v)
write('ScaleOracle','''import java.io.*;import java.lang.reflect.*;import java.util.*;
import org.audiveris.omr.sheet.*;import org.audiveris.omr.math.*;import org.audiveris.omr.run.*;import org.audiveris.omr.image.VerticalFilter;import ij.process.ByteProcessor;
public class ScaleOracle {
 static Object field(Object o,String name)throws Exception{Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(o);}
 static String range(Range r){return r==null?"null":"["+r.min+","+r.main+","+r.max+"]";}
 static String beam(Scale.BeamScale b){return b==null?"null":"{\\"thickness\\":"+b.thickness+",\\"extrapolated\\":"+b.extrapolated+"}";}
 static String peaks(HiLoPeakFinder f){List<Range> p=f.getPeaks();if(p==null)return "[]";return "["+String.join(",",p.stream().map(ScaleOracle::range).toList())+"]";}
 static String hist(IntegerFunction f){List<String> a=new ArrayList<>();for(int x=f.getXMin();x<=f.getXMax();x++)a.add(""+f.getValue(x));return "["+String.join(",",a)+"]";}
 public static void main(String[] args)throws Exception{
  DataInputStream in=new DataInputStream(System.in);int w=in.readInt(),h=in.readInt();ByteProcessor p=new ByteProcessor(w,h);
  for(int y=0;y<h;y++)for(int x=0;x<w;x++)p.set(x,y,in.readUnsignedByte());
  ByteProcessor binary=new VerticalFilter(p,0.7,0.9).filteredImage();
  java.security.MessageDigest digest=java.security.MessageDigest.getInstance("SHA-256");
  for(int y=0;y<h;y++)for(int x=0;x<w;x++)digest.update((byte)binary.get(x,y));
  String binaryHash=java.util.HexFormat.of().formatHex(digest.digest());
  ScaleBuilder builder=new ScaleBuilder(new Sheet(new RunTable(binary)));Scale s=null;String error=null;
  try{s=builder.retrieveScale();}catch(org.audiveris.omr.step.StepException e){String m=e.getMessage();error=m.contains("No regularly")?"no_regular_lines":m.contains("Too few black")?"too_few_black_pixels":m.contains("No significant")?"no_significant_black_lines":m.contains("too low")?"interline_too_low":m.contains("too high")?"interline_too_high":m;}
  Object keeper=field(builder,"histoKeeper");IntegerFunction black=(IntegerFunction)field(keeper,"blackFunction"),combo=(IntegerFunction)field(keeper,"comboFunction");
  String selected=s==null?"null":"{\\"line\\":"+range(s.line)+",\\"interline\\":"+range(s.interline)+",\\"small_interline\\":"+range(s.smallInterline)+",\\"beam\\":"+beam(s.beam)+",\\"small_beam\\":"+beam(s.smallBeam)+"}";
  System.out.println("{\\"status\\":\\""+(error==null?"ok":"failed")+"\\",\\"error\\":"+(error==null?"null":"\\""+error+"\\"")+",\\"max_black\\":"+field(keeper,"maxBlack")+",\\"max_white\\":"+field(keeper,"maxWhite")+",\\"black_histogram\\":"+hist(black)+",\\"combo_histogram\\":"+hist(combo)+",\\"black_peaks\\":"+peaks((HiLoPeakFinder)field(keeper,"blackFinder"))+",\\"combo_peaks\\":"+peaks((HiLoPeakFinder)field(keeper,"comboFinder"))+",\\"selected\\":"+selected+",\\\"binary_sha256\\\":\\\""+binaryHash+"\\\"}");
 }
}''')
jdk=Path(sys.argv[1]).resolve();subprocess.run([str(jdk/'bin/javac'),'-d',str(WORK/'classes')]+[str(s) for s in WORK.rglob('*.java')],check=True)
summary={'upstream_commit':PIN,'oracle':'unmodified Java ScaleBuilder, HiLoPeakFinder, IntegerFunction, VerticalFilter, RunsRetriever; default constants, no user overrides; sheet/plotting/data infrastructure stubbed','source_sha256':{r:hashlib.sha256((BASE/('org/audiveris/omr/'+r+'.java')).read_bytes()).hexdigest() for r in originals},'fixtures':[]}
for fixture in sorted((ROOT/'fixtures/scale').glob('*.pgm')):
 raw=fixture.read_bytes();magic,dim,maxval,pixels=raw.split(b'\n',3);w,h=map(int,dim.split());assert magic==b'P5' and maxval==b'255'
 out=WORK/(fixture.stem+'.pgm');report=WORK/(fixture.stem+'.json')
 for p in [out,report]:p.unlink(missing_ok=True)
 native=subprocess.run([str(ROOT/'target/debug/audiveris-native'),'scale',str(fixture),str(out),str(report)],capture_output=True,text=True)
 rust=json.loads(report.read_text())['scale']
 java=json.loads(subprocess.check_output([str(jdk/'bin/java'),'-Xmx192m','-Djava.awt.headless=true','-cp',str(WORK/'classes'),'ScaleOracle'],input=struct.pack('>ii',w,h)+pixels))
 java_binary_hash=java.pop('binary_sha256')
 native_binary_hash=hashlib.sha256(out.read_bytes().split(b'\n',3)[3]).hexdigest()
 assert native_binary_hash==java_binary_hash,(fixture.name,'binary pixel mismatch')
 if rust!=java:
  print('MISMATCH',fixture.name)
  for k in rust:
   if rust[k]!=java[k]: print(k,'native=',rust[k],'java=',java[k])
  raise SystemExit(1)
 assert native.returncode==(0 if java['status']=='ok' else 2)
 summary['fixtures'].append({'file':fixture.name,'sha256':hashlib.sha256(raw).hexdigest(),'exact_scale_json_match':True,'binary_sha256':java_binary_hash,'binary_pixels_match':True,'native_exit':native.returncode,'status':java['status'],'error':java['error'],'selected':java['selected'],'black_histogram_sha256':hashlib.sha256(json.dumps(java['black_histogram']).encode()).hexdigest(),'combo_histogram_sha256':hashlib.sha256(json.dumps(java['combo_histogram']).encode()).hexdigest()})
 print(fixture.name,java['status'],java['error'],java['selected'])
(ROOT/'evidence/scale-parity.json').write_text(json.dumps(summary,indent=2)+'\n')
