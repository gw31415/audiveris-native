#!/usr/bin/env python3
# Copyright © Audiveris 2026; test adaptation 2026. SPDX-License-Identifier: AGPL-3.0-or-later
import json,hashlib,subprocess,struct,sys,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];BASE=ROOT/'upstream/app/src/main/java/org/audiveris/omr';WORK=ROOT/'evidence/generated/java-clusters';WORK.mkdir(parents=True,exist_ok=True);hashes={}
assert subprocess.check_output(['git','-C',str(ROOT/'upstream'),'rev-parse','HEAD'],text=True).strip()=='9e1e55cd2746037d059345881c53e6a6754bffbd'
# Refuse dirty Java sources even when HEAD still matches the pin.
subprocess.run(['git','-C',str(ROOT/'upstream'),'diff','--exit-code','HEAD','--','app/src/main/java'],check=True,stdout=subprocess.DEVNULL)
def extract(s,sig):
 a=s.index(sig);b=s.index('{',a);d=1;e=b+1
 while d:d+=(s[e]=='{')-(s[e]=='}');e+=1
 return s[a:e]
def method(f,sig):
 t=extract((BASE/(f+'.java')).read_text(),sig);hashes[f+'/'+sig]=hashlib.sha256(t.encode()).hexdigest();return t
source=(ROOT/'evidence/generated/java-curved/CurvedOracle.java').read_text().replace('public class CurvedOracle','public class ClusterOracle')
source=source.replace('public class ClusterOracle {','public class ClusterOracle {static class Logger extends Log{}static class LoggerFactory{static Logger getLogger(Class<?>c){return new Logger();}}')
source=source.replace('import java.awt.Rectangle;','import java.awt.Rectangle;import java.awt.Point;import java.util.Map.Entry;')
# Infrastructure extensions; existing numerical methods stay untouched.
source=source.replace('List<Section>members=new ArrayList<>();','SortedSet<Section>members=new TreeSet<>(Comparator.comparingInt((Section s)->s.bounds.x).thenComparingInt(s->s.bounds.y).thenComparingInt(s->s.id));').replace('List<Section>getMembers()','SortedSet<Section>getMembers()').replace('members.get(0)','members.first()')
sort=extract(source,'void sortMembers()');source=source.replace(sort,'void sortMembers(){}')
source=source.replace('int getLength(Orientation o){return bounds.width;}','int getStartCoord(){return bounds.x;}int getStopCoord(){return bounds.x+bounds.width-1;}int getLength(Orientation o){return bounds.width;}')
source=source.replace('SectionCompound getPartOf(){return partOf;}','SectionCompound getPartOf(){return partOf;}SectionCompound getAncestor(){SectionCompound f=this;while(f.partOf!=null)f=f.partOf;return f;}')
source=source.replace('static class Filament extends SectionCompound{','static class Filament extends SectionCompound{'+method('glyph/dynamic/Filament','public int getTrueLength ()'))
source=source.replace('int il;Scale(int v){il=v;}','int il,maxFore;int getMaxFore(){return maxFore;}Scale(int v){il=v;}')
source=source.replace('static class InterlineScale{','static class InterlineScale{int min,main,max;InterlineScale(int a,int b,int c){min=a;main=b;max=c;}')
old=extract(source,'static class StaffFilament extends Filament')
staff='''static class StaffFilament extends Filament{SortedMap<Integer,FilamentComb>combs;LineCluster cluster;int clusterPos;StaffFilament(int il,Section s,int i){super(il,s,i);}void setCluster(LineCluster c,int p){cluster=c;clusterPos=p;}double yAt(double x){return getPositionAt(x,HORIZONTAL);}'''+''.join(method('sheet/grid/StaffFilament',sig)for sig in ['public void addComb','public SortedMap<Integer, FilamentComb> getCombs','public LineCluster getCluster','public int getClusterPos','public void include (StaffFilament that)','public boolean isWithinRange'])+'}'
source=source.replace(old,staff)
# Class bodies retain original algorithms; package/type scoping and audit-id initialization only.
for name in ['FilamentComb','LineCluster']:
 raw=(BASE/f'sheet/grid/{name}.java').read_text();body=extract(raw,'public class '+name).replace('public class '+name,'static class '+name,1);hashes['class_body/'+name]=hashlib.sha256(extract(raw,'public class '+name).encode()).hexdigest()
 if name=='FilamentComb':source=source.replace(extract(source,'static class FilamentComb'),body)
 else:
  body=body.replace('private final String id;','private final String id;int auditId;{auditId=created.size();created.add(this);}')
  source=source.replace('public static void main(', 'static List<LineCluster>created=new ArrayList<>();interface Vip{boolean isVip();void setVip(boolean b);}\n'+body+'\npublic static void main(',1)
geo=extract(source,'static class GeoUtil');source=source.replace(geo,geo[:-1]+method('math/GeoUtil','public static int xOverlap')+method('math/GeoUtil','public static int xGap')+'}')
old=extract(source,'static class Ratio');source=source.replace(old,'static class Ratio{double v=1.7;Ratio(){}Ratio(double x){v=x;}double getValue(){return v;}}')
source=source.replace('Ratio maxConsistentRatio=new Ratio();','Ratio maxConsistentRatio=new Ratio(),minClusterLengthRatio=new Ratio(.2),minClusterTablatureLengthRatio=new Ratio(.5);')
source=source.replace('int samplingDx,combMinMargin,combMaxMargin;','int samplingDx,combMinMargin,combMaxMargin,maxExpandDx,maxExtrapolationDx,maxExpandDy,maxMergeDx,maxMergeDxSingle,maxMergeDy,maxMergeCenterDy,clusterYMargin;')
source=source.replace('Sheet getSkew(){return this;}','Sheet getSkew(){return this;}Point2D deskewed(Point2D p){double c=1/Math.sqrt(1+slope*slope);return new Point2D.Double((p.getX()+slope*p.getY())*c,(-slope*p.getX()+p.getY())*c);}')
old=extract(source,'static class Clusters');extra='''Scale scale;Sheet sheet;Sheet skew;List<LineCluster>allClusters=new ArrayList<>();TreeSet<Integer>combSizes=new TreeSet<>(Arrays.asList(5));List<StaffFilament>discardedFilaments=new ArrayList<>();Comparator<LineCluster>byOrdinate=(a,b)->Double.compare(ordinateOf(a),ordinateOf(b));static Comparator<StaffFilament>byStartAbscissa=(a,b)->Double.compare(a.getStartPoint().getX(),b.getStartPoint().getX());static Comparator<StaffFilament>byStopAbscissa=(a,b)->Double.compare(a.getStopPoint().getX(),b.getStopPoint().getX());'''
extra+='\n'.join(method('sheet/grid/ClustersRetriever',sig)for sig in ['private void followCombsNetwork','private void connectAncestors','private List<LineCluster> createClusters','private void expandCluster (','private void expandClusters (','private double bestMatch','private boolean canMerge','private boolean checkCollision','private void mergeClusters','private void trimClusters','private void destroyNonDesiredClusters','private void mergeClusterPairs','private double computeAcceptableLength','private void discardNonClusteredFilaments','private void removeMergedClusters','private void removeMergedFilaments','private double ordinateOf (LineCluster','private Double ordinateOf (Point2D','private Double[] ordinatesOf'])
source=source.replace(old,old[:-1]+extra+'}')
source=source.replace('Range interlineScale;','InterlineScale interlineScale;').replace('new Range(ilmin,ilmax)','new InterlineScale(ilmin,il,ilmax)')
# Replace previous harness main with section-input setup followed by stage-specific checkpoints.
main=extract(source,'public static void main(');cut=main.index('List<String>filaments=');setup=main[:cut]
setup+='''Lines lines=new Lines();lines.filaments=new ArrayList<>(fs);lines.params.minLengthForSlopeCheck=il*4;lines.purgeCurvedFilaments();if(lines.filaments.size()<5){System.out.println("{\\"status\\":\\"blocked\\"}");return;}lines.sheet.slope=lines.retrieveGlobalSlope();lines.purgeSlopedFilaments();sc.maxFore=MAX_FORE;p.maxExpandDx=sc.toPixels(2);p.maxExtrapolationDx=sc.toPixels(6);p.maxExpandDy=sc.toPixels(.175);p.maxMergeDx=sc.toPixels(6);p.maxMergeDxSingle=sc.toPixels(10);p.maxMergeDy=sc.toPixels(.4);p.maxMergeCenterDy=sc.toPixels(1);p.clusterYMargin=sc.toPixels(2);Clusters c=new Clusters();c.pictureWidth=width;c.params=p;c.interlineScale=new InterlineScale(ilmin,il,ilmax);c.allFilaments=lines.filaments;c.scale=sc;c.sheet=c.skew=lines.sheet;c.retrieveCombs();List<Integer>input=c.allFilaments.stream().map(f->f.id).toList();events.clear();phase="network";c.followCombsNetwork();List<Integer>network=c.allFilaments.stream().map(f->f.id).toList();c.allClusters.addAll(c.createClusters(c.allFilaments,false));List<String>stages=new ArrayList<>();stages.add(snapshot(c.allClusters,"create"));c.expandClusters(c.allClusters,c.allFilaments);stages.add(snapshot(c.allClusters,"expand_1"));c.mergeClusters(c.allClusters,c.allFilaments);stages.add(snapshot(c.allClusters,"merge"));c.trimClusters();stages.add(snapshot(c.allClusters,"trim"));c.destroyNonDesiredClusters();stages.add(snapshot(c.allClusters,"desired_size"));c.mergeClusterPairs(c.allClusters,c.allFilaments);stages.add(snapshot(c.allClusters,"pair_and_length"));c.expandClusters(c.allClusters,c.allFilaments);stages.add(snapshot(c.allClusters,"expand_2"));c.discardNonClusteredFilaments();c.removeMergedFilaments(c.allFilaments);System.out.println("{\\"input_line_ids\\":"+input+",\\"network_line_ids\\":"+network+",\\"remaining_line_ids\\":"+c.allFilaments.stream().map(f->f.id).toList()+",\\"discarded_line_ids\\":"+c.discardedFilaments.stream().map(f->f.id).toList()+",\\"phases\\":["+String.join(",",stages)+"]}");}'''
# Add maxFore as explicit separate header field; preserve line main for factory setup.
setup=setup.replace('int width=in.readInt(),','int MAX_FORE=in.readInt();int width=in.readInt(),')
source=source.replace(main,setup)
source=source.replace('public static void main(','''static class Wrapper<T>{T value;Wrapper(T v){value=v;}}
static String snapshot(List<LineCluster>clusters,String phase){List<String>out=new ArrayList<>();for(LineCluster c:clusters){Rectangle savedBox=c.contourBox;Integer savedLength=c.trueLength;Rectangle b=c.getBounds();List<String>ls=new ArrayList<>();for(var e:c.lines.entrySet()){StaffFilament f=e.getValue();ls.add("{\\"pos\\":"+e.getKey()+",\\"filament\\":"+f.id+",\\"sections\\":"+f.members.stream().map(s->s.id).toList()+",\\"true_length\\":"+f.getTrueLength()+"}");}out.add("{\\"id\\":"+c.auditId+",\\"seed\\":"+c.id.substring(1)+",\\"lines\\":["+String.join(",",ls)+"],\\"bounds\\":["+b.x+","+b.y+","+b.width+","+b.height+"],\\"true_length\\":"+c.getTrueLength()+"}");c.contourBox=savedBox;c.trueLength=savedLength;}return "{\\"phase\\":\\""+phase+"\\",\\"clusters\\":["+String.join(",",out)+"]}";}
public static void main(''',1)

# Extend with verbatim default buildStaves and staff-manager short-neighbor logic.
source=source.replace('import java.awt.Point;','import java.awt.Point;import java.util.stream.Collectors;')
source=source.replace('static class StaffFilament extends Filament{','static class StaffFilament extends Filament implements LineInfo{public Point2D getEndPoint(HorizontalSide side){return side==LEFT?getStartPoint():getStopPoint();}')
source=source.replace('int getMaxFore(){return maxFore;}','Integer getFore(){return 2;}int getMaxFore(){return maxFore;}')
# Skew is only a type facade around the existing tested deskew transform.
source=source.replace('static class Sheet{','static class Skew{double slope;Point2D deskewed(Point2D p){double c=1/Math.sqrt(1+slope*slope);return new Point2D.Double((p.getX()+slope*p.getY())*c,(-slope*p.getX()+p.getY())*c);}}static class Sheet extends Skew{')
old=extract(source,'static class Lines');extra='Clusters clustersRetriever,smallClustersRetriever;Manager staffManager=new Manager();Scale scale;double globalSlope;'
extra+='\n'.join(method('sheet/grid/LinesRetriever',sig)for sig in ['private void buildStaves ()','private void purgeClusters (','private void purgeOneLineClustersRightIndented ('])
source=source.replace(old,old[:-1]+extra+'}')
source=source.replace('int minLengthForSlopeCheck;','int minLengthForSlopeCheck,minStaffLength,maxRightIndentation;double maxOneLineSlope=.001,minTrueLengthRatio=.3;')
old=extract(source,'static class Clusters');extra='List<LineCluster>getClusters(){return allClusters;}int getInterline(){return interlineScale.main;}'
extra+='\n'.join(method('sheet/grid/ClustersRetriever',sig)for sig in ['public static LineCluster getClusterBelow','public static List<LineCluster> getHalfClusters'])
extra+='Comparator<LineCluster>byLayout=(a,b)->{Point p=a.getCenter(),q=b.getCenter();return GeoUtil.xOverlap(a.getBounds(),b.getBounds())<0?Integer.compare(p.x,q.x):Double.compare(ordinateOf(p),ordinateOf(q));};'
source=source.replace(old,old[:-1]+extra+'}')
support='''enum HorizontalSide{LEFT,RIGHT}static final HorizontalSide LEFT=HorizontalSide.LEFT,RIGHT=HorizontalSide.RIGHT;
interface LineInfo{Point2D getEndPoint(HorizontalSide side);}
static class Staff{int id,interline;double left,right;List<LineInfo>lines;boolean small,shortStaff;Staff(int i,double l,double r,int il,List<LineInfo>f){id=i;left=l;right=r;interline=il;lines=f;}LineInfo getFirstLine(){return lines.get(0);}LineInfo getLastLine(){return lines.get(lines.size()-1);}void setSmall(){small=true;}void setShort(){shortStaff=true;}
'''+method('sheet/Staff','public boolean yOverlaps (Staff that)')+'''}
static class OneLineStaff extends Staff{OneLineStaff(int i,double l,double r,int il,List<LineInfo>f){super(i,l,r,il,f);}}
static class Tablature extends Staff{Tablature(int i,double l,double r,int il,List<LineInfo>f){super(i,l,r,il,f);}}
static class Manager{List<Staff>staves=new ArrayList<>();void reset(){staves.clear();}void addStaff(Staff s){staves.add(s);}int getIndexOf(Staff s){return staves.indexOf(s);}Staff getStaff(int i){return staves.get(i);}
'''+method('sheet/StaffManager','public void detectShortStaves ()')+method('sheet/StaffManager','public Staff horiNeighbor (Staff current,')+'}'
source=source.replace('public static void main(',support+'\npublic static void main(',1)
anchor='c.removeMergedFilaments(c.allFilaments);System.out.println('
assert anchor in source
code='''c.removeMergedFilaments(c.allFilaments);lines.clustersRetriever=c;lines.scale=sc;lines.globalSlope=lines.sheet.slope;lines.params.minStaffLength=sc.toPixels(30);lines.params.maxRightIndentation=sc.toPixels(5);lines.buildStaves();List<String>staffJson=new ArrayList<>();List<LineCluster>built=new ArrayList<>();for(Staff staff:lines.staffManager.staves){List<String>ls=new ArrayList<>();List<Integer>ids=new ArrayList<>();LineCluster cluster=((StaffFilament)staff.lines.get(0)).cluster;built.add(cluster);for(LineInfo info:staff.lines){StaffFilament f=(StaffFilament)info;ids.add(f.id);List<String>pts=new ArrayList<>();for(Point2D pt:f.points)pts.add("["+pt.getX()+","+pt.getY()+"]");ls.add("{\\"filament\\":"+f.id+",\\"section_ids\\":"+f.members.stream().map(a->a.id).toList()+",\\"points\\":["+String.join(",",pts)+"]}");}staffJson.add("{\\"id\\":"+staff.id+",\\"cluster\\":"+cluster.auditId+",\\"interline\\":"+staff.interline+",\\"left\\":"+staff.left+",\\"right\\":"+staff.right+",\\"small\\":"+staff.small+",\\"short\\":"+staff.shortStaff+",\\"line_ids\\":"+ids+",\\"lines\\":["+String.join(",",ls)+"]}");}stages.add(snapshot(built,"build_staves_length_layout"));System.out.println('''
source=source.replace(anchor,code)
source=source.replace('+String.join(",",stages)+"]}");}', '+String.join(",",stages)+"],\\"staffs\\":["+String.join(",",staffJson)+"]}");}')

source=source.replace('ClustersRetriever.getHalfClusters', 'Clusters.getHalfClusters').replace('ClustersRetriever.getClusterBelow', 'Clusters.getClusterBelow')
(WORK/'ClusterOracle.java').write_text(source)
for p in (ROOT/'evidence/generated/java-curved/org').rglob('*.java'):
 q=WORK/'org'/p.relative_to(ROOT/'evidence/generated/java-curved/org');q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q)
jdk=Path(sys.argv[1]);subprocess.run([str(jdk/'bin/javac'),'-d',str(WORK/'classes')]+[str(p)for p in WORK.rglob('*.java')],check=True)
report={'upstream_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','method_sha256':hashes,'harness_sha256':hashlib.sha256(source.encode()).hexdigest(),'independence':'native scale and sections into independent Java filament/cluster/Staff stage; not whole-image app parity','fixtures':[]};goldens=[]
for fixture in sorted((ROOT/'fixtures/sections').glob('*.pgm'))+list((ROOT/'fixtures/rendered').glob('*.pgm'))+sorted((ROOT/'fixtures/staff').glob('*.pgm')):
 out=WORK/(fixture.stem+'.pgm');path=WORK/(fixture.stem+'.json')
 for p in [out,path]:p.unlink(missing_ok=True)
 r=subprocess.run([str(ROOT/'target/debug/audiveris-native'),'staff-candidates',str(fixture),str(out),str(path)],capture_output=True,text=True);assert r.returncode in[0,2],r.stderr
 native=json.loads(path.read_text());sc=native['scale']['selected']
 if sc is None:continue
 ss=native['grid_preparation']['horizontal_sections'];il=sc['interline'];lh=sc['line'];vals=[lh[2],native['width'],*il,lh[1],len(ss)];data=struct.pack('>'+'i'*len(vals),*vals)
 for s in ss:
  vals=[s['id'],s['first_pos'],len(s['runs'])]+[x for r in s['runs']for x in r];data+=struct.pack('>'+'i'*len(vals),*vals)
 java=json.loads(subprocess.check_output([str(jdk/'bin/java'),'-Xmx256m','-Djava.awt.headless=true','-cp',str(WORK/'classes'),'ClusterOracle'],input=data));got=native['staff_candidates']
 if java.get('status')=='blocked':assert got['status']=='blocked';continue
 def compare(a,b,where=''):
  if isinstance(b,float):assert abs(a-b)<=1e-9+1e-12*abs(b),(where,a,b)
  elif isinstance(b,dict):
   for k,v in b.items():compare(a[k],v,where+'/'+k)
  elif isinstance(b,list):
   assert len(a)==len(b),(where,len(a),len(b))
   for i,(x,y)in enumerate(zip(a,b)):compare(x,y,where+'/'+str(i))
  else:assert a==b,(where,a,b)
 try:compare(got,java)
 except Exception:
  (WORK/'mismatch.json').write_text(json.dumps({'native':got,'java':java},indent=2));raise
 e={'file':str(fixture.relative_to(ROOT)),'input_lines':len(java['input_line_ids']),'after_network':len(java['network_line_ids']),'cluster_counts':[(v['phase'],len(v['clusters']))for v in java['phases']],'stages_exact':True,'staffs':len(java['staffs'])};print(e);report['fixtures'].append(e);goldens.append({'scale':sc,'sections':ss,'factory':native['filaments'],'java':java})
(ROOT/'evidence/cluster-parity.json').write_text(json.dumps(report,indent=2)+'\n');(ROOT/'fixtures/cluster-golden.json').write_text(json.dumps(goldens,separators=(',',':'))+'\n')
