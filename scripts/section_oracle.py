#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Original SectionFactory/JunctionPolicies plus verbatim transplanted LAG-dispatch methods.
Requires scale_oracle.py scaffold; never starts the full Audiveris GUI/application.
"""
import hashlib,json,shutil,struct,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];BASE=ROOT/'upstream/app/src/main/java';WORK=ROOT/'evidence/generated/java-sections';PIN='9e1e55cd2746037d059345881c53e6a6754bffbd'
assert subprocess.check_output(['git','-C',str(ROOT/'upstream'),'rev-parse','HEAD'],text=True).strip()==PIN
WORK.mkdir(parents=True,exist_ok=True)
for p in (ROOT/'evidence/generated/java-scale').rglob('*.java'):
 q=WORK/p.relative_to(ROOT/'evidence/generated/java-scale');q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q)
def write(n,t):
 p=WORK/(n+'.java');p.parent.mkdir(parents=True,exist_ok=True);p.write_text(t)
originals=['lag/SectionFactory','lag/JunctionPolicy','lag/JunctionRatioPolicy','lag/JunctionShiftPolicy']
for n in originals:
 p=BASE/('org/audiveris/omr/'+n+'.java');q=WORK/('org/audiveris/omr/'+n+'.java');q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q)
excerpts={}
def method(file,signature):
 s=(BASE/('org/audiveris/omr/'+file+'.java')).read_text();start=s.index(signature);pos=s.index('{',start);depth=1;end=pos+1
 while depth:
  if s[end]=='{':depth+=1
  elif s[end]=='}':depth-=1
  end+=1
 text=s[start:end];excerpts[file+'/'+signature]=hashlib.sha256(text.encode()).hexdigest();return text
stubs={
'org/audiveris/omr/run/Run':'''package org.audiveris.omr.run;public record Run(int start,int length){public Run(Run r){this(r.start,r.length);}public int getStart(){return start;}public int getLength(){return length;}public int getStop(){return start+length-1;}}''',
'org/audiveris/omr/run/RunTable':'''package org.audiveris.omr.run;import java.util.*;import java.util.function.Predicate;import java.awt.Rectangle;import ij.process.ByteProcessor;
public class RunTable {public final int w,h;public final Orientation orientation;public final List<List<Run>> rows=new ArrayList<>();
 public RunTable(Orientation o,int w,int h){this.w=w;this.h=h;orientation=o;for(int i=0;i<(o==Orientation.VERTICAL?w:h);i++)rows.add(new ArrayList<>());}
 public RunTable(ByteProcessor p){this(p,Orientation.VERTICAL);}public RunTable(ByteProcessor p,Orientation o){this(o,p.getWidth(),p.getHeight());new RunsRetriever(o,new RunsRetriever.RunAdapter(){public boolean isThreadSafe(){return false;}public boolean foreRun(int c,int p,int n){return true;}public boolean isFore(int c,int y){return (o==Orientation.VERTICAL?p.get(y,c):p.get(c,y))==0;}public void endPosition(int pos,List<Run> r){rows.set(pos,r);}}).retrieveRuns(new Rectangle(0,0,w,h));}
 public Orientation getOrientation(){return orientation;}public int getWidth(){return w;}public int getHeight(){return h;}public int getSize(){return rows.size();}public boolean isSequenceEmpty(int p){return rows.get(p).isEmpty();}public Iterator<Run> iterator(int p){return rows.get(p).iterator();}
 public RunTable copy(){RunTable t=new RunTable(orientation,w,h);for(int i=0;i<rows.size();i++)t.rows.get(i).addAll(rows.get(i));return t;}
 public RunTable purge(Predicate<Run> predicate,RunTable removed){for(int i=0;i<rows.size();i++){Iterator<Run> it=rows.get(i).iterator();while(it.hasNext()){Run r=it.next();if(predicate.test(r)){if(removed!=null)removed.rows.get(i).add(r);it.remove();}}}return this;}
 public ByteProcessor getBuffer(){ByteProcessor p=new ByteProcessor(w,h);for(int y=0;y<h;y++)for(int x=0;x<w;x++)p.set(x,y,255);for(int pos=0;pos<rows.size();pos++)for(Run r:rows.get(pos))for(int c=r.start();c<r.start()+r.length();c++){if(orientation==Orientation.VERTICAL)p.set(pos,c,0);else p.set(c,pos,0);}return p;}
}''',
'org/audiveris/omr/run/RunTableFactory':'''package org.audiveris.omr.run;import ij.process.ByteProcessor;import java.awt.Rectangle;public class RunTableFactory {final Orientation o;public RunTableFactory(Orientation o){this.o=o;}public RunTable createTable(ByteProcessor b){return new RunTable(b,o);}public RunTable createTable(ByteProcessor b,Rectangle r){throw new UnsupportedOperationException();}}''',
'org/audiveris/omr/lag/Section':'''package org.audiveris.omr.lag;import org.audiveris.omr.run.Run;public interface Section{Run getLastRun();int getMeanRunLength();}''',
'org/audiveris/omr/lag/DynamicSection':'''package org.audiveris.omr.lag;import org.audiveris.omr.run.*;import java.util.*;import java.awt.Point;
public class DynamicSection implements Section {public int id,firstPos,weight;public final Orientation orientation;public final List<Run> runs=new ArrayList<>();public DynamicSection(Orientation o){orientation=o;}public void setId(int i){id=i;}public void setFirstPos(int p){firstPos=p;}public void append(Run r){runs.add(new Run(r));weight+=r.getLength();}public Run getLastRun(){return runs.get(runs.size()-1);}public int getMeanRunLength(){return weight/runs.size();}public void translate(Point p){throw new UnsupportedOperationException();}}''',
'org/audiveris/omr/lag/BasicSection':'''package org.audiveris.omr.lag;import org.audiveris.omr.run.Run;public class BasicSection implements Section {final DynamicSection s;public BasicSection(DynamicSection s){this.s=s;}public Run getLastRun(){return s.getLastRun();}public int getMeanRunLength(){return s.getMeanRunLength();}}''',
'org/audiveris/omr/lag/Lag':'''package org.audiveris.omr.lag;import org.audiveris.omr.run.*;public interface Lag {Orientation getOrientation();void register(DynamicSection s);void addRunTable(RunTable t);}''',
'org/audiveris/omr/lag/BasicLag':'''package org.audiveris.omr.lag;import org.audiveris.omr.run.*;import java.util.*;public class BasicLag implements Lag {final Orientation o;public RunTable table;public final List<DynamicSection> sections=new ArrayList<>();public BasicLag(String n,Orientation o){this.o=o;}public Orientation getOrientation(){return o;}public void register(DynamicSection s){s.setId(sections.size()+1);sections.add(s);}public void addRunTable(RunTable t){table=t;}}''',
'org/audiveris/omr/lag/Lags':'''package org.audiveris.omr.lag;public class Lags {public static final String HLAG="h",VLAG="v";}''',
'org/audiveris/omr/OMR':'''package org.audiveris.omr;public class OMR {public static final Object gui=null;}''',
'org/audiveris/omr/sheet/grid/RunsViewer':'''package org.audiveris.omr.sheet.grid;public class RunsViewer {public RunsViewer(Object o){}public void display(Object... o){throw new UnsupportedOperationException();}}''',
'org/audiveris/omr/sheet/Sheet':'''package org.audiveris.omr.sheet;import org.audiveris.omr.run.RunTable;import org.audiveris.omr.lag.LagManager;public class Sheet {final RunTable r;Scale scale;final LagManager manager;public Sheet(RunTable r){this.r=r;manager=new LagManager(this);}public String getId(){return "oracle";}public SheetStub getStub(){return new SheetStub();}public Scale getScale(){return scale;}public void setScale(Scale s){scale=s;}public Picture getPicture(){return new Picture(r);}public int getWidth(){return r.getWidth();}public int getHeight(){return r.getHeight();}public LagManager getLagManager(){return manager;}}'''
}
for k,v in stubs.items():write(k,v)
p=WORK/'org/audiveris/omr/sheet/Scale.java';s=p.read_text().replace('public class Scale {','''public class Scale {public int getMaxFore(){return line.max;}public int toPixels(Fraction f){return (int)Math.rint(interline.main*f.value);}public static class Fraction {public final double value;public Fraction(double v,String s){value=v;}}''');p.write_text(s)
write('org/audiveris/omr/lag/LagManager','''package org.audiveris.omr.lag;import org.audiveris.omr.sheet.*;import org.audiveris.omr.run.*;import static org.audiveris.omr.run.Orientation.*;import org.audiveris.omr.constant.Constant;import java.util.*;
public class LagManager {final Sheet sheet;public final Map<String,Lag> lags=new HashMap<>();public LagManager(Sheet s){sheet=s;}void setLag(String s,Lag l){lags.put(s,l);}void setVipSections(Orientation o){}static class Constants{final Constant.Ratio ledgerThickness=new Constant.Ratio(1.2,"");final Scale.Fraction maxVerticalRunShift=new Scale.Fraction(.05,"");}static final Constants constants=new Constants();
'''+method('lag/LagManager','public RunTable dispatchRuns')+'\n'+method('lag/LagManager','public Lag buildHorizontalLag')+'\n'+method('lag/LagManager','public Lag buildVerticalLag')+'\n}')
write('org/audiveris/omr/sheet/grid/LinesRetriever','''package org.audiveris.omr.sheet.grid;import org.audiveris.omr.sheet.*;import org.audiveris.omr.lag.*;import org.audiveris.omr.run.*;import org.audiveris.omr.OMR;import static org.audiveris.omr.run.Orientation.*;
public class LinesRetriever {final Sheet sheet;public RunTable shortHoriTable;public Lag hLag;static class Flag{boolean isSet(){return false;}}static class Constants{final Flag displayRuns=new Flag();}static final Constants constants=new Constants();static class Parameters{int minRunLength;}final Parameters params=new Parameters();public LinesRetriever(Sheet s){sheet=s;params.minRunLength=s.getScale().toPixels(new Scale.Fraction(.25,""));}
'''+method('sheet/grid/LinesRetriever','public void createBothLags')+'\n}')
write('SectionOracle','''import java.io.*;import java.util.*;import org.audiveris.omr.run.*;import org.audiveris.omr.sheet.*;import org.audiveris.omr.sheet.grid.*;import org.audiveris.omr.lag.*;import org.audiveris.omr.image.VerticalFilter;import ij.process.ByteProcessor;
public class SectionOracle{
static String sections(List<DynamicSection> ss){List<String> a=new ArrayList<>();for(DynamicSection s:ss){int lo=Integer.MAX_VALUE,hi=0;List<String> runs=new ArrayList<>();for(Run r:s.runs){lo=Math.min(lo,r.start());hi=Math.max(hi,r.start()+r.length());runs.add("["+r.start()+","+r.length()+"]");}boolean h=s.orientation==Orientation.HORIZONTAL;String bounds=h?lo+","+s.firstPos+","+(hi-lo)+","+s.runs.size():s.firstPos+","+lo+","+s.runs.size()+","+(hi-lo);a.add("{\\"id\\":"+s.id+",\\"first_pos\\":"+s.firstPos+",\\"weight\\":"+s.weight+",\\"mean_run_length\\":"+s.getMeanRunLength()+",\\"bounds\\":["+bounds+"],\\"runs\\":["+String.join(",",runs)+"]}");}return "["+String.join(",",a)+"]";}
static int count(RunTable t){return t.rows.stream().mapToInt(List::size).sum();}
public static void main(String[] args)throws Exception{DataInputStream in=new DataInputStream(System.in);int w=in.readInt(),h=in.readInt();ByteProcessor p=new ByteProcessor(w,h);for(int y=0;y<h;y++)for(int x=0;x<w;x++)p.set(x,y,in.readUnsignedByte());ByteProcessor b=new VerticalFilter(p,.7,.9).filteredImage();Sheet sheet=new Sheet(new RunTable(b));try{sheet.setScale(new ScaleBuilder(sheet).retrieveScale());}catch(org.audiveris.omr.step.StepException e){System.out.println("{\\"status\\":\\"blocked\\",\\"error\\":\\"scale_failed\\"}");return;}LinesRetriever lr=new LinesRetriever(sheet);lr.createBothLags();BasicLag hLag=(BasicLag)lr.hLag,vLag=(BasicLag)sheet.getLagManager().lags.get(Lags.VLAG);Scale sc=sheet.getScale();System.out.println("{\\"stage\\":\\"lag_sections\\",\\"staff_detection\\":\\"not_implemented\\",\\"min_vertical_length\\":"+(1+(int)Math.rint(sc.line.max*1.2))+",\\"min_horizontal_length\\":"+(int)Math.rint(sc.interline.main*.25)+",\\"max_vertical_shift\\":"+(int)Math.rint(sc.interline.main*.05)+",\\"short_horizontal_runs\\":"+count(lr.shortHoriTable)+",\\"long_vertical_runs\\":"+count(vLag.table)+",\\"long_horizontal_runs\\":"+count(hLag.table)+",\\"horizontal_sections\\":"+sections(hLag.sections)+",\\"vertical_sections\\":"+sections(vLag.sections)+"}");}}
''')
jdk=Path(sys.argv[1]).resolve();subprocess.run([str(jdk/'bin/javac'),'-d',str(WORK/'classes')]+[str(p) for p in WORK.rglob('*.java')],check=True)
summary={'upstream_commit':PIN,'oracle':'unmodified SectionFactory/JunctionRatioPolicy/JunctionShiftPolicy; verbatim transplanted LagManager.dispatchRuns/buildHorizontalLag/buildVerticalLag and LinesRetriever.createBothLags; infrastructure/data adapters','source_sha256':{n:hashlib.sha256((BASE/('org/audiveris/omr/'+n+'.java')).read_bytes()).hexdigest() for n in originals},'verbatim_method_sha256':excerpts,'fixtures':[]}
fixtures=sorted((ROOT/'fixtures/sections').glob('*.pgm'))+sorted((ROOT/'fixtures/rendered').glob('*.pgm'))
for fixture in fixtures:
 raw=fixture.read_bytes();magic,dim,maxval,pixels=raw.split(b'\n',3);w,h=map(int,dim.split());assert magic==b'P5' and maxval==b'255'
 out=WORK/(fixture.stem+'.pgm');report=WORK/(fixture.stem+'.json')
 for p in [out,report]:p.unlink(missing_ok=True)
 r=subprocess.run([str(ROOT/'target/debug/audiveris-native'),'grid-prep',str(fixture),str(out),str(report)],capture_output=True,text=True)
 assert r.returncode in [0,2],(fixture,r.stderr)
 native=json.loads(report.read_text())['grid_preparation']
 java=json.loads(subprocess.check_output([str(jdk/'bin/java'),'-Xmx256m','-Djava.awt.headless=true','-cp',str(WORK/'classes'),'SectionOracle'],input=struct.pack('>ii',w,h)+pixels))
 if native!=java:
  (WORK/'mismatch.json').write_text(json.dumps({'native':native,'java':java},indent=2));raise SystemExit('MISMATCH '+str(fixture))
 entry={'file':str(fixture.relative_to(ROOT)),'input_sha256':hashlib.sha256(raw).hexdigest(),'width':w,'height':h,'exit':r.returncode,'exact_grid_json_match':True,'horizontal_sections':len(java.get('horizontal_sections',[])),'vertical_sections':len(java.get('vertical_sections',[])),'scale_blocked':java.get('status')=='blocked','result_sha256':hashlib.sha256(json.dumps(java,sort_keys=True).encode()).hexdigest()}
 summary['fixtures'].append(entry);print(entry)
(ROOT/'evidence/section-parity.json').write_text(json.dumps(summary,indent=2)+'\n')
