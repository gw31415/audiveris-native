#!/usr/bin/env python3
# Copyright © Audiveris 2026; test adaptation 2026. SPDX-License-Identifier: AGPL-3.0-or-later
import json,hashlib,subprocess,struct,sys,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];BASE=ROOT/'upstream/app/src/main/java/org/audiveris/omr';WORK=ROOT/'evidence/generated/java-curved';WORK.mkdir(parents=True,exist_ok=True);hashes={}
assert subprocess.check_output(['git','-C',str(ROOT/'upstream'),'rev-parse','HEAD'],text=True).strip()=='9e1e55cd2746037d059345881c53e6a6754bffbd'
def extract(text,sig):
 a=text.index(sig);b=text.index('{',a);depth=1;e=b+1
 while depth:depth+=(text[e]=='{')-(text[e]=='}');e+=1
 return text[a:e]
def method(file,sig):
 t=extract((BASE/(file+'.java')).read_text(),sig);hashes[file+'/'+sig]=hashlib.sha256(t.encode()).hexdigest();return t
base=(ROOT/'evidence/generated/java-filaments/FilamentOracle.java').read_text()
# Reuse unchanged BasicLine numerical-method/Section adapters from the previous harness.
line=extract(base,'static class Line ');section=extract(base,'static class Section ')
section=section.replace('int id,firstPos,weight;','static Comparator<Section>byPosition=Comparator.comparingInt(s->s.firstPos);int id,firstPos,weight;')
section=section.replace('boolean rectangle()', '''boolean touches(Section t){int y=firstPos;for(int[]r:runs){int ty=t.firstPos;for(int[]q:t.runs){if(GeoUtil.touch(new Rectangle(r[0],y,r[1],1),new Rectangle(q[0],ty,q[1],1)))return true;ty++;}y++;}return false;}
boolean rectangle()''')
section=section.replace('pc.size++;','pc.add(x,y);')
factory_methods='\n'.join(method('glyph/dynamic/FilamentFactory',s) for s in ['private void createInitialFilaments','public boolean isSectionFat','private boolean canMerge','private boolean contact','private double maxConsistentThickness','private void mergeFilaments','private List<F> expandFilaments','private void removeMergedFilaments'])
comb_helpers='\n'.join(extract(base,sig) for sig in ['static class FilamentComb','static class FilY','static class Range','static class Clusters'])
geo='static class GeoUtil{'+method('math/GeoUtil','public static boolean touch (Rectangle r1,')+'}'
compute=method('glyph/dynamic/CurvedFilament','public void computeLine ()');position=method('glyph/dynamic/CurvedFilament','public double getPositionAt')
line_methods='\n'.join(method('sheet/grid/LinesRetriever',sig) for sig in ['private void purgeCurvedFilaments ()','private double retrieveGlobalSlope ()','private List<StaffFilament> purgeSlopedFilaments ()'])
rotation='\n'.join(method('math/LineUtil',sig) for sig in ['public static double rotation (Line2D line,','public static double rotation (Point2D first,'])
compthick=method('glyph/dynamic/Compounds','public static double getThicknessAt (double coord,')
source='''// Copyright © Audiveris 2026. Verbatim methods and test adapters; AGPL-3.0-or-later. See LICENSE.
import java.awt.Rectangle;import java.awt.geom.Point2D;import java.awt.geom.Line2D;import java.io.*;import java.util.*;import static java.lang.Math.*;import org.audiveris.omr.math.NaturalSpline;
public class CurvedOracle {
static class Log{void trace(Object...o){}void debug(Object...o){}void info(Object...o){}void warn(Object...o){}boolean isDebugEnabled(){return false;}}static Log logger=new Log();
static class Orientation{static final Orientation HORIZONTAL=new Orientation();Rectangle oriented(Rectangle r){return r;}Rectangle absolute(Rectangle r){return r;}Point2D oriented(Point2D p){return p;}Point2D absolute(Point2D p){return p;}Line switchRef(Line l){return l;}}
static final Orientation HORIZONTAL=Orientation.HORIZONTAL,orientation=HORIZONTAL;
static class UndefinedLineException extends RuntimeException{UndefinedLineException(String s){super(s);}}static class NonInvertibleLineException extends RuntimeException{NonInvertibleLineException(String s){super(s);}}
'''+line+section+'''
static class PointsCollector{Rectangle roi;int size;List<Integer>xs=new ArrayList<>(),ys=new ArrayList<>();PointsCollector(Rectangle r){roi=new Rectangle(r);}void add(int x,int y){size++;xs.add(x);ys.add(y);}int getSize(){return size;}int[]getYValues(){return ys.stream().mapToInt(i->i).toArray();}int[]getXValues(){return xs.stream().mapToInt(i->i).toArray();}}
static class Scale{int il;Scale(int v){il=v;}int toPixels(double v){return (int)rint(il*v);}}
static class InterlineScale{static int toPixels(int il,double v){return(int)rint(il*v);}}
static String phase="merge";static List<String>events=new ArrayList<>();
static class SectionCompound{List<Section>members=new ArrayList<>();SectionCompound partOf;int id;boolean isVip(){return false;}void setVip(boolean b){}int getId(){return id;}List<Section>getMembers(){return members;}Section getFirstSection(){return members.get(0);}SectionCompound getPartOf(){return partOf;}Rectangle getBounds(){Rectangle b=null;for(Section s:members)if(b==null)b=s.getBounds();else b.add(s.getBounds());return b;}int getLength(Orientation o){return getBounds().width;}int getWeight(){return members.stream().mapToInt(s->s.weight).sum();}
void stealSections(SectionCompound that){events.add("{\\"phase\\":\\""+phase+"\\",\\"head\\":"+id+",\\"absorbed\\":"+that.id+"}");for(Section s:that.members)if(!members.contains(s))members.add(s);sortMembers();that.partOf=this;((Filament)this).invalidate();}
void sortMembers(){members.sort(Comparator.comparingInt((Section s)->s.bounds.x).thenComparingInt(s->s.bounds.y).thenComparingInt(s->s.id));}
Point2D getCentroid(Rectangle roi){return getCentroid(roi,1);}Point2D getCentroid(Rectangle roi,int min){double sx=0,sy=0;int n=0;for(Section s:members){int y=s.firstPos;for(int[]r:s.runs){for(int x=r[0];x<r[0]+r[1];x++)if(roi.contains(x,y)){sx+=x;sy+=y;n++;}y++;}}return n>=min?new Point2D.Double(sx/n,sy/n):null;}}
static class Filament extends SectionCompound{int interline,segmentLength;Point2D startPoint,stopPoint;List<Point2D>points;NaturalSpline spline;Filament(int il,Section s,int i){interline=il;segmentLength=il*4;id=i;members.add(s);}static double getProbeWidth(){return .5;}static double getProbeMinWeight(){return .2;}Orientation getRoughOrientation(){return HORIZONTAL;}void invalidate(){startPoint=stopPoint=null;points=null;spline=null;}NaturalSpline getSpline(){if(spline==null)computeLine();return spline;}Point2D getStartPoint(){if(spline==null)computeLine();return startPoint;}Point2D getStopPoint(){if(spline==null)computeLine();return stopPoint;}
'''+compute+'\n'+position+'''
}static class StaffFilament extends Filament{StaffFilament(int il,Section s,int i){super(il,s,i);}}
static class Compounds{static Comparator<SectionCompound>byReverseLength(Orientation o){return (a,b)->Integer.compare(b.getLength(o),a.getLength(o));}
'''+compthick+'''
static double getThicknessAt(double x,Orientation o,Scale sc,SectionCompound...f){return getThicknessAt(x,o,sc,null,f);}}
static class LineUtil{
'''+rotation+'''
static double getSlope(Point2D a,Point2D b){return(b.getY()-a.getY())/(b.getX()-a.getX());}}
static class Entities{static String ids(Object o){return "";}}
static class Ratio{double getValue(){return 1.7;}}static class Constants{Ratio maxConsistentRatio=new Ratio();}static Constants constants=new Constants();
static class Index{boolean isVipId(int i){return false;}}
static class Params{int samplingDx,combMinMargin,combMaxMargin;int minCoreSectionLength,maxThickness,probeWidth,maxCoordGap,maxPosGap,maxOverlapDeltaPos,maxOverlapSpace,maxExpansionSpace,maxInvolvingLength,maxPosGapForSlope,minLengthForDeltaSlope;double minSectionAspect=3,maxGapSlope=.5,maxDeltaSlope=.01;}
static class Factory<F extends Filament>{Params params;Scale scale;Index index=new Index();Map<Section,Boolean>fatSections=new HashMap<>();Set<Section>processed=new HashSet<>();int next;Factory(Params p,Scale sc){params=p;scale=sc;}boolean setFat(Section s,boolean b){fatSections.put(s,b);return b;}boolean isProcessed(Section s){return processed.contains(s);}@SuppressWarnings("unchecked")F createFilament(Section s){processed.add(s);return(F)new StaffFilament(scale.il,s,next++);}
'''+factory_methods+'''
}
'''+geo+comb_helpers+'''
static class StepException extends Exception{}static final String LINE_SEPARATOR="\\n";
static class Sheet{double slope;boolean tooFew;Sheet getSkew(){return this;}double getSlope(){return slope;}Sheet getStub(){return this;}String getId(){return "oracle";}void decideOnRemoval(String s,boolean b){tooFew=true;}}
static class LineParams{double maxFilamentRotation=.1,topRatioForSlope=.1,minSlope=.0002,maxSlopeDiff=.025;int minLengthForSlopeCheck;}
static class Lines{List<StaffFilament>filaments;LineParams params=new LineParams();Sheet sheet=new Sheet();
'''+line_methods+'''
}

public static void main(String[]args)throws Exception{DataInputStream in=new DataInputStream(System.in);int width=in.readInt(),ilmin=in.readInt(),il=in.readInt(),ilmax=in.readInt(),height=in.readInt(),n=in.readInt();Scale sc=new Scale(il);Params p=new Params();p.samplingDx=il;p.combMinMargin=p.combMaxMargin=sc.toPixels(.05);p.minCoreSectionLength=sc.toPixels(.5);p.maxThickness=(int)rint(height*1.5);p.probeWidth=sc.toPixels(.5);p.maxCoordGap=sc.toPixels(1.7);p.maxPosGap=(int)rint(height*.75);p.maxOverlapDeltaPos=sc.toPixels(.25);p.maxOverlapSpace=sc.toPixels(.16);p.maxExpansionSpace=sc.toPixels(.02);p.maxInvolvingLength=sc.toPixels(2);p.maxPosGapForSlope=sc.toPixels(.1);p.minLengthForDeltaSlope=sc.toPixels(10);List<Section>sections=new ArrayList<>();for(int i=0;i<n;i++){Section s=new Section();s.id=in.readInt();s.firstPos=in.readInt();int nr=in.readInt();s.runs=new int[nr][2];int lo=Integer.MAX_VALUE,hi=0;for(int j=0;j<nr;j++){int a=in.readInt(),b=in.readInt();s.runs[j]=new int[]{a,b};s.weight+=b;lo=min(lo,a);hi=max(hi,a+b);}s.bounds=new Rectangle(lo,s.firstPos,hi-lo,nr);sections.add(s);}Factory<StaffFilament>factory=new Factory<>(p,sc);List<StaffFilament>fs=new ArrayList<>();factory.createInitialFilaments(fs,sections);factory.mergeFilaments(fs);phase="expand";factory.expandFilaments(fs,sections);phase="merge";factory.mergeFilaments(fs);List<String>filaments=new ArrayList<>();for(StaffFilament f:fs){f.getStartPoint();List<String>pts=new ArrayList<>(),samples=new ArrayList<>();for(Point2D q:f.points)pts.add("["+q.getX()+","+q.getY()+"]");Rectangle b=f.getBounds();for(int j=0;j<=8;j++){double x=b.x+(b.width-1)*j/8.;samples.add("["+x+","+f.getPositionAt(x,HORIZONTAL)+"]");}filaments.add("{\\"id\\":"+f.id+",\\"section_ids\\":"+f.members.stream().map(s->s.id).toList()+",\\"bounds\\":["+b.x+","+b.y+","+b.width+","+b.height+"],\\"points\\":["+String.join(",",pts)+"],\\"sample_y\\":["+String.join(",",samples)+"]}");}Lines lines=new Lines();lines.filaments=new ArrayList<>(fs);lines.params.minLengthForSlopeCheck=il*4;lines.purgeCurvedFilaments();List<Integer>curvedRemoved=new ArrayList<>();for(StaffFilament f:fs)if(!lines.filaments.contains(f))curvedRemoved.add(f.id);Double slope=lines.filaments.isEmpty()?null:lines.retrieveGlobalSlope();if(slope!=null)lines.sheet.slope=slope;List<StaffFilament>sloped=lines.filaments.isEmpty()?new ArrayList<>():lines.purgeSlopedFilaments();String filtering="{\\"curved_removed\\":"+curvedRemoved+",\\"sloped_removed\\":"+sloped.stream().map(f->f.id).toList()+",\\"global_slope\\":"+slope+",\\"too_few_after_curvature\\":"+lines.sheet.tooFew+",\\"retained_ids\\":"+lines.filaments.stream().map(f->f.id).toList()+"}";Clusters c=new Clusters();c.pictureWidth=width;c.params=p;c.interlineScale=new Range(ilmin,ilmax);c.allFilaments=lines.filaments;c.retrieveCombs();List<String>cs=new ArrayList<>();for(var entry:c.colCombs.entrySet())for(FilamentComb comb:entry.getValue())cs.add("{\\"column\\":"+comb.col+",\\"x\\":"+c.colX[comb.col]+",\\"filament_ids\\":"+comb.fs.stream().map(f->f.id).toList()+",\\"ys\\":"+comb.ys+"}");System.out.println("{\\"filaments\\":["+String.join(",",filaments)+"],\\"events\\":["+String.join(",",events)+"],\\"combs\\":["+String.join(",",cs)+"],\\"line_filtering\\":"+filtering+"}");}
}
'''
(WORK/'CurvedOracle.java').write_text(source)
for n in ['NaturalSpline','GeoPath']:
 q=WORK/f'org/audiveris/omr/math/{n}.java';q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(BASE/f'math/{n}.java',q);hashes['full_source/'+n]=hashlib.sha256(q.read_bytes()).hexdigest()
p=WORK/'org/slf4j';p.mkdir(parents=True,exist_ok=True);(p/'Logger.java').write_text('package org.slf4j;public class Logger{public void info(Object...o){}}');(p/'LoggerFactory.java').write_text('package org.slf4j;public class LoggerFactory{public static Logger getLogger(Class<?>c){return new Logger();}}')
jdk=Path(sys.argv[1]);subprocess.run([str(jdk/'bin/javac'),'-d',str(WORK/'classes')]+[str(p)for p in WORK.rglob('*.java')],check=True)
report={'upstream_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','scope':'unmodified NaturalSpline/GeoPath; verbatim CurvedFilament compute/position and FilamentFactory selection/merge/expand/canMerge; explicit horizontal compound/data adapters; including curvature/slope filters, explicit empty-list removal boundary; before formal Staff network','method_sha256':hashes,'fixtures':[]};goldens=[]
for fixture in sorted((ROOT/'fixtures/sections').glob('*.pgm'))+list((ROOT/'fixtures/rendered').glob('*.pgm')):
 out=WORK/(fixture.stem+'.pgm');path=WORK/(fixture.stem+'.json')
 for q in[out,path]:q.unlink(missing_ok=True)
 r=subprocess.run([str(ROOT/'target/debug/audiveris-native'),'filaments',str(fixture),str(out),str(path)],capture_output=True,text=True);assert r.returncode in[0,2],r.stderr
 native=json.loads(path.read_text());scales=native['scale']['selected']
 if scales is None:continue
 ss=native['grid_preparation']['horizontal_sections'];il=scales['interline'];lh=scales['line'];vals=[native['width'],*il,lh[1],len(ss)];data=struct.pack('>'+'i'*len(vals),*vals)
 for s in ss:
  vals=[s['id'],s['first_pos'],len(s['runs'])]+[x for r in s['runs']for x in r];data+=struct.pack('>'+'i'*len(vals),*vals)
 java=json.loads(subprocess.check_output([str(jdk/'bin/java'),'-Xmx192m','-Djava.awt.headless=true','-cp',str(WORK/'classes'),'CurvedOracle'],input=data));got=native['filaments'];maximum=0.
 def compare(a,b,where=''):
  global maximum
  if isinstance(b,float):
   delta=abs(a-b);maximum=max(maximum,delta);assert delta<=1e-9+1e-12*abs(b),(where,a,b)
  elif isinstance(b,dict):
   for k,v in b.items():compare(a[k],v,where+'/'+k)
  elif isinstance(b,list):
   assert len(a)==len(b),(where,len(a),len(b))
   for i,(x,y)in enumerate(zip(a,b)):compare(x,y,where+'/'+str(i))
  else:assert a==b,(where,a,b)
 try:compare(got,java)
 except Exception:
  (WORK/'mismatch.json').write_text(json.dumps({'native':got,'java':java},indent=2));raise
 e={'file':str(fixture.relative_to(ROOT)),'filaments':len(java['filaments']),'merge_expand_events':len(java['events']),'combs':len(java['combs']),'retained_after_line_filters':len(java['line_filtering']['retained_ids']),'membership_event_order_exact':True,'max_geometry_error':maximum};print(e);report['fixtures'].append(e);goldens.append({'width':native['width'],'line':lh,'interline':il,'sections':ss,'java':java})
(ROOT/'evidence/curved-filament-parity.json').write_text(json.dumps(report,indent=2)+'\n');(ROOT/'fixtures/curved-filament-golden.json').write_text(json.dumps(goldens,separators=(',',':'))+'\n')
