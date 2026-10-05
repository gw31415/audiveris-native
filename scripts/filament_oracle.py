#!/usr/bin/env python3
# Copyright © Audiveris 2026. Rust/test adaptation 2026.
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Pinned verbatim numerical methods; explicit horizontal/rectangular geometry adapters."""
import hashlib,json,struct,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];BASE=ROOT/'upstream/app/src/main/java/org/audiveris/omr';WORK=ROOT/'evidence/generated/java-filaments';WORK.mkdir(parents=True,exist_ok=True)
PIN='9e1e55cd2746037d059345881c53e6a6754bffbd'
assert subprocess.check_output(['git','-C',str(ROOT/'upstream'),'rev-parse','HEAD'],text=True).strip()==PIN
hashes={}
def method(file,signature):
 s=(BASE/(file+'.java')).read_text();i=s.index(signature);b=s.index('{',i);depth=1;e=b+1
 while depth:
  depth+=(s[e]=='{')-(s[e]=='}');e+=1
 t=s[i:e];hashes[file+'/'+signature]=hashlib.sha256(t.encode()).hexdigest();return t
line='\n'.join(method('math/BasicLine',s) for s in ['private void compute ()','private void normalize ()','public void includePoint (double x,','public void checkLineParameters ()','public double getSlope ()','public double yAtX (double x)','public int yAtX (int x)'])
fat=method('glyph/dynamic/FilamentFactory','public boolean isSectionFat')
initial=method('glyph/dynamic/FilamentFactory','private void createInitialFilaments')
combs=method('sheet/grid/ClustersRetriever','private void retrieveCombs ()')
atx=method('sheet/grid/ClustersRetriever','private List<FilY> retrieveFilamentsAtX')
source='''// Copyright © Audiveris 2026. All rights reserved.
// Verbatim methods plus test adapters; AGPL-3.0-or-later, WITHOUT ANY WARRANTY.
// See root LICENSE and pinned upstream source headers.
import java.awt.Rectangle;import java.awt.geom.Point2D;import java.io.*;import java.util.*;import static java.lang.Math.*;
public class FilamentOracle {
static class Log {void trace(Object...a){}void info(Object...a){}void warn(Object...a){}void debug(Object...a){}boolean isDebugEnabled(){return false;}}
static final Log logger=new Log();static final Orientation HORIZONTAL=new Orientation();static final Orientation orientation=HORIZONTAL;
static class Orientation {Rectangle oriented(Rectangle r){return r;}Rectangle absolute(Rectangle r){return r;}Line switchRef(Line l){return l;}}
static class UndefinedLineException extends RuntimeException{UndefinedLineException(String s){super(s);}}
static class NonInvertibleLineException extends RuntimeException{NonInvertibleLineException(String s){super(s);}}
static class Line {boolean dirty=true,isRatherVertical;int n;double sx,sy,sx2,sy2,sxy,a,b,c;double xMin=Double.MAX_VALUE,xMax=-Double.MAX_VALUE,yMin=Double.MAX_VALUE,yMax=-Double.MAX_VALUE;
'''+line+'''
}
static class Section {int id,firstPos,weight;int[][]runs;Rectangle bounds;
boolean isVip(){return false;}int getLength(Orientation o){return bounds.width;}double getMeanThickness(Orientation o){return (double)weight/bounds.width;}double getMeanAspect(Orientation o){return bounds.width/getMeanThickness(o);}Rectangle getBounds(){return new Rectangle(bounds);}
Line getAbsoluteLine(){Line l=new Line();int y=firstPos;for(int[]r:runs){for(int x=r[0];x<r[0]+r[1];x++)l.includePoint(x,y);y++;}return l;}
void cumulate(PointsCollector pc){int y=firstPos;for(int[]r:runs){for(int x=r[0];x<r[0]+r[1];x++)if(pc.roi.contains(x,y))pc.size++;y++;}}
boolean rectangle(){for(int[]r:runs)if(!Arrays.equals(r,runs[0]))return false;return true;}}
static class PointsCollector {Rectangle roi;int size;PointsCollector(Rectangle r){roi=new Rectangle(r);}int getSize(){return size;}}
static class Params {int minCoreSectionLength,maxThickness,probeWidth,samplingDx,combMinMargin,combMaxMargin;double minSectionAspect=3;}
static class Index{boolean isVipId(int i){return false;}}
static class StaffFilament {Section section;StaffFilament(Section s){section=s;}boolean isVip(){return false;}void setVip(boolean b){}int getId(){return section.id;}Point2D getStartPoint(){return new Point2D.Double(section.bounds.x,y());}Point2D getStopPoint(){return new Point2D.Double(section.bounds.x+section.bounds.width-1,y());}double y(){return section.firstPos+(section.runs.length-1)/2.;}double getPositionAt(double x,Orientation o){return y();}}
static class Factory<F extends StaffFilament>{Params params;Factory(Params p){params=p;}Index index=new Index();Map<Section,Boolean>fatSections=new HashMap<>();boolean setFat(Section s,boolean b){fatSections.put(s,b);return b;}@SuppressWarnings("unchecked") F createFilament(Section s){return (F)new StaffFilament(s);}
'''+fat+'\n'+initial+'''
}
static class FilamentComb{int col;List<StaffFilament>fs=new ArrayList<>();List<Double>ys=new ArrayList<>();FilamentComb(int c){col=c;}void append(StaffFilament f,double y){fs.add(f);ys.add(y);}}
static class FilY{StaffFilament filament;double y;FilY(StaffFilament f,double v){filament=f;y=v;}static Comparator<FilY>byOrdinate=Comparator.comparingDouble(f->f.y);}
static class Range{int min,max;Range(int a,int b){min=a;max=b;}}
static class Clusters{int pictureWidth;Params params;Range interlineScale;int[]colX;Map<Integer,List<FilamentComb>>colCombs=new TreeMap<>();List<StaffFilament>allFilaments;
'''+combs+'\n'+atx+'''
}
static String ids(List<StaffFilament> fs){return fs.stream().map(f->""+f.getId()).toList().toString();}
public static void main(String[]args)throws Exception{DataInputStream in=new DataInputStream(System.in);int width=in.readInt(),min=in.readInt(),inter=in.readInt(),max=in.readInt(),height=in.readInt(),n=in.readInt();Params p=new Params();p.minCoreSectionLength=(int)rint(inter*.5);p.maxThickness=(int)rint(height*1.5);p.probeWidth=(int)rint(inter*.5);p.samplingDx=inter;p.combMinMargin=p.combMaxMargin=(int)rint(inter*.05);List<Section>sections=new ArrayList<>();for(int i=0;i<n;i++){Section s=new Section();s.id=in.readInt();s.firstPos=in.readInt();int nr=in.readInt();s.runs=new int[nr][2];int lo=Integer.MAX_VALUE,hi=0;for(int j=0;j<nr;j++){int a=in.readInt(),b=in.readInt();s.runs[j]=new int[]{a,b};s.weight+=b;lo=min(lo,a);hi=max(hi,a+b);}s.bounds=new Rectangle(lo,s.firstPos,hi-lo,nr);sections.add(s);}Factory<StaffFilament>f=new Factory<>(p);List<StaffFilament>seeds=new ArrayList<>();f.createInitialFilaments(seeds,sections);List<String>decisions=new ArrayList<>(),seedJson=new ArrayList<>();for(Section s:sections){String reason=s.getLength(HORIZONTAL)<p.minCoreSectionLength?"too_short":f.isSectionFat(s)?"too_fat":"seed";decisions.add("{\\"section_id\\":"+s.id+",\\"decision\\":\\""+reason+"\\"}");}
for(StaffFilament seed:seeds){Section s=seed.section;Line l=s.getAbsoluteLine();l.checkLineParameters();Rectangle b=s.bounds;seedJson.add("{\\"section_id\\":"+s.id+",\\"bounds\\":["+b.x+","+b.y+","+b.width+","+b.height+"],\\"line_abc\\":"+Arrays.toString(new double[]{l.a,l.b,l.c})+",\\"comb_eligible\\":"+s.rectangle()+"}");}
Clusters c=new Clusters();c.pictureWidth=width;c.params=p;c.interlineScale=new Range(min,max);c.allFilaments=seeds.stream().filter(x->x.section.rectangle()).toList();c.retrieveCombs();List<String>cs=new ArrayList<>();for(var entry:c.colCombs.entrySet())for(FilamentComb comb:entry.getValue())cs.add("{\\"column\\":"+comb.col+",\\"x\\":"+c.colX[comb.col]+",\\"section_ids\\":"+ids(comb.fs)+",\\"ys\\":"+comb.ys+"}");System.out.println("{\\"decisions\\":["+String.join(",",decisions)+"],\\"seeds\\":["+String.join(",",seedJson)+"],\\"combs\\":["+String.join(",",cs)+"]}");}
}
'''
(WORK/'FilamentOracle.java').write_text(source)
jdk=Path(sys.argv[1]);subprocess.run([str(jdk/'bin/javac'),'-d',str(WORK),str(WORK/'FilamentOracle.java')],check=True)
summary={'upstream_commit':PIN,'scope':'verbatim initial seed selection/fatness/BasicLine numerical methods and comb sampling; horizontal Section adapter; comb input restricted to constant rectangular unmerged seeds, NOT full StaffFilament/LineCluster pipeline','method_sha256':hashes,'fixtures':[]};goldens=[]
for fixture in sorted((ROOT/'fixtures/sections').glob('*.pgm'))+list((ROOT/'fixtures/rendered').glob('*.pgm')):
 out=WORK/(fixture.stem+'.pgm');report=WORK/(fixture.stem+'.json')
 for q in [out,report]:q.unlink(missing_ok=True)
 run=subprocess.run([str(ROOT/'target/debug/audiveris-native'),'filament-seeds',str(fixture),str(out),str(report)],capture_output=True,text=True);assert run.returncode in [0,2],run.stderr
 native=json.loads(report.read_text());sc=native['scale'];grid=native['grid_preparation']
 if 'filament_seeds' not in native:summary['fixtures'].append({'file':str(fixture.relative_to(ROOT)),'scale_blocked':True});continue
 # Root scale JSON uses array ranges; no inferred parameters.
 sections=grid['horizontal_sections'];ranges=sc['selected']
 il=ranges['interline'];lh=ranges['line'];header=[native['width'],*il,lh[1],len(sections)]
 data=struct.pack('>'+'i'*len(header),*header)
 for s in sections:
  vals=[s['id'],s['first_pos'],len(s['runs'])]+[x for r in s['runs'] for x in r];data+=struct.pack('>'+'i'*len(vals),*vals)
 java=json.loads(subprocess.check_output([str(jdk/'bin/java'),'-Xmx128m','-Djava.awt.headless=true','-cp',str(WORK),'FilamentOracle'],input=data))
 got=native['filament_seeds'];assert got['decisions']==java['decisions'];assert got['combs']==java['combs'];assert len(got['seeds'])==len(java['seeds']);err=0.
 for a,b in zip(got['seeds'],java['seeds']):
  for key in ['section_id','bounds','comb_eligible']:assert a[key]==b[key]
  for x,y in zip(a['line_abc'],b['line_abc']):err=max(err,abs(x-y));assert abs(x-y)<=1e-10+1e-12*abs(y)
 entry={'file':str(fixture.relative_to(ROOT)),'sections':len(sections),'seeds':len(java['seeds']),'rectangular_seeds':sum(s['comb_eligible'] for s in java['seeds']),'combs':len(java['combs']),'diagnostic_five_line_groups':len(got['five_line_seed_groups']),'decisions_combs_exact':True,'max_line_coefficient_error':err};summary['fixtures'].append(entry);print(entry)
 goldens.append({'width':native['width'],'interline':il,'line':lh,'sections':sections,'java':java})
(ROOT/'evidence/filament-parity.json').write_text(json.dumps(summary,indent=2)+'\n');(ROOT/'fixtures/filament-golden.json').write_text(json.dumps(goldens,separators=(',',':'))+'\n')
